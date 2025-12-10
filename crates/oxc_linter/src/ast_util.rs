use std::borrow::Cow;

use rustc_hash::FxHashSet;

use oxc_allocator::GetAddress;
use oxc_ast::{
    AstKind,
    ast::{BindingIdentifier, *},
};
use oxc_ecmascript::{ToBoolean, WithoutGlobalReferenceInformation};
use oxc_semantic::{AstNode, AstNodes, IsGlobalReference, NodeId, ReferenceId, Semantic, SymbolId};
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    identifier::is_irregular_whitespace,
    operator::{AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator},
};

use crate::{LintContext, utils::get_function_nearest_jsdoc_node};

/// Test if an AST node is a boolean value that never changes. Specifically we
/// test for:
/// 1. Literal booleans (`true` or `false`)
/// 2. Unary `!` expressions with a constant value
/// 3. Constant booleans created via the `Boolean` global function
pub fn is_static_boolean<'a>(expr: &Expression<'a>, semantic: &Semantic<'a>) -> bool {
    match expr {
        Expression::BooleanLiteral(_) => true,
        Expression::CallExpression(call_expr) => call_expr.is_constant(true, semantic),
        Expression::UnaryExpression(unary_expr) => {
            unary_expr.operator == UnaryOperator::LogicalNot
                && unary_expr.argument.is_constant(true, semantic)
        }
        _ => false,
    }
}

