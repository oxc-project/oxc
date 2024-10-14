use convert_case::{Boundary, Case, Converter};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

fn join_strings_disjunction(strings: &[String]) -> String {
    let mut list = String::new();
    for (i, s) in strings.iter().enumerate() {
        if i == 0 {
            list.push_str(s);
        } else if i == strings.len() - 1 {
            list.push_str(&format!(", or {s}"));
        } else {
            list.push_str(&format!(", {s}"));
        }
    }
    list
}

fn filename_case_diagnostic(filename: &str, valid_cases: &[(&str, Case)]) -> OxcDiagnostic {
    let case_names = valid_cases.iter().map(|(name, _)| format!("{name} case")).collect::<Vec<_>>();
    let message = format!("Filename should be in {}", join_strings_disjunction(&case_names));

    let trimmed_filename = filename.trim_matches('_');
    let converted_filenames = valid_cases
        .iter()
        .map(|(_, case)| {
            let converter =
                Converter::new().remove_boundaries(&[Boundary::LowerDigit, Boundary::DigitLower]);
            // get the leading characters that were trimmed, if any, else empty string
            let leading = filename.chars().take_while(|c| c == &'_').collect::<String>();
            let trailing = filename.chars().rev().take_while(|c| c == &'_').collect::<String>();
            format!("'{leading}{}{trailing}'", converter.to_case(*case).convert(trimmed_filename))
        })
        .collect::<Vec<_>>();

    let help_message =
        format!("Rename the file to {}", join_strings_disjunction(&converted_filenames));

    OxcDiagnostic::warn(message).with_label(Span::default()).with_help(help_message)
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)]
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
    /// Inconsistent file naming conventions can make it harder to locate files or to create new ones.
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
    FilenameCase,
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
            (self.camel_case, Case::Camel, "camel"),
            (self.kebab_case, Case::Kebab, "kebab"),
            (self.snake_case, Case::Snake, "snake"),
            (self.pascal_case, Case::Pascal, "pascal"),
        ];
        let mut enabled_cases = cases.iter().filter(|(enabled, _, _)| *enabled);

        if !enabled_cases.any(|(_, case, _)| {
            let converter =
                Converter::new().remove_boundaries(&[Boundary::LowerDigit, Boundary::DigitLower]);
            converter.to_case(*case).convert(filename) == filename
        }) {
            let valid_cases = cases
                .iter()
                .filter_map(
                    |(enabled, case, name)| if *enabled { Some((*name, *case)) } else { None },
                )
                .collect::<Vec<_>>();
            let filename = file_path.file_name().unwrap().to_string_lossy();
            ctx.diagnostic(filename_case_diagnostic(&filename, &valid_cases));
        }
    }
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

    Tester::new(FilenameCase::NAME, pass, fail).test_and_snapshot();
}
