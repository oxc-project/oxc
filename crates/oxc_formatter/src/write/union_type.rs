use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    format_args,
    formatter::{
        FormatResult, Formatter,
        prelude::*,
        trivia::{FormatLeadingComments, FormatTrailingComments},
    },
    generated::ast_nodes::{AstNode, AstNodes},
    parentheses::NeedsParentheses,
    utils::typescript::should_hug_type,
    write,
    write::FormatWrite,
};

impl<'a> FormatWrite<'a> for AstNode<'a, TSUnionType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let types = self.types();

        // ```ts
        // {
        //   a: string
        // } | null | void
        // ```
        // should be inlined and not be printed in the multi-line variant
        let should_hug = should_hug_type(self, f);
        if should_hug {
            return format_union_types(self.types(), true, f);
        }

        // Find the head of the nest union type chain
        // ```js
        // type Foo = | (| (A | B))
        //                  ^^^^^
        // ```
        // If the current union type is `A | B`
        // - `A | B` is the inner union type of `| (A | B)`
        // - `| (A | B)` is the inner union type of `| (| (A | B))`
        //
        // So the head of the current nested union type chain is `| (| (A | B))`
        // if we encounter a leading comment when navigating up the chain,
        // we consider the current union type as having leading comments
        let leading_comments = f.context().comments().comments_before(self.span().start);
        let has_leading_comments = !leading_comments.is_empty();
        let mut union_type_at_top = self;
        while let AstNodes::TSUnionType(parent) = union_type_at_top.parent {
            if parent.types().len() == 1 {
                union_type_at_top = parent;
            } else {
                break;
            }
        }

        let should_indent = {
            let parent = union_type_at_top.parent;

            // These parents have indent for their content, so we don't need to indent here
            !match parent {
                AstNodes::TSTypeAliasDeclaration(_) => has_leading_comments,
                AstNodes::TSTypeAssertion(_)
                | AstNodes::TSTupleType(_)
                | AstNodes::TSTypeParameterInstantiation(_) => true,
                _ => false,
            }
        };

        let types = format_with(|f| {
            if has_leading_comments {
                write!(f, FormatLeadingComments::Comments(leading_comments))?;
            }

            let leading_soft_line_break_or_space = should_indent && !has_leading_comments;

            let separator = format_with(|f| {
                if leading_soft_line_break_or_space {
                    write!(f, [soft_line_break_or_space()])?;
                }
                write!(f, [text("|"), space()])
            });

            write!(f, [if_group_breaks(&separator)])?;

            format_union_types(types, false, f)
        });

        let content = format_with(|f| {
            // it is necessary to add parentheses for unions in intersections
            // ```ts
            // type Some = B & (C | A) & D
            // ```
            if self.needs_parentheses(f) {
                return write!(f, [indent(&types), soft_line_break()]);
            }

            let is_inside_complex_tuple_type = match self.parent {
                AstNodes::TSTupleType(tuple) => tuple.element_types().len() > 1,
                _ => false,
            };

            if is_inside_complex_tuple_type {
                write!(
                    f,
                    [
                        indent(&format_args!(
                            if_group_breaks(&format_args!(text("("), soft_line_break())),
                            types
                        )),
                        soft_line_break(),
                        if_group_breaks(&text(")"))
                    ]
                )
            } else if should_indent {
                write!(f, [indent(&types)])
            } else {
                write!(f, [types])
            }
        });

        write!(f, [group(&content)])
    }
}

pub struct FormatTSType<'a, 'b> {
    next_node_span: Option<Span>,
    element: &'b AstNode<'a, TSType<'a>>,
    should_hug: bool,
}

impl<'a> Format<'a> for FormatTSType<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let format_element = format_once(|f| {
            self.element.fmt(f)?;
            Ok(())
        });
        if self.should_hug {
            write!(f, [format_element])?;
        } else {
            write!(f, [align(2, &format_element)])?;
        }

        if let Some(next_node_span) = self.next_node_span {
            let comments_before_separator =
                f.context().comments().comments_before_character(self.element.span().end, b'|');
            FormatTrailingComments::Comments(comments_before_separator).fmt(f)?;

            // ```ts
            // type Some = A |
            // // comment
            // B
            // ```
            // to
            // ```ts
            // type Some =
            // | A
            // // comment
            // | B
            // ```
            // If there is a leading own line comment between `|` and the next node, we need to put printing comments
            // before `|` instead of after it.
            if f.comments().has_leading_own_line_comment(next_node_span.start) {
                let comments = f.context().comments().comments_before(next_node_span.start);
                FormatTrailingComments::Comments(comments).fmt(f)?;
            }

            if self.should_hug {
                write!(f, [space()])?;
            } else {
                write!(f, [soft_line_break_or_space()])?;
            }
            write!(f, ["|"])
        } else if let AstNodes::TSUnionType(parent) = self.element.parent
            && parent.needs_parentheses(f)
        {
            // ```ts
            // type Foo = (
            // | "thing1" // comment1
            // | "thing2" // comment2
            //            ^^^^^^^^^^^ the following logic is to print comment2,
            // )[]; // comment 3
            //```
            // TODO: We may need to tweak `AstNode<'a, Vec<'a, T>>` iterator as some of Vec's last elements should have the following span.
            let comments =
                f.context().comments().end_of_line_comments_after(self.element.span().end);
            FormatTrailingComments::Comments(comments).fmt(f)
        } else {
            Ok(())
        }
    }
}

fn format_union_types<'a>(
    node: &AstNode<'a, Vec<'a, TSType<'a>>>,
    should_hug: bool,
    f: &mut Formatter<'_, 'a>,
) -> FormatResult<()> {
    f.join_with(space())
        .entries(node.iter().enumerate().map(|(index, item)| FormatTSType {
            next_node_span: node.get(index + 1).map(GetSpan::span),
            element: item,
            should_hug,
        }))
        .finish()
}
