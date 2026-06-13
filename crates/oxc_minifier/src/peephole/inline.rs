use crate::generated::ancestor::Ancestor;
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::{
    ConstantEvaluation, ConstantValue, str_has_lone_surrogate_encoding,
};
use oxc_span::GetSpan;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn init_symbol_value(decl: &VariableDeclarator<'a>, ctx: &mut TraverseCtx<'a>) {
        let BindingPattern::BindingIdentifier(ident) = &decl.id else { return };
        let Some(symbol_id) = ident.symbol_id.get() else { return };
        let value = if decl.kind.is_var() || Self::is_for_statement_init(ctx) {
            // - Skip constant value inlining for `var` declarations, due to TDZ problems.
            // - Set None for for statement initializers as the value of these are set by the for statement.
            None
        } else {
            decl.init.as_ref().map_or(Some(ConstantValue::Undefined), |e| e.evaluate_value(ctx))
        };
        let is_fresh_value = decl.init.as_ref().is_some_and(Self::is_fresh_value_expression);
        ctx.init_value(symbol_id, value, is_fresh_value);
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
        ctx.init_value(symbol_id, None, true);
    }

    /// Initialize symbol value for class declarations.
    /// Class declarations create fresh values, but classes with static setters
    /// are not considered fresh because property writes trigger setter side effects.
    pub fn init_class_declaration_symbol_value(class: &Class<'a>, ctx: &mut TraverseCtx<'a>) {
        let Some(id) = &class.id else { return };
        let Some(symbol_id) = id.symbol_id.get() else { return };
        let is_fresh = !Self::class_may_have_property_side_effects(class);
        ctx.init_value(symbol_id, None, is_fresh);
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
        // `ConstantValue::String` holds the encoded bytes but not the flag, so materializing
        // a lone-surrogate initializer via `value_to_expr` would emit a flagless literal. Only
        // the single-use branch below can reach a flagged string: the multi-reference branch is
        // gated on `s.len() <= 3`, well under the 7-byte minimum encoded form.
        if let ConstantValue::String(s) = cv
            && str_has_lone_surrogate_encoding(s)
        {
            return;
        }
        if symbol_value.read_references_count == 1
            || match cv {
                ConstantValue::Number(n) => n.fract() == 0.0 && *n >= -99.0 && *n <= 999.0,
                ConstantValue::BigInt(_) => false,
                ConstantValue::String(s) => s.len() <= 3,
                ConstantValue::Boolean(_) | ConstantValue::Undefined | ConstantValue::Null => true,
            }
        {
            *expr = ctx.value_to_expr(expr.span(), cv.clone());
            ctx.state.changed = true;
        }
    }
}
