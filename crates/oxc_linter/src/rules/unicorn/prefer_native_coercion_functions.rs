use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, FormalParameters, FunctionBody, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule, utils::get_first_parameter_name};

fn function(span: Span, called_fn: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The function is equivalent to `{called_fn}`. Call `{called_fn}` directly."
    ))
    .with_label(span)
}

fn array_callback(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The arrow function in the callback of the array is equivalent to `Boolean`. Replace the callback with `Boolean`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferNativeCoercionFunctions;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers built in functions, over custom ones with the same functionality.
    ///
    /// ### Why is this bad?
    ///
    /// If a function is equivalent to [`String`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String), [`Number`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number), [`BigInt`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt), [`Boolean`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Boolean), or [`Symbol`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol), you should use the built-in one directly.
    /// Wrapping the built-in in a function is moot.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = v => String(v);
    /// foo(1);
    /// const foo = v => Number(v);
    /// array.some((v, ) => /* comment */ v)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// String(1);
    /// Number(1);
    /// array.some(Boolean);
    /// ```
    PreferNativeCoercionFunctions,
    unicorn,
    pedantic,
    pending
);

impl Rule for PreferNativeCoercionFunctions {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ArrowFunctionExpression(arrow_expr) => {
                if arrow_expr.r#async || arrow_expr.params.items.is_empty() {
                    return;
                }

                if let Some(call_expr_ident) =
                    check_function(&arrow_expr.params, &arrow_expr.body, true)
                {
                    ctx.diagnostic(function(arrow_expr.span, call_expr_ident));
                }

                if check_array_callback_methods(
                    node.id(),
                    &arrow_expr.params,
                    &arrow_expr.body,
                    true,
                    ctx,
                ) {
                    ctx.diagnostic(array_callback(arrow_expr.span));
                }
            }
            AstKind::Function(func) => {
                if func.r#async || func.generator || func.params.items.is_empty() {
                    return;
                }
                if matches!(ctx.nodes().parent_kind(node.id()), AstKind::ObjectProperty(_)) {
                    return;
                }
                if let Some(function_body) = &func.body
                    && let Some(call_expr_ident) =
                        check_function(&func.params, function_body, false)
                {
                    ctx.diagnostic(function(func.span, call_expr_ident));
                }
            }
            _ => {}
        }
    }
}

fn check_function<'a>(
    arg: &'a FormalParameters,
    function_body: &'a FunctionBody,
    is_arrow: bool,
) -> Option<&'a str> {
    let first_parameter_name = get_first_parameter_name(arg)?;

    if function_body.statements.len() != 1 {
        return None;
    }

    if is_arrow && let Statement::ExpressionStatement(expr_stmt) = &function_body.statements[0] {
        return is_matching_native_coercion_function_call(
            &expr_stmt.expression,
            first_parameter_name,
        );
    }

    if let Statement::ReturnStatement(return_statement) = &function_body.statements[0]
        && let Some(return_expr) = &return_statement.argument
    {
        return is_matching_native_coercion_function_call(return_expr, first_parameter_name);
    }

    None
}

fn get_returned_ident<'a>(stmt: &'a Statement, is_arrow: bool) -> Option<&'a str> {
    if is_arrow && let Statement::ExpressionStatement(expr_stmt) = &stmt {
        return expr_stmt
            .expression
            .without_parentheses()
            .get_identifier_reference()
            .map(|v| v.name.as_str());
    }

    if let Statement::BlockStatement(block_stmt) = &stmt {
        if block_stmt.body.len() != 1 {
            return None;
        }
        return get_returned_ident(&block_stmt.body[0], is_arrow);
    }
    if let Statement::ReturnStatement(return_statement) = &stmt
        && let Some(return_expr) = &return_statement.argument
    {
        return return_expr
            .without_parentheses()
            .get_identifier_reference()
            .map(|v| v.name.as_str());
    }

    None
}

