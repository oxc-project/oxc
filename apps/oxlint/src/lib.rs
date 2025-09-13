/// Re-exported external linter types for use in `napi/oxlint`.
pub use oxc_linter::{
    ExternalLinter, ExternalLinterLintFileCb, ExternalLinterLoadPluginCb, LintFileResult,
    PluginLoadResult,
};

mod command;
mod js_plugins;
mod lint;
mod output_formatter;
mod result;
mod run;
mod walk;

#[cfg(test)]
mod tester;

/// Re-exported CLI-related items for use in `tasks/website`.
pub mod cli {
    pub use super::{command::*, lint::LintRunner, result::CliRunResult};
}

/// Main export for binary
pub use run::lint;

#[cfg(all(feature = "allocator", not(miri), not(target_family = "wasm")))]
#[global_allocator]
static GLOBAL: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;
