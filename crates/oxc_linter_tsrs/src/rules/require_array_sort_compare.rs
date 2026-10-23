// Port of internal/rules/require_array_sort_compare/require_array_sort_compare.go.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::TypeFlags;
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor, opt_bool, options_object};
use crate::utils;

fn build_require_compare_message() -> RuleMessage {
    RuleMessage::new("requireCompare", "Require 'compare' argument.")
}

pub struct RequireArraySortCompare {
    ignore_string_arrays: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    Ok(Box::new(RequireArraySortCompare {
        ignore_string_arrays: opt_bool(&m, "ignoreStringArrays", true),
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::CallExpression)];

impl Rule for RequireArraySortCompare {
    fn name(&self) -> &'static str {
        "require-array-sort-compare"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { o: self })
    }
}

struct Visitor {
    o: &'static RequireArraySortCompare,
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        if !node.arguments().is_empty() {
            return;
        }
        let callee = node.expression().unwrap();
        if !ast::is_access_expression(callee) {
            return;
        }
        let (property_name, found) = ctx.checker.get_accessed_property_name(callee);
        if !found || (property_name != "sort" && property_name != "toSorted") {
            return;
        }
        let callee_obj_type =
            utils::get_constrained_type_at_location(ctx.checker, callee.expression().unwrap());
        if self.o.ignore_string_arrays && ctx.checker.is_array_or_tuple_type(callee_obj_type) {
            let type_arguments = ctx.checker.get_type_arguments(callee_obj_type);
            if type_arguments.iter().all(|&t| {
                utils::is_type_flag_set(t, TypeFlags::String)
                    || utils::get_type_name(ctx.checker, t) == "string"
            }) {
                return;
            }
        }
        if utils::union_type_parts(callee_obj_type)
            .into_iter()
            .all(|t| ctx.checker.is_array_type(t))
        {
            ctx.report_node(node, build_require_compare_message());
        }
    }
}
