use oxc_allocator::Box;
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_syntax::reference::ReferenceFlag;
use oxc_traverse::TraverseCtx;

use super::TypeScript;

impl<'a> TypeScript<'a> {
    /// ```TypeScript
    /// import b = babel;
    /// import AliasModule = LongNameModule;
    ///
    /// ```JavaScript
    /// var b = babel;
    /// var AliasModule = LongNameModule;
    /// ```
    pub fn transform_declaration(&mut self, decl: &mut Declaration<'a>, ctx: &mut TraverseCtx<'a>) {
        match decl {
            Declaration::TSImportEqualsDeclaration(ts_import_equals)
                if ts_import_equals.import_kind.is_value() =>
            {
                *decl = self.transform_ts_import_equals(ts_import_equals, ctx);
            }
            _ => {}
        }
    }

    fn transform_ts_import_equals(
        &self,
        decl: &mut Box<'a, TSImportEqualsDeclaration<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Declaration<'a> {
        let kind = VariableDeclarationKind::Var;
        let decls = {
            let binding_identifier = BindingIdentifier::new(SPAN, decl.id.name.clone());
            let binding_pattern_kind = self.ctx.ast.binding_pattern_identifier(binding_identifier);
            let binding = self.ctx.ast.binding_pattern(binding_pattern_kind, None, false);
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

                    let callee = self.ctx.ast.identifier_reference_expression(
                        IdentifierReference::new(SPAN, "require".into()),
                    );
                    let arguments = self.ctx.ast.new_vec_single(Argument::from(
                        self.ctx.ast.literal_string_expression(reference.expression.clone()),
                    ));
                    self.ctx.ast.call_expression(SPAN, callee, arguments, false, None)
                }
            };
            self.ctx.ast.new_vec_single(self.ctx.ast.variable_declarator(
                SPAN,
                kind,
                binding,
                Some(init),
                false,
            ))
        };
        let variable_declaration = self.ctx.ast.variable_declaration(SPAN, kind, decls, false);

        Declaration::VariableDeclaration(variable_declaration)
    }

    fn transform_ts_type_name(
        &self,
        type_name: &mut TSTypeName<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match type_name {
            TSTypeName::IdentifierReference(ident) => {
                ident.reference_flag = ReferenceFlag::Read;
                if let Some(reference_id) = ident.reference_id.get() {
                    let reference = ctx.symbols_mut().get_reference_mut(reference_id);
                    *reference.flag_mut() = ReferenceFlag::Read;
                } else {
                    // unreachable!()
                }
                self.ctx.ast.identifier_reference_expression(ctx.ast.copy(ident))
            }
            TSTypeName::QualifiedName(qualified_name) => self.ctx.ast.static_member_expression(
                SPAN,
                self.transform_ts_type_name(&mut qualified_name.left, ctx),
                qualified_name.right.clone(),
                false,
            ),
        }
    }

    pub fn transform_ts_export_assignment(
        &mut self,
        export_assignment: &mut TSExportAssignment<'a>,
    ) {
        if self.ctx.source_type.is_module() {
            self.ctx
                .error(super::diagnostics::export_assignment_unsupported(export_assignment.span));
        }
    }
}
