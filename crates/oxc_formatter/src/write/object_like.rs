use oxc_ast::ast::*;

use crate::{
    formatter::{
        Buffer, Format, FormatResult, Formatter,
        prelude::{
            format_dangling_comments, format_with, group, soft_block_indent_with_maybe_space,
        },
    },
    options::Expand,
    write,
};

#[derive(Clone, Copy)]
pub enum ObjectLike<'a, 'b> {
    ObjectExpression(&'b ObjectExpression<'a>),
    TSTypeLiteral(&'b TSTypeLiteral<'a>),
}

impl<'a> ObjectLike<'a, '_> {
    fn members_have_leading_newline(&self) -> bool {
        false
        // TODO
        // match self {
        // JsObjectLike::JsObjectExpression(o) => o.members().syntax().has_leading_newline(),
        // JsObjectLike::TsObjectType(o) => o.members().syntax().has_leading_newline(),
        // }
    }

    fn members_are_empty(&self) -> bool {
        match self {
            Self::ObjectExpression(o) => o.properties.is_empty(),
            Self::TSTypeLiteral(o) => o.members.is_empty(),
        }
    }

    fn write_members(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::ObjectExpression(o) => o.properties.fmt(f),
            Self::TSTypeLiteral(o) => o.members.fmt(f),
        }
    }
}

impl<'a> Format<'a> for ObjectLike<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let members = format_with(|f| self.write_members(f));

        write!(f, "{")?;

        if self.members_are_empty() {
            // TODO
            // write!(f, [format_dangling_comments(self.syntax()).with_block_indent(),])?;
        } else {
            let should_insert_space_around_brackets = f.options().bracket_spacing.value();
            let should_expand = (f.options().expand == Expand::Auto
                && self.members_have_leading_newline())
                || f.options().expand == Expand::Always;

            // If the object type is the type annotation of the only parameter in a function,
            // try to hug the parameter; we don't create a group and inline the contents here.
            //
            // For example:
            // ```ts
            // const fn = ({ foo }: { foo: string }) => { ... };
            //                      ^ do not break properties here
            // ```
            // TODO
            // let should_hug = self.parent::<TsTypeAnnotation>().is_some_and(|node| {
            // node.parent::<JsFormalParameter>().is_some_and(|node| {
            // node.parent::<JsParameterList>().is_some_and(|node| node.len() == 1)
            // })
            // });
            let should_hug = false;

            let inner =
                soft_block_indent_with_maybe_space(&members, should_insert_space_around_brackets);

            if should_hug {
                write!(f, inner)?;
            } else {
                write!(f, [group(&inner).should_expand(should_expand)])?;
            }
        }

        write!(f, "}")
    }
}
