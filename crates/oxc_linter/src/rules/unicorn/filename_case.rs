use convert_case::{Case, Casing};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

fn filename_case_diagnostic(span: Span, case_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Filename should not be in {case_name} case")).with_label(span)
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)]
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
    /// Enforce a case style for filenames.
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
        let Some(case_type) = value.get(0) else {
            return Self::default();
        };

        if let Some(Value::String(s)) = case_type.get("case") {
            return match s.as_str() {
                "kebabCase" => Self { kebab_case: true, ..Self::default() },
                "camelCase" => Self { camel_case: true, ..Self::default() },
                "snakeCase" => Self { snake_case: true, ..Self::default() },
                "pascalCase" => Self { pascal_case: true, ..Self::default() },
                "underscoreCase" => Self { underscore_case: true, ..Self::default() },
                _ => Self::default(),
            };
        }

        if let Some(Value::String(s)) = case_type.get("case") {
            return match s.as_str() {
                "kebabCase" => Self { kebab_case: true, ..Self::default() },
                "camelCase" => Self { camel_case: true, ..Self::default() },
                "snakeCase" => Self { snake_case: true, ..Self::default() },
                "pascalCase" => Self { pascal_case: true, ..Self::default() },
                "underscoreCase" => Self { underscore_case: true, ..Self::default() },
                _ => Self::default(),
            };
        }

        if let Some(Value::Object(map)) = case_type.get("cases") {
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
            return filename_case;
        }

        Self::default()
    }

    fn run_once<'a>(&self, ctx: &LintContext<'_>) {
        let Some(filename) = ctx.file_path().file_stem().and_then(|s| s.to_str()) else {
            return;
        };

        let mut case_name = None;

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
                case_name.replace(name);
            }
        }

        if let Some(case_name) = case_name {
            ctx.diagnostic(filename_case_diagnostic(Span::default(), case_name));
        }
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        // should pass - camel_case, pascal_case both allowed
        ("", None, None, Some(PathBuf::from("foo/bar/baz/Que.tsx"))),
        // should pass - camel_case, pascal_case both allowed
        ("", None, None, Some(PathBuf::from("foo/bar/baz/QueAbc.tsx"))),
        ("", None, None, Some(PathBuf::from("ansiHTML.tsx"))),
    ];
    let fail = vec![
        // should pass - by default kebab_case is not allowed
        ("import foo from 'bar'", None, None, Some(PathBuf::from("foo/bar/baz/aaa-bbb.tsx"))),
        // should pass - by default snake_case is not allowed
        ("baz;", None, None, Some(PathBuf::from("foo/bar/baz/foo_bar.tsx"))),
    ];

    Tester::new(FilenameCase::NAME, pass, fail).test_and_snapshot();
}
