//! Visit binary and logical expression in a loop without recursion.
//!
//! Reference: <https://github.com/evanw/esbuild/blob/78f89e41d5e8a7088f4820351c6305cc339f8820/internal/js_printer/js_printer.go#L3266>
use std::ops::Not;

use oxc_ast::ast::{BinaryExpression, Expression, LogicalExpression};
use oxc_syntax::{
    operator::{BinaryOperator, LogicalOperator},
    precedence::{GetPrecedence, Precedence},
};

use crate::{gen::GenExpr, Codegen, Context, Operator};

#[derive(Clone, Copy)]
pub(crate) enum Binaryish<'a> {
    Binary(&'a BinaryExpression<'a>),
    Logical(&'a LogicalExpression<'a>),
}

impl<'a> Binaryish<'a> {
    pub fn left(&self) -> &'a Expression<'a> {
        match self {
            Self::Binary(e) => e.left.without_parentheses(),
            Self::Logical(e) => e.left.without_parentheses(),
        }
    }

    pub fn right(&self) -> &'a Expression<'a> {
        match self {
            Self::Binary(e) => e.right.without_parentheses(),
            Self::Logical(e) => e.right.without_parentheses(),
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
    fn gen(self, p: &mut Codegen) {
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
    pub left_ctx: Context,

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

            let left = v.e.left();
            let left_binary = match left {
                Expression::BinaryExpression(e) => Some(Binaryish::Binary(e)),
                Expression::LogicalExpression(e) => Some(Binaryish::Logical(e)),
                _ => None,
            };

            let Some(left_binary) = left_binary else {
                left.gen_expr(p, v.left_precedence, v.left_ctx);
                v.visit_right_and_finish(p);
                break;
            };

            p.binary_expr_stack.push(v);
            v = BinaryExpressionVisitor {
                e: left_binary,
                precedence: v.left_precedence,
                ctx: v.left_ctx,
                left_precedence: Precedence::Lowest,
                left_ctx: Context::empty(),
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
        self.operator = e.operator();

        self.wrap = self.precedence >= self.operator.precedence()
            || (self.operator == BinaryishOperator::Binary(BinaryOperator::In)
                && self.ctx.intersects(Context::FORBID_IN));

        if self.wrap {
            p.print_ascii_byte(b'(');
            self.ctx &= Context::FORBID_IN.not();
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
                if let Expression::LogicalExpression(logical_expr) = e.left() {
                    if matches!(logical_expr.operator, LogicalOperator::And | LogicalOperator::Or) {
                        self.left_precedence = Precedence::Prefix;
                    }
                }
                if let Expression::LogicalExpression(logical_expr) = e.right() {
                    if matches!(logical_expr.operator, LogicalOperator::And | LogicalOperator::Or) {
                        self.right_precedence = Precedence::Prefix;
                    }
                }
            }
            BinaryishOperator::Binary(BinaryOperator::Exponential) => {
                if matches!(e.left(), Expression::UnaryExpression(_)) {
                    self.left_precedence = Precedence::Call;
                }
            }
            _ => {}
        }

        true
    }

    pub fn visit_right_and_finish(&self, p: &mut Codegen) {
        p.print_soft_space();
        self.operator.gen(p);
        p.print_soft_space();
        self.e.right().gen_expr(p, self.right_precedence, self.ctx & Context::FORBID_IN);
        if self.wrap {
            p.print_ascii_byte(b')');
        }
    }
}
