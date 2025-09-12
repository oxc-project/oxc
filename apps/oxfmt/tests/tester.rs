use std::{env, fs, path::PathBuf};

use cow_utils::CowUtils;
use lazy_regex::Regex;

use oxfmt::cli::{FormatRunner, format_command};

#[derive(Debug, Default)]
pub struct Tester {
    cwd: PathBuf,
}

impl Tester {
    /// Creates a new Tester instance.
    ///
    /// # Panics
    /// Panics if the current working directory cannot be determined.
    pub fn new() -> Self {
        let cwd = env::current_dir().unwrap();

        // disable multiple workers for diagnostic
        // because the snapshot could change every time when we are analyzing multiple files
        // do not unwrap because we can set it only one time.
        let _ = rayon::ThreadPoolBuilder::new().num_threads(1).build_global();

        Self { cwd }
    }

    /// Runs a test without creating a snapshot (for write mode tests).
    ///
    /// # Panics
    /// Panics if command parsing fails.
    fn test(args: &[&str]) {
        let command = format_command().run_inner(args).unwrap();
        let mut output = Vec::new();
        FormatRunner::new(command).run(&mut output);
    }

    /// Runs a single test case and creates a snapshot.
    pub fn test_and_snapshot(&self, args: &[&str]) {
        self.test_and_snapshot_multiple(&[args]);
    }

    /// Runs multiple test cases and creates snapshots.
    ///
    /// # Panics
    /// Panics if the current working directory cannot be determined or if command parsing fails.
    pub fn test_and_snapshot_multiple(&self, multiple_args: &[&[&str]]) {
        let mut output: Vec<u8> = Vec::new();
        let current_cwd = std::env::current_dir().unwrap();
        let relative_dir = self.cwd.strip_prefix(&current_cwd).unwrap_or(&self.cwd);

        for args in multiple_args {
            let options = format_command().run_inner(*args).unwrap();
            let args_string = args.join(" ");

            output.extend_from_slice(b"########## \n");
            output.extend_from_slice(format!("arguments: {args_string}\n").as_bytes());
            output.extend_from_slice(
                format!("working directory: {}\n", relative_dir.to_str().unwrap()).as_bytes(),
            );
            output.extend_from_slice(b"----------\n");
            let result = FormatRunner::new(options).run(&mut output);

            output.extend_from_slice(b"----------\n");
            output.extend_from_slice(format!("CLI result: {result:?}\n").as_bytes());
            output.extend_from_slice(b"----------\n");

            output.push(b'\n');
        }

        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        settings.set_snapshot_suffix("oxfmt");

        let output_string = &String::from_utf8(output).unwrap();
        let regex = Regex::new(r"\d+(?:\.\d+)?s|\d+ms").unwrap();
        let output_string = regex.replace_all(output_string, "<variable>ms").into_owned();

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

    /// Tests the write mode where files are actually modified.
    /// Similar to oxlint's test_fix method.
    ///
    /// # Panics
    /// Panics if the file content doesn't match expected values.
    pub fn test_write(file: &str, before: &str, after: &str) {
        // Write initial content to file
        fs::write(file, before).unwrap();

        // Run formatter with --write flag
        Self::test(&["-w", file]);

        // Verify file was modified correctly
        #[expect(clippy::disallowed_methods)]
        let new_content = fs::read_to_string(file).unwrap().replace("\r\n", "\n");
        assert_eq!(new_content, after, "Formatted file content doesn't match expected");

        let modified_before = fs::metadata(file).unwrap().modified().unwrap();
        // Run formatter again with --write flag
        Self::test(&["-w", file]);

        let modified_after = fs::metadata(file).unwrap().modified().unwrap();
        assert_eq!(
            modified_before, modified_after,
            "File should not be modified when already formatted"
        );

        // Clean up - restore original content for next test run
        fs::remove_file(file).unwrap();
    }
}
