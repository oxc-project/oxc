use oxc_ast::ast::*;
use oxc_formatter_core::{Buffer, BufferExtensions, Format, ScratchBuffer};
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{
        JsFormatter,
        prelude::{FormatElements, format_once, line_suffix_boundary, *},
        trivia::{FormatTrailingComments, is_alignable_block_comment},
    },
    print::{
        BinaryLikeExpression, alias_union_breaks_after_operator,
        is_line_ending_trailing_jsdoc_comment, type_alias_left_end, union_prints_itself,
    },
    utils::{
        format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
        member_chain::is_member_call_chain,
        object::{format_property_key, write_member_name},
        typecast::is_cast_target,
    },
    write,
};

use super::string::{FormatLiteralStringToken, StringLiteralParentKind};

#[derive(Clone, Copy)]
pub enum AssignmentLike<'a, 'b> {
    VariableDeclarator(&'b AstNode<'a, VariableDeclarator<'a>>),
    AssignmentExpression(&'b AstNode<'a, AssignmentExpression<'a>>),
    ObjectProperty(&'b AstNode<'a, ObjectProperty<'a>>),
    BindingProperty(&'b AstNode<'a, BindingProperty<'a>>),
    PropertyDefinition(&'b AstNode<'a, PropertyDefinition<'a>>),
    AccessorProperty(&'b AstNode<'a, AccessorProperty<'a>>),
    TSTypeAliasDeclaration(&'b AstNode<'a, TSTypeAliasDeclaration<'a>>),
}

/// Determines how a assignment like be formatted
///
/// Assignment like are:
/// - Assignment
/// - Object property member
/// - Variable declaration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentLikeLayout {
    /// First break right-hand side, then after operator.
    /// ```js
    /// {
    ///   "array-key": [
    ///     {
    ///       "nested-key-1": 1,
    ///       "nested-key-2": 2,
    ///     },
    ///   ]
    /// }
    /// ```
    Fluid,

    /// First break after operator, then the sides are broken independently on their own lines.
    /// There is a soft line break after operator token.
    /// ```js
    /// {
    ///     "enough-long-key-to-break-line":
    ///         1 + 2,
    ///     "not-long-enough-key":
    ///         "but long enough string to break line",
    /// }
    /// ```
    BreakAfterOperator,

    /// First break right-hand side, then left-hand side. There are not any soft line breaks
    /// between left and right parts
    /// ```js
    /// {
    ///     key1: "123",
    ///     key2: 123,
    ///     key3: class MyClass {
    ///        constructor() {},
    ///     },
    /// }
    /// ```
    NeverBreakAfterOperator,

    /// This is a special layout usually used for long variable declarations or assignment expressions
    /// This layout is hit, usually, when we are in the "middle" of the chain:
    ///
    /// ```js
    /// var a =
    ///     loreum =
    ///     ipsum =
    ///         "foo";
    /// ```
    ///
    /// Given the previous snippet, then `loreum` and `ipsum` will be formatted using the `Chain` layout.
    Chain,

    /// This is a special layout usually used for long variable declarations or assignment expressions
    /// This layout is hit, usually, when we are in the end of a chain:
    /// ```js
    /// var a = loreum = ipsum = "foo";
    /// ```
    ///
    /// Given the previous snippet, then `"foo"` formatted  using the `ChainTail` layout.
    ChainTail,

    /// This layout is used in cases where we want to "break" the left hand side
    /// of assignment like expression, but only when the group decides to do it.
    ///
    /// ```js
    /// const a {
    ///     loreum: { ipsum },
    ///     something_else,
    ///     happy_days: { fonzy }
    /// } = obj;
    /// ```
    ///
    /// The snippet triggers the layout because the left hand side contains a "complex destructuring"
    /// which requires having the properties broke on different lines.
    BreakLeftHandSide,

    /// This is a special case of the "chain" layout collection. This is triggered when there's
    /// a series of simple assignments (at least three) and in the middle we have an arrow function
    /// and this function followed by two more arrow functions.
    ///
    /// This layout will break the right hand side of the tail on a new line and add a new level
    /// of indentation
    ///
    /// ```js
    /// lorem =
    ///     fff =
    ///     ee =
    ///         () => (fff) => () => (fefef) => () => fff;
    /// ```
    ChainTailArrowFunction,

    /// Layout used when the operator and right hand side are part of a `JsInitializerClause<
    /// that has a suppression comment.
    SuppressedInitializer,
}

/// The left side's trailing comments up to the operator (see DIVERGENCES.md#eol-comment-after-assign-colon).
fn format_left_trailing_comments(start: u32, f: &mut JsFormatter<'_, '_>) {
    let end_of_line_comments = f.context().comments().end_of_line_comments_after(start);

    let comments = if end_of_line_comments.is_empty() {
        let comments = f.context().comments().comments_before_character(start, b'=');
        if comments.iter().any(|c| c.preceded_by_newline()) { &[] } else { comments }
    } else if end_of_line_comments.last().is_some_and(|c| c.is_multiline_block()) {
        // A line-ending multiline block is promoted own-line above the right-hand side
        &[]
    } else {
        end_of_line_comments
    };

    FormatTrailingComments::Comments(comments).fmt(f);
}

/// The minimum number of overlapping characters between left and right hand side
const MIN_OVERLAP_FOR_BREAK: u8 = 3;

impl<'a> AssignmentLike<'a, '_> {
    fn write_left(&self, f: &mut JsFormatter<'_, 'a>) -> bool {
        match self {
            AssignmentLike::VariableDeclarator(declarator) => {
                if declarator.init.is_some() {
                    write!(
                        f,
                        [
                            FormatNodeWithoutTrailingComments(&declarator.id()),
                            declarator.type_annotation()
                        ]
                    );
                    format_left_trailing_comments(declarator.id.span().end, f);
                } else {
                    write!(
                        f,
                        [
                            declarator.id(),
                            declarator.definite.then_some("!"),
                            declarator.type_annotation()
                        ]
                    );
                }
                false
            }
            AssignmentLike::AssignmentExpression(assignment) => {
                write!(f, [FormatNodeWithoutTrailingComments(&assignment.left()),]);
                format_left_trailing_comments(assignment.left.span().end, f);
                false
            }
            AssignmentLike::ObjectProperty(property) => {
                let text_width_for_break =
                    (f.options().indent_width.value() + MIN_OVERLAP_FOR_BREAK) as usize;

                // Handle computed properties
                if property.computed {
                    write!(f, ["[", property.key(), "]"]);
                    f.source_text().span_width(property.key.span()) + 2 < text_width_for_break
                } else if property.shorthand {
                    let PropertyKey::StaticIdentifier(ident) = &property.key else {
                        unreachable!("Expected static identifier for shorthand property");
                    };
                    write!(f, text(ident.name.as_str()));
                    false
                } else {
                    let width = write_member_name(property.key(), f);

                    width < text_width_for_break
                }
            }
            AssignmentLike::BindingProperty(property) => {
                if property.shorthand {
                    // Left-hand side only. See the explanation in the `has_only_left_hand_side` method.
                    if property.value.is_binding_identifier()
                        || property.value.is_assignment_pattern()
                    {
                        write!(f, property.value());
                    }
                    return false;
                }

                let text_width_for_break =
                    (f.options().indent_width.value() + MIN_OVERLAP_FOR_BREAK) as usize;

                // Handle computed properties
                if property.computed {
                    write!(f, ["[", property.key(), "]"]);
                    f.source_text().span_width(property.key.span()) + 2 < text_width_for_break
                } else {
                    let width = write_member_name(property.key(), f);

                    width < text_width_for_break
                }
            }
            AssignmentLike::PropertyDefinition(property) => {
                write!(f, [property.decorators()]);

                if property.declare {
                    write!(f, ["declare", space()]);
                }
                if let Some(accessibility) = property.accessibility {
                    write!(f, [accessibility.as_str(), space()]);
                }
                if property.r#static {
                    write!(f, ["static", space()]);
                }
                if property.r#type == PropertyDefinitionType::TSAbstractPropertyDefinition {
                    write!(f, ["abstract", space()]);
                }
                if property.r#override {
                    write!(f, ["override", space()]);
                }
                if property.readonly {
                    write!(f, ["readonly", space()]);
                }
                format_property_key(property.key(), property.computed, f);
                if property.optional {
                    write!(f, "?");
                }
                if property.definite {
                    write!(f, "!");
                }
                if let Some(type_annotation) = property.type_annotation() {
                    write!(f, type_annotation);
                }

                false // Class properties don't use "short" key logic
            }
            AssignmentLike::AccessorProperty(property) => {
                write!(f, [property.decorators()]);

                if let Some(accessibility) = property.accessibility {
                    write!(f, [accessibility.as_str(), space()]);
                }
                if property.r#static {
                    write!(f, ["static", space()]);
                }
                if property.r#type.is_abstract() {
                    write!(f, ["abstract", space()]);
                }
                if property.r#override {
                    write!(f, ["override", space()]);
                }
                write!(f, ["accessor", space()]);
                format_property_key(property.key(), property.computed, f);
                if property.definite {
                    write!(f, "!");
                }
                if let Some(type_annotation) = property.type_annotation() {
                    write!(f, type_annotation);
                }

                false // Class properties don't use "short" key logic
            }
            AssignmentLike::TSTypeAliasDeclaration(declaration) => {
                write!(f, [declaration.declare.then_some("declare "), "type "]);

                if let Some(type_parameters) = &declaration.type_parameters() {
                    write!(
                        f,
                        [declaration.id(), FormatNodeWithoutTrailingComments(type_parameters)]
                    );
                } else {
                    write!(f, [FormatNodeWithoutTrailingComments(declaration.id())]);
                }
                let start = type_alias_left_end(declaration);

                format_left_trailing_comments(start, f);

                false
            }
        }
    }

    fn write_operator(&self, f: &mut JsFormatter<'_, 'a>) {
        match self {
            Self::VariableDeclarator(variable_declarator) => {
                debug_assert!(variable_declarator.init.is_some());
                write!(f, [space(), "="]);
            }
            Self::AssignmentExpression(assignment) => {
                let operator = assignment.operator.as_str();
                write!(f, [space(), operator]);
            }
            Self::ObjectProperty(property) => {
                debug_assert!(!property.shorthand);
                write!(f, [":"]);
            }
            Self::BindingProperty(property) => {
                if !property.shorthand {
                    write!(f, [":"]);
                }
            }
            Self::PropertyDefinition(property_class_member) => {
                debug_assert!(property_class_member.value().is_some());
                write!(f, [space(), "="]);
            }
            Self::AccessorProperty(property) => {
                debug_assert!(property.value().is_some());
                write!(f, [space(), "="]);
            }
            Self::TSTypeAliasDeclaration(_) => {
                write!(f, [space(), "="]);
            }
        }
    }

    fn write_right(&self, f: &mut JsFormatter<'_, 'a>, layout: AssignmentLikeLayout) {
        match self {
            Self::VariableDeclarator(declarator) => {
                write!(f, [with_assignment_layout(declarator.init().unwrap(), Some(layout))]);
            }
            Self::AssignmentExpression(assignment) => {
                let right = assignment.right();
                write!(f, [with_assignment_layout(right, Some(layout))]);
            }
            Self::ObjectProperty(property) => {
                let value = property.value();
                write!(f, [with_assignment_layout(value, Some(layout))]);
            }
            Self::BindingProperty(property) => {
                write!(f, property.value());
            }
            Self::PropertyDefinition(property) => {
                write!(f, [with_assignment_layout(property.value().unwrap(), Some(layout))]);
            }
            Self::AccessorProperty(property) => {
                write!(f, [with_assignment_layout(property.value().unwrap(), Some(layout))]);
            }
            Self::TSTypeAliasDeclaration(declaration) => {
                write!(f, [declaration.type_annotation()]);
            }
        }
    }

    /// Returns the layout variant for an assignment like depending on right expression and left part length
    /// [Prettier applies]: <https://github.com/prettier/prettier/blob/main/src/language-js/print/assignment.js>
    fn layout(
        &self,
        is_left_short: bool,
        left_may_break: bool,
        has_line_comment_on_operator_line: bool,
        f: &mut JsFormatter<'_, 'a>,
    ) -> AssignmentLikeLayout {
        let right_shape = self.right_expression_shape(f);
        if let Some(layout) = self.chain_formatting_layout(right_shape, f) {
            return layout;
        }

        if self.right_has_leading_alignable_block_comment(f) {
            return AssignmentLikeLayout::BreakAfterOperator;
        }

        if let Some(Expression::CallExpression(call_expression)) = right_shape
            && call_expression
                .callee
                .get_identifier_reference()
                .is_some_and(|ident| ident.name == "require")
            && !f.comments().has_leading_own_line_comment(call_expression.span.start)
        {
            return AssignmentLikeLayout::NeverBreakAfterOperator;
        }

        if self.should_break_left_hand_side(right_shape, left_may_break) {
            return AssignmentLikeLayout::BreakLeftHandSide;
        }

        // An alias-level union that runs its own printer owns the break and indentation under the `=`;
        // a suppressed one prints verbatim and is laid out like any other type.
        let alias_union_prints_itself = matches!(
            self,
            AssignmentLike::TSTypeAliasDeclaration(decl)
                if union_prints_itself(&decl.type_annotation, f.comments())
        );

        if has_line_comment_on_operator_line
            || self.should_break_after_operator(
                self.get_right_expression(),
                is_left_short,
                alias_union_prints_itself,
                f,
            )
        {
            return AssignmentLikeLayout::BreakAfterOperator;
        }

        if self.is_complex_type_alias() {
            return AssignmentLikeLayout::BreakLeftHandSide;
        }

        // The union's leading soft line break per member and one indent level;
        // the fluid group would stack a second indent on top when a long left-hand side makes it break before the union does.
        // (The end-of-line-comment case takes `BreakAfterOperator` above.)
        if alias_union_prints_itself {
            return AssignmentLikeLayout::NeverBreakAfterOperator;
        }

        if !left_may_break
            && (is_left_short
                || matches!(
                    right_shape,
                    Some(
                        Expression::ClassExpression(_)
                            | Expression::TemplateLiteral(_)
                            | Expression::TaggedTemplateExpression(_)
                            | Expression::BooleanLiteral(_)
                            | Expression::NumericLiteral(_)
                    )
                ))
        {
            return AssignmentLikeLayout::NeverBreakAfterOperator;
        }

        AssignmentLikeLayout::Fluid
    }

    /// The right expression as the shape-based layout rules see it.
    /// `None` for a cast target ([`is_cast_target`]): Prettier's `chooseLayout` sees a `ParenthesizedExpression` there.
    fn right_expression_shape(&self, f: &JsFormatter<'_, 'a>) -> Option<&Expression<'a>> {
        self.get_right_expression()
            .filter(|expr| !is_cast_target(expr.span(), f))
            .map(AsRef::as_ref)
    }

    /// A leading multi-line `*`-aligned block comment on the right-hand side
    /// breaks after the operator ahead of every shape rule (prettier/prettier#19180).
    /// Reads the unprinted view, so it runs after `write_left`.
    /// An alias-level union sees the same comment and hands its indent over (`alias_union_breaks_after_operator`).
    fn right_has_leading_alignable_block_comment(&self, f: &JsFormatter<'_, 'a>) -> bool {
        self.right_start().is_some_and(|right_start| {
            f.comments()
                .comments_before_iter(right_start)
                .any(|comment| is_alignable_block_comment(comment, f.source_text()))
        })
    }

    /// Start of the RHS: the value, the type annotation of a type alias.
    fn right_start(&self) -> Option<u32> {
        let span = match self {
            AssignmentLike::VariableDeclarator(declarator) => declarator.init.as_ref()?.span(),
            AssignmentLike::AssignmentExpression(assignment) => assignment.right.span(),
            AssignmentLike::ObjectProperty(property) => property.value.span(),
            AssignmentLike::BindingProperty(property) => property.value.span(),
            AssignmentLike::PropertyDefinition(property) => property.value.as_ref()?.span(),
            AssignmentLike::AccessorProperty(property) => property.value.as_ref()?.span(),
            AssignmentLike::TSTypeAliasDeclaration(decl) => decl.type_annotation.span(),
        };
        Some(span.start)
    }

    fn get_right_expression(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        match self {
            AssignmentLike::VariableDeclarator(variable_decorator) => variable_decorator.init(),
            AssignmentLike::AssignmentExpression(assignment) => Some(assignment.right()),
            AssignmentLike::ObjectProperty(property) => Some(property.value()),
            AssignmentLike::PropertyDefinition(property_class_member) => {
                property_class_member.value()
            }
            AssignmentLike::AccessorProperty(property) => property.value(),
            AssignmentLike::BindingProperty(_) | AssignmentLike::TSTypeAliasDeclaration(_) => None,
        }
    }

    /// Position just past the operator (`=`, `:`) when a comment sits between the left side and the right-hand side;
    /// `None` otherwise, nothing to hide or reorder then.
    fn commented_operator_position(&self, f: &JsFormatter<'_, 'a>) -> Option<u32> {
        let right_start = self.right_start()?;
        let comments = f.context().comments();
        if !comments.has_any_comment_in_range(self.left_end(), right_start) {
            return None;
        }
        let operator = match self {
            Self::ObjectProperty(_) | Self::BindingProperty(_) => b':',
            _ => b'=',
        };
        Some(comments.position_after_character(self.left_end(), operator))
    }

    /// The comments glued after the operator up to a line comment ending its line (`= /* c */ // d`),
    /// printed right after the operator (see `Comments::mark_suppressed_after_operator` for their suppression target).
    /// Comments the left side deferred are still pending: nothing glues, all lead the right-hand side in order.
    fn operator_line_run(operator_end: u32, f: &JsFormatter<'_, 'a>) -> &'a [Comment] {
        if f.context().comments().has_comment_before(operator_end) {
            return &[];
        }
        let run = f.context().comments().end_of_line_comments_after(operator_end);
        if run.last().is_some_and(|c| c.is_line()) { run } else { &[] }
    }

    /// End of the left-hand side (type annotation included), before the operator and any comments around it.
    /// Distinct from the comment-scan start in `write_left`, which begins at the id.
    fn left_end(&self) -> u32 {
        match self {
            Self::VariableDeclarator(declarator) => declarator
                .type_annotation
                .as_ref()
                .map_or(declarator.id.span().end, |annotation| annotation.span.end),
            Self::AssignmentExpression(assignment) => assignment.left.span().end,
            Self::ObjectProperty(property) => property.key.span().end,
            Self::BindingProperty(property) => property.key.span().end,
            Self::PropertyDefinition(property) => property
                .type_annotation
                .as_ref()
                .map_or(property.key.span().end, |annotation| annotation.span.end),
            Self::AccessorProperty(property) => property
                .type_annotation
                .as_ref()
                .map_or(property.key.span().end, |annotation| annotation.span.end),
            Self::TSTypeAliasDeclaration(declaration) => type_alias_left_end(declaration),
        }
    }

    /// Checks that a [AssignmentLike] consists only of the left part usually,
    /// when a [variable declarator](VariableDeclarator) doesn't have initializer.
    fn has_only_left_hand_side(&self) -> bool {
        match self {
            Self::AssignmentExpression(_) | Self::TSTypeAliasDeclaration(_) => false,
            Self::VariableDeclarator(declarator) => declarator.init.is_none(),
            Self::PropertyDefinition(property) => property.value().is_none(),
            Self::AccessorProperty(property) => property.value().is_none(),
            Self::BindingProperty(property) => {
                // Treats binding property has a left-hand side only
                // when the value is an assignment pattern,
                // because the `value` includes the `key` part.
                // e.g., `{ a = 1 }` the `a` is the `key` and `a = 1` is the
                // `value`, aka AssignmentPattern itself
                property.shorthand
                    && (property.value.is_binding_identifier()
                        || property.value.is_assignment_pattern())
            }
            Self::ObjectProperty(property) => property.shorthand,
        }
    }

    /// and if so, it return the layout type
    fn chain_formatting_layout(
        &self,
        right_shape: Option<&Expression>,
        f: &JsFormatter<'_, 'a>,
    ) -> Option<AssignmentLikeLayout> {
        let right_is_tail = !matches!(right_shape, Some(Expression::AssignmentExpression(_)));

        // The chain goes up two levels, by checking up to the great parent if all the conditions
        // are correctly met.
        let upper_chain_is_eligible =
            // First, we check if the current node is an assignment expression and not a cast target:
            // its parent would be Prettier's `ParenthesizedExpression`, which breaks the chain.
            if let Self::AssignmentExpression(assignment) = self
                && !is_cast_target(assignment.span, f)
            {
                // Then we check if the parent is assignment expression or variable declarator
                let parent = assignment.parent();
                // Determine if the chain is eligible based on the following checks:
                // 1. For variable declarators: only continue if this isn't the final assignment in the chain
                (matches!(parent, AstNodes::VariableDeclarator(_)) && !right_is_tail) ||
                // 2. For assignment expressions: continue unless this is the final assignment
                // in an expression statement or concise arrow body.
                matches!(parent, AstNodes::AssignmentExpression(parent_assignment)
                    if !right_is_tail || !matches!(
                        parent_assignment.parent(),
                        AstNodes::ArrowFunctionExpression(_) | AstNodes::ExpressionStatement(_)
                    )
                )
            } else {
                false
            };

        if upper_chain_is_eligible {
            if right_is_tail {
                match right_shape {
                    Some(Expression::ArrowFunctionExpression(arrow)) => {
                        if matches!(
                            arrow.get_expression(),
                            Some(Expression::ArrowFunctionExpression(_))
                        ) {
                            return Some(AssignmentLikeLayout::ChainTailArrowFunction);
                        }
                        Some(AssignmentLikeLayout::ChainTail)
                    }
                    _ => Some(AssignmentLikeLayout::ChainTail),
                }
            } else {
                Some(AssignmentLikeLayout::Chain)
            }
        } else {
            None
        }
    }

    /// Particular function that checks if the left hand side of a [AssignmentLike] should
    /// be broken on multiple lines
    fn should_break_left_hand_side(
        &self,
        right_shape: Option<&Expression>,
        left_may_break: bool,
    ) -> bool {
        if self.is_complex_destructuring() {
            return true;
        }

        let Self::VariableDeclarator(declarator) = self else {
            return false;
        };

        let type_annotation = declarator.type_annotation.as_ref();

        type_annotation.is_some_and(|ann| is_complex_type_annotation(ann))
            || (left_may_break
                && matches!(right_shape, Some(Expression::ArrowFunctionExpression(_))))
    }

    /// Checks if the current assignment is eligible for [AssignmentLikeLayout::BreakAfterOperator]
    ///
    /// This function is small wrapper around [should_break_after_operator] because it has to work
    /// for nodes that belong to TypeScript too.
    fn should_break_after_operator(
        &self,
        right_expression: Option<&AstNode<'a, Expression<'a>>>,
        is_left_short: bool,
        alias_union_prints_itself: bool,
        f: &mut JsFormatter<'_, 'a>,
    ) -> bool {
        let comments = f.context().comments();
        if let Some(right_expression) = right_expression {
            should_break_after_operator(right_expression, is_left_short, f)
        } else if let AssignmentLike::TSTypeAliasDeclaration(decl) = self {
            let annotation_start = decl.type_annotation.span().start;
            // For TSTypeAliasDeclaration, check if the type annotation is a union type with comments
            match &decl.type_annotation {
                TSType::TSConditionalType(conditional_type) => {
                    let is_generic = |ts_type: &TSType<'a>| -> bool {
                        match ts_type {
                            TSType::TSFunctionType(function) => function.type_parameters.is_some(),
                            TSType::TSTypeReference(reference) => {
                                reference.type_arguments.is_some()
                            }
                            _ => false,
                        }
                    };
                    is_generic(&conditional_type.check_type)
                        || is_generic(&conditional_type.extends_type)
                        || comments.has_leading_own_line_comment(annotation_start)
                }
                // `TSUnionType` has its own indentation logic,
                // EXCEPT when the union suppresses it and relies on the operator-side break + indent instead.
                TSType::TSUnionType(_) if alias_union_prints_itself => {
                    alias_union_breaks_after_operator(
                        decl,
                        comments
                            .comments_before_iter(annotation_start)
                            .any(is_line_ending_trailing_jsdoc_comment),
                        comments,
                    )
                }
                // For a single-member `TSIntersectionType`,
                // we need to check for leading own-line comments before the type inside the `TSIntersectionType`.
                // This is because Prettier treats a single-member `TSIntersectionType` as the literal type inside of it,
                // so it checks whether there's a leading own-line comment before the start of the literal type.
                TSType::TSIntersectionType(intersection_type)
                    if intersection_type.types.len() == 1 =>
                {
                    comments.has_leading_own_line_comment(
                        intersection_type.types.first().unwrap().span().start,
                    )
                }
                _ => {
                    // Only comments on their own line force the break
                    comments.has_leading_own_line_comment(annotation_start)
                }
            }
        } else {
            false
        }
    }

    fn is_complex_type_alias(&self) -> bool {
        let AssignmentLike::TSTypeAliasDeclaration(type_alias) = self else {
            return false;
        };

        let Some(type_parameters) = &type_alias.type_parameters else {
            return false;
        };

        type_parameters.params.len() > 1
            && type_parameters
                .params
                .iter()
                .any(|param| param.constraint.is_some() || param.default.is_some())
    }

    fn is_complex_destructuring(&self) -> bool {
        match self {
            AssignmentLike::VariableDeclarator(variable_decorator) => {
                let BindingPattern::ObjectPattern(object) = &variable_decorator.id else {
                    return false;
                };

                if object.len() <= 2 {
                    return false;
                }

                object
                    .properties
                    .iter()
                    .any(|property| !property.shorthand || property.value.is_assignment_pattern())
            }
            AssignmentLike::AssignmentExpression(assignment) => {
                let AssignmentTarget::ObjectAssignmentTarget(object) = &assignment.left else {
                    return false;
                };

                if object.len() <= 2 {
                    return false;
                }

                object.properties.iter().any(|property| match property {
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(
                        property_identifier,
                    ) => property_identifier.init.is_some(),
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(_) => true,
                })
            }
            AssignmentLike::ObjectProperty(_)
            | AssignmentLike::BindingProperty(_)
            | AssignmentLike::PropertyDefinition(_)
            | AssignmentLike::AccessorProperty(_)
            | AssignmentLike::TSTypeAliasDeclaration(_) => false,
        }
    }
}