/// Checks if a branch node of `LogicalExpression` short circuits the whole condition
fn is_logical_identity(op: LogicalOperator, expr: &Expression) -> bool {
    match expr {
        expr if expr.is_literal() => {
            let boolean_value = expr.to_boolean(&WithoutGlobalReferenceInformation {});
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
    fn is_constant(&self, in_boolean_position: bool, semantic: &Semantic<'a>) -> bool;
}

impl<'a> IsConstant<'a, '_> for Expression<'a> {
    fn is_constant(&self, in_boolean_position: bool, semantic: &Semantic<'a>) -> bool {
        match self {
            Self::ArrowFunctionExpression(_)
            | Self::FunctionExpression(_)
            | Self::ClassExpression(_)
            | Self::ObjectExpression(_) => true,
            Self::TemplateLiteral(template) => {
                let test_quasis = in_boolean_position
                    && template.quasis.iter().any(|quasi| {
                        quasi.value.cooked.as_ref().is_some_and(|cooked| !cooked.is_empty())
                    });
                let test_expressions =
                    template.expressions.iter().all(|expr| expr.is_constant(false, semantic));
                test_quasis || test_expressions
            }
            Self::ArrayExpression(expr) => {
                if in_boolean_position {
                    return true;
                }
                expr.elements.iter().all(|element| element.is_constant(false, semantic))
            }
            Self::UnaryExpression(expr) => match expr.operator {
                UnaryOperator::Void => true,
                UnaryOperator::Typeof if in_boolean_position => true,
                UnaryOperator::LogicalNot => expr.argument.is_constant(true, semantic),
                _ => expr.argument.is_constant(false, semantic),
            },
            Self::BinaryExpression(expr) => {
                expr.operator != BinaryOperator::In
                    && expr.left.is_constant(false, semantic)
                    && expr.right.is_constant(false, semantic)
            }
            Self::LogicalExpression(expr) => {
                let is_left_constant = expr.left.is_constant(in_boolean_position, semantic);
                let is_right_constant = expr.right.is_constant(in_boolean_position, semantic);
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
                AssignmentOperator::Assign => expr.right.is_constant(in_boolean_position, semantic),
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
                .is_some_and(|last| last.is_constant(in_boolean_position, semantic)),
            Self::CallExpression(call_expr) => call_expr.is_constant(in_boolean_position, semantic),
            Self::ParenthesizedExpression(paren_expr) => {
                paren_expr.expression.is_constant(in_boolean_position, semantic)
            }
            Self::Identifier(ident) => {
                ident.name == "undefined" && semantic.is_reference_to_global_variable(ident)
            }
            _ if self.is_literal() => true,
            _ => false,
        }
    }
}

impl<'a> IsConstant<'a, '_> for CallExpression<'a> {
    fn is_constant(&self, _in_boolean_position: bool, semantic: &Semantic<'a>) -> bool {
        if let Expression::Identifier(ident) = &self.callee
            && ident.name == "Boolean"
            && self.arguments.iter().next().is_none_or(|first| first.is_constant(true, semantic))
        {
            return semantic.is_reference_to_global_variable(ident);
        }
        false
    }
}

impl<'a> IsConstant<'a, '_> for Argument<'a> {
    fn is_constant(&self, in_boolean_position: bool, semantic: &Semantic<'a>) -> bool {
        match self {
            Self::SpreadElement(element) => element.is_constant(in_boolean_position, semantic),
            match_expression!(Self) => {
                self.to_expression().is_constant(in_boolean_position, semantic)
            }
        }
    }
}

impl<'a> IsConstant<'a, '_> for ArrayExpressionElement<'a> {
    fn is_constant(&self, in_boolean_position: bool, semantic: &Semantic<'a>) -> bool {
        match self {
            Self::SpreadElement(element) => element.is_constant(in_boolean_position, semantic),
            match_expression!(Self) => {
                self.to_expression().is_constant(in_boolean_position, semantic)
            }
            Self::Elision(_) => true,
        }
    }
}

impl<'a> IsConstant<'a, '_> for SpreadElement<'a> {
    fn is_constant(&self, in_boolean_position: bool, semantic: &Semantic<'a>) -> bool {
        self.argument.is_constant(in_boolean_position, semantic)
    }
}

/// Return the innermost `Function` or `ArrowFunctionExpression` Node
/// enclosing the specified node
pub fn get_enclosing_function<'a, 'b>(
    node: &'b AstNode<'a>,
    semantic: &'b Semantic<'a>,
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
        current_node = semantic.nodes().parent_node(current_node.id());
    }
}

/// Jump to the outer most of chained parentheses if any
pub fn outermost_paren<'a, 'b>(
    node: &'b AstNode<'a>,
    semantic: &'b Semantic<'a>,
) -> &'b AstNode<'a> {
    let mut node = node;

    loop {
        let parent = semantic.nodes().parent_node(node.id());
        if let AstKind::ParenthesizedExpression(_) = parent.kind() {
            node = parent;
            continue;
        }

        break;
    }

    node
}

pub fn outermost_paren_parent<'a, 'b>(
    node: &'b AstNode<'a>,
    semantic: &'b Semantic<'a>,
) -> Option<&'b AstNode<'a>> {
    semantic
        .nodes()
        .ancestors(node.id())
        .find(|parent| !matches!(parent.kind(), AstKind::ParenthesizedExpression(_)))
}

pub fn nth_outermost_paren_parent<'a, 'b>(
    node: &'b AstNode<'a>,
    semantic: &'b Semantic<'a>,
    n: usize,
) -> Option<&'b AstNode<'a>> {
    semantic
        .nodes()
        .ancestors(node.id())
        .filter(|parent| !matches!(parent.kind(), AstKind::ParenthesizedExpression(_)))
        .nth(n)
}

/// Iterate over parents of `node`, skipping nodes that are also ignored by
/// [`Expression::get_inner_expression`].
pub fn iter_outer_expressions<'a, 's>(
    nodes: &'s AstNodes<'a>,
    node_id: NodeId,
) -> impl Iterator<Item = AstKind<'a>> + 's {
    nodes.ancestor_kinds(node_id).filter(|parent| {
        !matches!(
            parent,
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
    ident: &IdentifierReference<'a>,
    semantic: &'b Semantic<'a>,
) -> Option<&'b AstNode<'a>> {
    let symbol_id = get_symbol_id_of_variable(ident, semantic)?;
    let symbol_table = semantic.scoping();
    Some(semantic.nodes().get_node(symbol_table.symbol_declaration(symbol_id)))
}

pub fn get_declaration_from_reference_id<'a, 'b>(
    reference_id: ReferenceId,
    semantic: &'b Semantic<'a>,
) -> Option<&'b AstNode<'a>> {
    let symbol_table = semantic.scoping();
    let symbol_id = symbol_table.get_reference(reference_id).symbol_id()?;
    Some(semantic.nodes().get_node(symbol_table.symbol_declaration(symbol_id)))
}

pub fn get_symbol_id_of_variable(
    ident: &IdentifierReference,
    semantic: &Semantic<'_>,
) -> Option<SymbolId> {
    semantic.scoping().get_reference(ident.reference_id()).symbol_id()
}

pub fn extract_regex_flags<'a>(
    args: &'a oxc_allocator::Vec<'a, Argument<'a>>,
) -> Option<RegExpFlags> {
    if args.len() <= 1 {
        return None;
    }
    let flag_arg = match &args[1] {
        Argument::StringLiteral(flag_arg) => flag_arg.value,
        Argument::TemplateLiteral(template) => template.single_quasi()?,
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
    if let Some(min_arg_count) = min_arg_count
        && call_expr.arguments.len() < min_arg_count
    {
        return false;
    }

    if let Some(max_arg_count) = max_arg_count
        && call_expr.arguments.len() > max_arg_count
    {
        return false;
    }

    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return false;
    };

    if let Some(objects) = objects {
        let Expression::Identifier(ident) = member_expr.object().get_inner_expression() else {
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
    if let Some(min_arg_count) = min_arg_count
        && new_expr.arguments.len() < min_arg_count
    {
        return false;
    }
    if let Some(max_arg_count) = max_arg_count
        && new_expr.arguments.len() > max_arg_count
    {
        return false;
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
    let member_expr = call_expr.callee.get_inner_expression().as_member_expression()?;
    member_expr.static_property_info()
}

pub fn get_new_expr_ident_name<'a>(new_expr: &'a NewExpression<'a>) -> Option<&'a str> {
    let Expression::Identifier(ident) = new_expr.callee.without_parentheses() else {
        return None;
    };

    Some(ident.name.as_str())
}

pub fn is_global_require_call(call_expr: &CallExpression, ctx: &Semantic) -> bool {
    if call_expr.arguments.len() != 1 {
        return false;
    }
    call_expr.callee.is_global_reference_name("require", ctx.scoping())
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
    ctx: &Semantic<'b>,
) -> Option<&'b BindingIdentifier<'b>> {
    let parent = outermost_paren_parent(node, ctx)?;
    let decl = parent.kind().as_variable_declarator()?;

    decl.id.get_binding_identifier()
}

/// Get the first identifier reference within a member expression chain or
/// standalone reference.
///
/// For example, when called on the right-hand side of this [`AssignmentExpression`]:
/// ```ts
/// let x = a
/// //      ^
/// let y = a.b.c
/// //      ^
/// ```
///
/// As this function walks down the member expression chain, if no identifier
/// reference is found, it returns [`Err`] with the leftmost expression.
/// ```ts
/// let x = 1 + 1
/// //      ^^^^^ Err(BinaryExpression)
/// let y = this.foo.bar
/// //      ^^^^ Err(ThisExpression)
/// ```
pub fn leftmost_identifier_reference<'a, 'b: 'a>(
    expr: &'b Expression<'a>,
) -> Result<&'a IdentifierReference<'a>, &'b Expression<'a>> {
    match expr {
        Expression::Identifier(ident) => Ok(ident.as_ref()),
        Expression::StaticMemberExpression(mem) => leftmost_identifier_reference(&mem.object),
        Expression::ComputedMemberExpression(mem) => leftmost_identifier_reference(&mem.object),
        Expression::PrivateFieldExpression(mem) => leftmost_identifier_reference(&mem.object),
        _ => Err(expr),
    }
}

fn is_definitely_non_error_type(ty: &TSType) -> bool {
    match ty {
        TSType::TSNumberKeyword(_)
        | TSType::TSStringKeyword(_)
        | TSType::TSBooleanKeyword(_)
        | TSType::TSNullKeyword(_)
        | TSType::TSUndefinedKeyword(_) => true,
        TSType::TSUnionType(union) => union.types.iter().all(is_definitely_non_error_type),
        TSType::TSIntersectionType(intersect) => {
            intersect.types.iter().all(is_definitely_non_error_type)
        }
        _ => false,
    }
}
/// Get the preceding indentation string before the start of a Span in a given source_text string slice. Useful for maintaining the format of source code when applying a linting fix.
///
/// Slice into source_text until the start of given Span.
/// Then, get the preceding spaces from the last line of the source_text.
/// If there are any non-whitespace characters preceding the Span in the last line of source_text, return None.
///
/// Examples:
///
/// 1. Given the following source_text (with 2 preceding spaces):
///
/// ```ts
///   break
/// ```
///
/// and the Span encapsulating the break statement,
///
/// this function will return "  " (2 preceding spaces).
///
/// 2. Given the following source_text:
///
/// ```ts
/// const foo = 'bar'; break;
/// ```
///
/// and the Span encapsulating the break statement,
///
/// this function will return None because there is non-whitespace before the statement,
/// meaning the line of source_text containing the Span is not indented on a new line.
pub fn get_preceding_indent_str(source_text: &str, span: Span) -> Option<&str> {
    let span_start = span.start as usize;
    let preceding_source_text = &source_text[..span_start];

    // only return last line if is whitespace
    preceding_source_text.lines().last().filter(|&line| line.trim().is_empty())
}

pub fn could_be_error(ctx: &LintContext, expr: &Expression) -> bool {
    could_be_error_impl(ctx, expr, &mut FxHashSet::default())
}

fn could_be_error_impl(
    ctx: &LintContext,
    expr: &Expression,
    visited: &mut FxHashSet<SymbolId>,
) -> bool {
    match expr.get_inner_expression() {
        Expression::NewExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::CallExpression(_)
        | Expression::ChainExpression(_)
        | Expression::YieldExpression(_)
        | Expression::PrivateFieldExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::TaggedTemplateExpression(_) => true,
        Expression::AssignmentExpression(expr) => {
            if matches!(expr.operator, AssignmentOperator::Assign | AssignmentOperator::LogicalAnd)
            {
                return could_be_error_impl(ctx, &expr.right, visited);
            }

            if matches!(
                expr.operator,
                AssignmentOperator::LogicalOr | AssignmentOperator::LogicalNullish
            ) {
                return expr
                    .left
                    .get_expression()
                    .is_none_or(|expr| could_be_error_impl(ctx, expr, visited))
                    || could_be_error_impl(ctx, &expr.right, visited);
            }

            false
        }
        Expression::SequenceExpression(expr) => {
            expr.expressions.last().is_some_and(|expr| could_be_error_impl(ctx, expr, visited))
        }
        Expression::LogicalExpression(expr) => {
            if matches!(expr.operator, LogicalOperator::And) {
                return could_be_error_impl(ctx, &expr.right, visited);
            }

            could_be_error_impl(ctx, &expr.left, visited)
                || could_be_error_impl(ctx, &expr.right, visited)
        }
        Expression::ConditionalExpression(expr) => {
            could_be_error_impl(ctx, &expr.consequent, visited)
                || could_be_error_impl(ctx, &expr.alternate, visited)
        }
        Expression::Identifier(ident) => {
            let reference = ctx.scoping().get_reference(ident.reference_id());
            let Some(symbol_id) = reference.symbol_id() else {
                return true;
            };

            // Check if we've already visited this symbol to prevent infinite recursion
            // Return true (could be error) when we encounter a circular reference since we can't determine the type
            if !visited.insert(symbol_id) {
                return true;
            }

            let decl = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));
            match decl.kind() {
                AstKind::VariableDeclarator(decl) => {
                    if let Some(init) = &decl.init {
                        could_be_error_impl(ctx, init, visited)
                    } else {
                        // TODO: warn about throwing undefined
                        false
                    }
                }
                AstKind::Function(_)
                | AstKind::Class(_)
                | AstKind::TSModuleDeclaration(_)
                | AstKind::TSGlobalDeclaration(_)
                | AstKind::TSEnumDeclaration(_) => false,
                AstKind::FormalParameter(param) => !param
                    .pattern
                    .type_annotation
                    .as_ref()
                    .is_some_and(|annot| is_definitely_non_error_type(&annot.type_annotation)),
                _ => true,
            }
        }
        _ => false,
    }
}

pub fn is_callee<'a>(node: &AstNode<'a>, semantic: &Semantic<'a>) -> bool {
    let parent = outermost_paren_parent(node, semantic);
    parent.is_some_and(|node | matches!(node.kind(), AstKind::CallExpression(call_expr) if call_expr.callee.span().contains_inclusive(node.kind().span())))
}

fn has_jsdoc_this_tag<'a>(semantic: &Semantic<'a>, node: &AstNode<'a>) -> bool {
    let Some(jsdocs) = get_function_nearest_jsdoc_node(node, semantic)
        .and_then(|node| semantic.jsdoc().get_all_by_node(semantic.nodes(), node))
    else {
        return false;
    };

    for jsdoc in jsdocs {
        for tag in jsdoc.tags() {
            if tag.kind.parsed() == "this" {
                return true;
            }
        }
    }

    false
}

const METHOD_WHICH_HAS_THIS_ARG: [&str; 10] = [
    "every",
    "filter",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "flatMap",
    "forEach",
    "map",
    "some",
];

pub fn is_default_this_binding<'a>(
    semantic: &Semantic<'a>,
    node: &AstNode<'a>,
    cap_is_constructor: bool,
) -> bool {
    let is_anonymous = match node.kind() {
        AstKind::Function(func) if cap_is_constructor => {
            let is_constructor = func.id.as_ref().is_some_and(|id| {
                id.name.chars().next().is_some_and(|char| char.is_ascii_uppercase())
            });

            if is_constructor || has_jsdoc_this_tag(semantic, node) {
                return false;
            }

            func.id.is_some()
        }
        AstKind::StaticBlock(_) => {
            return false;
        }
        _ => true,
    };

    let is_anonymous_and_cap_is_constructor = is_anonymous && cap_is_constructor;

    let mut current_node = node;
    loop {
        let parent = semantic.nodes().parent_node(current_node.id());
        let parent_kind = parent.kind();
        match parent_kind {
            AstKind::ChainExpression(_)
            | AstKind::ConditionalExpression(_)
            | AstKind::LogicalExpression(_)
            | AstKind::ParenthesizedExpression(_) => {
                current_node = parent;
            }
            AstKind::ReturnStatement(_) => {
                let upper_func = semantic.nodes().ancestors(parent.id()).find(|node| {
                    matches!(
                        node.kind(),
                        AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
                    )
                });
                if upper_func.is_none_or(|node| !is_callee(node, semantic)) {
                    return true;
                }
                current_node = parent;
            }
            AstKind::ArrowFunctionExpression(expr) => {
                if current_node.span() != expr.body.span || !is_callee(parent, semantic) {
                    return true;
                }
                current_node = parent;
            }
            AstKind::ObjectProperty(obj) => {
                return obj.value.span() != current_node.span();
            }
            AstKind::MethodDefinition(method) => {
                return method.value.span != current_node.span();
            }
            AstKind::PropertyDefinition(def) => {
                if let Some(expr) = &def.value {
                    return expr.span() != current_node.span();
                }
                return true;
            }
            AstKind::AssignmentExpression(expr) => {
                if expr.left.is_member_expression() {
                    return false;
                }

                let is_constructor = is_anonymous_and_cap_is_constructor
                    && expr.left.get_identifier_name().is_some_and(|name| {
                        name.chars().next().is_some_and(|char| char.is_ascii_uppercase())
                    });

                return !is_constructor;
            }
            AstKind::AssignmentPattern(pattern) => {
                let is_constructor = is_anonymous_and_cap_is_constructor
                    && pattern.left.get_identifier_name().is_some_and(|name| {
                        name.chars().next().is_some_and(|char| char.is_ascii_uppercase())
                    });

                return !is_constructor;
            }
            AstKind::VariableDeclarator(var) => {
                let is_constructor = is_anonymous_and_cap_is_constructor
                    && var.init.as_ref().is_some_and(|init| init.span() == current_node.span())
                    && var.id.get_identifier_name().is_some_and(|name| {
                        name.chars().next().is_some_and(|char| char.is_ascii_uppercase())
                    });

                return !is_constructor;
            }
            AstKind::StaticMemberExpression(_) | AstKind::ComputedMemberExpression(_) => {
                let member_expr_kind = parent_kind.as_member_expression_kind().unwrap();
                if member_expr_kind.object().span() == current_node.span()
                    && member_expr_kind
                        .static_property_name()
                        .is_some_and(|name| name == "apply" || name == "bind" || name == "call")
                {
                    let node = outermost_paren_parent(parent, semantic).unwrap();
                    if let AstKind::CallExpression(call_expr) = node.kind()
                        && let Some(arg) =
                            call_expr.arguments.first().and_then(|arg| arg.as_expression())
                    {
                        return arg.is_null_or_undefined();
                    }
                }
                return true;
            }
            AstKind::CallExpression(call_expr) => {
                if call_expr.callee.is_specific_member_access("Reflect", "apply") {
                    return call_expr.arguments.len() != 3
                        || call_expr.arguments[0].span() != current_node.span()
                        || call_expr.arguments[1]
                            .as_expression()
                            .is_some_and(Expression::is_null_or_undefined);
                }
                if call_expr.callee.is_specific_member_access("Array", "from") {
                    return call_expr.arguments.len() != 3
                        || call_expr.arguments[1].span() != current_node.span()
                        || call_expr.arguments[2]
                            .as_expression()
                            .is_some_and(Expression::is_null_or_undefined);
                }
                if call_expr.callee.get_member_expr().is_some_and(|mem_expr| {
                    mem_expr
                        .static_property_name()
                        .is_some_and(|name| METHOD_WHICH_HAS_THIS_ARG.binary_search(&name).is_ok())
                }) {
                    return call_expr.arguments.len() != 2
                        || call_expr.arguments[0].span() != current_node.span()
                        || call_expr.arguments[1]
                            .as_expression()
                            .is_some_and(Expression::is_null_or_undefined);
                }
                return true;
            }
            _ => return true,
        }
    }
}

pub fn get_static_property_name<'a>(parent_node: &AstNode<'a>) -> Option<Cow<'a, str>> {
    let (key, computed) = match parent_node.kind() {
        AstKind::PropertyDefinition(definition) => (&definition.key, definition.computed),
        AstKind::MethodDefinition(method_definition) => {
            (&method_definition.key, method_definition.computed)
        }
        AstKind::ObjectProperty(property) => (&property.key, property.computed),
        _ => return None,
    };

    if key.is_identifier() && !computed {
        return key.name();
    }

    if matches!(key, PropertyKey::NullLiteral(_)) {
        return Some("null".into());
    }

    match key {
        PropertyKey::RegExpLiteral(regex) => Some(Cow::Owned(regex.regex.to_string())),
        PropertyKey::BigIntLiteral(bigint) => Some(Cow::Borrowed(bigint.value.as_str())),
        PropertyKey::TemplateLiteral(template) => {
            if template.expressions.is_empty()
                && template.quasis.len() == 1
                && let Some(cooked) = &template.quasis[0].value.cooked
            {
                return Some(Cow::Borrowed(cooked.as_str()));
            }

            None
        }
        _ => None,
    }
}

