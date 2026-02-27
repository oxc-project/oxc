use oxc_ast::ast::*;

use crate::{
    ast_nodes::AstNode,
    format_args,
    formatter::{Format, Formatter, prelude::*, trivia::format_dangling_comments},
    options::TrailingSeparator,
    write,
};

use super::parameters::{ParameterLayout, ParameterList};

/// Formats `FormalParameters` for Vue `v-for`(LHS) / `v-slot` binding contexts.
///
/// Prettier wraps the source as `function _(...) {}` before calling `textToDoc()`,
/// so the input is always a `FormalParameters` node extracted from the synthetic function.
///
/// Trailing commas at the parameter list level are suppressed,
/// while trailing commas inside nested patterns (e.g., destructured objects/arrays)
/// follow the normal `trailingComma` setting.
///
/// For `v-for` with multiple params (e.g., `(item, index) in items`),
/// - call `.with_parens()` to wrap the list with parentheses
/// - apply the nested indent/group structure from Prettier's `html-binding.js`
///   - <https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/print/html-binding.js#L14>
///
/// For `v-slot` bindings (e.g., `{ item }`), use the default (no parens).
pub struct FormatVueBindingParams<'a, 'b> {
    node: &'b AstNode<'a, FormalParameters<'a>>,
    with_parens: bool,
}

impl<'a, 'b> FormatVueBindingParams<'a, 'b> {
    pub fn new(node: &'b AstNode<'a, FormalParameters<'a>>, with_parens: bool) -> Self {
        Self { node, with_parens }
    }
}

impl<'a> Format<'a> for FormatVueBindingParams<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let list = ParameterList::with_layout(self.node, None, ParameterLayout::Default)
            .with_omit_trailing_separator();

        if self.with_parens {
            write!(
                f,
                [
                    "(",
                    indent(&format_args!(soft_line_break(), group(&list))),
                    soft_line_break(),
                    ")"
                ]
            );
        } else {
            write!(f, list);
        }
    }
}

/// Formats `TSTypeParameterDeclaration` for Vue `<script generic="...">`.
///
/// Prettier wraps the source as `type T<...> = any` before calling `textToDoc()`,
/// so the input is always a `TSTypeParameterDeclaration` extracted from the synthetic type alias.
///
/// Outputs the comma-separated type parameter list with `soft_block_indent` wrapping with comments.
/// Does NOT include angle bracket delimiters `<`/`>` or group wrapping.
/// And trailing commas are suppressed.
pub struct FormatVueScriptGeneric<'a, 'b> {
    decl: &'b AstNode<'a, TSTypeParameterDeclaration<'a>>,
}

impl<'a, 'b> FormatVueScriptGeneric<'a, 'b> {
    pub fn new(decl: &'b AstNode<'a, TSTypeParameterDeclaration<'a>>) -> Self {
        Self { decl }
    }
}

impl<'a> Format<'a> for FormatVueScriptGeneric<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let params = self.decl.params();

        soft_block_indent(&format_with(|f| {
            f.join_with(soft_line_break_or_space()).entries_with_trailing_separator(
                params,
                ",",
                TrailingSeparator::Omit,
            );
        }))
        .fmt(f);

        format_dangling_comments(self.decl.span).with_soft_block_indent().fmt(f);
    }
}
