use std::{env, fs, path::PathBuf};

use cow_utils::CowUtils;
use lazy_regex::Regex;
use serde_json::Value;

use crate::cli::{CliRunResult, CliRunner, lint_command};

pub struct Tester {
    cwd: PathBuf,
}

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
        let _ = CliRunner::new(options, None).with_cwd(self.cwd.clone()).run(&mut output);
    }

    pub fn test_output(&self, args: &[&str]) -> (String, CliRunResult) {
        let mut new_args = vec!["--silent"];
        new_args.extend(args);

        let options = lint_command().run_inner(new_args.as_slice()).unwrap();
        let mut output = Vec::new();
        let result = CliRunner::new(options, None).with_cwd(self.cwd.clone()).run(&mut output);
        (String::from_utf8(output).unwrap(), result)
    }

    pub fn test_output_verbose(&self, args: &[&str]) -> String {
        let options = lint_command().run_inner(args).unwrap();
        let mut output = Vec::new();
        let _ = CliRunner::new(options, None).with_cwd(self.cwd.clone()).run(&mut output);

        String::from_utf8(output).unwrap()
    }

    pub fn test_fix(file: &str, before: &str, after: &str) {
        Self::test_fix_with_args(file, before, after, &[]);
    }

    /// Test fix with additional CLI arguments (e.g., `--type-aware` for tsgolint)
    pub fn test_fix_with_args(file: &str, before: &str, after: &str, extra_args: &[&str]) {
        use std::fs;
        #[expect(clippy::disallowed_methods)]
        let content_original = fs::read_to_string(file).unwrap().replace("\r\n", "\n");
        assert_eq!(content_original, before);

        let mut args = vec!["--fix"];
        args.extend(extra_args);
        args.push(file);
        Tester::new().test(&args);

        #[expect(clippy::disallowed_methods)]
        let new_content = fs::read_to_string(file).unwrap().replace("\r\n", "\n");
        assert_eq!(new_content, after);

        Tester::new().test(&args);

        // File should not be modified if no fix is applied.
        let modified_before: std::time::SystemTime =
            fs::metadata(file).unwrap().modified().unwrap();
        let modified_after = fs::metadata(file).unwrap().modified().unwrap();
        assert_eq!(modified_before, modified_after);

        // Write the file back.
        fs::write(file, before).unwrap();
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

            output.extend_from_slice(b"########## \n");
            output.extend_from_slice(format!("arguments: {args_string}\n").as_bytes());
            output.extend_from_slice(
                format!("working directory: {}\n", relative_dir.to_str().unwrap()).as_bytes(),
            );
            output.extend_from_slice(b"----------\n");
            let result = CliRunner::new(options, None).with_cwd(self.cwd.clone()).run(&mut output);

            output.extend_from_slice(b"----------\n");
            output.extend_from_slice(format!("CLI result: {result:?}\n").as_bytes());
            output.extend_from_slice(b"----------\n");

            output.push(b'\n');
        }

        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        settings.set_snapshot_suffix("oxlint");

        let output_string = &String::from_utf8(output).unwrap();
        let regex = Regex::new(r"\d+(?:\.\d+)?s|\d+ms").unwrap();
        let output_string = regex.replace_all(output_string, "<variable>ms").into_owned();
        let regex = Regex::new(r#""start_time": \d+\.\d+"#).unwrap();
        let output_string = regex.replace_all(&output_string, r#""start_time": <variable>"#);
        // Censor the oxlint version in SARIF output so snapshots survive version bumps.
        let is_sarif = multiple_args
            .iter()
            .flat_map(|args| args.iter())
            .any(|arg| *arg == "--format=sarif" || *arg == "sarif");
        let output_string = if is_sarif {
            let regex =
                Regex::new(r#""version":\s*"[^"]+",(\s+)"semanticVersion":\s*"[^"]+""#).unwrap();
            regex.replace_all(
                &output_string,
                r#""version": "<variable>",${1}"semanticVersion": "<variable>""#,
            )
        } else {
            output_string
        };

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

/**
 * Some test for suppression required to handle files, and assert file contest. This struct exist only for:
 * - Automatic dispose: If a test fail or success, all the files used are going to be reverted to his original form.
 * - Save file on panic config: If a test is failing, we can keep the result. This file will be override each execution. JS files are no kept between execution, those will be always reverted.
 * - Hide the boilerplate code to setup this tests.
 *
 * The suppression fixtures have always the same structure:
 * /suppression_X
 * |-----files/
 * |--------- test.js (Required)
 * |--------- test-backup.js (Required) The file content that should be reverted after the test is done.
 * |-----oxlintrc.json
 * |-----oxlint-suppression.json (Optional)
 * |-----oxlint-suppression-expected.json (Optional) Be aware of adding an ending new line.
 * |-----oxlint-suppression-backup.json: Required if an initial file is present. Otherwise the file generated will be deleted.
 */
pub struct SuppressionTester {
    cwd: PathBuf,
    fixture_name: String,
    have_backup_file: bool,
    have_expected_file: bool,
    have_setup_file: bool,
    have_files_fixed: bool,
    oxlint_suppression_file_name: String,
}

impl SuppressionTester {
    pub fn new() -> Self {
        let cwd = env::current_dir().unwrap();

        // disable multiple workers for diagnostic
        // because the snapshot could change every time when we are analyzing multiple files
        // do not unwrap because we can set it only one time.
        let _ = rayon::ThreadPoolBuilder::new().num_threads(1).build_global();

        Self {
            cwd,
            fixture_name: String::new(),
            have_backup_file: false,
            have_expected_file: false,
            have_setup_file: false,
            have_files_fixed: false,
            oxlint_suppression_file_name: String::from("oxlint-suppressions.json"),
        }
    }

    pub fn with_cwd(mut self, fixture_name: &str) -> Self {
        self.cwd.push(format!("fixtures/suppression/{fixture_name}"));
        self.fixture_name = fixture_name.to_string();
        self
    }

    pub fn with_setup_file(mut self, have_setup_file: bool) -> Self {
        self.have_setup_file = have_setup_file;
        self
    }

    pub fn with_expected_file(mut self, have_expected_file: bool) -> Self {
        self.have_expected_file = have_expected_file;
        self
    }

    pub fn with_backup_file(mut self, have_backup_file: bool) -> Self {
        self.have_backup_file = have_backup_file;
        self
    }

    pub fn with_files_fixed(mut self, have_files_fixed: bool) -> Self {
        self.have_files_fixed = have_files_fixed;
        self
    }

    fn assert_message(&self, expected_file: bool) -> String {
        if expected_file {
            format!("{} not found in {}/", self.oxlint_suppression_file_name, self.fixture_name)
        } else {
            format!("{} found in {}/", self.oxlint_suppression_file_name, self.fixture_name)
        }
    }

    pub fn test(&self, args: &[&str]) {
        assert!(
            fs::exists(self.cwd.join::<&str>(self.oxlint_suppression_file_name.as_ref())).unwrap()
                == self.have_setup_file,
            "{}",
            self.assert_message(self.have_setup_file)
        );

        Tester::new().with_cwd(self.cwd.clone()).test(args);

        assert!(
            fs::exists(self.cwd.join::<&str>(self.oxlint_suppression_file_name.as_ref())).unwrap()
                == self.have_expected_file,
            "{}",
            self.assert_message(self.have_expected_file)
        );

        if self.have_expected_file {
            let new_content_string = fs::read_to_string(
                self.cwd.join::<&str>(self.oxlint_suppression_file_name.as_ref()),
            )
            .unwrap_or_else(|_| {
                panic!("Unable to read the new {}", self.oxlint_suppression_file_name)
            });
            let new_content: Value =
                serde_json::from_str(&new_content_string).unwrap_or_else(|_| {
                    panic!("Unable to deserialize the new {}", self.oxlint_suppression_file_name)
                });

            let expected_content_string = fs::read_to_string(
                self.cwd.join("oxlint-suppressions-expected.json"),
            )
            .expect("Unable to read the expected content oxlint-suppressions-expected.json");

            let expected_content: Value = serde_json::from_str(&expected_content_string).unwrap();

            assert_eq!(
                new_content, expected_content,
                "The suppression generated doesn't match the expected"
            );
        }
    }
}

impl Drop for SuppressionTester {
    fn drop(&mut self) {
        if self.have_expected_file {
            match fs::remove_file(self.cwd.join::<&str>(self.oxlint_suppression_file_name.as_ref()))
            {
                Ok(()) => {}
                Err(err) => panic!(
                    "Unable to delete the setup file in fixture {} with error {}",
                    self.cwd
                        .join::<&str>(self.oxlint_suppression_file_name.as_ref())
                        .to_string_lossy(),
                    err
                ),
            }
        }

        if self.have_backup_file {
            let oxlint_file_dest =
                self.cwd.join::<&str>(self.oxlint_suppression_file_name.as_ref());
            let oxlint_file_source = self.cwd.join("oxlint-suppressions-backup.json");
            match fs::copy(oxlint_file_source, oxlint_file_dest) {
                Ok(_) => {}
                Err(err) => panic!(
                    "Unable to replace the suppression setup with the backup in fixture {} with error {}",
                    self.cwd
                        .join::<&str>(self.oxlint_suppression_file_name.as_ref())
                        .to_string_lossy(),
                    err
                ),
            }
        }

        if self.have_files_fixed {
            let js_file_dest = self.cwd.join("files/test.ts");
            let js_file_source = self.cwd.join("files/test-backup.ts");
            match fs::copy(js_file_source, js_file_dest) {
                Ok(_) => {}
                Err(err) => panic!(
                    "Unable to replace the linted filed with the backup in fixture {} with error {}",
                    self.cwd
                        .join::<&str>(self.oxlint_suppression_file_name.as_ref())
                        .to_string_lossy(),
                    err
                ),
            }
        }
    }
}
