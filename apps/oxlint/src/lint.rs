use std::{
    env,
    io::BufWriter,
    path::{Path, PathBuf},
    time::Instant,
};

use ignore::gitignore::Gitignore;

use oxc_diagnostics::{DiagnosticService, GraphicalReportHandler};
use oxc_linter::{
    loader::LINT_PARTIAL_LOADER_EXT, AllowWarnDeny, ConfigStoreBuilder, InvalidFilterKind,
    LintFilter, LintOptions, LintService, LintServiceOptions, Linter, Oxlintrc,
};
use oxc_span::VALID_EXTENSIONS;

use crate::{
    cli::{
        CliRunResult, LintCommand, LintResult, MiscOptions, OutputOptions, Runner, WarningOptions,
    },
    output_formatter::{OutputFormat, OutputFormatter},
    walk::{Extensions, Walk},
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

    fn run(self) -> CliRunResult {
        let format_str = self.options.output_options.format;
        let output_formatter = OutputFormatter::new(format_str);

        if self.options.list_rules {
            let mut stdout = BufWriter::new(std::io::stdout());
            output_formatter.all_rules(&mut stdout);
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
            output_options,
            misc_options,
            ..
        } = self.options;

        let mut paths = paths;
        let provided_path_count = paths.len();
        let now = Instant::now();

        // The ignore crate whitelists explicit paths, but priority
        // should be given to the ignore file. Many users lint
        // automatically and pass a list of changed files explicitly.
        // To accommodate this, unless `--no-ignore` is passed,
        // pre-filter the paths.
        if !paths.is_empty() && !ignore_options.no_ignore {
            let (ignore, _err) = Gitignore::new(&ignore_options.ignore_path);
            paths.retain(|p| if p.is_dir() { true } else { !ignore.matched(p, false).is_ignore() });
        }

        // Append cwd to all paths
        paths = paths.into_iter().map(|x| self.cwd.join(x)).collect();

        if paths.is_empty() {
            // If explicit paths were provided, but all have been
            // filtered, return early.
            if provided_path_count > 0 {
                return CliRunResult::LintResult(LintResult {
                    duration: now.elapsed(),
                    deny_warnings: warning_options.deny_warnings,
                    ..LintResult::default()
                });
            }

            paths.push(self.cwd.clone());
        }

        let filter = match Self::get_filters(filter) {
            Ok(filter) => filter,
            Err(e) => return e,
        };

        let extensions = VALID_EXTENSIONS
            .iter()
            .chain(LINT_PARTIAL_LOADER_EXT.iter())
            .copied()
            .collect::<Vec<&'static str>>();

        let config_search_result =
            Self::find_oxlint_config(&self.cwd, basic_options.config.as_ref());

        if let Err(err) = config_search_result {
            return err;
        }

        let mut oxlintrc = config_search_result.unwrap();
        let oxlint_wd = oxlintrc.path.parent().unwrap_or(&self.cwd).to_path_buf();

        let paths = Walk::new(&oxlint_wd, &paths, &ignore_options, &oxlintrc.ignore_patterns)
            .with_extensions(Extensions(extensions))
            .paths();

        let number_of_files = paths.len();

        enable_plugins.apply_overrides(&mut oxlintrc.plugins);

        let oxlintrc_for_print =
            if misc_options.print_config { Some(oxlintrc.clone()) } else { None };
        let config_builder =
            ConfigStoreBuilder::from_oxlintrc(false, oxlintrc).with_filters(filter);

        if let Some(basic_config_file) = oxlintrc_for_print {
            return CliRunResult::PrintConfigResult {
                config_file: config_builder.resolve_final_config_file(basic_config_file),
            };
        }

        let mut options = LintServiceOptions::new(self.cwd, paths)
            .with_cross_module(config_builder.plugins().has_import());

        let lint_config = match config_builder.build() {
            Ok(config) => config,
            Err(diagnostic) => {
                let handler = GraphicalReportHandler::new();
                let mut err = String::new();
                handler.render_report(&mut err, &diagnostic).unwrap();
                return CliRunResult::InvalidOptions {
                    message: format!("Failed to parse configuration file.\n{err}"),
                };
            }
        };

        let linter =
            Linter::new(LintOptions::default(), lint_config).with_fix(fix_options.fix_kind());

        let tsconfig = basic_options.tsconfig;
        if let Some(path) = tsconfig.as_ref() {
            if path.is_file() {
                options = options.with_tsconfig(path);
            } else {
                let path = if path.is_relative() { options.cwd().join(path) } else { path.clone() };
                return CliRunResult::InvalidOptions {
                    message: format!(
                        "The tsconfig file {path:?} does not exist, Please provide a valid tsconfig file.",
                    ),
                };
            }
        }

        let lint_service = LintService::new(linter, options);
        let mut diagnostic_service =
            Self::get_diagnostic_service(&warning_options, &output_options, &misc_options);

        // Spawn linting in another thread so diagnostics can be printed immediately from diagnostic_service.run.
        rayon::spawn({
            let tx_error = diagnostic_service.sender().clone();
            let lint_service = lint_service.clone();
            move || {
                lint_service.run(&tx_error);
            }
        });
        diagnostic_service.run();

        CliRunResult::LintResult(LintResult {
            duration: now.elapsed(),
            number_of_rules: lint_service.linter().number_of_rules(),
            number_of_files,
            number_of_warnings: diagnostic_service.warnings_count(),
            number_of_errors: diagnostic_service.errors_count(),
            max_warnings_exceeded: diagnostic_service.max_warnings_exceeded(),
            deny_warnings: warning_options.deny_warnings,
            print_summary: matches!(output_options.format, OutputFormat::Default),
        })
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
        warning_options: &WarningOptions,
        output_options: &OutputOptions,
        misc_options: &MiscOptions,
    ) -> DiagnosticService {
        let mut diagnostic_service = DiagnosticService::default()
            .with_quiet(warning_options.quiet)
            .with_silent(misc_options.silent)
            .with_max_warnings(warning_options.max_warnings);

        match output_options.format {
            OutputFormat::Default => {}
            OutputFormat::Json => diagnostic_service.set_json_reporter(),
            OutputFormat::Unix => diagnostic_service.set_unix_reporter(),
            OutputFormat::Checkstyle => diagnostic_service.set_checkstyle_reporter(),
            OutputFormat::Github => diagnostic_service.set_github_reporter(),
        }
        diagnostic_service
    }

    // moved into a separate function for readability, but it's only ever used
    // in one place.
    fn get_filters(
        filters_arg: Vec<(AllowWarnDeny, String)>,
    ) -> Result<Vec<LintFilter>, CliRunResult> {
        let mut filters = Vec::with_capacity(filters_arg.len());

        for (severity, filter_arg) in filters_arg {
            match LintFilter::new(severity, filter_arg) {
                Ok(filter) => {
                    filters.push(filter);
                }
                Err(InvalidFilterKind::Empty) => {
                    return Err(CliRunResult::InvalidOptions {
                        message: format!("Cannot {severity} an empty filter."),
                    });
                }
                Err(InvalidFilterKind::PluginMissing(filter)) => {
                    return Err(CliRunResult::InvalidOptions {
                        message: format!(
                            "Failed to {severity} filter {filter}: Plugin name is missing. Expected <plugin>/<rule>"
                        ),
                    });
                }
                Err(InvalidFilterKind::RuleMissing(filter)) => {
                    return Err(CliRunResult::InvalidOptions {
                        message: format!(
                            "Failed to {severity} filter {filter}: Rule name is missing. Expected <plugin>/<rule>"
                        ),
                    });
                }
            }
        }

        Ok(filters)
    }

    // finds the oxlint config
    // when config is provided, but not found, an CliRunResult is returned, else the oxlintrc config file is returned
    // when no config is provided, it will search for the default file names in the current working directory
    // when no file is found, the default configuration is returned
    fn find_oxlint_config(cwd: &Path, config: Option<&PathBuf>) -> Result<Oxlintrc, CliRunResult> {
        if let Some(config_path) = config {
            let full_path = cwd.join(config_path);
            return match Oxlintrc::from_file(&full_path) {
                Ok(config) => Ok(config),
                Err(diagnostic) => {
                    let handler = GraphicalReportHandler::new();
                    let mut err = String::new();
                    handler.render_report(&mut err, &diagnostic).unwrap();
                    return Err(CliRunResult::InvalidOptions {
                        message: format!("Failed to parse configuration file.\n{err}"),
                    });
                }
            };
        }
        // no config argument is provided,
        // auto detect default config file from current work directory
        // or return the default configuration, when no valid file is found
        let config_path = cwd.join(Self::DEFAULT_OXLINTRC);
        Oxlintrc::from_file(&config_path).or_else(|_| Ok(Oxlintrc::default()))
    }
}

