use oxc_ast::ast::{Argument, Expression};
use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-import(no-amd): Do not use AMD `require` and `define` calls.")]
#[diagnostic(severity(warning), help("Expected imports instead of AMD {1}()"))]
struct NoAmdDiagnostic(#[label] pub Span, Atom);

#[derive(Debug, Default, Clone)]
pub struct NoAmd;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbid AMD `require` and `define` calls.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // fail
    /// require([a, b], function() {} );
    /// // pass
    /// require('../name');
    /// require(`../name`);
    /// ```
    NoAmd,
    nursery
);

/// https://github.com/import-js/eslint-plugin-import/blob/main/src/rules/no-amd.js
impl Rule for NoAmd {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // not in top level
        if node.scope_id() != ctx.scopes().root_scope_id() {
            return;
        }
        if let AstKind::CallExpression(call_expr) = node.kind() {
            if let Expression::Identifier(ref identifier) = &call_expr.callee {
                if identifier.name != "define" && identifier.name != "require" {
                    return;
                }

                if call_expr.arguments.len() != 2 {
                    return;
                }

                if let Argument::Expression(Expression::ArrayExpression(_)) = call_expr.arguments[0]
                {
                    ctx.diagnostic(NoAmdDiagnostic(identifier.span, identifier.name.clone()));
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
        r#"setTimeout(foo, 100)"#,
        // non-identifier callee
        r#"(a || b)(1, 2, 3)"#,
        // nested scope is fine
        r#"function x() { define(["a"], function (a) {}) }"#,
        r#"function x() { require(["a"], function (a) {}) }"#,
        // unmatched arg types/number
        r#"define(0, 1, 2)"#,
        r#"define("a")"#,
    ];

    let fail = vec![
        "require([a, b], function() {})",
        "define([a, b], function() {})",
        r#"define([], function() {})"#,
        r#"define(["a"], function(a) { console.log(a); })"#,
        r#"require([], function() {})"#,
        r#"require(["a"], function(a) { console.log(a); })"#,
    ];

    Tester::new_without_config(NoAmd::NAME, pass, fail)
        .change_rule_path("no-amd.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
