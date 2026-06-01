use oxc_ast::ast::*;

use crate::{
    ast_nodes::AstNode,
    format_args,
    formatter::{Format, JsFormatter, prelude::*, trivia::format_dangling_comments},
    options::TrailingSeparator,
    write,
};

use super::parameters::{ParameterLayout, ParameterList};

/// Formats `FormalParameters` placed in a binding position.
///
/// The input is a `FormalParameters` node extracted from a synthetic `function _(...) {}`
/// wrapper (see the `FragmentContext` input contract).
///
/// Trailing commas at the parameter list level are suppressed,
/// while trailing commas inside nested patterns (e.g., destructured objects/arrays)
/// follow the normal `trailingComma` setting.
///
/// For a binding-LHS position with multiple params (e.g., `(item, index)`),
/// - `with_parens` wraps the list with parentheses
/// - the nested indent/group structure mirrors Prettier's `html-binding.js`
///   - <https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/print/html-binding.js#L14>
///
/// For a plain binding position (e.g., `{ item }`), use the default (no parens).
pub struct FormatFunctionParams<'a, 'b> {
    node: &'b AstNode<'a, FormalParameters<'a>>,
    with_parens: bool,
}

impl<'a, 'b> FormatFunctionParams<'a, 'b> {
    pub fn new(node: &'b AstNode<'a, FormalParameters<'a>>, with_parens: bool) -> Self {
        Self { node, with_parens }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for FormatFunctionParams<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
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

/// Formats a `TSTypeParameterDeclaration` placed in a standard declaration position.
///
/// The input is a `TSTypeParameterDeclaration` extracted from a synthetic `type T<...> = any`
/// wrapper (see the `FragmentContext` input contract).
///
/// Outputs the comma-separated type parameter list with `soft_block_indent` wrapping with comments.
/// Does NOT include angle bracket delimiters `<`/`>` or group wrapping.
/// And trailing commas are suppressed.
pub struct FormatTypeParameters<'a, 'b> {
    decl: &'b AstNode<'a, TSTypeParameterDeclaration<'a>>,
}

impl<'a, 'b> FormatTypeParameters<'a, 'b> {
    pub fn new(decl: &'b AstNode<'a, TSTypeParameterDeclaration<'a>>) -> Self {
        Self { decl }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for FormatTypeParameters<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
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
