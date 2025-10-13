use std::ops::Deref;

use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    ast_util::is_method_call,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn prefer_structured_clone_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `structuredClone(…)` to create a deep clone.")
        .with_help("Switch to `structuredClone(…)`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferStructuredClone(Box<PreferStructuredCloneConfig>);

#[derive(Debug, Clone)]
pub struct PreferStructuredCloneConfig {
    allowed_functions: Vec<String>,
}

impl Default for PreferStructuredCloneConfig {
    fn default() -> Self {
        Self { allowed_functions: vec!["cloneDeep".to_string(), "utils.clone".to_string()] }
    }
}

impl Deref for PreferStructuredClone {
    type Target = PreferStructuredCloneConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer using structuredClone to create a deep clone.
    ///
    /// ### Why is this bad?
    ///
    /// structuredClone is the modern way to create a deep clone of a value.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const clone = JSON.parse(JSON.stringify(foo));
    ///
    /// const clone = _.cloneDeep(foo);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const clone = structuredClone(foo);
    /// ```
    PreferStructuredClone,
    unicorn,
    style,
    suggestion,
);

impl Rule for PreferStructuredClone {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);

        let allowed_functions = config
            .and_then(|config| config.get("functions"))
            .and_then(serde_json::Value::as_array)
            .map(|v| {
                v.iter().filter_map(serde_json::Value::as_str).map(ToString::to_string).collect()
            })
            .unwrap_or(vec![String::from("cloneDeep"), String::from("utils.clone")]);

        Self(Box::new(PreferStructuredCloneConfig { allowed_functions }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.arguments.len() != 1 {
            return;
        }

        if call_expr.optional {
            return;
        }

        if is_method_call(call_expr, Some(&["JSON"]), Some(&["parse"]), Some(1), Some(1)) {
            let Some(first_argument) = call_expr.arguments[0].as_expression() else {
                return;
            };

            let Expression::CallExpression(inner_call_expr) = first_argument.without_parentheses()
            else {
                return;
            };

            if inner_call_expr.optional {
                return;
            }

            if !is_method_call(
                inner_call_expr,
                Some(&["JSON"]),
                Some(&["stringify"]),
                Some(1),
                Some(1),
            ) {
                return;
            }

            let Some(first_argument) = inner_call_expr.arguments[0].as_expression() else {
                return;
            };

            ctx.diagnostic_with_suggestion(
                prefer_structured_clone_diagnostic(call_expr.span),
                |fixer| replace_with_structured_clone(fixer, call_expr, first_argument),
            );
        } else if let Some(first_argument) = call_expr.arguments[0].as_expression() {
            for function in &self.allowed_functions {
                if let Some((object, method)) = function.split_once('.') {
                    if is_method_call(call_expr, Some(&[object]), Some(&[method]), None, None) {
                        ctx.diagnostic_with_suggestion(
                            prefer_structured_clone_diagnostic(call_expr.span),
                            |fixer| replace_with_structured_clone(fixer, call_expr, first_argument),
                        );
                    }
                } else if is_method_call(call_expr, None, Some(&[function]), None, None)
                    || is_method_call(call_expr, Some(&[function]), None, None, None)
                    || call_expr.callee.is_specific_id(function)
                {
                    ctx.diagnostic_with_suggestion(
                        prefer_structured_clone_diagnostic(call_expr.span),
                        |fixer| replace_with_structured_clone(fixer, call_expr, first_argument),
                    );
                }
            }
        }
    }
}

fn replace_with_structured_clone(
    fixer: RuleFixer<'_, '_>,
    call_expr: &CallExpression<'_>,
    first_argument: &Expression<'_>,
) -> RuleFix {
    let mut codegen = fixer.codegen();
    codegen.print_str("structuredClone(");
    codegen.print_expression(first_argument);
    codegen.print_str(")");
    fixer.replace(call_expr.span, codegen.into_source_text())
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("structuredClone(foo)", None),
        ("JSON.parse(new JSON.stringify(foo))", None),
        ("new JSON.parse(JSON.stringify(foo))", None),
        ("JSON.parse(JSON.stringify())", None),
        ("JSON.parse(JSON.stringify(...foo))", None),
        ("JSON.parse(JSON.stringify(foo, extraArgument))", None),
        ("JSON.parse(...JSON.stringify(foo))", None),
        ("JSON.parse(JSON.stringify(foo), extraArgument)", None),
        ("JSON.parse(JSON.stringify?.(foo))", None),
        ("JSON.parse(JSON?.stringify(foo))", None),
        ("JSON.parse?.(JSON.stringify(foo))", None),
        // ("JSON?.parse(JSON.stringify(foo))", None),
        ("JSON.parse(JSON.not_stringify(foo))", None),
        ("JSON.parse(not_JSON.stringify(foo))", None),
        ("JSON.not_parse(JSON.stringify(foo))", None),
        ("not_JSON.parse(JSON.stringify(foo))", None),
        ("JSON.stringify(JSON.parse(foo))", None),
        ("JSON.parse(JSON.stringify(foo, undefined, 2))", None),
        ("new _.cloneDeep(foo)", None),
        ("notMatchedFunction(foo)", None),
        ("_.cloneDeep()", None),
        ("_.cloneDeep(...foo)", None),
        ("_.cloneDeep(foo, extraArgument)", None),
        // ("_.cloneDeep?.(foo)", None),
        // ("_?.cloneDeep(foo)", None),
    ];

    let fail = vec![
        ("JSON.parse((JSON.stringify((foo))))", None),
        ("JSON.parse(JSON.stringify(foo))", None),
        ("JSON.parse(JSON.stringify(foo),)", None),
        ("JSON.parse(JSON.stringify(foo,))", None),
        ("JSON.parse(JSON.stringify(foo,),)", None),
        ("JSON.parse( ((JSON.stringify)) (foo))", None),
        ("(( JSON.parse)) (JSON.stringify(foo))", None),
        ("JSON.parse(JSON.stringify( ((foo)) ))", None),
        ("JSON.parse(JSON.stringify( ((foo.bar['hello'])) ))", None),
        (
            "
            function foo() {
                    return JSON
                            .parse(
                                    JSON.
                                            stringify(
                                                    bar,
                                            ),
                            );
            }
            ",
            None,
        ),
        ("_.cloneDeep(foo)", None),
        ("lodash.cloneDeep(foo)", None),
        ("lodash.cloneDeep(foo,)", None),
        (
            "myCustomDeepCloneFunction(foo,)",
            Some(serde_json::json!([{"functions": ["myCustomDeepCloneFunction"]}])),
        ),
        ("my.cloneDeep(foo,)", Some(serde_json::json!([{"functions": ["my.cloneDeep"]}]))),
    ];

    let fix = vec![
        ("JSON.parse((JSON.stringify((foo))))", "structuredClone(foo)", None),
        ("JSON.parse(JSON.stringify(foo))", "structuredClone(foo)", None),
        ("JSON.parse(JSON.stringify(foo),)", "structuredClone(foo)", None),
        ("JSON.parse(JSON.stringify(foo,))", "structuredClone(foo)", None),
        ("JSON.parse(JSON.stringify(foo,),)", "structuredClone(foo)", None),
        ("JSON.parse( ((JSON.stringify)) (foo))", "structuredClone(foo)", None),
        ("(( JSON.parse)) (JSON.stringify(foo))", "structuredClone(foo)", None),
        ("JSON.parse(JSON.stringify( ((foo)) ))", "structuredClone(foo)", None),
        (
            "JSON.parse(JSON.stringify( ((foo.bar['hello'])) ))",
            "structuredClone(foo.bar['hello'])",
            None,
        ),
        (
            "
            function foo() {
                    return JSON
                            .parse(
                                    JSON.
                                            stringify(
                                                    bar,
                                            ),
                            );
            }
            ",
            "
            function foo() {
                    return structuredClone(bar);
            }
            ",
            None,
        ),
        ("_.cloneDeep(foo)", "structuredClone(foo)", None),
        ("lodash.cloneDeep(foo)", "structuredClone(foo)", None),
        ("lodash.cloneDeep(foo,)", "structuredClone(foo)", None),
        (
            "myCustomDeepCloneFunction(foo,)",
            "structuredClone(foo)",
            Some(serde_json::json!([{"functions": ["myCustomDeepCloneFunction"]}])),
        ),
        (
            "my.cloneDeep(foo,)",
            "structuredClone(foo)",
            Some(serde_json::json!([{"functions": ["my.cloneDeep"]}])),
        ),
    ];

    Tester::new(PreferStructuredClone::NAME, PreferStructuredClone::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
