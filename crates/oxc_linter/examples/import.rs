//! Lint from a given path

use std::{env, path::Path, sync::mpsc};

use oxc_linter::{LintOptions, Runner};

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_linter --example import`
// or `cargo watch -x "run -p oxc_linter --example import"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name).canonicalize().unwrap();
    let (tx_error, rx_error) = mpsc::channel();
    let options = LintOptions::default();
    let runner = Runner::new(options);
    runner.run_path(path.into_boxed_path(), &tx_error);
    runner.process_diagnostics(&rx_error);
}
