use oxc_ast::{
    AstKind,
    ast::{
        AssignmentExpression, AssignmentTarget, Expression, IdentifierReference,
        SimpleAssignmentTarget,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    operator::BinaryOperator::{GreaterEqualThan, GreaterThan, LessEqualThan, LessThan},
    operator::{AssignmentOperator, BinaryOperator, UnaryOperator, UpdateOperator},
};

use crate::fixer::{RuleFix, RuleFixer};
use crate::{AstNode, context::LintContext, rule::Rule};

fn for_direction_diagnostic(test_span: Span, update_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The update clause in this loop moves the variable in the wrong direction")
        .with_help("Use while loop for intended infinite loop")
        .with_labels([
            test_span.label("This test moves in the wrong direction"),
            update_span.label("with this update"),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct ForDirection;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `for` loops where the update clause moves the counter in the wrong
    /// direction, preventing the loop from reaching its stop condition.
    ///
    /// ### Why is this bad?
    ///
    /// A `for` loop with a stop condition that can never be reached will run
    /// infinitely. While infinite loops can be intentional, they are usually written
    /// as `while` loops. More often, an infinite `for` loop is a bug.
    ///
    /// ### Options
    ///
    /// No options available for this rule.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```js
    /// /* eslint for-direction: "error" */
    ///
    /// for (var i = 0; i < 10; i--) {
    /// }
    ///
    /// for (var i = 10; i >= 0; i++) {
    /// }
    ///
    /// for (var i = 0; i > 10; i++) {
    /// }
    ///
    /// for (var i = 0; 10 > i; i--) {
    /// }
    ///
    /// const n = -2;
    /// for (let i = 0; i < 10; i += n) {
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```js
    /// /* eslint for-direction: "error" */
    ///
    /// for (var i = 0; i < 10; i++) {
    /// }
    ///
    /// for (var i = 0; 10 > i; i++) { // with counter "i" on the right
    /// }
    ///
    /// for (let i = 10; i >= 0; i += this.step) { // direction unknown
    /// }
    ///
    /// for (let i = MIN; i <= MAX; i -= 0) { // not increasing or decreasing
    /// }
    /// ```
    ForDirection,
    eslint,
    correctness,
    fix_dangerous
);

#[derive(Debug, Eq, PartialEq)]
enum UpdateDirection {
    Forward,
    Backward,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum CounterPosition {
    Left,
    Right,
}

impl Rule for ForDirection {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ForStatement(for_loop) = node.kind() else {
            return;
        };

        let Some(Expression::BinaryExpression(test)) = &for_loop.test else {
            return;
        };

        let Some((counter, counter_position)) = extract_counter(&test.left, &test.right) else {
            return;
        };

        let Some(expected_update_direction) =
            get_expected_update_direction(test.operator, counter_position)
        else {
            return;
        };

        let Some(update_expr) = &for_loop.update else {
            return;
        };

        let Some(update_direction) = get_update_direction(update_expr, counter) else {
            return;
        };

        if update_direction != expected_update_direction {
            ctx.diagnostic_with_dangerous_fix(
                for_direction_diagnostic(test.span, get_update_span(update_expr)),
                |fixer| apply_rule_fix(&fixer, update_expr),
            );
        }
    }
}

fn extract_counter<'a>(
    left: &'a Expression<'a>,
    right: &'a Expression<'a>,
) -> Option<(&'a IdentifierReference<'a>, CounterPosition)> {
    match (left, right) {
        (Expression::Identifier(counter), _) => Some((counter, CounterPosition::Left)),
        (_, Expression::Identifier(counter)) => Some((counter, CounterPosition::Right)),
        _ => None,
    }
}

fn get_expected_update_direction(
    operator: BinaryOperator,
    counter_position: CounterPosition,
) -> Option<UpdateDirection> {
    match (operator, counter_position) {
        (LessEqualThan | LessThan, CounterPosition::Right)
        | (GreaterEqualThan | GreaterThan, CounterPosition::Left) => {
            Some(UpdateDirection::Backward)
        }
        (LessEqualThan | LessThan, CounterPosition::Left)
        | (GreaterEqualThan | GreaterThan, CounterPosition::Right) => {
            Some(UpdateDirection::Forward)
        }
        _ => None,
    }
}

fn get_fixer_replace_operator(update: &Expression) -> &'static str {
    match update {
        Expression::UpdateExpression(update) => match update.operator {
            UpdateOperator::Increment => "--",
            UpdateOperator::Decrement => "++",
        },
        Expression::AssignmentExpression(update) => match update.operator {
            AssignmentOperator::Addition => "-=",
            AssignmentOperator::Subtraction => "+=",
            _ => "",
        },
        _ => "",
    }
}

fn get_fixer_replace_span(update: &Expression) -> Span {
    match update {
        Expression::UpdateExpression(update) => {
            let arg_span = update.argument.span();
            let upd_span = update.span();

            if upd_span.start == arg_span.start {
                Span::new(arg_span.end, upd_span.end)
            } else {
                Span::new(upd_span.start, arg_span.start)
            }
        }
        Expression::AssignmentExpression(update) => {
            Span::new(update.left.span().end, update.right.span().start)
        }
        _ => Span::new(0, 0),
    }
}

fn apply_rule_fix<'a>(fixer: &RuleFixer<'_, 'a>, update: &Expression) -> RuleFix<'a> {
    let span = get_fixer_replace_span(update);
    let replacement = get_fixer_replace_operator(update);

    fixer.replace(span, replacement)
}

fn get_update_direction(
    update: &Expression,
    counter: &IdentifierReference,
) -> Option<UpdateDirection> {
    match update {
        // match increment or decrement
        Expression::UpdateExpression(update) => match &update.argument {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(id) if id.name == counter.name => {
                Some(match update.operator {
                    UpdateOperator::Increment => UpdateDirection::Forward,
                    UpdateOperator::Decrement => UpdateDirection::Backward,
                })
            }
            _ => None,
        },
        // match add assign or subtract assign
        Expression::AssignmentExpression(assign) => match &assign.left {
            AssignmentTarget::AssignmentTargetIdentifier(id) if id.name == counter.name => {
                get_assignment_direction(assign)
            }
            _ => None,
        },
        // can't determine other kinds of updates
        _ => None,
    }
}

fn get_update_span(update: &Expression) -> Span {
    match update {
        Expression::UpdateExpression(update) => update.span,
        Expression::AssignmentExpression(assign) => assign.span,
        _ => unreachable!(
            "get_update_span should only be called with UpdateExpression or AssignmentExpression"
        ),
    }
}

fn get_assignment_direction(assign: &AssignmentExpression) -> Option<UpdateDirection> {
    let operator = &assign.operator;
    let right = &assign.right;
    let is_positive = match right {
        Expression::NumericLiteral(r) if r.value != 0.0 => r.value.is_sign_positive(),
        Expression::UnaryExpression(right) => right.operator != UnaryOperator::UnaryNegation,
        _ => return None,
    };

    match operator {
        AssignmentOperator::Addition => {
            Some(if is_positive { UpdateDirection::Forward } else { UpdateDirection::Backward })
        }
        AssignmentOperator::Subtraction => {
            Some(if is_positive { UpdateDirection::Backward } else { UpdateDirection::Forward })
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // test if '++', '--'
        ("for(var i = 0; i < 10; i++){}", None),
        ("for(var i = 0; i <= 10; i++){}", None),
        ("for(var i = 10; i > 0; i--){}", None),
        ("for(var i = 10; i >= 0; i--){}", None),
        // test if '++', '--' with counter 'i' on the right side of test condition
        ("for(var i = 0; 10 > i; i++){}", None),
        ("for(var i = 0; 10 >= i; i++){}", None),
        ("for(var i = 10; 0 < i; i--){}", None),
        ("for(var i = 10; 0 <= i; i--){}", None),
        // test if '+=', '-=',
        ("for(var i = 0; i < 10; i+=1){}", None),
        ("for(var i = 0; i <= 10; i+=1){}", None),
        ("for(var i = 0; i < 10; i-=-1){}", None),
        ("for(var i = 0; i <= 10; i-=-1){}", None),
        ("for(var i = 10; i > 0; i-=1){}", None),
        ("for(var i = 10; i >= 0; i-=1){}", None),
        ("for(var i = 10; i > 0; i+=-1){}", None),
        ("for(var i = 10; i >= 0; i+=-1){}", None),
        // test if '+=', '-=' with counter 'i' on the right side of test condition
        ("for(var i = 0; 10 > i; i+=1){}", None),
        // test if no update.
        ("for(var i = 10; i > 0;){}", None),
        ("for(var i = 10; i >= 0;){}", None),
        ("for(var i = 10; i < 0;){}", None),
        ("for(var i = 10; i <= 0;){}", None),
        ("for(var i = 0; i < 10; i+=0){}", None),
        ("for(var i = 0; i < 10; i-=0){}", None),
        ("for(var i = 10; i > 0; i+=0){}", None),
        ("for(var i = 10; i > 0; i-=0){}", None),
        ("for(var i = 10; i <= 0; j++){}", None),
        ("for(var i = 10; i <= 0; j--){}", None),
        ("for(var i = 10; i >= 0; j++){}", None),
        ("for(var i = 10; i >= 0; j--){}", None),
        ("for(var i = 10; i >= 0; j += 2){}", None),
        ("for(var i = 10; i >= 0; j -= 2){}", None),
        ("for(var i = 10; i >= 0; i |= 2){}", None),
        ("for(var i = 10; i >= 0; i %= 2){}", None),
        ("for(var i = 0; i < MAX; i += STEP_SIZE);", None),
        ("for(var i = 0; i < MAX; i -= STEP_SIZE);", None),
        ("for(var i = 10; i > 0; i += STEP_SIZE);", None),
        // other cond-expressions.
        ("for(var i = 0; i !== 10; i+=1){}", None),
        ("for(var i = 0; i === 10; i+=1){}", None),
        ("for(var i = 0; i == 10; i+=1){}", None),
        ("for(var i = 0; i != 10; i+=1){}", None),
    ];

    let fail = vec![
        // test if '++', '--'
        ("for (var i = 0; i < 10; i--){}", None),
        ("for (var i = 0; i <= 10; i--){}", None),
        ("for(var i = 10; i > 10; i++){}", None),
        ("for(var i = 10; i >= 0; i++){}", None),
        // test if '++', '--' with counter 'i' on the right side of test condition
        ("for(var i = 0; 10 > i; i--){}", None),
        ("for(var i = 0; 10 >= i; i--){}", None),
        ("for(var i = 10; 10 < i; i++){}", None),
        ("for(var i = 10; 0 <= i; i++){}", None),
        // test if '+=', '-='
        ("for(var i = 0; i < 10; i-=1){}", None),
        ("for(var i = 0; i <= 10; i-=1){}", None),
        ("for(var i = 10; i > 10; i+=1){}", None),
        ("for(var i = 10; i >= 0; i+=1){}", None),
        ("for(var i = 0; i < 10; i+=-1){}", None),
        ("for(var i = 0; i <= 10; i+=-1){}", None),
        ("for(var i = 10; i > 10; i-=-1){}", None),
        ("for(var i = 10; i >= 0; i-=-1){}", None),
        // test if '+=', '-=' with counter 'i' on the right side of test condition
        ("for(var i = 0; 10 > i; i-=1){}", None),
    ];

    let fix = vec![
        ("for(var i = 0; i < 10; i--){}", "for(var i = 0; i < 10; i++){}", None),
        ("for(var i = 10; i > 0; i++){}", "for(var i = 10; i > 0; i--){}", None),
        ("for(var i = 0; i < 10; i-=1){}", "for(var i = 0; i < 10; i+=1){}", None),
        ("for(var i = 10; i > 0; i+=1){}", "for(var i = 10; i > 0; i-=1){}", None),
        ("for(var i = 0; i < 10; i+=-1){}", "for(var i = 0; i < 10; i-=-1){}", None),
        ("for(var i = 10; i > 0; i-=-1){}", "for(var i = 10; i > 0; i+=-1){}", None),
        ("for(var i = 0; i < 10; --i){}", "for(var i = 0; i < 10; ++i){}", None),
        ("for(var i = 0; i < 10; -- i){}", "for(var i = 0; i < 10; ++i){}", None),
        ("for(var i = 0; i < 10; i -= 1){}", "for(var i = 0; i < 10; i+=1){}", None),
        // variables of different lengths
        ("for(var ii = 0; ii < 10; ii--){}", "for(var ii = 0; ii < 10; ii++){}", None),
        ("for(var ii = 10; ii > 0; ii+=1){}", "for(var ii = 10; ii > 0; ii-=1){}", None),
    ];

    Tester::new(ForDirection::NAME, ForDirection::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
