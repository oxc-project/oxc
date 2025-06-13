use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    formatter::{
        Buffer, Format, FormatResult, Formatter,
        prelude::{format_with, group, soft_block_indent_with_maybe_space},
    },
    generated::ast_nodes::AstNode,
    options::Expand,
    write,
};

#[derive(Clone, Copy)]
pub enum ObjectLike<'a, 'b> {
    ObjectExpression(&'b AstNode<'a, ObjectExpression<'a>>),
    TSTypeLiteral(&'b AstNode<'a, TSTypeLiteral<'a>>),
}

impl<'a> ObjectLike<'a, '_> {
    fn members_have_leading_newline(&self, f: &Formatter<'_, 'a>) -> bool {
        // TODO: Polish the code
        match self {
            Self::ObjectExpression(o) => o.as_ref().properties.first().is_some_and(|p| {
                Span::new(o.span().start, p.span().start)
                    .source_text(f.source_text())
                    .contains('\n')
            }),
            Self::TSTypeLiteral(o) => o.as_ref().members.first().is_some_and(|p| {
                Span::new(o.span().start, p.span().start)
                    .source_text(f.source_text())
                    .contains('\n')
            }),
        }
    }

    fn members_are_empty(&self) -> bool {
        match self {
            Self::ObjectExpression(o) => o.properties().is_empty(),
            Self::TSTypeLiteral(o) => o.members().is_empty(),
        }
    }

    fn write_members(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::ObjectExpression(o) => o.properties().fmt(f),
            Self::TSTypeLiteral(o) => o.members().fmt(f),
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
                && self.members_have_leading_newline(f))
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
