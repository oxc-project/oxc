mod code_actions;
mod commands;
mod config_walker;
mod error_with_position;
mod isolated_lint_handler;
mod options;
mod server_linter;
#[cfg(test)]
mod tester;

pub use code_actions::CODE_ACTION_KIND_SOURCE_FIX_ALL_OXC;
pub use commands::FIX_ALL_COMMAND_ID;
pub use server_linter::ServerLinterBuilder;

const LINT_CONFIG_FILE: &str = ".oxlintrc.json";
