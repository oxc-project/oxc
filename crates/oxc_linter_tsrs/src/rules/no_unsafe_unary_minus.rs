// Port of internal/rules/no_unsafe_unary_minus/no_unsafe_unary_minus.go.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::TypeFlags;
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};
use crate::utils;

fn build_unary_minus_message(t: &str) -> RuleMessage {
    RuleMessage::new(
        "unaryMinus",
        format!(
            "Argument of unary negation should be assignable to number | bigint but is {t} instead."
        ),
    )
}

pub struct NoUnsafeUnaryMinus;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoUnsafeUnaryMinus))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::PrefixUnaryExpression)];

impl Rule for NoUnsafeUnaryMinus {
    fn name(&self) -> &'static str {
        "no-unsafe-unary-minus"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let expr = node.as_prefix_unary_expression();
        if expr.operator != Kind::MinusToken {
            return;
        }

        let operand = ast::skip_parentheses(expr.operand);
        if ast::is_numeric_literal(operand) || ast::is_big_int_literal(operand) {
            return;
        }

        let arg_type = utils::get_constrained_type_at_location(ctx.checker, expr.operand);

        for t in utils::union_type_parts(arg_type) {
            if !utils::is_type_flag_set(
                t,
                TypeFlags::Any | TypeFlags::Never | TypeFlags::BigIntLike | TypeFlags::NumberLike,
            ) {
                let type_string = utils::type_to_string(ctx.checker, t);
                ctx.report_node(node, build_unary_minus_message(&type_string));
                break;
            }
        }
    }
}
