use oxc_ast::ast::{TSMappedType, TSMappedTypeModifierOperator};

use crate::{
    FormatResult,
    formatter::{Formatter, SourceText, prelude::*, trivia::FormatLeadingComments},
    generated::ast_nodes::AstNode,
    write,
    write::semicolon::OptionalSemicolon,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, TSMappedType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let type_parameter = self.type_parameter();
        let name_type = self.name_type();
        let should_expand = has_line_break_before_property_name(self, f.source_text());

        let type_annotation_has_leading_comment =
            f.comments().has_comment_before(type_parameter.span.start);

        let format_inner = format_with(|f| {
            if should_expand {
                let comments =
                    f.context().comments().comments_before_character(self.span.start, b'[');
                write!(f, FormatLeadingComments::Comments(comments))?;
            }

            if let Some(readonly) = self.readonly() {
                let prefix = match readonly {
                    TSMappedTypeModifierOperator::True => "",
                    TSMappedTypeModifierOperator::Plus => "+",
                    TSMappedTypeModifierOperator::Minus => "-",
                };
                write!(f, [prefix, "readonly", space()])?;
            }

            let format_inner_inner = format_with(|f| {
                write!(f, "[")?;
                write!(f, type_parameter.name())?;
                if let Some(constraint) = &type_parameter.constraint() {
                    write!(f, [space(), "in", space(), constraint])?;
                }
                if let Some(default) = &type_parameter.default() {
                    write!(f, [space(), "=", space(), default])?;
                }
                if let Some(name_type) = &name_type {
                    write!(f, [space(), "as", space(), name_type])?;
                }
                write!(f, "]")?;
                if let Some(optional) = self.optional() {
                    write!(
                        f,
                        match optional {
                            TSMappedTypeModifierOperator::True => "?",
                            TSMappedTypeModifierOperator::Plus => "+?",
                            TSMappedTypeModifierOperator::Minus => "-?",
                        }
                    )?;
                }
                Ok(())
            });

            write!(f, [space(), group(&format_inner_inner)])?;
            if let Some(type_annotation) = &self.type_annotation() {
                write!(f, [":", space(), type_annotation])?;
            }
            write!(f, if_group_breaks(&OptionalSemicolon))
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
        )
    }
}

/// Check if the user introduced a new line inside the node, but only if
/// that new line occurs at or before the property name. For example,
/// this would break:
///   { [
///     A in B]: T}
/// Because the line break occurs before `A`, the property name. But this
/// would _not_ break:
///   { [A
///     in B]: T}
/// Because the break is _after_ the `A`.
fn has_line_break_before_property_name(node: &TSMappedType, f: SourceText) -> bool {
    f.contains_newline_between(node.span.start, node.type_parameter.span.start)
}
