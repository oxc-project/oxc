use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn detect_non_literal_require_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("require() called with a non-literal argument")
        .with_help("Avoid calling require() with dynamic expressions. This can lead to arbitrary module loading.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DetectNonLiteralRequire;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects calls to `require()` where the argument is not a string literal.
    ///
    /// ### Why is this bad?
    ///
    /// Calling `require()` with a non-literal argument allows an attacker to load
    /// arbitrary modules if they can control the input. This can lead to remote
    /// code execution.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// require(userInput);
    /// require(path + "/module");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// require("fs");
    /// require("./myModule");
    /// ```
    DetectNonLiteralRequire,
    oxc,
    suspicious,
    none
);

impl Rule for DetectNonLiteralRequire {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Expression::Identifier(callee) = &call_expr.callee else {
            return;
        };

        if callee.name != "require" {
            return;
        }

        let Some(arg) = call_expr.arguments.first().and_then(|a| a.as_expression()) else {
            return;
        };

        match arg {
            Expression::StringLiteral(_) => return,
            Expression::TemplateLiteral(tpl) if tpl.expressions.is_empty() => return,
            _ => {}
        }

        ctx.diagnostic(detect_non_literal_require_diagnostic(call_expr.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"require("fs")"#,
        r#"require("./myModule")"#,
        "require(`static-template`)",
        "console.log('hello')",
    ];

    let fail = vec![
        "require(userInput)",
        "require(path + '/module')",
        "require(getModuleName())",
        "require(`dynamic-${name}`)",
    ];

    Tester::new(DetectNonLiteralRequire::NAME, DetectNonLiteralRequire::PLUGIN, pass, fail)
        .test_and_snapshot();
}
