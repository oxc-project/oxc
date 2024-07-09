use oxc_allocator::Box;
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_span::{Atom, GetSpan, SPAN};

use crate::{diagnostics::default_export_inferred, IsolatedDeclarations};

impl<'a> IsolatedDeclarations<'a> {
    pub fn transform_export_named_declaration(
        &mut self,
        decl: &ExportNamedDeclaration<'a>,
    ) -> Option<ExportNamedDeclaration<'a>> {
        let decl = self.transform_declaration(decl.declaration.as_ref()?, false)?;

        Some(ExportNamedDeclaration {
            span: decl.span(),
            declaration: Some(decl),
            specifiers: self.ast.new_vec(),
            source: None,
            export_kind: ImportOrExportKind::Value,
            with_clause: None,
        })
    }

    pub fn create_unique_name(&mut self, name: &str) -> Atom<'a> {
        let mut binding = self.ast.new_atom(name);
        let mut i = 1;
        while self.scope.has_reference(&binding) {
            binding = self.ast.new_atom(format!("{name}_{i}").as_str());
            i += 1;
        }
        binding
    }

    pub fn transform_export_default_declaration(
        &mut self,
        decl: &ExportDefaultDeclaration<'a>,
    ) -> Option<(Option<VariableDeclaration<'a>>, ExportDefaultDeclaration<'a>)> {
        let declaration = match &decl.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(decl) => self
                .transform_function(decl, Some(false))
                .map(|d| (None, ExportDefaultDeclarationKind::FunctionDeclaration(d))),
            ExportDefaultDeclarationKind::ClassDeclaration(decl) => self
                .transform_class(decl, Some(false))
                .map(|d| (None, ExportDefaultDeclarationKind::ClassDeclaration(d))),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => {
                Some((None, self.ast.copy(&decl.declaration)))
            }
            expr @ match_expression!(ExportDefaultDeclarationKind) => {
                let expr = expr.to_expression();
                if matches!(expr, Expression::Identifier(_)) {
                    None
                } else {
                    // declare const _default: Type
                    let kind = VariableDeclarationKind::Const;
                    let name = self.create_unique_name("_default");
                    let id = self.ast.binding_pattern_kind_binding_identifier(SPAN, &name);
                    let type_annotation = self
                        .infer_type_from_expression(expr)
                        .map(|ts_type| self.ast.ts_type_annotation(SPAN, ts_type));

                    if type_annotation.is_none() {
                        self.error(default_export_inferred(expr.span()));
                    }

                    let id = self.ast.binding_pattern(id, type_annotation, false);
                    let declarations = self
                        .ast
                        .new_vec_single(self.ast.variable_declarator(SPAN, kind, id, None, true));

                    Some((
                        Some(VariableDeclaration {
                            span: SPAN,
                            kind,
                            declarations,
                            declare: self.is_declare(),
                        }),
                        ExportDefaultDeclarationKind::from(
                            self.ast.expression_identifier_reference(SPAN, &name),
                        ),
                    ))
                }
            }
        };

        declaration.map(|(var_decl, declaration)| {
            let exported = ModuleExportName::IdentifierName(IdentifierName::new(
                SPAN,
                self.ast.new_atom("default"),
            ));
            (var_decl, ExportDefaultDeclaration { span: decl.span, declaration, exported })
        })
    }

    pub fn transform_import_declaration(
        &self,
        decl: &ImportDeclaration<'a>,
    ) -> Option<Box<'a, ImportDeclaration<'a>>> {
        let specifiers = decl.specifiers.as_ref()?;

        let mut specifiers = self.ast.copy(specifiers);
        specifiers.retain(|specifier| match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                self.scope.has_reference(&specifier.local.name)
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                self.scope.has_reference(&specifier.local.name)
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                self.scope.has_reference(specifier.name().as_str())
            }
        });
        if specifiers.is_empty() {
            // We don't need to print this import statement
            None
        } else {
            Some(self.ast.alloc_import_declaration(
                decl.span,
                Some(specifiers),
                self.ast.copy(&decl.source),
                self.ast.copy(&decl.with_clause),
                decl.import_kind,
            ))
        }
    }
}
