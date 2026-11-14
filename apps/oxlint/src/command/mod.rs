mod ignore;
mod lint;

use std::path::{Path, PathBuf};

use bpaf::{Bpaf, Parser, construct, long, positional};

pub use self::{
    ignore::IgnoreOptions,
    lint::{
        DebugOption, EnablePlugins, LintCommand, OutputOptions, ReportUnusedDirectives,
        WarningOptions, lint_command,
    },
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Miscellaneous
#[derive(Debug, Clone, Bpaf)]
pub struct MiscOptions {
    /// Do not display any diagnostics
    #[bpaf(switch, hide_usage)]
    pub silent: bool,

    /// Do not exit with an error when no files are selected for linting
    /// (for example, after applying ignore patterns)
    #[bpaf(switch, hide_usage)]
    pub no_error_on_unmatched_pattern: bool,

    /// Number of threads to use. Set to 1 for using only 1 CPU core.
    #[bpaf(argument("INT"), hide_usage)]
    pub threads: Option<usize>,

    #[bpaf(external, optional, hide_usage)]
    pub print_config: Option<PrintConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrintConfig {
    Root,
    File(PathBuf),
}

fn print_config() -> impl Parser<PrintConfig> {
    let flag = long("print-config")
        .help("Print the root configuration, or the effective configuration for FILE. No linting is performed.")
        .req_flag(());
    let file = positional::<PathBuf>("FILE")
        .guard(|path| validate_path(path), PATHS_ERROR_MESSAGE)
        .optional();
    construct!(flag, file)
        .adjacent()
        .map(|((), file)| file.map_or(PrintConfig::Root, PrintConfig::File))
}

fn validate_path(path: &Path) -> bool {
    path.components().all(|c| c != std::path::Component::ParentDir)
}

#[expect(clippy::ptr_arg)]
fn validate_paths(paths: &Vec<PathBuf>) -> bool {
    paths.iter().all(|path| validate_path(path))
}

const PATHS_ERROR_MESSAGE: &str = "PATH must not contain \"..\"";

#[cfg(test)]
mod misc_options {
    use super::{MiscOptions, PrintConfig, lint::lint_command};

    fn get_misc_options(arg: &str) -> MiscOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        lint_command().run_inner(args.as_slice()).unwrap().misc_options
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

    #[test]
    fn print_config() {
        let cases: &[(&[&str], Option<PrintConfig>, &[&str])] = &[
            (&[], None, &[]),
            (&["--print-config"], Some(PrintConfig::Root), &[]),
            (&["files", "--print-config"], Some(PrintConfig::Root), &["files"]),
            (&["--print-config", "--type-aware", "files"], Some(PrintConfig::Root), &["files"]),
            (
                &["--print-config", "-c", "config.json", "files"],
                Some(PrintConfig::Root),
                &["files"],
            ),
            (&["--print-config", "file.ts"], Some(PrintConfig::File("file.ts".into())), &[]),
            (
                &["--print-config", "file.ts", "--type-aware"],
                Some(PrintConfig::File("file.ts".into())),
                &[],
            ),
        ];
        for (args, expected, paths) in cases {
            let options = lint_command().run_inner(*args).unwrap();
            assert_eq!(&options.misc_options.print_config, expected, "{args:?}");
            assert_eq!(
                options.paths,
                paths.iter().map(std::path::PathBuf::from).collect::<Vec<_>>(),
                "{args:?}"
            );
        }
        for args in [
            &["--print-config", "../file.ts"][..],
            &["--print-config", "--unknown"],
            &["--print-config", "one.ts", "--print-config", "two.ts"],
        ] {
            assert!(lint_command().run_inner(args).is_err(), "{args:?}");
        }
    }
}
