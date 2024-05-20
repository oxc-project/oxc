use oxc_ast::{
    ast::{
        AssignmentExpression, AssignmentTarget, Expression, IdentifierReference,
        SimpleAssignmentTarget,
    },
    AstKind,
};
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};

use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, UnaryOperator, UpdateOperator};

use crate::{context::LintContext, rule::Rule, AstNode};

fn for_direction_diagnostic(span0: Span, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(for-direction): The update clause in this loop moves the variable in the wrong direction")
        .with_help("Use while loop for intended infinite loop")
        .with_labels([LabeledSpan::new_with_span(Some("This test moves in the wrong direction".into()), span0), LabeledSpan::new_with_span(Some("with this update".into()), span1)])
}

#[derive(Debug, Default, Clone)]
pub struct ForDirection;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow "for" loop update causing the counter to move in the wrong direction.
    ///
    /// ### Why is this bad?
    /// A for loop that is known to run infinitely or never run is considered a bug.
    ///
    /// ### Example
    /// ```javascript
    /// for (var i = 0; i < 10; i--) {}
    ///
    /// for (var = 10; i >= 0; i++) {}
    /// ```
    ForDirection,
    correctness
);

impl Rule for ForDirection {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ForStatement(for_loop) = node.kind() {
            if let Some(Expression::BinaryExpression(test)) = &for_loop.test {
                let (counter, counter_position) = match (&test.left, &test.right) {
                    (Expression::Identifier(counter), _) => (counter, LEFT),
                    (_, Expression::Identifier(counter)) => (counter, RIGHT),
                    _ => return,
                };
                let test_operator = &test.operator;
                let wrong_direction = match (test_operator, counter_position) {
                    (BinaryOperator::LessEqualThan | BinaryOperator::LessThan, RIGHT)
                    | (BinaryOperator::GreaterEqualThan | BinaryOperator::GreaterThan, LEFT) => {
                        FORWARD
                    }
                    (BinaryOperator::LessEqualThan | BinaryOperator::LessThan, LEFT)
                    | (BinaryOperator::GreaterEqualThan | BinaryOperator::GreaterThan, RIGHT) => {
                        BACKWARD
                    }
                    _ => return,
                };
                if let Some(update) = &for_loop.update {
                    let update_direction = get_update_direction(update, counter);
                    if update_direction == wrong_direction {
                        let update_span = get_update_span(update);
                        ctx.diagnostic(for_direction_diagnostic(test.span, update_span));
                    }
                }
            }
        }
    }
}

type UpdateDirection = i32;
const FORWARD: UpdateDirection = 1;
const BACKWARD: UpdateDirection = -1;
const UNKNOWN: UpdateDirection = 0;

type CounterPosition<'a> = &'a str;
const LEFT: CounterPosition = "left";
const RIGHT: CounterPosition = "right";

fn get_update_direction(update: &Expression, counter: &IdentifierReference) -> UpdateDirection {
    match update {
        // match increment or decrement
        Expression::UpdateExpression(update) => {
            if let SimpleAssignmentTarget::AssignmentTargetIdentifier(id) = &update.argument {
                if id.name != counter.name {
                    return UNKNOWN;
                }
                match update.operator {
                    UpdateOperator::Increment => FORWARD,
                    UpdateOperator::Decrement => BACKWARD,
                }
            } else {
                UNKNOWN
            }
        }
        // match add assign or subtract assign
        Expression::AssignmentExpression(assign) => {
            if let AssignmentTarget::AssignmentTargetIdentifier(id) = &assign.left {
                if id.name != counter.name {
                    return UNKNOWN;
                }
                get_assignment_direction(assign)
            } else {
                UNKNOWN
            }
        }
        // can't determine other kinds of updates
        _ => UNKNOWN,
    }
}

fn get_update_span(update: &Expression) -> Span {
    match update {
        Expression::UpdateExpression(update) => update.span,
        Expression::AssignmentExpression(assign) => assign.span,
        _ => unreachable!(),
    }
}

fn get_assignment_direction(assign: &AssignmentExpression) -> UpdateDirection {
    let operator = &assign.operator;
    let right = &assign.right;
    let positive = match right {
        Expression::NumericLiteral(r) => r.value.is_sign_positive(),
        Expression::UnaryExpression(right) => right.operator != UnaryOperator::UnaryNegation,
        _ => return UNKNOWN,
    };

    let mut direction = match operator {
        AssignmentOperator::Addition => FORWARD,
        AssignmentOperator::Subtraction => BACKWARD,
        _ => return UNKNOWN,
    };

    if !positive {
        direction = -direction;
    }
    direction
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

    Tester::new(ForDirection::NAME, pass, fail).test_and_snapshot();
}
