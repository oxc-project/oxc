use rustc_hash::FxBuildHasher;

mod backend;
mod capabilities;
mod code_actions;
mod commands;
mod file_system;
mod formatter;
mod linter;
mod options;
#[cfg(test)]
mod tester;
mod tools;
mod utils;
mod worker;

pub use crate::backend::Backend;
pub use crate::linter::server_linter::ServerLinter;
pub use crate::tools::{LintTool, ToolImplementation};
pub use crate::worker::WorkspaceWorker;

pub type ConcurrentHashMap<K, V> = papaya::HashMap<K, V, FxBuildHasher>;

pub const LINT_CONFIG_FILE: &str = ".oxlintrc.json";
pub const FORMAT_CONFIG_FILES: &[&str; 2] = &[".oxfmtrc.json", ".oxfmtrc.jsonc"];
