mod ignore;
mod lint;

use bpaf::Bpaf;
use std::path::PathBuf;

pub use self::{
    ignore::IgnoreOptions,
    lint::{lint_command, LintCommand, OutputFormat, OutputOptions, WarningOptions},
};

const VERSION: &str = match option_env!("OXC_VERSION") {
    Some(v) => v,
    None => "dev",
};

/// Miscellaneous
#[derive(Debug, Clone, Bpaf)]
pub struct MiscOptions {
    /// Do not display any diagnostics
    #[bpaf(switch, hide_usage)]
    pub silent: bool,

    /// Number of threads to use. Set to 1 for using only 1 CPU core
    #[bpaf(argument("INT"), hide_usage)]
    pub threads: Option<usize>,
}

#[allow(clippy::ptr_arg)]
fn validate_paths(paths: &Vec<PathBuf>) -> bool {
    if paths.is_empty() {
        true
    } else {
        paths.iter().all(|p| p.components().all(|c| c != std::path::Component::ParentDir))
    }
}

const PATHS_ERROR_MESSAGE: &str = "PATH must not contain \"..\"";

#[cfg(target_os = "windows")]
#[allow(clippy::needless_pass_by_value)]
fn expand_glob(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let match_options = glob::MatchOptions {
        case_sensitive: true,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    paths
        .iter()
        .filter_map(|path| path.to_str())
        .filter_map(|path| glob::glob_with(path, match_options).ok())
        .flatten()
        .filter_map(Result::ok)
        .collect()
}

#[cfg(not(target_os = "windows"))]
#[allow(clippy::needless_pass_by_value)]
fn expand_glob(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    // no-op on any os other than windows, since they expand globs
    paths
}

#[cfg(test)]
mod misc_options {
    use super::lint::lint_command;
    use super::MiscOptions;

    fn get_misc_options(arg: &str) -> MiscOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        lint_command().run_inner(args.as_slice()).unwrap().misc_options
    }

    #[test]
    fn default() {
        let options = get_misc_options(".");
        assert!(options.threads.is_none());
    }

    #[test]
    fn threads() {
        let options = get_misc_options("--threads 4 .");
        assert_eq!(options.threads, Some(4));
    }
}
