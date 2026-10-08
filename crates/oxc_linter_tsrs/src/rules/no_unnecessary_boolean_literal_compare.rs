// Port of internal/rules/no_unnecessary_boolean_literal_compare.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::TypeFlags;
use tsrs_core::P;

use crate::rule::{
    Ctx, Listener, Rule, RuleFix, RuleMessage, RuleVisitor, opt_bool, options_object,
};
use crate::utils;

fn comparing_nullable_to_false() -> RuleMessage {
    RuleMessage::new(
        "comparingNullableToFalse",
        "This expression unnecessarily compares a nullable boolean value to false instead of using the ?? operator to provide a default.",
    )
}
fn comparing_nullable_to_true_direct() -> RuleMessage {
    RuleMessage::new(
        "comparingNullableToTrueDirect",
        "This expression unnecessarily compares a nullable boolean value to true instead of using it directly.",
    )
}
fn comparing_nullable_to_true_negated() -> RuleMessage {
    RuleMessage::new(
        "comparingNullableToTrueNegated",
        "This expression unnecessarily compares a nullable boolean value to true instead of negating it.",
    )
}
fn direct() -> RuleMessage {
    RuleMessage::new(
        "direct",
        "This expression unnecessarily compares a boolean value to a boolean instead of using it directly.",
    )
}
fn negated() -> RuleMessage {
    RuleMessage::new(
        "negated",
        "This expression unnecessarily compares a boolean value to a boolean instead of negating it.",
    )
}
fn no_strict_null_check() -> RuleMessage {
    RuleMessage::new(
        "noStrictNullCheck",
        "This rule requires the `strictNullChecks` compiler option to be turned on to function correctly.",
    )
}

