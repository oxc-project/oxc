use oxc_ast::ast::*;

use crate::{
    formatter::{
        Buffer, Comments, Format, FormatResult, Formatter,
        prelude::{
            format_dangling_comments, format_with, group, soft_block_indent_with_maybe_space,
        },
    },
    generated::ast_nodes::AstNode,
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

impl<'a> ObjectPatternLike<'a, '_> {
    fn span(&self) -> Span {
        match self {
            Self::ObjectPattern(o) => o.span(),
            Self::ObjectAssignmentTarget(o) => o.span(),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::ObjectPattern(o) => o.is_empty(),
            Self::ObjectAssignmentTarget(o) => o.is_empty(),
        }
    }

    fn is_inline(&self, comments: &Comments) -> bool {
        // TODO
        false
        // let parent_kind = self.syntax().parent.kind();

        // Ok(
        // (matches!(parent_kind, Some(JsSyntaxKind::JS_FORMAL_PARAMETER))
        // || self.is_hug_parameter(comments))
        // && !self.l_curly_token()?.leading_trivia().has_skipped(),
        // )
    }

    fn should_break_properties(&self) -> bool {
        false
        // TODO
        // let parent_kind = self.syntax().parent.kind();

        // Catch only has a single expression in the declaration, so it will
        // be the direct parent of the object pattern, and the pattern should
        // not break unless it has to there.
        //
        // Parameters in function-like expressions are also kept inline, and
        // all parameters end up wrapped by a `JsFormalParameter` node, as
        // checked here. Note that this is also checked ahead of time by the
        // `is_inline` function.
        // if matches!(
        // parent_kind,
        // Some(JsSyntaxKind::JS_CATCH_DECLARATION | JsSyntaxKind::JS_FORMAL_PARAMETER)
        // ) {
        // return false;
        // }

        // match self {
        // JsObjectPatternLike::JsObjectAssignmentPattern(node) => {
        // node.properties().iter().any(|property| {
        // if let Ok(
        // AnyJsObjectAssignmentPatternMember::JsObjectAssignmentPatternProperty(node),
        // ) = property
        // {
        // let pattern = node.pattern();
        // matches!(
        // pattern,
        // Ok(AnyJsAssignmentPattern::JsObjectAssignmentPattern(_)
        // | AnyJsAssignmentPattern::JsArrayAssignmentPattern(_))
        // )
        // } else {
        // false
        // }
        // })
        // }
        // JsObjectPatternLike::JsObjectBindingPattern(node) => {
        // node.properties().iter().any(|property| {
        // if let Ok(AnyJsObjectBindingPatternMember::JsObjectBindingPatternProperty(
        // node,
        // )) = property
        // {
        // let pattern = node.pattern();

        // matches!(
        // pattern,
        // Ok(AnyJsBindingPattern::JsObjectBindingPattern(_)
        // | AnyJsBindingPattern::JsArrayBindingPattern(_))
        // )
        // } else {
        // false
        // }
        // })
        // }
        // }
    }

    fn is_in_assignment_like(&self) -> bool {
        // TODO
        false
        // matches!(
        // self.syntax().parent.kind(),
        // Some(JsSyntaxKind::JS_ASSIGNMENT_EXPRESSION | JsSyntaxKind::JS_VARIABLE_DECLARATOR),
        // )
    }

    fn is_hug_parameter(&self, comments: &Comments) -> bool {
        false
        // match self {
        // JsObjectPatternLike::JsObjectAssignmentPattern(_) => false,
        // JsObjectPatternLike::JsObjectBindingPattern(binding) => binding
        // .parent::<AnyJsFormalParameter>()
        // .and_then(|parameter| parameter.syntax().grand_parent())
        // .and_then(FormatAnyJsParameters::cast)
        // .is_some_and(|parameters| {
        // should_hug_function_parameters(&parameters, comments, false).unwrap_or(false)
        // }),
        // }
    }

    fn layout(&self, comments: &Comments) -> ObjectPatternLayout {
        if self.is_empty() {
            return ObjectPatternLayout::Empty;
        }

        if self.is_inline(comments) {
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

        match self.layout(f.comments()) {
            ObjectPatternLayout::Empty => {
                write!(f, format_dangling_comments(self.span()).with_soft_block_indent())?;
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