#[cfg(test)]
mod test {
    use std::{env, path::MAIN_SEPARATOR_STR};

    use super::LintRunner;
    use crate::cli::{lint_command, CliRunResult, LintResult, Runner};

    fn test(args: &[&str]) -> LintResult {
        let mut new_args = vec!["--silent"];
        new_args.extend(args);
        let options = lint_command().run_inner(new_args.as_slice()).unwrap();
        match LintRunner::new(options).run() {
            CliRunResult::LintResult(lint_result) => lint_result,
            other => panic!("{other:?}"),
        }
    }

    fn test_with_cwd(cwd: &str, args: &[&str]) -> LintResult {
        let mut new_args = vec!["--silent"];
        new_args.extend(args);

        let options = lint_command().run_inner(new_args.as_slice()).unwrap();

        let mut current_cwd = env::current_dir().unwrap();

        let part_cwd = if MAIN_SEPARATOR_STR == "/" {
            cwd.into()
        } else {
            #[expect(clippy::disallowed_methods)]
            cwd.replace('/', MAIN_SEPARATOR_STR)
        };

        current_cwd.push(part_cwd);

        match LintRunner::new(options).with_cwd(current_cwd).run() {
            CliRunResult::LintResult(lint_result) => lint_result,
            other => panic!("{other:?}"),
        }
    }