fn is_matching_native_coercion_function_call<'a>(
    expr: &'a Expression,
    first_arg_name: &'a str,
) -> Option<&'a str> {
    let Expression::CallExpression(call_expr) = expr else {
        return None;
    };

    if call_expr.optional || call_expr.arguments.is_empty() {
        return None;
    }

    let Expression::Identifier(callee_ident) = &call_expr.callee else {
        return None;
    };

    let fn_name = callee_ident.name.as_str();

    if !NATIVE_COERCION_FUNCTION_NAMES.contains(&fn_name) {
        return None;
    }

    let Argument::Identifier(arg_ident) = &call_expr.arguments[0] else {
        return None;
    };

    if arg_ident.name == first_arg_name {
        return Some(fn_name);
    }
    None
}

fn check_array_callback_methods(
    node_id: NodeId,
    arg: &FormalParameters,
    function_body: &FunctionBody,
    is_arrow: bool,
    ctx: &LintContext,
) -> bool {
    let parent = ctx.nodes().parent_node(node_id);

    let AstKind::CallExpression(call_expr) = parent.kind() else {
        return false;
    };
    if call_expr
        .arguments
        .first()
        .is_none_or(|arg| arg.span() != ctx.nodes().get_node(node_id).kind().span())
    {
        return false;
    }
    if call_expr.optional {
        return false;
    }

    let Some(callee_member_expr) = call_expr.callee.as_member_expression() else {
        return false;
    };
    if callee_member_expr.optional() {
        return false;
    }
    let Some(method_name) = callee_member_expr.static_property_name() else {
        return false;
    };
    if !ARRAY_METHODS_WITH_BOOLEAN_CALLBACK.contains(&method_name) {
        return false;
    }

    let Some(first_param_name) = get_first_parameter_name(arg) else {
        return false;
    };

    let Some(first_stmt) = function_body.statements.first() else {
        return false;
    };

    let Some(returned_ident) = get_returned_ident(first_stmt, is_arrow) else {
        return false;
    };

    first_param_name == returned_ident
}

const NATIVE_COERCION_FUNCTION_NAMES: [&str; 5] =
    ["BigInt", "Boolean", "Number", "String", "Symbol"];

const ARRAY_METHODS_WITH_BOOLEAN_CALLBACK: [&str; 7] =
    ["every", "filter", "find", "findIndex", "findLast", "findLastIndex", "some"];

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo = async v => String(v)",
        "const foo = v => String",
        "const foo = v => v",
        "const foo = v => NotString(v)",
        "const foo = v => String(notFirstParameterName)",
        "const foo = v => new String(v)",
        "const foo = v => String?.(v)",
        "const foo = async function (v) {return String(v);}",
        "const foo = function * (v) {return String(v);}",
        "const foo = async function * (v) {return String(v);}",
        "const foo = function * (v) {yield String(v);}",
        "const foo = async function (v) {await String(v);}",
        "const foo = function (v) {return;}",
        "({get foo() {return String(v)}})",
        "({set foo(v) {return String(v)}})",
        "array.some?.(v => v)",
        "array?.some(v => v)",
        "array.notSome(v => v)",
        "array.some(callback, v => v)",
        "some(v => v)",
        "array.some(v => notFirstParameterName)",
        "array.some(function(v) {return notFirstParameterName;})",
        "array.some(function(v) {return;})",
        "array.some(function(v) {return v.v;})",
        "cells.every((cellRowIdx, cellColIdx, tableLoop, cellLoop) => {});",
    ];

    let fail = vec![
        "const foo = v => String(v)",
        "const foo = v => Number(v)",
        "const foo = v => BigInt(v)",
        "const foo = v => Boolean(v)",
        "const foo = v => Symbol(v)",
        "function foo(v) { return String(v); }",
        "export default function foo(v) { return String(v); }",
        "export default function (v) { return String(v); }",
        "const foo = (v, extra) => String(v)",
        "const foo = (v, ) => String(v, extra)",
        "const foo = (v, ) => /* comment */ String(v)",
        "array.every(v => v)",
        "array.filter(v => v)",
        "array.find(v => v)",
        "array.findLast(v => v)",
        "array.some(v => v)",
        "array.findIndex(v => v)",
        "array.findLastIndex(v => v)",
        "array.some(v => v)",
        "array.some((v, extra) => v)",
        "array.some((v, ) => /* comment */ v)",
    ];

    Tester::new(
        PreferNativeCoercionFunctions::NAME,
        PreferNativeCoercionFunctions::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
