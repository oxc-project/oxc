use oxc_ast::AstKind;
use oxc_ast::ast::{Argument, Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn use_regex_literals_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use a regular expression literal instead of the `RegExp` constructor.")
        .with_help("Replace `new RegExp(\"pattern\")` with `/pattern/` for better readability and performance.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct UseRegexLiterals;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of the `RegExp` constructor with string literals
    /// that can be expressed as regular expression literals.
    ///
    /// ### Why is this bad?
    ///
    /// Regular expression literals are more concise, do not require escaping
    /// special characters, and are compiled at parse time rather than runtime.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// new RegExp("abc");
    /// RegExp("abc");
    /// new RegExp("abc", "g");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /abc/;
    /// /abc/g;
    /// new RegExp(variable);
    /// new RegExp("abc" + "def");
    /// ```
    UseRegexLiterals,
    eslint,
    style,
    pending
);

impl Rule for UseRegexLiterals {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NewExpression(expr) => {
                if let Expression::Identifier(ident) = &expr.callee {
                    if ident.name == "RegExp" && is_simple_regex_args(&expr.arguments) {
                        ctx.diagnostic(use_regex_literals_diagnostic(expr.span));
                    }
                }
            }
            AstKind::CallExpression(expr) => {
                if let Expression::Identifier(ident) = &expr.callee {
                    if ident.name == "RegExp" && is_simple_regex_args(&expr.arguments) {
                        ctx.diagnostic(use_regex_literals_diagnostic(expr.span));
                    }
                }
            }
            _ => {}
        }
    }
}

/// Check if the arguments are simple enough to be expressed as a regex literal:
/// - First arg is a string literal (the pattern)
/// - Optional second arg is a string literal (the flags)
fn is_simple_regex_args(args: &[Argument]) -> bool {
    if args.is_empty() || args.len() > 2 {
        return false;
    }

    // First argument must be a simple string literal
    let Some(Expression::StringLiteral(pattern)) = args[0].as_expression() else {
        return false;
    };

    // Check pattern doesn't contain newlines or other chars that can't go in a regex literal
    if pattern.value.contains('\n') || pattern.value.contains('\r') {
        return false;
    }

    // If there's a second argument, it must also be a string literal
    if args.len() == 2 {
        let Some(Expression::StringLiteral(_)) = args[1].as_expression() else {
            return false;
        };
    }

    true
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "/abc/;",
        "/abc/g;",
        "new RegExp(variable);",
        "RegExp(variable);",
        "new RegExp(variable, 'g');",
        "new RegExp('abc', flags);",
        "new RegExp('abc', 'g', extra);",
        "RegExp();",
    ];

    let fail = vec![
        "new RegExp('abc');",
        "RegExp('abc');",
        "new RegExp('abc', 'g');",
        "RegExp('abc', 'ig');",
        "new RegExp('test');",
    ];

    Tester::new(UseRegexLiterals::NAME, UseRegexLiterals::PLUGIN, pass, fail).test_and_snapshot();
}