    fn test_invalid_options(args: &[&str]) -> String {
        let mut new_args = vec!["--quiet"];
        new_args.extend(args);
        let options = lint_command().run_inner(new_args.as_slice()).unwrap();
        match LintRunner::new(options).run() {
            CliRunResult::InvalidOptions { message } => message,
            other => {
                panic!("Expected InvalidOptions, got {other:?}");
            }
        }
    }

    #[test]
    fn no_arg() {
        let args = &[];
        let result = test(args);
        assert!(result.number_of_rules > 0);
        assert!(result.number_of_warnings > 0);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn dir() {
        let args = &["fixtures/linter"];
        let result = test(args);
        assert!(result.number_of_rules > 0);
        assert_eq!(result.number_of_files, 3);
        assert_eq!(result.number_of_warnings, 3);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn cwd() {
        let args = &["debugger.js"];
        let result = test_with_cwd("fixtures/linter", args);
        assert!(result.number_of_rules > 0);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 1);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn file() {
        let args = &["fixtures/linter/debugger.js"];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 1);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn multi_files() {
        let args = &["fixtures/linter/debugger.js", "fixtures/linter/nan.js"];
        let result = test(args);
        assert_eq!(result.number_of_files, 2);
        assert_eq!(result.number_of_warnings, 2);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn wrong_extension() {
        let args = &["foo.asdf"];
        let result = test(args);
        assert_eq!(result.number_of_files, 0);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn ignore_pattern() {
        let args =
            &["--ignore-pattern", "**/*.js", "--ignore-pattern", "**/*.vue", "fixtures/linter"];
        let result = test(args);
        assert_eq!(result.number_of_files, 0);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 0);
    }

    /// When a file is explicitly passed as a path and `--no-ignore`
    /// is not present, the ignore file should take precedence.
    /// See https://github.com/oxc-project/oxc/issues/1124
    #[test]
    fn ignore_file_overrides_explicit_args() {
        let args = &["--ignore-path", "fixtures/linter/.customignore", "fixtures/linter/nan.js"];
        let result = test(args);
        assert_eq!(result.number_of_files, 0);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn ignore_file_no_ignore() {
        let args = &[
            "--ignore-path",
            "fixtures/linter/.customignore",
            "--no-ignore",
            "fixtures/linter/nan.js",
        ];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 1);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn ignore_flow() {
        let args = &["--import-plugin", "fixtures/flow/index.mjs"];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    // https://github.com/oxc-project/oxc/issues/7406
    fn ignore_flow_import_plugin_directory() {
        let args = &["--import-plugin", "-A all", "-D no-cycle", "fixtures/flow/"];
        let result = test(args);
        assert_eq!(result.number_of_files, 2);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn filter_allow_all() {
        let args = &["-A", "all", "fixtures/linter"];
        let result = test(args);
        assert!(result.number_of_files > 0);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn filter_allow_one() {
        let args = &["-W", "correctness", "-A", "no-debugger", "fixtures/linter/debugger.js"];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn filter_error() {
        let args = &["-D", "correctness", "fixtures/linter/debugger.js"];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 1);
    }

    #[test]
    fn eslintrc_error() {
        let args = &["-c", "fixtures/linter/eslintrc.json", "fixtures/linter/debugger.js"];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 1);
    }

    #[test]
    fn eslintrc_off() {
        let args = &["-c", "fixtures/eslintrc_off/eslintrc.json", "fixtures/eslintrc_off/test.js"];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 1); // triggered by no_empty_file
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn oxlint_config_auto_detection() {
        let args = &["debugger.js"];
        let result = test_with_cwd("fixtures/auto_config_detection", args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 1);
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
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 1);
        assert_eq!(result.number_of_errors, 0);
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
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 1);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn eslintrc_with_env() {
        let args = &[
            "-c",
            "fixtures/eslintrc_env/eslintrc_env_browser.json",
            "fixtures/eslintrc_env/test.js",
        ];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 0);
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
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 0);
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
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 1);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn no_console_off() {
        let args =
            &["-c", "fixtures/no_console_off/eslintrc.json", "fixtures/no_console_off/test.js"];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn typescript_eslint() {
        let args = &[
            "-c",
            "fixtures/typescript_eslint/eslintrc.json",
            "fixtures/typescript_eslint/test.ts",
        ];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 3);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn typescript_eslint_off() {
        let args = &[
            "-c",
            "fixtures/typescript_eslint/eslintrc.json",
            "--disable-typescript-plugin",
            "fixtures/typescript_eslint/test.ts",
        ];
        test(args);
    }

    #[test]
    fn lint_vue_file() {
        let args = &["fixtures/vue/debugger.vue"];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 2);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn lint_empty_vue_file() {
        let args = &["fixtures/vue/empty.vue"];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn lint_astro_file() {
        let args = &["fixtures/astro/debugger.astro"];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 4);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn lint_svelte_file() {
        let args = &["fixtures/svelte/debugger.svelte"];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 1);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn test_tsconfig_option() {
        // passed
        test(&["--tsconfig", "fixtures/tsconfig/tsconfig.json"]);

        // failed
        assert!(test_invalid_options(&["--tsconfig", "oxc/tsconfig.json"])
            .contains("oxc/tsconfig.json\" does not exist, Please provide a valid tsconfig file."));
    }

    #[test]
    fn test_enable_vitest_rule_without_plugin() {
        let args = &[
            "-c",
            "fixtures/eslintrc_vitest_replace/eslintrc.json",
            "fixtures/eslintrc_vitest_replace/foo.test.js",
        ];
        test(args);
    }

    #[test]
    fn test_enable_vitest_plugin() {
        let args = &[
            "--vitest-plugin",
            "-c",
            "fixtures/eslintrc_vitest_replace/eslintrc.json",
            "fixtures/eslintrc_vitest_replace/foo.test.js",
        ];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_errors, 1);
    }

    #[test]
    fn test_import_plugin_enabled_in_config() {
        let args = &["-c", "fixtures/import/.oxlintrc.json", "fixtures/import/test.js"];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 1);
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
        let _ = test(args);
        #[expect(clippy::disallowed_methods)]
        let new_content = fs::read_to_string(file).unwrap().replace("\r\n", "\n");
        assert_eq!(new_content, "\n");

        // File should not be modified if no fix is applied.
        let modified_before = fs::metadata(file).unwrap().modified().unwrap();
        let _ = test(args);
        let modified_after = fs::metadata(file).unwrap().modified().unwrap();
        assert_eq!(modified_before, modified_after);

        // Write the file back.
        fs::write(file, content_original).unwrap();
    }

    #[test]
    fn test_print_config_ban_all_rules() {
        let args = &["-A", "all", "--print-config"];
        let options = lint_command().run_inner(args).unwrap();
        let ret = LintRunner::new(options).run();
        let CliRunResult::PrintConfigResult { config_file: config } = ret else {
            panic!("Expected PrintConfigResult, got {ret:?}")
        };

        #[expect(clippy::disallowed_methods)]
        let expect_json = std::fs::read_to_string("fixtures/print_config/normal/expect.json")
            .unwrap()
            .replace("\r\n", "\n");
        assert_eq!(config, expect_json.trim());
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
        let options = lint_command().run_inner(args).unwrap();
        let ret = LintRunner::new(options).run();
        let CliRunResult::PrintConfigResult { config_file: config } = ret else {
            panic!("Expected PrintConfigResult, got {ret:?}")
        };

        #[expect(clippy::disallowed_methods)]
        let expect_json = std::fs::read_to_string("fixtures/print_config/ban_rules/expect.json")
            .unwrap()
            .replace("\r\n", "\n");

        assert_eq!(config, expect_json.trim());
    }

    #[test]
    fn test_overrides() {
        let result =
            test(&["-c", "fixtures/overrides/.oxlintrc.json", "fixtures/overrides/test.js"]);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 1);

        let result =
            test(&["-c", "fixtures/overrides/.oxlintrc.json", "fixtures/overrides/test.ts"]);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 1);
        assert_eq!(result.number_of_errors, 1);

        let result =
            test(&["-c", "fixtures/overrides/.oxlintrc.json", "fixtures/overrides/other.jsx"]);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 1);
    }

