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
//!
//! Lowering class fields moves their initializers out of the field, which would also lose the
//! name NamedEvaluation gives an anonymous class or function initializer:
//!
//! ```js
//! class C { x = () => {}; [k] = class {} }
//! // ->
//! class C {
//!   constructor() {
//!     _defineProperty(this, "x", { x: () => {} }.x);
//!     _defineProperty(this, k, _setFunctionName(class {}, k));
//!   }
//! }
//! ```
//!
//! [`ClassProperties::name_field_initializer`] names such initializers (see [`FunctionName`]
//! for the forms used). Anonymous classes are named when the class itself is exited, so a class
//! which is lowered too is named before its static initializers run.

use oxc_allocator::{ArenaBox, ArenaVec, GetAddress, TakeIn};
use oxc_ast::{StaticPropertyName, ast::*};
use oxc_span::SPAN;
use oxc_str::{Ident, JSStr, Str};
use oxc_syntax::{
    identifier::is_identifier_name, keyword::is_reserved_keyword_or_global_object,
    reference::ReferenceId, scope::ScopeId, symbol::SymbolFlags,
};
use oxc_traverse::{Ancestor, BoundIdentifier};

use crate::{
    common::{
        computed_key::key_needs_temp_var,
        duplicate::duplicate_expression,
        helper_loader::{Helper, helper_call_expr},
    },
    context::TraverseCtx,
    utils::{
        ast_builder::create_assignment,
        class_name::contextual_class_name,
        decorated_class::{class_body_has_auto_accessors, class_has_decorators},
    },
};

use super::ClassProperties;

