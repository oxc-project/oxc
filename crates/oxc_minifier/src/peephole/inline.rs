use crate::generated::ancestor::Ancestor;
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::{ConstantEvaluation, ConstantValue};
use oxc_span::GetSpan;
use oxc_syntax::{scope::ScopeId, symbol::SymbolId};

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn init_symbol_value(decl: &VariableDeclarator<'a>, ctx: &mut TraverseCtx<'a>) {
        let BindingPattern::BindingIdentifier(ident) = &decl.id else { return };
        let Some(symbol_id) = ident.symbol_id.get() else { return };
        // Evaluate the initializer's constant once; reuse it for the value-context
        // constant and the boolean-falsy fact below. `None` for a non-constant or
        // absent initializer.
        let init_constant = decl.init.as_ref().and_then(|e| e.evaluate_value(ctx));
        // Whether the initializer is an explicit falsy constant (not the implicit
        // `undefined` of `var x;`, which `init_constant` leaves `None`). Fed to
        // `init_value`, which turns it into the `boolean_falsy` fact (see
        // `SymbolValue::boolean_falsy`).
        let falsy_init = init_constant.as_ref().is_some_and(Self::is_falsy_constant);
        let value = if Self::is_for_statement_init(ctx) {
            // for-statement initializers have their value set by the for statement itself.
            None
        } else if decl.kind.is_var() && !Self::is_hoisted_var_inlineable(decl, symbol_id, ctx) {
            // `var` is hoisted: reads before the initializer line see `undefined`.
            // Skip unless the safety predicate proves no such read exists.
            None
        } else {
            // No initializer hoists to `undefined`; otherwise reuse the constant.
            decl.init.as_ref().map_or(Some(ConstantValue::Undefined), |_| init_constant)
        };
        let is_fresh_value = decl.init.as_ref().is_some_and(Self::is_fresh_value_expression);
        ctx.init_value(symbol_id, value, is_fresh_value, falsy_init);
    }

    /// A `ConstantValue` that coerces to `false` (`false`, `0`/`-0`/`NaN`, `""`,
    /// `null`, `undefined`). BigInt is skipped conservatively.
    fn is_falsy_constant(cv: &ConstantValue<'a>) -> bool {
        match cv {
            ConstantValue::Boolean(b) => !b,
            ConstantValue::Number(n) => n.is_nan() || *n == 0.0,
            ConstantValue::String(s) => s.as_ref().is_empty(),
            ConstantValue::Null | ConstantValue::Undefined => true,
            ConstantValue::BigInt(_) => false,
        }
    }

    /// Predicate for inlining a hoisted `var x = <literal>;`. True when no read
    /// can observe `x` as its hoisted `undefined`:
    /// - the declarator sits at the current body's top scope and that body is
    ///   still in its declarative prelude;
    /// - it has an initializer (uninitialized `var foo;` would inline to
    ///   `undefined`, which churns existing tests for marginal benefit);
    /// - script-mode top-level vars are excluded (they alias the global object);
    /// - at program scope, if the module loads any other module (`import`,
    ///   `export … from`, `export * from`), skip: a cyclic importer can call
    ///   into our exports and observe any var our exported functions/classes
    ///   close over, regardless of export status;
    /// - every read sits inside a nested function/arrow body (the gap
    ///   `substitute_single_use_symbol` can't reach). Multiple such reads are
    ///   fine: the prelude check proves none observes the hoisted `undefined`,
    ///   so the value is constant at every read, and the small-value rule /
    ///   write-count guard in `inline_identifier_reference` decide whether each
    ///   read actually folds (e.g. a write-once falsy flag read by `if (flag)`
    ///   throughout — the Svelte/Vue `hydrating` shape, #14001).
    ///
    /// Limitation: the constant is recorded here at the declarator's exit, so a
    /// reader in a function declared *before* the var in source order has
    /// already been visited and won't be inlined. Safe but suboptimal; the
    /// common "flag declared at the top" pattern is unaffected.
    fn is_hoisted_var_inlineable(
        decl: &VariableDeclarator<'a>,
        symbol_id: SymbolId,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        if decl.init.is_none() || Self::keep_top_level_var_in_script_mode(ctx) {
            return false;
        }
        // `body_unsafe` is set by a preceding non-declarative statement, and the
        // program root additionally starts unsafe when the module has loaders
        // (see `enter_program`) — so this one check covers the cyclic-import gate.
        let &(body_scope, body_unsafe) = ctx.state.body_unsafe_stack.last();
        if body_unsafe || ctx.current_scope_id() != body_scope {
            return false;
        }
        // At least one read, and every read crosses a function boundary.
        let mut reads = ctx.scoping().get_resolved_references(symbol_id).filter(|r| r.is_read());
        let Some(first) = reads.next() else { return false };
        if !Self::read_crosses_function_boundary(first.scope_id(), body_scope, ctx) {
            return false;
        }
        reads.all(|read| Self::read_crosses_function_boundary(read.scope_id(), body_scope, ctx))
    }

    /// True if the scope chain from `read_scope` to `body_scope` (exclusive of
    /// `body_scope`) crosses any `Function` scope. `substitute_single_use_symbol`
    /// only walks adjacent statements within the same call frame, so this is
    /// the gap our path fills.
    fn read_crosses_function_boundary(
        read_scope: ScopeId,
        body_scope: ScopeId,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        let scoping = ctx.scoping();
        scoping
            .scope_ancestors(read_scope)
            .take_while(|&s| s != body_scope)
            .any(|s| scoping.scope_flags(s).is_function())
    }

    /// Check if an expression creates a fresh value that cannot alias another binding
    /// and has no setters/getters that could trigger side effects on property writes.
    fn is_fresh_value_expression(expr: &Expression<'a>) -> bool {
        match expr {
            Expression::ArrayExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_) => true,
            Expression::ObjectExpression(obj) => {
                // Object literals with setter/getter properties are not safe to treat as fresh.
                // Setters trigger side effects on property writes.
                // Getter-only properties throw TypeError in strict mode on write.
                // Also check property values for nested setters/getters.
                !obj.properties.iter().any(|prop| {
                    matches!(
                        prop,
                        ObjectPropertyKind::ObjectProperty(p)
                            if matches!(p.kind, PropertyKind::Set | PropertyKind::Get)
                                || Self::expression_has_setter_or_getter(&p.value)
                                // `{ __proto__: ... }` sets the prototype chain and could
                                // install setters that make property writes side-effectful.
                                || (p.kind == PropertyKind::Init
                                    && !p.computed
                                    && p.key.is_specific_static_name("__proto__"))
                    )
                })
            }
            Expression::ClassExpression(class) => {
                !Self::class_may_have_property_side_effects(class)
            }
            _ => false,
        }
    }

    /// Check if a class may have side effects on property writes.
    /// Returns `true` if the class has static setters, static accessor properties,
    /// static property definitions with values, or an `extends` clause.
    /// Following SWC's approach: any class with static property definitions
    /// is not considered fresh, because the static initializer runs during
    /// class creation and defines the property via `[[DefineOwnProperty]]`.
    fn class_may_have_property_side_effects(class: &Class<'a>) -> bool {
        // Classes with `extends` may inherit static setters from the parent.
        // We can't statically determine the parent's static setters,
        // so conservatively mark as non-fresh.
        if class.super_class.is_some() {
            return true;
        }
        class.body.body.iter().any(|element| match element {
            ClassElement::MethodDefinition(method) => {
                method.r#static && method.kind == MethodDefinitionKind::Set
            }
            // `static accessor foo` auto-generates a getter+setter pair
            ClassElement::AccessorProperty(prop) => prop.r#static,
            // Any static property definition with a value prevents fresh marking.
            // The value is evaluated during class creation and could interact with
            // property writes in unexpected ways (e.g. nested setters, proxies).
            ClassElement::PropertyDefinition(prop) => prop.r#static && prop.value.is_some(),
            _ => false,
        })
    }

    /// Check if an expression contains setter or getter definitions (recursively).
    fn expression_has_setter_or_getter(expr: &Expression<'a>) -> bool {
        match expr {
            Expression::ObjectExpression(obj) => obj.properties.iter().any(|prop| {
                matches!(
                    prop,
                    ObjectPropertyKind::ObjectProperty(p)
                        if matches!(p.kind, PropertyKind::Set | PropertyKind::Get)
                            || Self::expression_has_setter_or_getter(&p.value)
                )
            }),
            Expression::ClassExpression(class) => Self::class_may_have_property_side_effects(class),
            _ => false,
        }
    }

    /// Initialize symbol value for function declarations.
    /// Function declarations always create fresh values (cannot alias another binding).
    pub fn init_function_declaration_symbol_value(
        id: Option<&BindingIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Some(id) = id else { return };
        let Some(symbol_id) = id.symbol_id.get() else { return };
        ctx.init_value(symbol_id, None, true, false);
    }

    /// Initialize symbol value for class declarations.
    /// Class declarations create fresh values, but classes with static setters
    /// are not considered fresh because property writes trigger setter side effects.
    pub fn init_class_declaration_symbol_value(class: &Class<'a>, ctx: &mut TraverseCtx<'a>) {
        let Some(id) = &class.id else { return };
        let Some(symbol_id) = id.symbol_id.get() else { return };
        let is_fresh = !Self::class_may_have_property_side_effects(class);
        ctx.init_value(symbol_id, None, is_fresh, false);
    }

    fn is_for_statement_init(ctx: &TraverseCtx<'a>) -> bool {
        ctx.ancestors().nth(1).is_some_and(Ancestor::is_parent_of_for_statement_left)
    }

    pub fn inline_identifier_reference(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::Identifier(ident) = expr else { return };
        let reference_id = ident.reference_id();
        let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else { return };
        let Some(symbol_value) = ctx.state.symbol_values.get_symbol_value(symbol_id) else {
            return;
        };
        // Skip if there are write references.
        if symbol_value.write_references_count > 0 {
            return;
        }
        let Some(cv) = &symbol_value.initialized_constant else { return };
        if symbol_value.read_references_count == 1
            || match cv {
                ConstantValue::Number(n) => n.fract() == 0.0 && *n >= -99.0 && *n <= 999.0,
                ConstantValue::BigInt(_) => false,
                ConstantValue::String(s) => s.len() <= 3,
                ConstantValue::Boolean(_) | ConstantValue::Undefined | ConstantValue::Null => true,
            }
        {
            let new_expr = ctx.value_to_expr(expr.span(), cv.clone());
            ctx.replace_expression(expr, new_expr);
        }
    }
}
