use oxc_allocator::{Box as ArenaBox, CloneIn, TakeIn, Vec as ArenaVec};
use oxc_ast::{NONE, ast::*};
use oxc_span::{Atom, GetSpan, SPAN};

use crate::{IsolatedDeclarations, diagnostics::default_export_inferred};

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
            NONE,
        ))
    }

    pub(crate) fn create_unique_name(&self, name: &str) -> Atom<'a> {
        let mut binding = self.ast.atom(name);
        let mut i = 1;
        while self.scope.has_reference(&binding) {
            binding = self.ast.atom(format!("{name}_{i}").as_str());
            i += 1;
        }
        binding
    }

    pub(crate) fn transform_export_default_declaration(
        &self,
        decl: &ExportDefaultDeclaration<'a>,
    ) -> Option<(Option<Statement<'a>>, Statement<'a>)> {
        let declaration = match &decl.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(decl) => Some((
                None,
                ExportDefaultDeclarationKind::FunctionDeclaration(
                    self.transform_function(decl, Some(false)),
                ),
            )),
            ExportDefaultDeclarationKind::ClassDeclaration(decl) => Some((
                None,
                ExportDefaultDeclarationKind::ClassDeclaration(
                    self.transform_class(decl, Some(false)),
                ),
            )),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => {
                Some((None, decl.declaration.clone_in(self.ast.allocator)))
            }
            declaration @ match_expression!(ExportDefaultDeclarationKind) => self
                .transform_export_expression(decl.span, declaration.to_expression())
                .map(|(var_decl, expr)| (var_decl, ExportDefaultDeclarationKind::from(expr))),
        };

        declaration.map(|(var_decl, declaration)| {
            // When `var_decl` is Some, the comments are moved to the variable declaration, otherwise
            // keep the comments on the export default declaration to avoid losing them.
            // ```ts
            // // comment
            // export default function(): void {}
            //
            // // comment
            // export default 1;
            // ```
            //
            // to
            //
            // ```ts
            // // comment
            // export default function(): void;
            //
            // // comment
            // const _default = 1;
            // export default _default;
            // ```

            let span = if var_decl.is_some() { SPAN } else { decl.span };
            let declaration =
                self.ast.module_declaration_export_default_declaration(span, declaration);
            (var_decl, Statement::from(declaration))
        })
    }

    fn transform_export_expression(
        &self,
        decl_span: Span,
        expr: &Expression<'a>,
    ) -> Option<(Option<Statement<'a>>, Expression<'a>)> {
        if matches!(expr, Expression::Identifier(_)) {
            None
        } else {
            // declare const _default: Type
            let kind = VariableDeclarationKind::Const;
            let name = self.create_unique_name("_default");
            let id = self.ast.binding_pattern_kind_binding_identifier(SPAN, name);
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
                decl_span,
                kind,
                declarations,
                self.is_declare(),
            ));
            Some((Some(variable_statement), self.ast.expression_identifier(SPAN, name)))
        }
    }

    pub(crate) fn transform_ts_export_assignment(
        &self,
        decl: &TSExportAssignment<'a>,
    ) -> Option<(Option<Statement<'a>>, Statement<'a>)> {
        self.transform_export_expression(decl.span, &decl.expression).map(|(var_decl, expr)| {
            (
                var_decl,
                Statement::from(self.ast.module_declaration_ts_export_assignment(SPAN, expr)),
            )
        })
    }

    pub(crate) fn transform_import_declaration(
        &self,
        decl: &ImportDeclaration<'a>,
    ) -> Option<ArenaBox<'a, ImportDeclaration<'a>>> {
        let specifiers = decl.specifiers.as_ref()?;

        let mut new_specifiers = self.ast.vec_with_capacity(specifiers.len());
        specifiers.iter().for_each(|specifier| {
            let is_referenced = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    self.scope.has_reference(&specifier.local.name)
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    self.scope.has_reference(&specifier.local.name)
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                    self.scope.has_reference(&specifier.name())
                }
            };
            if is_referenced {
                new_specifiers.push(specifier.clone_in(self.ast.allocator));
            }
        });

        if new_specifiers.is_empty() {
            // We don't need to print this import statement
            None
        } else {
            Some(self.ast.alloc_import_declaration(
                decl.span,
                Some(new_specifiers),
                decl.source.clone(),
                None,
                decl.with_clause.clone_in(self.ast.allocator),
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
    pub(crate) fn strip_export_keyword(&self, stmts: &mut ArenaVec<'a, Statement<'a>>) {
        stmts.iter_mut().for_each(|stmt| {
            if let Statement::ExportNamedDeclaration(decl) = stmt {
                if let Some(declaration) = &mut decl.declaration {
                    *stmt = Statement::from(declaration.take_in(self.ast));
                }
            }
        });
    }
}
