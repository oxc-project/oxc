//! Visit binary and logical expression in a loop without recursion.
//!
//! Reference: <https://github.com/evanw/esbuild/blob/78f89e41d5e8a7088f4820351c6305cc339f8820/internal/js_printer/js_printer.go#L3266>

use std::ops::Not;

use oxc_ast::ast::{BinaryExpression, Expression, LogicalExpression};
use oxc_syntax::{
    node::NodeId,
    operator::{BinaryOperator, LogicalOperator},
    precedence::{GetPrecedence, Precedence},
};

use crate::{Codegen, Context, Operator, cjs_module_lexer, r#gen::GenExpr};

#[derive(Clone, Copy)]
pub enum Binaryish<'a> {
    Binary(&'a BinaryExpression<'a>),
    Logical(&'a LogicalExpression<'a>),
}

impl<'a> Binaryish<'a> {
    pub fn left(&self, p: &Codegen) -> &'a Expression<'a> {
        match self {
            Self::Binary(e) => {
                if p.attached_comments.is_empty() {
                    e.left.without_parentheses()
                } else {
                    &e.left
                }
            }
            Self::Logical(e) => {
                if p.attached_comments.is_empty() {
                    e.left.without_parentheses()
                } else {
                    &e.left
                }
            }
        }
    }

    pub fn right(&self, p: &Codegen) -> &'a Expression<'a> {
        match self {
            Self::Binary(e) => {
                if p.attached_comments.is_empty() {
                    e.right.without_parentheses()
                } else {
                    &e.right
                }
            }
            Self::Logical(e) => {
                if p.attached_comments.is_empty() {
                    e.right.without_parentheses()
                } else {
                    &e.right
                }
            }
        }
    }

    pub fn node_id(self) -> NodeId {
        match self {
            Self::Binary(node) => node.node_id(),
            Self::Logical(node) => node.node_id(),
        }
    }

