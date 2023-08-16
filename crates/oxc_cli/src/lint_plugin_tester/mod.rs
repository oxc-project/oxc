use std::path::{Path, PathBuf};

use clap::{builder::ValueParser, Arg, ArgMatches, Command};

use crate::{plugin::test_queries, CliRunResult, Runner, RunnerOptions};

#[derive(Debug)]
pub struct LintPluginTestOptions {
    pub plugin_path: PathBuf,
}

#[allow(clippy::fallible_impl_from)]
impl<'a> From<&'a ArgMatches> for LintPluginTestOptions {
    fn from(matches: &'a ArgMatches) -> Self {
        Self {
            plugin_path: matches
                .get_one::<PathBuf>("plugin-path")
                .cloned()
                .unwrap_or_else(|| Path::new("./.oxc/plugins").to_path_buf()),
        }
    }
}

impl RunnerOptions for LintPluginTestOptions {
    fn build_args(cmd: Command) -> Command {
        cmd.arg(
            Arg::new("plugin-path")
                .long("plugin-path")
                .required(false)
                .value_parser(ValueParser::path_buf())
                .help("This option allows you to specify a path to search for linter plugins."),
        )
    }
}

pub struct LintPluginTestRunner {
    options: LintPluginTestOptions,
}

impl Runner for LintPluginTestRunner {
    type Options = LintPluginTestOptions;

    const ABOUT: &'static str = "Run the tests for each plugin.";
    const NAME: &'static str = "plugin-test";

    fn new(options: Self::Options) -> Self {
        Self { options }
    }

    fn run(&self) -> CliRunResult {
        let now = std::time::Instant::now();
        test_queries(self.options.plugin_path.clone());
        let duration = now.elapsed();
        // 0 or would have asserted
        CliRunResult::LintPluginTestResult { duration, number_of_diagnostics: 0 }
    }
}
