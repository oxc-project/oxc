use oxc_allocator::Box;
use oxc_ast::ast::*;
use oxc_span::SPAN;

use super::TypeScript;

impl<'a> TypeScript<'a> {
    fn transform_ts_type_name(&self, type_name: &mut TSTypeName<'a>) -> Expression<'a> {
        match type_name {
            TSTypeName::IdentifierReference(reference) => {
                self.ctx.borrow().ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    reference.name.clone(),
                ))
            }
            TSTypeName::QualifiedName(qualified_name) => {
                self.ctx.borrow().ast.static_member_expression(
                    SPAN,
                    self.transform_ts_type_name(&mut qualified_name.left),
                    qualified_name.right.clone(),
                    false,
                )
            }
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
            let binding_pattern_kind =
                self.ctx.borrow().ast.binding_pattern_identifier(binding_identifier);
            let binding = self.ctx.borrow().ast.binding_pattern(binding_pattern_kind, None, false);

            let init = match &mut *decl.module_reference {
                TSModuleReference::TypeName(type_name) => self.transform_ts_type_name(type_name),
                TSModuleReference::ExternalModuleReference(reference) => {
                    let callee = self.ctx.borrow().ast.identifier_reference_expression(
                        IdentifierReference::new(SPAN, "require".into()),
                    );
                    let arguments = self.ctx.borrow().ast.new_vec_single(Argument::Expression(
                        self.ctx
                            .borrow()
                            .ast
                            .literal_string_expression(reference.expression.clone()),
                    ));
                    self.ctx.borrow().ast.call_expression(SPAN, callee, arguments, false, None)
                }
            };
            self.ctx.borrow().ast.new_vec_single(self.ctx.borrow().ast.variable_declarator(
                SPAN,
                kind,
                binding,
                Some(init),
                false,
            ))
        };
        let variable_declaration =
            self.ctx.borrow().ast.variable_declaration(SPAN, kind, decls, Modifiers::empty());

        Declaration::VariableDeclaration(variable_declaration)
    }
}
