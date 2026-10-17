// Port of internal/rules/no_unsafe_type_assertion/no_unsafe_type_assertion.go.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{Checker, ObjectFlags, Type};
use tsrs_core::P;

use crate::rule::{Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleMessage, RuleVisitor};
use crate::utils;

fn build_unsafe_of_any_type_assertion_message(t: &str) -> RuleMessage {
    RuleMessage::with_help(
        "unsafeOfAnyTypeAssertion",
        format!("Unsafe assertion from {t} detected."),
        "Consider using type guards or a safer assertion.",
    )
}
fn build_unsafe_to_any_type_assertion_message(t: &str) -> RuleMessage {
    RuleMessage::with_help(
        "unsafeToAnyTypeAssertion",
        format!("Unsafe assertion to {t} detected."),
        "Consider using a more specific type to ensure safety.",
    )
}
fn build_unsafe_to_unconstrained_type_assertion_message(t: &str) -> RuleMessage {
    RuleMessage::new(
        "unsafeToUnconstrainedTypeAssertion",
        format!(
            "Unsafe type assertion: '{t}' could be instantiated with an arbitrary type which could be unrelated to the original type."
        ),
    )
}
fn build_unsafe_type_assertion_message(t: &str) -> RuleMessage {
    RuleMessage::new(
        "unsafeTypeAssertion",
        format!("Unsafe type assertion: type '{t}' is more narrow than the original type."),
    )
}
fn build_unsafe_type_assertion_assignable_to_constraint_message(t: &str) -> RuleMessage {
    RuleMessage::new(
        "unsafeTypeAssertionAssignableToConstraint",
        format!(
            "Unsafe type assertion: the original type is assignable to the constraint of type '{t}', but '{t}' could be instantiated with a different subtype of its constraint."
        ),
    )
}

fn get_any_type_name(t: P<Type>) -> &'static str {
    if utils::is_intrinsic_error_type(t) {
        return "error typed";
    }
    "`any`"
}

fn is_object_literal_type(t: P<Type>) -> bool {
    utils::is_object_type(t) && t.object_flags().intersects(ObjectFlags::ObjectLiteral)
}

fn get_assertion_range(
    ctx: &Ctx,
    node: P<Node>,
    expression: P<Node>,
    type_annotation: P<Node>,
) -> (i32, i32) {
    if ast::is_as_expression(node) {
        let as_keyword_range =
            tsrs_scanner::get_scanner_for_source_file(ctx.file, expression.end()).token_range();
        return (as_keyword_range.pos(), type_annotation.end());
    }
    let mut s = tsrs_scanner::get_scanner_for_source_file(ctx.file, node.pos());
    let opening_angle_bracket = s.token_range();
    s.reset_pos(type_annotation.end());
    s.scan();
    let closing_angle_bracket = s.token_range();
    (opening_angle_bracket.pos(), closing_angle_bracket.end())
}

fn type_label(c: &mut Checker, t: P<Type>) -> String {
    if utils::is_intrinsic_error_type(t) {
        return "error".to_string();
    }
    utils::type_to_string(c, t)
}

pub struct NoUnsafeTypeAssertion;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoUnsafeTypeAssertion))
}

const LISTENERS: &[Listener] =
    &[Listener::Enter(Kind::AsExpression), Listener::Enter(Kind::TypeAssertionExpression)];

impl Rule for NoUnsafeTypeAssertion {
    fn name(&self) -> &'static str {
        "no-unsafe-type-assertion"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn report(
    ctx: &mut Ctx,
    node: P<Node>,
    expression: P<Node>,
    type_annotation: P<Node>,
    expression_type: P<Type>,
    asserted_type: P<Type>,
    message: RuleMessage,
) {
    let (pos, end) = get_assertion_range(ctx, node, expression, type_annotation);
    let (ep, ee) = ctx.trim(expression);
    let (tp, te) = ctx.trim(type_annotation);
    let original_type = type_label(ctx.checker, expression_type);
    let asserted = type_label(ctx.checker, asserted_type);
    ctx.report_diagnostic(RuleDiagnostic {
        pos,
        end,
        message,
        labeled_ranges: vec![
            LabeledRange {
                label: format!("Original expression has type `{original_type}`."),
                pos: ep,
                end: ee,
            },
            LabeledRange { label: format!("Asserted type is `{asserted}`."), pos: tp, end: te },
        ],
    });
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _listener: Listener, node: P<Node>) {
        let expression = node.expression().unwrap();
        let type_annotation = node.type_node().unwrap();
        let expression_type = ctx.checker.get_type_at_location(expression);
        let asserted_type = ctx.checker.get_type_at_location(type_annotation);
        let report = |ctx: &mut Ctx, message: RuleMessage| {
            report(ctx, node, expression, type_annotation, expression_type, asserted_type, message)
        };

        if expression_type == asserted_type {
            return;
        }

        // handle cases when asserting unknown ==> any.
        if utils::is_type_any_type(asserted_type) && utils::is_type_unknown_type(expression_type) {
            report(ctx, build_unsafe_to_any_type_assertion_message("`any`"));
            return;
        }

        if let Some((_, sender)) = utils::is_unsafe_assignment(
            expression_type,
            asserted_type,
            ctx.checker,
            Some(expression),
        ) {
            report(ctx, build_unsafe_of_any_type_assertion_message(get_any_type_name(sender)));
            return;
        }

        if let Some((_, sender)) = utils::is_unsafe_assignment(
            asserted_type,
            expression_type,
            ctx.checker,
            Some(type_annotation),
        ) {
            report(ctx, build_unsafe_to_any_type_assertion_message(get_any_type_name(sender)));
            return;
        }

        // Use the widened type in case of an object literal so isTypeAssignableTo() won't fail on
        // excess property check.
        let mut expression_widened_type = expression_type;
        if is_object_literal_type(expression_type) {
            expression_widened_type = ctx.checker.get_widened_type(expression_type);
        }

        if ctx.checker.is_type_assignable_to(expression_widened_type, asserted_type) {
            return;
        }

        // Produce a more specific error message when targeting a type parameter
        if utils::is_type_parameter(asserted_type) {
            let Some(asserted_type_constraint) =
                ctx.checker.get_base_constraint_of_type(asserted_type)
            else {
                // asserting to an unconstrained type parameter is unsafe
                let s = utils::type_to_string(ctx.checker, asserted_type);
                report(ctx, build_unsafe_to_unconstrained_type_assertion_message(&s));
                return;
            };

            // special case message if the original type is assignable to the constraint of the
            // target type parameter
            if ctx.checker.is_type_assignable_to(expression_widened_type, asserted_type_constraint)
            {
                let s = utils::type_to_string(ctx.checker, asserted_type);
                report(ctx, build_unsafe_type_assertion_assignable_to_constraint_message(&s));
                return;
            }
        }

        let s = utils::type_to_string(ctx.checker, asserted_type);
        report(ctx, build_unsafe_type_assertion_message(&s));
    }
}
