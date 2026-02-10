use std::{
    env,
    ffi::OsStr,
    fmt::Debug,
    io::{ErrorKind, Write},
    path::{Path, PathBuf, absolute},
    sync::Arc,
    time::Instant,
};

use cow_utils::CowUtils;
use ignore::{gitignore::Gitignore, overrides::OverrideBuilder};

use oxc_diagnostics::{DiagnosticSender, DiagnosticService, GraphicalReportHandler, OxcDiagnostic};
use oxc_linter::{
    AllowWarnDeny, ConfigStore, ConfigStoreBuilder, ExternalLinter, ExternalPluginStore,
    InvalidFilterKind, LintFilter, LintOptions, LintRunner, LintServiceOptions, Linter,
};

#[cfg(feature = "napi")]
use crate::js_config::JsConfigLoaderCb;
use crate::{
    cli::{CliRunResult, LintCommand, MiscOptions, ReportUnusedDirectives, WarningOptions},
    config_loader::{CliConfigLoadError, ConfigLoadError, ConfigLoader},
    output_formatter::{LintCommandInfo, OutputFormatter},
    walk::Walk,
};
use oxc_linter::LintIgnoreMatcher;

pub struct CliRunner {
    options: LintCommand,
    cwd: PathBuf,
    external_linter: Option<ExternalLinter>,
    /// Callback for loading JavaScript/TypeScript config files (experimental).
    /// This is only available when running via Node.js with NAPI.
    #[cfg(feature = "napi")]
    js_config_loader: Option<JsConfigLoaderCb>,
}

impl Debug for CliRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("CliRunner");
        s.field("options", &self.options).field("cwd", &self.cwd).field(
            "external_linter",
            if self.external_linter.is_some() { &"Some(ExternalLinter)" } else { &"None" },
        );
        #[cfg(feature = "napi")]
        s.field(
            "js_config_loader",
            if self.js_config_loader.is_some() { &"Some(JsConfigLoaderCb)" } else { &"None" },
        );
        s.finish()
    }
}

impl CliRunner {
    /// # Panics
    pub fn new(options: LintCommand, external_linter: Option<ExternalLinter>) -> Self {
        Self {
            options,
            cwd: env::current_dir().expect("Failed to get current working directory"),
            external_linter,
            #[cfg(feature = "napi")]
            js_config_loader: None,
        }
    }

