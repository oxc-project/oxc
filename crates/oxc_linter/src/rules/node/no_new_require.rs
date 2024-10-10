use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_new_require(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected use of new with require")
        .with_label(span)
        .with_help("Initialise the constructor separate from the import statement")
}

#[derive(Debug, Default, Clone)]
pub struct NoNewRequire;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Warn about calling `new` on `require`.
    ///
    /// ### Why is this bad?
    ///
    /// The `require` function is used to include modules and might return a constructor. As this
    /// is not always the case this can be confusing.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var appHeader = new require('app-header');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var AppHeader = require('app-header');
    /// var appHeader = new AppHeader();
    /// ```
    NoNewRequire,
    restriction);

impl Rule for NoNewRequire {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expression) = node.kind() else {
            return;
        };

        if !new_expression.callee.is_specific_id("require") {
            return;
        };

        ctx.diagnostic(no_new_require(new_expression.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var appHeader = require('app-header')",
        "var AppHeader = new (require('app-header'))",
        "var AppHeader = new (require('headers').appHeader)",
        "var AppHeader = require('app-header'); var appHeader = new AppHeader();",
    ];

    let fail = vec![
        "var appHeader = new require('app-header')",
        "var appHeader = new require('headers').appHeader",
    ];

    Tester::new(NoNewRequire::NAME, pass, fail).test_and_snapshot();
}