    pub fn operator(&self) -> BinaryishOperator {
        match self {
            Self::Binary(e) => BinaryishOperator::Binary(e.operator),
            Self::Logical(e) => BinaryishOperator::Logical(e.operator),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BinaryishOperator {
    Binary(BinaryOperator),
    Logical(LogicalOperator),
}

impl BinaryishOperator {
    fn is_binary(self) -> bool {
        matches!(self, Self::Binary(_))
    }
}

impl Binaryish<'_> {
    #[inline]
    fn needs_parens(self, precedence: Precedence, parent: BinaryishOperator, ctx: Context) -> bool {
        let operator = self.operator();
        (precedence >= operator.precedence()
            && (parent.is_binary() || precedence != parent.precedence()))
            || (operator == BinaryishOperator::Binary(BinaryOperator::In)
                && ctx.contains(Context::FORBID_IN))
    }
}

fn print_binary_operator(op: BinaryOperator, p: &mut Codegen) {
    let operator = op.as_str();
    if op.is_keyword() {
        p.print_space_before_identifier();
        p.print_str(operator);
    } else {
        let op: Operator = op.into();
        p.print_space_before_operator(op);
        p.print_str(operator);
        p.prev_op = Some(op);
        p.prev_op_end = p.code().len();
    }
}

impl BinaryishOperator {
    fn r#gen(self, p: &mut Codegen) {
        match self {
            Self::Binary(op) => print_binary_operator(op, p),
            Self::Logical(op) => p.print_str(op.as_str()),
        }
    }
}

impl GetPrecedence for BinaryishOperator {
    fn precedence(&self) -> Precedence {
        match self {
            Self::Binary(op) => op.precedence(),
            Self::Logical(op) => op.precedence(),
        }
    }
}

impl BinaryishOperator {
    pub fn lower_precedence(self) -> Precedence {
        match self {
            Self::Binary(op) => op.lower_precedence(),
            Self::Logical(op) => op.lower_precedence(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct BinaryExpressionVisitor<'a> {
    pub e: Binaryish<'a>,
    pub precedence: Precedence,
    pub ctx: Context,

    pub left_precedence: Precedence,

    pub operator: BinaryishOperator,
    pub wrap: bool,
    pub right_precedence: Precedence,
}

impl<'a> BinaryExpressionVisitor<'a> {
    pub fn gen_expr(v: Self, p: &mut Codegen<'a>) {
        let mut v = v;
        let stack_bottom = p.binary_expr_stack.len();
        loop {
            if !v.check_and_prepare(p) {
                break;
            }

            let left = v.e.left(p);
            let left_binary = match left {
                Expression::BinaryExpression(e) => Some(Binaryish::Binary(e)),
                Expression::LogicalExpression(e) => Some(Binaryish::Logical(e)),
                _ => None,
            };

            let Some(left_binary) = left_binary else {
                if !cjs_module_lexer::try_print_equality_string(p, v.operator, left) {
                    left.print_expr(p, v.left_precedence, v.ctx);
                }
                v.visit_right_and_finish(p);
                break;
            };

            if !p.attached_comments.is_empty() {
                p.start_binary_comments(left_binary.node_id(), |_| {
                    left_binary.needs_parens(v.left_precedence, v.operator, v.ctx)
                });
            }
            p.binary_expr_stack.push(v);
            v = BinaryExpressionVisitor {
                e: left_binary,
                precedence: v.left_precedence,
                ctx: v.ctx,
                left_precedence: Precedence::Lowest,
                operator: v.operator,
                wrap: false,
                right_precedence: Precedence::Lowest,
            };
        }

        loop {
            let len = p.binary_expr_stack.len();
            if len == 0 || len - 1 < stack_bottom {
                break;
            }
            let v = p.binary_expr_stack.pop().unwrap();
            v.visit_right_and_finish(p);
        }
    }

    pub fn check_and_prepare(&mut self, p: &mut Codegen) -> bool {
        let e = self.e;

        // We don't need to print parentheses if both sides use the same logical operator
        // For example: `(a     &&     b)         && c` should be printed as `a && b && c`
        //                      ^^  e.operator()  ^^ self.operator
        self.wrap = e.needs_parens(self.precedence, self.operator, self.ctx);
        self.operator = e.operator();

        if self.wrap {
            p.print_ascii_byte(b'(');
            // `for (1 * (x == a in b);;);`
            //           ^^^^^^^^^^^^ has been wrapped in parens, so it doesn't need to
            //                        print parens for `a in b` again.
            self.ctx &= Context::FORBID_IN.not();
            p.print_deferred_leading_comments(e.node_id());
        }

        self.left_precedence = self.operator.lower_precedence();
        self.right_precedence = self.operator.lower_precedence();

        if self.operator.precedence().is_right_associative() {
            self.left_precedence = self.operator.precedence();
        }

        if self.operator.precedence().is_left_associative() {
            self.right_precedence = self.operator.precedence();
        }

        match self.operator {
            BinaryishOperator::Logical(LogicalOperator::Coalesce) => {
                if let Expression::LogicalExpression(logical_expr) = e.left(p)
                    && matches!(logical_expr.operator, LogicalOperator::And | LogicalOperator::Or)
                {
                    self.left_precedence = Precedence::Prefix;
                }
                if let Expression::LogicalExpression(logical_expr) = e.right(p)
                    && matches!(logical_expr.operator, LogicalOperator::And | LogicalOperator::Or)
                {
                    self.right_precedence = Precedence::Prefix;
                }
            }
            BinaryishOperator::Binary(BinaryOperator::Exponential) => {
                // The base of `**` must be an `UpdateExpression`, so a unary/await base
                // must be parenthesized. Negative numbers and BigInts print with a
                // leading `-`, i.e. as a unary operator.
                if matches!(
                    e.left(p),
                    Expression::UnaryExpression(_)
                        | Expression::AwaitExpression(_)
                        | Expression::TSTypeAssertion(_)
                        | Expression::NumericLiteral(_)
                        | Expression::BigIntLiteral(_)
                ) {
                    self.left_precedence = Precedence::Call;
                }
            }
            BinaryishOperator::Binary(BinaryOperator::BitwiseOR | BinaryOperator::BitwiseAnd) => {
                // Without parentheses, `|` or `&` becomes part of the type in
                // `(value satisfies Type) | other` or `(value satisfies Type) & other`.
                if matches!(e.left(p), Expression::TSSatisfiesExpression(_)) {
                    self.left_precedence = Precedence::Compare;
                }
            }

            _ => {}
        }

        if let Expression::PrivateInExpression(e) = self.e.left(p) {
            e.print_expr(p, self.left_precedence, self.ctx);
            self.visit_right_and_finish(p);
            return false;
        }

        true
    }

    pub fn visit_right_and_finish(&self, p: &mut Codegen) {
        p.print_soft_space();
        self.operator.r#gen(p);
        p.print_soft_space();
        let right = self.e.right(p);
        if let Binaryish::Logical(e) = self.e {
            // Annotation-gated (see the helper's doc): statements get merged
            // into logical RHS positions on mutated ASTs. Pass the unstripped
            // right — `Binaryish::right()` removes the paren layers the helper
            // needs to probe.
            p.print_annotation_comments_before_expression(&e.right);
        }
        if !cjs_module_lexer::try_print_equality_string(p, self.operator, right) {
            right.print_expr(p, self.right_precedence, self.ctx);
        }
        if self.wrap {
            p.print_trailing_comments_inside_parens(self.e.node_id());
            p.print_ascii_byte(b')');
        }
        if !p.attached_comments.is_empty() {
            p.finish_binary_comments(self.e.node_id());
        }
    }
}
