//! Rewrite import extensions
//!
//! This plugin is used to rewrite/remove extensions from import/export source.
//! It is only handled source that contains `/` or `\` in the source.
//!
//! Based on Babel's [plugin-rewrite-ts-imports](https://github.com/babel/babel/blob/3bcfee232506a4cebe410f02042fb0f0adeeb0b1/packages/babel-preset-typescript/src/plugin-rewrite-ts-imports.ts)

use oxc_allocator::GetAllocator;
use oxc_ast::ast::{
    ExportAllDeclaration, ExportFromDeclaration, Expression, ImportDeclaration, ImportExpression,
    StringLiteral, TemplateElement, TemplateElementValue,
};
use oxc_str::{JSStr, JSStrBuilder, Str};
use oxc_traverse::Traverse;

use crate::{TypeScriptOptions, context::TraverseCtx, state::TransformState};

use super::options::RewriteExtensionsMode;

pub struct TypeScriptRewriteExtensions {
    mode: RewriteExtensionsMode,
}

/// Given a specifier value, compute the replacement `JSStr` if the extension
/// should be rewritten/removed. Returns `None` when no rewriting is needed.
fn rewritten_specifier<'a>(
    value: JSStr<'a>,
    mode: RewriteExtensionsMode,
    ctx: &TraverseCtx<'a>,
) -> Option<JSStr<'a>> {
    let Some(value) = value.as_str() else {
        return rewritten_wtf8_specifier(value, mode, ctx);
    };
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
        JSStr::from(without_extension)
    } else {
        Str::from_strs_array_in([without_extension, replace], ctx).into()
    })
}

#[cold]
fn rewritten_wtf8_specifier<'a>(
    value: JSStr<'a>,
    mode: RewriteExtensionsMode,
    ctx: &TraverseCtx<'a>,
) -> Option<JSStr<'a>> {
    let units: Vec<_> = value.encode_utf16().collect();
    if !units.iter().any(|&unit| unit == u16::from(b'/') || unit == 0x5C) {
        return None;
    }
    let dot = units.iter().rposition(|&unit| unit == u16::from(b'.'))?;
    let replacement = match &units[dot + 1..] {
        [0x6D, 0x74, 0x73] => ".mjs",
        [0x63, 0x74, 0x73] => ".cjs",
        [0x74, 0x73] | [0x74, 0x73, 0x78] => ".js",
        _ => return None,
    };
    let mut builder = JSStrBuilder::with_capacity_in(value.len(), ctx.allocator());
    builder.push_utf16(&units[..dot]);
    if !mode.is_remove() {
        builder.push_str(replacement);
    }
    Some(builder.into_js_str())
}

impl TypeScriptRewriteExtensions {
    pub fn new(options: &TypeScriptOptions) -> Option<Self> {
        options.rewrite_import_extensions.map(|mode| Self { mode })
    }

    pub fn rewrite_extensions<'a>(&self, source: &mut StringLiteral<'a>, ctx: &TraverseCtx<'a>) {
        if let Some(rewritten) = rewritten_specifier(source.value, self.mode, ctx) {
            source.value = rewritten;
            source.raw = None;
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
                if let Some(value) = template.single_quasi()
                    && let Some(rewritten) = rewritten_specifier(value, self.mode, ctx)
                {
                    // Read cooked text so escaped extensions are rewritten correctly too.
                    if let Some(raw) = rewritten.as_str() {
                        let quasi = &mut template.quasis[0];
                        *quasi = TemplateElement::new_escape_raw(
                            quasi.span,
                            TemplateElementValue { raw: raw.into(), cooked: Some(rewritten) },
                            true,
                            ctx,
                        );
                    } else {
                        // A string literal has the same value and lets codegen escape
                        // lone surrogates without exposing them as UTF-8 raw text.
                        node.source =
                            Expression::new_string_literal(template.span, rewritten, None, ctx);
                    }
                }
            }
            _ => {}
        }
    }
}
