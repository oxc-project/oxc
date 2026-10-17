// Port of internal/rules/non_nullable_type_assertion_style/non_nullable_type_assertion_style.go.

use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, Kind, Node, OperatorPrecedence};
use tsrs_checker::{Checker, Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleSuggestion, RuleVisitor};
use crate::utils;

fn build_prefer_non_null_assertion_message() -> RuleMessage {
    RuleMessage::new(
        "preferNonNullAssertion",
        "Use a ! assertion to more succinctly remove null and undefined from the type.",
    )
}

pub struct NonNullableTypeAssertionStyle;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NonNullableTypeAssertionStyle))
}

const LISTENERS: &[Listener] =
    &[Listener::Enter(Kind::AsExpression), Listener::Enter(Kind::TypeAssertionExpression)];

impl Rule for NonNullableTypeAssertionStyle {
    fn name(&self) -> &'static str {
        "non-nullable-type-assertion-style"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn get_types_if_not_loose(c: &mut Checker, node: P<Node>) -> Option<Vec<P<Type>>> {
    let t = c.get_type_at_location(node);
    if utils::is_type_flag_set(t, TypeFlags::Any | TypeFlags::Unknown) {
        return None;
    }
    Some(utils::union_type_parts(t))
}

fn could_be_nullable(c: &mut Checker, mut t: P<Type>) -> bool {
    if utils::is_type_parameter(t) {
        match c.get_base_constraint_of_type(t) {
            Some(constraint) => t = constraint,
            None => return true,
        }
    }
    utils::union_type_parts(t).into_iter().any(|p| utils::is_type_flag_set(p, TypeFlags::Nullable))
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _listener: Listener, node: P<Node>) {
        if ast::is_const_assertion(node) {
            return;
        }
        let expression = node.expression().unwrap();
        let Some(original_types) = get_types_if_not_loose(ctx.checker, expression) else {
            return;
        };
        let type_annotation = node.type_node().unwrap();
        let Some(asserted_types) = get_types_if_not_loose(ctx.checker, type_annotation) else {
            return;
        };

        let mut non_nullable_original_type: FxHashSet<P<Type>> = FxHashSet::default();
        for &t in &original_types {
            if !utils::is_type_flag_set(t, TypeFlags::Nullable) {
                non_nullable_original_type.insert(t);
            }
        }
        if non_nullable_original_type.len() == original_types.len() {
            return;
        }

        let mut asserted_types_set: FxHashSet<P<Type>> = FxHashSet::default();
        for &t in &asserted_types {
            if could_be_nullable(ctx.checker, t) || !non_nullable_original_type.contains(&t) {
                return;
            }
            asserted_types_set.insert(t);
        }
        if non_nullable_original_type.iter().any(|t| !asserted_types_set.contains(t)) {
            return;
        }

        let higher_precedence_than_unary =
            ast::get_expression_precedence(expression) > OperatorPrecedence::Unary;
        let (remove_pos, remove_end) = if ast::is_assertion_expression(node) {
            (expression.end(), node.end())
        } else {
            (node.pos(), expression.pos())
        };
        let mut fixes = vec![ctx.fix_remove_range(remove_pos, remove_end)];
        if higher_precedence_than_unary {
            fixes.push(ctx.fix_insert_after(expression, "!"));
        } else {
            fixes.push(ctx.fix_insert_before(expression, "("));
            fixes.push(ctx.fix_insert_after(expression, ")!"));
        }
        ctx.report_node_with_suggestions(node, build_prefer_non_null_assertion_message(), |_| {
            vec![RuleSuggestion { message: build_prefer_non_null_assertion_message(), fixes }]
        });
    }
}
