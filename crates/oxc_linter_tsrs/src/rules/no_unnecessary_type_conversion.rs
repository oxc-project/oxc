// Port of internal/rules/no_unnecessary_type_conversion/no_unnecessary_type_conversion.go.

use tsrs_ast::{self as ast, Kind, Node, SymbolFlags};
use tsrs_checker::{LiteralValue, Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{
    Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleFix, RuleMessage, RuleSuggestion,
    RuleVisitor,
};
use crate::utils;

const BIG_INT_CONVERSION_BUILTINS: &[&str] = &["BigInt", "BigIntConstructor"];
const BOOLEAN_CONVERSION_BUILTINS: &[&str] = &["Boolean", "BooleanConstructor"];
const NUMBER_CONVERSION_BUILTINS: &[&str] = &["Number", "NumberConstructor"];
const STRING_CONVERSION_BUILTINS: &[&str] = &["String", "StringConstructor"];

fn build_unnecessary_type_conversion_diagnostic(
    conversion: (i32, i32),
    expression: (i32, i32),
    expression_type: &str,
) -> RuleDiagnostic {
    RuleDiagnostic {
        message: RuleMessage::new(
            "unnecessaryTypeConversion",
            "This type conversion does not change the type or value of the expression.",
        ),
        pos: conversion.0,
        end: conversion.1,
        labeled_ranges: vec![LabeledRange {
            pos: expression.0,
            end: expression.1,
            label: format!("This expression already has type '{expression_type}'."),
        }],
    }
}

fn build_suggest_remove_message() -> RuleMessage {
    RuleMessage::new("suggestRemove", "Remove the type conversion.")
}

fn build_suggest_satisfies_message() -> RuleMessage {
    RuleMessage::new(
        "suggestSatisfies",
        "Instead, assert that the value satisfies the primitive type.",
    )
}

pub struct NoUnnecessaryTypeConversion;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoUnnecessaryTypeConversion))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::BinaryExpression),
    Listener::Enter(Kind::CallExpression),
    Listener::Enter(Kind::PrefixUnaryExpression),
];

impl Rule for NoUnnecessaryTypeConversion {
    fn name(&self) -> &'static str {
        "no-unnecessary-type-conversion"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn does_underlying_type_match_flag(t: P<Type>, type_flag: TypeFlags) -> bool {
    utils::union_type_parts(t).into_iter().all(|part| utils::is_type_flag_set(part, type_flag))
}

fn is_empty_string_literal(node: P<Node>) -> bool {
    ast::is_string_literal(node) && node.as_string_literal().text().is_empty()
}

fn is_enum_type(t: P<Type>) -> bool {
    utils::is_type_flag_set(t, TypeFlags::EnumLike)
}

fn is_enum_member_type(t: P<Type>) -> bool {
    t.symbol().is_some_and(|s| s.flags.get().intersects(SymbolFlags::EnumMember))
}

fn is_all_number_literal_integers(t: P<Type>) -> bool {
    let parts = utils::union_type_parts(t);
    if parts.is_empty() {
        return false;
    }
    for part in parts {
        if !utils::is_type_flag_set(part, TypeFlags::NumberLiteral) {
            return false;
        }
        // Go: strconv.ParseFloat(literal.String()) then math.Trunc(value) == value; the jsnum string
        // round-trips to the same float (including Infinity/NaN).
        let Some(LiteralValue::Number(n)) = part.as_literal_type().value() else {
            return false;
        };
        let value = n.0;
        if value.trunc() != value {
            return false;
        }
    }
    true
}

fn is_node_parenthesized(node: P<Node>) -> bool {
    let parent = node.parent().unwrap();
    ast::is_parenthesized_expression(parent)
        && parent.as_parenthesized_expression().expression.get() == node
}

fn is_object_expression_in_one_line_return(node: P<Node>, inner_node: P<Node>) -> bool {
    let parent = node.parent().unwrap();
    ast::is_arrow_function(parent)
        && parent.body() == Some(node)
        && ast::is_object_literal_expression(inner_node)
}

fn is_weak_precedence_parent(node: P<Node>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind() {
        Kind::PostfixUnaryExpression
        | Kind::PrefixUnaryExpression
        | Kind::BinaryExpression
        | Kind::ConditionalExpression
        | Kind::AwaitExpression => return true,
        _ => {}
    }
    if ast::is_property_access_expression(parent) {
        return parent.as_property_access_expression().expression == node;
    }
    if ast::is_element_access_expression(parent) {
        return parent.as_element_access_expression().expression == node;
    }
    if ast::is_call_expression(parent) || ast::is_new_expression(parent) {
        return parent.expression() == Some(node);
    }
    if ast::is_tagged_template_expression(parent) {
        return parent.as_tagged_template_expression().tag == node;
    }
    false
}

fn get_node_text(ctx: &Ctx, node: P<Node>) -> &'static str {
    let (pos, end) = ctx.trim(node);
    &ctx.text()[pos as usize..end as usize]
}