    /// # Panics
    pub fn run(self, stdout: &mut dyn Write) -> CliRunResult {
        let format_str = self.options.output_options.format;
        let output_formatter = OutputFormatter::new(format_str);

        let LintCommand {
            paths,
            filter,
            basic_options,
            warning_options,
            ignore_options,
            fix_options,
            enable_plugins,
            misc_options,
            disable_nested_config,
            inline_config_options,
            suppression_options,
            ..
        } = self.options;

        if basic_options.init {
            return crate::mode::run_init(&self.cwd, stdout);
        }

        let external_linter = self.external_linter.as_ref();

        let mut paths = paths;
        let provided_path_count = paths.len();
        let now = Instant::now();

        let filters = match Self::get_filters(filter) {
            Ok(filters) => filters,
            Err((result, message)) => {
                print_and_flush_stdout(stdout, &message);
                return result;
            }
        };

        let handler = if cfg!(any(test, feature = "testing")) {
            GraphicalReportHandler::new_themed(miette::GraphicalTheme::none())
        } else {
            GraphicalReportHandler::new()
        };

        let mut override_builder = None;

        if !ignore_options.no_ignore {
            let mut builder = OverrideBuilder::new(&self.cwd);

            if !ignore_options.ignore_pattern.is_empty() {
                for pattern in &ignore_options.ignore_pattern {
                    // Meaning of ignore pattern is reversed
                    // <https://docs.rs/ignore/latest/ignore/overrides/struct.OverrideBuilder.html#method.add>
                    let pattern = format!("!{pattern}");
                    builder.add(&pattern).unwrap();
                }
            }

            let builder = builder.build().unwrap();

            // The ignore crate whitelists explicit paths, but priority
            // should be given to the ignore file. Many users lint
            // automatically and pass a list of changed files explicitly.
            // To accommodate this, unless `--no-ignore` is passed,
            // pre-filter the paths.
            if !paths.is_empty() {
                let (ignore, _err) = Gitignore::new(&ignore_options.ignore_path);

                paths.retain_mut(|p| {
                    // Try to prepend cwd to all paths
                    let Ok(mut path) = absolute(self.cwd.join(&p)) else {
                        return false;
                    };

                    std::mem::swap(p, &mut path);

                    if path.is_dir() {
                        true
                    } else {
                        !(builder.matched(p, false).is_ignore()
                            || ignore.matched(path, false).is_ignore())
                    }
                });
            }

            override_builder = Some(builder);
        }

        if paths.is_empty() {
            // If explicit paths were provided, but all have been
            // filtered, return early.
            if provided_path_count > 0 {
                if let Some(end) = output_formatter.lint_command_info(&LintCommandInfo {
                    number_of_files: 0,
                    number_of_rules: None,
                    threads_count: rayon::current_num_threads(),
                    start_time: now.elapsed(),
                }) {
                    print_and_flush_stdout(stdout, &end);
                }

                return CliRunResult::LintNoFilesFound;
            }

            paths.push(self.cwd.clone());
        }

        let walker = Walk::new(&paths, &ignore_options, override_builder);
        let mut paths = walker.paths();

        // NAPI tests build `oxlint` with `testing` feature enabled.
        // In NAPI tests, sort file paths if oxlint is run with `--threads 1`.
        // This guarantees files are linted in a deterministic order.
        //
        // Note: Sorting paths would not be sufficient to guarantee deterministic linting order unless
        // `--threads 1` is also used, because otherwise linting happens in parallel on multiple threads,
        // which also produces non-determinism.
        if cfg!(feature = "testing") && misc_options.threads == Some(1) {
            paths.sort_unstable();
        }

        let mut external_plugin_store = ExternalPluginStore::new(self.external_linter.is_some());

        // Setup JS workspace before loading any configs (config parsing can load JS plugins).
        if let Some(external_linter) = &external_linter {
            let res = (external_linter.create_workspace)(self.cwd.to_string_lossy().into_owned());

            if let Err(err) = res {
                print_and_flush_stdout(stdout, &format!("Failed to setup JS workspace:\n{err}\n"));
                return CliRunResult::JsPluginWorkspaceSetupFailed;
            }
        }

        let search_for_nested_configs = !disable_nested_config &&
            // If the `--config` option is explicitly passed, we should not search for nested config files
            // as the passed config file takes absolute precedence.
            basic_options.config.is_none() &&
            !misc_options.print_config &&
            !self.options.list_rules;

        let config_result = {
            let mut config_loader =
                ConfigLoader::new(external_linter, &mut external_plugin_store, &filters, None);
            #[cfg(feature = "napi")]
            {
                config_loader = config_loader.with_js_config_loader(self.js_config_loader.as_ref());
            }
            config_loader.load_root_and_nested(
                &self.cwd,
                basic_options.config.as_ref(),
                &paths,
                search_for_nested_configs,
            )
        };

        let (mut root_config, nested_configs, nested_ignore_patterns) = match config_result {
            Ok(loaded) => (loaded.root, loaded.nested, loaded.nested_ignore_patterns),
            Err(error) => {
                match error {
                    CliConfigLoadError::RootConfig(error) => {
                        print_and_flush_stdout(
                            stdout,
                            &format!(
                                "Failed to parse oxlint configuration file.\n{}\n",
                                render_report(&handler, &error)
                            ),
                        );
                    }
                    CliConfigLoadError::NestedConfigs(errors) => {
                        if let Some(error) = errors.into_iter().next() {
                            let message = match &error {
                                ConfigLoadError::Parse { path, error } => {
                                    format!(
                                        "Failed to parse oxlint configuration file at {}.\n{}\n",
                                        path.to_string_lossy().cow_replace('\\', "/"),
                                        render_report(&handler, error)
                                    )
                                }
                                ConfigLoadError::Build { path, error } => {
                                    format!(
                                        "Failed to build configuration from {}.\n{}\n",
                                        path.to_string_lossy().cow_replace('\\', "/"),
                                        render_report(
                                            &handler,
                                            &OxcDiagnostic::error(error.clone())
                                        )
                                    )
                                }
                                ConfigLoadError::JsConfigFileFoundButJsRuntimeNotAvailable => {
                                    "Error: JavaScript/TypeScript config files found but JS runtime not available.\n\
                                     This is an experimental feature that requires running oxlint via Node.js.\n\
                                     Please use JSON config files (.oxlintrc.json) instead, or run oxlint via the npm package.\n".to_string()
                                }
                                ConfigLoadError::Diagnostic(error) => {
                                    let report = render_report(&handler, error);
                                    format!("Failed to parse oxlint configuration file.\n{report}\n")
                                }
                            };
                            print_and_flush_stdout(stdout, &message);
                        }
                    }
                }

                return CliRunResult::InvalidOptionConfig;
            }
        };

        {
            let mut plugins = root_config.plugins.unwrap_or_default();
            enable_plugins.apply_overrides(&mut plugins);
            root_config.plugins = Some(plugins);
        }

        let base_ignore_patterns = root_config.ignore_patterns.clone();

        let config_builder = match ConfigStoreBuilder::from_oxlintrc(
            false,
            root_config.clone(),
            external_linter,
            &mut external_plugin_store,
            None,
        ) {
            Ok(builder) => builder,
            Err(e) => {
                print_and_flush_stdout(
                    stdout,
                    &format!(
                        "Failed to parse oxlint configuration file.\n{}\n",
                        render_report(&handler, &OxcDiagnostic::error(e.to_string()))
                    ),
                );
                return CliRunResult::InvalidOptionConfig;
            }
        }
        .with_filters(&filters);

        if misc_options.print_config {
            return crate::mode::run_print_config(&config_builder, root_config, stdout);
        }

        let lint_config = match config_builder.build(&mut external_plugin_store) {
            Ok(config) => config,
            Err(e) => {
                print_and_flush_stdout(
                    stdout,
                    &format!(
                        "Failed to build configuration.\n{}\n",
                        render_report(&handler, &OxcDiagnostic::error(e.to_string()))
                    ),
                );
                return CliRunResult::InvalidOptionConfig;
            }
        };

        if self.options.list_rules {
            return crate::mode::run_rules(&lint_config, &output_formatter, stdout);
        }

        let ignore_matcher =
            { LintIgnoreMatcher::new(&base_ignore_patterns, &self.cwd, nested_ignore_patterns) };

        // If no external rules, discard `ExternalLinter`
        let mut external_linter = self.external_linter;
        if external_plugin_store.is_empty() {
            external_linter = None;
        }

        // TODO(refactor): pull this into a shared function, so that the language server can use
        // the same functionality.
        let use_cross_module = lint_config.plugins().has_import()
            || nested_configs.values().any(|config| config.plugins().has_import());
        let mut options =
            LintServiceOptions::new(self.cwd.clone()).with_cross_module(use_cross_module);

        let report_unused_directives = match inline_config_options.report_unused_directives {
            ReportUnusedDirectives::WithoutSeverity(true) => Some(AllowWarnDeny::Warn),
            ReportUnusedDirectives::WithSeverity(Some(severity)) => Some(severity),
            _ => None,
        };
        let (mut diagnostic_service, tx_error) =
            Self::get_diagnostic_service(&output_formatter, &warning_options, &misc_options);

        let config_store = ConfigStore::new(lint_config, nested_configs, external_plugin_store);

        // Send JS plugins config to JS side
        if let Some(external_linter) = &external_linter {
            let res = config_store.external_plugin_store().setup_rule_configs(
                self.cwd.to_string_lossy().into_owned(),
                None,
                external_linter,
            );
            if let Err(err) = res {
                print_and_flush_stdout(
                    stdout,
                    &format!("Failed to setup JS plugin options:\n{err}\n"),
                );
                return CliRunResult::InvalidOptionConfig;
            }
        }

        let files_to_lint = paths
            .into_iter()
            .filter(|path| !ignore_matcher.should_ignore(Path::new(path)))
            .collect::<Vec<Arc<OsStr>>>();

        let linter = Linter::new(LintOptions::default(), config_store, external_linter)
            .with_fix(fix_options.fix_kind())
            .with_report_unused_directives(report_unused_directives)
            .with_suppress_all(suppression_options.suppress_all)
            .with_prune_suppressions(suppression_options.prune_suppressions);

        let number_of_files = files_to_lint.len();
        let tsconfig = basic_options.tsconfig;
        if let Some(path) = tsconfig.as_ref() {
            if path.is_file() {
                options = options.with_tsconfig(path);
            } else {
                let path = if path.is_relative() { options.cwd().join(path) } else { path.clone() };

                print_and_flush_stdout(
                    stdout,
                    &format!(
                        "The tsconfig file {:?} does not exist, Please provide a valid tsconfig file.\n",
                        path.to_string_lossy().cow_replace('\\', "/")
                    ),
                );

                return CliRunResult::InvalidOptionTsConfig;
            }
        }

        let number_of_rules = linter.number_of_rules(self.options.type_aware);

        // Create the LintRunner
        // TODO: Add a warning message if `tsgolint` cannot be found, but type-aware rules are enabled
        let lint_runner = match LintRunner::builder(options, linter)
            .with_type_aware(self.options.type_aware)
            .with_type_check(self.options.type_check)
            .with_silent(misc_options.silent)
            .with_fix_kind(fix_options.fix_kind())
            .with_suppress_all(suppression_options.suppress_all)
            .with_prune_suppressions(suppression_options.prune_suppressions)
            .build()
        {
            Ok(runner) => runner,
            Err(err) => {
                print_and_flush_stdout(stdout, &err);
                return CliRunResult::TsGoLintError;
            }
        };

        match lint_runner.lint_files(&files_to_lint, tx_error.clone()) {
            Ok(lint_runner) => {
                lint_runner.report_unused_directives(report_unused_directives, &tx_error);
            }
            Err(err) => {
                print_and_flush_stdout(stdout, &err);
                return CliRunResult::TsGoLintError;
            }
        }

        drop(tx_error);

        let diagnostic_result = diagnostic_service.run(stdout);

        if let Some(end) = output_formatter.lint_command_info(&LintCommandInfo {
            number_of_files,
            number_of_rules,
            threads_count: rayon::current_num_threads(),
            start_time: now.elapsed(),
        }) {
            print_and_flush_stdout(stdout, &end);
        }

        if diagnostic_result.errors_count() > 0 {
            CliRunResult::LintFoundErrors
        } else if warning_options.deny_warnings && diagnostic_result.warnings_count() > 0 {
            CliRunResult::LintNoWarningsAllowed
        } else if diagnostic_result.max_warnings_exceeded() {
            CliRunResult::LintMaxWarningsExceeded
        } else {
            CliRunResult::LintSucceeded
        }
    }
}

