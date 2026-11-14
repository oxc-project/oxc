//! Stands in for `tsgolint headless` in the `linter_type_aware` benchmark.
//!
//! Reads the payload oxlint writes to stdin and, for every file in it, reports [`DIAGNOSTICS_PER_FILE`]
//! diagnostics of the file's first rule, at line starts spread over the file. No type checking happens, so the
//! benchmark measures only oxlint's side: reading the messages and turning them into reported diagnostics.

use std::io::{self, BufWriter, Read, Write};

/// Enough diagnostics per file that a per-diagnostic cost in oxlint (a copy of the source text, say) dominates.
const DIAGNOSTICS_PER_FILE: usize = 200;

/// `MessageType::Diagnostic` in oxlint's `tsgolint.rs`.
const MESSAGE_TYPE_DIAGNOSTIC: u8 = 1;

fn main() {
    let mut payload = String::new();
    io::stdin().read_to_string(&mut payload).expect("read the payload from stdin");
    let payload: serde_json::Value = serde_json::from_str(&payload).expect("payload is JSON");

    let mut out = BufWriter::new(io::stdout().lock());
    for config in payload["configs"].as_array().expect("payload has configs") {
        let Some(rule) = config["rules"].as_array().and_then(|rules| rules.first()) else {
            continue;
        };
        let rule = rule["name"].as_str().expect("rule has a name");
        for path in config["file_paths"].as_array().expect("config has file paths") {
            let path = path.as_str().expect("file path is a string");
            let source = std::fs::read_to_string(path).expect("read the linted file");
            let line_starts: Vec<usize> = std::iter::once(0)
                .chain(source.match_indices('\n').map(|(i, _)| i + 1))
                .filter(|&start| start < source.len())
                .collect();
            let step = (line_starts.len() / DIAGNOSTICS_PER_FILE).max(1);
            for &pos in line_starts.iter().step_by(step).take(DIAGNOSTICS_PER_FILE) {
                let diagnostic = serde_json::json!({
                    "kind": 0,
                    "range": { "pos": pos, "end": pos + 1 },
                    "message": { "id": "benchmark", "description": "Reported by the benchmark's stand-in tsgolint.", "help": null },
                    "file_path": path,
                    "rule": rule,
                });
                let bytes = serde_json::to_vec(&diagnostic).expect("serialize the diagnostic");
                let len = u32::try_from(bytes.len()).expect("message fits in u32");
                out.write_all(&len.to_le_bytes()).expect("write to stdout");
                out.write_all(&[MESSAGE_TYPE_DIAGNOSTIC]).expect("write to stdout");
                out.write_all(&bytes).expect("write to stdout");
            }
        }
    }
    out.flush().expect("flush stdout");
}
