use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        Formatter,
        prelude::*,
        trivia::{FormatLeadingComments, FormatTrailingComments},
    },
    parentheses::NeedsParentheses,
    print::FormatWrite,
    utils::{suppressed::FormatSuppressedNode, typescript::should_hug_type},
    write,
};

impl<'a> FormatWrite<'a> for AstNode<'a, TSUnionType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
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

        // Find the head of the nested union type chain.
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
        let comment_info = LeadingCommentsInfo::from_comments(leading_comments);
        let mut union_type_at_top = self;
        while let AstNodes::TSUnionType(parent) = union_type_at_top.parent()
            && parent.types().len() == 1
        {
            union_type_at_top = parent;
        }

        let should_indent = {
            let parent = union_type_at_top.parent();

            // These parents have indent for their content, so we don't need to indent here
            match parent {
                AstNodes::TSTypeAliasDeclaration(alias) => {
                    should_indent_alias_union(alias, comment_info, f)
                }
                AstNodes::TSTypeAssertion(_)
                | AstNodes::TSTupleType(_)
                | AstNodes::TSTypeParameterInstantiation(_) => false,
                _ => true,
            }
        };

        let types = format_with(|f| {
            let is_suppressed = leading_comments
                .iter()
                .rev()
                .any(|comment| f.comments().is_suppression_comment(comment));

            let suppressed_node_span =
                if is_suppressed { self.types.first().unwrap().span() } else { Span::default() };

            let leading_soft_line_break_or_space = should_indent && !comment_info.has_comments();

            let separator = format_with(|f| {
                if leading_soft_line_break_or_space {
                    write!(f, [soft_line_break_or_space()]);
                }
                write!(f, [token("|"), space()]);
            });

            write!(f, [if_group_breaks(&separator)]);

            format_union_types(types, suppressed_node_span, false, f);
        });

        let content = format_with(|f| {
            // it is necessary to add parentheses for unions in intersections
            // ```ts
            // type Some = B & (C | A) & D
            // ```
            if self.needs_parentheses(f) {
                return write!(f, [indent(&types), soft_line_break()]);
            }

            let is_inside_complex_tuple_type = match self.parent() {
                AstNodes::TSTupleType(tuple) => tuple.element_types().len() > 1,
                _ => false,
            };

            if is_inside_complex_tuple_type {
                write!(
                    f,
                    [
                        indent(&format_args!(
                            if_group_breaks(&format_args!(token("("), soft_line_break())),
                            types
                        )),
                        soft_line_break(),
                        if_group_breaks(&token(")"))
                    ]
                );
            } else {
                write!(f, [types]);
            }
        });

        let format_inner_content = format_with(|f| {
            let only_type = union_type_at_top.types.len() == 1;
            let has_own_line_comment = comment_info.has_own_line_comment
                || (matches!(union_type_at_top.parent(), AstNodes::TSTypeAliasDeclaration(_))
                    && comment_info.has_trailing_own_line_non_jsdoc_block_comment);
            write!(
                f,
                [
                    ((has_own_line_comment && !only_type)
                        || (comment_info.has_end_of_line_comment && only_type))
                        .then(soft_line_break),
                    FormatLeadingComments::Comments(leading_comments),
                    (!comment_info.has_end_of_line_comment && has_own_line_comment && only_type)
                        .then(soft_line_break),
                    group(&content)
                ]
            );
        });

        if should_indent && !self.needs_parentheses(f) {
            write!(f, [group(&indent(&format_inner_content))]);
        } else {
            write!(f, [group(&format_inner_content)]);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct LeadingCommentsInfo {
    has_comments: bool,
    has_own_line_comment: bool,
    has_end_of_line_comment: bool,
    has_trailing_own_line_non_jsdoc_block_comment: bool,
    has_trailing_own_line_jsdoc_comment: bool,
}

impl LeadingCommentsInfo {
    fn from_comments(comments: &[Comment]) -> Self {
        let mut info = Self { has_comments: !comments.is_empty(), ..Self::default() };
        for comment in comments {
            info.has_own_line_comment |= comment.preceded_by_newline();
            info.has_end_of_line_comment |= comment.followed_by_newline();
            info.has_trailing_own_line_non_jsdoc_block_comment |= comment.is_block()
                && comment.is_trailing()
                && comment.followed_by_newline()
                && !matches!(comment.content, CommentContent::Jsdoc | CommentContent::JsdocLegal);
            info.has_trailing_own_line_jsdoc_comment |=
                matches!(comment.content, CommentContent::Jsdoc | CommentContent::JsdocLegal)
                    && comment.is_trailing()
                    && comment.followed_by_newline();
        }
        info
    }

    #[inline]
    fn has_comments(self) -> bool {
        self.has_comments
    }
}

fn should_indent_alias_union<'a>(
    alias: &AstNode<'a, TSTypeAliasDeclaration<'a>>,
    comment_info: LeadingCommentsInfo,
    f: &Formatter<'_, 'a>,
) -> bool {
    // When a union starts after a trailing own-line JSDoc comment
    // (e.g. `=(/** ... */\n| A)`),
    // type-alias indentation is already applied by the parent assignment printer.
    if comment_info.has_trailing_own_line_jsdoc_comment {
        return false;
    }

    !f.comments().printed_comments().last().is_some_and(|comment| {
        comment.span.start
            > alias
                .type_parameters()
                .map_or(alias.id.span.end, |type_parameters| type_parameters.span.end)
            && comment.followed_by_newline()
    })
}

fn format_union_types<'a>(
    node: &AstNode<'a, Vec<'a, TSType<'a>>>,
    mut suppressed_node_span: Span,
    should_hug: bool,
    f: &mut Formatter<'_, 'a>,
) {
    let mut node_iter = node.iter().peekable();
    while let Some(element) = node_iter.next() {
        let element_span = element.span();

        if suppressed_node_span == element_span {
            let comments = f.context().comments().comments_before(suppressed_node_span.start);
            FormatLeadingComments::Comments(comments).fmt(f);
            let needs_parentheses = element.needs_parentheses(f);
            if needs_parentheses {
                write!(f, "(");
            }
            write!(f, [FormatSuppressedNode(element_span)]);
            if needs_parentheses {
                write!(f, ")");
            }
        } else if should_hug {
            write!(f, [element]);
        } else {
            write!(f, [align(2, &element)]);
        }

        if let Some(next_node_span) = node_iter.peek().map(GetSpan::span) {
            if f.comments().is_suppressed(next_node_span.start) {
                suppressed_node_span = next_node_span;
            }

            let comments_before_separator =
                f.context().comments().comments_before_character(element_span.end, b'|');
            FormatTrailingComments::Comments(comments_before_separator).fmt(f);

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
                FormatTrailingComments::Comments(comments).fmt(f);
            }

            if should_hug {
                write!(f, [space()]);
            } else {
                write!(f, [soft_line_break_or_space()]);
            }
            write!(f, ["|"]);
        } else if let AstNodes::TSUnionType(parent) = element.parent()
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
            write!(f, FormatTrailingComments::Comments(comments));
        }

        if node_iter.peek().is_some() {
            write!(f, space());
        }
    }
}
