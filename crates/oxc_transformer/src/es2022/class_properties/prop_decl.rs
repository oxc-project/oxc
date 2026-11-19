//! ES2022: Class Properties
//! Transform of class property declarations (instance or static properties).

use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::ast::*;
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_span::{GetSpan, SPAN, Span};
use oxc_str::static_ident;
use oxc_syntax::reference::ReferenceFlags;
use oxc_syntax::scope::ScopeFlags;

use crate::{
    common::helper_loader::{Helper, helper_call_expr},
    context::TraverseCtx,
    utils::ast_builder::{create_assignment, create_variable_declaration},
};

use super::{ClassProperties, utils::create_underscore_ident_name};

// Instance properties
impl<'a> ClassProperties<'a> {
    /// Convert instance property to initialization expression.
    /// Property `prop = 123;` -> Expression `this.prop = 123` or `_defineProperty(this, "prop", 123)`.
    pub(super) fn convert_instance_property(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        instance_inits: &mut Vec<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Get value
        let mut value = prop.value.take();
        if let Some(value) = &mut value {
            replace_new_target_in_initializer(value, ctx);
        }

        let span = prop.span;
        let init_expr = if let PropertyKey::PrivateIdentifier(ident) = &mut prop.key {
            let value = value.unwrap_or_else(|| Expression::new_void_0(SPAN, ctx));
            self.create_private_instance_init_assignment(ident, value, span, ctx)
        } else {
            let value = match value {
                Some(value) => value,
                // Do not need to convert property to `assignee.prop = void 0` if no initializer exists when
                // `set_public_class_fields` and `remove_class_fields_without_initializer`
                // are both true.
                // This is to align `TypeScript` with `useDefineForClassFields: false`.
                None if self.set_public_class_fields
                    && self.remove_class_fields_without_initializer =>
                {
                    return;
                }
                None => Expression::new_void_0(SPAN, ctx),
            };

            // Convert to assignment or `_defineProperty` call, depending on `loose` option
            let this = Expression::new_this_expression(SPAN, ctx);
            self.create_init_assignment(prop, value, this, false, ctx)
        };
        instance_inits.push(init_expr);
    }

    /// Create init assignment for private instance prop, to be inserted into class constructor.
    ///
    /// Loose: `Object.defineProperty(this, _prop, {writable: true, value: value})`
    /// Not loose: `_classPrivateFieldInitSpec(this, _prop, value)`
    fn create_private_instance_init_assignment(
        &self,
        ident: &PrivateIdentifier<'a>,
        value: Expression<'a>,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let this = Expression::new_this_expression(SPAN, ctx);
        if self.current_class().bindings.private_fields_as_properties() {
            self.create_private_init_assignment_loose(ident, value, this, span, ctx)
        } else {
            self.create_private_field_init_assignment_not_loose(ident, this, value, ctx)
        }
    }

    /// `_classPrivateFieldInitSpec(receiver, _prop, value)`
    fn create_private_field_init_assignment_not_loose(
        &self,
        ident: &PrivateIdentifier<'a>,
        receiver: Expression<'a>,
        value: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let private_props = self.current_class().private_props.lowered().unwrap();
        let prop = &private_props[&ident.name];
        let arguments = ArenaVec::from_array_in(
            [
                Argument::from(receiver),
                Argument::from(prop.binding.create_read_expression(ctx)),
                Argument::from(value),
            ],
            ctx,
        );
        helper_call_expr(Helper::ClassPrivateFieldInitSpec, arguments, ctx)
    }
}

