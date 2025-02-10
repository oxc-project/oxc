#[cfg(test)]
use crate::cli::{lint_command, LintRunner};
#[cfg(test)]
use crate::runner::Runner;
#[cfg(test)]
use cow_utils::CowUtils;
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

        // disable multiple workers for diagnostic
        // because the snapshot could change every time when we are analyzing multiple files
        // do not unwrap because we can set it only one time.
        let _ = rayon::ThreadPoolBuilder::new().num_threads(1).build_global();

        Self { cwd }
    }

    pub fn with_cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd.push(cwd);
        self
    }

    pub fn test(&self, args: &[&str]) {
        let mut new_args = vec!["--silent"];
        new_args.extend(args);

        let options = lint_command().run_inner(new_args.as_slice()).unwrap();
        let mut output = Vec::new();
        let _ = LintRunner::new(options).with_cwd(self.cwd.clone()).run(&mut output);
    }

    pub fn test_and_snapshot(&self, args: &[&str]) {
        self.test_and_snapshot_multiple(&[args]);
    }

    pub fn test_and_snapshot_multiple(&self, multiple_args: &[&[&str]]) {
        let mut output: Vec<u8> = Vec::new();
        let current_cwd = std::env::current_dir().unwrap();
        let relative_dir = self.cwd.strip_prefix(&current_cwd).unwrap_or(&self.cwd);

        for args in multiple_args {
            let options = lint_command().run_inner(*args).unwrap();
            let args_string = args.join(" ");

            output.extend_from_slice("########## \n".as_bytes());
            output.extend_from_slice(format!("arguments: {args_string}\n").as_bytes());
            output.extend_from_slice(
                format!("working directory: {}\n", relative_dir.to_str().unwrap()).as_bytes(),
            );
            output.extend_from_slice("----------\n".as_bytes());
            let result = LintRunner::new(options).with_cwd(self.cwd.clone()).run(&mut output);

            output.extend_from_slice("----------\n".as_bytes());
            output.extend_from_slice(format!("CLI result: {result:?}\n").as_bytes());
            output.extend_from_slice("----------\n".as_bytes());

            output.push(b'\n');
        }

        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        settings.set_snapshot_suffix("oxlint");

        let regex = Regex::new(r"\d+ms").unwrap();

        let output_string = &String::from_utf8(output).unwrap();
        let output_string = regex.replace_all(output_string, "<variable>ms");

        // do not output the current working directory, each machine has a different one
        let cwd_string = current_cwd.to_str().unwrap();
        let cwd_string = cwd_string.cow_replace('\\', "/").to_string(); // for windows
        let output_string = output_string.cow_replace(&cwd_string, "<cwd>");

        let full_args_list =
            multiple_args.iter().map(|args| args.join(" ")).collect::<Vec<String>>().join(" ");

        let snapshot_file_name = format!("{}_{}", relative_dir.to_str().unwrap(), full_args_list);

        // windows can not handle filenames with *
        // allow replace instead of cow_replace. It only test
        let snapshot_file_name = snapshot_file_name.cow_replace('*', "_").to_string();
        settings.bind(|| {
            insta::assert_snapshot!(snapshot_file_name, output_string);
        });
    }
}
