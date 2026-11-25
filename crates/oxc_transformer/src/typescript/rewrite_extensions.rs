//! Rewrite import extensions
//!
//! This plugin is used to rewrite/remove extensions from import/export source.
//! It is only handled source that contains `/` or `\` in the source.
//!
//! Based on Babel's [plugin-rewrite-ts-imports](https://github.com/babel/babel/blob/3bcfee232506a4cebe410f02042fb0f0adeeb0b1/packages/babel-preset-typescript/src/plugin-rewrite-ts-imports.ts)

use oxc_ast::ast::{
    ExportAllDeclaration, ExportFromDeclaration, Expression, ImportDeclaration, ImportExpression,
    StringLiteral, TemplateLiteral,
};
use oxc_str::{JSStr, JSStrBuilder, Str};
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
    let replace = replacement_extension(extension)?;

    Some(if mode.is_remove() {
        Str::from(without_extension)
    } else {
        Str::from_strs_array_in([without_extension, replace], ctx)
    })
}

fn replacement_extension(extension: &str) -> Option<&'static str> {
    match extension {
        "mts" => Some(".mjs"),
        "cts" => Some(".cjs"),
        "ts" | "tsx" => Some(".js"),
        _ => None,
    }
}

/// Like [`rewritten_specifier`], for a value that may contain lone surrogates.
/// The part before the extension is copied unchanged.
fn rewritten_js_specifier<'a>(
    value: JSStr<'a>,
    mode: RewriteExtensionsMode,
    ctx: &TraverseCtx<'a>,
) -> Option<JSStr<'a>> {
    if let Some(value) = value.as_str() {
        return rewritten_specifier(value, mode, ctx).map(JSStr::from);
    }

    if !value.contains(|c| c == '/' || c == '\\') {
        return None;
    }

    let dot = value.rfind('.')?;
    // An extension with a lone surrogate is not valid UTF-8 and matches no rewrite.
    let extension = str::from_utf8(&value.as_bytes()[dot + 1..]).ok()?;
    let replace = replacement_extension(extension)?;

    let mut builder = JSStrBuilder::with_capacity_in(dot + replace.len(), ctx);
    let mut len = 0;
    for ch in value.chars() {
        if len == dot {
            break;
        }
        len += ch.len_bytes();
        builder.push_js_char(ch);
    }
    if !mode.is_remove() {
        builder.push_str(replace);
    }
    Some(builder.into_js_str())
}

impl TypeScriptRewriteExtensions {
    pub fn new(options: &TypeScriptOptions) -> Option<Self> {
        options.rewrite_import_extensions.map(|mode| Self { mode })
    }

    pub fn rewrite_extensions<'a>(&self, source: &mut StringLiteral<'a>, ctx: &TraverseCtx<'a>) {
        if let Some(rewritten) = rewritten_js_specifier(source.value, self.mode, ctx) {
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
            quasi.value.cooked = Some(rewritten.into());
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

    fn enter_export_from_declaration(
        &mut self,
        node: &mut ExportFromDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if node.export_kind.is_type() {
            return;
        }
        self.rewrite_extensions(&mut node.source, ctx);
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