/// Gets the name and kind of the given function node.
/// @see <https://github.com/eslint/eslint/blob/48117b27e98639ffe7e78a230bfad9a93039fb7f/lib/rules/utils/ast-utils.js#L1762>
pub fn get_function_name_with_kind<'a>(
    node: &AstNode<'a>,
    parent_node: &AstNode<'a>,
) -> Cow<'a, str> {
    let (name, is_async, is_generator) = match node.kind() {
        AstKind::Function(func) => (func.name(), func.r#async, func.generator),
        AstKind::ArrowFunctionExpression(arrow_func) => (None, arrow_func.r#async, false),
        _ => (None, false, false),
    };

    let mut tokens: Vec<Cow<'a, str>> = vec![];

    match parent_node.kind() {
        AstKind::MethodDefinition(definition) => {
            if !definition.computed && definition.key.is_private_identifier() {
                tokens.push(Cow::Borrowed("private"));
            } else if let Some(accessibility) = definition.accessibility {
                tokens.push(Cow::Borrowed(accessibility.as_str()));
            }

            if definition.r#static {
                tokens.push(Cow::Borrowed("static"));
            }
        }
        AstKind::PropertyDefinition(definition) => {
            if !definition.computed && definition.key.is_private_identifier() {
                tokens.push(Cow::Borrowed("private"));
            } else if let Some(accessibility) = definition.accessibility {
                tokens.push(Cow::Borrowed(accessibility.as_str()));
            }

            if definition.r#static {
                tokens.push(Cow::Borrowed("static"));
            }
        }
        _ => {}
    }

    if is_async {
        tokens.push(Cow::Borrowed("async"));
    }

    if is_generator {
        tokens.push(Cow::Borrowed("generator"));
    }

    match parent_node.kind() {
        AstKind::MethodDefinition(method_definition) => match method_definition.kind {
            MethodDefinitionKind::Constructor => tokens.push(Cow::Borrowed("constructor")),
            MethodDefinitionKind::Get => tokens.push(Cow::Borrowed("getter")),
            MethodDefinitionKind::Set => tokens.push(Cow::Borrowed("setter")),
            MethodDefinitionKind::Method => tokens.push(Cow::Borrowed("method")),
        },
        AstKind::PropertyDefinition(_) => tokens.push(Cow::Borrowed("method")),
        _ => tokens.push(Cow::Borrowed("function")),
    }

    let method_name = match parent_node.kind() {
        AstKind::MethodDefinition(method_definition)
            if !method_definition.computed && method_definition.key.is_private_identifier() =>
        {
            method_definition.key.name()
        }
        AstKind::PropertyDefinition(definition) => {
            if !definition.computed && definition.key.is_private_identifier() {
                definition.key.name()
            } else if let Some(static_name) = get_static_property_name(parent_node) {
                Some(static_name)
            } else if let Some(name) = name {
                Some(Cow::Borrowed(name.as_str()))
            } else {
                None
            }
        }
        _ => {
            if let Some(static_name) = get_static_property_name(parent_node) {
                Some(static_name)
            } else if let Some(name) = name {
                Some(Cow::Borrowed(name.as_str()))
            } else {
                None
            }
        }
    };

    if let Some(method_name) = method_name {
        tokens.push(Cow::Owned(format!("`{method_name}`")));
    }

    Cow::Owned(tokens.join(" "))
}