/// Checks if the function is entitled to be printed with layout [AssignmentLikeLayout::BreakAfterOperator]
///
/// Based on <https://github.com/prettier/prettier/blob/0273e33fc691e28e4ab3f3c8ee86918b65cf823d/src/language-js/print/assignment.js#L196-L264>
fn should_break_after_operator<'a>(
    right: &AstNode<'a, Expression<'a>>,
    is_left_short: bool,
    f: &mut JsFormatter<'_, 'a>,
) -> bool {
    if right.is_jsx() {
        return false;
    }

    let comments = f.context().comments();
    for comment in comments.comments_before_iter(right.span().start) {
        // Like Prettier's `hasLeadingOwnLineComment` (the same rule as the type-alias arms):
        // only a comment on its own line (followed by a newline) forces the break.
        if comment.followed_by_newline() {
            return true;
        }

        // A tight type-cast comment hugs its parenthesized node,
        // so it doesn't force the break itself,
        // and the comments after it sit inside the cast's parentheses and belong to the inner node, stop scanning;
        // the shape-based rules below still apply.
        if comments.is_type_cast_comment(comment) {
            break;
        }
    }

    // A cast-wrapped RHS has no shape (`x = /** @type {T} */ (a || b);` stays inline)
    if is_cast_target(right.span(), f) {
        return false;
    }

    match right.as_ref() {
        // head is a long chain, meaning that right -> right are both assignment expressions
        // (a cast-wrapped right is opaque, not an assignment)
        Expression::AssignmentExpression(assignment) => {
            matches!(assignment.right, Expression::AssignmentExpression(_))
                && !is_cast_target(assignment.right.span(), f)
        }
        Expression::BinaryExpression(_) | Expression::SequenceExpression(_) => true,
        Expression::LogicalExpression(logical) => {
            !BinaryLikeExpression::can_inline_logical_expr(logical)
        }
        Expression::ConditionalExpression(conditional) => match &conditional.test {
            // A cast-parenthesized test is opaque, same as the whole-RHS case above
            test if is_cast_target(test.span(), f) => false,
            Expression::BinaryExpression(_) => true,
            Expression::LogicalExpression(logical) => {
                !BinaryLikeExpression::can_inline_logical_expr(logical)
            }
            _ => false,
        },
        Expression::ClassExpression(class) => !class.decorators.is_empty(),
        // Based on https://github.com/prettier/prettier/blob/0273e33fc691e28e4ab3f3c8ee86918b65cf823d/src/language-js/print/assignment.js#L235-L263
        _ if is_left_short => false,
        _ => get_innermost_expression(right, f).is_some_and(|inner| {
            matches!(inner.as_ref(), Expression::StringLiteral(_))
                || is_poorly_breakable_member_or_call_chain(inner, f)
        }),
    }
}

