use convert_case::{Boundary, Case, Converter};
use cow_utils::CowUtils;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Clone)]
pub struct FilenameCase {
    /// Whether kebab case is allowed.
    kebab_case: bool,
    /// Whether camel case is allowed.
    camel_case: bool,
    /// Whether snake case is allowed.
    snake_case: bool,
    /// Whether pascal case is allowed.
    pascal_case: bool,
}

impl Default for FilenameCase {
    fn default() -> Self {
        Self { kebab_case: true, camel_case: false, snake_case: false, pascal_case: false }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces specific case styles for filenames. By default, kebab case is enforced.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent file naming conventions make it harder to locate files, navigate projects, and enforce
    /// consistency across a codebase. Standardizing naming conventions improves readability, reduces cognitive
    /// overhead, and aligns with best practices in large-scale development.
    ///
    /// ### Cases
    ///
    /// Examples of **correct** filenames for each case:
    ///
    /// #### `kebabCase`
    ///
    /// - `some-file-name.js`
    /// - `some-file-name.test.js`
    /// - `some-file-name.test-utils.js`
    ///
    /// #### `camelCase`
    ///
    /// - `someFileName.js`
    /// - `someFileName.test.js`
    /// - `someFileName.testUtils.js`
    ///
    /// #### `snakeCase`
    ///
    /// - `some_file_name.js`
    /// - `some_file_name.test.js`
    /// - `some_file_name.test_utils.js`
    ///
    /// #### `pascalCase`
    ///
    /// - `SomeFileName.js`
    /// - `SomeFileName.Test.js`
    /// - `SomeFileName.TestUtils.js`
    ///
    /// ### Options
    ///
    /// Use `kebabCase` as the default option.
    ///
    /// #### case
    ///
    /// `{ type: 'kebabCase' | 'camelCase' | 'snakeCase' | 'pascalCase' }`
    ///
    /// You can set the case option like this:
    /// ```json
    /// "unicorn/filename-case": [
    ///   "error",
    ///   {
    ///     "case": "kebabCase"
    ///   }
    /// ]
    /// ```
    ///
    /// #### cases
    ///
    /// `{ type: { [key in 'kebabCase' | 'camelCase' | 'snakeCase' | 'pascalCase']?: boolean } }`
    ///
    /// You can set the case option like this:
    /// ```json
    /// "unicorn/filename-case": [
    ///   "error",
    ///   {
    ///     "cases": {
    ///       "camelCase": true,
    ///       "pascalCase": true
    ///     }
    ///   }
    /// ]
    /// ```
    FilenameCase,
    unicorn,
    style
);

impl Rule for FilenameCase {
    fn from_configuration(value: serde_json::Value) -> Self {
        let Some(case_type) = value.get(0) else {
            return Self::default();
        };

        let off =
            Self { kebab_case: false, camel_case: false, snake_case: false, pascal_case: false };

        if let Some(Value::String(s)) = case_type.get("case") {
            return match s.as_str() {
                "kebabCase" => Self { kebab_case: true, ..off },
                "camelCase" => Self { camel_case: true, ..off },
                "snakeCase" => Self { snake_case: true, ..off },
                "pascalCase" => Self { pascal_case: true, ..off },
                _ => Self::default(),
            };
        }

        if let Some(Value::Object(map)) = case_type.get("cases") {
            let mut filename_case = off;
            for (key, value) in map {
                match (key.as_str(), value) {
                    ("kebabCase", Value::Bool(b)) => filename_case.kebab_case = *b,
                    ("camelCase", Value::Bool(b)) => filename_case.camel_case = *b,
                    ("snakeCase", Value::Bool(b)) => filename_case.snake_case = *b,
                    ("pascalCase", Value::Bool(b)) => filename_case.pascal_case = *b,
                    _ => (),
                }
            }
            return filename_case;
        }

        Self::default()
    }

