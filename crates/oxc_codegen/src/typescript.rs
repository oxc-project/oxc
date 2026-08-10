//! TypeScript-specific expression ambiguity helpers.
//!
//! TypeScript may reinterpret `<`, `>`, `>>`, and `>>>` tokens as the
//! delimiters of type arguments. These helpers identify unparenthesized angle
//! tokens at the edges of emitted expressions. They deliberately do not try to
//! predict whether the token after a closing angle will make TypeScript commit
//! to that interpretation: grouping a structurally matched pair is smaller and
//! less fragile than duplicating the parser and printer here.

use oxc_ast::ast::Expression;
use oxc_syntax::{
    operator::{BinaryOperator, LogicalOperator},
    precedence::{GetPrecedence, Precedence},
};

use crate::Context;

/// Determines which expressions in a comma-separated list need parentheses to
/// prevent TypeScript from interpreting them as type arguments.
pub struct TypeArgumentList {
    last_closer: Option<usize>,
    precedence: Precedence,
    ctx: Context,
}

impl TypeArgumentList {
    pub fn new<'a, T>(
        is_typescript: bool,
        items: &[T],
        close_expression: impl for<'b> Fn(&'b T) -> Option<&'b Expression<'a>>,
        precedence: Precedence,
        ctx: Context,
    ) -> Self {
        let last_closer = (is_typescript && items.len() > 1)
            .then(|| {
                items.iter().rposition(|item| {
                    close_expression(item).is_some_and(|expression| {
                        expression_starts_with_ts_type_argument_close(expression, precedence, ctx)
                    })
                })
            })
            .flatten();

        Self { last_closer, precedence, ctx }
    }

    pub fn precedence_for(&self, index: usize, expression: Option<&Expression<'_>>) -> Precedence {
        if self.last_closer.is_some_and(|closer| index < closer)
            && expression.is_some_and(|expression| {
                expression_ends_with_ts_type_argument_open(expression, self.precedence, self.ctx)
            })
        {
            Precedence::Shift
        } else {
            self.precedence
        }
    }
}

/// Returns whether the emitted suffix of `expression` exposes a `<` or `<<`
/// token that TypeScript can reinterpret as the start of type arguments.
///
/// For example, when printing `(a < b) > /x/`, this returns true for the left
/// expression `a < b`. The printer retains the parentheses instead of emitting
/// the ambiguous `a < b > /x/`.
///
/// The walk follows emitted suffixes and is iterative for deeply associated
/// binary trees. The worklist is only allocated for an unwrapped nested
/// sequence, where any flattened item can carry the opening token.
pub fn expression_ends_with_ts_type_argument_open(
    expression: &Expression<'_>,
    precedence: Precedence,
    ctx: Context,
) -> bool {
    let (mut expression, mut precedence, mut ctx) = (expression, precedence, ctx);
    let mut pending = Vec::new();

    loop {
        loop {
            expression = expression.without_parentheses();

            match expression {
                Expression::BinaryExpression(binary) => {
                    let operator_precedence = binary.operator.precedence();
                    if precedence >= operator_precedence
                        || (binary.operator == BinaryOperator::In && ctx.forbid_in())
                    {
                        break;
                    }

                    if matches!(
                        binary.operator,
                        BinaryOperator::LessThan | BinaryOperator::ShiftLeft
                    ) {
                        return true;
                    }

                    if matches!(
                        binary.operator,
                        BinaryOperator::BitwiseOR | BinaryOperator::BitwiseAnd
                    ) {
                        // The binary printer protects exposed opening operands
                        // of `|` and `&`, so none escape this subtree.
                        break;
                    }

                    precedence = if operator_precedence.is_left_associative() {
                        operator_precedence
                    } else {
                        binary.operator.lower_precedence()
                    };
                    expression = &binary.right;
                }

                Expression::LogicalExpression(logical) => {
                    let operator_precedence = logical.operator.precedence();
                    if precedence > operator_precedence {
                        break;
                    }

                    precedence = if logical.operator == LogicalOperator::Coalesce
                        && matches!(
                            logical.right.without_parentheses(),
                            Expression::LogicalExpression(right)
                                if matches!(
                                    right.operator,
                                    LogicalOperator::And | LogicalOperator::Or
                                )
                        ) {
                        Precedence::Prefix
                    } else {
                        operator_precedence
                    };
                    expression = &logical.right;
                }

                Expression::SequenceExpression(sequence) => {
                    if precedence < Precedence::Comma {
                        let expression_ctx = ctx.and_forbid_call(false);
                        pending.extend(
                            sequence
                                .expressions
                                .iter()
                                .map(|expression| (expression, Precedence::Lowest, expression_ctx)),
                        );
                    }
                    break;
                }

                Expression::AssignmentExpression(assignment) => {
                    if precedence >= Precedence::Assign {
                        break;
                    }
                    expression = &assignment.right;
                    precedence = Precedence::Comma;
                }

                Expression::ArrowFunctionExpression(arrow) => {
                    if arrow.pife || precedence >= Precedence::Assign {
                        break;
                    }
                    let Some(body) = arrow.body.as_expression() else { break };
                    expression = body;
                    precedence = Precedence::Comma;
                }

                Expression::YieldExpression(yield_expression) => {
                    if precedence >= Precedence::Assign {
                        break;
                    }
                    let Some(argument) = &yield_expression.argument else { break };
                    expression = argument;
                    precedence = Precedence::Yield;
                    ctx = Context::empty();
                }

                Expression::ConditionalExpression(conditional) => {
                    if precedence >= Precedence::Conditional {
                        break;
                    }
                    expression = &conditional.alternate;
                    precedence = Precedence::Yield;
                    ctx &= Context::FORBID_IN;
                }

                Expression::PrivateInExpression(private_in) => {
                    if precedence >= Precedence::Compare {
                        break;
                    }
                    expression = &private_in.right;
                    precedence = Precedence::Equals;
                    ctx = Context::FORBID_IN;
                }

                _ => break,
            }
        }

        let Some(next) = pending.pop() else { break };
        (expression, precedence, ctx) = next;
    }

    false
}