/// Traverses nested unary-like expressions to find the innermost one.
///
/// Example: `void !!(await test())` returns the `await test()` expression.
///
/// `None` when the walk reaches a cast target (`!/** @type {T} */ ("s")`): a cast target has no shape.
/// The caller has already checked the entry node.
fn get_innermost_expression<'a, 'b>(
    mut current: &'b AstNode<'a, Expression<'a>>,
    f: &JsFormatter<'_, 'a>,
) -> Option<&'b AstNode<'a, Expression<'a>>> {
    loop {
        let argument = match current.as_ast_nodes() {
            AstNodes::UnaryExpression(unary) => unary.argument(),
            AstNodes::TSNonNullExpression(non_null) => non_null.expression(),
            AstNodes::AwaitExpression(expr) => expr.argument(),
            AstNodes::YieldExpression(expr) => match expr.argument() {
                Some(argument) => argument,
                None => break,
            },
            _ => break,
        };
        if is_cast_target(argument.span(), f) {
            return None;
        }
        current = argument;
    }

    Some(current)
}

impl<'a> Format<'a, JsFormatContext<'a>> for AssignmentLike<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        // If there's only left hand side, we just write it and return
        if self.has_only_left_hand_side() {
            self.write_left(f);
            return;
        }

        let format_content = format_with(|f| {
            // We create a temporary buffer because the left hand side has to conditionally add a group based on the layout,
            // but the layout can only be computed by knowing the width of the left hand side.
            // The left hand side can be a member, and that has a width can can be known only when it's formatted
            // (it can incur in some transformation, like removing some escapes, etc.).
            //
            // 1. we create a scratch accumulator as a temporary heap buffer
            //    (see `AccumulatorBuffer` for why neither the arena nor the shared scratch fits)
            // 2. we write the left hand side into the buffer and retrieve the `is_left_short` info which is computed only when we format it
            // 3. we compute the layout
            // 4. we write the left node inside the main buffer based on the layout
            let mut formatted_left = ScratchBuffer::new();
            // The left side's trailing run stops at the operator:
            // comments past it are hidden while it prints and print after the operator (`operator_line_run`) or lead the right-hand side.
            // A raw hide, not `format_content_without_comments_after`: its suppression assert would fire
            // for `= // prettier-ignore`, whose target is re-keyed, not lost.
            let operator_end = self.commented_operator_position(f);
            let view_limit =
                operator_end.map(|end| f.context_mut().comments_mut().limit_comments_up_to(end));
            let is_left_short =
                self.write_left(&mut Formatter::new(&mut formatted_left.writer(f.state_mut())));
            if let Some(view_limit) = view_limit {
                f.context_mut().comments_mut().restore_view_limit(view_limit);
            }
            let operator_line_run =
                operator_end.map_or(&[][..], |end| Self::operator_line_run(end, f));
            // A line comment on the operator's line, before it (printed by the left side) or after it (the run):
            // the pending `line_suffix` must be followed by the break, or it flushes past the right-hand side
            let has_line_comment_on_operator_line = !operator_line_run.is_empty()
                || f.context().comments().printed_line_comment_after(self.left_end()).is_some();
            let left_may_break = formatted_left.may_directly_break();

            let left = format_once(move |f| f.write_elements(formatted_left.drain()));

            // Compare name only if we are in a position of computing it.
            // If not (for example, left is not an identifier), then let's fallback to false,
            // so we can continue the chain of checks
            let layout =
                self.layout(is_left_short, left_may_break, has_line_comment_on_operator_line, f);
            let right = format_with(|f| self.write_right(f, layout));

            // Whether `BreakAfterOperator` must keep the comment order around the operator (no group):
            // when a line comment ends the operator's line, and for non-conditional type aliases,
            // those also reach the arm via own-line-comment paths where nothing was printed,
            // and their union interplay needs the ungrouped variant.
            let keeps_comment_order = matches!(
                self,
                AssignmentLike::TSTypeAliasDeclaration(decl)
                    if !matches!(decl.type_annotation, TSType::TSConditionalType(_))
            ) || has_line_comment_on_operator_line;

            let inner_content = format_with(|f| {
                if matches!(&layout, AssignmentLikeLayout::BreakLeftHandSide) {
                    write!(f, [left]);
                } else {
                    write!(f, [group(&left)]);
                }

                if layout != AssignmentLikeLayout::SuppressedInitializer {
                    self.write_operator(f);
                    if !operator_line_run.is_empty() {
                        write!(f, [FormatTrailingComments::Comments(operator_line_run)]);
                        if operator_line_run
                            .iter()
                            .any(|c| f.context().comments().is_suppression_comment(c))
                            && let Some(start) = self.right_start()
                        {
                            f.context_mut().comments_mut().mark_suppressed_after_operator(start);
                        }
                    }
                }

                #[expect(clippy::match_same_arms)]
                match layout {
                    AssignmentLikeLayout::Fluid => {
                        let group_id = f.group_id("assignment_like");
                        write!(
                            f,
                            [
                                group(&indent(&soft_line_break_or_space()))
                                    .with_group_id(Some(group_id)),
                                line_suffix_boundary(),
                                indent_if_group_breaks(&right, group_id)
                            ]
                        );
                    }
                    AssignmentLikeLayout::BreakAfterOperator => {
                        // NOTE: Prettier instead lets the comment flush past a fitting right-hand side (prettier#14617 family).
                        // Ungrouped, the break follows the enclosing group, expanded by the line comment.
                        // No `line_suffix_boundary` here: it would fail the left side's fit measurement
                        // and expand its type arguments (`let a: Foo<X, Y> = // c`).
                        if keeps_comment_order {
                            write!(f, [soft_line_indent_or_space(&right)]);
                        } else {
                            write!(f, [group(&soft_line_indent_or_space(&right))]);
                        }
                    }
                    AssignmentLikeLayout::NeverBreakAfterOperator => {
                        write!(f, [space(), right]);
                    }
                    AssignmentLikeLayout::BreakLeftHandSide => {
                        write!(f, [space(), group(&right)]);
                    }
                    AssignmentLikeLayout::Chain => {
                        write!(f, [soft_line_break_or_space(), right]);
                    }
                    AssignmentLikeLayout::ChainTail => {
                        write!(f, [soft_line_indent_or_space(&right)]);
                    }
                    AssignmentLikeLayout::ChainTailArrowFunction => {
                        write!(f, [space(), right]);
                    }
                    AssignmentLikeLayout::SuppressedInitializer => {
                        unreachable!();
                        // self.write_suppressed_initializer(f)
                    }
                }
            });

            match layout {
                // Layouts that don't need enclosing group
                AssignmentLikeLayout::Chain
                | AssignmentLikeLayout::ChainTail
                | AssignmentLikeLayout::SuppressedInitializer => {
                    write!(f, [&inner_content]);
                }
                _ => {
                    write!(f, [group(&inner_content)]);
                }
            }
        });

        write!(f, [format_content]);
    }
}

