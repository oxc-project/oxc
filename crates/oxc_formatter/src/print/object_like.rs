use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{
        Buffer, Format, Formatter,
        prelude::{format_with, group, soft_block_indent_with_maybe_space},
        trivia::format_dangling_comments,
    },
    options::Expand,
    print::parameters::{get_this_param, should_hug_function_parameters},
    write,
};

#[derive(Clone, Copy)]
pub enum ObjectLike<'a, 'b> {
    ObjectExpression(&'b AstNode<'a, ObjectExpression<'a>>),
    TSTypeLiteral(&'b AstNode<'a, TSTypeLiteral<'a>>),
}

impl<'a> ObjectLike<'a, '_> {
    fn span(&self) -> Span {
        match self {
            ObjectLike::ObjectExpression(o) => o.span,
            ObjectLike::TSTypeLiteral(o) => o.span,
        }
    }

    fn should_hug(&self, f: &Formatter<'_, 'a>) -> bool {
        // Check if the object type is the type annotation of the only parameter in a function.
        // This prevents breaking object properties in cases like:
        // const fn = ({ foo }: { foo: string }) => { ... };
        matches!(self, Self::TSTypeLiteral(node) if {
            // Check if parent is TSTypeAnnotation
            matches!(node.parent, AstNodes::TSTypeAnnotation(type_ann) if {
                match &type_ann.parent {
                    AstNodes::FormalParameter(param) if param.initializer.is_none() => {
                        let AstNodes::FormalParameters(parameters) = &param.parent else {
                            unreachable!()
                        };
                        let this_param = get_this_param(parameters.parent);
                        should_hug_function_parameters(parameters, this_param, false, f)

                    }
                    AstNodes::TSThisParameter(param) => {
                        matches!(param.parent, AstNodes::Function(func) if {
                            should_hug_function_parameters(func.params(), Some(param), false, f)
                        })
                    },
                    _ => false,
                }
            })
        })
    }

    fn members_have_leading_newline(&self, f: &Formatter<'_, 'a>) -> bool {
        match self {
            Self::ObjectExpression(o) => o.as_ref().properties.first().is_some_and(|p| {
                f.source_text().contains_newline_between(o.span.start, p.span().start)
            }),
            Self::TSTypeLiteral(o) => o.as_ref().members.first().is_some_and(|p| {
                f.source_text().contains_newline_between(o.span().start, p.span().start)
            }),
        }
    }

    fn members_are_empty(&self) -> bool {
        match self {
            Self::ObjectExpression(o) => o.properties().is_empty(),
            Self::TSTypeLiteral(o) => o.members().is_empty(),
        }
    }

    /// Check if the object is inside a JSX spread attribute or JSX spread child.
    /// In these contexts, empty objects with comments should use soft line breaks
    /// so they can stay on one line if they fit.
    fn is_inside_jsx_spread(&self) -> bool {
        match self {
            Self::ObjectExpression(o) => {
                matches!(o.parent, AstNodes::JSXSpreadAttribute(_) | AstNodes::JSXSpreadChild(_))
            }
            Self::TSTypeLiteral(_) => false,
        }
    }

    fn write_members(&self, f: &mut Formatter<'_, 'a>) {
        match self {
            Self::ObjectExpression(o) => o.properties().fmt(f),
            Self::TSTypeLiteral(o) => o.members().fmt(f),
        }
    }
}

impl<'a> Format<'a> for ObjectLike<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let members = format_with(|f| self.write_members(f));

        write!(f, "{");

        if self.members_are_empty() {
            // In JSX spread attributes/children, use soft indent so the object
            // can stay on one line if it fits (e.g., `<div {...{/* comment */}} />`).
            // In other contexts like variable declarations, use block indent to
            // expand the object (e.g., `var a = {\n  /* comment */\n};`).
            if self.is_inside_jsx_spread() {
                write!(f, format_dangling_comments(self.span()).with_soft_block_indent());
            } else {
                write!(f, format_dangling_comments(self.span()).with_block_indent());
            }
        } else {
            let should_insert_space_around_brackets = f.options().bracket_spacing.value();
            let should_expand =
                f.options().expand == Expand::Auto && self.members_have_leading_newline(f);

            // If the object type is the type annotation of the only parameter in a function,
            // try to hug the parameter; we don't create a group and inline the contents here.
            //
            // For example:
            // ```ts
            // const fn = ({ foo }: { foo: string }) => { ... };
            //                      ^ do not break properties here
            // ```
            let should_hug = self.should_hug(f);

            let inner =
                soft_block_indent_with_maybe_space(&members, should_insert_space_around_brackets);

            if should_hug {
                write!(f, inner);
            } else {
                write!(f, [group(&inner).should_expand(should_expand)]);
            }
        }

        write!(f, "}");
    }
}
