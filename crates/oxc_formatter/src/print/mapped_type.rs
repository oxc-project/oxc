use oxc_ast::ast::{TSMappedType, TSMappedTypeModifierOperator};

use crate::{
    ast_nodes::AstNode,
    formatter::{Formatter, SourceText, prelude::*, trivia::FormatLeadingComments},
    print::semicolon::OptionalSemicolon,
    utils::suppressed::FormatSuppressedNode,
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, TSMappedType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        if f.comments().is_suppressed(self.key.span.start) {
            return write!(f, FormatSuppressedNode(self.span));
        }

        let key = self.key();
        let constraint = self.constraint();
        let name_type = self.name_type();
        let should_expand = has_line_break_after_opening_brace(self, f.source_text());

        let format_inner = format_with(|f| {
            if should_expand {
                let comments = if f.comments().has_leading_own_line_comment(self.key.span.start) {
                    f.context().comments().comments_before(self.key.span.start)
                } else {
                    f.context().comments().comments_before_character(self.span.start, b'[')
                };
                write!(f, FormatLeadingComments::Comments(comments));
            }

            if let Some(readonly) = self.readonly() {
                let prefix = match readonly {
                    TSMappedTypeModifierOperator::True => "",
                    TSMappedTypeModifierOperator::Plus => "+",
                    TSMappedTypeModifierOperator::Minus => "-",
                };
                write!(f, [prefix, "readonly", space()]);
            }

            let format_inner_inner = format_with(|f| {
                write!(f, "[");
                write!(f, key);
                write!(f, [space(), "in", space(), constraint]);
                if let Some(name_type) = &name_type {
                    write!(f, [space(), "as", space(), name_type]);
                }
                key.format_trailing_comments(f);
                write!(f, "]");
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
            });

            write!(f, [group(&format_inner_inner)]);
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

/// Check if the user introduced a new line immediately after the opening brace.
/// For example, this would break:
///   {
///     readonly [A in B]: T}
/// Because the line break occurs right after `{`. But this would _not_ break:
///   { readonly
///     [A in B]: T}
/// Because the break is not immediately after `{`.
fn has_line_break_after_opening_brace(node: &TSMappedType, f: SourceText) -> bool {
    // Check if there's a newline immediately after `{` (before any non-whitespace)
    f.has_newline_after(node.span.start + 1)
}
