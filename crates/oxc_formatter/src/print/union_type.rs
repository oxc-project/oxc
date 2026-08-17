//! NOTE: This printer deliberately stays on Prettier 3.8-style expansion (one member per line with leading `|`)
//! and will probably NOT follow Prettier 3.9's fit-to-width rewrite
//! (prettier#18827: collapse when it fits, conditional-type `? |` hugging).
//!
//! Not final yet, but that is the working assumption:
//! the per-member layout is the diff-friendly output, and the rewrite is still contested upstream (prettier#19733, open).
//! The remaining `typescript/union/**` conformance failures are this collapse cluster plus the documented comment-placement divergences;
//! once the collapse decision is final, reclassify that cluster as an entry too.
//! Current plan: intersection types unify INTO this printer's layout (see the NOTE in `intersection_type.rs`).

use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        Comments, JsFormatter,
        prelude::*,
        trivia::{FormatLeadingComments, FormatTrailingComments},
    },
    parentheses::NeedsParentheses,
    print::FormatWrite,
    utils::{suppressed::FormatSuppressedNode, typescript::should_hug_type},
    write,
};

impl<'a> FormatWrite<'a> for AstNode<'a, TSUnionType<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        let types = self.types();

        let is_alias_level = matches!(self.parent(), AstNodes::TSTypeAliasDeclaration(_));

        // ```ts
        // {
        //   a: string
        // } | null | void
        // ```
        // should be inlined and not be printed in the multi-line variant
        if should_hug_type(self, f) {
            // Don't take the hug shortcut when a single-member union at the
            // type-alias level has own-line leading comments.
            // Those comments must be handled by the normal union formatting path,
            // so they are printed with correct indentation.

            // `type A = | "VALUE"` is a single-member union that is not
            // parenthesized or nested — a candidate for the hug shortcut.
            let is_single_plain_member = self.types.len() == 1
                && !matches!(
                    self.types.first(),
                    Some(TSType::TSParenthesizedType(_) | TSType::TSUnionType(_))
                );
            // ```ts
            // type A =
            //   /** JSDoc */
            //   | 'VALUE';
            // ```
            let has_comment_before_pipe =
                f.comments().has_leading_own_line_comment(self.span().start);
            // ```ts
            // type A = |
            //   // Comment
            //   'VALUE';
            // ```
            let has_comment_after_pipe = is_single_plain_member
                && f.comments().has_leading_own_line_comment(self.types[0].span().start);

            let has_alias_level_own_line_comments =
                is_alias_level && (has_comment_before_pipe || has_comment_after_pipe);
            if !has_alias_level_own_line_comments {
                return format_union_types(self.types(), Span::default(), true, f);
            }
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
        // For a single plain member at the alias level, also collect comments
        // between `|` and the member (see the hug-shortcut guard above).
        let is_single_plain_member = self.types.len() == 1
            && !matches!(
                self.types.first(),
                Some(TSType::TSParenthesizedType(_) | TSType::TSUnionType(_))
            );
        let leading_comments = if is_alias_level && is_single_plain_member {
            f.context().comments().comments_before(self.types[0].span().start)
        } else {
            f.context().comments().comments_before(self.span().start)
        };
        let mut union_type_at_top = self;
        while let AstNodes::TSUnionType(parent) = union_type_at_top.parent()
            && parent.types().len() == 1
        {
            union_type_at_top = parent;
        }

        // Where leading comments print relative to the synthesized leading `|`:
        // comments go behind it, one canonical form regardless of the source shape.
        // ```ts
        // type X = /* c */ A | B_Long;
        // // also
        // type X = | /* c */ A | B_Long;
        // // also
        // type X =
        //   /* c */ A
        //   | B_Long;
        //
        // // will be
        // type X =
        //   | /* c */ A
        //   | B_Long;
        // ```
        // NOTE: Prettier keeps the comment before the `|` for nested single-member paren sources (`| (/* c */ | A ...`),
        // yet reformats that own output into the canonical form above;
        // we normalize directly to the fixed point.
        //
        // Only comments that END their source line stay before the `|`:
        // moving one would force a break between the `|` and its member.
        // ```ts
        // type X = /* c */
        //   A | B_Long;
        //
        // // will be
        // type X =
        //   /* c */
        //   A | B_Long;
        // ```
        // This also keeps own-line comments own-line, and line comments harmless (`| // c` would swallow the member).
        //
        // All-or-nothing: a partial move would reorder comments or split a same-line group,
        // so one line-ending comment keeps the whole list before the `|`.
        //
        // The guard is about the printed `|`.
        // A single plain member prints no operator, so a line-ending BLOCK comment there inlines with the member instead
        // (`= | /* c */\n'A'` gives `= /* c */ 'A'`, matching Prettier).
        // The sibling rule for the formatter-added `(` is `format_outer_leading_comments_and_open_paren`,
        // keyed on the source `(` instead.
        let comment_info = LeadingCommentsInfo::from_comments(leading_comments);
        let all_inline = !comment_info.has_end_of_line_comment;
        let (before_pipe_comments, inline_member_comments) = if all_inline {
            (&leading_comments[..0], leading_comments)
        } else {
            (leading_comments, &leading_comments[..0])
        };

        // A `?`/`:` branch hugs its leading comments behind the operator,
        // like conditional expressions do: no break before the comments, and no union indent on top of the conditional's alignment
        // (the content must sit one level under `?`, not one level under the comment)
        // ```ts
        // type T = X extends Y
        //   ? /** c */
        //     A | B
        //   : Z;
        // ```
        let is_conditional_branch = match union_type_at_top.parent() {
            AstNodes::TSConditionalType(conditional) => {
                let span = union_type_at_top.span();
                conditional.true_type.span() == span || conditional.false_type.span() == span
            }
            _ => false,
        };

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
                // A check/extends-position union falls through to `_ => true`
                AstNodes::TSConditionalType(_) if is_conditional_branch => {
                    before_pipe_comments.is_empty()
                }
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

            let leading_soft_line_break_or_space = should_indent && before_pipe_comments.is_empty();

            let separator = format_with(|f| {
                if leading_soft_line_break_or_space {
                    write!(f, [soft_line_break_or_space()]);
                }
                write!(f, [token("|"), space()]);
            });

            write!(f, [if_group_breaks(&separator)]);
            FormatLeadingComments::Comments(inline_member_comments).fmt(f);

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
            // An own-line leading comment breaks BEFORE itself and stays above the first member
            // ```ts
            // type A =
            //   /* c */
            //   | A
            //   | B;
            // ```
            // breaking after instead would hoist the comment onto the `=` line
            // (`type A = /* c */`), which is not own-line anymore and not idempotent.
            // Conditional branches hug instead (no break, see `is_conditional_branch`).
            let breaks_before_comments = (has_own_line_comment
                || (comment_info.has_end_of_line_comment && only_type))
                && !is_conditional_branch;
            write!(
                f,
                [
                    breaks_before_comments.then(soft_line_break),
                    FormatLeadingComments::Comments(before_pipe_comments),
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
    has_own_line_comment: bool,
    has_end_of_line_comment: bool,
    has_trailing_own_line_non_jsdoc_block_comment: bool,
    has_trailing_own_line_jsdoc_comment: bool,
}

impl LeadingCommentsInfo {
    fn from_comments(comments: &[Comment]) -> Self {
        let mut info = Self::default();
        for comment in comments {
            info.has_own_line_comment |= comment.preceded_by_newline();
            info.has_end_of_line_comment |= comment.followed_by_newline();
            info.has_trailing_own_line_non_jsdoc_block_comment |= comment.is_block()
                && comment.is_trailing()
                && comment.followed_by_newline()
                && !matches!(comment.content, CommentContent::Jsdoc | CommentContent::JsdocLegal);
            info.has_trailing_own_line_jsdoc_comment |= is_trailing_own_line_jsdoc_comment(comment);
        }
        info
    }
}

pub fn is_trailing_own_line_jsdoc_comment(comment: &Comment) -> bool {
    matches!(comment.content, CommentContent::Jsdoc | CommentContent::JsdocLegal)
        && comment.is_trailing()
        && comment.followed_by_newline()
}

/// End of a type alias's left-hand side:
/// after the type parameters when present, after the name otherwise.
pub fn type_alias_left_end(decl: &TSTypeAliasDeclaration) -> u32 {
    decl.type_parameters
        .as_ref()
        .map_or(decl.id.span.end, |type_parameters| type_parameters.span.end)
}

/// Whether an alias-level union relies on the assignment's operator-side break + indent
/// instead of owning them itself (`should_indent_alias_union` is the negation).
/// True when a comment ends the `=` line:
/// - a trailing own-line JSDoc comment, still pending for the union's leading-comments pass
///   the caller decides the comment range to scan, which differs per site
///   ```ts
///   type A = (/** ... */
///   | A
///   | B);
///   ```
/// - an end-of-line comment already consumed by the assignment's left side
///   (hence a printed-comments check, usable after write_left has run);
///   any comment ending the `=` line counts, eol block comments included
///   (`type A = // c` and `type A = /* c */ // c`)
pub fn alias_union_breaks_after_operator(
    decl: &TSTypeAliasDeclaration,
    has_trailing_own_line_jsdoc_comment: bool,
    comments: &Comments,
) -> bool {
    has_trailing_own_line_jsdoc_comment
        || comments.printed_comments().last().is_some_and(|comment| {
            comment.span.start > type_alias_left_end(decl) && comment.followed_by_newline()
        })
}

fn should_indent_alias_union<'a>(
    alias: &AstNode<'a, TSTypeAliasDeclaration<'a>>,
    comment_info: LeadingCommentsInfo,
    f: &JsFormatter<'_, 'a>,
) -> bool {
    !alias_union_breaks_after_operator(
        alias,
        comment_info.has_trailing_own_line_jsdoc_comment,
        f.comments(),
    )
}

fn format_union_types<'a>(
    node: &AstNode<'a, ArenaVec<'a, TSType<'a>>>,
    mut suppressed_node_span: Span,
    should_hug: bool,
    f: &mut JsFormatter<'_, 'a>,
) {
    let mut node_iter = node.iter().peekable();
    while let Some(element) = node_iter.next() {
        let element_span = element.span();
        let has_trailing_suppression_comment =
            f.comments().has_trailing_suppression_comment(element_span.end);

        if suppressed_node_span == element_span || has_trailing_suppression_comment {
            let comments = f.context().comments().comments_before(element_span.start);
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
            //
            // NOTE: this is the same hoist rule as `format_hoisted_leading_comments`, but it must stay trailing-style:
            // Prettier preserves a blank line before the comment in union chains (`lines_before`-driven),
            // while collapsing it in binary-like chains (leading-style).
            // One shared rendering would diverge on one of the two.
            //
            // The asymmetry is an attachment artifact on Prettier's side:
            // only its trailing-comment printer preserves the empty line (`printTrailingComment`),
            // and this comment attaches as trailing there, leading in binary-like chains.
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
            // TODO: We may need to tweak `AstNode<'a, ArenaVec<'a, T>>` iterator as some of Vec's last elements should have the following span.
            let comments = f.context().comments().end_of_line_comments_after(element_span.end);
            write!(f, FormatTrailingComments::Comments(comments));
        }

        if node_iter.peek().is_some() {
            write!(f, space());
        }
    }
}
