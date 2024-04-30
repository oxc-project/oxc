use oxc_allocator::Box;
use oxc_ast::ast::*;
use oxc_span::SPAN;

use super::{
    diagnostics::{ExportAssignmentUnsupported, ImportEqualsRequireUnsupported},
    TypeScript,
};

impl<'a> TypeScript<'a> {
    fn transform_ts_type_name(&self, type_name: &mut TSTypeName<'a>) -> Expression<'a> {
        match type_name {
            TSTypeName::IdentifierReference(reference) => {
                self.ctx.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    reference.name.clone(),
                ))
            }
            TSTypeName::QualifiedName(qualified_name) => self.ctx.ast.static_member_expression(
                SPAN,
                self.transform_ts_type_name(&mut qualified_name.left),
                qualified_name.right.clone(),
                false,
            ),
        }
    }

    /// ```TypeScript
    /// import b = babel;
    /// import AliasModule = LongNameModule;
    ///
    /// ```JavaScript
    /// var b = babel;
    /// var AliasModule = LongNameModule;
    /// ```
    pub fn transform_ts_import_equals(
        &self,
        decl: &mut Box<'a, TSImportEqualsDeclaration<'a>>,
    ) -> Declaration<'a> {
        let kind = VariableDeclarationKind::Var;
        let decls = {
            let binding_identifier = BindingIdentifier::new(SPAN, decl.id.name.clone());
            let binding_pattern_kind = self.ctx.ast.binding_pattern_identifier(binding_identifier);
            let binding = self.ctx.ast.binding_pattern(binding_pattern_kind, None, false);
            let decl_span = decl.span;

            let init = match &mut decl.module_reference {
                type_name @ match_ts_type_name!(TSModuleReference) => {
                    self.transform_ts_type_name(&mut *type_name.to_ts_type_name_mut())
                }
                TSModuleReference::ExternalModuleReference(reference) => {
                    if self.ctx.source_type.is_module() {
                        self.ctx.error(ImportEqualsRequireUnsupported(decl_span));
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
        let variable_declaration =
            self.ctx.ast.variable_declaration(SPAN, kind, decls, Modifiers::empty());

        Declaration::VariableDeclaration(variable_declaration)
    }

    pub fn transform_ts_export_assignment(
        &mut self,
        export_assignment: &mut TSExportAssignment<'a>,
    ) {
        if self.ctx.source_type.is_module() {
            self.ctx.error(ExportAssignmentUnsupported(export_assignment.span));
        }
    }
}
