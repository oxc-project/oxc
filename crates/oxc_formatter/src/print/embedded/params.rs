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
    // Trailing commas only at the list's level: nested patterns follow `trailingComma`
    let list = ParameterList::with_layout(params, None, ParameterLayout::Default)
        .with_omit_trailing_separator();
    let params = format_with(|f| {
        if with_parens {
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
    });
    if ends_in_line_comment {
        write!(f, soft_block_indent(&params));
    } else {
        write!(f, group(&params));
    }
}

/// Writes `decl`'s type parameters, extracted from the `type T<...> = any`
/// a [`JsRoot::TypeParameters`](crate::JsRoot::TypeParameters) is parsed in:
/// with `soft_block_indent` and its comments, but no `<`/`>`, group or trailing comma.
pub(super) fn write_type_parameters<'a>(
    decl: &AstNode<'_, 'a, TSTypeParameterDeclaration<'a>>,
    f: &mut JsFormatter<'_, 'a>,
) {
    let params = decl.params();
    soft_block_indent(&format_with(|f| {
        f.join_with(soft_line_break_or_space()).entries_with_trailing_separator(
            &params,
            ",",
            TrailingSeparator::Omit,
        );
    }))
    .fmt(f);
    format_dangling_comments(decl.span).with_soft_block_indent().fmt(f);
}