// get the top iterator
// example: this.state.a.b.c.d => this.state
pub fn get_outer_member_expression<'a, 'b>(
    assignment: &'b SimpleAssignmentTarget<'a>,
) -> Option<&'b StaticMemberExpression<'a>> {
    match assignment {
        SimpleAssignmentTarget::StaticMemberExpression(expr) => {
            let mut node = &**expr;
            loop {
                if node.object.is_null() {
                    return Some(node);
                }

                if let Some(MemberExpression::StaticMemberExpression(object)) =
                    node.object.as_member_expression()
                    && !object.property.name.is_empty()
                {
                    node = object;

                    continue;
                }

                return Some(node);
            }
        }
        _ => None,
    }
}
/// Check if a node is an argument (not callee) of a call or new expression
pub fn is_node_call_like_argument<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    // foo.bar<string>(arg0, arg1)
    // ^^^^^^^
    //        ^^^^^^^^
    //                ^^^^^^^^^^^
    // A call/new expression has 3 parts, callee, type arguments, and arguments.
    // This function checks if the given node is within the arguments part.
    // The type parameter part **must** be `TSTypeParameterInstantiation` if present,
    // so we can early return false in that case.
    let parent = ctx.nodes().parent_node(node.id());

    if matches!(node.kind(), AstKind::TSTypeParameterInstantiation(_)) {
        return false;
    }

    match parent.kind() {
        AstKind::CallExpression(call) => node.address() != call.callee.address(),
        AstKind::NewExpression(new_expr) => node.address() != new_expr.callee.address(),
        _ => false,
    }
}