/// Formats an expression and passes the assignment layout to its formatting function if the expressions
/// formatting rule takes the layout as an option.
pub struct WithAssignmentLayout<'a, 'b> {
    expression: &'b AstNode<'a, Expression<'a>>,
    layout: Option<AssignmentLikeLayout>,
}

pub fn with_assignment_layout<'a, 'b>(
    expression: &'b AstNode<'a, Expression<'a>>,
    layout: Option<AssignmentLikeLayout>,
) -> WithAssignmentLayout<'a, 'b> {
    WithAssignmentLayout { expression, layout }
}

impl<'a> Format<'a, JsFormatContext<'a>> for WithAssignmentLayout<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        // An arrow needs the layout inside its `write`,
        // which is reached through the shared generated `fmt` (suppression, type casts, parentheses, comments);
        // the span-keyed context slot hands it across that frame.
        // A cast target has no shape; the layout never reaches the arrow inside.
        if let (Some(layout), AstNodes::ArrowFunctionExpression(arrow)) =
            (self.layout, self.expression.as_ast_nodes())
            && !is_cast_target(arrow.span(), f)
        {
            f.context_mut().set_arrow_assignment_layout(arrow.span(), layout);
            arrow.fmt(f);
            f.context_mut().clear_arrow_assignment_layout();
        } else {
            self.expression.fmt(f);
        }
    }
}

