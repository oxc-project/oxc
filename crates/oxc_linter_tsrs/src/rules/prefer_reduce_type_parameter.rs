// Port of internal/rules/prefer_reduce_type_parameter/prefer_reduce_type_parameter.go.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};
use crate::utils;

fn build_prefer_type_parameter_message() -> RuleMessage {
    RuleMessage::new(
        "preferTypeParameter",
        "Unnecessary assertion: Array#reduce accepts a type parameter for the default value.",
    )
}

pub struct PreferReduceTypeParameter;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(PreferReduceTypeParameter))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::CallExpression)];

impl Rule for PreferReduceTypeParameter {
    fn name(&self) -> &'static str {
        "prefer-reduce-type-parameter"
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
        let args = node.arguments();
        if args.len() < 2 {
            return;
        }
        let second_arg = args[1];
        if second_arg.kind() != Kind::AsExpression
            && second_arg.kind() != Kind::TypeAssertionExpression
        {
            return;
        }
        let callee = node.expression().unwrap();
        if !ast::is_access_expression(callee) {
            return;
        }
        let (property_name, found) = ctx.checker.get_accessed_property_name(callee);
        if !found || property_name != "reduce" {
            return;
        }
        let assertion_expr = second_arg.expression().unwrap();
        let assertion_type = second_arg.type_node().unwrap();
        let initializer_type = ctx.checker.get_type_at_location(assertion_expr);
        let asserted_type = ctx.checker.get_type_at_location(assertion_type);
        // don't report this if the resulting fix will be a type error
        if !ctx.checker.is_type_assignable_to(initializer_type, asserted_type) {
            return;
        }
        let callee_obj_type =
            utils::get_constrained_type_at_location(ctx.checker, callee.expression().unwrap());
        if utils::type_recurser(callee_obj_type, &mut |t| !ctx.checker.is_array_or_tuple_type(t)) {
            return;
        }
        ctx.report_node_with_fixes(second_arg, build_prefer_type_parameter_message(), |ctx| {
            let mut fixes = Vec::with_capacity(2);
            if second_arg.kind() == Kind::AsExpression {
                fixes.push(ctx.fix_remove_range(assertion_expr.end(), assertion_type.end()));
            } else {
                fixes.push(ctx.fix_remove_range(second_arg.pos(), assertion_expr.pos()));
            }
            if node.type_argument_list().is_none() {
                let text =
                    &ctx.text()[assertion_type.pos() as usize..assertion_type.end() as usize];
                fixes.push(ctx.fix_insert_after(callee, format!("<{text}>")));
            }
            fixes
        });
    }
}
