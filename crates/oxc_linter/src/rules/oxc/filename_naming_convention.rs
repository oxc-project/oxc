use convert_case::{Boundary, Case, Converter};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn filename_naming_convention_diagnostic(
    _span: Span,
    filename: &str,
    pattern: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Filename `{filename}` does not match the configured naming convention."
    ))
    .with_help(format!(
        "Rename the file so its basename matches the case style `{pattern}` for this path."
    ))
    .with_label(Span::default())
}

#[derive(Debug, Default, Clone)]
pub struct FilenameNamingConvention(Box<FilenameNamingConventionConfig>);

#[derive(Debug, Default, Clone)]
pub struct FilenameNamingConventionConfig {
    rules: Vec<(String, NamingConvention)>,
    ignore_middle_extensions: bool,
}

#[derive(Debug, Clone, Copy)]
#[expect(clippy::enum_variant_names)]
enum NamingConvention {
    CamelCase,
    FlatCase,
    KebabCase,
    PascalCase,
    ScreamingSnakeCase,
    SnakeCase,
}

impl NamingConvention {
    fn from_config_value(value: &str) -> Option<Self> {
        match value {
            "CAMEL_CASE" => Some(Self::CamelCase),
            "FLAT_CASE" => Some(Self::FlatCase),
            "KEBAB_CASE" => Some(Self::KebabCase),
            "PASCAL_CASE" => Some(Self::PascalCase),
            "SCREAMING_SNAKE_CASE" => Some(Self::ScreamingSnakeCase),
            "SNAKE_CASE" => Some(Self::SnakeCase),
            _ => None,
        }
    }

    fn matches(self, value: &str) -> bool {
        if value.contains('.') {
            return false;
        }

        let converter =
            Converter::new().remove_boundaries(&[Boundary::LowerDigit, Boundary::DigitLower]);

        match self {
            Self::CamelCase => converter.to_case(Case::Camel).convert(value) == value,
            Self::FlatCase => value
                .chars()
                .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit()),
            Self::KebabCase => converter.to_case(Case::Kebab).convert(value) == value,
            Self::PascalCase => converter.to_case(Case::Pascal).convert(value) == value,
            Self::ScreamingSnakeCase => converter.to_case(Case::UpperSnake).convert(value) == value,
            Self::SnakeCase => converter.to_case(Case::Snake).convert(value) == value,
        }
    }

    fn as_config_value(self) -> &'static str {
        match self {
            Self::CamelCase => "CAMEL_CASE",
            Self::FlatCase => "FLAT_CASE",
            Self::KebabCase => "KEBAB_CASE",
            Self::PascalCase => "PASCAL_CASE",
            Self::ScreamingSnakeCase => "SCREAMING_SNAKE_CASE",
            Self::SnakeCase => "SNAKE_CASE",
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces path-aware filename naming conventions.
    ///
    /// ### Why is this bad?
    ///
    /// Monorepos often rely on consistent filename casing for discoverability,
    /// import ergonomics, and cross-platform filesystem safety.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // path: src/core/fileRead.test.ts
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// // path: src/core/file-read.test.ts
    /// ```
    FilenameNamingConvention,
    oxc,
    style,
    none
);

impl Rule for FilenameNamingConvention {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        let mut config = FilenameNamingConventionConfig::default();

        if let Some(pattern_object) = value.get(0).and_then(serde_json::Value::as_object) {
            config.rules = pattern_object
                .iter()
                .filter_map(|(pattern, convention)| {
                    convention
                        .as_str()
                        .and_then(NamingConvention::from_config_value)
                        .map(|naming| (pattern.clone(), naming))
                })
                .collect();
        }

        config.ignore_middle_extensions = value
            .get(1)
            .and_then(serde_json::Value::as_object)
            .and_then(|options| options.get("ignoreMiddleExtensions"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Ok(Self(Box::new(config)))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let Some(raw_filename) = ctx.file_path().file_name().and_then(|filename| filename.to_str())
        else {
            return;
        };

        let basename = strip_extension(raw_filename, self.0.ignore_middle_extensions);
        let suffixes = normalized_path_suffixes(ctx.file_path());

        for (pattern, naming) in &self.0.rules {
            if suffixes.iter().any(|suffix| fast_glob::glob_match(pattern, suffix))
                && !naming.matches(basename)
            {
                ctx.diagnostic(filename_naming_convention_diagnostic(
                    Span::default(),
                    raw_filename,
                    naming.as_config_value(),
                ));
                return;
            }
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host()
    }
}

fn strip_extension(filename: &str, ignore_middle_extensions: bool) -> &str {
    if ignore_middle_extensions {
        filename.split('.').next().unwrap_or(filename)
    } else {
        filename.rsplit_once('.').map_or(filename, |(name, _)| name)
    }
}

fn normalized_path_suffixes(path: &std::path::Path) -> Vec<String> {
    let components = path
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>();

    (0..components.len()).map(|index| components[index..].join("/")).collect()
}

#[test]
fn test() {
    use std::path::PathBuf;

    use serde_json::Value;
    use serde_json::json;

    use crate::tester::Tester;

    fn test_case(
        path: &'static str,
        config: Option<Value>,
    ) -> (&'static str, Option<Value>, Option<Value>, Option<PathBuf>) {
        ("", config, None, Some(PathBuf::from(path)))
    }

    let pass = vec![
        test_case(
            "src/core/file-read.ts",
            Some(
                json!([{ "**/*.{ts,mts,js,mjs}": "KEBAB_CASE" }, { "ignoreMiddleExtensions": true }]),
            ),
        ),
        test_case(
            "src/core/file-read.test.ts",
            Some(
                json!([{ "**/*.{ts,mts,js,mjs}": "KEBAB_CASE" }, { "ignoreMiddleExtensions": true }]),
            ),
        ),
        test_case(
            "scripts/build.mjs",
            Some(
                json!([{ "**/*.{ts,mts,js,mjs}": "KEBAB_CASE" }, { "ignoreMiddleExtensions": true }]),
            ),
        ),
        test_case(
            "README.md",
            Some(
                json!([{ "**/*.{ts,mts,js,mjs}": "KEBAB_CASE" }, { "ignoreMiddleExtensions": true }]),
            ),
        ),
    ];

    let fail = vec![
        test_case(
            "src/core/fileRead.ts",
            Some(
                json!([{ "**/*.{ts,mts,js,mjs}": "KEBAB_CASE" }, { "ignoreMiddleExtensions": true }]),
            ),
        ),
        test_case(
            "src/core/FileRead.ts",
            Some(
                json!([{ "**/*.{ts,mts,js,mjs}": "KEBAB_CASE" }, { "ignoreMiddleExtensions": true }]),
            ),
        ),
        test_case(
            "src/core/fileRead.test.ts",
            Some(
                json!([{ "**/*.{ts,mts,js,mjs}": "KEBAB_CASE" }, { "ignoreMiddleExtensions": true }]),
            ),
        ),
        test_case(
            "src/core/file-read.test.ts",
            Some(
                json!([{ "**/*.{ts,mts,js,mjs}": "KEBAB_CASE" }, { "ignoreMiddleExtensions": false }]),
            ),
        ),
    ];

    Tester::new(FilenameNamingConvention::NAME, FilenameNamingConvention::PLUGIN, pass, fail)
        .test_and_snapshot();
}
