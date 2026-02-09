//! Utilities to duplicate expressions.

use std::array;

use oxc_allocator::CloneIn;
use oxc_ast::ast::{AssignmentOperator, Expression};
use oxc_span::SPAN;
use oxc_syntax::reference::ReferenceFlags;
use oxc_traverse::BoundIdentifier;

use crate::context::TraverseCtx;

use super::var_declarations::VarDeclarationsStore;

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
    // TODO: Handle if in a function's params
    let temp_var_binding = match &expr {
        Expression::Identifier(ident) => {
            let reference_id = ident.reference_id();
            let reference = ctx.scoping().get_reference(reference_id);
            if let Some(symbol_id) = reference.symbol_id()
                && (!mutated_symbol_needs_temp_var || !ctx.scoping().symbol_is_mutated(symbol_id))
            {
                // Reading bound identifier cannot have side effects, so no need for temp var
                let binding = BoundIdentifier::new(ident.name, symbol_id);
                let references =
                    array::from_fn(|_| binding.create_spanned_read_expression(ident.span, ctx));
                return (expr, references);
            }

            // Previously `x += 1` (`x` read + write), but moving to `_x = x` (`x` read only)
            let reference = ctx.scoping_mut().get_reference_mut(reference_id);
            *reference.flags_mut() = ReferenceFlags::Read;

            VarDeclarationsStore::create_uid_var(&ident.name, ctx)
        }
        // Reading any of these cannot have side effects, so no need for temp var
        Expression::ThisExpression(_)
        | Expression::Super(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::StringLiteral(_) => {
            let references = array::from_fn(|_| expr.clone_in(ctx.ast.allocator));
            return (expr, references);
        }
        // Template literal cannot have side effects if it has no expressions.
        // If it *does* have expressions, but they're all literals, then also cannot have side effects,
        // but don't bother checking for that as it shouldn't occur in real world code.
        // Why would you write "`x${9}z`" when you can just write "`x9z`"?
        // Note: "`x${foo}`" *can* have side effects if `foo` is an object with a `toString` method.
        Expression::TemplateLiteral(lit) if lit.expressions.is_empty() => {
            let references = array::from_fn(|_| {
                ctx.ast.expression_template_literal(
                    lit.span,
                    ctx.ast.vec_from_iter(lit.quasis.iter().cloned()),
                    ctx.ast.vec(),
                )
            });
            return (expr, references);
        }
        // Anything else requires temp var
        _ => VarDeclarationsStore::create_uid_var_based_on_node(&expr, ctx),
    };

    let assignment = ctx.ast.expression_assignment(
        SPAN,
        AssignmentOperator::Assign,
        temp_var_binding.create_target(ReferenceFlags::Write, ctx),
        expr,
    );

    let references = array::from_fn(|_| temp_var_binding.create_read_expression(ctx));

    (assignment, references)
}
