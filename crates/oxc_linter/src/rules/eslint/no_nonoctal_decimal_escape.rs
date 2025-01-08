use lazy_static::lazy_static;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::{Captures, Regex};

use crate::{context::LintContext, rule::Rule, AstNode};

fn replacement(escape_sequence: &str, replacement: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Don't use '{escape_sequence}' escape sequence."))
        .with_help(format!("Replace '{escape_sequence}' with '{replacement}'. This maintains the current functionality."))
        .with_label(span)
}

fn escape_backslash(escape_sequence: &str, replacement: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Don't use '{escape_sequence}' escape sequence."))
        .with_help(format!("Replace '{escape_sequence}' with '{replacement}' to include the actual backslash character."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNonoctalDecimalEscape;

declare_oxc_lint!(
    /// ### What it does
    /// This rule disallows \8 and \9 escape sequences in string literals
    ///
    /// ### Why is this bad?
    /// ECMAScript specification treats \8 and \9 in string literals as a legacy feature
    ///
    /// ### Example
    /// ```javascript
    /// incorrect:
    /// "\8"
    /// "\9"
    /// correct:
    /// "8"
    /// "\\9"
    /// ```
    NoNonoctalDecimalEscape,
    eslint,
    correctness,
    pending
);

impl Rule for NoNonoctalDecimalEscape {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::StringLiteral(literal) = node.kind() {
            check_string(ctx, literal.span.source_text(ctx.source_text()));
        }
    }
}
trait StickyRegex {
    fn sticky_captures<'h>(&self, haystack: &'h str, start: usize)
        -> (Option<Captures<'h>>, usize);
}

impl StickyRegex for Regex {
    fn sticky_captures<'h>(
        &self,
        haystack: &'h str,
        start: usize,
    ) -> (Option<Captures<'h>>, usize) {
        let c_opt = self.captures_at(haystack, start);
        if let Some(captures) = c_opt {
            let capture = captures.get(0).unwrap();
            if capture.start() == start {
                return (Some(captures), capture.end());
            }
        }
        (None, 0)
    }
}

fn quick_test(s: &str) -> bool {
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek().is_some_and(|c| *c == '8' || *c == '9') {
            return true;
        }
    }
    false
}

#[allow(clippy::cast_possible_truncation)]
fn check_string(ctx: &LintContext<'_>, string: &str) {
    lazy_static! {
        static ref NONOCTAL_REGEX: Regex =
            Regex::new(r"(?:[^\\]|(?P<previousEscape>\\.))*?(?P<decimalEscape>\\[89])").unwrap();
    }

    // Need at least 2 characters
    if string.len() <= 1 {
        return;
    }

    if !quick_test(string) {
        return;
    }

    let mut start: usize = 0;
    while let (Some(captures), new_start) = NONOCTAL_REGEX.sticky_captures(string, start) {
        let previous_escape = captures.name("previousEscape");
        let decimal_escape = captures.name("decimalEscape").unwrap();
        let decimal_escape_span =
            Span::new(decimal_escape.start() as u32, decimal_escape.end() as u32);
        let decimal_escape_str = decimal_escape.as_str();

        if let Some(prev_match) = previous_escape {
            if prev_match.as_str().eq("\\0") {
                ctx.diagnostic(replacement(
                    &(prev_match.as_str().to_string() + decimal_escape_str),
                    "\\u00008",
                    Span::new(prev_match.start() as u32, decimal_escape_span.end),
                ));
                ctx.diagnostic(replacement(decimal_escape_str, "\\u0038", decimal_escape_span));
            }
        } else {
            ctx.diagnostic(replacement(
                decimal_escape_str,
                &decimal_escape_str[1..],
                decimal_escape_span,
            ));
        }

        ctx.diagnostic(escape_backslash(
            decimal_escape_str,
            &format!("\\{decimal_escape_str}"),
            decimal_escape_span,
        ));

        start = new_start;
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"8",
        r"var \u8888",
        r"/\8/",
        r"''",
        r"'foo'",
        r"'8'",
        r"'9'",
        r"'foo8'",
        r"'foo9bar'",
        r"'\ '",
        r"'\\'",
        r"'\a'",
        r"'\n'",
        r"'\0'",
        r"'\1'",
        r"'\7'",
        r"'\01'",
        r"'\08'",
        r"'\19'",
        r"'\t9'",
        r"'\👍8'",
        r"'\\8'",
        r"'\\9'",
        r"'\\8\\9'",
        r"'\\ \\8'",
        r"'\\\\9'",
        r"'\\9bar'",
        r"'a\\8'",
        r"'foo\\8'",
        r"'foo\\8bar'",
        r"'9\\9'",
        r"'n\n8'",
        r"'n\nn\n8'",
        r"'\1.8'",
        r"'\1\28'",
        r"'\x99'",
        r"'\\\x38'",
        r"\u99999",
        r"'\\\n8'",
        r"'\\\n\\\\9'",
    ];

    let fail = vec![
        r"'\8'",
        r"'\9'",
        r#""\8""#,
        r"'f\9'",
        r"'xo\9'",
        r"'foo\9'",
        r"'foo\8bar'",
        r"'👍\8'",
        r"'\\\8'",
        r"'\\\\\9'",
        r"'foo\\\8'",
        r"'\ \8'",
        r"'\1\9'",
        r"'foo\1\9'",
        r"'\n\n\8\n'",
        r"'\n.\n\8\n'",
        r"'\n.\nn\8\n'",
        r"'\👍\8'",
        r"'\\8\9'",
        r"'\8\\9'",
        r"'\8 \\9'",
        r"'\8\8'",
        r"'\9\8'",
        r"'foo\8bar\9baz'",
        r"'\8\1\9'",
        r"'\9\n9\\9\9'",
        r"'\8\\\9'",
        r"var foo = '\8'; bar('\9')",
        r"var foo = '8'\n  bar = '\\9'",
        r"'\\n\8'",
        r"'\\n\9'",
        r"'\\\\n\8'",
        r"'foo\\nbar\9baz'",
        r"'\0\8'",
        r"'foo\0\9bar'",
        r"'\1\0\8'",
        r"'\0\8\9'",
        r"'\8\0\9'",
        r"'0\8'",
        r"'\\0\8'",
        r"'\0 \8'",
        r"'\01\8'",
        r"'\0\1\8'",
        r"'\0\\n\8'",
    ];

    Tester::new(NoNonoctalDecimalEscape::NAME, NoNonoctalDecimalEscape::PLUGIN, pass, fail)
        .test_and_snapshot();
}
