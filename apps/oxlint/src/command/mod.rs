mod ignore;
mod lint;

use std::path::PathBuf;

use usage_rs::Args;

pub use self::{
    ignore::IgnoreOptions,
    lint::{DebugOption, LintCommand, OutputOptions, ReportUnusedDirectives, WarningOptions},
};

/// Miscellaneous
#[derive(Debug, Clone, Args)]
pub struct MiscOptions {
    /// Do not display any diagnostics
    #[usage(long)]
    pub silent: bool,

    /// Do not exit with an error when no files are selected for linting
    /// (for example, after applying ignore patterns)
    #[usage(long)]
    pub no_error_on_unmatched_pattern: bool,

    /// Number of threads to use. Set to 1 for using only 1 CPU core.
    #[usage(long, value_name = "INT")]
    pub threads: Option<usize>,

    /// This option outputs the configuration to be used.
    /// When present, no linting is performed and only config-related options are valid.
    #[usage(long)]
    pub print_config: bool,
}

fn invalid_path(paths: &[PathBuf]) -> Option<&PathBuf> {
    paths.iter().find(|path| {
        path.components().any(|component| component == std::path::Component::ParentDir)
    })
}

const PATHS_ERROR_MESSAGE: &str = "PATH must not contain \"..\"";

#[cfg(test)]
mod misc_options {
    use super::{MiscOptions, lint::LintCommand};

    fn get_misc_options(arg: &str) -> MiscOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        LintCommand::parse_from(args.as_slice()).unwrap().misc_options
    }

    #[test]
    fn default() {
        let options = get_misc_options(".");
        assert!(!options.no_error_on_unmatched_pattern);
        assert!(options.threads.is_none());
    }

    #[test]
    fn no_error_on_unmatched_pattern() {
        let options = get_misc_options("--no-error-on-unmatched-pattern .");
        assert!(options.no_error_on_unmatched_pattern);
    }

    #[test]
    fn threads() {
        let options = get_misc_options("--threads 4 .");
        assert_eq!(options.threads, Some(4));
    }
}
