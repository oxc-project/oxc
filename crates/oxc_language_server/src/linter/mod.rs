mod code_actions;
mod commands;
mod config_walker;
mod error_with_position;
mod isolated_lint_handler;
mod options;
mod server_linter;
#[cfg(test)]
mod tester;

pub use server_linter::ServerLinterBuilder;

const LINT_CONFIG_FILE: &str = ".oxlintrc.json";