pub struct NoUnnecessaryBooleanLiteralCompare {
    allow_comparing_nullable_booleans_to_false: bool,
    allow_comparing_nullable_booleans_to_true: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    Ok(Box::new(NoUnnecessaryBooleanLiteralCompare {
        allow_comparing_nullable_booleans_to_false: opt_bool(
            &m,
            "allowComparingNullableBooleansToFalse",
            true,
        ),
        allow_comparing_nullable_booleans_to_true: opt_bool(
            &m,
            "allowComparingNullableBooleansToTrue",
            true,
        ),
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::BinaryExpression)];

impl Rule for NoUnnecessaryBooleanLiteralCompare {
    fn name(&self) -> &'static str {
        "no-unnecessary-boolean-literal-compare"
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let options = ctx.program.options();
        if !utils::is_strict_compiler_option_enabled(&options, options.strict_null_checks) {
            ctx.report_range(0, 0, no_strict_null_check());
        }
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static NoUnnecessaryBooleanLiteralCompare,
}

struct BooleanComparison {
    expression: P<Node>,
    literal_boolean_in_comparison: bool,
    negated: bool,
    expression_is_nullable_boolean: bool,
}

fn is_nullable_boolean(t: P<tsrs_checker::Type>) -> bool {
    if !utils::is_union_type(t) {
        return false;
    }
    let mut flags = TypeFlags::empty();
    for &p in t.types() {
        flags |= p.flags();
    }
    flags.intersects(TypeFlags::Nullable) && flags.intersects(TypeFlags::BooleanLike)
}

fn is_conditional_test(mut node: P<Node>) -> bool {
    while let Some(parent) = node.parent() {
        match parent.kind() {
            Kind::ParenthesizedExpression => {}
            Kind::BinaryExpression => {
                let op = parent.as_binary_expression().operator_token.kind();
                if op != Kind::AmpersandAmpersandToken && op != Kind::BarBarToken {
                    return false;
                }
            }
            Kind::IfStatement => return parent.as_if_statement().expression == node,
            Kind::WhileStatement => return parent.as_while_statement().expression == node,
            Kind::DoStatement => return parent.as_do_statement().expression == node,
            Kind::ForStatement => return parent.as_for_statement().condition == Some(node),
            Kind::ConditionalExpression => {
                return parent.as_conditional_expression().condition == node;
            }
            Kind::PrefixUnaryExpression => {
                return parent.as_prefix_unary_expression().operator == Kind::ExclamationToken;
            }
            _ => return false,
        }
        node = parent;
    }
    false
}

fn get_boolean_comparison(ctx: &mut Ctx, node: P<Node>) -> Option<BooleanComparison> {
    let b = node.as_binary_expression();
    let op = b.operator_token.kind();
    let negated = match op {
        Kind::ExclamationEqualsToken | Kind::ExclamationEqualsEqualsToken => true,
        Kind::EqualsEqualsToken | Kind::EqualsEqualsEqualsToken => false,
        _ => return None,
    };
    let deconstruct = |against: P<Node>, expression: P<Node>| -> Option<(bool, P<Node>)> {
        match against.kind() {
            Kind::TrueKeyword => Some((true, expression)),
            Kind::FalseKeyword => Some((false, expression)),
            _ => None,
        }
    };
    let (literal_true, expression) = deconstruct(ast::skip_parentheses(b.right.get()), b.left)
        .or_else(|| deconstruct(ast::skip_parentheses(b.left), b.right.get()))?;
    let t = ctx.checker.get_type_at_location(expression);
    let (constraint, is_type_parameter) = utils::get_constraint_info(ctx.checker, t);
    let constraint = match constraint {
        Some(c) => c,
        None if is_type_parameter => return None,
        None => return None,
    };
    let mut res = BooleanComparison {
        expression,
        literal_boolean_in_comparison: literal_true,
        negated,
        expression_is_nullable_boolean: false,
    };
    if utils::is_type_flag_set(constraint, TypeFlags::BooleanLike) {
        return Some(res);
    }
    if is_nullable_boolean(constraint) {
        res.expression_is_nullable_boolean = true;
        return Some(res);
    }
    None
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let Some(cmp) = get_boolean_comparison(ctx, node) else {
            return;
        };
        let opts = self.rule;
        if cmp.expression_is_nullable_boolean
            && ((cmp.literal_boolean_in_comparison
                && opts.allow_comparing_nullable_booleans_to_true)
                || (!cmp.literal_boolean_in_comparison
                    && opts.allow_comparing_nullable_booleans_to_false))
        {
            return;
        }
        let msg = if cmp.expression_is_nullable_boolean {
            if cmp.literal_boolean_in_comparison {
                if cmp.negated {
                    comparing_nullable_to_true_negated()
                } else {
                    comparing_nullable_to_true_direct()
                }
            } else {
                comparing_nullable_to_false()
            }
        } else if cmp.negated {
            negated()
        } else {
            direct()
        };
        ctx.report_node_with_fixes(node, msg, |ctx| {
            let mut parent = node.parent().unwrap();
            while ast::is_parenthesized_expression(parent) {
                parent = parent.parent().unwrap();
            }
            let is_unary_negation = ast::is_prefix_unary_expression(parent)
                && parent.as_prefix_unary_expression().operator == Kind::ExclamationToken;
            let should_negate = cmp.negated != cmp.literal_boolean_in_comparison;
            let mutated = if is_unary_negation { parent } else { node };
            let text = ctx.text();
            let expr_text = &text[cmp.expression.pos() as usize..cmp.expression.end() as usize];
            if cmp.expression_is_nullable_boolean
                && cmp.literal_boolean_in_comparison
                && should_negate != is_unary_negation
                && !is_conditional_test(mutated)
            {
                let mut t = expr_text.trim().to_string();
                if !utils::is_strong_precedence_node(cmp.expression) {
                    t = format!("({t})");
                }
                return vec![ctx.fix_replace(mutated, format!("(!!{t})"))];
            }
            let mut fixes: Vec<RuleFix> = Vec::with_capacity(6);
            fixes.push(ctx.fix_replace(mutated, expr_text));
            if should_negate == is_unary_negation {
                fixes.push(ctx.fix_insert_before(mutated, "!"));
                if !utils::is_strong_precedence_node(cmp.expression) {
                    fixes.push(ctx.fix_insert_before(mutated, "("));
                    fixes.push(ctx.fix_insert_after(mutated, ")"));
                }
            }
            if cmp.expression_is_nullable_boolean && !cmp.literal_boolean_in_comparison {
                fixes.push(ctx.fix_insert_before(mutated, "("));
                fixes.push(ctx.fix_insert_after(mutated, " ?? true)"));
            }
            fixes
        });
    }
}
