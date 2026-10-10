// Port of internal/rules/no_array_delete/no_array_delete.go.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{Checker, Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{
    Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleMessage, RuleSuggestion, RuleVisitor,
};
use crate::utils;

fn build_no_array_delete_message() -> RuleMessage {
    RuleMessage::new(
        "noArrayDelete",
        "Using the `delete` operator with an array expression is unsafe.",
    )
}

fn build_use_splice_message() -> RuleMessage {
    RuleMessage::new("useSplice", "Use `array.splice()` instead.")
}

pub struct NoArrayDelete;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoArrayDelete))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::DeleteExpression)];

impl Rule for NoArrayDelete {
    fn name(&self) -> &'static str {
        "no-array-delete"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn is_underlying_type_array(c: &mut Checker, t: P<Type>) -> bool {
    if utils::is_type_flag_set(t, TypeFlags::Union) {
        return t.types().iter().all(|&t| c.is_array_or_tuple_type(t));
    }
    if utils::is_type_flag_set(t, TypeFlags::Intersection) {
        return t.types().iter().any(|&t| c.is_array_or_tuple_type(t));
    }
    c.is_array_or_tuple_type(t)
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let delete_expression = ast::skip_parentheses(node.as_delete_expression().expression);
        if !ast::is_element_access_expression(delete_expression) {
            return;
        }
        let expression = delete_expression.as_element_access_expression();

        let arg_type = utils::get_constrained_type_at_location(ctx.checker, expression.expression);
        if !is_underlying_type_array(ctx.checker, arg_type) {
            return;
        }

        let delete_token_range = tsrs_scanner::get_range_of_token_at_position(ctx.file, node.pos());
        let array_expr_range = ctx.trim(expression.expression);

        ctx.report_diagnostic_with_suggestions(
            RuleDiagnostic {
                pos: delete_token_range.pos(),
                end: delete_token_range.end(),
                message: build_no_array_delete_message(),
                labeled_ranges: vec![LabeledRange {
                    label: "This expression evaluates to an array.".to_string(),
                    pos: array_expr_range.0,
                    end: array_expr_range.1,
                }],
            },
            |ctx| {
                let argument_range = ctx.trim(expression.argument_expression);
                let left_bracket_token_range =
                    tsrs_scanner::get_range_of_token_at_position(ctx.file, array_expr_range.1);
                let right_bracket_token_range =
                    tsrs_scanner::get_range_of_token_at_position(ctx.file, argument_range.1);
                vec![RuleSuggestion {
                    message: build_use_splice_message(),
                    fixes: vec![
                        ctx.fix_remove_range(delete_token_range.pos(), delete_token_range.end()),
                        ctx.fix_replace_range(
                            left_bracket_token_range.pos(),
                            left_bracket_token_range.end(),
                            ".splice(",
                        ),
                        ctx.fix_replace_range(
                            right_bracket_token_range.pos(),
                            right_bracket_token_range.end(),
                            ", 1)",
                        ),
                    ],
                }]
            },
        );
    }
}
