use serde_json::Value;

use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(max-len): Line is too long (current length: {current_length:?}, maximum: {max:?})")]
#[diagnostic(
    severity(warning),
    help("Consider breaking this line into multiple lines or shortening comments/codes where applicable")
)]
struct MaxLenDiagnostic {
    current_length: usize,
    max: usize,
    #[label]
    span: Span,
}

#[derive(Debug, Default, Clone)]
pub struct MaxLen(Box<MaxLenConfig>);

#[derive(Debug, Clone)]
pub struct MaxLenConfig {
    max: usize,
}

impl std::ops::Deref for MaxLen {
    type Target = MaxLenConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for MaxLenConfig {
    fn default() -> Self {
        Self { max: 80 } // default max length is often considered to be 80 characters
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforce a maximum length of characters per line
    ///
    /// ### Why is this bad?
    /// Lines that are too long can be difficult to read, understand, and maintain.
    /// Excessively long lines can also result in horizontal scrolling which hinders readability.
    /// While there is no objective maximum length considered acceptable for a line,
    /// a commonly used standard is 80 characters per line.
    ///
    /// ### Example
    /// ```javascript
    /// // This line is fine
    /// const example = "This is a concise example.";
    ///
    /// // This line might be considered too long and difficult to read without wrapping or horizontal scrolling
    /// const tooLongExample = "This line is an example of a line that stretches far beyond the conventional length and could be hard to read.";
    /// ```
    MaxLen,
    pedantic
);

impl Rule for MaxLen {
    fn from_configuration(value: Value) -> Self {
        let max = value
            .as_number()
            .and_then(serde_json::Number::as_u64)
            .map_or(80, |v| usize::try_from(v).unwrap_or(80)); // default max length of 80 chars if not specified

        Self(Box::new(MaxLenConfig { max }))
    }

    fn run_once(&self, ctx: &LintContext) {
        for (line_number, line) in ctx.source_text().lines().enumerate() {
            let line_length = line.chars().count();
            if line_length > self.max {
                let error_start = ctx
                    .source_text()
                    .lines()
                    .take(line_number)
                    .map(|line| line.chars().count() + 1) // padding 1 for '\n'
                    .sum::<usize>();

                let error = MaxLenDiagnostic {
                    current_length: line_length,
                    max: self.max,
                    span: Span::new(
                        u32::try_from(error_start).unwrap_or(u32::MIN),
                        u32::try_from(error_start + line_length).unwrap_or(u32::MAX),
                    ),
                };

                ctx.diagnostic(error);
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Examples of JavaScript code that should pass the max len lint with default max character of 80
        ("const x = 42;", None),
        ("const greeting = \"Hello, World!\";", None),
        ("// This is a short comment", None),
        ("const example = \"This line is short enough to pass the lint check.\";", None),
    ];

    let fail = vec![
        // Examples of JavaScript code that should fail the max len lint with default max character of 80
        (
            "const longLine = \"This is a sentence that is deliberately made to go beyond the typical 80 character limit to show that the linting rule works.\";",
            Some(serde_json::json!([80]))
        ),
        (
            "// This comment is definitely too long for a single line and should be broken up according to the max len rule which we expect to fail here",
            Some(serde_json::json!([80]))
        ),
        (
            "const anotherLongLine = \"Lines of code or comments should not extend beyond the 80 characters in length to be easily readable and maintainable.\";",
            Some(serde_json::json!([80]))
        ),
    ];

    Tester::new(MaxLen::NAME, pass, fail).test_and_snapshot();
}