    #[test]
    fn test_overrides_directories() {
        let result =
            test(&["-c", "fixtures/overrides/directories-config.json", "fixtures/overrides"]);
        assert_eq!(result.number_of_files, 7);
        assert_eq!(result.number_of_warnings, 2);
        assert_eq!(result.number_of_errors, 2);
    }

    #[test]
    fn test_config_ignore_patterns_extension() {
        let result = test(&[
            "-c",
            "fixtures/config_ignore_patterns/ignore_extension/eslintrc.json",
            "fixtures/config_ignore_patterns/ignore_extension",
        ]);
        assert_eq!(result.number_of_files, 1);
    }

    #[test]
    fn test_config_ignore_patterns_directory() {
        let result = test_with_cwd(
            "fixtures/config_ignore_patterns/ignore_directory",
            &["-c", "eslintrc.json"],
        );
        assert_eq!(result.number_of_files, 1);
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
        let result = test(args);
        assert_eq!(result.number_of_files, 0);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn test_jest_and_vitest_alias_rules() {
        let args = &[
            "-c",
            "fixtures/jest_and_vitest_alias_rules/oxlint-jest.json",
            "fixtures/jest_and_vitest_alias_rules/test.js",
        ];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 1);

        let args = &[
            "-c",
            "fixtures/jest_and_vitest_alias_rules/oxlint-vitest.json",
            "fixtures/jest_and_vitest_alias_rules/test.js",
        ];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 1);
    }

    #[test]
    fn test_eslint_and_typescript_alias_rules() {
        let args = &[
            "-c",
            "fixtures/eslint_and_typescript_alias_rules/oxlint-eslint.json",
            "fixtures/eslint_and_typescript_alias_rules/test.js",
        ];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 1);

        let args = &[
            "-c",
            "fixtures/eslint_and_typescript_alias_rules/oxlint-typescript.json",
            "fixtures/eslint_and_typescript_alias_rules/test.js",
        ];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 1);
    }
}
