//! ES2022: Class Properties
//! Transform of class property declarations (instance or static properties).

use oxc_ast::{ast::*, NONE};
use oxc_span::SPAN;
use oxc_syntax::reference::ReferenceFlags;
use oxc_traverse::TraverseCtx;

use crate::common::helper_loader::Helper;

use super::{
    utils::{create_assignment, create_underscore_ident_name, create_variable_declaration},
    ClassProperties,
};

// Instance properties
impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    /// Convert instance property to initialization expression.
    /// Property `foo = 123;` -> Expression `this.foo = 123` or `_defineProperty(this, "foo", 123)`.
    pub(super) fn convert_instance_property(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        instance_inits: &mut Vec<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Get value
        let value = match &mut prop.value {
            Some(value) => {
                self.transform_instance_initializer(value, ctx);
                ctx.ast.move_expression(value)
            }
            None => ctx.ast.void_0(SPAN),
        };

        let init_expr = if let PropertyKey::PrivateIdentifier(ident) = &mut prop.key {
            self.create_private_instance_init_assignment(ident, value, ctx)
        } else {
            // Convert to assignment or `_defineProperty` call, depending on `loose` option
            let this = ctx.ast.expression_this(SPAN);
            self.create_init_assignment(prop, value, this, false, ctx)
        };
        instance_inits.push(init_expr);
    }

    /// Create init assignment for private instance prop, to be inserted into class constructor.
    ///
    /// Loose: `Object.defineProperty(this, _prop, {writable: true, value: value})`
    /// Not loose: `_classPrivateFieldInitSpec(this, _prop, value)`
    fn create_private_instance_init_assignment(
        &mut self,
        ident: &PrivateIdentifier<'a>,
        value: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if self.private_fields_as_properties {
            let this = ctx.ast.expression_this(SPAN);
            self.create_private_init_assignment_loose(ident, value, this, ctx)
        } else {
            self.create_private_instance_init_assignment_not_loose(ident, value, ctx)
        }
    }

    /// `_classPrivateFieldInitSpec(this, _prop, value)`
    fn create_private_instance_init_assignment_not_loose(
        &mut self,
        ident: &PrivateIdentifier<'a>,
        value: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let private_props = self.private_props_stack.last().unwrap();
        let prop = &private_props.props[&ident.name];
        let arguments = ctx.ast.vec_from_array([
            Argument::from(ctx.ast.expression_this(SPAN)),
            Argument::from(prop.binding.create_read_expression(ctx)),
            Argument::from(value),
        ]);
        // TODO: Should this have span of original `PropertyDefinition`?
        self.ctx.helper_call_expr(Helper::ClassPrivateFieldInitSpec, SPAN, arguments, ctx)
    }
}

