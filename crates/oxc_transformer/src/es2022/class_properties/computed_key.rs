//! ES2022: Class Properties
//! Transform of class property/method computed keys.

use oxc_ast::ast::*;
use oxc_syntax::symbol::SymbolFlags;
use oxc_traverse::TraverseCtx;

use super::{utils::create_assignment, ClassProperties};

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    /// Substitute temp var for method computed key.
    /// `class C { [x()]() {} }` -> `let _x; _x = x(); class C { [_x]() {} }`
    /// This transform is only required if class has properties or a static block.
    pub(super) fn substitute_temp_var_for_method_computed_key(
        &mut self,
        method: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // TODO: Don't alter numeric literal key e.g. `class C { 123() {} }`
        // TODO: Don't re-create key if it doesn't need to be altered
        let Some(key) = method.key.as_expression_mut() else { return };

        // TODO: Don't alter key if it's provable evaluating it has no side effects.
        // TODO(improve-on-babel): It's unnecessary to create temp vars for method keys unless:
        // 1. Properties also have computed keys.
        // 2. Some of those properties' computed keys have side effects and require temp vars.
        // 3. At least one property satisfying the above is after this method,
        //    or class contains a static block which is being transformed
        //    (static blocks are always evaluated after computed keys, regardless of order)
        method.key = PropertyKey::from(self.create_computed_key_temp_var(key, ctx));
    }

    /// Convert computed property/method key to a temp var.
    ///
    /// Transformation is:
    /// * Class declaration:
    ///   `class C { [x()] = 1; }` -> `let _x; _x = x(); class C { constructor() { this[_x] = 1; } }`
    /// * Class expression:
    ///   `C = class { [x()] = 1; }` -> `let _x; C = (_x = x(), class C { constructor() { this[_x] = 1; } })`
    ///
    /// This function:
    /// * Creates the `let _x;` statement and inserts it.
    /// * Creates the `_x = x()` assignments.
    /// * Inserts assignments before class declaration, or adds to `state` if class expression.
    /// * Returns `_x`.
    pub(super) fn create_computed_key_temp_var(
        &mut self,
        key: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let key = ctx.ast.move_expression(key);

        // Bound vars and literals do not need temp var - return unchanged.
        // e.g. `let x = 'x'; class C { [x] = 1; }` or `class C { ['x'] = 1; }`
        //
        // `this` does not have side effects, but it needs a temp var anyway, because `this` in computed
        // key and `this` within class constructor resolve to different `this` bindings.
        // So we need to create a temp var outside of the class to get the correct `this`.
        // `class C { [this] = 1; }`
        // -> `let _this; _this = this; class C { constructor() { this[_this] = 1; } }`
        //
        // TODO(improve-on-babel): Can avoid the temp var if key is for a static prop/method,
        // as in that case the usage of `this` stays outside the class.
        //
        // TODO: Do fuller analysis to detect expressions which cannot have side effects e.g. `'x' + 'y'`.
        let cannot_have_side_effects = match &key {
            Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::StringLiteral(_) => true,
            Expression::Identifier(ident) => {
                // Cannot have side effects if is bound.
                // Additional check that the var is not mutated is required for cases like
                // `let x = 1; class { [x] = 1; [++x] = 2; }`
                // `++x` is hoisted to before class in output, so `x` in 1st key would get the wrong
                // value unless it's hoisted out too.
                // TODO: Add an exec test for this odd case.
                // TODO(improve-on-babel): That case is rare.
                // Test for it in first pass over class elements, and avoid temp vars where possible.
                match ctx.symbols().get_reference(ident.reference_id()).symbol_id() {
                    Some(symbol_id) => {
                        // TODO: Use `SymbolTable::symbol_is_mutated`
                        ctx.symbols().get_flags(symbol_id).contains(SymbolFlags::ConstVariable)
                            || ctx
                                .symbols()
                                .get_resolved_references(symbol_id)
                                .all(|reference| !reference.is_write())
                    }
                    None => false,
                }
            }
            _ => false,
        };
        if cannot_have_side_effects {
            return key;
        }

        // We entered transform via `enter_expression` or `enter_statement`,
        // so `ctx.current_scope_id()` is the scope outside the class
        let parent_scope_id = ctx.current_scope_id();
        // TODO: Handle if is a class expression defined in a function's params.
        let binding =
            ctx.generate_uid_based_on_node(&key, parent_scope_id, SymbolFlags::BlockScopedVariable);

        self.ctx.var_declarations.insert_let(&binding, None, ctx);

        let assignment = create_assignment(&binding, key, ctx);
        self.insert_before.push(assignment);

        binding.create_read_expression(ctx)
    }
}