/// A chain that has no calls at all or all of whose calls have no arguments
/// or have only one which [is_short_argument], except for member call chains
/// [Prettier applies]: <https://github.com/prettier/prettier/blob/a043ac0d733c4d53f980aa73807a63fc914f23bd/src/language-js/print/assignment.js#L329>
fn is_poorly_breakable_member_or_call_chain<'a>(
    expression: &AstNode<'a, Expression<'a>>,
    f: &mut JsFormatter<'_, 'a>,
) -> bool {
    let threshold = f.options().line_width.value() / 4;

    // Only call and member chains are poorly breakable
    // - `obj.member.prop`
    // - `obj.member()()`
    let mut is_chain = false;

    // Only chains with simple head are poorly breakable
    // Simple head is `JsIdentifierExpression` or `JsThisExpression`
    let mut is_chain_head_simple = false;

    // Keeping track of all call expressions in the chain to check them later
    let mut call_expressions = vec![];

    let mut expression = expression.as_ast_nodes();

    loop {
        // A cast target ends the chain without a simple head
        if is_cast_target(expression.span(), f) {
            break;
        }
        expression = match expression {
            AstNodes::TSNonNullExpression(assertion) => assertion.expression().as_ast_nodes(),
            AstNodes::CallExpression(call_expression) => {
                is_chain = true;
                let callee = &call_expression.callee();
                call_expressions.push(call_expression);
                callee.as_ast_nodes()
            }
            AstNodes::StaticMemberExpression(node) => {
                is_chain = true;
                node.object().as_ast_nodes()
            }
            AstNodes::ComputedMemberExpression(node) => {
                is_chain = true;
                node.object().as_ast_nodes()
            }
            AstNodes::PrivateFieldExpression(node) => {
                is_chain = true;
                node.object().as_ast_nodes()
            }
            AstNodes::ChainExpression(chain) => {
                is_chain = true;
                chain.expression().as_ast_nodes()
            }
            AstNodes::IdentifierReference(_) | AstNodes::ThisExpression(_) => {
                is_chain_head_simple = true;
                break;
            }
            _ => {
                break;
            }
        }
    }

    if !is_chain || !is_chain_head_simple {
        return false;
    }

    if call_expressions.is_empty() {
        return true;
    }

    if f.comments().has_comment_in_span(call_expressions[0].span) {
        return false;
    }

    for call_expression in &call_expressions {
        let args = &call_expression.arguments;

        let is_breakable_call = match args.len() {
            0 => false,
            1 => match args.iter().next() {
                Some(first_argument) => first_argument
                    .as_expression()
                    .is_none_or(|e| !is_short_argument(e, threshold, f)),
                None => false,
            },
            _ => true,
        };

        if is_breakable_call {
            return false;
        }

        let is_breakable_type_arguments = call_expression
            .type_arguments()
            .is_some_and(|type_arguments| is_complex_type_arguments(type_arguments, f));

        if is_breakable_type_arguments {
            return false;
        }
    }

    !is_member_call_chain(call_expressions[0], f)
}