type WrapFn<'a> = &'a dyn Fn(&[String]) -> String;

fn build_wrapping_fix(
    ctx: &Ctx,
    node: P<Node>,
    inner_nodes: &[P<Node>],
    wrap: Option<WrapFn>,
) -> RuleFix {
    let inner_codes: Vec<String> = inner_nodes
        .iter()
        .map(|&inner_node| {
            let code = get_node_text(ctx, inner_node);
            if !utils::is_strong_precedence_node(inner_node)
                || is_object_expression_in_one_line_return(node, inner_node)
            {
                format!("({code})")
            } else {
                code.to_string()
            }
        })
        .collect();
    let Some(wrap) = wrap else {
        return ctx.fix_replace(node, inner_codes.join(""));
    };
    let mut code = wrap(&inner_codes);
    if is_weak_precedence_parent(node) && !is_node_parenthesized(node) {
        code = format!("({code})");
    }
    ctx.fix_replace(node, code)
}

fn build_suggestions(
    ctx: &Ctx,
    node: P<Node>,
    primitive_type: &str,
    inner_nodes: &[P<Node>],
) -> Vec<RuleSuggestion> {
    let satisfies = |code: &[String]| format!("{} satisfies {primitive_type}", code[0]);
    vec![
        RuleSuggestion {
            message: build_suggest_remove_message(),
            fixes: vec![build_wrapping_fix(ctx, node, inner_nodes, None)],
        },
        RuleSuggestion {
            message: build_suggest_satisfies_message(),
            fixes: vec![build_wrapping_fix(ctx, node, inner_nodes, Some(&satisfies))],
        },
    ]
}

fn report_binary_plus(ctx: &mut Ctx, node: P<Node>) {
    let expr = node.as_binary_expression();
    let left = expr.left;
    let right = expr.right.get();
    if is_empty_string_literal(right) {
        let left_type = utils::get_constrained_type_at_location(ctx.checker, left);
        if does_underlying_type_match_flag(left_type, TypeFlags::StringLike) {
            let type_string = utils::type_to_string(ctx.checker, left_type);
            let d = build_unnecessary_type_conversion_diagnostic(
                (left.end(), node.end()),
                ctx.trim(left),
                &type_string,
            );
            ctx.report_diagnostic_with_suggestions(d, |ctx| {
                build_suggestions(ctx, node, "string", &[left])
            });
        }
        return;
    }
    if is_empty_string_literal(left) {
        let right_type = utils::get_constrained_type_at_location(ctx.checker, right);
        if does_underlying_type_match_flag(right_type, TypeFlags::StringLike) {
            let right_start = ctx.trim(right).0;
            let type_string = utils::type_to_string(ctx.checker, right_type);
            let d = build_unnecessary_type_conversion_diagnostic(
                (ctx.trim(node).0, right_start),
                ctx.trim(right),
                &type_string,
            );
            ctx.report_diagnostic_with_suggestions(d, |ctx| {
                build_suggestions(ctx, node, "string", &[right])
            });
        }
    }
}

