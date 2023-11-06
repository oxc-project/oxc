use std::hash::{Hash, Hasher};

use oxc_ast::AstKind;
use oxc_semantic::AstNode;
use oxc_span::{Atom, GetSpan};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator};
use rustc_hash::FxHasher;

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut hasher = FxHasher::default();
    t.hash(&mut hasher);
    hasher.finish()
}
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::context::LintContext;

/// Test if an AST node is a boolean value that never changes. Specifically we
/// test for:
/// 1. Literal booleans (`true` or `false`)
/// 2. Unary `!` expressions with a constant value
/// 3. Constant booleans created via the `Boolean` global function
pub fn is_static_boolean<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    match expr {
        Expression::BooleanLiteral(_) => true,
        Expression::CallExpression(call_expr) => call_expr.is_constant(true, ctx),
        Expression::UnaryExpression(unary_expr) => {
            unary_expr.operator == UnaryOperator::LogicalNot
                && unary_expr.argument.is_constant(true, ctx)
        }
        _ => false,
    }
}

/// Checks if a branch node of `LogicalExpression` short circuits the whole condition
fn is_logical_identity(op: LogicalOperator, expr: &Expression) -> bool {
    match expr {
        expr if expr.is_literal() => {
            let boolean_value = expr.get_boolean_value();
            (op == LogicalOperator::Or && boolean_value == Some(true))
                || (op == LogicalOperator::And && boolean_value == Some(false))
        }
        Expression::UnaryExpression(unary_expr) => {
            op == LogicalOperator::And && unary_expr.operator == UnaryOperator::Void
        }
        Expression::LogicalExpression(logical_expr) => {
            op == logical_expr.operator
                && (is_logical_identity(logical_expr.operator, &logical_expr.left)
                    || is_logical_identity(logical_expr.operator, &logical_expr.right))
        }
        Expression::AssignmentExpression(assign_expr) => {
            matches!(
                assign_expr.operator,
                AssignmentOperator::LogicalAnd | AssignmentOperator::LogicalOr
            ) && ((op == LogicalOperator::And
                && assign_expr.operator == AssignmentOperator::LogicalAnd)
                || (op == LogicalOperator::Or
                    && assign_expr.operator == AssignmentOperator::LogicalOr))
                && is_logical_identity(op, &assign_expr.right)
        }
        Expression::ParenthesizedExpression(expr) => is_logical_identity(op, &expr.expression),
        _ => false,
    }
}

/// Checks if a  node has a constant truthiness value.
/// `inBooleanPosition`:
///   `true` if checking the test of a condition.
///   `false` in all other cases.
///   When `false`, checks if -- for both string and number --
///   if coerced to that type, the value will be constant.
pub trait IsConstant<'a, 'b> {
    fn is_constant(&self, in_boolean_position: bool, ctx: &LintContext<'a>) -> bool;
}

impl<'a, 'b> IsConstant<'a, 'b> for Expression<'a> {
    fn is_constant(&self, in_boolean_position: bool, ctx: &LintContext<'a>) -> bool {
        match self {
            Self::ArrowExpression(_)
            | Self::FunctionExpression(_)
            | Self::ClassExpression(_)
            | Self::ObjectExpression(_) => true,
            Self::TemplateLiteral(template) => {
                let test_quasis = in_boolean_position
                    && template.quasis.iter().any(|quasi| {
                        quasi.value.cooked.as_ref().map_or(false, |cooked| !cooked.is_empty())
                    });
                let test_expressions =
                    template.expressions.iter().all(|expr| expr.is_constant(false, ctx));
                test_quasis || test_expressions
            }
            Self::ArrayExpression(expr) => {
                if in_boolean_position {
                    return true;
                }
                expr.elements.iter().all(|element| element.is_constant(false, ctx))
            }
            Self::UnaryExpression(expr) => match expr.operator {
                UnaryOperator::Void => true,
                UnaryOperator::Typeof if in_boolean_position => true,
                UnaryOperator::LogicalNot => expr.argument.is_constant(true, ctx),
                _ => expr.argument.is_constant(false, ctx),
            },
            Self::BinaryExpression(expr) => {
                expr.operator != BinaryOperator::In
                    && expr.left.is_constant(false, ctx)
                    && expr.right.is_constant(false, ctx)
            }
            Self::LogicalExpression(expr) => {
                let is_left_constant = expr.left.is_constant(in_boolean_position, ctx);
                let is_right_constant = expr.right.is_constant(in_boolean_position, ctx);
                let is_left_short_circuit =
                    is_left_constant && is_logical_identity(expr.operator, &expr.left);
                let is_right_short_circuit = in_boolean_position
                    && is_right_constant
                    && is_logical_identity(expr.operator, &expr.right);
                (is_left_constant && is_right_constant)
                    || is_left_short_circuit
                    || is_right_short_circuit
            }
            Self::NewExpression(_) => in_boolean_position,
            Self::AssignmentExpression(expr) => match expr.operator {
                AssignmentOperator::Assign => expr.right.is_constant(in_boolean_position, ctx),
                AssignmentOperator::LogicalAnd if in_boolean_position => {
                    is_logical_identity(LogicalOperator::And, &expr.right)
                }
                AssignmentOperator::LogicalOr if in_boolean_position => {
                    is_logical_identity(LogicalOperator::Or, &expr.right)
                }
                _ => false,
            },
            Self::SequenceExpression(sequence_expr) => sequence_expr
                .expressions
                .iter()
                .last()
                .map_or(false, |last| last.is_constant(in_boolean_position, ctx)),
            Self::CallExpression(call_expr) => call_expr.is_constant(in_boolean_position, ctx),
            Self::ParenthesizedExpression(paren_expr) => {
                paren_expr.expression.is_constant(in_boolean_position, ctx)
            }
            Self::Identifier(ident) => {
                ident.name == "undefined" && ctx.semantic().is_reference_to_global_variable(ident)
            }
            _ if self.is_literal() => true,
            _ => false,
        }
    }
}

