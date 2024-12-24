//! ES2022: Class Properties
//! Transform of private method uses e.g. `this.#method()`.

use oxc_ast::ast::{Argument, Expression, FunctionType, MethodDefinition, PropertyKey, Statement};
use oxc_semantic::ScopeFlags;
use oxc_span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::Helper;

use super::ClassProperties;

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
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
    /// Returns `true` if the method was converted.
    pub(super) fn convert_private_method(
        &mut self,
        method: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        let PropertyKey::PrivateIdentifier(ident) = &method.key else {
            return false;
        };

        let mut function = ctx.ast.move_function(&mut method.value);

        let temp_binding = self.classes_stack.find_private_prop(ident).prop_binding;
        function.id = Some(temp_binding.create_binding_identifier(ctx));
        function.r#type = FunctionType::FunctionDeclaration;

        let scope_id = function.scope_id();
        let new_parent_id = ctx.current_block_scope_id();
        ctx.scopes_mut().change_parent_id(scope_id, Some(new_parent_id));
        *ctx.scopes_mut().get_flags_mut(scope_id) -= ScopeFlags::StrictMode;

        let function = ctx.ast.alloc(function);
        self.insert_after_stmts.push(Statement::FunctionDeclaration(function));

        true
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
