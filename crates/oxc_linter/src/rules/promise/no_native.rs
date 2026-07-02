use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_native_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("\"Promise\" is not defined")
        .with_help(
            "Bring your own `Promise` into scope first (e.g. `var Promise = require('bluebird')`) so the code still runs where there is no native one.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNative;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires a `Promise` to be brought into scope explicitly — via an import
    /// or an assignment — before it is used, rather than leaning on the native
    /// global.
    ///
    /// ### Why is this bad?
    ///
    /// Ancient ES5 engines (looking at you, IE11) ship no native `Promise`, so
    /// reaching for the global there throws at runtime. Declaring your own keeps
    /// the code portable and makes the polyfill obvious to the next reader.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// new Promise(function (resolve, reject) {});
    /// var x = Promise.resolve("good");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// var Promise = require("bluebird");
    /// var x = Promise.resolve("good");
    /// ```
    NoNative,
    promise,
    restriction,
    version = "1.71.0",
    short_description = "Require creating a `Promise` constructor before using it in an ES5 environment.",
);

impl Rule for NoNative {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IdentifierReference(ident) = node.kind() else {
            return;
        };

        if ident.name == "Promise" && ctx.is_reference_to_global_variable(ident) {
            ctx.diagnostic(no_native_diagnostic(ident.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"var Promise = null; function x() { return Promise.resolve("hi"); }"#,
        r#"var Promise = window.Promise || require("bluebird"); var x = Promise.reject();"#,
        r#"import Promise from "bluebird"; var x = Promise.reject();"#,
        "function foo(Promise) { return Promise.resolve(); }",
    ];

    let fail = vec![
        "new Promise(function (resolve, reject) {})",
        "Promise.resolve()",
        r#"Promise.reject(new Error("oops"))"#,
        "var x = Promise.all([a, b]);",
    ];

    Tester::new(NoNative::NAME, NoNative::PLUGIN, pass, fail).test_and_snapshot();
}
