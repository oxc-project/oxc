use std::{
    fmt::Write,
    path::{Path, PathBuf},
};

use similar::TextDiff;
use walkdir::WalkDir;

use oxc_allocator::Allocator;
use oxc_formatter::{
    FormatOptions, Formatter, JsdocOptions, LineWidth, enable_jsx_source_type, get_parse_options,
};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn root() -> PathBuf {
    oxc_tasks_common::project_root().join("tasks").join("prettier_conformance")
}

fn fixtures_root() -> PathBuf {
    root().join("jsdoc").join("fixtures")
}

fn snap_root() -> PathBuf {
    root().join("jsdoc").join("snapshots")
}

pub struct JsdocTestRunner {
    filter: Option<String>,
    debug: bool,
}

impl JsdocTestRunner {
    pub fn new(filter: Option<String>, debug: bool) -> Self {
        Self { filter, debug }
    }

    /// # Panics
    pub fn run(&self) {
        let fixture_pairs = self.collect_fixture_pairs();

        if self.filter.is_some() {
            // Debug mode: run filtered tests and print diffs
            for (input_path, expected_path) in &fixture_pairs {
                self.test_fixture(input_path, expected_path, true);
            }
            return;
        }

        // Run all tests and generate report
        let mut total: u32 = 0;
        let mut passed: u32 = 0;
        let mut failed_reports = String::new();
        failed_reports.push_str("# Failed\n\n");
        failed_reports.push_str("| Fixture | Ratio |\n");
        failed_reports.push_str("| :------ | :---: |\n");

        for (input_path, expected_path) in &fixture_pairs {
            total += 1;
            let (is_pass, ratio) = self.test_fixture(input_path, expected_path, false);
            if is_pass {
                passed += 1;
            } else {
                let rel_path = input_path
                    .strip_prefix(fixtures_root())
                    .unwrap()
                    .to_string_lossy();
                writeln!(failed_reports, "| {rel_path} | {ratio:.2}% |").unwrap();
            }
        }

        let percentage =
            if total > 0 { (f64::from(passed) / f64::from(total)) * 100.0 } else { 0.0 };

        let summary = format!("jsdoc compatibility: {passed}/{total} ({percentage:.2}%)");

        println!("{summary}");

        let snapshot = format!("{summary}\n\n{failed_reports}");
        std::fs::create_dir_all(snap_root()).unwrap();
        std::fs::write(snap_root().join("jsdoc.snap.md"), snapshot).unwrap();
    }

    /// Walk fixtures directory, collect pairs of (input, expected_output).
    ///
    /// A fixture pair is a file `foo.{js,ts,jsx,tsx}` paired with
    /// `foo.output.{js,ts,jsx,tsx}` in the same directory.
    fn collect_fixture_pairs(&self) -> Vec<(PathBuf, PathBuf)> {
        let mut pairs = Vec::new();

        for entry in WalkDir::new(fixtures_root())
            .min_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
        {
            let path = entry.path();
            let name = path.file_name().unwrap().to_string_lossy();

            // Skip output files and non-source files
            if name.contains(".output.") || name.starts_with('.') {
                continue;
            }

            // Skip non-source files (options.json, etc.)
            let ext = path.extension().map(|e| e.to_string_lossy()).unwrap_or_default();
            if !matches!(ext.as_ref(), "js" | "ts" | "jsx" | "tsx") {
                continue;
            }

            // Check for matching output file
            let stem = path.file_stem().unwrap().to_string_lossy();
            let output_name = format!("{stem}.output.{ext}");
            let output_path = path.with_file_name(&output_name);

            if !output_path.exists() {
                continue;
            }

            // Apply filter
            if let Some(filter) = &self.filter
                && !path.to_string_lossy().contains(filter.as_str())
            {
                continue;
            }

            pairs.push((path.to_path_buf(), output_path));
        }

        pairs.sort_unstable();
        pairs
    }

    /// Test a single fixture.
    /// Returns (passed, similarity_ratio_percentage).
    fn test_fixture(
        &self,
        input_path: &Path,
        expected_path: &Path,
        print_diff: bool,
    ) -> (bool, f32) {
        let source_text = std::fs::read_to_string(input_path).unwrap();
        let expected = std::fs::read_to_string(expected_path).unwrap();

        let jsdoc_options = Self::load_jsdoc_options(input_path);

        let Some(actual) = Self::run_oxfmt(&source_text, input_path, &jsdoc_options) else {
            if self.debug || print_diff {
                println!(
                    "SKIP (parse error): {}",
                    input_path.strip_prefix(fixtures_root()).unwrap().to_string_lossy()
                );
            }
            return (false, 0.0);
        };

        let is_pass = actual == expected;
        let diff = TextDiff::from_lines(&expected, &actual);
        let ratio = diff.ratio() * 100.0;

        if print_diff || self.debug {
            let rel_path = input_path
                .strip_prefix(fixtures_root())
                .unwrap()
                .to_string_lossy();

            if is_pass {
                println!("PASS: {rel_path}");
            } else {
                println!("FAIL: {rel_path} (similarity: {ratio:.1}%)");
                oxc_tasks_common::print_text_diff(&diff);
                println!();
            }
        }

        let ratio = ratio as f32;
        (is_pass, ratio)
    }

    /// Load per-fixture JsdocOptions.
    /// Checks for a per-file sidecar `{stem}.options.json` first, then
    /// directory-level `options.json`. Falls back to default options.
    fn load_jsdoc_options(input_path: &Path) -> JsdocOptions {
        let dir = input_path.parent().unwrap();

        // Per-file options: e.g. 033-not-capitalizing-false.options.json
        let stem = input_path.file_stem().unwrap().to_string_lossy();
        let per_file_path = dir.join(format!("{stem}.options.json"));
        if per_file_path.exists() {
            return Self::parse_jsdoc_options(&per_file_path);
        }

        // Per-directory options: options.json
        let dir_options_path = dir.join("options.json");
        if dir_options_path.exists() {
            return Self::parse_jsdoc_options(&dir_options_path);
        }

        JsdocOptions::default()
    }

    fn parse_jsdoc_options(path: &Path) -> JsdocOptions {
        let content = std::fs::read_to_string(path).unwrap();
        let mut options = JsdocOptions::default();

        // Simple JSON parsing for known options
        if content.contains("\"capitalize_descriptions\": false") {
            options.capitalize_descriptions = false;
        }
        if content.contains("\"separate_tag_groups\": true") {
            options.separate_tag_groups = true;
        }
        if content.contains("\"separate_returns_from_param\": true") {
            options.separate_returns_from_param = true;
        }
        if content.contains("\"bracket_spacing\": true") {
            options.bracket_spacing = true;
        }

        options
    }

    fn run_oxfmt(source_text: &str, path: &Path, jsdoc_options: &JsdocOptions) -> Option<String> {
        let allocator = Allocator::default();

        let source_type = SourceType::from_path(path).unwrap_or_default();
        let source_type = enable_jsx_source_type(source_type);

        let ret = Parser::new(&allocator, source_text, source_type)
            .with_options(get_parse_options())
            .parse();
        if ret.panicked {
            return None;
        }

        // Use printWidth=80 to match Prettier's default (oxfmt defaults to 100)
        let options = FormatOptions {
            line_width: LineWidth::try_from(80).unwrap(),
            jsdoc: Some(jsdoc_options.clone()),
            ..FormatOptions::default()
        };
        let formatted = Formatter::new(&allocator, options).build(&ret.program);
        Some(formatted)
    }
}
