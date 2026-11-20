//! ES2022: Class Properties
//! Preserve the `name` of anonymous classes.
//!
//! An anonymous class gets its `name` from NamedEvaluation of its surrounding context.
//! e.g. `C` in `const C = class {}`, or `""` when there is no such context, as in `f(class {})`.
//!
//! When a class expression is transformed, it's moved out of that context into a sequence
//! and/or an assignment to a temp var:
//!
//! ```js
//! const C = class { static x = 1 };
//! // ->
//! const C = (_Class = class {}, _defineProperty(_Class, "x", 1), _Class);
//! ```
//!
//! `C` no longer names the class, and `_Class = class {}` names it `_Class` instead.
//!
//! To preserve the original name, we restore the NamedEvaluation:
//!
//! * If context provides a name, give the class that name: `_Class = class C {}`.
//!   If that would shadow a binding referenced inside the class, perform NamedEvaluation in
//!   an object literal instead: `_Class = { C: class {} }.C`.
//! * Otherwise, prevent NamedEvaluation in the temp var assignment: `_Class = (0, class {})`.

use oxc_allocator::TakeIn;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_span::SPAN;
use oxc_str::{Ident, JSStr, Str};
use oxc_syntax::{
    identifier::is_identifier_name, keyword::is_reserved_keyword_or_global_object,
    symbol::SymbolFlags,
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
        // Leave classes which are lowered by other transforms (decorators) unchanged
        if class.id.is_some()
            || class.declare
            || class_has_decorators(class)
            || class_body_has_auto_accessors(&class.body)
        {
            return;
        }

        let needs_temp_var = class.body.body.iter().any(|element| match element {
            ClassElement::PropertyDefinition(prop) => prop.r#static && !prop.declare,
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

    /// Preserve the `name` of an anonymous class expression which is being moved out of the
    /// position where it would receive its name.
    ///
    /// * `const C = class {}` -> `class C {}`, if naming the class can't shadow an outer `C`.
    /// * `const C = class { m() { C } }` -> `{ C: class { m() { C } } }.C`
    /// * `f(class {})` -> `(0, class {})`, if `assigned_to_temp`. If the class is not assigned to
    ///   a temp var, it's last in a sequence expression, so already doesn't receive a name.
    ///
    /// Must be called before the class is moved, while `ctx` still points at its original position.
    pub(super) fn preserve_anonymous_class_name(
        mut class_expr: Expression<'a>,
        assigned_to_temp: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if let Some(name) = Self::contextual_name(ctx) {
            let Expression::ClassExpression(class) = &mut class_expr else { unreachable!() };
            if let Some(ident_name) = name.as_str()
                && Self::can_name_class(class, ident_name)
            {
                // `class C {}`
                let scope_id = class.scope_id();
                let binding =
                    ctx.generate_binding(Ident::from(ident_name), scope_id, SymbolFlags::Class);
                class.id = Some(binding.create_binding_identifier(ctx));
                return class_expr;
            }
            return Self::create_named_evaluation(class_expr, name, ctx);
        }
        if assigned_to_temp {
            // `(0, class {})`
            let zero = Expression::new_number_0(SPAN, ctx);
            return Expression::new_sequence_expression(SPAN, [zero, class_expr], ctx);
        }
        class_expr
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
            // `{ x: class {} }`
            // Computed keys other than string literals (`{ [x]: class {} }`) aren't handled.
            Ancestor::ObjectPropertyValue(prop) => {
                let name = match prop.key() {
                    PropertyKey::StaticIdentifier(ident) => JSStr::from(ident.name),
                    PropertyKey::StringLiteral(lit) => lit.value,
                    _ => return None,
                };
                // `{ __proto__: class {} }` sets the prototype, without NamedEvaluation.
                // `{ ["__proto__"]: class {} }` is a normal property.
                (*prop.computed() || name != "__proto__").then_some(name)
            }
            // `class C { static x = class {} }`
            Ancestor::PropertyDefinitionValue(prop) => match prop.key() {
                PropertyKey::StaticIdentifier(ident) => Some(JSStr::from(ident.name)),
                PropertyKey::StringLiteral(lit) => Some(lit.value),
                PropertyKey::PrivateIdentifier(ident) => {
                    Some(JSStr::from(Str::from_strs_array_in(["#", ident.name.as_str()], ctx)))
                }
                _ => None,
            },
            ancestor => contextual_class_name(&ancestor).map(JSStr::from),
        }
    }

    /// Whether giving the class a binding `name` produces the same `name` without changing the
    /// meaning of any code in the class.
    ///
    /// The binding would shadow any outer binding `name` referenced inside the class,
    /// or referenced by a direct `eval` inside the class.
    fn can_name_class(class: &Class<'a>, name: &str) -> bool {
        if !is_identifier_name(name)
            || is_reserved_keyword_or_global_object(name)
            || matches!(name, "arguments" | "eval")
        {
            return false;
        }
        let mut finder = ReferenceFinder { name, found: false };
        finder.visit_class(class);
        !finder.found
    }

    /// `{ name: class {} }.name`, or `{ "na-me": class {} }["na-me"]`.
    fn create_named_evaluation(
        class_expr: Expression<'a>,
        name: JSStr<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        let create_object = |key, computed| {
            let property = ObjectPropertyKind::new_object_property(
                SPAN,
                PropertyKind::Init,
                key,
                class_expr,
                false,
                false,
                computed,
                ctx,
            );
            Expression::new_object_expression(SPAN, [property], ctx)
        };

        if let Some(ident_name) =
            name.as_str().filter(|name| *name != "__proto__" && is_identifier_name(name))
        {
            // `{ name: class {} }.name`
            let key = PropertyKey::new_static_identifier(SPAN, ident_name, ctx);
            let object = create_object(key, false);
            let property = IdentifierName::new(SPAN, ident_name, ctx);
            Expression::new_static_member_expression(SPAN, object, property, false, ctx)
        } else {
            // `{ "na-me": class {} }["na-me"]`.
            // `{ ["__proto__"]: class {} }` is a normal property, unlike `{ __proto__: class {} }`.
            let key = PropertyKey::new_string_literal(SPAN, name, None, ctx);
            let object = create_object(key, name == "__proto__");
            let key = Expression::new_string_literal(SPAN, name, None, ctx);
            Expression::new_computed_member_expression(SPAN, object, key, false, ctx)
        }
    }
}

/// Finds references to `name`, or to `eval`, which may reference any name.
struct ReferenceFinder<'n> {
    name: &'n str,
    found: bool,
}

impl<'a> Visit<'a> for ReferenceFinder<'_> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if ident.name == self.name || ident.name == "eval" {
            self.found = true;
        }
    }
}
