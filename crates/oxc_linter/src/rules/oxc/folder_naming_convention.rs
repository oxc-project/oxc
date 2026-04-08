use convert_case::{Boundary, Case, Converter};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn folder_naming_convention_diagnostic(_span: Span, folder: &str, pattern: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Folder `{folder}` does not match the configured naming convention."
    ))
    .with_help(format!(
        "Rename the folder so matched path segments follow the case style `{pattern}`."
    ))
    .with_label(Span::default())
}

#[derive(Debug, Default, Clone)]
pub struct FolderNamingConvention(Box<FolderNamingConventionConfig>);

#[derive(Debug, Default, Clone)]
pub struct FolderNamingConventionConfig {
    rules: Vec<(String, NamingConvention)>,
    ignore_words: Vec<String>,
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
    /// Enforces path-aware folder naming conventions.
    ///
    /// ### Why is this bad?
    ///
    /// Consistent directory naming keeps imports, navigation, and repository
    /// structure predictable across packages.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // path: src/core_utils/file-read.ts
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// // path: src/core/utils/file-read.ts
    /// ```
    FolderNamingConvention,
    oxc,
    style,
    none
);

impl Rule for FolderNamingConvention {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        let mut config = FolderNamingConventionConfig::default();

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

        config.ignore_words = value
            .get(1)
            .and_then(serde_json::Value::as_object)
            .and_then(|options| options.get("ignoreWords"))
            .and_then(serde_json::Value::as_array)
            .map(|words| {
                words
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(Self(Box::new(config)))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let Some(parent) = ctx.file_path().parent() else {
            return;
        };

        let components = normalized_components(parent);

        for (index, folder_name) in components.iter().enumerate() {
            if self.0.ignore_words.iter().any(|word| word == folder_name) {
                continue;
            }

            for (pattern, naming) in &self.0.rules {
                if folder_matches_pattern(&components, index, pattern)
                    && !naming.matches(folder_name)
                {
                    ctx.diagnostic(folder_naming_convention_diagnostic(
                        Span::default(),
                        folder_name,
                        naming.as_config_value(),
                    ));
                    return;
                }
            }
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host()
    }
}

fn normalized_components(path: &std::path::Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect()
}

fn folder_matches_pattern(components: &[String], end_index: usize, pattern: &str) -> bool {
    for start_index in 0..=end_index {
        let candidate = format!("{}/", components[start_index..=end_index].join("/"));
        if path_matches_pattern(pattern, &candidate) {
            return true;
        }
    }

    false
}

fn path_matches_pattern(pattern: &str, candidate: &str) -> bool {
    if fast_glob::glob_match(pattern, candidate) {
        return true;
    }

    let Some(negation_start) = pattern.find("!(") else {
        return false;
    };
    let Some(relative_end) = pattern[negation_start + 2..].find(')') else {
        return false;
    };

    let negation_end = negation_start + 2 + relative_end;
    let excluded = &pattern[negation_start + 2..negation_end];

    if candidate.trim_end_matches('/').rsplit('/').next().is_some_and(|segment| segment == excluded)
    {
        return false;
    }

    let fallback_pattern =
        format!("{}*{}", &pattern[..negation_start], &pattern[negation_end + 1..]);
    fast_glob::glob_match(&fallback_pattern, candidate)
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
        test_case("src/core/file-read.ts", Some(json!([{ "src/**/": "KEBAB_CASE" }]))),
        test_case("src/core/utils/file-read.ts", Some(json!([{ "src/**/": "KEBAB_CASE" }]))),
        test_case(
            "src/features/__tests__/file-read.test.ts",
            Some(json!([{ "src/**/!(__tests__)/": "KEBAB_CASE" }])),
        ),
        test_case("tests/helpers/file-read.ts", Some(json!([{ "src/**/": "KEBAB_CASE" }]))),
    ];

    let fail = vec![
        test_case("src/core_utils/file-read.ts", Some(json!([{ "src/**/": "KEBAB_CASE" }]))),
        test_case("src/core/FeatureFlags/file-read.ts", Some(json!([{ "src/**/": "KEBAB_CASE" }]))),
        test_case(
            "src/features/BadFolder/file-read.ts",
            Some(json!([{ "src/**/!(__tests__)/": "KEBAB_CASE" }])),
        ),
    ];

    Tester::new(FolderNamingConvention::NAME, FolderNamingConvention::PLUGIN, pass, fail)
        .test_and_snapshot();
}
