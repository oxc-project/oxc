use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        FormatResult, Formatter,
        prelude::*,
        trivia::{FormatLeadingComments, FormatTrailingComments},
        write,
    },
    parentheses::NeedsParentheses,
    utils::{suppressed::FormatSuppressedNode, typescript::should_hug_type},
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
            return format_union_types(self.types(), Span::default(), true, f);
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
        while let AstNodes::TSUnionType(parent) = union_type_at_top.parent
            && parent.types().len() == 1
        {
            union_type_at_top = parent;
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
            let suppressed_node_span = if f.comments().is_suppressed(self.span.start) {
                self.types.first().unwrap().span()
            } else {
                Span::default()
            };

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

            format_union_types(types, suppressed_node_span, false, f)
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

fn format_union_types<'a>(
    node: &AstNode<'a, Vec<'a, TSType<'a>>>,
    mut suppressed_node_span: Span,
    should_hug: bool,
    f: &mut Formatter<'_, 'a>,
) -> FormatResult<()> {
    let mut node_iter = node.iter().peekable();
    while let Some(element) = node_iter.next() {
        let element_span = element.span();

        if suppressed_node_span == element_span {
            let comments = f.context().comments().comments_before(suppressed_node_span.start);
            FormatLeadingComments::Comments(comments).fmt(f)?;
            let needs_parentheses = element.needs_parentheses(f);
            if needs_parentheses {
                write!(f, "(")?;
            }
            write!(f, [FormatSuppressedNode(element_span)])?;
            if needs_parentheses {
                write!(f, ")")?;
            }
        } else if should_hug {
            write!(f, [element])?;
        } else {
            write!(f, [align(2, &element)])?;
        }

        if let Some(next_node_span) = node_iter.peek().map(GetSpan::span) {
            if f.comments().is_suppressed(next_node_span.start) {
                suppressed_node_span = next_node_span;
            }

            let comments_before_separator =
                f.context().comments().comments_before_character(element_span.end, b'|');
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

            if should_hug {
                write!(f, [space()])?;
            } else {
                write!(f, [soft_line_break_or_space()])?;
            }
            write!(f, ["|"])?;
        } else if let AstNodes::TSUnionType(parent) = element.parent
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
            let comments = f.context().comments().end_of_line_comments_after(element_span.end);
            write!(f, FormatTrailingComments::Comments(comments))?;
        }

        if node_iter.peek().is_some() {
            write!(f, space())?;
        }
    }

    Ok(())
}
