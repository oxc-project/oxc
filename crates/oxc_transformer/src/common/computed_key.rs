//! Utilities to computed key expressions.

use std::cell::Cell;

use oxc_ast::ast::{Class, Expression, TSConditionalType};
use oxc_ast_visit::Visit;
use oxc_semantic::{ScopeFlags, ScopeId, SymbolFlags};
use oxc_span::SPAN;

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
    match key {
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
pub fn create_computed_key_temp_var<'a>(
    key: Expression<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> (/* assignment */ Expression<'a>, /* identifier */ Expression<'a>) {
    let outer_scope_id = ctx.current_block_scope_id();
    // TODO: Handle if is a class expression defined in a function's params.
    let binding =
        ctx.generate_uid_based_on_node(&key, outer_scope_id, SymbolFlags::BlockScopedVariable);

    ctx.state.var_declarations.insert_let(&binding, None, &ctx.ast);

    reparent_computed_key_scopes(&key, outer_scope_id, ctx);
    let assignment = create_assignment(&binding, key, SPAN, ctx);
    let ident = binding.create_read_expression(ctx);

    (assignment, ident)
}

pub fn reparent_computed_key_scopes<'a>(
    key: &Expression<'a>,
    parent_scope_id: ScopeId,
    ctx: &mut TraverseCtx<'a>,
) {
    let make_sloppy_mode = !ctx.scoping().scope_flags(parent_scope_id).is_strict_mode();
    ComputedKeyScopeVisitor::new(parent_scope_id, make_sloppy_mode, ctx).visit_expression(key);
}

struct ComputedKeyScopeVisitor<'a, 'ctx> {
    parent_scope_id: ScopeId,
    make_sloppy_mode: bool,
    scope_depth: u32,
    strict_scope_depth: u32,
    ctx: &'ctx mut TraverseCtx<'a>,
}

impl<'a, 'ctx> ComputedKeyScopeVisitor<'a, 'ctx> {
    fn new(
        parent_scope_id: ScopeId,
        make_sloppy_mode: bool,
        ctx: &'ctx mut TraverseCtx<'a>,
    ) -> Self {
        Self { parent_scope_id, make_sloppy_mode, scope_depth: 0, strict_scope_depth: 0, ctx }
    }
}

impl<'a> Visit<'a> for ComputedKeyScopeVisitor<'a, '_> {
    #[inline]
    fn enter_scope(&mut self, flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        let scope_id = scope_id.get().unwrap();
        if self.scope_depth == 0 {
            self.ctx.scoping_mut().change_scope_parent_id(scope_id, Some(self.parent_scope_id));
        }
        self.scope_depth += 1;

        if !self.make_sloppy_mode {
            return;
        }
        if self.strict_scope_depth > 0 {
            self.strict_scope_depth += 1;
        } else if flags.is_strict_mode() {
            self.strict_scope_depth = 1;
        } else {
            *self.ctx.scoping_mut().scope_flags_mut(scope_id) -= ScopeFlags::StrictMode;
        }
    }

    #[inline]
    fn leave_scope(&mut self) {
        self.scope_depth -= 1;
        if self.strict_scope_depth > 0 {
            self.strict_scope_depth -= 1;
        }
    }

    #[inline]
    fn visit_class(&mut self, class: &Class<'a>) {
        self.visit_decorators(&class.decorators);
        self.enter_scope(ScopeFlags::StrictMode, &class.scope_id);
        self.leave_scope();
    }

    #[inline]
    fn visit_ts_conditional_type(&mut self, conditional: &TSConditionalType<'a>) {
        self.visit_ts_type(&conditional.check_type);
        self.enter_scope(ScopeFlags::empty(), &conditional.scope_id);
        self.visit_ts_type(&conditional.extends_type);
        self.visit_ts_type(&conditional.true_type);
        self.leave_scope();
        self.visit_ts_type(&conditional.false_type);
    }
}