// Static properties
impl<'a> ClassProperties<'a> {
    /// Convert static property to initialization expression.
    /// Property `static prop = 123;` -> Expression `C.prop = 123` or `_defineProperty(C, "prop", 123)`.
    pub(super) fn convert_static_property(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Get value.
        // Transform it to replace `this` and references to class name with temp var for class.
        // Also transform `super`.
        let value = prop.value.take().map(|mut value| {
            self.transform_static_initializer(&mut value, ctx);
            value
        });

        let span = prop.span;
        if let PropertyKey::PrivateIdentifier(ident) = &mut prop.key {
            let value = value.unwrap_or_else(|| Expression::new_void_0(SPAN, ctx));
            self.insert_private_static_init_assignment(ident, value, span, ctx);
        } else {
            let value = match value {
                Some(value) => value,
                // Do not need to convert property to `assignee.prop = void 0` if no initializer exists when
                // `set_public_class_fields` and `remove_class_fields_without_initializer`
                // are both true.
                // This is to align `TypeScript` with `useDefineForClassFields: false`.
                None if self.set_public_class_fields
                    && self.remove_class_fields_without_initializer =>
                {
                    return self.extract_computed_key(prop, ctx);
                }
                None => Expression::new_void_0(SPAN, ctx),
            };

            // Convert to assignment or `_defineProperty` call, depending on `loose` option
            let class_details = self.current_class();
            let class_binding = class_details.bindings.outer_binding(class_details.is_declaration);

            let assignee = class_binding.create_read_expression(ctx);
            let init_expr = self.create_init_assignment(prop, value, assignee, true, ctx);
            self.insert_expr_after_class(init_expr, ctx);
        }
    }

    /// Insert after class:
    ///
    /// Not loose:
    /// * Class declaration: `var _prop = {_: value};`
    /// * Class expression: `_prop = {_: value}`
    ///
    /// Loose:
    /// `Object.defineProperty(Class, _prop, {writable: true, value: value});`
    fn insert_private_static_init_assignment(
        &mut self,
        ident: &PrivateIdentifier<'a>,
        value: Expression<'a>,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let class_details = self.current_class();
        let private_prop = &class_details.private_props.lowered().unwrap()[&ident.name];
        if private_prop.uses_weakmap {
            let class_binding = class_details.bindings.outer_binding(class_details.is_declaration);
            let receiver = class_binding.create_read_expression(ctx);
            let init =
                self.create_private_field_init_assignment_not_loose(ident, receiver, value, ctx);
            self.insert_expr_after_class(init, ctx);
        } else if class_details.bindings.private_fields_as_properties() {
            self.insert_private_static_init_assignment_loose(ident, value, span, ctx);
        } else {
            self.insert_private_static_init_assignment_not_loose(ident, value, ctx);
        }
    }

    /// Insert after class:
    /// `Object.defineProperty(Class, _prop, {writable: true, value: value});`
    fn insert_private_static_init_assignment_loose(
        &mut self,
        ident: &PrivateIdentifier<'a>,
        value: Expression<'a>,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let class_details = self.current_class();
        let class_binding = class_details.bindings.outer_binding(class_details.is_declaration);

        let assignee = class_binding.create_read_expression(ctx);
        let assignment =
            self.create_private_init_assignment_loose(ident, value, assignee, span, ctx);
        self.insert_expr_after_class(assignment, ctx);
    }

    /// Insert after class:
    ///
    /// * Class declaration: `var _prop = {_: value};`
    /// * Class expression: `_prop = {_: value}`
    fn insert_private_static_init_assignment_not_loose(
        &mut self,
        ident: &PrivateIdentifier<'a>,
        value: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // `_prop = {_: value}`
        let property = ObjectPropertyKind::new_object_property(
            SPAN,
            PropertyKind::Init,
            PropertyKey::StaticIdentifier(ArenaBox::new_in(create_underscore_ident_name(ctx), ctx)),
            value,
            false,
            false,
            false,
            ctx,
        );
        let obj = Expression::new_object_expression(SPAN, [property], ctx);

        // Insert after class
        let class_details = self.current_class();
        let private_props = class_details.private_props.lowered().unwrap();
        let prop_binding = &private_props[&ident.name].binding;

        if class_details.is_declaration {
            // `var _prop = {_: value};`
            let var_decl = create_variable_declaration(prop_binding, obj, ctx);
            self.insert_after_stmts.push(var_decl);
        } else {
            // `_prop = {_: value}`
            let assignment = create_assignment(prop_binding, obj, SPAN, ctx);
            self.insert_after_exprs.push(assignment);
        }
    }
}