impl CliRunner {
    #[must_use]
    pub fn with_cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = cwd;
        self
    }

    #[cfg(feature = "napi")]
    #[must_use]
    pub fn with_config_loader(mut self, config_loader: Option<JsConfigLoaderCb>) -> Self {
        self.js_config_loader = config_loader;
        self
    }

    fn get_diagnostic_service(
        reporter: &OutputFormatter,
        warning_options: &WarningOptions,
        misc_options: &MiscOptions,
    ) -> (DiagnosticService, DiagnosticSender) {
        let (service, sender) = DiagnosticService::new(reporter.get_diagnostic_reporter());
        (
            service
                .with_quiet(warning_options.quiet)
                .with_silent(misc_options.silent)
                .with_max_warnings(warning_options.max_warnings),
            sender,
        )
    }

    // moved into a separate function for readability, but it's only ever used
    // in one place.
    fn get_filters(
        filters_arg: Vec<(AllowWarnDeny, String)>,
    ) -> Result<Vec<LintFilter>, (CliRunResult, String)> {
        let mut filters = Vec::with_capacity(filters_arg.len());

        for (severity, filter_arg) in filters_arg {
            match LintFilter::new(severity, filter_arg) {
                Ok(filter) => {
                    filters.push(filter);
                }
                Err(InvalidFilterKind::Empty) => {
                    return Err((
                        CliRunResult::InvalidOptionSeverityWithoutFilter,
                        format!("Cannot {severity} an empty filter.\n"),
                    ));
                }
                Err(InvalidFilterKind::PluginMissing(filter)) => {
                    return Err((
                        CliRunResult::InvalidOptionSeverityWithoutPluginName,
                        format!(
                            "Failed to {severity} filter {filter}: Plugin name is missing. Expected <plugin>/<rule>\n"
                        ),
                    ));
                }
                Err(InvalidFilterKind::RuleMissing(filter)) => {
                    return Err((
                        CliRunResult::InvalidOptionSeverityWithoutRuleName,
                        format!(
                            "Failed to {severity} filter {filter}: Rule name is missing. Expected <plugin>/<rule>\n"
                        ),
                    ));
                }
            }
        }

        Ok(filters)
    }
}

pub fn print_and_flush_stdout(stdout: &mut dyn Write, message: &str) {
    stdout.write_all(message.as_bytes()).or_else(check_for_writer_error).unwrap();
    stdout.flush().unwrap();
}

fn check_for_writer_error(error: std::io::Error) -> Result<(), std::io::Error> {
    // Do not panic when the process is killed (e.g. piping into `less`).
    if matches!(error.kind(), ErrorKind::Interrupted | ErrorKind::BrokenPipe) {
        Ok(())
    } else {
        Err(error)
    }
}

fn render_report(handler: &GraphicalReportHandler, diagnostic: &OxcDiagnostic) -> String {
    let mut err = String::new();
    handler.render_report(&mut err, diagnostic).unwrap();
    err
}

#[cfg(test)]
mod test {
    use std::fs;

    use crate::{DEFAULT_OXLINTRC_NAME, tester::Tester};
    use oxc_linter::rules::RULES;

    // lints the full directory of fixtures,
    // so do not snapshot it, test only
    #[test]
    fn no_arg() {
        let args = &[];
        Tester::new().test(args);
    }