    fn run_once<'a>(&self, ctx: &LintContext<'_>) {
        let file_path = ctx.file_path();
        let Some(filename) = file_path.file_stem().and_then(|s| s.to_str()) else {
            return;
        };

        // get filename up to the first dot, or the whole filename if there is no dot
        let filename = filename.split('.').next().unwrap_or(filename);
        // ignore all leading and trailing underscores
        let filename = filename.trim_matches('_');

        let cases = [
            (self.camel_case, Case::Camel, "camel case"),
            (self.kebab_case, Case::Kebab, "kebab case"),
            (self.snake_case, Case::Snake, "snake case"),
            (self.pascal_case, Case::Pascal, "pascal case"),
        ];

        let mut valid = Vec::new();
        for (enabled, case, name) in cases {
            if enabled {
                let converter = Converter::new()
                    .remove_boundaries(&[Boundary::LOWER_DIGIT, Boundary::DIGIT_LOWER]);
                let converter = converter.to_case(case);

                if converter.convert(filename) == filename {
                    return;
                }

                valid.push((converter, name));
            }
        }

        let filename = file_path.file_name().unwrap().to_string_lossy();
        ctx.diagnostic(filename_case_diagnostic(&filename, valid));
    }
}

fn filename_case_diagnostic(filename: &str, valid_cases: Vec<(Converter, &str)>) -> OxcDiagnostic {
    let trimmed_filename = filename.trim_matches('_');
    let valid_cases_len = valid_cases.len();

    let mut message = String::from("Filename should be in ");
    let mut help_message = String::from("Rename the file to ");

    for (i, (converter, name)) in valid_cases.into_iter().enumerate() {
        let filename = format!(
            "'{}'",
            filename.cow_replace(trimmed_filename, &converter.convert(trimmed_filename))
        );

        let (name, filename) = if i == 0 {
            (name, filename.as_ref())
        } else if i == valid_cases_len - 1 {
            (&*format!(", or {name}"), &*format!(", or {filename}"))
        } else {
            (&*format!(", {name}"), &*format!(", {filename}"))
        };

        message.push_str(name);
        help_message.push_str(filename);
    }

    OxcDiagnostic::warn(message).with_label(Span::default()).with_help(help_message)
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    fn test_case(
        path: &'static str,
        casing: &'static str,
    ) -> (&'static str, Option<Value>, Option<Value>, Option<PathBuf>) {
        (
            "",
            if casing.is_empty() {
                None
            } else {
                Some(serde_json::json!(
                    [{ "case": casing }]
                ))
            },
            None,
            Some(PathBuf::from(path)),
        )
    }

    fn test_cases<const N: usize>(
        path: &'static str,
        casings: [&'static str; N],
    ) -> (&'static str, Option<Value>, Option<Value>, Option<PathBuf>) {
        (
            "",
            if casings.is_empty() {
                None
            } else {
                let mut map = serde_json::Map::new();
                // turn ["camelCase", "snakeCase"] into [{ "cases": { "camelCase": true, "snakeCase": true } }]
                for casing in casings {
                    map.insert(casing.to_string(), Value::Bool(true));
                }
                Some(serde_json::json!([{ "cases": map }]))
            },
            None,
            Some(PathBuf::from(path)),
        )
    }

    let pass = vec![
        test_cases("src/foo/fooBar.js", ["camelCase"]),
        test_case("src/foo/bar.js", "camelCase"),
        test_case("src/foo/fooBar.js", "camelCase"),
        test_case("src/foo/bar.test.js", "camelCase"),
        test_case("src/foo/fooBar.test.js", "camelCase"),
        test_case("src/foo/fooBar.test-utils.js", "camelCase"),
        test_case("src/foo/fooBar.test_utils.js", "camelCase"),
        test_case("src/foo/.test_utils.js", "camelCase"),
        test_case("src/foo/foo.js", "snakeCase"),
        test_case("src/foo/foo_bar.js", "snakeCase"),
        test_case("src/foo/foo.test.js", "snakeCase"),
        test_case("src/foo/foo_bar.test.js", "snakeCase"),
        test_case("src/foo/foo_bar.test_utils.js", "snakeCase"),
        test_case("src/foo/foo_bar.test-utils.js", "snakeCase"),
        test_case("src/foo/.test-utils.js", "snakeCase"),
        test_case("src/foo/foo.js", "kebabCase"),
        test_case("src/foo/foo-bar.js", "kebabCase"),
        test_case("src/foo/foo.test.js", "kebabCase"),
        test_case("src/foo/foo-bar.test.js", "kebabCase"),
        test_case("src/foo/foo-bar.test-utils.js", "kebabCase"),
        test_case("src/foo/foo-bar.test_utils.js", "kebabCase"),
        test_case("src/foo/.test_utils.js", "kebabCase"),
        test_case("src/foo/Foo.js", "pascalCase"),
        test_case("src/foo/FooBar.js", "pascalCase"),
        test_case("src/foo/Foo.test.js", "pascalCase"),
        test_case("src/foo/FooBar.test.js", "pascalCase"),
        test_case("src/foo/FooBar.test-utils.js", "pascalCase"),
        test_case("src/foo/FooBar.test_utils.js", "pascalCase"),
        test_case("src/foo/.test_utils.js", "pascalCase"),
        test_case("spec/iss47Spec.js", "camelCase"),
        test_case("spec/iss47Spec100.js", "camelCase"),
        test_case("spec/i18n.js", "camelCase"),
        test_case("spec/iss47-spec.js", "kebabCase"),
        test_case("spec/iss-47-spec.js", "kebabCase"),
        test_case("spec/iss47-100spec.js", "kebabCase"),
        test_case("spec/i18n.js", "kebabCase"),
        test_case("spec/iss47_spec.js", "snakeCase"),
        test_case("spec/iss_47_spec.js", "snakeCase"),
        test_case("spec/iss47_100spec.js", "snakeCase"),
        test_case("spec/i18n.js", "snakeCase"),
        test_case("spec/Iss47Spec.js", "pascalCase"),
        test_case("spec/Iss47.100spec.js", "pascalCase"),
        test_case("spec/I18n.js", "pascalCase"),
        test_case("src/foo/_fooBar.js", "camelCase"),
        test_case("src/foo/___fooBar.js", "camelCase"),
        test_case("src/foo/_foo_bar.js", "snakeCase"),
        test_case("src/foo/___foo_bar.js", "snakeCase"),
        test_case("src/foo/_foo-bar.js", "kebabCase"),
        test_case("src/foo/___foo-bar.js", "kebabCase"),
        test_case("src/foo/_FooBar.js", "pascalCase"),
        test_case("src/foo/___FooBar.js", "pascalCase"),
        test_case("src/foo/$foo.js", "pascalCase"),
        test_cases("src/foo/foo-bar.js", []),
        test_cases("src/foo/fooBar.js", ["camelCase"]),
        test_cases("src/foo/FooBar.js", ["kebabCase", "pascalCase"]),
        test_cases("src/foo/___foo_bar.js", ["snakeCase", "pascalCase"]),
    ];

    let fail = vec![
        test_case("src/foo/foo_bar.js", ""),
        // todo: linter does not support uppercase JS files
        // test_case("src/foo/foo_bar.JS", "camelCase"),
        test_case("src/foo/foo_bar.test.js", "camelCase"),
        test_case("test/foo/foo_bar.test_utils.js", "camelCase"),
        test_case("test/foo/fooBar.js", "snakeCase"),
        test_case("test/foo/fooBar.test.js", "snakeCase"),
        test_case("test/foo/fooBar.testUtils.js", "snakeCase"),
        test_case("test/foo/fooBar.js", "kebabCase"),
        test_case("test/foo/fooBar.test.js", "kebabCase"),
        test_case("test/foo/fooBar.testUtils.js", "kebabCase"),
        test_case("test/foo/fooBar.js", "pascalCase"),
        test_case("test/foo/foo_bar.test.js", "pascalCase"),
        test_case("test/foo/foo-bar.test-utils.js", "pascalCase"),
        test_case("src/foo/_FOO-BAR.js", "camelCase"),
        test_case("src/foo/___FOO-BAR.js", "camelCase"),
        test_case("src/foo/_FOO-BAR.js", "snakeCase"),
        test_case("src/foo/___FOO-BAR.js", "snakeCase"),
        test_case("src/foo/_FOO-BAR.js", "kebabCase"),
        test_case("src/foo/___FOO-BAR.js", "kebabCase"),
        test_case("src/foo/_FOO-BAR.js", "pascalCase"),
        test_case("src/foo/___FOO-BAR.js", "pascalCase"),
        test_cases("src/foo/foo_bar.js", []),
        test_cases("src/foo/foo-bar.js", ["camelCase", "pascalCase"]),
        test_cases("src/foo/_foo_bar.js", ["camelCase", "pascalCase", "kebabCase"]),
        test_cases("src/foo/_FOO-BAR.js", ["snakeCase"]),
        test_case("src/foo/[foo_bar].js", ""),
        test_case("src/foo/$foo_bar.js", ""),
        test_case("src/foo/$fooBar.js", ""),
        test_cases("src/foo/{foo_bar}.js", ["camelCase", "pascalCase", "kebabCase"]),
    ];

    Tester::new(FilenameCase::NAME, FilenameCase::PLUGIN, pass, fail).test_and_snapshot();
}
