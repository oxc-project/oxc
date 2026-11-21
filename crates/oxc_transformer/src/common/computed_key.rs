//! Utilities to computed key expressions.

use oxc_ast::ast::{AssignmentTarget, Expression};
use oxc_semantic::SymbolFlags;
use oxc_span::SPAN;
use oxc_syntax::operator::AssignmentOperator;
use oxc_traverse::BoundIdentifier;

use crate::{context::TraverseCtx, utils::ast_builder::create_assignment};

/// Check if temp var is required for `key`.
///
/// `this` does not have side effects, but in this context, it needs a temp var anyway, because `this`
/// in computed key and `this` within class constructor resolve to different `this` bindings.
/// So we need to create a temp var outside of the class to get the correct `this`.
/// `class C { [this] = 1; }`
/// -> `let _this; _this = this; class C { constructor() { this[_this] = 1; } }`
//
// TODO(improve-on-babel): Can avoid the temp var if key is for a static prop/method,
// as in that case the usage of `this` stays outside the class.
pub fn key_needs_temp_var(key: &Expression, ctx: &TraverseCtx) -> bool {
    // TypeScript syntax such as `k as any` is erased, so it doesn't affect the key.
    match key.get_inner_expression() {
        // Literals cannot have side effects.
        // e.g. `let x = 'x'; class C { [x] = 1; }` or `class C { ['x'] = 1; }`.
        Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::StringLiteral(_) => false,
        // Template literal cannot have side effects if it has no expressions.
        // If it *does* have expressions, but they're all literals, then also cannot have side effects,
        // but don't bother checking for that as it shouldn't occur in real world code.
        // Why would you write "`x${9}z`" when you can just write "`x9z`"?
        // Note: "`x${foo}`" *can* have side effects if `foo` is an object with a `toString` method.
        Expression::TemplateLiteral(lit) => !lit.expressions.is_empty(),
        // `IdentifierReference`s can have side effects if is unbound.
        //
        // If var is mutated, it also needs a temp var, because of cases like
        // `let x = 1; class { [x] = 1; [++x] = 2; }`
        // `++x` is hoisted to before class in output, so `x` in 1st key would get the wrong value
        // unless it's hoisted out too.
        //
        // TODO: Add an exec test for this odd case.
        // TODO(improve-on-babel): That case is rare.
        // Test for it in first pass over class elements, and avoid temp vars where possible.
        Expression::Identifier(ident) => {
            match ctx.scoping().get_reference(ident.reference_id()).symbol_id() {
                Some(symbol_id) => ctx.scoping().symbol_is_mutated(symbol_id),
                None => true,
            }
        }
        // Treat any other expression as possibly having side effects e.g. `foo()`.
        // TODO: Do fuller analysis to detect expressions which cannot have side effects.
        // e.g. `"x" + "y"`.
        _ => true,
    }
}

/// Create `let _x;` statement and insert it.
/// Return `_x = x()` assignment, and `_x` identifier referencing same temp var.
///
/// If `key` already assigns a temp var created by another transform (`[_x = x()]`), and nothing
/// else writes it, that temp var is reused instead of creating another.
pub fn create_computed_key_temp_var<'a>(
    key: Expression<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> (/* assignment */ Expression<'a>, /* identifier */ Expression<'a>) {
    if let Some(ident) = assigned_single_write_var(&key, ctx) {
        return (key, ident);
    }

    let outer_scope_id = ctx.current_block_scope_id();
    // TODO: Handle if is a class expression defined in a function's params.
    let binding =
        ctx.generate_uid_based_on_node(&key, outer_scope_id, SymbolFlags::BlockScopedVariable);

    ctx.state.var_declarations.insert_let(&binding, None, &ctx.ast);

    let assignment = create_assignment(&binding, key, SPAN, ctx);
    let ident = binding.create_read_expression(ctx);

    (assignment, ident)
}

/// If `key` is `_x = ...` created by a transform, and that assignment is the only write to the
/// temp var `_x`, return a read of `_x`. `_x` then always holds the key's value after the key is
/// evaluated. `_x` must be a `let` in the block a new temp var would go in, so that each
/// evaluation of the class, such as in a loop, keeps its own key. A variable written in the source
/// is not reused: it is shared, for example between classes created by the same function.
fn assigned_single_write_var<'a>(
    key: &Expression<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Option<Expression<'a>> {
    let Expression::AssignmentExpression(assignment) = key else { return None };
    if assignment.operator != AssignmentOperator::Assign || !assignment.span.is_empty() {
        return None;
    }
    let AssignmentTarget::AssignmentTargetIdentifier(target) = &assignment.left else {
        return None;
    };
    let symbol_id = ctx.scoping().get_reference(target.reference_id()).symbol_id()?;
    let scoping = ctx.scoping();
    if !scoping.symbol_span(symbol_id).is_empty()
        || !scoping.symbol_flags(symbol_id).contains(SymbolFlags::BlockScopedVariable)
        || scoping.symbol_scope_id(symbol_id) != ctx.current_block_scope_id()
    {
        return None;
    }
    let writes = ctx.scoping().get_resolved_references(symbol_id).filter(|r| r.is_write()).count();
    if writes != 1 {
        return None;
    }
    Some(BoundIdentifier::new(target.name, symbol_id).create_read_expression(ctx))
}
