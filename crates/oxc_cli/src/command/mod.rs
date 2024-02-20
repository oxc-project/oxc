mod format;
mod ignore;
mod lint;

use bpaf::Bpaf;

pub use self::{
    format::{format_command, FormatOptions},
    ignore::IgnoreOptions,
    lint::{lint_command, LintOptions, OutputFormat, OutputOptions, WarningOptions},
};

use self::{format::format_options, lint::lint_options};

const VERSION: &str = match option_env!("OXC_VERSION") {
    Some(v) => v,
    None => "dev",
};

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version(VERSION))]
pub enum CliCommand {
    /// Lint this repository
    #[bpaf(command)]
    Lint(#[bpaf(external(lint_options))] LintOptions),

    /// Format this repository
    #[bpaf(command)]
    Format(#[bpaf(external(format_options))] FormatOptions),
}

impl CliCommand {
    pub fn handle_threads(&self) {
        match self {
            Self::Lint(options) => {
                Self::set_rayon_threads(options.misc_options.threads);
            }
            Self::Format(options) => {
                Self::set_rayon_threads(options.misc_options.threads);
            }
        }
    }

    fn set_rayon_threads(threads: Option<usize>) {
        if let Some(threads) = threads {
            rayon::ThreadPoolBuilder::new().num_threads(threads).build_global().unwrap();
        }
    }
}

/// Miscellaneous
#[derive(Debug, Clone, Bpaf)]
pub struct MiscOptions {
    /// Number of threads to use. Set to 1 for using only 1 CPU core
    #[bpaf(argument("INT"), hide_usage)]
    pub threads: Option<usize>,
}

#[cfg(test)]
mod misc_options {
    use super::lint::lint_command;
    use super::MiscOptions;

    fn get_misc_options(arg: &str) -> MiscOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        lint_command().run_inner(args.as_slice()).unwrap().lint_options.misc_options
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
