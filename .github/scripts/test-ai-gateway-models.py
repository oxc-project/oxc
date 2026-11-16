"""Compare native streaming and tool use using the triage bot's CI credentials."""

import collections
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request


PROMPT = """Fix this standalone Rust function without changing its signature. It must return the absolute difference for every pair of i32 inputs, including i32::MIN and i32::MAX, without panicking. Use submit_fix to return only the corrected function source, without Markdown.

fn abs_difference(a: i32, b: i32) -> u32 {
    (a - b).abs() as u32
}

After receiving the tool result, reply exactly VERIFIED if validation passed, or FAILED otherwise. Do not call another tool.
"""
SCHEMA = {
    "type": "object",
    "properties": {"source": {"type": "string"}},
    "required": ["source"],
    "additionalProperties": False,
}
HARNESS = """
#[test]
fn boundary_values() {
    let inputs = [i32::MIN, i32::MIN + 1, -1, 0, 1, i32::MAX - 1, i32::MAX];
    for a in inputs {
        for b in inputs {
            let expected = (i64::from(a) - i64::from(b)).unsigned_abs() as u32;
            assert_eq!(abs_difference(a, b), expected, "a={a}, b={b}");
        }
    }
}

#[test]
fn deterministic_samples() {
    let mut state = 123456789_u64;
    for _ in 0..4096 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let a = (state >> 32) as i32;
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let b = (state >> 32) as i32;
        let expected = (i64::from(a) - i64::from(b)).unsigned_abs() as u32;
        assert_eq!(abs_difference(a, b), expected, "a={a}, b={b}");
    }
}
"""


def redact(text):
    for key in ("CLOUDFLARE_API_KEY", "CLOUDFLARE_ACCOUNT_ID"):
        value = os.environ.get(key)
        if value:
            text = text.replace(value, "[redacted]")
    return text


def read_events(response):
    lines = []
    for raw in response:
        line = raw.decode().rstrip("\r\n")
        if line:
            lines.append(line)
        elif lines:
            data = "\n".join(line[5:].lstrip() for line in lines if line.startswith("data:"))
            lines = []
            if data and data != "[DONE]":
                yield json.loads(data)
    if lines:
        raise RuntimeError("SSE response ended with an incomplete event")


def parse_stream(response, protocol):
    message = None
    counts = collections.Counter()
    fragments = collections.defaultdict(str)
    for event in read_events(response):
        kind = event.get("type", "unknown")
        counts[kind] += 1
        if kind in ("error", "response.failed", "response.incomplete"):
            raise RuntimeError(json.dumps(event))
        if protocol == "responses":
            if kind == "response.completed":
                message = event["response"]
        elif kind == "message_start":
            message = event["message"]
        elif kind == "content_block_start":
            message["content"].append(event["content_block"])
        elif kind == "content_block_delta":
            index = event["index"]
            block = message["content"][index]
            delta = event["delta"]
            if delta["type"] == "input_json_delta":
                fragments[index] += delta["partial_json"]
            elif delta["type"] == "text_delta":
                block["text"] += delta["text"]
            elif delta["type"] == "thinking_delta":
                block["thinking"] += delta["thinking"]
            elif delta["type"] == "signature_delta":
                block["signature"] = block.get("signature", "") + delta["signature"]
        elif kind == "content_block_stop" and event["index"] in fragments:
            message["content"][event["index"]]["input"] = json.loads(fragments[event["index"]])
        elif kind == "message_delta":
            message.update(event["delta"])
            message["usage"].update(event.get("usage", {}))
    terminal = "response.completed" if protocol == "responses" else "message_stop"
    if not message or not counts[terminal]:
        raise RuntimeError("Stream ended without a completed response")
    return message, dict(counts)


def request(model, protocol, payload):
    token = os.environ.get("CLOUDFLARE_API_KEY")
    account = os.environ.get("CLOUDFLARE_ACCOUNT_ID")
    if not token or not account:
        raise RuntimeError("The triage Cloudflare CI credentials are unavailable")
    url = f"https://api.cloudflare.com/client/v4/accounts/{account}/ai/v1/{protocol}"
    req = urllib.request.Request(
        url,
        data=json.dumps(dict(payload, model=model, stream=True)).encode(),
        headers={
            "Authorization": "Bearer " + token,
            "Content-Type": "application/json",
            "cf-aig-skip-cache": "true",
            "cf-aig-max-attempts": "1",
        },
        method="POST",
    )
    start = time.monotonic()
    try:
        with urllib.request.urlopen(req, timeout=90) as response:
            if "text/event-stream" not in response.headers.get("Content-Type", ""):
                raise RuntimeError("Expected an SSE response: " + response.read().decode()[:1000])
            message, events = parse_stream(response, protocol)
            return message, {
                "passed": True,
                "http_status": response.status,
                "seconds": round(time.monotonic() - start, 2),
                "returned_model": message.get("model"),
                "usage": message.get("usage"),
                "stream_events": events,
            }
    except urllib.error.HTTPError as error:
        raise RuntimeError(f"HTTP {error.code}: {error.read().decode()[:2000]}") from None


def base_payload(protocol):
    if protocol == "responses":
        return {
            "max_output_tokens": 2048,
            "reasoning": {"effort": "low"},
            "store": False,
            "include": ["reasoning.encrypted_content"],
            "tools": [{"type": "function", "name": "submit_fix", "description": "Return a corrected Rust function.", "parameters": SCHEMA, "strict": True}],
        }
    return {
        "max_tokens": 2048,
        "tools": [{"name": "submit_fix", "description": "Return a corrected Rust function.", "input_schema": SCHEMA}],
    }