/// Check if a node's span is contained within any argument span in a call expression
#[inline]
pub fn is_node_within_call_argument<'a>(
    node: &AstNode<'a>,
    call: &CallExpression<'a>,
    target_arg_index: usize,
) -> bool {
    // Early exit for out-of-bounds index
    if target_arg_index >= call.arguments.len() {
        return false;
    }

    let target_arg = &call.arguments[target_arg_index]; // Direct indexing, no Option unwrap
    let node_span = node.span();
    let arg_span = target_arg.span();
    node_span.start >= arg_span.start && node_span.end <= arg_span.end
}

/// Determines if a semicolon is needed before inserting code that starts with
/// certain characters (`[`, `(`, `/`, `+`, `-`, `` ` ``) that could be misinterpreted
/// due to Automatic Semicolon Insertion (ASI) rules.
///
/// Returns `true` if the node is at the start of an `ExpressionStatement` and the
/// character before it could cause the replacement to be parsed as a continuation
/// of the previous expression.
pub fn could_be_asi_hazard(node: &AstNode, ctx: &LintContext) -> bool {
    let node_span = node.span();

    // Find the enclosing ExpressionStatement, bailing early for nodes that can't
    // be at statement start position
    let mut expr_stmt_span = None;
    for ancestor in ctx.nodes().ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::ExpressionStatement(expr_stmt) => {
                expr_stmt_span = Some(expr_stmt.span);
                break;
            }
            // Expression types that can have our node at their start position
            AstKind::CallExpression(_)
            | AstKind::ComputedMemberExpression(_)
            | AstKind::StaticMemberExpression(_)
            | AstKind::PrivateFieldExpression(_)
            | AstKind::ChainExpression(_)
            | AstKind::TaggedTemplateExpression(_)
            | AstKind::SequenceExpression(_)
            | AstKind::AssignmentExpression(_)
            | AstKind::LogicalExpression(_)
            | AstKind::BinaryExpression(_)
            | AstKind::ConditionalExpression(_)
            | AstKind::AwaitExpression(_)
            | AstKind::ParenthesizedExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSTypeAssertion(_)
            | AstKind::TSInstantiationExpression(_) => {}
            _ => return false,
        }
    }

    let Some(expr_stmt_span) = expr_stmt_span else {
        return false;
    };

    // Node must be at the start of the statement for ASI hazard to apply
    if node_span.start != expr_stmt_span.start {
        return false;
    }

    if expr_stmt_span.start == 0 {
        return false;
    }

    let source_text = ctx.source_text();
    let last_char = find_last_meaningful_char(source_text, expr_stmt_span.start, ctx);

    let Some(last_char) = last_char else {
        return false;
    };

    // Characters that could cause ASI issues when followed by `[`, `(`, `/`, etc.
    matches!(last_char, ')' | ']' | '}' | '"' | '\'' | '`' | '+' | '-' | '/' | '.')
        || last_char.is_alphanumeric()
        || last_char == '_'
        || last_char == '$'
}

