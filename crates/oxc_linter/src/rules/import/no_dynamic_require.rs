use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_dnyamic_require_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected a literal string or immutable template literal")
        .with_help("Replace the argument with a literal string or immutable template literal")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDynamicRequire {
    esmodule: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbid imports which use an expression for the module argument.
    ///
    /// ### Why is this bad?
    ///
    /// Import statements which use an expression resolved at runtime makes it to find where the
    /// import comes from and some static code analysis tools might not be able to resolve them.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// require(name);
    /// require(`../${name}`);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// require('../name');
    /// require(`../name`);
    /// ```
    NoDynamicRequire,
    restriction,
);

impl Rule for NoDynamicRequire {
    fn from_configuration(value: serde_json::Value) -> Self {
        let esmodule = value
            .get(0)
            .and_then(|config| config.get("esmodule"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Self { esmodule }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportExpression(import) => {
                if self.esmodule && !is_static_value(&import.source) {
                    ctx.diagnostic(no_dnyamic_require_diagnostic(import.source.span()));
                }
            }
            AstKind::CallExpression(call) => {
                if call.arguments.is_empty() {
                    return;
                }

                if !call.callee.is_specific_id("require") {
                    return;
                }

                let Some(expr) = &call.arguments[0].as_expression() else {
                    return;
                };

                if !is_static_value(expr) {
                    ctx.diagnostic(no_dnyamic_require_diagnostic(call.callee.span()));
                }
            }
            _ => {}
        };
    }
}

fn is_static_value(expr: &Expression) -> bool {
    expr.is_string_literal() && expr.is_immutable_value()
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r#"import _ from "lodash""#, None),
        (r#"require("foo")"#, None),
        ("require(`foo`)", None),
        (r#"require("./foo")"#, None),
        (r#"require("@scope/foo")"#, None),
        ("require()", None),
        (r#"require("./foo", "bar" + "okay")"#, None),
        (r#"var foo = require("foo")"#, None),
        ("var foo = require(`foo`)", None),
        (r#"var foo = require("./foo")"#, None),
        (r#"var foo = require("@scope/foo")"#, None),
        (r#"import("foo")"#, Some(json!([{ "esmodule": true }]))),
        ("import(`foo`)", Some(json!([{ "esmodule": true }]))),
        (r#"import("./foo")"#, Some(json!([{ "esmodule": true }]))),
        (r#"import("@scope/foo")"#, Some(json!([{ "esmodule": true }]))),
        (r#"var foo = import("foo")"#, Some(json!([{ "esmodule": true }]))),
        ("var foo = import(`foo`)", Some(json!([{ "esmodule": true }]))),
        (r#"var foo = import("./foo")"#, Some(json!([{ "esmodule": true }]))),
        (r#"var foo = import("@scope/foo")"#, Some(json!([{ "esmodule": true }]))),
    ];

    let fail = vec![
        (r#"require("../" + name)"#, None),
        ("require(`../${name}`)", None),
        ("require(name)", None),
        ("require(name())", None),
        ("require(`foo${x}`)", None),
        ("var foo = require(`foo${x}`)", None),
        (r#"require(name + "foo", "bar")"#, Some(json!([{ "esmodule": true }]))),
        (r#"import("../" + "name")"#, Some(json!([{ "esmodule": true }]))),
        ("import(`../${name}`)", Some(json!([{ "esmodule": true }]))),
        ("import(name)", Some(json!([{ "esmodule": true }]))),
        ("import(name())", Some(json!([{ "esmodule": true }]))),
    ];

    Tester::new(NoDynamicRequire::NAME, pass, fail).test_and_snapshot();
}
