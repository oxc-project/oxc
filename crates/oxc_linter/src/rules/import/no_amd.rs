use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_amd_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use AMD `require` and `define` calls.")
        .with_help(format!("Expected imports instead of AMD {name}()"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAmd;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids the use of AMD `require` and `define` calls.
    ///
    /// ### Why is this bad?
    ///
    /// AMD (Asynchronous Module Definition) is an older module format
    /// that is less common in modern JavaScript development, especially
    /// with the widespread use of ES6 modules and CommonJS in Node.js.
    /// AMD introduces unnecessary complexity and is often considered outdated.
    /// This rule enforces the use of more modern module systems to improve
    /// maintainability and consistency across the codebase.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// require([a, b], function() {} );
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// require('../name');
    /// require(`../name`);
    /// ```
    NoAmd,
    import,
    restriction
);

/// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/no-amd.md>
impl Rule for NoAmd {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // not in top level
        if node.scope_id() != ctx.scopes().root_scope_id() {
            return;
        }
        if let AstKind::CallExpression(call_expr) = node.kind() {
            if let Expression::Identifier(identifier) = &call_expr.callee {
                if identifier.name != "define" && identifier.name != "require" {
                    return;
                }

                if call_expr.arguments.len() != 2 {
                    return;
                }

                if let Argument::ArrayExpression(_) = call_expr.arguments[0] {
                    ctx.diagnostic(no_amd_diagnostic(identifier.span, identifier.name.as_str()));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var _ = require('lodash')",
        "var find = require('lodash.find')",
        "var foo = require('./foo')",
        "var foo = require('../foo')",
        "var foo = require('foo')",
        "var foo = require('./')",
        "var foo = require('@scope/foo')",
        "var bar = require('./bar/index')",
        r#"import "x";"#,
        r#"import x from "x""#,
        r#"var x = require("x")"#,
        r#"require("x")"#,
        // 2-args, not an array
        r#"require("x", "y")"#,
        // random other function
        r"setTimeout(foo, 100)",
        // non-identifier callee
        r"(a || b)(1, 2, 3)",
        // nested scope is fine
        r#"function x() { define(["a"], function (a) {}) }"#,
        r#"function x() { require(["a"], function (a) {}) }"#,
        // unmatched arg types/number
        r"define(0, 1, 2)",
        r#"define("a")"#,
    ];

    let fail = vec![
        "require([a, b], function() {})",
        "define([a, b], function() {})",
        r"define([], function() {})",
        r#"define(["a"], function(a) { console.log(a); })"#,
        r"require([], function() {})",
        r#"require(["a"], function(a) { console.log(a); })"#,
    ];

    Tester::new(NoAmd::NAME, NoAmd::PLUGIN, pass, fail)
        .change_rule_path("no-amd.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
