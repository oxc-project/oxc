use oxc_allocator::Box;
use oxc_ast::{ast::*, NONE};
use oxc_span::SPAN;
use oxc_syntax::reference::ReferenceFlags;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

pub struct TypeScriptModule<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> TypeScriptModule<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for TypeScriptModule<'a, 'ctx> {
    /// ```TypeScript
    /// import b = babel;
    /// import AliasModule = LongNameModule;
    ///
    /// ```JavaScript
    /// var b = babel;
    /// var AliasModule = LongNameModule;
    /// ```
    fn enter_declaration(&mut self, decl: &mut Declaration<'a>, ctx: &mut TraverseCtx<'a>) {
        match decl {
            Declaration::TSImportEqualsDeclaration(ts_import_equals)
                if ts_import_equals.import_kind.is_value() =>
            {
                *decl = self.transform_ts_import_equals(ts_import_equals, ctx);
            }
            _ => {}
        }
    }

    fn enter_ts_export_assignment(
        &mut self,
        export_assignment: &mut TSExportAssignment<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        if self.ctx.source_type.is_module() {
            self.ctx
                .error(super::diagnostics::export_assignment_unsupported(export_assignment.span));
        }
    }
}

impl<'a, 'ctx> TypeScriptModule<'a, 'ctx> {
    fn transform_ts_import_equals(
        &self,
        decl: &mut Box<'a, TSImportEqualsDeclaration<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Declaration<'a> {
        let kind = VariableDeclarationKind::Var;
        let decls = {
            let binding_pattern_kind =
                ctx.ast.binding_pattern_kind_binding_identifier(SPAN, &decl.id.name);
            let binding = ctx.ast.binding_pattern(binding_pattern_kind, NONE, false);
            let decl_span = decl.span;

            let init = match &mut decl.module_reference {
                type_name @ match_ts_type_name!(TSModuleReference) => {
                    self.transform_ts_type_name(&mut *type_name.to_ts_type_name_mut(), ctx)
                }
                TSModuleReference::ExternalModuleReference(reference) => {
                    if self.ctx.source_type.is_module() {
                        self.ctx.error(super::diagnostics::import_equals_require_unsupported(
                            decl_span,
                        ));
                    }

                    let callee = ctx.ast.expression_identifier_reference(SPAN, "require");
                    let arguments = ctx.ast.vec1(Argument::from(
                        ctx.ast.expression_from_string_literal(reference.expression.clone()),
                    ));
                    ctx.ast.expression_call(SPAN, callee, NONE, arguments, false)
                }
            };
            ctx.ast.vec1(ctx.ast.variable_declarator(SPAN, kind, binding, Some(init), false))
        };

        ctx.ast.declaration_variable(SPAN, kind, decls, false)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn transform_ts_type_name(
        &self,
        type_name: &mut TSTypeName<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match type_name {
            TSTypeName::IdentifierReference(ident) => {
                let ident = ident.clone();
                let reference_id = ident.reference_id.get().unwrap();
                let reference = ctx.symbols_mut().get_reference_mut(reference_id);
                *reference.flags_mut() = ReferenceFlags::Read;
                ctx.ast.expression_from_identifier_reference(ident)
            }
            TSTypeName::QualifiedName(qualified_name) => ctx
                .ast
                .member_expression_static(
                    SPAN,
                    self.transform_ts_type_name(&mut qualified_name.left, ctx),
                    qualified_name.right.clone(),
                    false,
                )
                .into(),
        }
    }
}