// Static properties
impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    /// Convert static property to initialization expression.
    /// Property `static foo = 123;` -> Expression `C.foo = 123` or `_defineProperty(C, "foo", 123)`.
    pub(super) fn convert_static_property(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Get value, and transform it to replace `this` with reference to class name,
        // and transform class property accesses (`object.#prop`)
        let value = match &mut prop.value {
            Some(value) => {
                self.transform_static_initializer(value, ctx);
                ctx.ast.move_expression(value)
            }
            None => ctx.ast.void_0(SPAN),
        };

        if let PropertyKey::PrivateIdentifier(ident) = &mut prop.key {
            self.insert_private_static_init_assignment(ident, value, ctx);
        } else {
            // Convert to assignment or `_defineProperty` call, depending on `loose` option
            let class_binding = if self.is_declaration {
                // Class declarations always have a name except `export default class {}`.
                // For default export, binding is created when static prop found in 1st pass.
                self.class_bindings.name.as_ref().unwrap()
            } else {
                // Binding is created when static prop found in 1st pass.
                self.class_bindings.temp.as_ref().unwrap()
            };

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
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.private_fields_as_properties {
            self.insert_private_static_init_assignment_loose(ident, value, ctx);
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
        ctx: &mut TraverseCtx<'a>,
    ) {
        // TODO: This logic appears elsewhere. De-duplicate it.
        let class_binding = if self.is_declaration {
            // Class declarations always have a name except `export default class {}`.
            // For default export, binding is created when static prop found in 1st pass.
            self.class_bindings.name.as_ref().unwrap()
        } else {
            // Binding is created when static prop found in 1st pass.
            self.class_bindings.temp.as_ref().unwrap()
        };

        let assignee = class_binding.create_read_expression(ctx);
        let assignment = self.create_private_init_assignment_loose(ident, value, assignee, ctx);
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
        let property = ctx.ast.object_property_kind_object_property(
            SPAN,
            PropertyKind::Init,
            PropertyKey::StaticIdentifier(ctx.ast.alloc(create_underscore_ident_name(ctx))),
            value,
            false,
            false,
            false,
        );
        let obj = ctx.ast.expression_object(SPAN, ctx.ast.vec1(property), None);

        // Insert after class
        let private_props = self.private_props_stack.last().unwrap();
        let prop = &private_props.props[&ident.name];

        if self.is_declaration {
            // `var _prop = {_: value};`
            let var_decl = create_variable_declaration(&prop.binding, obj, ctx);
            self.insert_after_stmts.push(var_decl);
        } else {
            // `_prop = {_: value}`
            let assignment = create_assignment(&prop.binding, obj, ctx);
            self.insert_after_exprs.push(assignment);
        }
    }
}

// Used for both instance and static properties
impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    /// `assignee.foo = value` or `_defineProperty(assignee, "foo", value)`
    fn create_init_assignment(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        value: Expression<'a>,
        assignee: Expression<'a>,
        is_static: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if self.set_public_class_fields {
            // `assignee.foo = value`
            self.create_init_assignment_loose(prop, value, assignee, is_static, ctx)
        } else {
            // `_defineProperty(assignee, "foo", value)`
            self.create_init_assignment_not_loose(prop, value, assignee, ctx)
        }
    }

    /// `this.foo = value` or `_Class.foo = value`
    fn create_init_assignment_loose(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        value: Expression<'a>,
        assignee: Expression<'a>,
        is_static: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        // In-built static props `name` and `length` need to be set with `_defineProperty`
        let needs_define = |name| is_static && (name == "name" || name == "length");

        let left = match &mut prop.key {
            PropertyKey::StaticIdentifier(ident) => {
                if needs_define(&ident.name) {
                    return self.create_init_assignment_not_loose(prop, value, assignee, ctx);
                }
                ctx.ast.member_expression_static(SPAN, assignee, ident.as_ref().clone(), false)
            }
            PropertyKey::StringLiteral(str_lit) if needs_define(&str_lit.value) => {
                return self.create_init_assignment_not_loose(prop, value, assignee, ctx);
            }
            key @ match_expression!(PropertyKey) => {
                // TODO: This can also be a numeric key (non-computed). Maybe other key types?
                let key = self.create_computed_key_temp_var(key.to_expression_mut(), ctx);
                ctx.ast.member_expression_computed(SPAN, assignee, key, false)
            }
            PropertyKey::PrivateIdentifier(_) => {
                // Handled in `convert_instance_property` and `convert_static_property`
                unreachable!();
            }
        };

        // TODO: Should this have span of the original `PropertyDefinition`?
        ctx.ast.expression_assignment(
            SPAN,
            AssignmentOperator::Assign,
            AssignmentTarget::from(left),
            value,
        )
    }

    /// `_defineProperty(this, "foo", value)` or `_defineProperty(_Class, "foo", value)`
    fn create_init_assignment_not_loose(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        value: Expression<'a>,
        assignee: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let key = match &mut prop.key {
            PropertyKey::StaticIdentifier(ident) => {
                ctx.ast.expression_string_literal(ident.span, ident.name.clone(), None)
            }
            key @ match_expression!(PropertyKey) => {
                // TODO: This can also be a numeric key (non-computed). Maybe other key types?
                self.create_computed_key_temp_var(key.to_expression_mut(), ctx)
            }
            PropertyKey::PrivateIdentifier(_) => {
                // Handled in `convert_instance_property` and `convert_static_property`
                unreachable!();
            }
        };

        let arguments = ctx.ast.vec_from_array([
            Argument::from(assignee),
            Argument::from(key),
            Argument::from(value),
        ]);
        // TODO: Should this have span of the original `PropertyDefinition`?
        self.ctx.helper_call_expr(Helper::DefineProperty, SPAN, arguments, ctx)
    }

    /// `Object.defineProperty(<assignee>, _prop, {writable: true, value: value})`
    fn create_private_init_assignment_loose(
        &mut self,
        ident: &PrivateIdentifier<'a>,
        value: Expression<'a>,
        assignee: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        // `Object.defineProperty`
        let object_symbol_id = ctx.scopes().find_binding(ctx.current_scope_id(), "Object");
        let object = ctx.create_ident_expr(
            SPAN,
            Atom::from("Object"),
            object_symbol_id,
            ReferenceFlags::Read,
        );
        let property = ctx.ast.identifier_name(SPAN, "defineProperty");
        let callee =
            Expression::from(ctx.ast.member_expression_static(SPAN, object, property, false));

        // `{writable: true, value: <value>}`
        let prop_def = ctx.ast.expression_object(
            SPAN,
            ctx.ast.vec_from_array([
                ctx.ast.object_property_kind_object_property(
                    SPAN,
                    PropertyKind::Init,
                    ctx.ast.property_key_identifier_name(SPAN, Atom::from("writable")),
                    ctx.ast.expression_boolean_literal(SPAN, true),
                    false,
                    false,
                    false,
                ),
                ctx.ast.object_property_kind_object_property(
                    SPAN,
                    PropertyKind::Init,
                    ctx.ast.property_key_identifier_name(SPAN, Atom::from("value")),
                    value,
                    false,
                    false,
                    false,
                ),
            ]),
            None,
        );

        let private_props = self.private_props_stack.last().unwrap();
        let prop = &private_props.props[&ident.name];
        let arguments = ctx.ast.vec_from_array([
            Argument::from(assignee),
            Argument::from(prop.binding.create_read_expression(ctx)),
            Argument::from(prop_def),
        ]);
        // TODO: Should this have span of original `PropertyDefinition`?
        ctx.ast.expression_call(SPAN, callee, NONE, arguments, false)
    }
}
