use oxc_ast::{ast::BindingIdentifier, AstKind};
use oxc_ecmascript::ToBoolean;
use oxc_semantic::{AstNode, IsGlobalReference, NodeId, SymbolId};
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator};

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
            let boolean_value = expr.to_boolean();
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
            Self::ArrowFunctionExpression(_)
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
            match_expression!(Self) => self.to_expression().is_constant(in_boolean_position, ctx),
        }
    }
}

impl<'a, 'b> IsConstant<'a, 'b> for ArrayExpressionElement<'a> {
    fn is_constant(&self, in_boolean_position: bool, ctx: &LintContext<'a>) -> bool {
        match self {
            Self::SpreadElement(element) => element.is_constant(in_boolean_position, ctx),
            match_expression!(Self) => self.to_expression().is_constant(in_boolean_position, ctx),
            Self::Elision(_) => true,
        }
    }
}

impl<'a, 'b> IsConstant<'a, 'b> for SpreadElement<'a> {
    fn is_constant(&self, in_boolean_position: bool, ctx: &LintContext<'a>) -> bool {
        self.argument.is_constant(in_boolean_position, ctx)
    }
}

/// Return the innermost `Function` or `ArrowFunctionExpression` Node
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
        if matches!(current_node.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_))
        {
            return Some(current_node);
        }
        current_node = ctx.nodes().parent_node(current_node.id())?;
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
    ctx.nodes()
        .iter_parents(node.id())
        .skip(1)
        .find(|parent| !matches!(parent.kind(), AstKind::ParenthesizedExpression(_)))
}

pub fn nth_outermost_paren_parent<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
    n: usize,
) -> Option<&'b AstNode<'a>> {
    ctx.nodes()
        .iter_parents(node.id())
        .skip(1)
        .filter(|parent| !matches!(parent.kind(), AstKind::ParenthesizedExpression(_)))
        .nth(n)
}
/// Iterate over parents of `node`, skipping nodes that are also ignored by
/// [`Expression::get_inner_expression`].
pub fn iter_outer_expressions<'a, 'ctx>(
    ctx: &'ctx LintContext<'a>,
    node_id: NodeId,
) -> impl Iterator<Item = &'ctx AstNode<'a>> + 'ctx {
    ctx.nodes().iter_parents(node_id).skip(1).filter(|parent| {
        !matches!(
            parent.kind(),
            AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSInstantiationExpression(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TSTypeAssertion(_)
        )
    })
}

pub fn get_declaration_of_variable<'a, 'b>(
    ident: &IdentifierReference,
    ctx: &'b LintContext<'a>,
) -> Option<&'b AstNode<'a>> {
    let symbol_id = get_symbol_id_of_variable(ident, ctx)?;
    let symbol_table = ctx.semantic().symbols();
    Some(ctx.nodes().get_node(symbol_table.get_declaration(symbol_id)))
}

pub fn get_symbol_id_of_variable(
    ident: &IdentifierReference,
    ctx: &LintContext<'_>,
) -> Option<SymbolId> {
    let symbol_table = ctx.semantic().symbols();
    let reference_id = ident.reference_id.get()?;
    let reference = symbol_table.get_reference(reference_id);
    reference.symbol_id()
}

pub fn extract_regex_flags<'a>(
    args: &'a oxc_allocator::Vec<'a, Argument<'a>>,
) -> Option<RegExpFlags> {
    if args.len() <= 1 {
        return None;
    }
    let flag_arg = match &args[1] {
        Argument::StringLiteral(flag_arg) => flag_arg.value.clone(),
        Argument::TemplateLiteral(template) if template.is_no_substitution_template() => {
            template.quasi().expect("no-substitution templates always have a quasi")
        }
        _ => return None,
    };
    let mut flags = RegExpFlags::empty();
    for ch in flag_arg.chars() {
        let flag = RegExpFlags::try_from(ch).ok()?;
        flags |= flag;
    }
    Some(flags)
}

pub fn is_method_call<'a>(
    call_expr: &CallExpression<'a>,
    objects: Option<&[&'a str]>,
    methods: Option<&[&'a str]>,
    min_arg_count: Option<usize>,
    max_arg_count: Option<usize>,
) -> bool {
    if let Some(min_arg_count) = min_arg_count {
        if call_expr.arguments.len() < min_arg_count {
            return false;
        }
    }

    if let Some(max_arg_count) = max_arg_count {
        if call_expr.arguments.len() > max_arg_count {
            return false;
        }
    }

    let Some(member_expr) = call_expr.callee.without_parentheses().as_member_expression() else {
        return false;
    };

    if let Some(objects) = objects {
        let Expression::Identifier(ident) = member_expr.object().without_parentheses() else {
            return false;
        };
        if !objects.contains(&ident.name.as_str()) {
            return false;
        }
    }

    if let Some(methods) = methods {
        let Some(static_property_name) = member_expr.static_property_name() else {
            return false;
        };
        if !methods.contains(&static_property_name) {
            return false;
        }
    }

    true
}

pub fn is_new_expression<'a>(
    new_expr: &NewExpression<'a>,
    names: &[&'a str],
    min_arg_count: Option<usize>,
    max_arg_count: Option<usize>,
) -> bool {
    if let Some(min_arg_count) = min_arg_count {
        if new_expr.arguments.len() < min_arg_count {
            return false;
        }
    }
    if let Some(max_arg_count) = max_arg_count {
        if new_expr.arguments.len() > max_arg_count {
            return false;
        }
    }

    let Expression::Identifier(ident) = new_expr.callee.without_parentheses() else {
        return false;
    };

    if !names.contains(&ident.name.as_str()) {
        return false;
    }

    true
}

pub fn call_expr_method_callee_info<'a>(
    call_expr: &'a CallExpression<'a>,
) -> Option<(Span, &'a str)> {
    let member_expr = call_expr.callee.without_parentheses().as_member_expression()?;
    member_expr.static_property_info()
}

pub fn get_new_expr_ident_name<'a>(new_expr: &'a NewExpression<'a>) -> Option<&'a str> {
    let Expression::Identifier(ident) = new_expr.callee.without_parentheses() else {
        return None;
    };

    Some(ident.name.as_str())
}

pub fn is_global_require_call(call_expr: &CallExpression, ctx: &LintContext) -> bool {
    if call_expr.arguments.len() != 1 {
        return false;
    }
    call_expr.callee.is_global_reference_name("require", ctx.symbols())
}

pub fn is_function_node(node: &AstNode) -> bool {
    match node.kind() {
        AstKind::Function(f) if f.is_function_declaration() => true,
        AstKind::Function(f) if f.is_expression() => true,
        AstKind::ArrowFunctionExpression(_) => true,
        _ => false,
    }
}

pub fn get_function_like_declaration<'b>(
    node: &AstNode<'b>,
    ctx: &LintContext<'b>,
) -> Option<&'b BindingIdentifier<'b>> {
    let parent = outermost_paren_parent(node, ctx)?;
    let decl = parent.kind().as_variable_declarator()?;

    decl.id.get_binding_identifier()
}
