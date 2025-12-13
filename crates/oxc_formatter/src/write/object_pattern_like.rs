use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{
        Buffer, Format, Formatter,
        prelude::{format_with, group, soft_block_indent_with_maybe_space},
        trivia::format_dangling_comments,
    },
    write,
};

use super::{
    assignment_pattern_property_list::AssignmentTargetPropertyList,
    binding_property_list::BindingPropertyList,
};

pub enum ObjectPatternLike<'a, 'b> {
    ObjectPattern(&'b AstNode<'a, ObjectPattern<'a>>),
    ObjectAssignmentTarget(&'b AstNode<'a, ObjectAssignmentTarget<'a>>),
}

impl GetSpan for ObjectPatternLike<'_, '_> {
    fn span(&self) -> Span {
        match self {
            Self::ObjectPattern(node) => node.span,
            Self::ObjectAssignmentTarget(node) => node.span,
        }
    }
}

impl<'a> ObjectPatternLike<'a, '_> {
    fn is_empty(&self) -> bool {
        match self {
            Self::ObjectPattern(o) => o.is_empty(),
            Self::ObjectAssignmentTarget(o) => o.is_empty(),
        }
    }

    fn is_inline(&self, _f: &Formatter<'_, 'a>) -> bool {
        match self {
            Self::ObjectPattern(node) => match node.parent {
                AstNodes::FormalParameter(_) => true,
                AstNodes::AssignmentPattern(_) => {
                    matches!(node.parent.parent(), AstNodes::FormalParameter(_))
                }
                _ => false,
            },
            Self::ObjectAssignmentTarget(_) => false,
        }
    }

    /// Based on <https://github.com/prettier/prettier/blob/2d6877fcd1b78f2624e22d0ddb17a895ab12ac07/src/language-js/print/object.js#L77-L103>
    fn should_break_properties(&self) -> bool {
        match self {
            Self::ObjectPattern(node) => {
                let parent_is_parameter_or_assignment_pattern = matches!(
                    node.parent,
                    AstNodes::CatchParameter(_)
                        | AstNodes::FormalParameter(_)
                        | AstNodes::AssignmentPattern(_)
                );

                if parent_is_parameter_or_assignment_pattern {
                    return false;
                }

                node.properties.iter().any(|property| {
                    matches!(
                        property.value.kind,
                        BindingPatternKind::ArrayPattern(_) | BindingPatternKind::ObjectPattern(_)
                    )
                })
            }
            Self::ObjectAssignmentTarget(node) => {
                node.properties.iter().any(|property| match property {
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(_) => false,
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                        matches!(
                            &prop.binding,
                            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(_)
                                | AssignmentTargetMaybeDefault::ArrayAssignmentTarget(_)
                        )
                    }
                })
            }
        }
    }

    fn is_in_assignment_like(&self) -> bool {
        match self {
            Self::ObjectPattern(node) => matches!(node.parent, AstNodes::VariableDeclarator(_)),
            Self::ObjectAssignmentTarget(node) => matches!(
                node.parent,
                AstNodes::AssignmentExpression(_) | AstNodes::VariableDeclarator(_)
            ),
        }
    }

    fn layout(&self, f: &Formatter<'_, 'a>) -> ObjectPatternLayout {
        if self.is_empty() {
            return ObjectPatternLayout::Empty;
        }

        if self.is_inline(f) {
            return ObjectPatternLayout::Inline;
        }

        let break_properties = self.should_break_properties();

        if break_properties {
            ObjectPatternLayout::Group { expand: true }
        } else if self.is_in_assignment_like() {
            ObjectPatternLayout::Inline
        } else {
            ObjectPatternLayout::Group { expand: false }
        }
    }

    fn write_properties(&self, f: &mut Formatter<'_, 'a>) {
        match self {
            Self::ObjectPattern(o) => BindingPropertyList::new(o.properties(), o.rest()).fmt(f),
            Self::ObjectAssignmentTarget(o) => {
                AssignmentTargetPropertyList::new(o.properties(), o.rest()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for ObjectPatternLike<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let should_insert_space_around_brackets = f.options().bracket_spacing.value();
        let format_properties = format_with(|f| {
            write!(
                f,
                soft_block_indent_with_maybe_space(
                    &format_with(|f| self.write_properties(f)),
                    should_insert_space_around_brackets
                )
            );
        });

        write!(f, ["{"]);

        match self.layout(f) {
            ObjectPatternLayout::Empty => {
                write!(f, format_dangling_comments(self.span()).with_block_indent());
            }
            ObjectPatternLayout::Inline => {
                write!(f, format_properties);
            }
            ObjectPatternLayout::Group { expand } => {
                write!(f, group(&format_properties).should_expand(expand));
            }
        }

        write!(f, "}");
    }
}

#[derive(Debug, Copy, Clone)]
enum ObjectPatternLayout {
    /// Wrap the properties in a group with `should_expand` equal to `expand`.
    ///
    /// This is the default layout when no other special case applies.
    Group { expand: bool },

    /// Layout for a pattern without any properties.
    Empty,

    /// Don't wrap the properties in a group and instead "inline" them in the parent.
    ///
    /// Desired if the pattern is a parameter of a function that `should hug` OR
    /// if the pattern is the left side of an assignment.
    Inline,
}