fn report_plus_equals(ctx: &mut Ctx, node: P<Node>) {
    let expr = node.as_binary_expression();
    let left = expr.left;
    if !is_empty_string_literal(expr.right.get()) {
        return;
    }
    if !ast::is_identifier(left) {
        return;
    }
    let left_type = utils::get_constrained_type_at_location(ctx.checker, left);
    if !does_underlying_type_match_flag(left_type, TypeFlags::StringLike) {
        return;
    }
    let type_string = utils::type_to_string(ctx.checker, left_type);
    let d =
        build_unnecessary_type_conversion_diagnostic(ctx.trim(node), ctx.trim(left), &type_string);
    ctx.report_diagnostic_with_suggestions(d, |ctx| {
        let parent = node.parent().unwrap();
        let remove_fix = if ast::is_expression_statement(parent) {
            ctx.fix_replace(parent, "")
        } else {
            build_wrapping_fix(ctx, node, &[left], None)
        };
        let satisfies = |code: &[String]| format!("{} satisfies string", code[0]);
        vec![
            RuleSuggestion { message: build_suggest_remove_message(), fixes: vec![remove_fix] },
            RuleSuggestion {
                message: build_suggest_satisfies_message(),
                fixes: vec![build_wrapping_fix(ctx, node, &[left], Some(&satisfies))],
            },
        ]
    });
}

fn report_built_in_conversion_call(ctx: &mut Ctx, node: P<Node>) {
    let callee = node.as_call_expression().expression;
    let args = node.arguments();
    if !ast::is_identifier(callee) || args.len() != 1 {
        return;
    }
    let arg = args[0];
    if ast::is_spread_element(arg) {
        return;
    }
    let callee_name = callee.as_identifier().text();
    let (type_flag, builtins) = match callee_name {
        "BigInt" => (TypeFlags::BigIntLike, BIG_INT_CONVERSION_BUILTINS),
        "Boolean" => (TypeFlags::BooleanLike, BOOLEAN_CONVERSION_BUILTINS),
        "Number" => (TypeFlags::NumberLike, NUMBER_CONVERSION_BUILTINS),
        "String" => (TypeFlags::StringLike, STRING_CONVERSION_BUILTINS),
        _ => return,
    };
    let callee_type = ctx.checker.get_type_at_location(callee);
    if !utils::is_builtin_symbol_like(ctx.program, ctx.checker, callee_type, builtins) {
        return;
    }
    let arg_type = utils::get_constrained_type_at_location(ctx.checker, arg);
    if !does_underlying_type_match_flag(arg_type, type_flag) {
        return;
    }
    let primitive_type = callee_name.to_lowercase();
    let type_string = utils::type_to_string(ctx.checker, arg_type);
    let d =
        build_unnecessary_type_conversion_diagnostic(ctx.trim(callee), ctx.trim(arg), &type_string);
    ctx.report_diagnostic_with_suggestions(d, |ctx| {
        build_suggestions(ctx, node, &primitive_type, &[arg])
    });
}

fn report_string_to_string_call(ctx: &mut Ctx, node: P<Node>) {
    let callee = node.as_call_expression().expression;
    if !ast::is_property_access_expression(callee) || !node.arguments().is_empty() {
        return;
    }
    let member_expr = callee.as_property_access_expression();
    let name = member_expr.name();
    if name.text() != "toString" {
        return;
    }
    let object = member_expr.expression;
    let object_type = utils::get_constrained_type_at_location(ctx.checker, object);
    if is_enum_type(object_type) || is_enum_member_type(object_type) {
        return;
    }
    if !does_underlying_type_match_flag(object_type, TypeFlags::StringLike) {
        return;
    }
    let type_string = utils::type_to_string(ctx.checker, object_type);
    let d = build_unnecessary_type_conversion_diagnostic(
        (name.pos(), node.end()),
        ctx.trim(object),
        &type_string,
    );
    ctx.report_diagnostic_with_suggestions(d, |ctx| {
        build_suggestions(ctx, node, "string", &[object])
    });
}

