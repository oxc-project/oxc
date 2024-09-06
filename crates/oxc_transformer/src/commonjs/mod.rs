pub mod options;
mod types;
mod utils;

use crate::commonjs::options::CommonjsOptions;
use crate::commonjs::utils::export::{
    create_declared_named_exports, create_default_exports, create_export_star_exports,
    create_listed_named_exports, create_reexported_named_exports,
    create_renamed_export_star_exports,
};
use crate::context::Ctx;
use oxc_allocator::CloneIn;
use oxc_ast::ast::{
    BindingPattern, ExportAllDeclaration, ExportDefaultDeclaration, ExportDefaultDeclarationKind,
    ExportNamedDeclaration, Expression, ImportDeclaration, ImportDeclarationSpecifier,
    ModuleExportName, Program, PropertyKey, Statement, TSTypeAnnotation,
};
use oxc_span::SPAN;
use oxc_traverse::{Traverse, TraverseCtx};
use utils::import;

pub struct Commonjs<'a> {
    ctx: Ctx<'a>,
    options: CommonjsOptions,
}

impl<'a> Commonjs<'a> {
    pub fn new(options: CommonjsOptions, ctx: Ctx<'a>) -> Self {
        Self { ctx, options }
    }
}

#[allow(clippy::needless_pass_by_value)]
impl<'a> Commonjs<'a> {
    pub fn transform_import_declaration(
        &mut self,
        node: oxc_allocator::Box<ImportDeclaration<'a>>,
    ) -> Statement<'a> {
        match &node.specifiers {
            None => import::create_empty_require(node.source.value.as_str(), &self.ctx.ast)
                .clone_in(self.ctx.ast.allocator),
            Some(specifiers) => {
                let star_specifier = specifiers.iter().find(|specifier| {
                    matches!(specifier, ImportDeclarationSpecifier::ImportNamespaceSpecifier(_))
                });
                if let Some(specifier) = star_specifier {
                    import::create_namespaced_require(
                        &node.source.value,
                        match specifier {
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns) => {
                                ns.local.name.as_str()
                            }
                            _ => unreachable!(),
                        },
                        &self.ctx.ast,
                        false,
                    )
                    .clone_in(self.ctx.ast.allocator)
                } else {
                    let assignees: Vec<(PropertyKey<'a>, BindingPattern<'a>)> = specifiers
                        .iter()
                        .map(|specifier| match specifier {
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(decl) => (
                                self.ctx.ast.property_key_identifier_name(SPAN, "default"),
                                self.ctx.ast.binding_pattern(
                                    self.ctx.ast.binding_pattern_kind_binding_identifier(
                                        SPAN,
                                        &decl.local.name,
                                    ),
                                    None::<TSTypeAnnotation>,
                                    false,
                                ),
                            ),
                            ImportDeclarationSpecifier::ImportSpecifier(decl) => (
                                match &decl.imported {
                                    ModuleExportName::IdentifierName(name) => self
                                        .ctx
                                        .ast
                                        .property_key_identifier_name(SPAN, name.name.as_str()),
                                    ModuleExportName::StringLiteral(literal) => {
                                        self.ctx.ast.property_key_expression(
                                            self.ctx
                                                .ast
                                                .expression_string_literal(SPAN, &literal.value),
                                        )
                                    }
                                    ModuleExportName::IdentifierReference(_) => unreachable!(),
                                },
                                self.ctx.ast.binding_pattern(
                                    self.ctx.ast.binding_pattern_kind_binding_identifier(
                                        SPAN,
                                        &decl.local.name,
                                    ),
                                    None::<TSTypeAnnotation>,
                                    false,
                                ),
                            ),
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                                unreachable!()
                            }
                        })
                        .collect();
                    import::create_named_require(
                        &node.source.value,
                        assignees,
                        &self.ctx.ast,
                        false,
                    )
                    .clone_in(self.ctx.ast.allocator)
                }
            }
        }
    }

    pub fn transform_export_default_declaration(
        &mut self,
        node: oxc_allocator::Box<ExportDefaultDeclaration<'a>>,
    ) -> Statement<'a> {
        let expr = node.declaration.clone_in(self.ctx.ast.allocator);

        let expression: Expression<'a> = match expr {
            ExportDefaultDeclarationKind::FunctionDeclaration(decl) => {
                self.ctx.ast.expression_from_function(decl)
            }
            ExportDefaultDeclarationKind::ClassDeclaration(decl) => {
                self.ctx.ast.expression_from_class(decl)
            }
            _ => expr.into_expression(),
        };
        create_default_exports(expression, &self.ctx.ast).clone_in(self.ctx.ast.allocator)
    }

    pub fn transform_export_named_declaration(
        &mut self,
        node: oxc_allocator::Box<ExportNamedDeclaration<'a>>,
    ) -> oxc_allocator::Vec<'a, Statement<'a>> {
        if let Some(decl) = &node.declaration {
            create_declared_named_exports(decl.clone_in(self.ctx.ast.allocator), &self.ctx.ast)
                .clone_in(self.ctx.ast.allocator)
        } else if let Some(src) = &node.source {
            let specifiers = node.specifiers.clone_in(self.ctx.ast.allocator);
            create_reexported_named_exports(specifiers, src.value.as_str(), &self.ctx.ast)
                .clone_in(self.ctx.ast.allocator)
        } else {
            let specifiers = node.specifiers.clone_in(self.ctx.ast.allocator);
            create_listed_named_exports(specifiers, &self.ctx.ast).clone_in(self.ctx.ast.allocator)
        }
    }

    pub fn transform_export_all_declaration(
        &mut self,
        node: oxc_allocator::Box<ExportAllDeclaration<'a>>,
    ) -> oxc_allocator::Vec<'a, Statement<'a>> {
        if let Some(reexported) = &node.exported {
            create_renamed_export_star_exports(
                node.source.value.as_str(),
                reexported.clone_in(self.ctx.ast.allocator),
                &self.ctx.ast,
            )
            .clone_in(self.ctx.ast.allocator)
        } else {
            create_export_star_exports(node.source.value.as_str(), &self.ctx.ast)
                .clone_in(self.ctx.ast.allocator)
        }
    }
}

impl<'a> Traverse<'a> for Commonjs<'a> {
    fn exit_program(&mut self, program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        if !self.options.transform_import_and_export {
            return;
        }
        let mut latest = self.ctx.ast.vec();

        for stmt in self.ctx.ast.move_vec(&mut program.body) {
            match stmt {
                Statement::ImportDeclaration(s) => {
                    latest.push(self.transform_import_declaration(s));
                }
                Statement::ExportNamedDeclaration(s) => {
                    let results = self.transform_export_named_declaration(s);
                    latest.extend(results);
                }
                Statement::ExportDefaultDeclaration(s) => {
                    latest.push(self.transform_export_default_declaration(s));
                }
                Statement::ExportAllDeclaration(s) => {
                    let results = self.transform_export_all_declaration(s);
                    latest.extend(results);
                }
                _ => {
                    latest.push(stmt);
                }
            }
        }

        program.body = latest;
    }
}
