#[cfg(test)]
use crate::cli::{lint_command, CliRunResult, LintResult, LintRunner};
#[cfg(test)]
use crate::runner::Runner;
#[cfg(test)]
use regex::Regex;
#[cfg(test)]
use std::{env, path::PathBuf};
#[cfg(test)]
pub struct Tester {
    cwd: PathBuf,
}

#[cfg(test)]
impl Tester {
    pub fn new() -> Self {
        let cwd = env::current_dir().unwrap();

        Self { cwd }
    }

    pub fn with_cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd.push(cwd);
        self
    }

    pub fn get_lint_result(&self, args: &[&str]) -> LintResult {
        let mut new_args = vec!["--silent"];
        new_args.extend(args);

        let options = lint_command().run_inner(new_args.as_slice()).unwrap();
        let mut output = Vec::new();
        match LintRunner::new(options).with_cwd(self.cwd.clone()).run(&mut output) {
            CliRunResult::LintResult(lint_result) => lint_result,
            other => panic!("{other:?}"),
        }
    }

    pub fn get_invalid_option_result(&self, args: &[&str]) -> String {
        let mut new_args = vec!["--silent"];
        new_args.extend(args);

        let options = lint_command().run_inner(new_args.as_slice()).unwrap();
        let mut output = Vec::new();
        match LintRunner::new(options).with_cwd(self.cwd.clone()).run(&mut output) {
            CliRunResult::InvalidOptions { message } => message,
            other => {
                panic!("Expected InvalidOptions, got {other:?}");
            }
        }
    }

    pub fn test_and_snapshot(&self, args: &[&str]) {
        let mut settings = insta::Settings::clone_current();

        let options = lint_command().run_inner(args).unwrap();
        let mut output: Vec<u8> = Vec::new();
        let args_string = args.join(" ");

        output.extend_from_slice(format!("########## \n{args_string}\n----------\n").as_bytes());
        let _ = LintRunner::new(options).with_cwd(self.cwd.clone()).run(&mut output);
        output.push(b'\n');

        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        settings.set_snapshot_suffix("oxlint");

        let regex = Regex::new(r"\d+ms|\d+ threads?").unwrap();

        let output_string = &String::from_utf8(output).unwrap();
        let output_string = regex.replace_all(output_string, "<variable>");

        settings.bind(|| {
            insta::assert_snapshot!(format!("{}", args_string), output_string);
        });
    }
}
