//! ES2022: Class Properties
//! Preserve the `name` of anonymous classes.
//!
//! An anonymous class gets its `name` from NamedEvaluation of its surrounding context.
//! e.g. `C` in `const C = class {}`, or `""` when there is no such context, as in `f(class {})`.
//!
//! When a class expression is transformed, it's moved out of that context, and
//! `_Class = class {}` would name it `_Class` instead:
//!
//! ```js
//! const C = class { static x = 1 };
//! // ->
//! const C = (_Class = class C {}, _defineProperty(_Class, "x", 1), _Class);
//! ```
//!
//! To preserve the original name:
//!
//! * If context provides a name, give the class that name: `_Class = class C {}`.
//!   If that would shadow a binding referenced inside the class, perform NamedEvaluation in
//!   an object literal instead: `_Class = { C: class {} }.C`.
//! * Otherwise, prevent NamedEvaluation in the temp var assignment: `_Class = (0, class {})`.

use oxc_allocator::TakeIn;
use oxc_ast::{StaticPropertyName, ast::*};
use oxc_span::SPAN;
use oxc_str::{Ident, JSStr, Str};
use oxc_syntax::{
    identifier::is_identifier_name, keyword::is_reserved_keyword_or_global_object,
    reference::ReferenceId, symbol::SymbolFlags,
};
use oxc_traverse::Ancestor;

use crate::{
    context::TraverseCtx,
    utils::{
        class_name::contextual_class_name,
        decorated_class::{class_body_has_auto_accessors, class_has_decorators},
    },
};

use super::ClassProperties;

impl<'a> ClassProperties<'a> {
    /// Convert anonymous `export default class {}` to a class expression, if transforming it
    /// would require a temp var for the class.
    ///
    /// A class declaration would have to be given a name to be referenced by the static
    /// initializers following it, which would change its `name` from `"default"`.
    /// Class expressions preserve the name instead (see [`Self::preserve_anonymous_class_name`]).
    ///
    /// ```js
    /// export default class { static x = 1 }
    /// // ->
    /// var _Class;
    /// export default (_Class = { default: class {} }.default, _defineProperty(_Class, "x", 1), _Class);
    /// ```
    pub(super) fn transform_anonymous_export_default_class(
        &self,
        stmt: &mut Statement<'a>,
        ctx: &TraverseCtx<'a>,
    ) {
        let Statement::ExportDefaultDeclaration(export) = stmt else { return };
        let ExportDefaultDeclarationKind::ClassDeclaration(class) = &mut export.declaration else {
            return;
        };
        // Leave classes which are lowered by other transforms (decorators) unchanged.
        // Legacy decorators also lower classes with decorated method parameters.
        if class.id.is_some()
            || class.declare
            || class_has_decorators(class)
            || class_body_has_auto_accessors(&class.body)
            || Self::class_has_decorated_parameters(class)
        {
            return;
        }

        // Public fields without an initializer are removed with these options
        let removes_uninitialized_fields = self.remove_class_fields_without_initializer
            && (self.set_public_class_fields || ctx.state.assumptions.set_public_class_fields);
        let needs_temp_var = class.body.body.iter().any(|element| match element {
            ClassElement::PropertyDefinition(prop) => {
                prop.r#static
                    && !prop.declare
                    && !(removes_uninitialized_fields
                        && prop.value.is_none()
                        && !prop.key.is_private_identifier())
            }
            ClassElement::MethodDefinition(method) => {
                method.r#static && method.key.is_private_identifier()
            }
            ClassElement::StaticBlock(_) => self.transform_static_blocks,
            ClassElement::AccessorProperty(_) | ClassElement::TSIndexSignature(_) => false,
        });
        if !needs_temp_var {
            return;
        }

        let mut class = class.take_in(ctx);
        class.r#type = ClassType::ClassExpression;
        export.declaration = ExportDefaultDeclarationKind::ClassExpression(ctx.alloc(class));
    }

    fn class_has_decorated_parameters(class: &Class<'a>) -> bool {
        class.body.body.iter().any(|element| {
            matches!(element, ClassElement::MethodDefinition(method)
                if method.value.params.items.iter().any(|param| !param.decorators.is_empty()))
        })
    }

    /// Preserve the `name` of a class expression which is being moved out of its naming context
    /// (see module docs). `assigned_to_temp` is whether it'll be assigned to the class temp var.
    ///
    /// Must be called before the class is moved, while `ctx` still points at its original position.
    pub(super) fn preserve_anonymous_class_name(
        mut class_expr: Expression<'a>,
        assigned_to_temp: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let Expression::ClassExpression(class) = &mut class_expr else { unreachable!() };
        if class.id.is_some() {
            return class_expr;
        }

        let Some(name) = Self::contextual_name(ctx) else {
            if !assigned_to_temp {
                // Last in a sequence expression, so already doesn't receive a name
                return class_expr;
            }
            // `(0, class {})`
            let zero = Expression::new_number_0(SPAN, ctx);
            return Expression::new_sequence_expression(SPAN, [zero, class_expr], ctx);
        };

        if let Some(ident_name) = name.as_str()
            && Self::can_name_class(class, ident_name, ctx)
        {
            // `class C {}`
            let binding =
                ctx.generate_binding(Ident::from(ident_name), class.scope_id(), SymbolFlags::Class);
            class.id = Some(binding.create_binding_identifier(ctx));
            return class_expr;
        }
        Self::create_named_evaluation(class_expr, name, ctx)
    }