/// This function checks if [`Argument`] is short
/// We need it to decide if [`CallExpression`] with the argument is breakable or not
/// If the argument is short the function call isn't breakable
/// [Prettier applies]: <https://github.com/prettier/prettier/blob/0273e33fc691e28e4ab3f3c8ee86918b65cf823d/src/language-js/utils/index.js#L433-L484>
fn is_short_argument(argument: &Expression, threshold: u16, f: &JsFormatter) -> bool {
    match argument {
        Expression::Identifier(identifier) => identifier.name.len() <= threshold as usize,
        Expression::UnaryExpression(unary_expression) => {
            is_short_argument(&unary_expression.argument, threshold, f)
        }
        Expression::RegExpLiteral(regex) => regex.regex.pattern.text.len() <= threshold as usize,
        Expression::StringLiteral(literal) => {
            let formatter = FormatLiteralStringToken::new(
                f.source_text().text_for(literal.as_ref()),
                false,
                StringLiteralParentKind::Expression,
            );

            formatter.clean_text(f).width() <= threshold as usize
        }
        Expression::TemplateLiteral(literal) => {
            // Besides checking length exceed we also need to check that the template doesn't have any expressions.
            // It means that the elements of the template are empty or have only one `JsTemplateChunkElement` element
            // Prettier: https://github.com/prettier/prettier/blob/a043ac0d733c4d53f980aa73807a63fc914f23bd/src/language-js/print/assignment.js#L402-L405
            literal.quasis.len() == 1 && {
                let raw = literal.quasis[0].value.raw;
                raw.len() <= threshold as usize && !raw.contains('\n')
            }
        }
        Expression::CallExpression(call) => {
            call.arguments.is_empty()
                && matches!(&call.callee, Expression::Identifier(ident) if ident.name.len() <= (threshold as usize).saturating_sub(2))
        }
        Expression::ThisExpression(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NumericLiteral(_) => true,
        _ => false,
    }
}

/// This function checks if `TSTypeArguments` is complex.
/// We need it to decide if `CallExpression` with the type arguments is breakable or not.
/// If the type arguments is complex the function call is breakable.
///
/// <https://github.com/prettier/prettier/blob/a043ac0d733c4d53f980aa73807a63fc914f23bd/src/language-js/print/assignment.js#L432-L459>
fn is_complex_type_arguments<'a>(
    type_arguments: &AstNode<'a, TSTypeParameterInstantiation<'a>>,
    f: &mut JsFormatter<'_, 'a>,
) -> bool {
    let params = &type_arguments.params;
    if params.len() > 1 {
        return true;
    }

    if params.first().is_some_and(|param| {
        matches!(
            param,
            TSType::TSUnionType(_) | TSType::TSIntersectionType(_) | TSType::TSTypeLiteral(_)
        )
    }) {
        return true;
    }

    // Prettier: `willBreak(print(typeArgsKeyName))`
    f.speculate_will_break(type_arguments)
}

/// [Prettier applies]: <https://github.com/prettier/prettier/blob/fde0b49d7866e203ca748c306808a87b7c15548f/src/language-js/print/assignment.js#L278>
pub fn is_complex_type_annotation(annotation: &TSTypeAnnotation) -> bool {
    match &annotation.type_annotation {
        TSType::TSTypeReference(reference_type) => {
            let Some(type_arguments) = &reference_type.type_arguments else {
                return false;
            };
            let argument_list_len = type_arguments.params.len();

            if argument_list_len <= 1 {
                return false;
            }

            type_arguments
                .params
                .iter()
                .any(|argument| {
                    if matches!(argument, TSType::TSConditionalType(_)) {
                        return true;
                    }

                    let is_complex_type = matches!(
                        argument,
                        TSType::TSTypeReference(reference_type)
                            if reference_type.type_arguments.as_ref().is_some_and(|type_args| !type_args.params.is_empty())
                    );

                    is_complex_type
                })
        }
        _ => false,
    }
}
