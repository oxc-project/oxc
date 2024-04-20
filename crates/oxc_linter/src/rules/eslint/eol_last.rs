use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{context::LintContext, fixer::Fix, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(eol-last): {msg:?}.")]
#[diagnostic(severity(warning))]
struct EolLastDiagnostic {
    msg: String,
    #[label]
    span: Span,
}

#[derive(Debug, Clone)]
pub struct EolLast {
    mode: String,
}

impl Default for EolLast {
    fn default() -> Self {
        Self { mode: String::from("always") }
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// Require or disallow newline at the end of files.
    ///
    /// ### Why is this bad?
    /// Inconsistent file endings can cause unnecessary diff in version control, lint errors, and issues between different OSes.
    ///
    /// ### Example
    /// ```rust
    /// // Bad
    /// let x = 5
    /// // Good
    /// let x = 5\n
    /// ```
    EolLast,
    correctness
);

impl Rule for EolLast {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self { mode: value.get(0).and_then(Value::as_str).unwrap_or("always").to_string() }
    }

    fn run_once(&self, ctx: &LintContext) {
        let full_text = ctx.source_text();
        if full_text.is_empty() {
            return;
        }

        let lf = "\n";
        let crlf: &str = "\r\n";
        let text_len = full_text.len();
        let mut end_index = text_len;
        while end_index > 0
            && (full_text.as_bytes()[end_index - 1] == b'\n'
                || full_text.as_bytes()[end_index - 1] == b'\r')
        {
            // skip lf
            end_index -= 1;
            if full_text.as_bytes()[end_index] == b'\n'
                && end_index > 0
                && full_text.as_bytes()[end_index - 1] == b'\r'
            {
                // skip cr
                end_index -= 1;
            }
        }

        let has_end_newline = text_len > end_index;
        let mut append_content = if full_text.ends_with(crlf) { crlf } else { lf };
        let mut mode = self.mode.clone();
        if &mode == "unix" {
            mode = "always".to_string();
        } else if &mode == "windows" {
            mode = "always".to_string();
            append_content = crlf;
        }

        let span_end = u32::try_from(text_len).unwrap_or(0);
        if &mode == "always" && !has_end_newline {
            let span = Span::new(span_end, span_end);
            let error = EolLastDiagnostic {
                msg: "Newline required at end of file but not found.".to_string(),
                span,
            };

            ctx.diagnostic_with_fix(error, || Fix::new(append_content, span));
        } else if &mode == "never" && has_end_newline {
            let span_start = u32::try_from(end_index).unwrap_or(0);
            let span = Span::new(span_start, span_end);
            let error =
                EolLastDiagnostic { msg: "Newline not allowed at end of file.".to_string(), span };

            ctx.diagnostic_with_fix(error, || Fix::new("", span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("", None),
        ("\n", None),
        ("var a = 123;\n", None),
        ("var a = 123;\n\n", None),
        ("var a = 123;\n   \n", None),
        ("\r\n", None),
        ("var a = 123;\r\n", None),
        ("var a = 123;\r\n\r\n", None),
        ("var a = 123;\r\n   \r\n", None),
        ("var a = 123;", Some(json!(["never"]))),
        ("var a = 123;\nvar b = 456;", Some(json!(["never"]))),
        ("var a = 123;\r\nvar b = 456;", Some(json!(["never"]))),
        // Deprecated: `"unix"` parameter
        ("", Some(json!(["unix"]))),
        ("\n", Some(json!(["unix"]))),
        ("var a = 123;\n", Some(json!(["unix"]))),
        ("var a = 123;\n\n", Some(json!(["unix"]))),
        ("var a = 123;\n   \n", Some(json!(["unix"]))),
        // Deprecated: `"unix"` parameter
        ("", Some(json!(["windows"]))),
        ("\n", Some(json!(["windows"]))),
        ("\r\n", Some(json!(["windows"]))),
        ("var a = 123;\r\n", Some(json!(["windows"]))),
        ("var a = 123;\r\n\r\n", Some(json!(["windows"]))),
        ("var a = 123;\r\n   \r\n", Some(json!(["windows"]))),
    ];

    let fail = vec![
        ("var a = 123;", None),
        ("var a = 123;\n   ", None),
        ("var a = 123;\n", Some(json!(["never"]))),
        ("var a = 123;\r\n", Some(json!(["never"]))),
        ("var a = 123;\r\n\r\n", Some(json!(["never"]))),
        ("var a = 123;\nvar b = 456;\n", Some(json!(["never"]))),
        ("var a = 123;\r\nvar b = 456;\r\n", Some(json!(["never"]))),
        ("var a = 123;\n\n", Some(json!(["never"]))),
        // Deprecated: `"unix"` parameter
        ("var a = 123;", Some(json!(["unix"]))),
        ("var a = 123;\n   ", Some(json!(["unix"]))),
        // Deprecated: `"windows"` parameter
        ("var a = 123;", Some(json!(["windows"]))),
        ("var a = 123;\r\n   ", Some(json!(["windows"]))),
    ];

    let fix = vec![
        ("var a = 123;", "var a = 123;\n", None),
        ("var a = 123;\n   ", "var a = 123;\n   \n", None),
        ("var a = 123;\n", "var a = 123;", Some(json!(["never"]))),
        ("var a = 123;\r\n", "var a = 123;", Some(json!(["never"]))),
        ("var a = 123;\r\n\r\n", "var a = 123;", Some(json!(["never"]))),
        ("var a = 123;\nvar b = 456;\n", "var a = 123;\nvar b = 456;", Some(json!(["never"]))),
        (
            "var a = 123;\r\nvar b = 456;\r\n",
            "var a = 123;\r\nvar b = 456;",
            Some(json!(["never"])),
        ),
        ("var a = 123;\n\n", "var a = 123;", Some(json!(["never"]))),
        // Deprecated: `"unix"` parameter
        ("var a = 123;", "var a = 123;\n", Some(json!(["unix"]))),
        ("var a = 123;\n   ", "var a = 123;\n   \n", Some(json!(["unix"]))),
        // Deprecated: `"windows"` parameter
        ("var a = 123;", "var a = 123;\r\n", Some(json!(["windows"]))),
        ("var a = 123;\r\n   ", "var a = 123;\r\n   \r\n", Some(json!(["windows"]))),
    ];

    Tester::new(EolLast::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
