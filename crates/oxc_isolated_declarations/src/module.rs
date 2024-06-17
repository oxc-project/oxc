#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use oxc_allocator::Box;
use oxc_ast::Visit;
use oxc_span::{GetSpan, SPAN};

use crate::IsolatedDeclarations;

impl<'a> IsolatedDeclarations<'a> {
    pub fn transform_export_named_declaration(
        &mut self,
        decl: &ExportNamedDeclaration<'a>,
    ) -> Option<ExportNamedDeclaration<'a>> {
        let decl = self.transform_declaration(decl.declaration.as_ref()?, false)?;

        Some(ExportNamedDeclaration {
            span: decl.span(),
            declaration: Some(decl),
            specifiers: self.ctx.ast.new_vec(),
            source: None,
            export_kind: ImportOrExportKind::Value,
            with_clause: None,
        })
    }

    pub fn transform_export_default_declaration(
        &mut self,
        decl: &ExportDefaultDeclaration<'a>,
    ) -> Option<(Option<VariableDeclaration<'a>>, ExportDefaultDeclaration<'a>)> {
        let declaration = match &decl.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(decl) => self
                .transform_function(decl)
                .map(|d| (None, ExportDefaultDeclarationKind::FunctionDeclaration(d))),
            ExportDefaultDeclarationKind::ClassDeclaration(decl) => self
                .transform_class(decl)
                .map(|d| (None, ExportDefaultDeclarationKind::ClassDeclaration(d))),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(interface_decl) => {
                self.visit_ts_interface_declaration(interface_decl);
                Some((None, self.ctx.ast.copy(&decl.declaration)))
            }
            expr @ match_expression!(ExportDefaultDeclarationKind) => {
                let expr = expr.to_expression();
                if matches!(expr, Expression::Identifier(_)) {
                    None
                } else {
                    // declare const _default: Type
                    let kind = VariableDeclarationKind::Const;
                    let name = self.ctx.ast.new_atom("_default");
                    let id = self
                        .ctx
                        .ast
                        .binding_pattern_identifier(BindingIdentifier::new(SPAN, name.clone()));
                    let type_annotation = self
                        .infer_type_from_expression(expr)
                        .map(|ts_type| self.ctx.ast.ts_type_annotation(SPAN, ts_type));

                    let id = BindingPattern { kind: id, type_annotation, optional: false };
                    let declarations = self.ctx.ast.new_vec_single(
                        self.ctx.ast.variable_declarator(SPAN, kind, id, None, true),
                    );

                    Some((
                        Some(VariableDeclaration {
                            span: SPAN,
                            kind,
                            declarations,
                            modifiers: self.modifiers_declare(),
                        }),
                        ExportDefaultDeclarationKind::from(
                            self.ctx.ast.identifier_reference_expression(
                                self.ctx.ast.identifier_reference(SPAN, &name),
                            ),
                        ),
                    ))
                }
            }
        };

        declaration.map(|(var_decl, declaration)| {
            let exported = ModuleExportName::Identifier(IdentifierName::new(
                SPAN,
                self.ctx.ast.new_atom("default"),
            ));
            (var_decl, ExportDefaultDeclaration { span: decl.span, declaration, exported })
        })
    }

    pub fn transform_import_declaration(
        &self,
        decl: &ImportDeclaration<'a>,
    ) -> Option<Box<'a, ImportDeclaration<'a>>> {
        let specifiers = decl.specifiers.as_ref()?;

        let mut specifiers = self.ctx.ast.copy(specifiers);
        specifiers.retain(|specifier| match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                self.scope.has_reference(&specifier.local.name)
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                self.scope.has_reference(&specifier.local.name)
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                self.scope.has_reference(&self.ctx.ast.new_atom(&specifier.name()))
            }
        });
        if specifiers.is_empty() {
            // We don't need to print this import statement
            None
        } else {
            Some(self.ctx.ast.import_declaration(
                decl.span,
                Some(specifiers),
                self.ctx.ast.copy(&decl.source),
                self.ctx.ast.copy(&decl.with_clause),
                decl.import_kind,
            ))
        }
    }
}