    #[test]
    fn dir() {
        let args = &["fixtures/linter"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn cwd() {
        let args = &["debugger.js"];
        Tester::new().with_cwd("fixtures/linter".into()).test_and_snapshot(args);
    }

    #[test]
    fn file() {
        let args = &["fixtures/linter/debugger.js"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn multi_files() {
        let args = &["fixtures/linter/debugger.js", "fixtures/linter/nan.js"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn wrong_extension() {
        let args = &["foo.asdf"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn ignore_pattern() {
        let args =
            &["--ignore-pattern", "**/*.js", "--ignore-pattern", "**/*.vue", "fixtures/linter"];
        Tester::new().test_and_snapshot(args);
    }

    /// When a file is explicitly passed as a path and `--no-ignore`
    /// is not present, the ignore file should take precedence.
    /// See https://github.com/oxc-project/oxc/issues/1124
    #[test]
    fn ignore_file_overrides_explicit_args() {
        let args = &["--ignore-path", "fixtures/linter/.customignore", "fixtures/linter/nan.js"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn ignore_file_no_ignore() {
        let args = &[
            "--ignore-path",
            "fixtures/linter/.customignore",
            "--no-ignore",
            "fixtures/linter/nan.js",
        ];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn ignore_flow() {
        let args = &["--import-plugin", "fixtures/flow/index.mjs"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    // https://github.com/oxc-project/oxc/issues/7406
    fn ignore_flow_import_plugin_directory() {
        let args = &["--import-plugin", "-A all", "-D no-cycle", "fixtures/flow/"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    // https://github.com/oxc-project/oxc/issues/9023
    fn ignore_file_current_dir() {
        let args1 = &[];
        let args2 = &["."];
        Tester::new()
            .with_cwd("fixtures/ignore_file_current_dir".into())
            .test_and_snapshot_multiple(&[args1, args2]);
    }

    #[test]
    // https://github.com/oxc-project/oxc/issues/13204
    fn ignore_pattern_non_glob_syntax() {
        let args1 = &[];
        let args2 = &["."];
        Tester::new()
            .with_cwd("fixtures/ignore_pattern_non_glob_syntax".into())
            .test_and_snapshot_multiple(&[args1, args2]);
    }

    #[test]
    fn ignore_patterns_empty_nested() {
        let args1 = &[];
        let args2 = &["."];
        Tester::new()
            .with_cwd("fixtures/ignore_patterns_empty_nested".into())
            .test_and_snapshot_multiple(&[args1, args2]);
    }

    #[test]
    fn ignore_patterns_relative() {
        let args1 = &[];
        let args2 = &["."];
        Tester::new()
            .with_cwd("fixtures/ignore_patterns_relative".into())
            .test_and_snapshot_multiple(&[args1, args2]);
    }

    #[test]
    fn ignore_patterns_with_symlink() {
        let args1 = &[];
        let args2 = &["."];
        Tester::new()
            .with_cwd("fixtures/ignore_patterns_symlink".into())
            .test_and_snapshot_multiple(&[args1, args2]);
    }

    #[test]
    fn ignore_patterns_whitelist() {
        let args1 = &[];
        let args2 = &["."];
        Tester::new()
            .with_cwd("fixtures/ignore_patterns_whitelist".into())
            .test_and_snapshot_multiple(&[args1, args2]);
    }

    #[test]
    fn filter_allow_all() {
        let args = &["-A", "all", "fixtures/linter"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn filter_allow_one() {
        let args = &["-W", "correctness", "-A", "no-debugger", "fixtures/linter/debugger.js"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn filter_error() {
        let args = &["-D", "correctness", "fixtures/linter/debugger.js"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn eslintrc_error() {
        let args = &["-c", "fixtures/linter/eslintrc.json", "fixtures/linter/debugger.js"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn eslintrc_off() {
        let args = &["-c", "fixtures/eslintrc_off/eslintrc.json", "fixtures/eslintrc_off/test.js"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn oxlint_config_auto_detection() {
        let args = &["debugger.js"];
        Tester::new().with_cwd("fixtures/auto_config_detection".into()).test_and_snapshot(args);
    }

    #[test]
    #[cfg(not(target_os = "windows"))] // Skipped on Windows due to snapshot diffs from path separators (`/` vs `\`)
    fn oxlint_config_auto_detection_parse_error() {
        let args = &["debugger.js"];
        Tester::new().with_cwd("fixtures/auto_config_parse_error".into()).test_and_snapshot(args);
    }

    #[test]
    fn eslintrc_no_undef() {
        let args = &[
            "-W",
            "no-undef",
            "-c",
            "fixtures/no_undef/eslintrc.json",
            "fixtures/no_undef/test.js",
        ];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn eslintrc_no_env() {
        let args = &[
            "-W",
            "no-undef",
            "-c",
            "fixtures/eslintrc_env/eslintrc_no_env.json",
            "fixtures/eslintrc_env/test.js",
        ];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn eslintrc_with_env() {
        let args = &[
            "-c",
            "fixtures/eslintrc_env/eslintrc_env_browser.json",
            "fixtures/eslintrc_env/test.js",
        ];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn no_empty_allow_empty_catch() {
        let args = &[
            "-c",
            "fixtures/no_empty_allow_empty_catch/eslintrc.json",
            "-W",
            "no-empty",
            "fixtures/no_empty_allow_empty_catch/test.js",
        ];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn no_empty_disallow_empty_catch() {
        let args = &[
            "-c",
            "fixtures/no_empty_disallow_empty_catch/eslintrc.json",
            "-W",
            "no-empty",
            "fixtures/no_empty_disallow_empty_catch/test.js",
        ];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn no_console_off() {
        let args =
            &["-c", "fixtures/no_console_off/eslintrc.json", "fixtures/no_console_off/test.js"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn typescript_eslint() {
        let args = &[
            "-c",
            "fixtures/typescript_eslint/eslintrc.json",
            "fixtures/typescript_eslint/test.ts",
        ];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn typescript_eslint_off() {
        let args = &[
            "-c",
            "fixtures/typescript_eslint/eslintrc.json",
            "--disable-typescript-plugin",
            "fixtures/typescript_eslint/test.ts",
        ];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn js_and_jsx() {
        let args = &["fixtures/linter/js_as_jsx.js"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn lint_vue_file() {
        let args = &["fixtures/vue/debugger.vue"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn lint_empty_vue_file() {
        let args = &["fixtures/vue/empty.vue"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn lint_invalid_vue_file() {
        let args = &["fixtures/vue/invalid.vue"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn lint_astro_file() {
        let args = &["fixtures/astro/debugger.astro"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn lint_svelte_file() {
        let args = &["fixtures/svelte/debugger.svelte"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn test_tsconfig_option() {
        // passed
        Tester::new().with_cwd("fixtures/tsconfig".into()).test(&["--tsconfig", "tsconfig.json"]);

        // failed
        Tester::new()
            .with_cwd("fixtures/tsconfig".into())
            .test_and_snapshot(&["--tsconfig", "non-exists.json"]);
    }

    #[test]
    fn test_enable_vitest_rule_without_plugin() {
        let args = &[
            "-c",
            "fixtures/eslintrc_vitest_replace/eslintrc.json",
            "fixtures/eslintrc_vitest_replace/foo.test.js",
        ];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn test_enable_vitest_plugin() {
        let args = &[
            "--vitest-plugin",
            "-c",
            "fixtures/eslintrc_vitest_replace/eslintrc.json",
            "fixtures/eslintrc_vitest_replace/foo.test.js",
        ];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn test_import_plugin_enabled_in_config() {
        let args_1 = &["-c", ".oxlintrc.json", "test.js"];
        // support import-x namespace see #8779
        let args_2 = &["-c", ".oxlintrc-import-x.json", "test.js"];
        Tester::new()
            .with_cwd("fixtures/import".into())
            .test_and_snapshot_multiple(&[args_1, args_2]);
    }

    #[test]
    fn test_fix() {
        Tester::test_fix("fixtures/fix_argument/fix.js", "debugger\n", "\n");
        Tester::test_fix(
            "fixtures/fix_argument/fix.vue",
            "<script>debugger;</script>\n<script>debugger;</script>\n",
            "<script></script>\n<script></script>\n",
        );
    }

    #[test]
    fn test_print_config_ban_all_rules() {
        let args = &["-A", "all", "--print-config"];
        Tester::new().with_cwd("fixtures".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_print_config_ban_rules() {
        let args = &[
            "-c",
            "fixtures/print_config/ban_rules/eslintrc.json",
            "-A",
            "all",
            "-D",
            "eqeqeq",
            "--print-config",
        ];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn test_init_config() {
        assert!(!fs::exists(DEFAULT_OXLINTRC_NAME).unwrap());

        let args = &["--init"];
        Tester::new().with_cwd("fixtures".into()).test(args);

        assert!(fs::exists(DEFAULT_OXLINTRC_NAME).unwrap());

        fs::remove_file(DEFAULT_OXLINTRC_NAME).unwrap();
    }

    #[test]
    fn test_overrides() {
        let args_1 = &["-c", "fixtures/overrides/.oxlintrc.json", "fixtures/overrides/test.js"];
        let args_2 = &["-c", "fixtures/overrides/.oxlintrc.json", "fixtures/overrides/test.ts"];
        let args_3 = &["-c", "fixtures/overrides/.oxlintrc.json", "fixtures/overrides/other.jsx"];
        Tester::new().test_and_snapshot_multiple(&[args_1, args_2, args_3]);
    }

    #[test]
    fn test_overrides_directories() {
        let args = &["-c", "fixtures/overrides/directories-config.json", "fixtures/overrides"];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn test_overrides_envs_and_global() {
        let args = &["-c", ".oxlintrc.json", "."];
        Tester::new().with_cwd("fixtures/overrides_env_globals".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_ignore_patterns() {
        let args = &["-c", "./test/eslintrc.json", "--ignore-pattern", "*.ts", "."];

        Tester::new()
            .with_cwd("fixtures/config_ignore_patterns/with_oxlintrc".into())
            .test_and_snapshot(args);
    }

    #[test]
    fn test_config_ignore_patterns_extension() {
        let args = &[
            "-c",
            "fixtures/config_ignore_patterns/ignore_extension/eslintrc.json",
            "fixtures/config_ignore_patterns/ignore_extension",
        ];

        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn test_config_ignore_patterns_special_extension() {
        let args = &[
            "-c",
            "fixtures/config_ignore_patterns/ignore_extension/eslintrc.json",
            "fixtures/config_ignore_patterns/ignore_extension/main.js",
        ];

        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn test_config_ignore_patterns_directory() {
        let args = &["-c", "eslintrc.json"];
        Tester::new()
            .with_cwd("fixtures/config_ignore_patterns/ignore_directory".into())
            .test_and_snapshot(args);
    }

    // Issue: <https://github.com/oxc-project/oxc/pull/7566>
    #[test]
    fn ignore_path_with_relative_files() {
        let args = &[
            "--ignore-path",
            "fixtures/issue_7566/.oxlintignore",
            "fixtures/issue_7566/tests/main.js",
            "fixtures/issue_7566/tests/function/main.js",
        ];
        Tester::new().test_and_snapshot(args);
    }

    #[test]
    fn test_jest_and_vitest_alias_rules() {
        let args_1 = &["-c", "oxlint-jest.json", "test.js"];
        let args_2 = &["-c", "oxlint-vitest.json", "test.js"];
        Tester::new()
            .with_cwd("fixtures/jest_and_vitest_alias_rules".into())
            .test_and_snapshot_multiple(&[args_1, args_2]);
    }

    #[test]
    fn test_eslint_and_typescript_alias_rules() {
        let args_1 = &["-c", "oxlint-eslint.json", "test.js"];
        let args_2 = &["-c", "oxlint-typescript.json", "test.js"];
        Tester::new()
            .with_cwd("fixtures/eslint_and_typescript_alias_rules".into())
            .test_and_snapshot_multiple(&[args_1, args_2]);
    }

    #[test]
    fn test_disable_eslint_and_unicorn_alias_rules() {
        let args_1 = &["-c", ".oxlintrc-eslint.json", "test.js"];
        let args_2 = &["-c", ".oxlintrc-unicorn.json", "test.js"];
        Tester::new()
            .with_cwd("fixtures/disable_eslint_and_unicorn_alias_rules".into())
            .test_and_snapshot_multiple(&[args_1, args_2]);
    }

    #[test]
    // Test to ensure that a vitest rule based on the jest rule is
    // handled correctly when it has a different name.
    // e.g. `vitest/no-restricted-vi-methods` vs `jest/no-restricted-jest-methods`
    fn test_disable_vitest_rules() {
        let args =
            &["-c", ".oxlintrc-vitest.json", "--report-unused-disable-directives", "test.js"];
        Tester::new().with_cwd("fixtures/disable_vitest_rules".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_two_rules_with_same_rule_name_from_different_plugins() {
        // Issue: <https://github.com/oxc-project/oxc/issues/8485>
        let args = &["-c", ".oxlintrc.json", "test.js"];
        Tester::new()
            .with_cwd("fixtures/two_rules_with_same_rule_name".into())
            .test_and_snapshot(args);
    }

    #[test]
    fn test_report_unused_directives() {
        let args = &["-c", ".oxlintrc.json", "--report-unused-disable-directives"];

        Tester::new().with_cwd("fixtures/report_unused_directives".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_nested_config() {
        let args = &[];
        Tester::new().with_cwd("fixtures/nested_config".into()).test_and_snapshot(args);

        let args = &["--disable-nested-config"];
        Tester::new().with_cwd("fixtures/extends_config".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_nested_config_subdirectory() {
        // This tests the specific scenario from issue #10156
        // where a file is located in a subdirectory of a directory with a config file
        let args = &["package3-deep-config"];
        Tester::new().with_cwd("fixtures/nested_config".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_nested_config_explicit_config_precedence() {
        // `--config` takes absolute precedence over nested configs, and will be used for
        // linting all files rather than the nested configuration files.
        let args = &["--config", "oxlint-no-console.json"];
        Tester::new().with_cwd("fixtures/nested_config".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_nested_config_filter_precedence() {
        // CLI arguments take precedence over nested configs, but apply over top of the nested
        // config files, rather than replacing them.
        let args = &["-A", "no-console"];
        Tester::new().with_cwd("fixtures/nested_config".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_nested_config_explicit_config_and_filter_precedence() {
        // Combining `--config` and CLI filters should make the passed config file be
        // used for all files, but still override any rules specified in the config file.
        let args = &["-A", "no-console", "--config", "oxlint-no-console.json"];
        Tester::new().with_cwd("fixtures/nested_config".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_extends_explicit_config() {
        // Check that referencing a config file that extends other config files works as expected
        let args = &["--config", "extends_rules_config.json", "console.js"];
        Tester::new().with_cwd("fixtures/extends_config".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_extends_extends_config() {
        // Check that using a config that extends a config which extends a config works
        let args = &["--config", "relative_paths/extends_extends_config.json", "console.js"];
        Tester::new().with_cwd("fixtures/extends_config".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_extends_overrides() {
        // Check that using a config with overrides works as expected
        let args = &["overrides"];
        Tester::new().with_cwd("fixtures/extends_config".into()).test_and_snapshot(args);

        // Check that using a config which extends a config with overrides works as expected
        let args = &["overrides_same_directory"];
        Tester::new().with_cwd("fixtures/extends_config".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_nested_config_multi_file_analysis_imports() {
        let args = &["issue_10054"];
        Tester::new().with_cwd("fixtures".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_cross_modules_with_nested_config() {
        let args = &[];
        Tester::new()
            .with_cwd("fixtures/cross_module_nested_config".into())
            .test_and_snapshot(args);
    }

    #[test]
    fn test_cross_modules_with_extended_config() {
        let args = &[];
        Tester::new()
            .with_cwd("fixtures/cross_module_extended_config".into())
            .test_and_snapshot(args);
    }

    #[test]
    fn test_import_plugin_being_enabled_correctly() {
        // https://github.com/oxc-project/oxc/pull/10597
        let args = &["--import-plugin", "-D", "import/no-cycle"];
        Tester::new().with_cwd("fixtures/import-cycle".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_rule_config_being_enabled_correctly() {
        let args = &["-c", ".oxlintrc.json"];
        Tester::new().with_cwd("fixtures/issue_11054".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_plugins_in_overrides_enabled_correctly() {
        let args = &["-c", ".oxlintrc.json"];
        Tester::new().with_cwd("fixtures/overrides_with_plugin".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_plugins_inside_overrides_categories_enabled_correctly() {
        let args = &["-c", ".oxlintrc.json"];
        Tester::new().with_cwd("fixtures/issue_10394".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_jsx_a11y_label_has_associated_control() {
        let args = &["-c", ".oxlintrc.json"];
        Tester::new().with_cwd("fixtures/issue_11644".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_dot_folder() {
        Tester::new().with_cwd("fixtures/dot_folder".into()).test_and_snapshot(&[]);
    }

    #[test]
    fn test_rules_json_output() {
        let args = &["--rules", "-f=json"];
        let stdout = Tester::new().with_cwd("fixtures".into()).test_output(args);

        // Parse output as JSON array. If parsing fails, the test will fail.
        let rules: Vec<serde_json::Value> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON");
        assert!(!rules.is_empty(), "The rules list should not be empty");

        // Ensure that the number of rules matches the RULES constant, all rules should be listed.
        assert_eq!(rules.len(), RULES.len(), "The number of rules should match the RULES constant");

        // Validate the structure of JSON objects.
        for rule in &rules {
            let rule_obj = rule.as_object().unwrap();
            assert!(rule_obj.contains_key("scope"), "Rule should contain 'scope' field");
            assert!(rule_obj.contains_key("value"), "Rule should contain 'value' field");
            assert!(rule_obj.contains_key("category"), "Rule should contain 'category' field");
            assert!(rule_obj.contains_key("type_aware"), "Rule should contain 'type_aware' field");
            assert!(rule_obj.contains_key("fix"), "Rule should contain 'fix' field");
            assert!(rule_obj.contains_key("default"), "Rule should contain 'default' field");
            assert!(rule_obj.contains_key("docs_url"), "Rule should contain 'docs_url' field");
        }

        // Verify that the rules list is sorted by scope and value.
        let rule_names: Vec<(&str, &str)> = rules
            .iter()
            .map(|rule| {
                let obj = rule.as_object().unwrap();
                (obj["scope"].as_str().unwrap(), obj["value"].as_str().unwrap())
            })
            .collect();
        assert!(rule_names.is_sorted(), "The rules list should be sorted by scope and value");
    }

    #[test]
    fn test_disable_directive_issue_13311() {
        // Test that exhaustive-deps diagnostics are reported at the dependency array
        // so that disable directives work correctly
        // Issue: https://github.com/oxc-project/oxc/issues/13311
        let args = &["test.jsx", "test2.d.ts"];
        Tester::new()
            .with_cwd("fixtures/disable_directive_issue_13311".into())
            .test_and_snapshot(args);
    }

    // ToDo: `tsgolint` does not support `big-endian`?
    #[test]
    #[cfg(not(target_endian = "big"))]
    fn test_tsgolint() {
        let args = &["--type-aware"];
        Tester::new().with_cwd("fixtures/tsgolint".into()).test_and_snapshot(args);
    }

    #[test]
    #[cfg(not(target_endian = "big"))]
    fn test_tsgolint_silent() {
        let args = &["--type-aware", "--silent"];
        Tester::new().with_cwd("fixtures/tsgolint".into()).test_and_snapshot(args);
    }

    #[test]
    #[cfg(not(target_endian = "big"))]
    fn test_tsgolint_config() {
        // TODO: test with other rules as well once diagnostics are more stable
        let args = &["--type-aware", "-c", "config-test.json"];
        Tester::new().with_cwd("fixtures/tsgolint".into()).test_and_snapshot(args);
    }

    #[test]
    #[cfg(not(target_endian = "big"))]
    fn test_tsgolint_type_error() {
        let args = &["--type-aware", "--type-check"];
        Tester::new().with_cwd("fixtures/tsgolint_type_error".into()).test_and_snapshot(args);
    }

    #[test]
    #[cfg(not(target_endian = "big"))]
    fn test_tsgolint_no_typescript_files() {
        // tsgolint shouldn't run when no files need type aware linting
        let args = &["--type-aware", "test.svelte"];
        Tester::new().with_cwd("fixtures/tsgolint".into()).test_and_snapshot(args);
    }

    #[cfg(not(target_endian = "big"))]
    #[test]
    fn test_tsgolint_unused_disable_directives() {
        // Test that unused disable directives are reported with type-aware rules
        let args = &["--type-aware", "--report-unused-disable-directives", "unused.ts"];
        Tester::new()
            .with_cwd("fixtures/tsgolint_disable_directives".into())
            .test_and_snapshot(args);
    }

    #[cfg(not(target_endian = "big"))]
    #[test]
    fn test_tsgolint_disable_directives() {
        // Test that disable directives work with type-aware rules
        let args = &["--type-aware", "test.ts"];
        Tester::new()
            .with_cwd("fixtures/tsgolint_disable_directives".into())
            .test_and_snapshot(args);
    }

    #[test]
    #[cfg(all(not(target_os = "windows"), not(target_endian = "big")))]
    fn test_tsgolint_config_error() {
        let args = &["--type-aware"];
        Tester::new().with_cwd("fixtures/tsgolint_config_error".into()).test_and_snapshot(args);
    }

    #[test]
    #[cfg(all(not(target_os = "windows"), not(target_endian = "big")))]
    fn test_tsgolint_tsconfig_extends_config_err() {
        let args = &["--type-aware", "-D", "no-floating-promises"];
        Tester::new()
            .with_cwd("fixtures/tsgolint_tsconfig_extends_config_err".into())
            .test_and_snapshot(args);
    }

    #[test]
    #[cfg(all(not(target_os = "windows"), not(target_endian = "big")))]
    fn test_tsgolint_rule_options() {
        // Test that rule options are correctly passed to tsgolint
        // See: https://github.com/oxc-project/oxc/issues/16182
        let args = &["--type-aware"];
        Tester::new().with_cwd("fixtures/tsgolint_rule_options".into()).test_and_snapshot(args);
    }

    #[test]
    #[cfg(all(not(target_os = "windows"), not(target_endian = "big")))]
    fn test_tsgolint_fix() {
        Tester::test_fix_with_args(
            "fixtures/tsgolint_fix/fix.ts",
            "// This file has a fixable tsgolint error: no-unnecessary-type-assertion
// The type assertion `as string` is unnecessary because str is already a string
const str: string = 'hello';
const redundant = str as string;

export { redundant };
",
            "// This file has a fixable tsgolint error: no-unnecessary-type-assertion
// The type assertion `as string` is unnecessary because str is already a string
const str: string = 'hello';
const redundant = str;

export { redundant };
",
            &["--type-aware", "-D", "no-unnecessary-type-assertion"],
        );
    }

    #[test]
    fn test_invalid_config_invalid_config_enum() {
        Tester::new().with_cwd("fixtures/invalid_config_enum".into()).test_and_snapshot(&[]);
    }

    #[test]
    fn test_invalid_config_invalid_config_extra_options() {
        Tester::new()
            .with_cwd("fixtures/invalid_config_extra_options".into())
            .test_and_snapshot(&[]);
    }

    #[test]
    fn test_invalid_config_rule_without_config_but_options() {
        Tester::new()
            .with_cwd("fixtures/invalid_config_rules_without_config".into())
            .test_and_snapshot(&[]);
    }

    #[test]
    // Ensure the config validation works with vitest/no-hooks, which
    // is an alias of jest/no-hooks.
    fn test_invalid_config_invalid_config_with_rule_alias() {
        Tester::new()
            .with_cwd("fixtures/invalid_config_with_rule_alias".into())
            .test_and_snapshot(&[]);
    }

    #[test]
    fn test_invalid_config_invalid_config_in_override() {
        Tester::new().with_cwd("fixtures/invalid_config_in_override".into()).test_and_snapshot(&[]);
    }

    #[test]
    fn test_invalid_config_invalid_config_multiple_rules() {
        Tester::new()
            .with_cwd("fixtures/invalid_config_multiple_rules".into())
            .test_and_snapshot(&[]);
    }

    #[test]
    fn test_invalid_config_nested() {
        Tester::new().with_cwd("fixtures/invalid_config_nested".into()).test_and_snapshot(&[]);
    }

    #[test]
    fn test_invalid_config_invalid_config_type_difference() {
        Tester::new()
            .with_cwd("fixtures/invalid_config_type_difference".into())
            .test_and_snapshot(&[]);
    }

    #[test]
    fn test_invalid_config_invalid_config_extends() {
        Tester::new().with_cwd("fixtures/extends_invalid_config".into()).test_and_snapshot(&[]);
    }

    #[test]
    fn test_invalid_config_invalid_config_sort_imports() {
        Tester::new()
            .with_cwd("fixtures/invalid_config_sort_imports".into())
            .test_and_snapshot(&[]);
    }

    #[test]
    fn test_valid_complex_config() {
        Tester::new().with_cwd("fixtures/valid_complex_config".into()).test_and_snapshot(&[]);
    }

    /// Test that rules with dummy `config = Value` declarations can accept
    /// configuration options without errors. This test should be removed in
    /// the future, once these rules have been updated to use proper config
    /// structs.
    #[test]
    fn test_valid_config_rules_with_dummy_config() {
        Tester::new()
            .with_cwd("fixtures/valid_config_rules_with_dummy_config".into())
            .test_and_snapshot(&[]);
    }

    #[test]
    fn test_invalid_config_invalid_config_complex_enum() {
        Tester::new()
            .with_cwd("fixtures/invalid_config_complex_enum".into())
            .test_and_snapshot(&[]);
    }

    #[test]
    fn test_invalid_config_invalid_config_tuple_rules() {
        Tester::new().with_cwd("fixtures/invalid_config_tuple_rules".into()).test_and_snapshot(&[]);
    }
}

#[cfg(test)]
mod suppression {
    use std::{env, fs};

    use crate::tester::Tester;

    #[test]
    fn test_suppression_not_file_reporting_errors() {
        let args = &[];
        Tester::new()
            .with_cwd("fixtures/suppression_not_file_reporting_errors".into())
            .test_and_snapshot(args);
    }
    #[test]
    fn test_suppression_not_reporting_new_errors() {
        let args = &[];
        Tester::new()
            .with_cwd("fixtures/suppression_not_reporting_new_errors".into())
            .test_and_snapshot(args);
    }

    #[test]
    fn test_suppression_report_only_from_one_file() {
        let args = &[];
        Tester::new()
            .with_cwd("fixtures/suppression_report_only_from_one_file".into())
            .test_and_snapshot(args);
    }

    #[test]
    fn test_suppression_eslint_file_format() {
        let args = &[];
        Tester::new()
            .with_cwd("fixtures/suppression_eslint_file_format".into())
            .test_and_snapshot(args);
    }

    #[test]
    fn test_suppression_less_rules_violations_warning() {
        let args = &[];
        Tester::new()
            .with_cwd("fixtures/suppression_less_rules_violations_warning".into())
            .test_and_snapshot(args);
    }

    #[test]
    fn test_suppression_prune_errors_warning() {
        let args = &[];
        Tester::new()
            .with_cwd("fixtures/suppression_prune_errors_warning".into())
            .test_and_snapshot(args);
    }

    #[test]
    fn test_suppression_report_one_of_the_errors_from_one_file() {
        let args = &[];
        Tester::new()
            .with_cwd("fixtures/suppression_report_one_of_the_errors_from_one_file".into())
            .test_and_snapshot(args);
    }

    #[test]
    fn test_suppression_without_file() {
        let args = &[];
        Tester::new().with_cwd("fixtures/suppression_without_file".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_suppression_report_one_new_error_but_filter_the_rest() {
        let args = &[];
        Tester::new()
            .with_cwd("fixtures/suppression_report_one_new_error_but_filter_the_rest".into())
            .test_and_snapshot(args);
    }

    #[test]
    fn test_suppression_with_suppress_all_arg_and_no_file() {
        let cwd = env::current_dir().unwrap();
        let fixture_path =
            "fixtures/suppression_with_suppress_all_arg_and_no_file/oxlint-suppressions.json";
        let fixture_buf = cwd.join(fixture_path);
        let expected_buf = cwd.join(
            "fixtures/suppression_with_suppress_all_arg_and_no_file/oxlint-suppressions-expected.json",
        );
        let fixture_path = fixture_buf.to_str().unwrap();
        assert!(
            !fs::exists(fixture_path).unwrap(),
            "oxlint-suppression found in fixtures/suppression_with_suppress_all_arg_and_no_file/oxlint-suppressions.json"
        );

        let args = &["--suppress-all"];
        Tester::new()
            .with_cwd("fixtures/suppression_with_suppress_all_arg_and_no_file".into())
            .test(args);

        assert!(
            fs::exists(fixture_path).unwrap(),
            "oxlint-suppression not found in fixtures/suppression_with_suppress_all_arg_and_no_file/oxlint-suppressions.json"
        );

        let stdout = Tester::new()
            .with_cwd("fixtures/suppression_with_suppress_all_arg_and_no_file".into())
            .test_output(args);

        assert!(stdout.starts_with("Found 0 warnings and 0 errors."), "Unexpected errors found");

        let new_content = fs::read_to_string(cwd.join(fixture_path))
            .expect("Unable to read the new oxlint-suppressions.json");
        let expected_content = fs::read_to_string(expected_buf)
            .expect("Unable to read the expected content oxlint-suppressions-expected.json");

        assert_eq!(
            new_content, expected_content,
            "The suppression generated doesn't match the expected"
        );

        fs::remove_file(fixture_path).unwrap();
    }

    #[test]
    fn test_suppression_with_prune_all_arg_and_no_file() {
        let cwd = env::current_dir().unwrap();
        let fixture_buf = cwd
            .join("fixtures/suppression_with_prune_all_arg_and_no_file/oxlint-suppressions.json");
        let fixture_path = fixture_buf.to_str().unwrap();
        assert!(
            !fs::exists(fixture_path).unwrap(),
            "oxlint-suppression found in fixtures/suppression_with_prune_all_arg_and_no_file/oxlint-suppressions.json"
        );

        let args = &["--prune-suppressions"];
        Tester::new()
            .with_cwd("fixtures/suppression_with_prune_all_arg_and_no_file".into())
            .test(args);

        assert!(
            !fs::exists(fixture_path).unwrap(),
            "oxlint-suppression found in fixtures/suppression_with_prune_all_arg_and_no_file/oxlint-suppressions.json"
        );
    }

    #[test]
    fn test_suppression_with_suppress_all_arg_and_pruned_errors() {
        let cwd = env::current_dir().unwrap();
        let fixture_path =
            "fixtures/suppression_with_arg_and_pruned_errors/oxlint-suppressions.json";
        let fixture_buf = cwd.join(fixture_path);
        let expected_buf = cwd.join(
            "fixtures/suppression_with_arg_and_pruned_errors/oxlint-suppressions-expected.json",
        );
        let backup_buf = cwd.join(
            "fixtures/suppression_with_arg_and_pruned_errors/oxlint-suppressions-expected.json",
        );
        let args = &["--suppress-all"];

        Tester::new().with_cwd("fixtures/suppression_with_arg_and_pruned_errors".into()).test(args);

        let new_content = fs::read_to_string(fixture_buf)
            .expect("Unable to read the new oxlint-suppressions.json");
        let expected_content = fs::read_to_string(expected_buf)
            .expect("Unable to read the expected content oxlint-suppressions-expected.json");

        assert_eq!(
            new_content, expected_content,
            "The suppression generated doesn't match the expected"
        );

        fs::remove_file(cwd.join(fixture_path)).unwrap();
        fs::copy(backup_buf, cwd.join(fixture_path)).unwrap();
    }

    #[test]
    fn test_suppression_with_prune_suppressions_arg_and_pruned_errors() {
        let cwd = env::current_dir().unwrap();
        let fixture_path =
            "fixtures/suppression_with_arg_and_pruned_errors/oxlint-suppressions.json";
        let fixture_buf = cwd.join(fixture_path);
        let expected_buf = cwd.join(
            "fixtures/suppression_with_arg_and_pruned_errors/oxlint-suppressions-expected.json",
        );
        let backup_buf = cwd.join(
            "fixtures/suppression_with_arg_and_pruned_errors/oxlint-suppressions-expected.json",
        );
        let args = &["--prune-suppressions"];

        Tester::new().with_cwd("fixtures/suppression_with_arg_and_pruned_errors".into()).test(args);

        let new_content = fs::read_to_string(fixture_buf)
            .expect("Unable to read the new oxlint-suppressions.json");
        let expected_content = fs::read_to_string(expected_buf)
            .expect("Unable to read the expected content oxlint-suppressions-expected.json");

        assert_eq!(
            new_content, expected_content,
            "The suppression generated doesn't match the expected"
        );

        fs::remove_file(cwd.join(fixture_path)).unwrap();
        fs::copy(backup_buf, cwd.join(fixture_path)).unwrap();
    }

    #[test]
    fn test_suppression_with_suppress_all_arg_and_increased_errors() {
        let cwd = env::current_dir().unwrap();
        let fixture_path =
            "fixtures/suppression_with_arg_and_increased_errors/oxlint-suppressions.json";
        let fixture_buf = cwd.join(fixture_path);
        let expected_buf = cwd.join(
            "fixtures/suppression_with_arg_and_increased_errors/oxlint-suppressions-expected.json",
        );
        let backup_buf = cwd.join(
            "fixtures/suppression_with_arg_and_increased_errors/oxlint-suppressions-expected.json",
        );
        let args = &["--suppress-all"];

        Tester::new()
            .with_cwd("fixtures/suppression_with_arg_and_increased_errors".into())
            .test(args);

        let new_content = fs::read_to_string(fixture_buf)
            .expect("Unable to read the new oxlint-suppressions.json");
        let expected_content = fs::read_to_string(expected_buf)
            .expect("Unable to read the expected content oxlint-suppressions-expected.json");

        assert_eq!(
            new_content, expected_content,
            "The suppression generated doesn't match the expected"
        );

        fs::remove_file(cwd.join(fixture_path)).unwrap();
        fs::copy(backup_buf, cwd.join(fixture_path)).unwrap();
    }

    #[test]
    fn test_suppression_with_prune_suppressions_arg_and_increased_errors() {
        let cwd = env::current_dir().unwrap();
        let fixture_path =
            "fixtures/suppression_with_arg_and_increased_errors/oxlint-suppressions.json";
        let fixture_buf = cwd.join(fixture_path);
        let expected_buf = cwd.join(
            "fixtures/suppression_with_arg_and_increased_errors/oxlint-suppressions-expected.json",
        );
        let backup_buf = cwd.join(
            "fixtures/suppression_with_arg_and_increased_errors/oxlint-suppressions-expected.json",
        );
        let args = &["--prune-suppressions"];

        Tester::new()
            .with_cwd("fixtures/suppression_with_arg_and_increased_errors".into())
            .test(args);

        let new_content = fs::read_to_string(fixture_buf)
            .expect("Unable to read the new oxlint-suppressions.json");
        let expected_content = fs::read_to_string(expected_buf)
            .expect("Unable to read the expected content oxlint-suppressions-expected.json");

        assert_eq!(
            new_content, expected_content,
            "The suppression generated doesn't match the expected"
        );

        fs::remove_file(cwd.join(fixture_path)).unwrap();
        fs::copy(backup_buf, cwd.join(fixture_path)).unwrap();
    }

    #[test]
    fn test_suppression_with_suppress_all_arg_and_decreased_errors() {
        let cwd = env::current_dir().unwrap();
        let fixture_path =
            "fixtures/suppression_with_arg_and_decreased_errors/oxlint-suppressions.json";
        let fixture_buf = cwd.join(fixture_path);
        let expected_buf = cwd.join(
            "fixtures/suppression_with_arg_and_decreased_errors/oxlint-suppressions-expected.json",
        );
        let backup_buf = cwd.join(
            "fixtures/suppression_with_arg_and_decreased_errors/oxlint-suppressions-expected.json",
        );
        let args = &["--suppress-all"];

        Tester::new()
            .with_cwd("fixtures/suppression_with_arg_and_decreased_errors".into())
            .test(args);

        let new_content = fs::read_to_string(fixture_buf)
            .expect("Unable to read the new oxlint-suppressions.json");
        let expected_content = fs::read_to_string(expected_buf)
            .expect("Unable to read the expected content oxlint-suppressions-expected.json");

        assert_eq!(
            new_content, expected_content,
            "The suppression generated doesn't match the expected"
        );

        fs::remove_file(cwd.join(fixture_path)).unwrap();
        fs::copy(backup_buf, cwd.join(fixture_path)).unwrap();
    }

    #[test]
    fn test_suppression_with_prune_suppressions_arg_and_decreased_errors() {
        let cwd = env::current_dir().unwrap();
        let fixture_path =
            "fixtures/suppression_with_arg_and_decreased_errors/oxlint-suppressions.json";
        let fixture_buf = cwd.join(fixture_path);
        let expected_buf = cwd.join(
            "fixtures/suppression_with_arg_and_decreased_errors/oxlint-suppressions-expected.json",
        );
        let backup_buf = cwd.join(
            "fixtures/suppression_with_arg_and_decreased_errors/oxlint-suppressions-expected.json",
        );
        let args = &["--prune-suppressions"];

        Tester::new()
            .with_cwd("fixtures/suppression_with_arg_and_decreased_errors".into())
            .test(args);

        let new_content = fs::read_to_string(fixture_buf)
            .expect("Unable to read the new oxlint-suppressions.json");
        let expected_content = fs::read_to_string(expected_buf)
            .expect("Unable to read the expected content oxlint-suppressions-expected.json");

        assert_eq!(
            new_content, expected_content,
            "The suppression generated doesn't match the expected"
        );

        fs::remove_file(cwd.join(fixture_path)).unwrap();
        fs::copy(backup_buf, cwd.join(fixture_path)).unwrap();
    }
}
