use oxc_allocator::{Address, Vec};
use oxc_ast::{AstKind, ast::*};

use crate::{
    format_args,
    formatter::{
        Buffer, Format, FormatError, FormatResult, Formatter, GroupId, prelude::*,
        separated::FormatSeparatedIter,
    },
    generated::ast_nodes::{AstNode, AstNodes},
    options::{FormatTrailingCommas, TrailingSeparator},
    write,
};

impl<'a> Format<'a> for AstNode<'a, Vec<'a, TSTypeParameter<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // Type parameter lists of arrow function expressions have to include at least one comma
        // to avoid any ambiguity with JSX elements.
        // Thus, we have to add a trailing comma when there is a single type parameter.
        // The comma can be omitted in the case where the single parameter has a constraint,
        // i.i. an `extends` clause.
        let trailing_separator = if self.len() == 1
        // This only concern sources that allow JSX or a restricted standard variant.
        && f.context().source_type().is_jsx()
        && matches!(self.parent.parent(), AstNodes::ArrowFunctionExpression(_))
        // Ignore Type parameter with an `extends` clause or a default type.
        && !self.first().is_some_and(|t| t.constraint().is_some() || t.default().is_some())
        {
            TrailingSeparator::Mandatory
        } else {
            FormatTrailingCommas::ES5.trailing_separator(f.options())
        };

        f.join_with(soft_line_break_or_space())
            .entries(
                FormatSeparatedIter::new(self.iter(), ",")
                    .with_trailing_separator(trailing_separator),
            )
            .finish()
    }
}

pub struct FormatTsTypeParametersOptions {
    pub group_id: Option<GroupId>,
    pub is_type_or_interface_decl: bool,
}

pub struct FormatTsTypeParameters<'a, 'b> {
    decl: &'b AstNode<'a, TSTypeParameterDeclaration<'a>>,
    options: FormatTsTypeParametersOptions,
}

impl<'a, 'b> FormatTsTypeParameters<'a, 'b> {
    pub fn new(
        decl: &'b AstNode<'a, TSTypeParameterDeclaration<'a>>,
        options: FormatTsTypeParametersOptions,
    ) -> Self {
        Self { decl, options }
    }
}

impl<'a> Format<'a> for FormatTsTypeParameters<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.decl.params().is_empty() && self.options.is_type_or_interface_decl {
            write!(f, "<>")
        } else {
            write!(
                f,
                [group(&format_args!("<", soft_block_indent(&self.decl.params()), ">"))
                    .with_group_id(self.options.group_id)]
            )
        }
    }
}
