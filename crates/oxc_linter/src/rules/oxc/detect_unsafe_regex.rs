use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn detect_unsafe_regex_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unsafe regular expression: potential ReDoS vulnerability")
        .with_help("Avoid regular expressions with nested quantifiers or overlapping alternations that can cause catastrophic backtracking. Simplify the regex or use a linear-time regex engine.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DetectUnsafeRegex;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects regular expressions that are potentially vulnerable to Regular
    /// Expression Denial of Service (ReDoS).
    ///
    /// ### Why is this bad?
    ///
    /// Regular expressions with certain patterns (nested quantifiers, overlapping
    /// alternations) can take exponential time to evaluate on crafted input. An
    /// attacker can exploit this to cause denial of service.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const re = /(a+)+$/;
    /// const re = /([a-zA-Z]+)*$/;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const re = /^[a-z]+$/;
    /// const re = /^\d{1,10}$/;
    /// ```
    DetectUnsafeRegex,
    oxc,
    suspicious,
    none
);

/// Simple heuristic check for potentially unsafe regex patterns.
/// Looks for nested quantifiers like (a+)+, (a*)+, (a+)*, etc.
fn is_unsafe_pattern(pattern: &str) -> bool {
    // Check for nested quantifiers: a group with a quantifier followed by another quantifier
    // Patterns like (x+)+, (x+)*, (x*)+, (x*)*
    let bytes = pattern.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'(' {
            // Find matching closing paren
            let mut depth = 1;
            let mut j = i + 1;
            let mut has_quantifier_inside = false;

            while j < len && depth > 0 {
                match bytes[j] {
                    b'\\' => {
                        j += 1; // skip escaped char
                    }
                    b'(' => depth += 1,
                    b')' => depth -= 1,
                    b'+' | b'*' if depth == 1 => {
                        has_quantifier_inside = true;
                    }
                    _ => {}
                }
                j += 1;
            }

            // j is now past the closing paren
            if has_quantifier_inside
                && j < len
                && (bytes[j] == b'+' || bytes[j] == b'*' || bytes[j] == b'{')
            {
                return true;
            }
        }
        i += 1;
    }

    false
}

impl Rule for DetectUnsafeRegex {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::RegExpLiteral(regexp) => {
                if is_unsafe_pattern(regexp.regex.pattern.text.as_str()) {
                    ctx.diagnostic(detect_unsafe_regex_diagnostic(regexp.span));
                }
            }
            AstKind::NewExpression(new_expr) => {
                let Expression::Identifier(callee) = &new_expr.callee else {
                    return;
                };
                if callee.name != "RegExp" {
                    return;
                }
                let Some(Expression::StringLiteral(pattern)) =
                    new_expr.arguments.first().and_then(|a| a.as_expression())
                else {
                    return;
                };
                if is_unsafe_pattern(pattern.value.as_str()) {
                    ctx.diagnostic(detect_unsafe_regex_diagnostic(new_expr.span));
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const re = /^[a-z]+$/",
        r"const re = /^\d{1,10}$/",
        "const re = /foo|bar/",
        r"const re = /\w+/",
        r#"const re = new RegExp("^[a-z]+$")"#,
    ];

    let fail = vec![
        "const re = /(a+)+$/",
        "const re = /([a-zA-Z]+)*$/",
        "const re = /(a+){2,}/",
        r#"const re = new RegExp("(a+)+$")"#,
    ];

    Tester::new(DetectUnsafeRegex::NAME, DetectUnsafeRegex::PLUGIN, pass, fail).test_and_snapshot();
}
