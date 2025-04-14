use std::{
    env, fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf, absolute},
    time::Instant,
};

use cow_utils::CowUtils;
use ignore::{gitignore::Gitignore, overrides::OverrideBuilder};
use oxc_diagnostics::{DiagnosticService, GraphicalReportHandler, OxcDiagnostic};
use oxc_linter::{
    AllowWarnDeny, ConfigStore, ConfigStoreBuilder, InvalidFilterKind, LintFilter, LintOptions,
    LintService, LintServiceOptions, Linter, Oxlintrc,
};
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::Value;

use crate::{
    cli::{CliRunResult, LintCommand, MiscOptions, ReportUnusedDirectives, Runner, WarningOptions},
    output_formatter::{LintCommandInfo, OutputFormatter},
    walk::Walk,
};

#[derive(Debug)]
pub struct LintRunner {
    options: LintCommand,
    cwd: PathBuf,
}

impl Runner for LintRunner {
    type Options = LintCommand;

    fn new(options: Self::Options) -> Self {
        Self { options, cwd: env::current_dir().expect("Failed to get current working directory") }
    }

    fn run(self, stdout: &mut dyn Write) -> CliRunResult {
        let format_str = self.options.output_options.format;
        let output_formatter = OutputFormatter::new(format_str);

        if self.options.list_rules {
            if let Some(output) = output_formatter.all_rules() {
                stdout.write_all(output.as_bytes()).or_else(Self::check_for_writer_error).unwrap();
            }
            stdout.flush().unwrap();
            return CliRunResult::None;
        }

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
            ..
        } = self.options;

        let use_nested_config = !disable_nested_config &&
            // If the `--config` option is explicitly passed, we should not search for nested config files
            // as the passed config file takes absolute precedence.
            basic_options.config.is_none();

        let mut paths = paths;
        let provided_path_count = paths.len();
        let now = Instant::now();

        let filters = match Self::get_filters(filter) {
            Ok(filters) => filters,
            Err((result, message)) => {
                stdout.write_all(message.as_bytes()).or_else(Self::check_for_writer_error).unwrap();
                stdout.flush().unwrap();

                return result;
            }
        };

        let config_search_result =
            Self::find_oxlint_config(&self.cwd, basic_options.config.as_ref());

        if let Err(err) = config_search_result {
            stdout
                .write_all(format!("Failed to parse configuration file.\n{err}\n").as_bytes())
                .or_else(Self::check_for_writer_error)
                .unwrap();
            stdout.flush().unwrap();

            return CliRunResult::InvalidOptionConfig;
        }

        let mut oxlintrc = config_search_result.unwrap();
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
            if !oxlintrc.ignore_patterns.is_empty() {
                let oxlint_wd = oxlintrc.path.parent().unwrap_or(&self.cwd).to_path_buf();
                oxlintrc.ignore_patterns =
                    Self::adjust_ignore_patterns(&self.cwd, &oxlint_wd, oxlintrc.ignore_patterns);
                for pattern in &oxlintrc.ignore_patterns {
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
                    // Append cwd to all paths
                    let mut path = self.cwd.join(&p);

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
                    stdout.write_all(end.as_bytes()).or_else(Self::check_for_writer_error).unwrap();
                    stdout.flush().unwrap();
                }

                return CliRunResult::LintNoFilesFound;
            }

