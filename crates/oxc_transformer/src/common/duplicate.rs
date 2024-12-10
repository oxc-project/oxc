//! Utilities to duplicate expressions.

use std::{
    mem::{ManuallyDrop, MaybeUninit},
    ptr,
};

use oxc_ast::ast::{AssignmentOperator, Expression};
use oxc_span::SPAN;
use oxc_syntax::reference::ReferenceFlags;
use oxc_traverse::{BoundIdentifier, TraverseCtx};

use crate::TransformCtx;

impl<'a> TransformCtx<'a> {
    /// Duplicate expression to be used twice.
    ///
    /// If `expr` may have side effects, create a temp var `_expr` and assign to it.
    ///
    /// * `this` -> `this`, `this`
    /// * Bound identifier `foo` -> `foo`, `foo`
    /// * Unbound identifier `foo` -> `_foo = foo`, `_foo`
    /// * Anything else `foo()` -> `_foo = foo()`, `_foo`
    ///
    /// Returns 2 `Expression`s. The first may be an `AssignmentExpression`,
    /// and must be inserted into output first.
    pub(crate) fn duplicate_expression(
        &self,
        expr: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (Expression<'a>, Expression<'a>) {
        let (maybe_assignment, references) = self.duplicate_expression_multiple::<1>(expr, ctx);
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
    /// Returns 3 `Expression`s. The first may be an `AssignmentExpression`,
    /// and must be inserted into output first.
    #[expect(clippy::similar_names)]
    pub(crate) fn duplicate_expression_twice(
        &self,
        expr: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (Expression<'a>, Expression<'a>, Expression<'a>) {
        let (maybe_assignment, references) = self.duplicate_expression_multiple::<2>(expr, ctx);
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
    /// Returns `N + 1` x `Expression`s. The first may be an `AssignmentExpression`,
    /// and must be inserted into output first.
    pub(crate) fn duplicate_expression_multiple<const N: usize>(
        &self,
        expr: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (Expression<'a>, [Expression<'a>; N]) {
        // TODO: Handle if in a function's params
        let temp_var_binding = match &expr {
            Expression::Identifier(ident) => {
                let reference = ctx.symbols_mut().get_reference_mut(ident.reference_id());
                if let Some(symbol_id) = reference.symbol_id() {
                    // Reading bound identifier cannot have side effects, so no need for temp var
                    let binding = BoundIdentifier::new(ident.name.clone(), symbol_id);
                    let references =
                        create_array(|| binding.create_spanned_read_expression(ident.span, ctx));
                    return (expr, references);
                }

                // Previously `x += 1` (`x` read + write), but moving to `_x = x` (`x` read only)
                *reference.flags_mut() = ReferenceFlags::Read;

                self.var_declarations.create_uid_var(&ident.name, ctx)
            }
            Expression::ThisExpression(this) => {
                // Reading `this` cannot have side effects, so no need for temp var
                let references = create_array(|| ctx.ast.expression_this(this.span));
                return (expr, references);
            }
            _ => self.var_declarations.create_uid_var_based_on_node(&expr, ctx),
        };

        let assignment = ctx.ast.expression_assignment(
            SPAN,
            AssignmentOperator::Assign,
            temp_var_binding.create_target(ReferenceFlags::Write, ctx),
            expr,
        );

        let references = create_array(|| temp_var_binding.create_read_expression(ctx));

        (assignment, references)
    }
}

/// Create array of length `N`, with each item initialized with provided function `init`.
///
/// Implementation based on:
/// * <https://github.com/rust-lang/rust/issues/62875#issuecomment-513834029>
/// * <https://github.com/rust-lang/rust/issues/61956>
//
// `#[inline]` so compiler can inline `init()`, and it may unroll the loop if `init` is simple enough.
#[inline]
fn create_array<const N: usize, T, I: FnMut() -> T>(mut init: I) -> [T; N] {
    let mut array: [MaybeUninit<T>; N] = [const { MaybeUninit::uninit() }; N];
    for elem in &mut array {
        elem.write(init());
    }
    // Wrapping in `ManuallyDrop` should not be necessary because `MaybeUninit` does not impl `Drop`,
    // but do it anyway just to make sure, as it's mentioned in issues above.
    let mut array = ManuallyDrop::new(array);
    // SAFETY: All elements of array are initialized.
    // `[MaybeUninit<T>; N]` and `[T; N]` have same layout.
    unsafe { ptr::from_mut(&mut array).cast::<[T; N]>().read() }
}
