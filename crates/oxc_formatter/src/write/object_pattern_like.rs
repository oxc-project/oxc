use oxc_ast::ast::*;

use crate::{
    formatter::{
        Buffer, Comments, Format, FormatResult, Formatter,
        prelude::{format_with, group, soft_block_indent_with_maybe_space},
        trivia::{DanglingIndentMode, format_dangling_comments},
    },
    generated::ast_nodes::{AstNode, AstNodes},
    write,
    write::parameter_list::should_hug_function_parameters,
};

use super::{
    assignment_pattern_property_list::AssignmentTargetPropertyList,
    binding_property_list::BindingPropertyList,
};

pub enum ObjectPatternLike<'a, 'b> {
    ObjectPattern(&'b AstNode<'a, ObjectPattern<'a>>),
    ObjectAssignmentTarget(&'b AstNode<'a, ObjectAssignmentTarget<'a>>),
}

impl<'a> ObjectPatternLike<'a, '_> {
    fn span(&self) -> Span {
        match self {
            Self::ObjectPattern(o) => o.span,
            Self::ObjectAssignmentTarget(o) => o.span,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::ObjectPattern(o) => o.is_empty(),
            Self::ObjectAssignmentTarget(o) => o.is_empty(),
        }
    }

    fn is_inline(&self, f: &Formatter<'_, 'a>) -> bool {
        match self {
            Self::ObjectPattern(node) => {
                matches!(node.parent, AstNodes::FormalParameter(_)) || self.is_hug_parameter(f)
            }
            Self::ObjectAssignmentTarget(node) => {
                matches!(node.parent, AstNodes::FormalParameter(_))
            }
        }
    }

    fn should_break_properties(&self) -> bool {
        // Check parent node type
        let parent_is_catch_or_parameter = match self {
            Self::ObjectPattern(node) => {
                matches!(node.parent, AstNodes::CatchParameter(_) | AstNodes::FormalParameter(_))
            }
            Self::ObjectAssignmentTarget(node) => {
                matches!(node.parent, AstNodes::CatchParameter(_) | AstNodes::FormalParameter(_))
            }
        };

        if parent_is_catch_or_parameter {
            return false;
        }

        match self {
            Self::ObjectPattern(node) => {
                node.properties.iter().any(|property| match &property.value.kind {
                    BindingPatternKind::ObjectPattern(_) | BindingPatternKind::ArrayPattern(_) => {
                        true
                    }
                    BindingPatternKind::AssignmentPattern(assignment) => {
                        matches!(
                            assignment.left.kind,
                            BindingPatternKind::ObjectPattern(_)
                                | BindingPatternKind::ArrayPattern(_)
                        )
                    }
                    BindingPatternKind::BindingIdentifier(_) => false,
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

    fn is_hug_parameter(&self, f: &Formatter<'_, 'a>) -> bool {
        match self {
            Self::ObjectAssignmentTarget(_) => false,
            Self::ObjectPattern(node) => {
                matches!(node.parent, AstNodes::FormalParameter(param) if {
                    matches!(param.parent, AstNodes::FormalParameters(params) if {
                        should_hug_function_parameters(params, false, f)
                    })
                })
            }
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

    fn write_properties(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::ObjectPattern(o) => BindingPropertyList::new(o.properties(), o.rest()).fmt(f),
            Self::ObjectAssignmentTarget(o) => {
                AssignmentTargetPropertyList::new(o.properties(), o.rest()).fmt(f)
            }
        }
    }
}

impl<'a> Format<'a> for ObjectPatternLike<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let should_insert_space_around_brackets = f.options().bracket_spacing.value();
        let format_properties = format_with(|f| {
            write!(
                f,
                soft_block_indent_with_maybe_space(
                    &format_with(|f| self.write_properties(f)),
                    should_insert_space_around_brackets
                )
            )
        });

        write!(f, ["{"])?;

        match self.layout(f) {
            ObjectPatternLayout::Empty => {
                write!(f, format_dangling_comments(self.span()).with_block_indent())?;
            }
            ObjectPatternLayout::Inline => {
                write!(f, format_properties)?;
            }
            ObjectPatternLayout::Group { expand } => {
                write!(f, group(&format_properties).should_expand(expand))?;
            }
        }

        write!(f, "}")
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