// Used for both instance and static properties
impl<'a> ClassProperties<'a> {
    /// `assignee.prop = value` or `_defineProperty(assignee, "prop", value)`
    /// `#[inline]` because the caller has been checked `self.set_public_class_fields`.
    /// After inlining, the two `self.set_public_class_fields` checks may be folded into one.
    #[inline]
    fn create_init_assignment(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        value: Expression<'a>,
        assignee: Expression<'a>,
        is_static: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if self.set_public_class_fields {
            // `assignee.prop = value`
            self.create_init_assignment_loose(prop, value, assignee, is_static, ctx)
        } else {
            // `_defineProperty(assignee, "prop", value)`
            self.create_init_assignment_not_loose(prop, value, assignee, is_static, ctx)
        }
    }

    /// `this.prop = value` or `_Class.prop = value`
    fn create_init_assignment_loose(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        value: Expression<'a>,
        assignee: Expression<'a>,
        is_static: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        // In-built static props `name` and `length` need to be set with `_defineProperty`
        let needs_define = |name: &str| is_static && (name == "name" || name == "length");

        let left = match &mut prop.key {
            PropertyKey::StaticIdentifier(ident) => {
                if needs_define(&ident.name) {
                    return self
                        .create_init_assignment_not_loose(prop, value, assignee, is_static, ctx);
                }
                MemberExpression::new_static_member_expression(
                    SPAN,
                    assignee,
                    ident.as_ref().clone(),
                    false,
                    ctx,
                )
            }
            PropertyKey::StringLiteral(str_lit)
                if str_lit.value.as_str().is_some_and(needs_define) =>
            {
                return self
                    .create_init_assignment_not_loose(prop, value, assignee, is_static, ctx);
            }
            key @ match_expression!(PropertyKey) => {
                let key = key.to_expression_mut();
                // Note: Key can also be static `StringLiteral` or `NumericLiteral`.
                // `class C { 'x' = true; 123 = false; }`
                // No temp var is created for these.
                // TODO: Any other possible static key types?
                let key = self.create_computed_key_temp_var_if_required(key, is_static, ctx);
                MemberExpression::new_computed_member_expression(SPAN, assignee, key, false, ctx)
            }
            PropertyKey::PrivateIdentifier(_) => {
                // Handled in `convert_instance_property` and `convert_static_property`
                unreachable!();
            }
        };

        Expression::new_assignment_expression(
            prop.span,
            AssignmentOperator::Assign,
            AssignmentTarget::from(left),
            value,
            ctx,
        )
    }

    /// `_defineProperty(this, "prop", value)` or `_defineProperty(_Class, "prop", value)`
    fn create_init_assignment_not_loose(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        value: Expression<'a>,
        assignee: Expression<'a>,
        is_static: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let key = match &mut prop.key {
            PropertyKey::StaticIdentifier(ident) => {
                Expression::new_string_literal(ident.span, ident.name, None, ctx)
            }
            key @ match_expression!(PropertyKey) => {
                let key = key.to_expression_mut();
                // Note: Key can also be static `StringLiteral` or `NumericLiteral`.
                // `class C { 'x' = true; 123 = false; }`
                // No temp var is created for these.
                // TODO: Any other possible static key types?
                self.create_computed_key_temp_var_if_required(key, is_static, ctx)
            }
            PropertyKey::PrivateIdentifier(_) => {
                // Handled in `convert_instance_property` and `convert_static_property`
                unreachable!();
            }
        };

        let arguments = ArenaVec::from_array_in(
            [Argument::from(assignee), Argument::from(key), Argument::from(value)],
            ctx,
        );
        helper_call_expr(Helper::DefineProperty, arguments, ctx)
    }

