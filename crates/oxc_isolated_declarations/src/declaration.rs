#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use oxc_allocator::Box;
use oxc_ast::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, SPAN};
use oxc_syntax::scope::ScopeFlags;

use crate::{diagnostics::signature_computed_property_name, IsolatedDeclarations};

impl<'a> IsolatedDeclarations<'a> {
    pub fn transform_variable_declaration(
        &self,
        decl: &VariableDeclaration<'a>,
        check_binding: bool,
    ) -> Option<Box<'a, VariableDeclaration<'a>>> {
        if decl.modifiers.is_contains_declare() {
            None
        } else {
            let declarations =
                self.ast.new_vec_from_iter(decl.declarations.iter().filter_map(|declarator| {
                    self.transform_variable_declarator(declarator, check_binding)
                }));
            Some(self.transform_variable_declaration_with_new_declarations(decl, declarations))
        }
    }

    pub fn transform_variable_declaration_with_new_declarations(
        &self,
        decl: &VariableDeclaration<'a>,
        declarations: oxc_allocator::Vec<'a, VariableDeclarator<'a>>,
    ) -> Box<'a, VariableDeclaration<'a>> {
        self.ast.variable_declaration(
            decl.span,
            decl.kind,
            self.ast.new_vec_from_iter(declarations),
            self.modifiers_declare(),
        )
    }

    pub fn transform_variable_declarator(
        &self,
        decl: &VariableDeclarator<'a>,
        check_binding: bool,
    ) -> Option<VariableDeclarator<'a>> {
        if decl.id.kind.is_destructuring_pattern() {
            self.error(OxcDiagnostic::error(
                "Binding elements can't be exported directly with --isolatedDeclarations.",
            ));
            return None;
        }

        if check_binding {
            if let Some(name) = decl.id.get_identifier() {
                if !self.scope.has_reference(name) {
                    return None;
                }
            }
        }

        let mut binding_type = None;
        let mut init = None;
        if decl.id.type_annotation.is_none() {
            if let Some(init_expr) = &decl.init {
                // if kind is const and it doesn't need to infer type from expression
                if decl.kind.is_const() && !Self::is_need_to_infer_type_from_expression(init_expr) {
                    init = Some(self.ast.copy(init_expr));
                } else {
                    // otherwise, we need to infer type from expression
                    binding_type = self.infer_type_from_expression(init_expr);
                }
            }
            if init.is_none() && binding_type.is_none() {
                binding_type = Some(self.ast.ts_unknown_keyword(SPAN));
                self.error(
                  OxcDiagnostic::error("Variable must have an explicit type annotation with --isolatedDeclarations.")
                      .with_label(decl.id.span()),
              );
            }
        }
        let id = binding_type.map_or_else(
            || self.ast.copy(&decl.id),
            |ts_type| {
                self.ast.binding_pattern(
                    self.ast.copy(&decl.id.kind),
                    Some(self.ast.ts_type_annotation(SPAN, ts_type)),
                    decl.id.optional,
                )
            },
        );