/// Returns whether the emitted type-compatible prefix of `expression`
/// exposes a `>`, `>>`, or `>>>` token.
///
/// For example, when printing `(a < b) < (c >> /x/)`, this returns true for the
/// right expression `c >> /x/`. The printer retains the parentheses instead of
/// emitting the ambiguous `a < b < c >> /x/`. It also finds exposed closers in
/// expressions such as `(c > d) | T` and `T & (c >>> d)`.
///
/// `|` and `&` are valid TypeScript type separators, so both operands are
/// searched. Other operators only expose their emitted left edge before they
/// form a grammar barrier. The traversal is iterative for deep binary trees.
pub fn expression_starts_with_ts_type_argument_close(
    expression: &Expression<'_>,
    precedence: Precedence,
    ctx: Context,
) -> bool {
    let (mut expression, mut precedence, mut ctx) = (expression, precedence, ctx);
    let mut pending = Vec::new();

    loop {
        loop {
            expression = expression.without_parentheses();

            match expression {
                Expression::BinaryExpression(binary) => {
                    let operator_precedence = binary.operator.precedence();
                    if precedence >= operator_precedence
                        || (binary.operator == BinaryOperator::In && ctx.forbid_in())
                    {
                        break;
                    }

                    if matches!(
                        binary.operator,
                        BinaryOperator::GreaterThan
                            | BinaryOperator::ShiftRight
                            | BinaryOperator::ShiftRightZeroFill
                    ) {
                        return true;
                    }

                    let left_precedence = if operator_precedence.is_right_associative() {
                        operator_precedence
                    } else {
                        binary.operator.lower_precedence()
                    };

                    if matches!(
                        binary.operator,
                        BinaryOperator::BitwiseOR | BinaryOperator::BitwiseAnd
                    ) {
                        let right_precedence = if operator_precedence.is_left_associative() {
                            operator_precedence
                        } else {
                            binary.operator.lower_precedence()
                        };
                        pending.push((&binary.right, right_precedence, ctx));
                    }

                    expression = &binary.left;
                    precedence = left_precedence;
                }

                Expression::LogicalExpression(logical) => {
                    let operator_precedence = logical.operator.precedence();
                    if precedence > operator_precedence {
                        break;
                    }

                    let left_precedence = logical.operator.lower_precedence();
                    if (logical.operator == LogicalOperator::Coalesce
                        && matches!(
                            logical.left.without_parentheses(),
                            Expression::LogicalExpression(left)
                                if matches!(
                                    left.operator,
                                    LogicalOperator::And | LogicalOperator::Or
                                )
                        ))
                        || matches!(
                            logical.left.without_parentheses(),
                            Expression::LogicalExpression(left)
                                if left_precedence >= left.operator.precedence()
                        )
                    {
                        break;
                    }

                    expression = &logical.left;
                    precedence = left_precedence;
                }

                Expression::SequenceExpression(sequence) => {
                    if precedence < Precedence::Comma {
                        let expression_ctx = ctx.and_forbid_call(false);
                        pending.extend(
                            sequence
                                .expressions
                                .iter()
                                .map(|expression| (expression, Precedence::Lowest, expression_ctx)),
                        );
                    }
                    break;
                }

                Expression::ConditionalExpression(conditional) => {
                    if precedence >= Precedence::Conditional {
                        break;
                    }
                    expression = &conditional.test;
                    precedence = if matches!(
                        conditional.test.without_parentheses(),
                        Expression::TSAsExpression(_) | Expression::TSSatisfiesExpression(_)
                    ) {
                        Precedence::Compare
                    } else {
                        Precedence::Conditional
                    };
                    ctx &= Context::FORBID_IN;
                }

                Expression::ArrowFunctionExpression(arrow) => {
                    if arrow.r#async || arrow.pife || precedence >= Precedence::Assign {
                        break;
                    }
                    let Some(body) = arrow.body.as_expression() else { break };
                    expression = body;
                    precedence = Precedence::Comma;
                }

                Expression::TSAsExpression(as_expression) => {
                    if precedence >= Precedence::Compare {
                        break;
                    }
                    expression = &as_expression.expression;
                    precedence = Precedence::Exponentiation;
                }

                Expression::TSSatisfiesExpression(satisfies_expression) => {
                    if precedence >= Precedence::Compare {
                        break;
                    }
                    expression = &satisfies_expression.expression;
                    precedence = Precedence::Exponentiation;
                }

                _ => break,
            }
        }

        let Some(next) = pending.pop() else { break };
        (expression, precedence, ctx) = next;
    }

    false
}