impl<'a, 'b> IsConstant<'a, 'b> for CallExpression<'a> {
    fn is_constant(&self, _in_boolean_position: bool, ctx: &LintContext<'a>) -> bool {
        if let Expression::Identifier(ident) = &self.callee {
            if ident.name == "Boolean"
                && self.arguments.iter().next().map_or(true, |first| first.is_constant(true, ctx))
            {
                return ctx.semantic().is_reference_to_global_variable(ident);
            }
        }
        false
    }
}

impl<'a, 'b> IsConstant<'a, 'b> for Argument<'a> {
    fn is_constant(&self, in_boolean_position: bool, ctx: &LintContext<'a>) -> bool {
        match self {
            Self::SpreadElement(element) => element.is_constant(in_boolean_position, ctx),
            Self::Expression(expr) => expr.is_constant(in_boolean_position, ctx),
        }
    }
}

impl<'a, 'b> IsConstant<'a, 'b> for ArrayExpressionElement<'a> {
    fn is_constant(&self, in_boolean_position: bool, ctx: &LintContext<'a>) -> bool {
        match self {
            Self::SpreadElement(element) => element.is_constant(in_boolean_position, ctx),
            Self::Expression(expr) => expr.is_constant(in_boolean_position, ctx),
            Self::Elision(_) => true,
        }
    }
}

impl<'a, 'b> IsConstant<'a, 'b> for SpreadElement<'a> {
    fn is_constant(&self, in_boolean_position: bool, ctx: &LintContext<'a>) -> bool {
        self.argument.is_constant(in_boolean_position, ctx)
    }
}

pub trait IsPrivate {
    fn is_private(&self) -> bool;
}

impl<'a> IsPrivate for MethodDefinition<'a> {
    fn is_private(&self) -> bool {
        self.accessibility.map_or(false, |accessibility| accessibility == TSAccessibility::Private)
    }
}

impl<'a> IsPrivate for PropertyDefinition<'a> {
    fn is_private(&self) -> bool {
        self.accessibility.map_or(false, |accessibility| accessibility == TSAccessibility::Private)
    }
}

/// Return the innermost `Function` or `ArrowExpression` Node
/// enclosing the specified node
pub fn get_enclosing_function<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b AstNode<'a>> {
    let mut current_node = node;
    loop {
        if matches!(current_node.kind(), AstKind::Program(_)) {
            return None;
        }
        if matches!(current_node.kind(), AstKind::Function(_) | AstKind::ArrowExpression(_)) {
            return Some(current_node);
        }
        current_node = ctx.nodes().parent_node(current_node.id()).unwrap();
    }
}

/// Returns if `arg` is the `n`th (0-indexed) argument of `call`.
pub fn is_nth_argument<'a>(call: &CallExpression<'a>, arg: &Argument<'a>, n: usize) -> bool {
    let nth = &call.arguments[n];
    nth.span() == arg.span()
}

/// Jump to the outer most of chained parentheses if any
pub fn outermost_paren<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> &'b AstNode<'a> {
    let mut node = node;

    loop {
        if let Some(parent) = ctx.nodes().parent_node(node.id()) {
            if let AstKind::ParenthesizedExpression(_) = parent.kind() {
                node = parent;
                continue;
            }
        }

        break;
    }

    node
}

pub fn outermost_paren_parent<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b AstNode<'a>> {
    let mut node = node;

    loop {
        if let Some(parent) = ctx.nodes().parent_node(node.id()) {
            if let AstKind::ParenthesizedExpression(_) = parent.kind() {
                node = parent;
                continue;
            }
        }

        break;
    }

    ctx.nodes().parent_node(node.id())
}

pub fn get_name_from_property_key(key: &PropertyKey<'_>) -> Option<Atom> {
    match key {
        PropertyKey::Identifier(ident) => Some(ident.name.clone()),
        PropertyKey::PrivateIdentifier(ident) => {
            let name = ident.name.clone();

            Some(Atom::from(format!("#{name}")))
        }
        PropertyKey::Expression(expr) => match expr {
            Expression::StringLiteral(lit) => Some(lit.value.clone()),
            Expression::RegExpLiteral(lit) => Some(Atom::from(format!("{0}", lit.regex))),
            Expression::NumberLiteral(lit) => Some(Atom::from(lit.raw)),
            Expression::BigintLiteral(lit) => Some(Atom::from(format!("{0}", lit.value))),
            Expression::NullLiteral(_) => Some("null".into()),
            Expression::TemplateLiteral(lit) => {
                lit.expressions.is_empty().then(|| lit.quasi()).flatten().cloned()
            }
            _ => None,
        },
    }
}

pub fn get_declaration_of_variable<'a, 'b>(
    ident: &IdentifierReference,
    ctx: &'b LintContext<'a>,
) -> Option<&'b AstNode<'a>> {
    let symbol_table = ctx.semantic().symbols();
    let reference_id = ident.reference_id.get()?;
    let reference = symbol_table.get_reference(reference_id);
    let symbol_id = reference.symbol_id()?;
    Some(ctx.nodes().get_node(symbol_table.get_declaration(symbol_id)))
}
