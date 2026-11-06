//! [`JsRoot::FunctionParams`](crate::JsRoot::FunctionParams) (Vue `v-slot`, `v-for` left)
//! and [`JsRoot::TypeParameters`](crate::JsRoot::TypeParameters) (Vue `<script generic>`).

use oxc_ast::ast::*;

use crate::{
    ast_nodes::AstNode,
    format_args,
    formatter::{JsFormatter, prelude::*, trivia::format_dangling_comments},
    options::TrailingSeparator,
    write,
};

use super::super::parameters::{ParameterLayout, ParameterList};

/// Writes `params` in a group, in parens when `parens_when_many` and more than one or a rest element.
pub(super) fn write_function_params<'a>(
    params: &AstNode<'_, 'a, FormalParameters<'a>>,
    parens_when_many: bool,
    ends_in_line_comment: bool,
    f: &mut JsFormatter<'_, 'a>,
) {
    let with_parens = parens_when_many && (1 < params.items.len() || params.rest.is_some());
    let params = FormatFunctionParams { node: params, with_parens };
    if ends_in_line_comment {
        write!(f, soft_block_indent(&params));
    } else {
        write!(f, group(&params));
    }
}

/// Formats `FormalParameters` placed in a binding position.
///
/// The input is a `FormalParameters` node extracted from the `function _(...) {}`
/// a [`JsRoot::FunctionParams`](crate::JsRoot::FunctionParams) is parsed in.
///
/// Trailing commas at the parameter list level are suppressed,
/// while trailing commas inside nested patterns (e.g., destructured objects/arrays)
/// follow the normal `trailingComma` setting.
///
/// For a binding-LHS position with multiple params (e.g., `(item, index)`),
/// `with_parens` wraps the list with parentheses, which break around it.
///
/// For a plain binding position (e.g., `{ item }`), use the default (no parens).
struct FormatFunctionParams<'a, 'b> {
    node: &'b AstNode<'b, 'a, FormalParameters<'a>>,
    with_parens: bool,
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
/// The input is a `TSTypeParameterDeclaration` extracted from the `type T<...> = any`
/// a [`JsRoot::TypeParameters`](crate::JsRoot::TypeParameters) is parsed in.
///
/// Outputs the comma-separated type parameter list with `soft_block_indent` wrapping with comments.
/// Does NOT include angle bracket delimiters `<`/`>` or group wrapping.
/// And trailing commas are suppressed.
pub(super) struct FormatTypeParameters<'a, 'b> {
    pub decl: &'b AstNode<'b, 'a, TSTypeParameterDeclaration<'a>>,
}

impl<'a> Format<'a, JsFormatContext<'a>> for FormatTypeParameters<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let params = self.decl.params();

        soft_block_indent(&format_with(|f| {
            f.join_with(soft_line_break_or_space()).entries_with_trailing_separator(
                &params,
                ",",
                TrailingSeparator::Omit,
            );
        }))
        .fmt(f);

        format_dangling_comments(self.decl.span).with_soft_block_indent().fmt(f);
    }
}
