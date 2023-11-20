use std::{env, io::BufWriter, path::Path, vec::Vec};

use oxc_diagnostics::DiagnosticService;
use oxc_linter::{LintOptions, LintService, Linter};

use crate::{
    codeowners, command::LintOptions as CliLintOptions, walk::Walk, CliRunResult, CodeownerOptions,
    LintResult, Runner,
};

pub struct LintRunner {
    options: CliLintOptions,
}

impl Runner for LintRunner {
    type Options = CliLintOptions;

    fn new(options: Self::Options) -> Self {
        Self { options }
    }

    fn run(self) -> CliRunResult {
        if self.options.misc_options.rules {
            let mut stdout = BufWriter::new(std::io::stdout());
            Linter::print_rules(&mut stdout);
            return CliRunResult::None;
        }

        let CliLintOptions {
            paths,
            filter,
            warning_options,
            ignore_options,
            fix_options,
            misc_options,
            codeowner_options,
            enable_plugins,
        } = self.options;

        let mut paths = paths;

        if paths.is_empty() {
            if let Ok(cwd) = env::current_dir() {
                paths.push(cwd);
            } else {
                return CliRunResult::InvalidOptions {
                    message: "Failed to get current working directory.".to_string(),
                };
            }
        }

        let now = std::time::Instant::now();

        let paths = Walk::new(&paths, &ignore_options).paths();

        let paths = match Self::apply_codeowners_file(&codeowner_options, paths) {
            Ok(new_paths) => new_paths,
            Err(err) => return err,
        };

        let number_of_files = paths.len();

        let cwd = std::env::current_dir().unwrap().into_boxed_path();
        let lint_options = LintOptions::default()
            .with_filter(filter)
            .with_fix(fix_options.fix)
            .with_timing(misc_options.timing)
            .with_import_plugin(enable_plugins.import_plugin)
            .with_jest_plugin(enable_plugins.jest_plugin)
            .with_jsx_a11y_plugin(enable_plugins.jsx_a11y_plugin);
        let lint_service = LintService::new(cwd, &paths, lint_options);

        let diagnostic_service = DiagnosticService::default()
            .with_quiet(warning_options.quiet)
            .with_max_warnings(warning_options.max_warnings);

        // Spawn linting in another thread so diagnostics can be printed immediately from diagnostic_service.run.
        rayon::spawn({
            let tx_error = diagnostic_service.sender().clone();
            let lint_service = lint_service.clone();
            move || {
                lint_service.run(&tx_error);
            }
        });
        diagnostic_service.run();

        lint_service.linter().print_execution_times_if_enable();

        CliRunResult::LintResult(LintResult {
            duration: now.elapsed(),
            number_of_rules: lint_service.linter().number_of_rules(),
            number_of_files,
            number_of_warnings: diagnostic_service.warnings_count(),
            number_of_errors: diagnostic_service.errors_count(),
            max_warnings_exceeded: diagnostic_service.max_warnings_exceeded(),
            deny_warnings: warning_options.deny_warnings,
        })
    }
}

impl LintRunner {
    fn apply_codeowners_file(
        options: &CodeownerOptions,
        paths: Vec<Box<Path>>,
    ) -> Result<Vec<Box<Path>>, CliRunResult> {
        if options.codeowners_file.is_some() && options.codeowners.is_empty() {
            return Err(CliRunResult::InvalidOptions {
                message: "No wanted codeowners provided.".to_string(),
            });
        }

        let maybe_codeowners_file = options.codeowners_file.as_ref().map(codeowners::from_path);

        if let Some(owners) = maybe_codeowners_file {
            return Ok(paths
                .into_iter()
                .filter(|path_being_checked| {
                    // Strips the prefix of "./", because paths will look like "./foo/bar.js"
                    // however owners.of() will not match against these relative paths.
                    // So instead we simply strp the prefix and check against "foo/bar.js".
                    let path_to_check = path_being_checked
                        .strip_prefix("./")
                        .unwrap_or(path_being_checked)
                        .to_path_buf();

                    owners.of(path_to_check).map_or(false, |owners_of_path| {
                        owners_of_path
                            .iter()
                            .map(|owner| match owner {
                                codeowners::Owner::Email(s)
                                | codeowners::Owner::Team(s)
                                | codeowners::Owner::Username(s) => s,
                            })
                            .any(|owner| options.codeowners.contains(owner))
                    })
                })
                .collect::<Vec<_>>());
        } else if options.codeowners_file.is_some() {
            return Err(CliRunResult::InvalidOptions {
                message: "Codeowners file could not be read or parsed.".to_string(),
            });
        }

        Ok(paths)
    }
}

#[cfg(all(test, not(target_os = "windows")))]
mod test {
    use super::LintRunner;
    use crate::{lint_command, CliRunResult, LintResult, Runner};

    fn test(args: &[&str]) -> LintResult {
        let mut new_args = vec!["--quiet"];
        new_args.extend(args);
        let options = lint_command().run_inner(new_args.as_slice()).unwrap().lint_options;
        let CliRunResult::LintResult(lint_result) = LintRunner::new(options).run() else {
            unreachable!()
        };
        lint_result
    }

    #[test]
    fn timing() {
        let args = &["--timing", "fixtures"];
        // make sure this doesn't crash
        test(args);
    }

    #[test]
    fn no_arg() {
        let args = &[];
        let result = test(args);
        assert!(result.number_of_rules > 0);
        assert_eq!(result.number_of_files, 2);
        assert_eq!(result.number_of_warnings, 2);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn dir() {
        let args = &["fixtures"];
        let result = test(args);
        assert!(result.number_of_rules > 0);
        assert_eq!(result.number_of_files, 2);
        assert_eq!(result.number_of_warnings, 2);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn file() {
        let args = &["fixtures/debugger.js"];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 1);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn multi_files() {
        let args = &["fixtures/debugger.js", "fixtures/nan.js"];
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
        let args = &["--ignore-pattern", "**/*.js", "fixtures"];
        let result = test(args);
        assert_eq!(result.number_of_files, 0);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn filter_allow_all() {
        let args = &["-A", "all", "fixtures"];
        let result = test(args);
        assert!(result.number_of_files > 0);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 0);
    }

    #[test]
    fn filter_allow_one() {
        let args = &["-D", "correctness", "-A", "no-debugger", "fixtures/debugger.js"];
        let result = test(args);
        assert_eq!(result.number_of_files, 1);
        assert_eq!(result.number_of_warnings, 0);
        assert_eq!(result.number_of_errors, 0);
    }
}
