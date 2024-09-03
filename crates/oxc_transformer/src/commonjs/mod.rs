mod options;
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
    BindingPattern, ExportAllDeclaration, ExportDefaultDeclaration, ExportNamedDeclaration,
    ImportDeclaration, ImportDeclarationSpecifier, ModuleExportName, PropertyKey, TSTypeAnnotation,
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

impl<'a> Traverse<'a> for Commonjs<'a> {
    fn enter_import_declaration(
        &mut self,
        node: &mut ImportDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let stmt = match &node.specifiers {
            None => import::create_empty_require(&node.source.value, &self.ctx.ast),
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
                }
            }
        };
    }

    fn enter_export_default_declaration(
        &mut self,
        node: &mut ExportDefaultDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let expr = node.declaration.clone_in(self.ctx.ast.allocator);
        let stmt = create_default_exports(expr.into_expression(), &self.ctx.ast);
    }

    fn enter_export_named_declaration(
        &mut self,
        node: &mut ExportNamedDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let stmt = if let Some(decl) = &node.declaration {
            create_declared_named_exports(decl.clone_in(self.ctx.ast.allocator), &self.ctx.ast)
        } else if let Some(src) = &node.source {
            let specifiers = node.specifiers.clone_in(self.ctx.ast.allocator);
            create_reexported_named_exports(specifiers, src.value.as_str(), &self.ctx.ast)
        } else {
            let specifiers = node.specifiers.clone_in(self.ctx.ast.allocator);
            create_listed_named_exports(specifiers, &self.ctx.ast)
        };
    }

    fn enter_export_all_declaration(
        &mut self,
        node: &mut ExportAllDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let stmt = if let Some(reexported) = &node.exported {
            create_renamed_export_star_exports(
                node.source.value.as_str(),
                reexported.clone_in(self.ctx.ast.allocator),
                &self.ctx.ast,
            )
        } else {
            create_export_star_exports(node.source.value.as_str(), &self.ctx.ast)
        };
    }
}
