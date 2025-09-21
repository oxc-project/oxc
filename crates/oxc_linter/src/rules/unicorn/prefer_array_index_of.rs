use oxc_ast::{
    AstKind,
    ast::{Expression, FormalParameter, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::{AstNode, ast_util::is_method_call, context::LintContext, rule::Rule};

fn prefer_array_index_of_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer 'indexOf' over 'findIndex' for simple equality checks")
        .with_help("Use 'indexOf(value)' instead of 'findIndex(x => x === value)' for better clarity and performance")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferArrayIndexOf;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces using `indexOf` or `lastIndexOf` instead of `findIndex` or `findLastIndex`
    /// when the callback is a simple strict equality comparison.
    ///
    /// ### Why is this bad?
    ///
    /// Using `findIndex(x => x === value)` is unnecessarily verbose when `indexOf(value)`
    /// accomplishes the same thing more concisely and clearly. It also avoids the overhead
    /// of creating a callback function.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// values.findIndex(x => x === "foo");
    /// values.findLastIndex(x => x === "bar");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// values.indexOf("foo");
    /// values.lastIndexOf("bar");
    /// ```
    PreferArrayIndexOf,
    unicorn,
    style,
    pending
);

impl Rule for PreferArrayIndexOf {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_method_call(call_expr, None, Some(&["findIndex", "findLastIndex"]), Some(1), Some(1))
        {
            return;
        }

        let Some(cb) = call_expr.arguments[0].as_expression() else { return };
        if !is_simple_compare_callback_function(cb, ctx) {
            return;
        }

        ctx.diagnostic(prefer_array_index_of_diagnostic(
            call_expr
                .callee
                .as_member_expression()
                .and_then(oxc_ast::ast::MemberExpression::static_property_info)
                .map_or(call_expr.span, |(span, _)| span),
        ));
    }
}

