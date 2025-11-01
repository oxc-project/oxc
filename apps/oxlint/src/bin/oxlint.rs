use oxlint::cli::{CliRunner, lint_command};
use std::io::{BufWriter, stdout};
use std::process::Termination;

fn main() {
    // Get command line arguments
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();

    // Parse command using bpaf
    let cmd = lint_command();
    let command = match cmd.run_inner(&args[1..]) {
        Ok(cmd) => cmd,
        Err(e) => {
            e.print_message(100);
            let exit_code = if e.exit_code() == 0 { 0 } else { 1 };
            std::process::exit(exit_code);
        }
    };

    command.handle_threads();

    // Create runner without external linter (no JS plugins for standalone binary)
    let runner = CliRunner::new(command, None);

    // Run linter
    let mut stdout = BufWriter::new(stdout());
    let result = runner.run(&mut stdout);

    let exit_code = match result.report() {
        std::process::ExitCode::SUCCESS => 0,
        std::process::ExitCode::FAILURE => 1,
        _ => 1,
    };
    std::process::exit(exit_code);
}