fn report_unary_plus(ctx: &mut Ctx, node: P<Node>) {
    let expr = node.as_prefix_unary_expression();
    if expr.operator != Kind::PlusToken {
        return;
    }
    let operand = expr.operand;
    let arg_type = utils::get_constrained_type_at_location(ctx.checker, operand);
    if !does_underlying_type_match_flag(arg_type, TypeFlags::NumberLike) {
        return;
    }
    let type_string = utils::type_to_string(ctx.checker, arg_type);
    let d = build_unnecessary_type_conversion_diagnostic(
        (ctx.trim(node).0, ctx.trim(operand).0),
        ctx.trim(operand),
        &type_string,
    );
    ctx.report_diagnostic_with_suggestions(d, |ctx| {
        build_suggestions(ctx, node, "number", &[operand])
    });
}

/// The enclosing `op op operand` prefix expression when node is the inner half of a doubled operator.
fn doubled_outer(node: P<Node>, op: Kind) -> Option<P<Node>> {
    let expr = node.as_prefix_unary_expression();
    let parent = node.parent()?;
    if expr.operator != op
        || !ast::is_prefix_unary_expression(parent)
        || parent.as_prefix_unary_expression().operator != op
        || parent.as_prefix_unary_expression().operand != node
    {
        return None;
    }
    Some(parent)
}

fn report_double_bang(ctx: &mut Ctx, node: P<Node>) {
    let Some(outer_node) = doubled_outer(node, Kind::ExclamationToken) else {
        return;
    };
    let operand = node.as_prefix_unary_expression().operand;
    let arg_type = utils::get_constrained_type_at_location(ctx.checker, operand);
    if !does_underlying_type_match_flag(arg_type, TypeFlags::BooleanLike) {
        return;
    }
    let outer_start = ctx.trim(outer_node).0;
    let inner_start = ctx.trim(node).0;
    let type_string = utils::type_to_string(ctx.checker, arg_type);
    let d = build_unnecessary_type_conversion_diagnostic(
        (outer_start, inner_start + 1),
        ctx.trim(operand),
        &type_string,
    );
    ctx.report_diagnostic_with_suggestions(d, |ctx| {
        build_suggestions(ctx, outer_node, "boolean", &[operand])
    });
}

fn report_double_tilde(ctx: &mut Ctx, node: P<Node>) {
    let Some(outer_node) = doubled_outer(node, Kind::TildeToken) else {
        return;
    };
    let operand = node.as_prefix_unary_expression().operand;
    let arg_type = utils::get_constrained_type_at_location(ctx.checker, operand);
    if !is_all_number_literal_integers(arg_type) {
        return;
    }
    let outer_start = ctx.trim(outer_node).0;
    let inner_start = ctx.trim(node).0;
    let type_string = utils::type_to_string(ctx.checker, arg_type);
    let d = build_unnecessary_type_conversion_diagnostic(
        (outer_start, inner_start + 1),
        ctx.trim(operand),
        &type_string,
    );
    ctx.report_diagnostic_with_suggestions(d, |ctx| {
        build_suggestions(ctx, outer_node, "number", &[operand])
    });
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::BinaryExpression => match node.as_binary_expression().operator_token.kind() {
                Kind::PlusToken => report_binary_plus(ctx, node),
                Kind::PlusEqualsToken => report_plus_equals(ctx, node),
                _ => {}
            },
            Kind::CallExpression => {
                report_built_in_conversion_call(ctx, node);
                report_string_to_string_call(ctx, node);
            }
            Kind::PrefixUnaryExpression => {
                report_unary_plus(ctx, node);
                report_double_bang(ctx, node);
                report_double_tilde(ctx, node);
            }
            _ => {}
        }
    }
}
