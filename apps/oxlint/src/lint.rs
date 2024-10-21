use std::{env, io::BufWriter, time::Instant};

use ignore::gitignore::Gitignore;
use oxc_diagnostics::{DiagnosticService, GraphicalReportHandler};
use oxc_linter::{
    loader::LINT_PARTIAL_LOADER_EXT, AllowWarnDeny, InvalidFilterKind, LintFilter, LintService,
    LintServiceOptions, Linter, LinterBuilder, Oxlintrc,
};
use oxc_span::VALID_EXTENSIONS;

use crate::{
    cli::{
        CliRunResult, LintCommand, LintResult, MiscOptions, OutputFormat, OutputOptions, Runner,
        WarningOptions,
    },
    walk::{Extensions, Walk},
};

pub struct LintRunner {
    options: LintCommand,
}

impl Runner for LintRunner {
    type Options = LintCommand;

    fn new(options: Self::Options) -> Self {
        Self { options }
    }

    fn run(self) -> CliRunResult {
        if self.options.list_rules {
            let mut stdout = BufWriter::new(std::io::stdout());
            Linter::print_rules(&mut stdout);
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

            if let Ok(cwd) = env::current_dir() {
                paths.push(cwd);
            } else {
                return CliRunResult::InvalidOptions {
                    message: "Failed to get current working directory.".to_string(),
                };
            }
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

        let paths =
            Walk::new(&paths, &ignore_options).with_extensions(Extensions(extensions)).paths();

        let number_of_files = paths.len();

        let cwd = std::env::current_dir().unwrap();

        let mut oxlintrc = if let Some(config_path) = basic_options.config.as_ref() {
            match Oxlintrc::from_file(config_path) {
                Ok(config) => config,
                Err(diagnostic) => {
                    let handler = GraphicalReportHandler::new();
                    let mut err = String::new();
                    handler.render_report(&mut err, &diagnostic).unwrap();
                    return CliRunResult::InvalidOptions {
                        message: format!("Failed to parse configuration file.\n{err}"),
                    };
                }
            }
        } else {
            Oxlintrc::default()
        };

        enable_plugins.apply_overrides(&mut oxlintrc.plugins);
        let builder = LinterBuilder::from_oxlintrc(false, oxlintrc)
            .with_filters(filter)
            .with_fix(fix_options.fix_kind());

        let mut options =
            LintServiceOptions::new(cwd, paths).with_cross_module(builder.plugins().has_import());
        let linter = builder.build();

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
}

#[cfg(all(test, not(target_os = "windows")))]
mod test {
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
        assert_eq!(result.number_of_files, 2);
        assert_eq!(result.number_of_warnings, 2);
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
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 2);
        assert_eq!(result.number_of_errors, 0);
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
        assert_eq!(result.number_of_warnings, 2);
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
    fn test_enable_vitest_plugin() {
        let args = &[
            "-c",
            "fixtures/eslintrc_vitest_replace/eslintrc.json",
            "fixtures/eslintrc_vitest_replace/foo.test.js",
        ];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_errors, 0);

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
}
