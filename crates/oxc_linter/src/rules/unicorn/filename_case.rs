use convert_case::{Case, Casing};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(filename-case): Filename should not be in {1} case")]
#[diagnostic(severity(warning))]
struct FilenameCaseDiagnostic(#[label] pub Span, &'static str);

#[derive(Debug, Clone)]
pub struct FilenameCase {
    kebab_case: bool,
    camel_case: bool,
    snake_case: bool,
    pascal_case: bool,
    underscore_case: bool,
}

impl Default for FilenameCase {
    fn default() -> Self {
        Self {
            kebab_case: false,
            camel_case: true,
            snake_case: false,
            pascal_case: true,
            underscore_case: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Example
    /// ```
    FilenameCase,
    style
);

impl Rule for FilenameCase {
    fn from_configuration(value: serde_json::Value) -> Self {
        let Some(case_type) = value.get("cases") else { return Self::default() };

        match case_type {
            Value::String(s) => match s.as_str() {
                "kebabCase" => Self { kebab_case: true, ..Self::default() },
                "camelCase" => Self { camel_case: true, ..Self::default() },
                "snakeCase" => Self { snake_case: true, ..Self::default() },
                "pascalCase" => Self { pascal_case: true, ..Self::default() },
                "underscoreCase" => Self { underscore_case: true, ..Self::default() },
                _ => Self::default(),
            },
            Value::Object(map) => {
                let mut filename_case = Self::default();
                for (key, value) in map {
                    match (key.as_str(), value) {
                        ("kebabCase", Value::Bool(b)) => filename_case.kebab_case = *b,
                        ("camelCase", Value::Bool(b)) => filename_case.camel_case = *b,
                        ("snakeCase", Value::Bool(b)) => filename_case.snake_case = *b,
                        ("pascalCase", Value::Bool(b)) => filename_case.pascal_case = *b,
                        ("underscoreCase", Value::Bool(b)) => filename_case.underscore_case = *b,
                        _ => (),
                    }
                }
                filename_case
            }
            _ => Self::default(),
        }
    }

    fn run_once<'a>(&self, ctx: &LintContext<'_>) {
        let Some(filename) = ctx.file_path().file_stem().and_then(|s| s.to_str()) else { return };

        let mut case_name = "";

        let cases = [
            (Case::Kebab, "kebab", self.kebab_case),
            (Case::Camel, "camel", self.camel_case),
            (Case::Snake, "snake", self.snake_case),
            (Case::Pascal, "pascal", self.pascal_case),
            (Case::Pascal, "underscore", self.underscore_case),
        ];

        for (case, name, condition) in cases {
            if filename.to_case(case) == filename {
                if condition {
                    return;
                }
                case_name = name;
            }
        }

        ctx.diagnostic(FilenameCaseDiagnostic(Span::default(), case_name));
    }
}
