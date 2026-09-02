//! Rewrite import extensions
//!
//! This plugin is used to rewrite/remove extensions from import/export source.
//! It is only handled source that contains `/` or `\` in the source.
//!
//! Based on Babel's [plugin-rewrite-ts-imports](https://github.com/babel/babel/blob/3bcfee232506a4cebe410f02042fb0f0adeeb0b1/packages/babel-preset-typescript/src/plugin-rewrite-ts-imports.ts)

use oxc_allocator::GetAllocator;
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
    allocator: &impl GetAllocator<'a>,
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
        Str::from_strs_array_in([without_extension, replace], allocator)
    })
}

fn rewritten_js_specifier<'a>(
    value: JSStr<'a>,
    mode: RewriteExtensionsMode,
    allocator: &impl GetAllocator<'a>,
) -> Option<JSStr<'a>> {
    if let Some(value) = value.as_str() {
        return rewritten_specifier(value, mode, allocator).map(Into::into);
    }

    rewrite_js_specifier_with_lone_surrogate(value, mode, allocator)
}

#[cold]
fn rewrite_js_specifier_with_lone_surrogate<'a>(
    value: JSStr<'_>,
    mode: RewriteExtensionsMode,
    allocator: &impl GetAllocator<'a>,
) -> Option<JSStr<'a>> {
    let units = value.encode_utf16().collect::<Vec<_>>();
    if !units.iter().any(|unit| *unit == u16::from(b'/') || *unit == u16::from(b'\\')) {
        return None;
    }

    let (extension_len, replacement) =
        if units.ends_with(&[b'.'.into(), b'm'.into(), b't'.into(), b's'.into()]) {
            (4, ".mjs")
        } else if units.ends_with(&[b'.'.into(), b'c'.into(), b't'.into(), b's'.into()]) {
            (4, ".cjs")
        } else if units.ends_with(&[b'.'.into(), b't'.into(), b's'.into()]) {
            (3, ".js")
        } else if units.ends_with(&[b'.'.into(), b't'.into(), b's'.into(), b'x'.into()]) {
            (4, ".js")
        } else {
            return None;
        };

    let mut rewritten = JSStrBuilder::with_capacity_in(value.len(), allocator.allocator());
    rewritten.push_utf16(&units[..units.len() - extension_len]);
    if !mode.is_remove() {
        rewritten.push_str(replacement);
    }
    Some(rewritten.finish())
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

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;

    use super::*;

    #[test]
    fn rewrites_extension_after_lone_surrogate() {
        let allocator = Allocator::default();
        let value = JSStr::from_utf16_in(
            &[b'.'.into(), b'/'.into(), 0xD800, b'.'.into(), b't'.into(), b's'.into()],
            &&allocator,
        );
        let rewritten =
            rewritten_js_specifier(value, RewriteExtensionsMode::Rewrite, &&allocator).unwrap();

        assert_eq!(
            rewritten.encode_utf16().collect::<Vec<_>>(),
            [b'.'.into(), b'/'.into(), 0xD800, b'.'.into(), b'j'.into(), b's'.into()]
        );
    }
}
