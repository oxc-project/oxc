use oxc_ast::ast::{TSMappedType, TSMappedTypeModifierOperator};

use crate::{
    ast_nodes::AstNode,
    format_args,
    formatter::{
        prelude::*,
        trivia::{DanglingIndentMode, FormatDanglingComments},
    },
    print::semicolon::OptionalSemicolon,
    utils::suppressed::FormatSuppressedNode,
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, TSMappedType<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_suppressed(self.key.span.start) {
            return write!(f, FormatSuppressedNode(self.span));
        }

        let key = self.key();
        let constraint = self.constraint();
        let name_type = self.name_type();
        // Check if the user introduced a new line immediately after the opening brace.
        // For example, this would break:
        //   {
        //     readonly [A in B]: T}
        // Because the line break occurs right after `{`. But this would _not_ break:
        //   { readonly
        //     [A in B]: T}
        // Because the break is not immediately after `{`.
        // `+ 1` skips the opening `{`.
        let should_expand =
            f.source_text().has_line_terminator_after_skipping_comments(self.span.start + 1);

        let format_inner = format_with(|f| {
            // Comments between `{` and `[` are the mapped type's dangling comments, joined by hard line breaks.
            // After the last one: a hard line break when it is a line comment or the source breaks after it,
            // otherwise a group collapsing to a space when the member fits on the line.
            // Only the last comment goes inside that group: the ones before it always hard-break.
            let comments = f.context().comments().comments_before_character(self.span.start, b'[');
            if let Some((last, rest)) = comments.split_last() {
                if last.is_line() || last.followed_by_newline() {
                    write!(
                        f,
                        [
                            FormatDanglingComments::Comments {
                                comments,
                                indent: DanglingIndentMode::None
                            },
                            hard_line_break()
                        ]
                    );
                } else {
                    write!(
                        f,
                        [
                            FormatDanglingComments::Comments {
                                comments: rest,
                                indent: DanglingIndentMode::None
                            },
                            (!rest.is_empty()).then_some(hard_line_break()),
                            group(&format_args!(
                                FormatDanglingComments::Comments {
                                    comments: std::slice::from_ref(last),
                                    indent: DanglingIndentMode::None
                                },
                                soft_line_break_or_space()
                            ))
                        ]
                    );
                }
            }

            if let Some(readonly) = self.readonly() {
                let prefix = match readonly {
                    TSMappedTypeModifierOperator::True => "",
                    TSMappedTypeModifierOperator::Plus => "+",
                    TSMappedTypeModifierOperator::Minus => "-",
                };
                write!(f, [prefix, "readonly", space()]);
            }

            let format_key_in_constraint = format_with(|f| {
                write!(f, [key, space(), "in", space(), constraint]);
                if let Some(name_type) = &name_type {
                    write!(f, [space(), "as", space(), name_type]);
                }
                key.format_trailing_comments(f);
            });

            write!(f, ["[", group(&soft_block_indent(&format_key_in_constraint)), "]"]);
            if let Some(optional) = self.optional() {
                write!(
                    f,
                    match optional {
                        TSMappedTypeModifierOperator::True => "?",
                        TSMappedTypeModifierOperator::Plus => "+?",
                        TSMappedTypeModifierOperator::Minus => "-?",
                    }
                );
            }
            if let Some(type_annotation) = &self.type_annotation() {
                write!(f, [":", space(), type_annotation]);
            }
            write!(f, if_group_breaks(&OptionalSemicolon));
        });

        let should_insert_space_around_brackets = f.options().bracket_spacing.value();
        write!(
            f,
            [
                "{",
                group(&soft_block_indent_with_maybe_space(
                    &format_inner,
                    should_insert_space_around_brackets
                ))
                .should_expand(should_expand),
                "}",
            ]
        );
    }
}
