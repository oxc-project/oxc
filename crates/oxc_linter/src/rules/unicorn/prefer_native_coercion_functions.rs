use oxc_ast::{
    ast::{Argument, Expression, FormalParameters, FunctionBody, Statement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::get_first_parameter_name, AstNode};

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
                if arrow_expr.r#async || arrow_expr.params.items.len() == 0 {
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
                if func.r#async || func.generator || func.params.items.len() == 0 {
                    return;
                }
                if let Some(parent) = ctx.nodes().parent_node(node.id()) {
                    if matches!(parent.kind(), AstKind::ObjectProperty(_)) {
                        return;
                    }
                }
                if let Some(function_body) = &func.body {
                    if let Some(call_expr_ident) =
                        check_function(&func.params, function_body, false)
                    {
                        ctx.diagnostic(function(func.span, call_expr_ident));
                    }
                }
            }
            _ => {}
        }
    }
}

fn check_function(
    arg: &FormalParameters,
    function_body: &FunctionBody,
    is_arrow: bool,
) -> Option<&'static str> {
    let first_parameter_name = get_first_parameter_name(arg)?;

    if function_body.statements.len() != 1 {
        return None;
    }

    if is_arrow {
        if let Statement::ExpressionStatement(expr_stmt) = &function_body.statements[0] {
            return is_matching_native_coercion_function_call(
                &expr_stmt.expression,
                first_parameter_name,
            );
        }
    }

    if let Statement::ReturnStatement(return_statement) = &function_body.statements[0] {
        if let Some(return_expr) = &return_statement.argument {
            return is_matching_native_coercion_function_call(return_expr, first_parameter_name);
        }
    }

    None
}

fn get_returned_ident<'a>(stmt: &'a Statement, is_arrow: bool) -> Option<&'a str> {
    if is_arrow {
        if let Statement::ExpressionStatement(expr_stmt) = &stmt {
            return expr_stmt
                .expression
                .without_parentheses()
                .get_identifier_reference()
                .map(|v| v.name.as_str());
        }
    }

    if let Statement::BlockStatement(block_stmt) = &stmt {
        if block_stmt.body.len() != 1 {
            return None;
        }
        return get_returned_ident(&block_stmt.body[0], is_arrow);
    }
    if let Statement::ReturnStatement(return_statement) = &stmt {
        if let Some(return_expr) = &return_statement.argument {
            return return_expr
                .without_parentheses()
                .get_identifier_reference()
                .map(|v| v.name.as_str());
        }
    }

    None
}

fn is_matching_native_coercion_function_call(
    expr: &Expression,
    first_arg_name: &str,
) -> Option<&'static str> {
    let Expression::CallExpression(call_expr) = expr else {
        return None;
    };

    if call_expr.optional || call_expr.arguments.len() == 0 {
        return None;
    }

    let Expression::Identifier(callee_ident) = &call_expr.callee else {
        return None;
    };

    let fn_name = NATIVE_COERCION_FUNCTION_NAMES.get_key(callee_ident.name.as_str())?;

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
    let Some(parent) = ctx.nodes().parent_node(node_id) else {
        return false;
    };
    let AstKind::Argument(parent_call_expr_arg) = parent.kind() else {
        return false;
    };
    let Some(grand_parent) = ctx.nodes().parent_node(parent.id()) else {
        return false;
    };
    let AstKind::CallExpression(call_expr) = grand_parent.kind() else {
        return false;
    };

    if !std::ptr::eq(&call_expr.arguments[0], parent_call_expr_arg) {
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
    if !ARRAY_METHODS_WITH_BOOLEAN_CALLBACK.contains(method_name) {
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

const NATIVE_COERCION_FUNCTION_NAMES: phf::Set<&'static str> = phf::phf_set! {
    "String",
    "Number",
    "BigInt",
    "Boolean",
    "Symbol"
};

const ARRAY_METHODS_WITH_BOOLEAN_CALLBACK: phf::Set<&'static str> = phf::phf_set! {
    "every",
    "filter",
    "find",
    "findLast",
    "findIndex",
    "findLastIndex",
    "some"
};

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const foo = async v => String(v)",
        r"const foo = v => String",
        r"const foo = v => v",
        r"const foo = v => NotString(v)",
        r"const foo = v => String(notFirstParameterName)",
        r"const foo = v => new String(v)",
        r"const foo = v => String?.(v)",
        r"const foo = async function (v) {return String(v);}",
        r"const foo = function * (v) {return String(v);}",
        r"const foo = async function * (v) {return String(v);}",
        r"const foo = function * (v) {yield String(v);}",
        r"const foo = async function (v) {await String(v);}",
        r"const foo = function (v) {return;}",
        r"({get foo() {return String(v)}})",
        r"({set foo(v) {return String(v)}})",
        r"array.some?.(v => v)",
        r"array?.some(v => v)",
        r"array.notSome(v => v)",
        r"array.some(callback, v => v)",
        r"some(v => v)",
        r"array.some(v => notFirstParameterName)",
        r"array.some(function(v) {return notFirstParameterName;})",
        r"array.some(function(v) {return;})",
        r"array.some(function(v) {return v.v;})",
        r"cells.every((cellRowIdx, cellColIdx, tableLoop, cellLoop) => {});",
    ];

    let fail = vec![
        r"const foo = v => String(v)",
        r"const foo = v => Number(v)",
        r"const foo = v => BigInt(v)",
        r"const foo = v => Boolean(v)",
        r"const foo = v => Symbol(v)",
        r"function foo(v) { return String(v); }",
        r"export default function foo(v) { return String(v); }",
        r"export default function (v) { return String(v); }",
        r"const foo = (v, extra) => String(v)",
        r"const foo = (v, ) => String(v, extra)",
        r"const foo = (v, ) => /* comment */ String(v)",
        r"array.every(v => v)",
        r"array.filter(v => v)",
        r"array.find(v => v)",
        r"array.findLast(v => v)",
        r"array.some(v => v)",
        r"array.findIndex(v => v)",
        r"array.findLastIndex(v => v)",
        r"array.some(v => v)",
        r"array.some((v, extra) => v)",
        r"array.some((v, ) => /* comment */ v)",
    ];

    Tester::new(
        PreferNativeCoercionFunctions::NAME,
        PreferNativeCoercionFunctions::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