/// The name NamedEvaluation gives an anonymous class or function.
pub enum FunctionName<'a> {
    /// Statically known name, e.g. `x` for `x = class {}`, or `#x` for `#x = class {}`.
    ///
    /// Given as a binding (`class x {}`, `function x() {}`) where that is safe,
    /// otherwise by NamedEvaluation in an object literal (`{ x: () => {} }.x`).
    Static(JSStr<'a>),
    /// Property key known only at runtime,
    /// e.g. `_k` for `[k()] = class {}` after `_k = _toPropertyKey(k())`.
    ///
    /// Given with `_setFunctionName(class {}, _k)`, which also names symbol keys `[description]`.
    /// The expression must be free of side effects.
    Dynamic(Expression<'a>),
}

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
    /// `inferred_name` is the name recorded by [`Self::name_field_initializer`], if any.
    ///
    /// Must be called before the class is moved, while `ctx` still points at its original position.
    pub(super) fn preserve_anonymous_class_name(
        class_expr: Expression<'a>,
        assigned_to_temp: bool,
        inferred_name: Option<FunctionName<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let Expression::ClassExpression(class) = &class_expr else { unreachable!() };
        if class.id.is_some() {
            return class_expr;
        }

        let Some(name) =
            inferred_name.or_else(|| Self::contextual_name(ctx).map(FunctionName::Static))
        else {
            if !assigned_to_temp {
                // Last in a sequence expression, so already doesn't receive a name
                return class_expr;
            }
            // `(0, class {})`
            let zero = Expression::new_number_0(SPAN, ctx);
            return Expression::new_sequence_expression(SPAN, [zero, class_expr], ctx);
        };
        Self::name_anonymous_function(class_expr, name, ctx)
    }

    /// Name an anonymous class or function initializer of a class field, which is about to be
    /// moved out of the field, with the name NamedEvaluation would give it.
    ///
    /// Functions are named in place. Anonymous classes are recorded, and named when the class
    /// itself is exited (see [`Self::take_inferred_class_name`]).
    ///
    /// If the name comes from a computed key which isn't statically known, the key is moved to
    /// a temp var now: `[k()] = class {}` -> `[_k = _toPropertyKey(k())] = ...`.
    /// Lowering the property then reuses that temp var
    /// (see [`Self::create_computed_key_temp_var_if_required`]).
    ///
    /// Must be called before the initializer is traversed.
    pub(super) fn name_field_initializer(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Some(value) = &prop.value else { return };
        if !value.get_inner_expression().is_anonymous_function_definition() {
            return;
        }

        let name = if let Some(name) = Self::property_key_name(&prop.key, ctx) {
            FunctionName::Static(name)
        } else {
            let key = prop.key.to_expression_mut();
            FunctionName::Dynamic(self.duplicate_computed_key(key, ctx))
        };

        let value = prop.value.as_mut().unwrap();
        if let Expression::ClassExpression(class) = value.get_inner_expression() {
            self.inferred_class_names.insert(class.address(), name);
        } else {
            *value = Self::name_anonymous_function(value.take_in(ctx), name, ctx);
        }
    }

    /// Get a side-effect-free copy of computed property key `key`, converted to a property key.
    ///
    /// Unless the key is a literal, replace it with an assignment to a temp var
    /// (`k()` -> `_k = _toPropertyKey(k())`), and return the temp var.
    /// The key is converted once, at class definition, as `ToPropertyKey` may have side effects.
    fn duplicate_computed_key(
        &mut self,
        key: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if !matches!(key, Expression::Identifier(_)) && !key_needs_temp_var(key, ctx) {
            // Literal e.g. `[true]`
            let (original, copy) = duplicate_expression(key.take_in(ctx), false, ctx);
            *key = original;
            return copy;
        }

        // `_k = _toPropertyKey(k())`
        let original_key = key.take_in(ctx);
        let binding = ctx.generate_uid_based_on_node(
            &original_key,
            ctx.current_block_scope_id(),
            SymbolFlags::BlockScopedVariable,
        );
        ctx.state.var_declarations.insert_let(&binding, None, &ctx.ast);
        let arguments = ArenaVec::from_array_in([Argument::from(original_key)], ctx);
        let property_key = helper_call_expr(Helper::ToPropertyKey, arguments, ctx);
        let assignment = create_assignment(&binding, property_key, SPAN, ctx);
        let Expression::AssignmentExpression(assign_expr) = &assignment else { unreachable!() };
        self.computed_key_assignments.insert(assign_expr.address());
        *key = assignment;
        binding.create_read_expression(ctx)
    }

    /// If computed key `key` was moved to a temp var by [`Self::duplicate_computed_key`],
    /// return a reference to the temp var.
    pub(super) fn take_computed_key_temp_var(
        &mut self,
        key: &Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let Expression::AssignmentExpression(assign_expr) = key else { return None };
        if !self.computed_key_assignments.remove(&assign_expr.address()) {
            return None;
        }
        let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign_expr.left else {
            unreachable!()
        };
        let symbol_id = ctx.scoping().get_reference(ident.reference_id()).symbol_id().unwrap();
        Some(BoundIdentifier::new(ident.name, symbol_id).create_read_expression(ctx))
    }

    /// Take the name recorded for anonymous class `class` by [`Self::name_field_initializer`].
    pub(super) fn take_inferred_class_name(
        &mut self,
        class: &ArenaBox<'a, Class<'a>>,
    ) -> Option<FunctionName<'a>> {
        if self.inferred_class_names.is_empty() {
            return None;
        }
        self.inferred_class_names.remove(&class.address())
    }

    /// Give anonymous class or function `value` the name `name`, as NamedEvaluation would.
    ///
    /// `value` may be wrapped in parentheses or TypeScript syntax.
    pub(crate) fn name_anonymous_function(
        mut value: Expression<'a>,
        name: FunctionName<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let ident_name = match &name {
            FunctionName::Static(name) => name.as_str(),
            FunctionName::Dynamic(_) => None,
        };
        match value.get_inner_expression_mut() {
            Expression::ClassExpression(class) => {
                debug_assert!(class.id.is_none());
                // A static `name` member overrides the name anyway, and `_setFunctionName`
                // would overwrite it.
                // Note: A static computed key which evaluates to `"name"` is not detected.
                if Self::class_defines_static_name(class) {
                    return value;
                }
                if let Some(ident_name) = ident_name
                    && Self::can_bind_name(class.scope_id(), ident_name, ctx)
                {
                    // `class x {}`
                    let binding = ctx.generate_binding(
                        Ident::from(ident_name),
                        class.scope_id(),
                        SymbolFlags::Class,
                    );
                    class.id = Some(binding.create_binding_identifier(ctx));
                    return value;
                }
            }
            Expression::FunctionExpression(func) => {
                debug_assert!(func.id.is_none());
                // A function expression's name is bound in its own scope,
                // alongside its parameters and `var`s
                if let Some(ident_name) = ident_name
                    && ctx.scoping().get_binding(func.scope_id(), Ident::from(ident_name)).is_none()
                    && Self::can_bind_name(func.scope_id(), ident_name, ctx)
                {
                    // `function x() {}`
                    let binding = ctx.generate_binding(
                        Ident::from(ident_name),
                        func.scope_id(),
                        SymbolFlags::Function | SymbolFlags::FunctionExpression,
                    );
                    func.id = Some(binding.create_binding_identifier(ctx));
                    return value;
                }
            }
            _ => {}
        }

        match name {
            // `{ x: () => {} }.x`
            FunctionName::Static(name) => Self::create_named_evaluation(value, name, ctx),
            // `_setFunctionName(() => {}, _k)`
            FunctionName::Dynamic(key) => {
                let arguments =
                    ArenaVec::from_array_in([Argument::from(value), Argument::from(key)], ctx);
                helper_call_expr(Helper::SetFunctionName, arguments, ctx)
            }
        }
    }

    /// Whether class has a static method, accessor or field with key `name`.
    fn class_defines_static_name(class: &Class<'a>) -> bool {
        class.body.body.iter().any(|element| {
            element.r#static()
                && !element.property_key().is_some_and(PropertyKey::is_private_identifier)
                && element.static_name().is_some_and(|name| name == "name")
        })
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

    /// Whether giving the class or function with scope `scope_id` a binding `name` produces the
    /// same `name` without changing the meaning of any code in it.
    ///
    /// The binding would shadow any outer binding `name` referenced inside it,
    /// or referenced by a direct `eval` inside it.
    fn can_bind_name(scope_id: ScopeId, name: &str, ctx: &TraverseCtx<'a>) -> bool {
        if !is_identifier_name(name)
            || is_reserved_keyword_or_global_object(name)
            || matches!(name, "arguments" | "eval")
        {
            return false;
        }

        let scoping = ctx.scoping();
        if scoping.scope_flags(scope_id).contains_direct_eval() {
            return false;
        }
        let is_inside = |&reference_id: &ReferenceId| {
            let reference_scope_id = scoping.get_reference(reference_id).scope_id();
            scoping.scope_ancestors(reference_scope_id).any(|id| id == scope_id)
        };

        // References inside which resolve to an outer `name`, or a global `name`
        let name = Ident::from(name);
        let outer_scope_id = scoping.scope_parent_id(scope_id).unwrap();
        if let Some(symbol_id) = scoping.find_binding(outer_scope_id, name) {
            !scoping.get_resolved_reference_ids(symbol_id).iter().any(is_inside)
        } else {
            !scoping
                .root_unresolved_references()
                .get(&name)
                .is_some_and(|reference_ids| reference_ids.iter().any(is_inside))
        }
    }

    /// `{ name: class {} }.name`, or `{ "na-me": class {} }["na-me"]`.
    fn create_named_evaluation(
        value: Expression<'a>,
        name: JSStr<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        let create_object = |key| {
            let property = ObjectPropertyKind::new_object_property(
                SPAN,
                PropertyKind::Init,
                key,
                value,
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
                PropertyKey::new_string_literal(SPAN, name, None, ctx)
            } else {
                PropertyKey::new_static_string_literal(SPAN, name, None, ctx)
            };
            let object = create_object(key);
            let key = Expression::new_string_literal(SPAN, name, None, ctx);
            Expression::new_computed_member_expression(SPAN, object, key, false, ctx)
        }
    }
}
