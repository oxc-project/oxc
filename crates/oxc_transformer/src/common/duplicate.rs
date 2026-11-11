//! Utilities to duplicate expressions.

use std::array;

use oxc_allocator::{ArenaVec, CloneIn, GetAllocator};
use oxc_ast::ast::{AssignmentOperator, Expression};
use oxc_span::SPAN;
use oxc_syntax::reference::ReferenceFlags;
use oxc_traverse::BoundIdentifier;

use crate::context::TraverseCtx;

use super::var_declarations::VarDeclarationsStore;

/// Whether duplicating an expression requires a temporary variable.
pub fn expression_needs_temp_var(
    expr: &Expression<'_>,
    mutated_symbol_needs_temp_var: bool,
    ctx: &TraverseCtx<'_>,
) -> bool {
    match expr {
        Expression::Identifier(ident) => {
            match ctx.scoping().get_reference(ident.reference_id()).symbol_id() {
                Some(symbol_id) => {
                    mutated_symbol_needs_temp_var && ctx.scoping().symbol_is_mutated(symbol_id)
                }
                None => true,
            }
        }
        Expression::ThisExpression(_)
        | Expression::Super(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::StringLiteral(_) => false,
        Expression::TemplateLiteral(lit) => !lit.expressions.is_empty(),
        _ => true,
    }
}

/// Duplicate expression to be used twice.
///
/// If `expr` may have side effects, create a temp var `_expr` and assign to it.
///
/// * `this` -> `this`, `this`
/// * Bound identifier `foo` -> `foo`, `foo`
/// * Unbound identifier `foo` -> `_foo = foo`, `_foo`
/// * Anything else `foo()` -> `_foo = foo()`, `_foo`
///
/// If `mutated_symbol_needs_temp_var` is `true`, temp var will be created for a bound identifier,
/// if it's mutated (assigned to) anywhere in AST.
///
/// Returns 2 `Expression`s. The first may be an `AssignmentExpression`,
/// and must be inserted into output first.
pub fn duplicate_expression<'a>(
    expr: Expression<'a>,
    mutated_symbol_needs_temp_var: bool,
    ctx: &mut TraverseCtx<'a>,
) -> (Expression<'a>, Expression<'a>) {
    let (maybe_assignment, references) =
        duplicate_expression_multiple::<1>(expr, mutated_symbol_needs_temp_var, ctx);
    let [reference] = references;
    (maybe_assignment, reference)
}

/// Duplicate expression to be used 3 times.
///
/// If `expr` may have side effects, create a temp var `_expr` and assign to it.
///
/// * `this` -> `this`, `this`, `this`
/// * Bound identifier `foo` -> `foo`, `foo`, `foo`
/// * Unbound identifier `foo` -> `_foo = foo`, `_foo`, `_foo`
/// * Anything else `foo()` -> `_foo = foo()`, `_foo`, `_foo`
///
/// If `mutated_symbol_needs_temp_var` is `true`, temp var will be created for a bound identifier,
/// if it's mutated (assigned to) anywhere in AST.
///
/// Returns 3 `Expression`s. The first may be an `AssignmentExpression`,
/// and must be inserted into output first.
pub fn duplicate_expression_twice<'a>(
    expr: Expression<'a>,
    mutated_symbol_needs_temp_var: bool,
    ctx: &mut TraverseCtx<'a>,
) -> (Expression<'a>, Expression<'a>, Expression<'a>) {
    let (maybe_assignment, references) =
        duplicate_expression_multiple::<2>(expr, mutated_symbol_needs_temp_var, ctx);
    let [reference1, reference2] = references;
    (maybe_assignment, reference1, reference2)
}

/// Duplicate expression `N + 1` times.
///
/// If `expr` may have side effects, create a temp var `_expr` and assign to it.
///
/// * `this` -> `this`, [`this`; N]
/// * Bound identifier `foo` -> `foo`, [`foo`; N]
/// * Unbound identifier `foo` -> `_foo = foo`, [`_foo`; N]
/// * Anything else `foo()` -> `_foo = foo()`, [`_foo`; N]
///
/// If `mutated_symbol_needs_temp_var` is `true`, temp var will be created for a bound identifier,
/// if it's mutated (assigned to) anywhere in AST.
///
/// Returns `N + 1` x `Expression`s. The first may be an `AssignmentExpression`,
/// and must be inserted into output first.
pub fn duplicate_expression_multiple<'a, const N: usize>(
    expr: Expression<'a>,
    mutated_symbol_needs_temp_var: bool,
    ctx: &mut TraverseCtx<'a>,
) -> (Expression<'a>, [Expression<'a>; N]) {
    // The caller must provide an enclosing statement list with the required lifetime.
    // In particular, a function's parameter initializers cannot use its body declarations.
    if !expression_needs_temp_var(&expr, mutated_symbol_needs_temp_var, ctx) {
        let references = match &expr {
            Expression::Identifier(ident) => {
                let symbol_id =
                    ctx.scoping().get_reference(ident.reference_id()).symbol_id().unwrap();
                let binding = BoundIdentifier::new(ident.name, symbol_id);
                array::from_fn(|_| binding.create_spanned_read_expression(ident.span, ctx))
            }
            Expression::TemplateLiteral(lit) => array::from_fn(|_| {
                Expression::new_template_literal(
                    lit.span,
                    ArenaVec::from_iter_in(lit.quasis.iter().cloned(), ctx),
                    [],
                    ctx,
                )
            }),
            _ => array::from_fn(|_| expr.clone_in(ctx.allocator())),
        };
        return (expr, references);
    }

    if let Expression::Identifier(ident) = &expr {
        // Previously `x += 1` (`x` read + write), but moving to `_x = x` (`x` read only).
        let reference = ctx.scoping_mut().get_reference_mut(ident.reference_id());
        *reference.flags_mut() = ReferenceFlags::Read;
    }
    let temp_var_binding = VarDeclarationsStore::create_uid_var_based_on_node(&expr, ctx);

    let assignment = Expression::new_assignment_expression(
        SPAN,
        AssignmentOperator::Assign,
        temp_var_binding.create_target(ReferenceFlags::Write, ctx),
        expr,
        ctx,
    );

    let references = array::from_fn(|_| temp_var_binding.create_read_expression(ctx));

    (assignment, references)
}
