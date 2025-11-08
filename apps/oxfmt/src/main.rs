use std::io::BufWriter;

use oxfmt::{CliRunResult, FormatRunner, format_command, init_miette, init_tracing};

fn main() -> CliRunResult {
    init_tracing();
    init_miette();

    // Parse command line arguments from std::env::args()
    let command = format_command().run();

    command.handle_threads();

    // stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
    // See `https://github.com/rust-lang/rust/issues/60673`.
    let mut stdout = BufWriter::new(std::io::stdout());
    FormatRunner::new(command).run(&mut stdout)
}
