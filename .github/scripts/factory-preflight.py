#!/usr/bin/env python3
"""Check CI access and Opus inference without revealing account data or secrets."""
import json
import os
import urllib.error
import urllib.request

account = os.environ["CLOUDFLARE_ACCOUNT_ID"]
token = os.environ["CLOUDFLARE_API_TOKEN"]
checks = {
    "Workers": f"accounts/{account}/workers/services",
    "Containers": f"accounts/{account}/containers/applications",
    "Workflows": f"accounts/{account}/workflows",
    "R2": f"accounts/{account}/r2/buckets",
    "AI Gateway": f"accounts/{account}/ai-gateway/gateways",
}
results = {}
for product, path in checks.items():
    request = urllib.request.Request(
        "https://api.cloudflare.com/client/v4/" + path,
        headers={"Authorization": "Bearer " + token},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            payload = json.load(response)
            status = response.status
    except urllib.error.HTTPError as error:
        status = error.code
        try:
            payload = json.load(error)
        except (ValueError, UnicodeDecodeError):
            payload = {}
    results[product] = {
        "http_status": status,
        "success": payload.get("success", False),
        "error_codes": [error.get("code") for error in (payload.get("errors") or [])],
    }
    print(product + ": " + json.dumps(results[product]))

# Match Factory's native Anthropic routing and authentication, rather than
# assuming the account-level Unified Billing probe covers named gateways.
gateway = os.environ.get("FACTORY_AI_GATEWAY_ID") or "default"
probe = urllib.request.Request(
    f"https://gateway.ai.cloudflare.com/v1/{account}/{gateway}/anthropic/v1/messages",
    method="POST",
    headers={
        "cf-aig-authorization": "Bearer " + token,
        "anthropic-version": "2023-06-01",
        "content-type": "application/json",
        "cf-aig-collect-log-payload": "false",
    },
    data=json.dumps({
        "model": "claude-opus-5-5",
        "max_tokens": 32,
        "messages": [{"role": "user", "content": "Reply with OK."}],
    }).encode(),
)
try:
    with urllib.request.urlopen(probe, timeout=60) as response:
        payload = json.load(response)
        opus_status = response.status
        opus_generated = any(block.get("text") for block in payload.get("content", []))
except urllib.error.HTTPError as error:
    opus_status = error.code
    opus_generated = False
    # Classify known configuration errors without printing a response that
    # could contain account identifiers or authentication details.
    detail = error.read().decode("utf-8", errors="replace").lower()
    for category, markers in {
        "gateway missing": ["gateway does not exist", "gateway not found", "could not find gateway"],
        "authentication": ["authentication", "authorization", "unauthorized", "forbidden", "not authorized"],
        "billing": ["credits", "balance", "billing"],
        "provider key": ["provider key", "api key"],
    }.items():
        if any(marker in detail for marker in markers):
            print("Gateway native Opus error category: " + category)
print(f"Gateway native Opus 5.5: HTTP {opus_status}, generated={opus_generated}")

with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as summary:
    summary.write("Factory preflight (no resource configuration changes)\n\n")
    summary.write("| Product | HTTP status |\n| --- | --- |\n")
    for product, result in results.items():
        summary.write(f"| {product} | {result['http_status']} |\n")
    summary.write("\nRead access does not establish deployment permissions.\n")
    summary.write(f"\nNative Gateway Opus 5.5: HTTP {opus_status}; generated={opus_generated}.\n")