    /// `Object.defineProperty(<assignee>, _prop, {writable: true, value: value})`
    fn create_private_init_assignment_loose(
        &self,
        ident: &PrivateIdentifier<'a>,
        value: Expression<'a>,
        assignee: Expression<'a>,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        // `Object.defineProperty`
        let object_name = static_ident!("Object");
        let object_symbol_id = ctx.scoping().find_binding(ctx.current_scope_id(), object_name);
        let object =
            ctx.create_ident_expr(SPAN, object_name, object_symbol_id, ReferenceFlags::Read);
        let property = IdentifierName::new(SPAN, "defineProperty", ctx);
        let callee = Expression::new_static_member_expression(SPAN, object, property, false, ctx);

        // `{writable: true, value: <value>}`
        let prop_def = Expression::new_object_expression(
            SPAN,
            [
                ObjectPropertyKind::new_object_property(
                    SPAN,
                    PropertyKind::Init,
                    PropertyKey::new_static_identifier(SPAN, "writable", ctx),
                    Expression::new_boolean_literal(SPAN, true, ctx),
                    false,
                    false,
                    false,
                    ctx,
                ),
                ObjectPropertyKind::new_object_property(
                    SPAN,
                    PropertyKind::Init,
                    PropertyKey::new_static_identifier(SPAN, "value", ctx),
                    value,
                    false,
                    false,
                    false,
                    ctx,
                ),
            ],
            ctx,
        );

        let private_props = self.current_class().private_props.lowered().unwrap();
        let prop_binding = &private_props[&ident.name].binding;
        let arguments = ArenaVec::from_array_in(
            [
                Argument::from(assignee),
                Argument::from(prop_binding.create_read_expression(ctx)),
                Argument::from(prop_def),
            ],
            ctx,
        );
        Expression::new_call_expression(span, callee, None, arguments, false, ctx)
    }
}

/// `new.target` is `undefined` in a field initializer. Replace it before the initializer moves
/// into the constructor, where it would be the constructor instead.
///
/// Arrow functions inherit `new.target`; other functions and classes have their own.
fn replace_new_target_in_initializer<'a>(value: &mut Expression<'a>, ctx: &TraverseCtx<'a>) {
    struct Replacer<'c, 'a> {
        ctx: &'c TraverseCtx<'a>,
    }
    impl<'a> VisitMut<'a> for Replacer<'_, 'a> {
        fn visit_expression(&mut self, expr: &mut Expression<'a>) {
            if let Expression::NewTarget(new_target) = expr {
                *expr = Expression::new_void_0(new_target.span, self.ctx);
            } else {
                walk_mut::walk_expression(self, expr);
            }
        }
        fn visit_function(&mut self, _function: &mut Function<'a>, _flags: ScopeFlags) {}
        fn visit_class(&mut self, class: &mut Class<'a>) {
            // Computed keys and the heritage are evaluated in the initializer's context.
            if let Some(heritage) = &mut class.heritage {
                self.visit_expression(&mut heritage.expression);
            }
            for element in &mut class.body.body {
                let key = match element {
                    ClassElement::MethodDefinition(method) if method.computed => &mut method.key,
                    ClassElement::PropertyDefinition(prop) if prop.computed => &mut prop.key,
                    ClassElement::AccessorProperty(prop) if prop.computed => &mut prop.key,
                    _ => continue,
                };
                self.visit_property_key(key);
            }
        }
    }
    // `new.target` can't contain escapes, so an initializer without the text `target` has none.
    // Check only the initializer's own source text: checking the whole file for each field made
    // transforming large files with many fields quadratic. Generated initializers have an empty
    // span, so walk them.
    let span = value.span();
    if let Some(text) = ctx.state.source_text.get(span.start as usize..span.end as usize)
        && !text.is_empty()
        && !text.contains("target")
    {
        return;
    }
    Replacer { ctx }.visit_expression(value);
}
