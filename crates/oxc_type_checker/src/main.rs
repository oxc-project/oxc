//! `oxcheck` — experimental command-line entry point for [`oxc_type_checker`].
//!
//! A thin binary mirroring typescript-go's `cmd/tsgo/main.go`: it delegates straight to
//! [`oxc_type_checker::execute::command_line`]. All logic lives in the library.

use std::process::ExitCode;

fn main() -> ExitCode {
    oxc_type_checker::execute::command_line()
}
