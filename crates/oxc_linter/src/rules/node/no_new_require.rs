use oxc_ast::ast::Expression;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    AstNode,
};

fn no_new_require_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`const myInstance = new require('some-module')` is likely unintended as it does not invoke the constructor exported from `'some-module'`.")
        .with_help("Replace with one of the correct alternatives instead. `const myInstance = new (require('some-module'))`, `const myInstance = new (require('some-module'))`, or `const MyConstructor = require('some-module'); const myInstance = new MyConstructor();`")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNewRequire;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the likely incorrect code of `new require('some-module')`.
    ///
    /// ### Why is this bad?
    ///
    /// The code author likely intended to use `new (require('some-module'))` instead to invoke the constructor returned by `require('some-module')`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const myInstance = new require('some-module');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const SomeModule = require('some-module');
    /// const myInstance = new SomeModule();
    ///
    /// const myInstance = new (require('some-module'))();
    /// ```
    NoNewRequire,
    suspicious,

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for NoNewRequire {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };

        let Expression::Identifier(ident) = &new_expr.callee.without_parentheses() else {
            return;
        };
        if ident.name != "require" {
            return;
        }
        ctx.diagnostic(no_new_require_diagnostic(ident.span))
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var appHeader = require('app-header')",
        "var AppHeader = new (require('app-header'))",
        "var AppHeader = new (require('app-header'))()",
        "var AppHeader = new (require('headers').appHeader)",
    ];

    let fail = vec![
        "var appHeader = new require('app-header')",
        "var appHeader = new require('headers').appHeader",
    ];

    Tester::new(NoNewRequire::NAME, pass, fail).test_and_snapshot();
}
