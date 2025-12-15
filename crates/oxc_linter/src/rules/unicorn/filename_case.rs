use convert_case::{Boundary, Case, Converter};
use cow_utils::CowUtils;
use lazy_regex::{Regex, RegexBuilder};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn filename_case_diagnostic(message: String, help_message: String) -> OxcDiagnostic {
    OxcDiagnostic::warn(message).with_label(Span::default()).with_help(help_message)
}

#[derive(Debug, Clone, Default)]
pub struct FilenameCase(Box<FilenameCaseConfig>);

impl std::ops::Deref for FilenameCase {
    type Target = FilenameCaseConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct FilenameCaseConfig {
    kebab_case: bool,
    camel_case: bool,
    snake_case: bool,
    pascal_case: bool,
    ignore: Option<Regex>,
    multiple_file_extensions: bool,
}

impl Default for FilenameCaseConfig {
    fn default() -> Self {
        Self {
            kebab_case: true,
            camel_case: false,
            snake_case: false,
            pascal_case: false,
            ignore: None,
            multiple_file_extensions: true,
        }
    }
}

// Use a separate struct for configuration docs, as the main config struct is
// too different from the format of the end-user configuration options' shape.
#[derive(Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct FilenameCaseConfigJson {
    /// The case style(s) to allow/enforce for filenames. `true` means the case style is allowed, `false` means it is banned.
    ///
    /// You can set the `cases` option like this:
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
    cases: FilenameCaseConfigJsonCases,
    /// The case style to enforce for filenames.
    ///
    /// You can set the `case` option like this:
    /// ```json
    /// "unicorn/filename-case": [
    ///   "error",
    ///   {
    ///     "case": "kebabCase"
    ///   }
    /// ]
    /// ```
    case: FilenameCaseJsonOptions,
    /// A regular expression pattern for filenames to ignore.
    ///
    /// You can set the `ignore` option like this:
    /// ```json
    /// "unicorn/filename-case": [
    ///   "error",
    ///   {
    ///     "ignore": "^foo.*$"
    ///   }
    /// ]
    /// ```
    ignore: Option<Regex>,
    /// Whether to treat additional, `.`-separated parts of a filename as
    /// parts of the extension rather than parts of the filename.
    multiple_file_extensions: bool,
}

impl Default for FilenameCaseConfigJson {
    fn default() -> Self {
        Self {
            cases: FilenameCaseConfigJsonCases::default(),
            case: FilenameCaseJsonOptions::KebabCase,
            ignore: None,
            multiple_file_extensions: true,
        }
    }
}

#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct FilenameCaseConfigJsonCases {
    /// Whether kebab case is allowed, e.g. `some-file-name.js`.
    kebab_case: bool,
    /// Whether camel case is allowed, e.g. `someFileName.js`.
    camel_case: bool,
    /// Whether snake case is allowed, e.g. `some_file_name.js`.
    snake_case: bool,
    /// Whether pascal case is allowed, e.g. `SomeFileName.js`.
    pascal_case: bool,
}

impl Default for FilenameCaseConfigJsonCases {
    fn default() -> Self {
        Self { kebab_case: true, camel_case: false, snake_case: false, pascal_case: false }
    }
}

#[derive(Debug, Default, Clone, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[expect(clippy::enum_variant_names)]
enum FilenameCaseJsonOptions {
    #[default]
    KebabCase,
    CamelCase,
    SnakeCase,
    PascalCase,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a consistent case style for filenames to improve project organization and maintainability.
    /// By default, `kebab-case` is enforced, but other styles can be configured.
    ///
    /// Files named `index.js`, `index.ts`, etc. are exempt from this rule as they cannot reliably be
    /// renamed to other casings (mainly just a problem with PascalCase).
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent file naming conventions make it harder to locate files, navigate projects, and enforce
    /// consistency across a codebase. Standardizing naming conventions improves readability, reduces cognitive
    /// overhead, and aligns with best practices in large-scale development.
    ///
    /// ### Examples
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
    unicorn,
    style,
    config = FilenameCaseConfigJson,
);

impl Rule for FilenameCase {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut config =
            FilenameCaseConfig { multiple_file_extensions: true, ..Default::default() };

