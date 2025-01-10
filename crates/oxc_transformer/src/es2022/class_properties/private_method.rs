//! ES2022: Class Properties
//! Transform of private method uses e.g. `this.#method()`.

use oxc_ast::{ast::*, visit::walk_mut, VisitMut};
use oxc_semantic::ScopeFlags;
use oxc_span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::Helper;

use super::{
    super_converter::{ClassPropertiesSuperConverter, ClassPropertiesSuperConverterMode},
    ClassProperties,
};

impl<'a> ClassProperties<'a, '_> {
    /// Convert method definition where the key is a private identifier and
    /// insert it after the class.
    ///
    /// ```js
    /// class C {
    ///    #method() {}
    ///    set #prop(value) {}
    ///    get #prop() {return 0}
    /// }
    /// ```
    ///
    /// ->
    ///
    /// ```js
    /// class C {}
    /// function _method() {}
    /// function _set_prop(value) {}
    /// function _get_prop() {return 0}
    /// ```
    ///
    /// Returns statement to insert after class.
    pub(super) fn convert_private_method(
        &mut self,
        element: &mut ClassElement<'a>,
        ident: &PrivateIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        // We receive a `&mut ClassElement<'a>` and move it onto stack (16 bytes), instead of using
        // `move_function` to avoid moving a `Function` (large type) out of the arena onto stack and then
        // allocating it back into arena again.
        // `function` here is a `Box<Function>` - we mutate its properties in place in the arena.
        let element = ctx.ast.move_class_element(element);
        let ClassElement::MethodDefinition(method) = element else { unreachable!() };
        let MethodDefinition { value: mut function, span, kind, r#static: is_static, .. } =
            method.unbox();

        let resolved_private_prop = if kind == MethodDefinitionKind::Set {
            self.classes_stack.find_writeable_private_prop(ident)
        } else {
            self.classes_stack.find_readable_private_prop(ident)
        };
        let temp_binding = resolved_private_prop.unwrap().prop_binding;

        function.span = span;
        function.id = Some(temp_binding.create_binding_identifier(ctx));
        function.r#type = FunctionType::FunctionDeclaration;

        // Change parent scope of function to current scope id and remove
        // strict mode flag if parent scope is not strict mode.
        let scope_id = function.scope_id();
        let new_parent_id = ctx.current_scope_id();
        ctx.scopes_mut().change_parent_id(scope_id, Some(new_parent_id));
        let is_strict_mode = ctx.current_scope_flags().is_strict_mode();
        let flags = ctx.scopes_mut().get_flags_mut(scope_id);
        *flags -= ScopeFlags::GetAccessor | ScopeFlags::SetAccessor;
        if !is_strict_mode {
            // TODO: Needs to remove all child scopes' strict mode flag if child scope
            // is inherited from this scope.
            *flags -= ScopeFlags::StrictMode;
        }

        PrivateMethodVisitor::new(is_static, self, ctx)
            .visit_function(&mut function, ScopeFlags::Function);

        Statement::FunctionDeclaration(function)
    }

    // `_classPrivateMethodInitSpec(this, brand)`
    pub(super) fn create_class_private_method_init_spec(
        &self,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let brand = self.classes_stack.last().bindings.brand.as_ref().unwrap();
        let arguments = ctx.ast.vec_from_array([
            Argument::from(ctx.ast.expression_this(SPAN)),
            Argument::from(brand.create_read_expression(ctx)),
        ]);
        self.ctx.helper_call_expr(Helper::ClassPrivateMethodInitSpec, SPAN, arguments, ctx)
    }
}

/// Visitor to transform:
///
/// Almost the same as `super::static_block_and_prop_init::StaticVisitor`,
/// but only does following:
///
/// 1. Reference to class name to class temp var.
/// 2. Transform `super` expressions.
struct PrivateMethodVisitor<'a, 'ctx, 'v> {
    super_converter: ClassPropertiesSuperConverter<'a, 'ctx, 'v>,
    /// `TraverseCtx` object.
    ctx: &'v mut TraverseCtx<'a>,
}

impl<'a, 'ctx, 'v> PrivateMethodVisitor<'a, 'ctx, 'v> {
    fn new(
        r#static: bool,
        class_properties: &'v mut ClassProperties<'a, 'ctx>,
        ctx: &'v mut TraverseCtx<'a>,
    ) -> Self {
        let mode = if r#static {
            ClassPropertiesSuperConverterMode::StaticPrivateMethod
        } else {
            ClassPropertiesSuperConverterMode::PrivateMethod
        };
        Self { super_converter: ClassPropertiesSuperConverter::new(mode, class_properties), ctx }
    }
}

impl<'a> VisitMut<'a> for PrivateMethodVisitor<'a, '_, '_> {
    #[inline]
    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        match expr {
            // `super.prop`
            Expression::StaticMemberExpression(_) => {
                self.super_converter.transform_static_member_expression(expr, self.ctx);
            }
            // `super[prop]`
            Expression::ComputedMemberExpression(_) => {
                self.super_converter.transform_computed_member_expression(expr, self.ctx);
            }
            // `super.prop()`
            Expression::CallExpression(call_expr) => {
                self.super_converter
                    .transform_call_expression_for_super_member_expr(call_expr, self.ctx);
            }
            // `super.prop = value`, `super.prop += value`, `super.prop ??= value`
            Expression::AssignmentExpression(_) => {
                self.super_converter
                    .transform_assignment_expression_for_super_assignment_target(expr, self.ctx);
            }
            // `super.prop++`, `--super.prop`
            Expression::UpdateExpression(_) => {
                self.super_converter
                    .transform_update_expression_for_super_assignment_target(expr, self.ctx);
            }
            _ => {}
        }
        walk_mut::walk_expression(self, expr);
    }

    /// Transform reference to class name to temp var
    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        self.super_converter.class_properties.replace_class_name_with_temp_var(ident, self.ctx);
    }

    #[inline]
    fn visit_class(&mut self, _class: &mut Class<'a>) {
        // Ignore because we don't need to transform `super` for other classes.
    }
}
