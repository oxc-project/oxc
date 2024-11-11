use oxc_allocator::Box;
use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{Atom, GetSpan, SPAN};

use crate::{diagnostics::default_export_inferred, IsolatedDeclarations};

impl<'a> IsolatedDeclarations<'a> {
    pub(crate) fn transform_export_named_declaration(
        &mut self,
        prev_decl: &ExportNamedDeclaration<'a>,
    ) -> Option<ExportNamedDeclaration<'a>> {
        let decl = self.transform_declaration(prev_decl.declaration.as_ref()?, false)?;

        Some(self.ast.export_named_declaration(
            prev_decl.span,
            Some(decl),
            self.ast.vec(),
            None,
            ImportOrExportKind::Value,
            None::<WithClause>,
        ))
    }

    pub(crate) fn create_unique_name(&mut self, name: &str) -> Atom<'a> {
        let mut binding = self.ast.atom(name);
        let mut i = 1;
        while self.scope.has_reference(&binding) {
            binding = self.ast.atom(format!("{name}_{i}").as_str());
            i += 1;
        }
        binding
    }

    pub(crate) fn transform_export_default_declaration(
        &mut self,
        decl: &ExportDefaultDeclaration<'a>,
    ) -> Option<(Option<Statement<'a>>, Statement<'a>)> {
        let declaration = match &decl.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(decl) => self
                .transform_function(decl, Some(false))
                .map(|d| (None, ExportDefaultDeclarationKind::FunctionDeclaration(d))),
            ExportDefaultDeclarationKind::ClassDeclaration(decl) => self
                .transform_class(decl, Some(false))
                .map(|d| (None, ExportDefaultDeclarationKind::ClassDeclaration(d))),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => {
                // SAFETY: `ast.copy` is unsound! We need to fix.
                Some((None, unsafe { self.ast.copy(&decl.declaration) }))
            }
            declaration @ match_expression!(ExportDefaultDeclarationKind) => self
                .transform_export_expression(declaration.to_expression())
                .map(|(var_decl, expr)| (var_decl, ExportDefaultDeclarationKind::from(expr))),
        };

        declaration.map(|(var_decl, declaration)| {
            let exported =
                ModuleExportName::IdentifierName(self.ast.identifier_name(SPAN, "default"));
            let declaration = self.ast.module_declaration_export_default_declaration(
                decl.span,
                declaration,
                exported,
            );
            (var_decl, Statement::from(declaration))
        })
    }

    fn transform_export_expression(
        &mut self,
        expr: &Expression<'a>,
    ) -> Option<(Option<Statement<'a>>, Expression<'a>)> {
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
            let declarations =
                self.ast.vec1(self.ast.variable_declarator(SPAN, kind, id, None, false));

            let variable_statement = Statement::from(self.ast.declaration_variable(
                SPAN,
                kind,
                declarations,
                self.is_declare(),
            ));
            Some((Some(variable_statement), self.ast.expression_identifier_reference(SPAN, &name)))
        }
    }

    pub(crate) fn transform_ts_export_assignment(
        &mut self,
        decl: &TSExportAssignment<'a>,
    ) -> Option<(Option<Statement<'a>>, Statement<'a>)> {
        self.transform_export_expression(&decl.expression).map(|(var_decl, expr)| {
            (
                var_decl,
                Statement::from(self.ast.module_declaration_ts_export_assignment(decl.span, expr)),
            )
        })
    }

    pub(crate) fn transform_import_declaration(
        &self,
        decl: &ImportDeclaration<'a>,
    ) -> Option<Box<'a, ImportDeclaration<'a>>> {
        let specifiers = decl.specifiers.as_ref()?;

        // SAFETY: `ast.copy` is unsound! We need to fix.
        let mut specifiers = unsafe { self.ast.copy(specifiers) };
        specifiers.retain(|specifier| match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                self.scope.has_reference(&specifier.local.name)
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                self.scope.has_reference(&specifier.local.name)
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                self.scope.has_reference(&specifier.name())
            }
        });
        if specifiers.is_empty() {
            // We don't need to print this import statement
            None
        } else {
            Some(self.ast.alloc_import_declaration(
                decl.span,
                Some(specifiers),
                // SAFETY: `ast.copy` is unsound! We need to fix.
                unsafe { self.ast.copy(&decl.source) },
                // SAFETY: `ast.copy` is unsound! We need to fix.
                unsafe { self.ast.copy(&decl.with_clause) },
                decl.import_kind,
            ))
        }
    }

    /// Strip export keyword from ExportNamedDeclaration
    ///
    /// ```ts
    /// export const a = 1;
    /// export function b() {}
    /// ```
    /// to
    /// ```ts
    /// const a = 1;
    /// function b() {}
    /// ```
    pub(crate) fn strip_export_keyword(&self, stmts: &mut Vec<'a, Statement<'a>>) {
        stmts.iter_mut().for_each(|stmt| {
            if let Statement::ExportNamedDeclaration(decl) = stmt {
                if let Some(declaration) = &mut decl.declaration {
                    *stmt = Statement::from(self.ast.move_declaration(declaration));
                }
            }
        });
    }
}