def generate(model, protocol, root):
    payload = base_payload(protocol)
    user = {"role": "user", "content": PROMPT}
    payload["input" if protocol == "responses" else "messages"] = [user]
    message, result = request(model, protocol, payload)
    blocks = message["output" if protocol == "responses" else "content"]
    calls = [block for block in blocks if block["type"] == ("function_call" if protocol == "responses" else "tool_use")]
    if len(calls) != 1 or calls[0]["name"] != "submit_fix":
        raise RuntimeError("Expected exactly one streamed submit_fix tool call")
    call = calls[0]
    arguments = json.loads(call["arguments"]) if protocol == "responses" else call["input"]
    source = arguments["source"].strip()
    source = re.sub(r"^```(?:rust)?\s*\n|\n```$", "", source).strip()
    if "fn abs_difference(a: i32, b: i32) -> u32" not in source:
        raise RuntimeError("Model changed the required function signature")
    (root / "artifacts/fix.rs").write_text(source + "\n")
    (root / "context.json").write_text(json.dumps({"user": user, "message": message, "call": call}))
    return result


def validate(root):
    source = root / "artifacts/fix.rs"
    if not source.exists():
        return {"passed": False, "error": "No generated fix available"}
    harness = root / "artifacts/test.rs"
    harness.write_text(source.read_text() + HARNESS)
    results = {}
    # No inference credentials are passed to the compiler or generated program.
    env = {key: value for key, value in os.environ.items() if key in ("PATH", "HOME", "RUSTUP_HOME", "CARGO_HOME", "TMPDIR")}
    for profile, flags in (("debug", []), ("release", ["-O"])):
        binary = root / ("test-" + profile)
        compile_result = subprocess.run(["rustc", "--edition=2024", "--test", str(harness), "-o", str(binary), *flags], cwd=root, env=env, capture_output=True, text=True, timeout=30)
        output = compile_result.stdout + compile_result.stderr
        passed = compile_result.returncode == 0
        if passed:
            run = subprocess.run([str(binary)], cwd=root, env=env, capture_output=True, text=True, timeout=10)
            output += run.stdout + run.stderr
            passed = run.returncode == 0
        (root / f"artifacts/{profile}.log").write_text(output)
        results[profile] = passed
    return {"passed": all(results.values()), "profiles": results, "cases_per_profile": 4145}


def verify(model, protocol, root, report):
    if not report.get("generate", {}).get("passed"):
        return {"passed": False, "error": "Generation failed; skipped further inference"}
    context = json.loads((root / "context.json").read_text())
    payload = base_payload(protocol)
    output = json.dumps(report.get("validate", {"passed": False}))
    if protocol == "responses":
        payload["input"] = [context["user"], *context["message"]["output"], {"type": "function_call_output", "call_id": context["call"]["call_id"], "output": output}]
        payload["tool_choice"] = "none"
    else:
        payload["messages"] = [context["user"], {"role": "assistant", "content": context["message"]["content"]}, {"role": "user", "content": [{"type": "tool_result", "tool_use_id": context["call"]["id"], "content": output}]}]
    message, result = request(model, protocol, payload)
    if protocol == "responses":
        text = "".join(part.get("text", "") for item in message["output"] if item["type"] == "message" for part in item["content"] if part["type"] == "output_text")
    else:
        text = "".join(part.get("text", "") for part in message["content"] if part["type"] == "text")
    expected = "VERIFIED" if report.get("validate", {}).get("passed") else "FAILED"
    result.update(passed=text.strip() == expected, text=text)
    return result


def main():
    stage = sys.argv[1]
    model = os.environ["MODEL"]
    protocol = {"openai/gpt-6.1-sol": "responses", "anthropic/claude-opus-5.5": "messages"}[model]
    root = Path(os.environ.get("RESULTS_DIR") or str(Path(os.environ["RUNNER_TEMP"]) / "gateway-model-test"))
    (root / "artifacts").mkdir(parents=True, exist_ok=True)
    path = root / "artifacts/results.json"
    report = json.loads(path.read_text()) if path.exists() else {"model": model, "protocol": protocol}
    if stage == "summary":
        rows = [f"### {model}", "", "| Check | Result |", "| --- | --- |"]
        for name in ("generate", "validate", "verify"):
            result = report.get(name, {})
            detail = "PASS" if result.get("passed") else result.get("error", "FAIL")
            rows.append(f"| {name} | {detail.replace('|', '/').replace(chr(10), ' ')} |")
        rows.extend(["", "Synthetic Rust fix; 49 boundary pairs and 4096 deterministic pairs in debug and release. This is an API/tool smoke test, not an Oxc quality benchmark.", ""])
        summary = "\n".join(rows)
        print(summary)
        with open(os.environ.get("GITHUB_STEP_SUMMARY", root / "summary.md"), "a") as output:
            output.write(summary)
        return all(report.get(name, {}).get("passed") for name in ("generate", "validate", "verify"))
    try:
        if stage == "generate":
            result = generate(model, protocol, root)
        elif stage == "validate":
            result = validate(root)
        elif stage == "verify":
            result = verify(model, protocol, root, report)
        else:
            raise RuntimeError("Unknown stage")
    except Exception as error:
        result = {"passed": False, "error": redact(str(error))}
    report[stage] = result
    path.write_text(redact(json.dumps(report, indent=2)) + "\n")
    print(redact(json.dumps({"model": model, "stage": stage, **result})))
    return result.get("passed", False)


if __name__ == "__main__":
    sys.exit(0 if main() else 1)
