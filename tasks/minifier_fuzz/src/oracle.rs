use std::{
    io::Write,
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

const NODE_RUNNER: &str = r#"
const vm = require("node:vm");

function encode(value, seen = new Map()) {
  if (value === undefined) return ["undefined"];
  if (value === null) return ["null"];
  if (typeof value === "number") {
    if (Number.isNaN(value)) return ["number", "NaN"];
    if (Object.is(value, -0)) return ["number", "-0"];
    if (value === Infinity) return ["number", "+Infinity"];
    if (value === -Infinity) return ["number", "-Infinity"];
    return ["number", String(value)];
  }
  if (typeof value === "string" || typeof value === "boolean") {
    return [typeof value, value];
  }
  if (typeof value === "bigint") return ["bigint", String(value)];
  if (typeof value === "symbol") return ["symbol", value.description ?? null];
  if (typeof value === "function") return ["function"];

  if (seen.has(value)) return ["reference", seen.get(value)];
  const id = seen.size;
  seen.set(value, id);
  if (Array.isArray(value)) {
    const items = [];
    for (let index = 0; index < value.length; index++) {
      items.push(Object.hasOwn(value, index) ? encode(value[index], seen) : ["hole"]);
    }
    return ["array", id, items];
  }
  const entries = Object.keys(value)
    .sort()
    .map(key => [key, encode(value[key], seen)]);
  return ["object", id, entries];
}

function execute(source, timeout) {
  const logs = [];
  const sandbox = {
    console: {
      log(...args) {
        logs.push(encode(args));
      },
    },
  };
  try {
    vm.runInNewContext(source, sandbox, { timeout, displayErrors: false });
    return { status: "completed", logs };
  } catch (error) {
    if (error && error.code === "ERR_SCRIPT_EXECUTION_TIMEOUT") {
      return { status: "timed_out" };
    }
    return {
      status: "threw",
      name: error && error.name ? String(error.name) : "Error",
      message: error && error.message ? String(error.message) : String(error),
    };
  }
}

let input = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", chunk => input += chunk);
process.stdin.on("end", () => {
  const request = JSON.parse(input);
  const results = request.cases.map(testCase => ({
    original: execute(testCase.original, request.timeout_ms),
    minified: execute(testCase.minified, request.timeout_ms),
  }));
  process.stdout.write(JSON.stringify(results));
});
"#;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Outcome {
    Completed { logs: Value },
    Threw { name: String, message: String },
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "comparison", rename_all = "snake_case")]
pub enum Comparison {
    Equivalent { original: Outcome, minified: Outcome },
    Mismatch { original: Outcome, minified: Outcome },
    Skipped { original: Outcome, minified: Outcome },
    HarnessError { message: String },
}

#[derive(Debug, Serialize)]
struct Request<'a> {
    cases: Vec<Case<'a>>,
    timeout_ms: u64,
}

#[derive(Debug, Serialize)]
struct Case<'a> {
    original: &'a str,
    minified: &'a str,
}

#[derive(Debug, Deserialize)]
struct PairOutcome {
    original: Outcome,
    minified: Outcome,
}

#[derive(Debug, Clone, Copy)]
pub struct Oracle {
    timeout_ms: u64,
}

impl Oracle {
    pub fn new(timeout_ms: u64) -> Self {
        Self { timeout_ms }
    }

    pub fn compare(self, original: &str, minified: &str) -> Comparison {
        self.compare_many(&[(original, minified)]).pop().unwrap_or_else(|| {
            Comparison::HarnessError { message: "Node.js oracle returned no result".into() }
        })
    }

    pub fn compare_many(self, cases: &[(&str, &str)]) -> Vec<Comparison> {
        let request = Request {
            cases: cases.iter().map(|(original, minified)| Case { original, minified }).collect(),
            timeout_ms: self.timeout_ms,
        };
        let input = match serde_json::to_vec(&request) {
            Ok(input) => input,
            Err(error) => return harness_errors(cases.len(), &error.to_string()),
        };

        let mut child = match Command::new("node")
            .arg("-e")
            .arg(NODE_RUNNER)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => return harness_errors(cases.len(), &error.to_string()),
        };

        if let Some(mut stdin) = child.stdin.take()
            && let Err(error) = stdin.write_all(&input)
        {
            return harness_errors(cases.len(), &error.to_string());
        }

        let output = match child.wait_with_output() {
            Ok(output) => output,
            Err(error) => return harness_errors(cases.len(), &error.to_string()),
        };
        if !output.status.success() {
            return harness_errors(cases.len(), &String::from_utf8_lossy(&output.stderr));
        }

        match serde_json::from_slice::<Vec<PairOutcome>>(&output.stdout) {
            Ok(outcomes) if outcomes.len() == cases.len() => {
                outcomes.into_iter().map(classify).collect()
            }
            Ok(outcomes) => harness_errors(
                cases.len(),
                &format!(
                    "Node.js oracle returned {} results for {} cases",
                    outcomes.len(),
                    cases.len()
                ),
            ),
            Err(error) => harness_errors(cases.len(), &error.to_string()),
        }
    }
}

fn classify(pair: PairOutcome) -> Comparison {
    if !matches!(pair.original, Outcome::Completed { .. }) {
        return Comparison::Skipped { original: pair.original, minified: pair.minified };
    }
    if pair.original == pair.minified {
        Comparison::Equivalent { original: pair.original, minified: pair.minified }
    } else {
        Comparison::Mismatch { original: pair.original, minified: pair.minified }
    }
}

fn harness_errors(count: usize, message: &str) -> Vec<Comparison> {
    vec![Comparison::HarnessError { message: message.to_owned() }; count]
}