#[inline]
#[expect(clippy::cast_possible_wrap)]
fn is_utf8_char_boundary(b: u8) -> bool {
    (b as i8) >= -0x40
}

/// Find the last meaningful (non-whitespace, non-comment) character before `end_pos`.
fn find_last_meaningful_char(source_text: &str, end_pos: u32, ctx: &LintContext) -> Option<char> {
    let bytes = source_text.as_bytes();
    let comments = ctx.semantic().comments();

    let mut comment_idx = comments.partition_point(|c| c.span.start < end_pos);
    let mut current_comment_end: u32 = 0;
    let mut i = end_pos;

    // Handle case where end_pos is inside a comment
    if comment_idx > 0 {
        let prev_comment = &comments[comment_idx - 1];
        if end_pos <= prev_comment.span.end {
            i = prev_comment.span.start;
            comment_idx -= 1;
            if comment_idx > 0 {
                current_comment_end = comments[comment_idx - 1].span.end;
            }
        }
    }

    while i > 0 {
        if i <= current_comment_end && comment_idx > 0 {
            comment_idx -= 1;
            current_comment_end = comments[comment_idx].span.start;
            i = current_comment_end;
            continue;
        }

        i -= 1;

        let byte = bytes[i as usize];

        if byte.is_ascii_whitespace() {
            continue;
        }

        // Check if we're entering a comment from the end
        if comment_idx > 0 {
            let comment = &comments[comment_idx - 1];
            if i >= comment.span.start && i < comment.span.end {
                i = comment.span.start;
                comment_idx -= 1;
                if comment_idx > 0 {
                    current_comment_end = comments[comment_idx - 1].span.end;
                }
                continue;
            }
        }

        if byte.is_ascii() {
            return Some(byte as char);
        }

        // Multi-byte UTF-8: find the start byte (max 4 bytes per char)
        let i_usize = i as usize;
        let char_start = if is_utf8_char_boundary(bytes[i_usize.saturating_sub(1)]) {
            i_usize - 1
        } else if is_utf8_char_boundary(bytes[i_usize.saturating_sub(2)]) {
            i_usize - 2
        } else {
            i_usize - 3
        };

        let c = source_text[char_start..].chars().next().unwrap();

        // Skip irregular whitespace (NBSP, ZWNBSP, etc.)
        if is_irregular_whitespace(c) {
            #[expect(clippy::cast_possible_truncation)]
            {
                i = char_start as u32;
            }
            continue;
        }

        return Some(c);
    }

    None
}
