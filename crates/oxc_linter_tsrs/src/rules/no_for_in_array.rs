// Port of internal/rules/no_for_in_array/no_for_in_array.go.

use tsrs_ast::{Kind, Node};
use tsrs_checker::{Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};
use crate::utils;

fn build_for_in_violation_message() -> RuleMessage {
    RuleMessage::with_help(
        "forInViolation",
        "For-in loops over arrays skips holes, returns indices as strings, and may visit the prototype chain or other enumerable properties.",
        "Use a more robust iteration method such as for-of or array.forEach instead.",
    )
}

pub struct NoForInArray;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoForInArray))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::ForInStatement)];

impl Rule for NoForInArray {
    fn name(&self) -> &'static str {
        "no-for-in-array"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn has_arrayish_length(ctx: &mut Ctx, t: P<Type>) -> bool {
    let Some(length_property) = ctx.checker.get_property_of_type(t, "length") else {
        return false;
    };
    let length_type = ctx.checker.get_type_of_symbol(length_property);
    utils::is_type_flag_set(length_type, TypeFlags::NumberLike)
}

fn is_array_like(ctx: &mut Ctx, t: P<Type>) -> bool {
    utils::type_recurser(t, &mut |t| {
        utils::get_number_index_type(ctx.checker, t).is_some() && has_arrayish_length(ctx, t)
    })
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let t = utils::get_constrained_type_at_location(ctx.checker, node.expression().unwrap());
        if is_array_like(ctx, t) {
            let (pos, end) = utils::get_for_statement_head_loc(ctx.file, node);
            ctx.report_range(pos, end, build_for_in_violation_message());
        }
    }
}