    /// Get the name NamedEvaluation would give an anonymous class at the current position.
    fn contextual_name(ctx: &TraverseCtx<'a>) -> Option<JSStr<'a>> {
        let ancestor = ctx.ancestors().find(|ancestor| {
            !matches!(
                ancestor,
                Ancestor::ParenthesizedExpressionExpression(_)
                    | Ancestor::TSAsExpressionExpression(_)
                    | Ancestor::TSSatisfiesExpressionExpression(_)
                    | Ancestor::TSNonNullExpressionExpression(_)
                    | Ancestor::TSTypeAssertionExpression(_)
            )
        })?;
        match ancestor {
            // `{ x: class {} }`.
            // Computed keys which aren't literals (`{ [x]: class {} }`) aren't handled.
            Ancestor::ObjectPropertyValue(prop) => {
                let key = prop.key();
                // `{ __proto__: class {} }` sets the prototype, without NamedEvaluation.
                // `{ ["__proto__"]: class {} }` is a normal property.
                if key.is_specific_literal_name("__proto__") {
                    return None;
                }
                Self::property_key_name(key, ctx)
            }
            // `class C { static x = class {} }`
            Ancestor::PropertyDefinitionValue(prop) => Self::property_key_name(prop.key(), ctx),
            // `class C { accessor x = class {} }`
            Ancestor::AccessorPropertyValue(prop) => Self::property_key_name(prop.key(), ctx),
            ancestor => contextual_class_name(&ancestor).map(JSStr::from),
        }
    }

    /// Get the name NamedEvaluation gives a value of property `key`, if it's statically known.
    fn property_key_name(key: &PropertyKey<'a>, ctx: &TraverseCtx<'a>) -> Option<JSStr<'a>> {
        if let PropertyKey::PrivateIdentifier(ident) = key {
            return Some(JSStr::from(Str::from_strs_array_in(["#", ident.name.as_str()], ctx)));
        }
        Some(match key.static_name()? {
            StaticPropertyName::Borrowed(name) => name,
            StaticPropertyName::Owned(name) => JSStr::from(Str::from_str_in(name.as_str(), ctx)),
        })
    }

    /// Whether giving the class a binding `name` produces the same `name` without changing the
    /// meaning of any code in the class.
    ///
    /// The binding would shadow any outer binding `name` referenced inside the class,
    /// or referenced by a direct `eval` inside the class.
    fn can_name_class(class: &Class<'a>, name: &str, ctx: &TraverseCtx<'a>) -> bool {
        if !is_identifier_name(name)
            || is_reserved_keyword_or_global_object(name)
            || matches!(name, "arguments" | "eval")
        {
            return false;
        }

        let scoping = ctx.scoping();
        let class_scope_id = class.scope_id();
        if scoping.scope_flags(class_scope_id).contains_direct_eval() {
            return false;
        }
        let is_in_class = |&reference_id: &ReferenceId| {
            let scope_id = scoping.get_reference(reference_id).scope_id();
            scoping.scope_ancestors(scope_id).any(|scope_id| scope_id == class_scope_id)
        };

        // References in the class which resolve to an outer `name`, or a global `name`
        let name = Ident::from(name);
        let outer_scope_id = scoping.scope_parent_id(class_scope_id).unwrap();
        if let Some(symbol_id) = scoping.find_binding(outer_scope_id, name) {
            !scoping.get_resolved_reference_ids(symbol_id).iter().any(is_in_class)
        } else {
            !scoping
                .root_unresolved_references()
                .get(&name)
                .is_some_and(|reference_ids| reference_ids.iter().any(is_in_class))
        }
    }

    /// `{ name: class {} }.name`, or `{ "na-me": class {} }["na-me"]`.
    fn create_named_evaluation(
        class_expr: Expression<'a>,
        name: JSStr<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        let create_object = |key| {
            let property = ObjectPropertyKind::new_object_property(
                SPAN,
                PropertyKind::Init,
                key,
                class_expr,
                false,
                false,
                ctx,
            );
            Expression::new_object_expression(SPAN, [property], ctx)
        };

        if let Some(ident_name) =
            name.as_str().filter(|name| *name != "__proto__" && is_identifier_name(name))
        {
            // `{ name: class {} }.name`
            let key = PropertyKey::new_static_identifier(SPAN, ident_name, ctx);
            let object = create_object(key);
            let property = IdentifierName::new(SPAN, ident_name, ctx);
            Expression::new_static_member_expression(SPAN, object, property, false, ctx)
        } else {
            // `{ "na-me": class {} }["na-me"]`.
            // `{ ["__proto__"]: class {} }` is a normal property, unlike `{ __proto__: class {} }`.
            let key = if name == "__proto__" {
                PropertyKey::new_string_literal(SPAN, name, ctx)
            } else {
                PropertyKey::new_static_string_literal(SPAN, name, ctx)
            };
            let object = create_object(key);
            let key = Expression::new_string_literal(SPAN, name, ctx);
            Expression::new_computed_member_expression(SPAN, object, key, false, ctx)
        }
    }
}
