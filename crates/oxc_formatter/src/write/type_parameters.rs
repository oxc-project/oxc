use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    format_args,
    formatter::{
        Buffer, Format, FormatError, FormatResult, Formatter, GroupId, prelude::*,
        separated::FormatSeparatedIter,
    },
    options::TrailingSeparator,
    write,
};

impl<'a> Format<'a> for Vec<'a, TSTypeParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // Type parameter lists of arrow function expressions have to include at least one comma
        // to avoid any ambiguity with JSX elements.
        // Thus, we have to add a trailing comma when there is a single type parameter.
        // The comma can be omitted in the case where the single parameter has a constraint,
        // i.i. an `extends` clause.
        // TODO
        // let trailing_separator = if node.len() == 1
        // // This only concern sources that allow JSX or a restricted standard variant.
        // && !f.options().source_type().variant().is_standard()
        // && node.syntax().grand_parent().kind()
        // == Some(JsSyntaxKind::JS_ARROW_FUNCTION_EXPRESSION)
        // // Ignore Type parameter with an `extends` clause or a default type.
        // && !node.first().and_then(|param| param.ok())
        // .is_some_and(|type_parameter| type_parameter.constraint().is_some() || type_parameter.default().is_some())
        // {
        // TrailingSeparator::Mandatory
        // } else {
        // FormatTrailingCommas::ES5.trailing_separator(f.options())
        // };
        let trailing_separator = TrailingSeparator::Mandatory;

        f.join_with(&soft_line_break_or_space())
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
    decl: &'b TSTypeParameterDeclaration<'a>,
    options: FormatTsTypeParametersOptions,
}

impl<'a, 'b> FormatTsTypeParameters<'a, 'b> {
    pub fn new(
        decl: &'b TSTypeParameterDeclaration<'a>,
        options: FormatTsTypeParametersOptions,
    ) -> Self {
        Self { decl, options }
    }
}

impl<'a> Format<'a> for FormatTsTypeParameters<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.decl.params.is_empty() && self.options.is_type_or_interface_decl {
            write!(f, "<>")
        } else if self.decl.params.is_empty() {
            return Err(FormatError::SyntaxError);
        } else {
            write!(
                f,
                [group(&format_args!("<", soft_block_indent(&self.decl.params), ">"))
                    .with_group_id(self.options.group_id)]
            )
        }
    }
}
