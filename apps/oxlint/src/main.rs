use std::io::BufWriter;

use oxlint::cli::{CliRunResult, CliRunner, init_miette, init_tracing, lint_command};

fn main() -> CliRunResult {
    init_tracing();
    init_miette();

    // Parse command line arguments from std::env::args()
    let command = lint_command().run();

    command.handle_threads();

    // stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
    // See `https://github.com/rust-lang/rust/issues/60673`.
    let mut stdout = BufWriter::new(std::io::stdout());

    // Run without external linter (no JS plugins)
    CliRunner::new(command, None).run(&mut stdout)
}