            paths.push(self.cwd.clone());
        }

        let walker = Walk::new(&paths, &ignore_options, override_builder);
        let paths = walker.paths();

        let number_of_files = paths.len();

        // TODO(perf): benchmark whether or not it is worth it to store the configurations on a
        // per-file or per-directory basis, to avoid calling `.parent()` on every path.
        let mut nested_oxlintrc = FxHashMap::<&Path, Oxlintrc>::default();
        let mut nested_configs = FxHashMap::<PathBuf, ConfigStore>::default();

        if use_nested_config {
            // get all of the unique directories among the paths to use for search for
            // oxlint config files in those directories and their ancestors
            // e.g. `/some/file.js` will check `/some` and `/`
            //      `/some/other/file.js` will check `/some/other`, `/some`, and `/`
            let mut directories = FxHashSet::default();
            for path in &paths {
                let path = Path::new(path);
                // Start from the file's parent directory and walk up the tree
                let mut current = path.parent();
                while let Some(dir) = current {
                    // NOTE: Initial benchmarking showed that it was faster to iterate over the directories twice
                    // rather than constructing the configs in one iteration. It's worth re-benchmarking that though.
                    directories.insert(dir);
                    current = dir.parent();
                }
            }
            for directory in directories {
                if let Ok(config) = Self::find_oxlint_config_in_directory(directory) {
                    nested_oxlintrc.insert(directory, config);
                }
            }

            // iterate over each config and build the ConfigStore
            for (dir, oxlintrc) in nested_oxlintrc {
                // TODO(refactor): clean up all of the error handling in this function
                let builder = match ConfigStoreBuilder::from_oxlintrc(false, oxlintrc) {
                    Ok(builder) => builder,
                    Err(e) => {
                        let handler = GraphicalReportHandler::new();
                        let mut err = String::new();
                        handler
                            .render_report(&mut err, &OxcDiagnostic::error(e.to_string()))
                            .unwrap();
                        stdout
                            .write_all(
                                format!("Failed to parse configuration file.\n{err}\n").as_bytes(),
                            )
                            .or_else(Self::check_for_writer_error)
                            .unwrap();
                        stdout.flush().unwrap();

                        return CliRunResult::InvalidOptionConfig;
                    }
                }
                .with_filters(&filters);

                match builder.build() {
                    Ok(config) => nested_configs.insert(dir.to_path_buf(), config),
                    Err(diagnostic) => {
                        let handler = GraphicalReportHandler::new();
                        let mut err = String::new();
                        handler.render_report(&mut err, &diagnostic).unwrap();
                        stdout
                            .write_all(
                                format!("Failed to parse configuration file.\n{err}\n").as_bytes(),
                            )
                            .or_else(Self::check_for_writer_error)
                            .unwrap();
                        stdout.flush().unwrap();

                        return CliRunResult::InvalidOptionConfig;
                    }
                };
            }
        }

        enable_plugins.apply_overrides(&mut oxlintrc.plugins);

        let oxlintrc_for_print = if misc_options.print_config || basic_options.init {
            Some(oxlintrc.clone())
        } else {
            None
        };
        let config_builder = match ConfigStoreBuilder::from_oxlintrc(false, oxlintrc) {
            Ok(builder) => builder,
            Err(e) => {
                let handler = GraphicalReportHandler::new();
                let mut err = String::new();
                handler.render_report(&mut err, &OxcDiagnostic::error(e.to_string())).unwrap();
                stdout
                    .write_all(format!("Failed to parse configuration file.\n{err}\n").as_bytes())
                    .or_else(Self::check_for_writer_error)
                    .unwrap();
                stdout.flush().unwrap();

                return CliRunResult::InvalidOptionConfig;
            }
        }
        .with_filters(&filters);

        if let Some(basic_config_file) = oxlintrc_for_print {
            let config_file = config_builder.resolve_final_config_file(basic_config_file);
            if misc_options.print_config {
                stdout
                    .write_all(config_file.as_bytes())
                    .or_else(Self::check_for_writer_error)
                    .unwrap();
                stdout.write_all(b"\n").or_else(Self::check_for_writer_error).unwrap();
                stdout.flush().unwrap();

                return CliRunResult::PrintConfigResult;
            } else if basic_options.init {
                let schema_relative_path = "node_modules/oxlint/configuration_schema.json";
                let configuration = if self.cwd.join(schema_relative_path).is_file() {
                    let mut config_json: Value = serde_json::from_str(&config_file).unwrap();
                    if let Value::Object(ref mut obj) = config_json {
                        let mut json_object = serde_json::Map::new();
                        json_object.insert(
                            "$schema".to_string(),
                            format!("./{schema_relative_path}").into(),
                        );
                        json_object.extend(obj.clone());
                        *obj = json_object;
                    }
                    serde_json::to_string_pretty(&config_json).unwrap()
                } else {
                    config_file
                };

                if fs::write(Self::DEFAULT_OXLINTRC, configuration).is_ok() {
                    stdout
                        .write_all(b"Configuration file created\n")
                        .or_else(Self::check_for_writer_error)
                        .unwrap();
                    stdout.flush().unwrap();
                    return CliRunResult::ConfigFileInitSucceeded;
                }

                // failed case
                stdout
                    .write_all("Failed to create configuration file\n".as_bytes())
                    .or_else(Self::check_for_writer_error)
                    .unwrap();
                stdout.flush().unwrap();
                return CliRunResult::ConfigFileInitFailed;
            }
        }

        // TODO(refactor): pull this into a shared function, so that the language server can use
        // the same functionality.
        let use_cross_module = if use_nested_config {
            nested_configs.values().any(|config| config.plugins().has_import())
        } else {
            config_builder.plugins().has_import()
        };
        let mut options =
            LintServiceOptions::new(self.cwd, paths).with_cross_module(use_cross_module);

        let lint_config = match config_builder.build() {
            Ok(config) => config,
            Err(diagnostic) => {
                let handler = GraphicalReportHandler::new();
                let mut err = String::new();
                handler.render_report(&mut err, &diagnostic).unwrap();
                stdout
                    .write_all(format!("Failed to parse configuration file.\n{err}\n").as_bytes())
                    .or_else(Self::check_for_writer_error)
                    .unwrap();
                stdout.flush().unwrap();

                return CliRunResult::InvalidOptionConfig;
            }
        };

        let report_unused_directives = match inline_config_options.report_unused_directives {
            ReportUnusedDirectives::WithoutSeverity(true) => Some(AllowWarnDeny::Warn),
            ReportUnusedDirectives::WithSeverity(Some(severity)) => Some(severity),
            _ => None,
        };

        let linter = if use_nested_config {
            Linter::new_with_nested_configs(LintOptions::default(), lint_config, nested_configs)
                .with_fix(fix_options.fix_kind())
                .with_report_unused_directives(report_unused_directives)
        } else {
            Linter::new(LintOptions::default(), lint_config)
                .with_fix(fix_options.fix_kind())
                .with_report_unused_directives(report_unused_directives)
        };

        let tsconfig = basic_options.tsconfig;
        if let Some(path) = tsconfig.as_ref() {
            if path.is_file() {
                options = options.with_tsconfig(path);
            } else {
                let path = if path.is_relative() { options.cwd().join(path) } else { path.clone() };
                stdout.write_all(format!(
                    "The tsconfig file {:?} does not exist, Please provide a valid tsconfig file.\n",
                    path.to_string_lossy().cow_replace('\\', "/")
                ).as_bytes()).or_else(Self::check_for_writer_error).unwrap();
                stdout.flush().unwrap();

                return CliRunResult::InvalidOptionTsConfig;
            }
        }

        let mut lint_service = LintService::new(linter, options);
        let mut diagnostic_service =
            Self::get_diagnostic_service(&output_formatter, &warning_options, &misc_options);

        let number_of_rules = lint_service.linter().number_of_rules();

        // Spawn linting in another thread so diagnostics can be printed immediately from diagnostic_service.run.
        rayon::spawn({
            let tx_error = diagnostic_service.sender().clone();
            move || {
                lint_service.run(&tx_error);
            }
        });

        let diagnostic_result = diagnostic_service.run(stdout);

        if let Some(end) = output_formatter.lint_command_info(&LintCommandInfo {
            number_of_files,
            number_of_rules,
            threads_count: rayon::current_num_threads(),
            start_time: now.elapsed(),
        }) {
            stdout.write_all(end.as_bytes()).or_else(Self::check_for_writer_error).unwrap();
            stdout.flush().unwrap();
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

impl LintRunner {
    const DEFAULT_OXLINTRC: &'static str = ".oxlintrc.json";

    #[must_use]
    pub fn with_cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = cwd;
        self
    }

    fn get_diagnostic_service(
        reporter: &OutputFormatter,
        warning_options: &WarningOptions,
        misc_options: &MiscOptions,
    ) -> DiagnosticService {
        DiagnosticService::new(reporter.get_diagnostic_reporter())
            .with_quiet(warning_options.quiet)
            .with_silent(misc_options.silent)
            .with_max_warnings(warning_options.max_warnings)
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

    // finds the oxlint config
    // when config is provided, but not found, an String with the formatted error is returned, else the oxlintrc config file is returned
    // when no config is provided, it will search for the default file names in the current working directory
    // when no file is found, the default configuration is returned
    fn find_oxlint_config(cwd: &Path, config: Option<&PathBuf>) -> Result<Oxlintrc, String> {
        if let Some(config_path) = config {
            let full_path = match absolute(cwd.join(config_path)) {
                Ok(path) => path,
                Err(e) => {
                    let handler = GraphicalReportHandler::new();
                    let mut err = String::new();
                    handler
                        .render_report(
                            &mut err,
                            &OxcDiagnostic::error(format!(
                                "Failed to resolve config path {}: {e}",
                                config_path.display()
                            )),
                        )
                        .unwrap();
                    return Err(err);
                }
            };
            return match Oxlintrc::from_file(&full_path) {
                Ok(config) => Ok(config),
                Err(diagnostic) => {
                    let handler = GraphicalReportHandler::new();
                    let mut err = String::new();
                    handler.render_report(&mut err, &diagnostic).unwrap();
                    return Err(err);
                }
            };
        }
        // no config argument is provided,
        // auto detect default config file from current work directory
        // or return the default configuration, when no valid file is found
        let config_path = cwd.join(Self::DEFAULT_OXLINTRC);
        Oxlintrc::from_file(&config_path).or_else(|_| Ok(Oxlintrc::default()))
    }

    /// Looks in a directory for an oxlint config file, returns the oxlint config if it exists
    /// and returns `Err` if none exists or the file is invalid. Does not apply the default
    /// config file.
    fn find_oxlint_config_in_directory(dir: &Path) -> Result<Oxlintrc, String> {
        let possible_config_path = dir.join(Self::DEFAULT_OXLINTRC);
        if possible_config_path.is_file() {
            Oxlintrc::from_file(&possible_config_path).map_err(|e| {
                let handler = GraphicalReportHandler::new();
                let mut err = String::new();
                handler.render_report(&mut err, &e).unwrap();
                err
            })
        } else {
            // TODO: Better error handling here.
            Err("No oxlint config file found".to_string())
        }
    }

    fn check_for_writer_error(error: std::io::Error) -> Result<(), std::io::Error> {
        // Do not panic when the process is killed (e.g. piping into `less`).
        if matches!(error.kind(), ErrorKind::Interrupted | ErrorKind::BrokenPipe) {
            Ok(())
        } else {
            Err(error)
        }
    }

    fn adjust_ignore_patterns(
        base: &PathBuf,
        path: &PathBuf,
        ignore_patterns: Vec<String>,
    ) -> Vec<String> {
        if base == path {
            ignore_patterns
        } else {
            let relative_ignore_path =
                path.strip_prefix(base).map_or_else(|_| PathBuf::from("."), Path::to_path_buf);

            ignore_patterns
                .into_iter()
                .map(|pattern| {
                    let prefix_len = pattern.chars().take_while(|&c| c == '!').count();
                    let (prefix, pattern) = pattern.split_at(prefix_len);

                    let adjusted_path = relative_ignore_path.join(pattern);
                    format!("{prefix}{}", adjusted_path.to_string_lossy().cow_replace('\\', "/"))
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod test {
    use std::{fs, path::PathBuf};

    use super::LintRunner;
    use crate::tester::Tester;

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
        Tester::new().test(&["--tsconfig", "fixtures/tsconfig/tsconfig.json"]);

        // failed
        Tester::new().test_and_snapshot(&["--tsconfig", "oxc/tsconfig.json"]);
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
        use std::fs;
        let file = "fixtures/linter/fix.js";
        let args = &["--fix", file];
        let content_original = fs::read_to_string(file).unwrap();
        #[expect(clippy::disallowed_methods)]
        let content = content_original.replace("\r\n", "\n");
        assert_eq!(&content, "debugger\n");

        // Apply fix to the file.
        Tester::new().test(args);
        #[expect(clippy::disallowed_methods)]
        let new_content = fs::read_to_string(file).unwrap().replace("\r\n", "\n");
        assert_eq!(new_content, "\n");

        // File should not be modified if no fix is applied.
        let modified_before: std::time::SystemTime =
            fs::metadata(file).unwrap().modified().unwrap();
        Tester::new().test(args);
        let modified_after = fs::metadata(file).unwrap().modified().unwrap();
        assert_eq!(modified_before, modified_after);

        // Write the file back.
        fs::write(file, content_original).unwrap();
    }

    #[test]
    fn test_print_config_ban_all_rules() {
        let args = &["-A", "all", "--print-config"];
        Tester::new().test_and_snapshot(args);
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
        assert!(!fs::exists(LintRunner::DEFAULT_OXLINTRC).unwrap());

        let args = &["--init"];
        Tester::new().test(args);

        assert!(fs::exists(LintRunner::DEFAULT_OXLINTRC).unwrap());

        fs::remove_file(LintRunner::DEFAULT_OXLINTRC).unwrap();
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
    fn test_two_rules_with_same_rule_name_from_different_plugins() {
        // Issue: <https://github.com/oxc-project/oxc/issues/8485>
        let args = &["-c", ".oxlintrc.json", "test.js"];
        Tester::new()
            .with_cwd("fixtures/two_rules_with_same_rule_name".into())
            .test_and_snapshot(args);
    }

    #[test]
    fn test_report_unused_directives() {
        let args = &["-c", ".oxlintrc.json", "--report-unused-disable-directives", "test.js"];

        Tester::new().with_cwd("fixtures/report_unused_directives".into()).test_and_snapshot(args);
    }

    #[test]
    fn test_adjust_ignore_patterns() {
        let base = PathBuf::from("/project/root");
        let path = PathBuf::from("/project/root/src");
        let ignore_patterns =
            vec![String::from("target"), String::from("!dist"), String::from("!!dist")];

        let adjusted_patterns = LintRunner::adjust_ignore_patterns(&base, &path, ignore_patterns);

        assert_eq!(
            adjusted_patterns,
            vec![String::from("src/target"), String::from("!src/dist"), String::from("!!src/dist")]
        );
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
    fn test_config_path_with_parent_references() {
        let cwd = std::env::current_dir().unwrap();

        // Test case 1: Invalid path that should fail
        let invalid_config = PathBuf::from("child/../../fixtures/linter/eslintrc.json");
        let result = LintRunner::find_oxlint_config(&cwd, Some(&invalid_config));
        assert!(result.is_err(), "Expected config lookup to fail with invalid path");

        // Test case 2: Valid path that should pass
        let valid_config = PathBuf::from("fixtures/linter/eslintrc.json");
        let result = LintRunner::find_oxlint_config(&cwd, Some(&valid_config));
        assert!(result.is_ok(), "Expected config lookup to succeed with valid path");

        // Test case 3: Valid path using parent directory (..) syntax that should pass
        let valid_parent_config = PathBuf::from("fixtures/linter/../linter/eslintrc.json");
        let result = LintRunner::find_oxlint_config(&cwd, Some(&valid_parent_config));
        assert!(result.is_ok(), "Expected config lookup to succeed with parent directory syntax");

        // Verify the resolved path is correct
        if let Ok(config) = result {
            assert_eq!(
                config.path.file_name().unwrap().to_str().unwrap(),
                "eslintrc.json",
                "Config file name should be preserved after path resolution"
            );
        }
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
}
