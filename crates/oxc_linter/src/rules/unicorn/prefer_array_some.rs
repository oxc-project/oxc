use oxc_ast::{
    ast::{Argument, CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::{BinaryOperator, LogicalOperator, UnaryOperator};

use crate::{
    ast_util::{call_expr_method_callee_info, is_method_call, outermost_paren_parent},
    context::LintContext,
    rule::Rule,
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
enum PreferArraySomeDiagnostic {
    #[error("eslint-plugin-unicorn(prefer-array-some): Prefer `.some(…)` over `.find(…)`or `.findLast(…)`.")]
    #[diagnostic(severity(warning))]
    OverMethod(#[label] Span),
    #[error("eslint-plugin-unicorn(prefer-array-some): Prefer `.some(…)` over non-zero length check from `.filter(…)`.")]
    #[diagnostic(severity(warning))]
    NonZeroFilter(#[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct PreferArraySome;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers using [`Array#some`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/some) over [`Array#find()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/find), [`Array#findLast()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/findLast) and a non-zero length check on the result of [`Array#filter()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/filter)
    ///
    /// ### Why is this bad?
    ///
    /// Using `.some()` is more idiomatic and easier to read.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// const foo = array.find(fn) ? bar : baz;
    ///
    /// // Good
    /// const foo = array.some(fn) ? bar : baz;
    /// ```
    PreferArraySome,
    pedantic
);

impl Rule for PreferArraySome {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call_expr) => {
                if !is_method_call(call_expr, Some(&["find", "findLast"]), Some(1), Some(2)) {
                    return;
                }

                let is_compare = is_checking_undefined(node, call_expr, ctx);
                if !is_compare && !is_boolean_node(node, ctx) {
                    return;
                }

                ctx.diagnostic(PreferArraySomeDiagnostic::OverMethod(
                    // SAFETY: `call_expr_method_callee_info` returns `Some` if `is_method_call` returns `true`.
                    call_expr_method_callee_info(call_expr).unwrap().0,
                ));
            }
            AstKind::BinaryExpression(bin_expr) => {
                if !matches!(
                    bin_expr.operator,
                    BinaryOperator::GreaterThan | BinaryOperator::StrictInequality
                ) {
                    return;
                }

                let Expression::NumberLiteral(right_num_lit) = &bin_expr.right else { return };

                if right_num_lit.raw != "0" {
                    return;
                }

                let Expression::MemberExpression(left_member_expr) =
                    &bin_expr.left.without_parenthesized()
                else {
                    return;
                };

                let Some(static_property_name) = left_member_expr.static_property_name() else {
                    return;
                };

                if !matches!(static_property_name, "length") {
                    return;
                }

                let Expression::CallExpression(left_call_expr) =
                    &left_member_expr.object().without_parenthesized()
                else {
                    return;
                };

                if !is_method_call(left_call_expr, Some(&["filter"]), None, None) {
                    return;
                }

                let Some(Argument::Expression(first_filter_call_arg)) =
                    left_call_expr.arguments.first()
                else {
                    return;
                };

                if is_node_value_not_function(first_filter_call_arg) {
                    return;
                }

                ctx.diagnostic(PreferArraySomeDiagnostic::NonZeroFilter(
                    // SAFETY: `call_expr_method_callee_info` returns `Some` if `is_method_call` returns `true`.
                    call_expr_method_callee_info(left_call_expr).unwrap().0,
                ));
            }
            _ => {}
        }
    }
}

fn is_node_value_not_function(expr: &Expression) -> bool {
    if matches!(
        expr,
        Expression::ArrayExpression(_)
            | Expression::BinaryExpression(_)
            | Expression::ClassExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::TemplateLiteral(_)
            | Expression::UnaryExpression(_)
            | Expression::UpdateExpression(_)
    ) {
        return true;
    }
    if expr.is_literal() {
        return true;
    }
    if matches!(
        expr,
        Expression::AssignmentExpression(_)
            | Expression::AwaitExpression(_)
            | Expression::LogicalExpression(_)
            | Expression::NewExpression(_)
            | Expression::TaggedTemplateExpression(_)
            | Expression::ThisExpression(_)
    ) {
        return true;
    }
    if expr.is_undefined() {
        return true;
    }

    false
}

fn is_checking_undefined<'a, 'b>(
    node: &'b AstNode<'a>,
    _call_expr: &'b CallExpression<'a>,
    ctx: &'b LintContext<'a>,
) -> bool {
    let Some(parent) = outermost_paren_parent(node, ctx) else { return false };

    let AstKind::BinaryExpression(bin_expr) = parent.kind() else { return false };

    let right_without_paren = bin_expr.right.without_parenthesized();

    if matches!(
        bin_expr.operator,
        BinaryOperator::Inequality
            | BinaryOperator::Equality
            | BinaryOperator::StrictInequality
            | BinaryOperator::StrictEquality
    ) && right_without_paren.without_parenthesized().is_undefined()
    {
        return true;
    }

    if matches!(bin_expr.operator, BinaryOperator::Inequality | BinaryOperator::Equality)
        && right_without_paren.is_null()
    {
        return true;
    }

    false
}

fn is_logic_not(node: &AstKind) -> bool {
    let AstKind::UnaryExpression(logic_expr) = node else { return false };
    logic_expr.operator == UnaryOperator::UnaryNegation
}

fn is_boolean_call_argument(node: &AstKind) -> bool {
    let AstKind::CallExpression(call_expr) = node else { return false };
    let Expression::Identifier(ident) = &call_expr.callee else { return false };
    ident.name == "Boolean" && call_expr.arguments.len() == 1
}

fn is_logical_expression(node: &AstNode) -> bool {
    let AstKind::LogicalExpression(logical_expr) = node.kind() else { return false };

    matches!(logical_expr.operator, LogicalOperator::And | LogicalOperator::Or)
}

fn is_boolean_node<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> bool {
    let kind = node.kind();

    if is_logic_not(&kind) || is_boolean_call_argument(&kind) {
        return true;
    }

    let Some(parent) = outermost_paren_parent(node, ctx) else { return false };

    if matches!(
        parent.kind(),
        AstKind::IfStatement(_)
            | AstKind::ConditionalExpression(_)
            | AstKind::WhileStatement(_)
            | AstKind::DoWhileStatement(_)
            | AstKind::ForStatement(_)
    ) {
        return true;
    }

    if is_logical_expression(parent) {
        return is_boolean_node(parent, ctx);
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const bar = foo.find(fn)",
        r"const bar = foo.find(fn) || baz",
        r"if (foo.find(fn) ?? bar) {}",
        r"array.filter(fn).length > 0.",
        r"array.filter(fn).length > .0",
        r"array.filter(fn).length > 0.0",
        r"array.filter(fn).length > 0x00",
        r"array.filter(fn).length < 0",
        r"array.filter(fn).length >= 0",
        r"0 > array.filter(fn).length",
        r"array.filter(fn).length !== 0.",
        r"array.filter(fn).length !== .0",
        r"array.filter(fn).length !== 0.0",
        r"array.filter(fn).length !== 0x00",
        r"array.filter(fn).length != 0",
        r"array.filter(fn).length === 0",
        r"array.filter(fn).length == 0",
        r"array.filter(fn).length = 0",
        r"0 !== array.filter(fn).length",
        r"array.filter(fn).length >= 1",
        r"array.filter(fn).length >= 1.",
        r"array.filter(fn).length >= 1.0",
        r"array.filter(fn).length >= 0x1",
        r"array.filter(fn).length > 1",
        r"array.filter(fn).length < 1",
        r"array.filter(fn).length = 1",
        r"array.filter(fn).length += 1",
        r"1 >= array.filter(fn).length",
        r"array.filter(fn)?.length > 0",
        r"array.filter(fn)[length] > 0",
        r"array.filter(fn).notLength > 0",
        r"array.filter(fn).length() > 0",
        r"+array.filter(fn).length >= 1",
        r"array.filter?.(fn).length > 0",
        r"array?.filter(fn).length > 0",
        r"array.notFilter(fn).length > 0",
        r"array.filter.length > 0",
        r#"$element.filter(":visible").length > 0"#,
        r"foo.find(fn) == 0",
        r#"foo.find(fn) != """#,
        r"foo.find(fn) === null",
        r#"foo.find(fn) !== "null""#,
        r"foo.find(fn) >= undefined",
        r"foo.find(fn) instanceof undefined",
        r#"typeof foo.find(fn) === "undefined""#,
    ];

    let fail = vec![
        r"if (foo.find(fn)) {}",
        r"if (foo.findLast(fn)) {}",
        r#"if (array.find(element => element === "🦄")) {}"#,
        r#"const foo = array.find(element => element === "🦄") ? bar : baz;"#,
        r"array.filter(fn).length > 0",
        r"array.filter(fn).length !== 0",
        r"foo.find(fn) == null",
        r"foo.find(fn) == undefined",
        r"foo.find(fn) === undefined",
        r"foo.find(fn) != null",
        r"foo.find(fn) != undefined",
        r"foo.find(fn) !== undefined",
        r#"a = (( ((foo.find(fn))) == ((null)) )) ? "no" : "yes";"#,
    ];

    Tester::new_without_config(PreferArraySome::NAME, pass, fail).test_and_snapshot();
}