fn is_simple_compare_callback_function(expr: &Expression, ctx: &LintContext) -> bool {
    fn is_simple_compare(arg: &FormalParameter, expr: &Expression, ctx: &LintContext) -> bool {
        let Some(ident) = arg.pattern.get_binding_identifier() else { return false };

        if let Expression::BinaryExpression(expr) = expr
            && ctx.symbol_references(ident.symbol_id()).count() == 1
            && expr.operator == BinaryOperator::StrictEquality
            && (expr
                .left
                .get_identifier_reference()
                .and_then(|ident| ctx.scoping().get_reference(ident.reference_id()).symbol_id())
                == Some(ident.symbol_id())
                || expr.right.get_identifier_reference().and_then(|ident| {
                    ctx.scoping().get_reference(ident.reference_id()).symbol_id()
                }) == Some(ident.symbol_id()))
        {
            return true;
        }
        false
    }

    match expr.get_inner_expression() {
        Expression::ArrowFunctionExpression(arrow_function)
            if !arrow_function.r#async && arrow_function.params.items.len() == 1 =>
        {
            let query = if arrow_function.expression {
                if let Some(Statement::ExpressionStatement(expr)) =
                    arrow_function.body.statements.first()
                {
                    Some(&expr.expression)
                } else {
                    None
                }
            } else if let Some(Statement::ReturnStatement(ret)) =
                arrow_function.body.statements.first()
            {
                ret.argument.as_ref()
            } else {
                None
            };

            query.is_some_and(|expr| is_simple_compare(&arrow_function.params.items[0], expr, ctx))
        }
        Expression::FunctionExpression(function)
            if !function.r#async && !function.generator && function.params.items.len() == 1 =>
        {
            let query = if let Some(Statement::ReturnStatement(ret)) =
                function.body.as_ref().and_then(|stmts| stmts.statements.first())
            {
                ret.argument.as_ref()
            } else {
                None
            };

            query.is_some_and(|expr| is_simple_compare(&function.params.items[0], expr, ctx))
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        "const findIndex = foo.findIndex",
        "foo.findIndex()",
        "foo.findIndex(function (x) {return x === 1;}, bar)",
        "foo.findIndex(...[function (x) {return x === 1;}])",
        "new foo.findIndex(x => x === 1)",
        "findIndex(x => x === 1)",
        "foo[findIndex](x => x === 1)",
        "foo.not_findIndex(x => x === 1)",
        "foo.findIndex(myFunction)",
        "foo.findIndex((x, i) => x === i)",
        "foo.findIndex((x, i) => {return x === i})",
        "foo.findIndex(function(x, i) {return x === i})",
        "foo.findIndex(({x}) => x === 1)",
        "foo.findIndex(({x}) => {return x === 1})",
        "foo.findIndex(function({x}) {return x === 1})",
        "foo.findIndex(function * (x) {return x === 1})",
        "foo.findIndex(async (x) => x === 1)",
        "foo.findIndex(async (x) => {return x === 1})",
        "foo.findIndex(async function(x) {return x === 1})",
        "foo.findIndex(({x}) => {noop();return x === 1})",
        "foo.findIndex(({x}) => {bar(x === 1)})",
        "foo.findIndex(x => x - 1)",
        "foo.findIndex(x => {return x - 1})",
        "foo.findIndex(function (x){return x - 1})",
        "foo.findIndex(x => x !== 1)",
        "foo.findIndex(x => {return x !== 1})",
        "foo.findIndex(function (x){return x !== 1})",
        "foo.findIndex(x => 1 === 1.0)",
        "foo.findIndex(x => {return 1 === 1.0})",
        "foo.findIndex(function (x){return 1 === 1.0})",
        "foo.findIndex(x => x === x)",
        "foo.findIndex(x => y === 1)",
        r#"foo.findIndex(x => x + "foo" === "foo" + x)"#,
        r#"foo.findIndex(x => x === "foo" + x)"#,
        r#"foo.findIndex(x => x === (function (){return x === "1"})())"#,
        "foo.indexOf(0)",
        "const findLastIndex = foo.findLastIndex",
        "foo.findLastIndex()",
        "foo.findLastIndex(function (x) {return x === 1;}, bar)",
        "foo.findLastIndex(...[function (x) {return x === 1;}])",
        "new foo.findLastIndex(x => x === 1)",
        "findLastIndex(x => x === 1)",
        "foo[findLastIndex](x => x === 1)",
        "foo.not_findLastIndex(x => x === 1)",
        "foo.findLastIndex(myFunction)",
        "foo.findLastIndex((x, i) => x === i)",
        "foo.findLastIndex((x, i) => {return x === i})",
        "foo.findLastIndex(function(x, i) {return x === i})",
        "foo.findLastIndex(({x}) => x === 1)",
        "foo.findLastIndex(({x}) => {return x === 1})",
        "foo.findLastIndex(function({x}) {return x === 1})",
        "foo.findLastIndex(function * (x) {return x === 1})",
        "foo.findLastIndex(async (x) => x === 1)",
        "foo.findLastIndex(async (x) => {return x === 1})",
        "foo.findLastIndex(async function(x) {return x === 1})",
        "foo.findLastIndex(({x}) => {noop();return x === 1})",
        "foo.findLastIndex(({x}) => {bar(x === 1)})",
        "foo.findLastIndex(x => x - 1)",
        "foo.findLastIndex(x => {return x - 1})",
        "foo.findLastIndex(function (x){return x - 1})",
        "foo.findLastIndex(x => x !== 1)",
        "foo.findLastIndex(x => {return x !== 1})",
        "foo.findLastIndex(function (x){return x !== 1})",
        "foo.findLastIndex(x => 1 === 1.0)",
        "foo.findLastIndex(x => {return 1 === 1.0})",
        "foo.findLastIndex(function (x){return 1 === 1.0})",
        "foo.findLastIndex(x => x === x)",
        "foo.findLastIndex(x => y === 1)",
        r#"foo.findLastIndex(x => x + "foo" === "foo" + x)"#,
        r#"foo.findLastIndex(x => x === "foo" + x)"#,
        "foo.findLastIndex(x => x === (function (){return x === \"1\"})())",
        "foo.lastIndexOf(0)",
    ];

    let fail = vec![
        "values.findIndex(x => x === \"foo\")",
        "values.findIndex(x => \"foo\" === x)",
        "values.findIndex(x => {return x === \"foo\";})",
        "values.findIndex(function (x) {return x === \"foo\";})",
        "// 1\n(0, values)\n\t// 2\n\t./* 3 */findIndex /* 3 */ (\n\t\t/* 4 */\n\t\tx /* 5 */ => /* 6 */ x /* 7 */ === /* 8 */ \"foo\" /* 9 */\n\t) /* 10 */",
        "foo.findIndex(function (element) {\n\treturn element === bar.findIndex(x => x === 1);\n});",
        "values.findIndex(x => x === (0, \"foo\"))",
        "values.findIndex((x => x === (0, \"foo\")))",
        "function fn() {\n\tfoo.findIndex(x => x === arguments.length)\n}",
        "function fn() {\n\tfoo.findIndex(x => x === this[1])\n}",
        "values.findIndex(x => x === foo())",
        "foo.findIndex(function a(x) {\n\treturn x === (function (a) {\n\t\treturn a(this) === arguments[1]\n\t}).call(thisObject, anotherFunctionNamedA, secondArgument)\n})",
        "function foo() {\n\treturn (bar as string).findIndex(x => x === \"foo\");\n}",
        "values.findLastIndex(x => x === \"foo\")",
        "values.findLastIndex(x => \"foo\" === x)",
        "values.findLastIndex(x => {return x === \"foo\";})",
        "values.findLastIndex(function (x) {return x === \"foo\";})",
        "// 1\n(0, values)\n\t// 2\n\t./* 3 */findLastIndex /* 3 */ (\n\t\t/* 4 */\n\t\tx /* 5 */ => /* 6 */ x /* 7 */ === /* 8 */ \"foo\" /* 9 */\n\t) /* 10 */",
        "foo.findLastIndex(function (element) {\n\treturn element === bar.findLastIndex(x => x === 1);\n});",
        "values.findLastIndex(x => x === (0, \"foo\"))",
        "values.findLastIndex((x => x === (0, \"foo\")))",
        "function fn() {\n\tfoo.findLastIndex(x => x === arguments.length)\n}",
        "function fn() {\n\tfoo.findLastIndex(x => x === this[1])\n}",
        "values.findLastIndex(x => x === foo())",
        "foo.findLastIndex(function a(x) {\n\treturn x === (function (a) {\n\t\treturn a(this) === arguments[1]\n\t}).call(thisObject, anotherFunctionNamedA, secondArgument)\n})",
        "function foo() {\n\treturn (bar as string).findLastIndex(x => x === \"foo\");\n}",
    ];

    Tester::new(PreferArrayIndexOf::NAME, PreferArrayIndexOf::PLUGIN, pass, fail)
        .test_and_snapshot();
}