        Some(self.ast.variable_declarator(decl.span, decl.kind, id, init, decl.definite))
    }

    pub fn transform_using_declaration(
        &self,
        decl: &UsingDeclaration<'a>,
        check_binding: bool,
    ) -> Box<'a, VariableDeclaration<'a>> {
        let declarations =
            self.ast.new_vec_from_iter(decl.declarations.iter().filter_map(|declarator| {
                self.transform_variable_declarator(declarator, check_binding)
            }));
        self.transform_using_declaration_with_new_declarations(decl, declarations)
    }

    pub fn transform_using_declaration_with_new_declarations(
        &self,
        decl: &UsingDeclaration<'a>,
        declarations: oxc_allocator::Vec<'a, VariableDeclarator<'a>>,
    ) -> Box<'a, VariableDeclaration<'a>> {
        self.ast.variable_declaration(
            decl.span,
            VariableDeclarationKind::Const,
            declarations,
            self.modifiers_declare(),
        )
    }

    fn transform_ts_module_block(
        &mut self,
        block: &Box<'a, TSModuleBlock<'a>>,
    ) -> Box<'a, TSModuleBlock<'a>> {
        // We need to enter a new scope for the module block, avoid add binding to the parent scope
        self.scope.enter_scope(ScopeFlags::TsModuleBlock);
        let stmts = self.transform_statements_on_demand(&block.body);
        self.scope.leave_scope();
        self.ast.ts_module_block(SPAN, stmts)
    }

    pub fn transform_ts_module_declaration(
        &mut self,
        decl: &Box<'a, TSModuleDeclaration<'a>>,
    ) -> Box<'a, TSModuleDeclaration<'a>> {
        if decl.modifiers.is_contains_declare() {
            return self.ast.copy(decl);
        }

        let Some(body) = &decl.body else {
            return self.ast.copy(decl);
        };

        match body {
            TSModuleDeclarationBody::TSModuleDeclaration(decl) => {
                let inner = self.transform_ts_module_declaration(decl);
                return self.ast.ts_module_declaration(
                    decl.span,
                    self.ast.copy(&decl.id),
                    Some(TSModuleDeclarationBody::TSModuleDeclaration(inner)),
                    decl.kind,
                    self.modifiers_declare(),
                );
            }
            TSModuleDeclarationBody::TSModuleBlock(block) => {
                let body = self.transform_ts_module_block(block);
                return self.ast.ts_module_declaration(
                    decl.span,
                    self.ast.copy(&decl.id),
                    Some(TSModuleDeclarationBody::TSModuleBlock(body)),
                    decl.kind,
                    self.modifiers_declare(),
                );
            }
        }
    }

    pub fn transform_declaration(
        &mut self,
        decl: &Declaration<'a>,
        check_binding: bool,
    ) -> Option<Declaration<'a>> {
        match decl {
            Declaration::FunctionDeclaration(func) => {
                if !check_binding
                    || func.id.as_ref().is_some_and(|id| self.scope.has_reference(&id.name))
                {
                    self.transform_function(func).map(Declaration::FunctionDeclaration)
                } else {
                    None
                }
            }
            Declaration::VariableDeclaration(decl) => self
                .transform_variable_declaration(decl, check_binding)
                .map(Declaration::VariableDeclaration),
            Declaration::UsingDeclaration(decl) => Some(Declaration::VariableDeclaration(
                self.transform_using_declaration(decl, check_binding),
            )),
            Declaration::ClassDeclaration(decl) => {
                if !check_binding
                    || decl.id.as_ref().is_some_and(|id| self.scope.has_reference(&id.name))
                {
                    self.transform_class(decl).map(Declaration::ClassDeclaration)
                } else {
                    None
                }
            }
            Declaration::TSTypeAliasDeclaration(alias_decl) => {
                self.visit_ts_type_alias_declaration(alias_decl);
                if !check_binding || self.scope.has_reference(&alias_decl.id.name) {
                    Some(self.ast.copy(decl))
                } else {
                    None
                }
            }
            Declaration::TSInterfaceDeclaration(interface_decl) => {
                self.visit_ts_interface_declaration(interface_decl);
                if !check_binding || self.scope.has_reference(&interface_decl.id.name) {
                    Some(self.ast.copy(decl))
                } else {
                    None
                }
            }
            Declaration::TSEnumDeclaration(enum_decl) => {
                if !check_binding || self.scope.has_reference(&enum_decl.id.name) {
                    self.transform_ts_enum_declaration(enum_decl)
                } else {
                    None
                }
            }
            Declaration::TSModuleDeclaration(decl) => {
                if !check_binding
                    || matches!(
                        &decl.id,
                        TSModuleDeclarationName::Identifier(ident)
                            if self.scope.has_reference(&ident.name)
                    )
                {
                    Some(Declaration::TSModuleDeclaration(
                        self.transform_ts_module_declaration(decl),
                    ))
                } else {
                    None
                }
            }
            Declaration::TSImportEqualsDeclaration(decl) => {
                if !check_binding || self.scope.has_reference(&decl.id.name) {
                    Some(Declaration::TSImportEqualsDeclaration(self.ast.copy(decl)))
                } else {
                    None
                }
            }
        }
    }

    fn report_signature_property_key(&self, key: &PropertyKey<'a>, computed: bool) {
        if !computed {
            return;
        }

        let is_not_allowed = match key {
            PropertyKey::StaticIdentifier(_) | PropertyKey::Identifier(_) => false,
            PropertyKey::StaticMemberExpression(expr) => {
                !expr.get_first_object().is_identifier_reference()
            }
            key => !self.is_literal_key(key),
        };

        if is_not_allowed {
            self.error(signature_computed_property_name(key.span()));
        }
    }
}

impl<'a> Visit<'a> for IsolatedDeclarations<'a> {
    fn visit_ts_method_signature(&mut self, signature: &TSMethodSignature<'a>) {
        self.report_signature_property_key(&signature.key, signature.computed);
    }
    fn visit_ts_property_signature(&mut self, signature: &TSPropertySignature<'a>) {
        self.report_signature_property_key(&signature.key, signature.computed);
    }
}
