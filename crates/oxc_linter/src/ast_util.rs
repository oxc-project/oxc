use oxc_ast::{
    ast::{BindingIdentifier, *},
    AstKind,
};
use oxc_ecmascript::ToBoolean;
use oxc_semantic::{AstNode, IsGlobalReference, NodeId, ReferenceId, Semantic, SymbolId};
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator};

use crate::LintContext;

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
        if let Expression::Identifier(ident) = &self.callee {
            if ident.name == "Boolean"
                && self
                    .arguments
                    .iter()
                    .next()
                    .map_or(true, |first| first.is_constant(true, semantic))
            {
                return semantic.is_reference_to_global_variable(ident);
            }
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
        current_node = semantic.nodes().parent_node(current_node.id())?;
    }
}

/// Returns if `arg` is the `n`th (0-indexed) argument of `call`.
pub fn is_nth_argument<'a>(call: &CallExpression<'a>, arg: &Argument<'a>, n: usize) -> bool {
    let nth = &call.arguments[n];
    nth.span() == arg.span()
}

/// Jump to the outer most of chained parentheses if any
pub fn outermost_paren<'a, 'b>(
    node: &'b AstNode<'a>,
    semantic: &'b Semantic<'a>,
) -> &'b AstNode<'a> {
    let mut node = node;

    loop {
        if let Some(parent) = semantic.nodes().parent_node(node.id()) {
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
    semantic: &'b Semantic<'a>,
) -> Option<&'b AstNode<'a>> {
    semantic
        .nodes()
        .ancestors(node.id())
        .skip(1)
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
        .skip(1)
        .filter(|parent| !matches!(parent.kind(), AstKind::ParenthesizedExpression(_)))
        .nth(n)
}

/// Iterate over parents of `node`, skipping nodes that are also ignored by
/// [`Expression::get_inner_expression`].
pub fn iter_outer_expressions<'a, 's>(
    semantic: &'s Semantic<'a>,
    node_id: NodeId,
) -> impl Iterator<Item = AstKind<'a>> + 's {
    semantic.nodes().ancestor_kinds(node_id).skip(1).filter(|parent| {
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
    let symbol_table = semantic.symbols();
    Some(semantic.nodes().get_node(symbol_table.get_declaration(symbol_id)))
}

pub fn get_declaration_from_reference_id<'a, 'b>(
    reference_id: ReferenceId,
    semantic: &'b Semantic<'a>,
) -> Option<&'b AstNode<'a>> {
    let symbol_table = semantic.symbols();
    let symbol_id = symbol_table.get_reference(reference_id).symbol_id()?;
    Some(semantic.nodes().get_node(symbol_table.get_declaration(symbol_id)))
}

pub fn get_symbol_id_of_variable(
    ident: &IdentifierReference,
    semantic: &Semantic<'_>,
) -> Option<SymbolId> {
    semantic.symbols().get_reference(ident.reference_id()).symbol_id()
}

pub fn extract_regex_flags<'a>(
    args: &'a oxc_allocator::Vec<'a, Argument<'a>>,
) -> Option<RegExpFlags> {
    if args.len() <= 1 {
        return None;
    }
    let flag_arg = match &args[1] {
        Argument::StringLiteral(flag_arg) => flag_arg.value,
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

    let callee_without_parentheses = call_expr.callee.without_parentheses();
    let member_expr = match callee_without_parentheses {
        match_member_expression!(Expression) => callee_without_parentheses.to_member_expression(),
        Expression::ChainExpression(chain) => match chain.expression.member_expression() {
            Some(e) => e,
            None => return false,
        },
        _ => return false,
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

pub fn is_global_require_call(call_expr: &CallExpression, ctx: &Semantic) -> bool {
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
                return could_be_error(ctx, &expr.right);
            }

            if matches!(
                expr.operator,
                AssignmentOperator::LogicalOr | AssignmentOperator::LogicalNullish
            ) {
                return expr.left.get_expression().map_or(true, |expr| could_be_error(ctx, expr))
                    || could_be_error(ctx, &expr.right);
            }

            false
        }
        Expression::SequenceExpression(expr) => {
            expr.expressions.last().is_some_and(|expr| could_be_error(ctx, expr))
        }
        Expression::LogicalExpression(expr) => {
            if matches!(expr.operator, LogicalOperator::And) {
                return could_be_error(ctx, &expr.right);
            }

            could_be_error(ctx, &expr.left) || could_be_error(ctx, &expr.right)
        }
        Expression::ConditionalExpression(expr) => {
            could_be_error(ctx, &expr.consequent) || could_be_error(ctx, &expr.alternate)
        }
        Expression::Identifier(ident) => {
            let reference = ctx.symbols().get_reference(ident.reference_id());
            let Some(symbol_id) = reference.symbol_id() else {
                return true;
            };
            let decl = ctx.nodes().get_node(ctx.symbols().get_declaration(symbol_id));
            match decl.kind() {
                AstKind::VariableDeclarator(decl) => {
                    if let Some(init) = &decl.init {
                        could_be_error(ctx, init)
                    } else {
                        // TODO: warn about throwing undefined
                        false
                    }
                }
                AstKind::Function(_)
                | AstKind::Class(_)
                | AstKind::TSModuleDeclaration(_)
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
