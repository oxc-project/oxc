mod options;
mod types;
mod utils;

use crate::commonjs::options::CommonjsOptions;
use crate::context::Ctx;
use oxc_ast::ast::{
    BindingPattern, ImportDeclaration, ImportDeclarationSpecifier, ModuleExportName, PropertyKey,
    TSTypeAnnotation, TSTypeParameterInstantiation, VariableDeclarationKind,
};
use oxc_span::{Atom, SPAN};
use oxc_traverse::{Traverse, TraverseCtx};

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
            None => utils::create_empty_require(&node.source.value, &self.ctx.ast),
            Some(specifiers) => {
                let star_specifier = specifiers.iter().find(|specifier| {
                    matches!(specifier, ImportDeclarationSpecifier::ImportNamespaceSpecifier(_))
                });
                if let Some(specifier) = star_specifier {
                    utils::create_namespaced_require(
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
                                    _ => unreachable!(),
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
                            _ => unreachable!(),
                        })
                        .collect();
                    utils::create_general_require(
                        &node.source.value,
                        assignees,
                        &self.ctx.ast,
                        false,
                    )
                }
            }
        };
    }
}