        if let Some(value) = value.get(0) {
            config.kebab_case = false;
            if let Some(Value::String(val)) = value.get("ignore") {
                config.ignore = RegexBuilder::new(val).build().ok();
            }

            if let Some(Value::Bool(val)) = value.get("multipleFileExtensions") {
                config.multiple_file_extensions = *val;
            }

            if let Some(Value::String(s)) = value.get("case") {
                match s.as_str() {
                    "camelCase" => config.camel_case = true,
                    "snakeCase" => config.snake_case = true,
                    "pascalCase" => config.pascal_case = true,
                    _ => config.kebab_case = true,
                }
                return Self(Box::new(config));
            }

            if let Some(Value::Object(map)) = value.get("cases") {
                for (key, value) in map {
                    match (key.as_str(), value) {
                        ("kebabCase", Value::Bool(b)) => config.kebab_case = *b,
                        ("camelCase", Value::Bool(b)) => config.camel_case = *b,
                        ("snakeCase", Value::Bool(b)) => config.snake_case = *b,
                        ("pascalCase", Value::Bool(b)) => config.pascal_case = *b,
                        _ => (),
                    }
                }
                return Self(Box::new(config));
            }
        }

        config.kebab_case = true;
        Self(Box::new(config))
    }

    fn run_once<'a>(&self, ctx: &LintContext<'_>) {
        let Some(raw_filename) = ctx.file_path().file_name().and_then(|s| s.to_str()) else {
            return;
        };

        if raw_filename.starts_with('.')
            || self.ignore.as_ref().is_some_and(|r| r.is_match(raw_filename))
        {
            return;
        }

        let filename = if self.multiple_file_extensions {
            raw_filename.split('.').next()
        } else {
            raw_filename.rsplit_once('.').map(|(before, _)| before)
        };

        let filename = filename.unwrap_or(raw_filename);

        // Ignore files named "index" — they are often used as module entry points and
        // cannot reliably be renamed to other casings (e.g. "Index.js"), so allow them
        // regardless of the configured filename case.
        if filename.eq_ignore_ascii_case("index") {
            return;
        }

        let trimmed_filename = filename.trim_matches('_');

        let cases = [
            (self.camel_case, Case::Camel, "camelCase"),
            (self.kebab_case, Case::Kebab, "kebab-case"),
            (self.snake_case, Case::Snake, "snake_case"),
            (self.pascal_case, Case::Pascal, "PascalCase"),
        ];

        let mut valid_cases = Vec::new();
        for (enabled, case, name) in cases {
            if enabled {
                let converter = Converter::new()
                    .remove_boundaries(&[Boundary::LowerDigit, Boundary::DigitLower]);
                let converter = converter.to_case(case);

                if converter.convert(trimmed_filename) == trimmed_filename {
                    return;
                }

                valid_cases.push((converter, name));
            }
        }

        let valid_cases_len = valid_cases.len();

        let mut message = String::from("Filename should be in ");
        let mut help_message = String::from("Rename the file to ");

        for (i, (converter, name)) in valid_cases.into_iter().enumerate() {
            let filename = format!(
                "'{}'",
                raw_filename.cow_replace(trimmed_filename, &converter.convert(trimmed_filename))
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

        ctx.diagnostic(filename_case_diagnostic(message, help_message));
    }

    fn should_run(&self, ctx: &ContextHost<'_>) -> bool {
        ctx.is_first_sub_host()
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

    fn test_case_with_options(
        path: &'static str,
        options: Value,
    ) -> (&'static str, Option<Value>, Option<Value>, Option<PathBuf>) {
        ("", Some(options), None, Some(PathBuf::from(path)))
    }

    let pass = vec![
        // Default is to allow kebab-case
        ("", None, None, Some(PathBuf::from("foo-bar.tsx"))),
        ("", None, None, Some(PathBuf::from("src/foo-bar.tsx"))),
        ("", None, None, Some(PathBuf::from("src/bar/foo-bar.js"))),
        ("", None, None, Some(PathBuf::from("src/bar/foo.js"))),
        // Specific cases
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
        test_case("", "camelCase"),
        test_case("", "snakeCase"),
        test_case("", "kebabCase"),
        test_case("", "pascalCase"),
        test_case("src/foo/_fooBar.js", "camelCase"),
        test_case("src/foo/___fooBar.js", "camelCase"),
        test_case("src/foo/_foo_bar.js", "snakeCase"),
        test_case("src/foo/___foo_bar.js", "snakeCase"),
        test_case("src/foo/_foo-bar.js", "kebabCase"),
        test_case("src/foo/___foo-bar.js", "kebabCase"),
        test_case("src/foo/_FooBar.js", "pascalCase"),
        test_case("src/foo/___FooBar.js", "pascalCase"),
        test_case("src/foo/$foo.js", "pascalCase"),
        test_case("src/foo/[fooBar].js", "camelCase"),
        test_case("src/foo/{foo_bar}.js", "snakeCase"),
        test_cases("src/foo/foo-bar.js", []),
        test_cases("src/foo/fooBar.js", ["camelCase"]),
        test_cases("src/foo/FooBar.js", ["kebabCase", "pascalCase"]),
        test_cases("src/foo/___foo_bar.js", ["snakeCase", "pascalCase"]),
        test_case_with_options(
            "src/foo/index.js",
            serde_json::json!([{ "case": "kebabCase", "ignore": r"FOOBAR.js" }]),
        ),
        test_case_with_options(
            "src/foo/FOOBAR.js",
            serde_json::json!([{ "case": "kebabCase", "ignore": r"FOOBAR\.js" }]),
        ),
        test_case_with_options(
            "src/foo/BAR.js",
            serde_json::json!([{ "case": "kebabCase", "ignore": r"FOO.js|BAR.js" }]),
        ),
        test_case_with_options(
            "src/foo/fooBar.testUtils.js",
            serde_json::json!([{ "case": "camelCase", "multipleFileExtensions": false }]),
        ),
        test_case_with_options(
            "src/foo/foo_bar.test_utils.js",
            serde_json::json!([{ "case": "snakeCase", "multipleFileExtensions": false }]),
        ),
        ("", None, None, Some(PathBuf::from("foo-bar.tsx"))),
        // Ensure all `index` files are allowed, despite being in non-conforming case.
        test_case("index.js", "camelCase"),
        test_case("index.js", "snakeCase"),
        test_case("index.js", "kebabCase"),
        test_case("index.js", "pascalCase"),
        test_case("index.mjs", "camelCase"),
        test_case("index.mjs", "snakeCase"),
        test_case("index.mjs", "kebabCase"),
        test_case("index.mjs", "pascalCase"),
        test_case("index.cjs", "camelCase"),
        test_case("index.cjs", "snakeCase"),
        test_case("index.cjs", "kebabCase"),
        test_case("index.cjs", "pascalCase"),
        test_case("index.ts", "camelCase"),
        test_case("index.ts", "snakeCase"),
        test_case("index.ts", "kebabCase"),
        test_case("index.ts", "pascalCase"),
        test_case("index.tsx", "camelCase"),
        test_case("index.tsx", "snakeCase"),
        test_case("index.tsx", "kebabCase"),
        test_case("index.tsx", "pascalCase"),
        test_case("index.vue", "camelCase"),
        test_case("index.vue", "snakeCase"),
        test_case("index.vue", "kebabCase"),
        test_case("index.vue", "pascalCase"),
        test_case("foo/bar/index.vue", "pascalCase"),
    ];

    let fail = vec![
        test_case("src/foo/foo_bar.js", ""),
        // todo: linter does not support uppercase .JS files
        // test_case("src/foo/foo_bar.JS", "camelCase"),
        test_case("src/foo/foo_bar.test.js", "camelCase"),
        test_case("test/foo/foo_bar.test_utils.js", "camelCase"),
        test_case("test/foo/fooBar.js", "snakeCase"),
        test_case("test/foo/fooBar.test.js", "snakeCase"),
        test_case("test/foo/fooBar.testUtils.js", "snakeCase"),
        test_case("test/foo/fooBar.test_utils.js", "snakeCase"),
        test_case_with_options(
            "test/foo/foo_bar.testUtils.js",
            serde_json::json!([{ "case": "snakeCase", "multipleFileExtensions": false }]),
        ),
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
        test_case_with_options(
            "src/foo/FOOBAR.js",
            serde_json::json!([{ "case": "kebabCase", "ignore": r"foobar.js" }]),
        ),
        test_case_with_options(
            "src/foo/fooBar.TestUtils.js",
            serde_json::json!([{ "case": "camelCase", "multipleFileExtensions": false }]),
        ),
        test_case_with_options(
            "src/foo/foo_bar.test-utils.js",
            serde_json::json!([{ "case": "snakeCase", "multipleFileExtensions": false }]),
        ),
        ("", None, None, Some(PathBuf::from("FooBar.tsx"))),
        // should only report once
        ("<script></script><script setup></script>", None, None, Some(PathBuf::from("FooBar.vue"))),
    ];

    Tester::new(FilenameCase::NAME, FilenameCase::PLUGIN, pass, fail).test_and_snapshot();
}
