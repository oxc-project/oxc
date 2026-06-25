//! Rewrite import extensions
//!
//! This plugin is used to rewrite/remove extensions from import/export source.
//! It is only handled source that contains `/` or `\` in the source.
//!
//! Based on Babel's [plugin-rewrite-ts-imports](https://github.com/babel/babel/blob/3bcfee232506a4cebe410f02042fb0f0adeeb0b1/packages/babel-preset-typescript/src/plugin-rewrite-ts-imports.ts)

use oxc_ast::ast::{
    ExportAllDeclaration, ExportNamedDeclaration, Expression, ImportDeclaration, ImportExpression,
    StringLiteral, TemplateLiteral,
};
use oxc_str::Str;
use oxc_traverse::Traverse;

use crate::{TypeScriptOptions, context::TraverseCtx, state::TransformState};

use super::options::RewriteExtensionsMode;

pub struct TypeScriptRewriteExtensions {
    mode: RewriteExtensionsMode,
}

/// Given a specifier value, compute the replacement `Str` if the extension
/// should be rewritten/removed. Returns `None` when no rewriting is needed.
fn rewritten_specifier<'a>(
    value: &'a str,
    mode: RewriteExtensionsMode,
    ctx: &TraverseCtx<'a>,
) -> Option<Str<'a>> {
    if !value.contains(['/', '\\']) {
        return None;
    }

    let (without_extension, extension) = value.rsplit_once('.')?;

    let replace = match extension {
        "mts" => ".mjs",
        "cts" => ".cjs",
        "ts" | "tsx" => ".js",
        _ => return None,
    };

    Some(if mode.is_remove() {
        Str::from(without_extension)
    } else {
        ctx.ast.str_from_strs_array([without_extension, replace])
    })
}

impl TypeScriptRewriteExtensions {
    pub fn new(options: &TypeScriptOptions) -> Option<Self> {
        options.rewrite_import_extensions.map(|mode| Self { mode })
    }

    pub fn rewrite_extensions<'a>(&self, source: &mut StringLiteral<'a>, ctx: &TraverseCtx<'a>) {
        if let Some(rewritten) = rewritten_specifier(source.value.as_str(), self.mode, ctx) {
            source.value = rewritten;
            source.raw = None;
        }
    }

    fn rewrite_template_literal<'a>(
        &self,
        template: &mut TemplateLiteral<'a>,
        ctx: &TraverseCtx<'a>,
    ) {
        if !template.is_no_substitution_template() {
            return;
        }
        let quasi = &mut template.quasis[0];
        // Read the specifier value from raw (always present).
        // For no-substitution templates, raw and cooked are identical
        // unless the template contains escape sequences, which import
        // specifiers never do.
        if let Some(rewritten) = rewritten_specifier(quasi.value.raw.as_str(), self.mode, ctx) {
            quasi.value.raw = rewritten;
            quasi.value.cooked = Some(rewritten);
        }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for TypeScriptRewriteExtensions {
    fn enter_import_declaration(
        &mut self,
        node: &mut ImportDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if node.import_kind.is_type() {
            return;
        }
        self.rewrite_extensions(&mut node.source, ctx);
    }

    fn enter_export_named_declaration(
        &mut self,
        node: &mut ExportNamedDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if node.export_kind.is_type() {
            return;
        }
        if let Some(source) = node.source.as_mut() {
            self.rewrite_extensions(source, ctx);
        }
    }

    fn enter_export_all_declaration(
        &mut self,
        node: &mut ExportAllDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if node.export_kind.is_type() {
            return;
        }
        self.rewrite_extensions(&mut node.source, ctx);
    }

    fn enter_import_expression(
        &mut self,
        node: &mut ImportExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match &mut node.source {
            Expression::StringLiteral(source) => {
                self.rewrite_extensions(source, ctx);
            }
            Expression::TemplateLiteral(template) => {
                self.rewrite_template_literal(template, ctx);
            }
            _ => {}
        }
    }
}
