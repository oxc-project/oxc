use std::iter;

use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{
        Buffer, BufferExtensions, Format, Formatter, VecBuffer,
        prelude::{FormatElements, format_once, line_suffix_boundary, *},
        trivia::FormatTrailingComments,
    },
    utils::{
        format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
        member_chain::is_member_call_chain,
        object::{format_property_key, write_member_name},
    },
    write,
    write::{BinaryLikeExpression, FormatJsArrowFunctionExpressionOptions, FormatWrite},
};

use super::string::{FormatLiteralStringToken, StringLiteralParentKind};

#[derive(Clone, Copy)]
pub enum AssignmentLike<'a, 'b> {
    VariableDeclarator(&'b AstNode<'a, VariableDeclarator<'a>>),
    AssignmentExpression(&'b AstNode<'a, AssignmentExpression<'a>>),
    ObjectProperty(&'b AstNode<'a, ObjectProperty<'a>>),
    BindingProperty(&'b AstNode<'a, BindingProperty<'a>>),
    PropertyDefinition(&'b AstNode<'a, PropertyDefinition<'a>>),
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

/// Based on Prettier's behavior:
/// <https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/comments/handle-comments.js#L853-L883>
fn format_left_trailing_comments(
    start: u32,
    should_print_as_leading: bool,
    f: &mut Formatter<'_, '_>,
) {
    let end_of_line_comments = f.context().comments().end_of_line_comments_after(start);

    let comments = if end_of_line_comments.is_empty() {
        let comments = f.context().comments().comments_before_character(start, b'=');
        if comments.iter().any(|c| c.preceded_by_newline()) { &[] } else { comments }
    } else if should_print_as_leading || end_of_line_comments.last().is_some_and(|c| c.is_block()) {
        // No trailing comments for these expressions or if the trailing comment is a block comment
        &[]
    } else {
        end_of_line_comments
    };

    FormatTrailingComments::Comments(comments).fmt(f);
}

fn should_print_as_leading(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::TemplateLiteral(_)
            | Expression::TaggedTemplateExpression(_)
    )
}

/// The minimum number of overlapping characters between left and right hand side
const MIN_OVERLAP_FOR_BREAK: u8 = 3;

impl<'a> AssignmentLike<'a, '_> {
    fn write_left(&self, f: &mut Formatter<'_, 'a>) -> bool {
        match self {
            AssignmentLike::VariableDeclarator(declarator) => {
                if let Some(init) = &declarator.init {
                    write!(f, [FormatNodeWithoutTrailingComments(&declarator.id())]);
                    format_left_trailing_comments(
                        declarator.id.span().end,
                        should_print_as_leading(init),
                        f,
                    );
                } else {
                    write!(f, declarator.id());
                }
                false
            }
            AssignmentLike::AssignmentExpression(assignment) => {
                write!(f, [FormatNodeWithoutTrailingComments(&assignment.left()),]);
                format_left_trailing_comments(
                    assignment.left.span().end,
                    should_print_as_leading(&assignment.right),
                    f,
                );
                false
            }
            AssignmentLike::ObjectProperty(property) => {
                let text_width_for_break =
                    (f.options().indent_width.value() + MIN_OVERLAP_FOR_BREAK) as usize;

                // Handle computed properties
                if property.computed {
                    write!(f, ["[", property.key(), "]"]);
                    if property.shorthand {
                        false
                    } else {
                        f.source_text().span_width(property.key.span()) + 2 < text_width_for_break
                    }
                } else if property.shorthand {
                    write!(f, property.key());
                    false
                } else {
                    let width = write_member_name(property.key(), f);

                    width < text_width_for_break
                }
            }
            AssignmentLike::BindingProperty(property) => {
                if property.shorthand {
                    // Left-hand side only. See the explanation in the `has_only_left_hand_side` method.
                    if matches!(
                        property.value.kind,
                        BindingPatternKind::BindingIdentifier(_)
                            | BindingPatternKind::AssignmentPattern(_)
                    ) {
                        write!(f, property.value());
                    }
                    return false;
                }

                let text_width_for_break =
                    (f.options().indent_width.value() + MIN_OVERLAP_FOR_BREAK) as usize;

                // Handle computed properties
                if property.computed {
                    write!(f, ["[", property.key(), "]"]);
                    if property.shorthand {
                        false
                    } else {
                        f.source_text().span_width(property.key.span()) + 2 < text_width_for_break
                    }
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

                // Write the property key
                if property.computed {
                    write!(f, ["[", property.key(), "]"]);
                } else {
                    format_property_key(property.key(), f);
                }

                // Write optional, definite, and type annotation
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
            AssignmentLike::TSTypeAliasDeclaration(declaration) => {
                write!(f, [declaration.declare.then_some("declare "), "type "]);

                let start = if let Some(type_parameters) = &declaration.type_parameters() {
                    write!(
                        f,
                        [declaration.id(), FormatNodeWithoutTrailingComments(type_parameters)]
                    );
                    type_parameters.span.end
                } else {
                    write!(f, [FormatNodeWithoutTrailingComments(declaration.id())]);
                    declaration.id.span.end
                };

                format_left_trailing_comments(
                    start,
                    matches!(&declaration.type_annotation, TSType::TSTypeLiteral(_)),
                    f,
                );

                false
            }
        }
    }

    fn write_operator(&self, f: &mut Formatter<'_, 'a>) {
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
                write!(f, [":", space()]);
            }
            Self::BindingProperty(property) => {
                if !property.shorthand {
                    write!(f, [":", space()]);
                }
            }
            Self::PropertyDefinition(property_class_member) => {
                debug_assert!(property_class_member.value().is_some());
                write!(f, [space(), "="]);
            }
            Self::TSTypeAliasDeclaration(_) => {
                write!(f, [space(), "="]);
            }
        }
    }

    fn write_right(&self, f: &mut Formatter<'_, 'a>, layout: AssignmentLikeLayout) {
        match self {
            Self::VariableDeclarator(declarator) => {
                write!(
                    f,
                    [space(), with_assignment_layout(declarator.init().unwrap(), Some(layout))]
                );
            }
            Self::AssignmentExpression(assignment) => {
                let right = assignment.right();
                write!(f, [space(), with_assignment_layout(right, Some(layout))]);
            }
            Self::ObjectProperty(property) => {
                let value = property.value();
                write!(f, [with_assignment_layout(value, Some(layout))]);
            }
            Self::BindingProperty(property) => {
                write!(f, property.value());
            }
            Self::PropertyDefinition(property) => {
                write!(
                    f,
                    [space(), with_assignment_layout(property.value().unwrap(), Some(layout))]
                );
            }
            Self::TSTypeAliasDeclaration(declaration) => {
                if let AstNodes::TSUnionType(union) = declaration.type_annotation().as_ast_nodes() {
                    union.write(f);
                    union.format_trailing_comments(f);
                } else {
                    write!(f, [space(), declaration.type_annotation()]);
                }
            }
        }
    }

    /// Returns the layout variant for an assignment like depending on right expression and left part length
    /// [Prettier applies]: <https://github.com/prettier/prettier/blob/main/src/language-js/print/assignment.js>
    fn layout(
        &self,
        is_left_short: bool,
        left_may_break: bool,
        f: &Formatter<'_, 'a>,
    ) -> AssignmentLikeLayout {
        let right_expression = self.get_right_expression();
        if let Some(expr) = right_expression {
            if let Some(layout) = self.chain_formatting_layout(expr) {
                return layout;
            }

            if let Expression::CallExpression(call_expression) = expr.as_ref()
                && call_expression
                    .callee
                    .get_identifier_reference()
                    .is_some_and(|ident| ident.name == "require")
                && !f.comments().has_leading_own_line_comment(call_expression.span.start)
            {
                return AssignmentLikeLayout::NeverBreakAfterOperator;
            }
        }

        if self.should_break_left_hand_side(left_may_break) {
            return AssignmentLikeLayout::BreakLeftHandSide;
        }

        if self.should_break_after_operator(right_expression, f) {
            return AssignmentLikeLayout::BreakAfterOperator;
        }

        if self.is_complex_type_alias() {
            return AssignmentLikeLayout::BreakLeftHandSide;
        }

        if is_left_short {
            return AssignmentLikeLayout::NeverBreakAfterOperator;
        }

        // Before checking `BreakAfterOperator` layout, we need to unwrap the right expression from `JsUnaryExpression` or `TsNonNullAssertionExpression`
        // [Prettier applies]: https://github.com/prettier/prettier/blob/a043ac0d733c4d53f980aa73807a63fc914f23bd/src/language-js/print/assignment.js#L199-L211
        // Example:
        //  !"123" -> "123"
        //  void "123" -> "123"
        //  !!"string"! -> "string"
        let right_expression =
            iter::successors(right_expression, |expression| match expression.as_ast_nodes() {
                AstNodes::UnaryExpression(unary) => Some(unary.argument()),
                AstNodes::TSNonNullExpression(assertion) => Some(assertion.expression()),
                _ => None,
            })
            .last();

        if matches!(right_expression.map(AsRef::as_ref), Some(Expression::StringLiteral(_))) {
            return AssignmentLikeLayout::BreakAfterOperator;
        }

        let is_poorly_breakable =
            right_expression.is_some_and(|expr| is_poorly_breakable_member_or_call_chain(expr, f));

        if is_poorly_breakable {
            return AssignmentLikeLayout::BreakAfterOperator;
        }

        if !left_may_break
            && matches!(
                right_expression.map(AsRef::as_ref),
                Some(
                    Expression::ClassExpression(_)
                        | Expression::TemplateLiteral(_)
                        | Expression::TaggedTemplateExpression(_)
                        | Expression::BooleanLiteral(_)
                        | Expression::NumericLiteral(_)
                )
            )
        {
            return AssignmentLikeLayout::NeverBreakAfterOperator;
        }

        AssignmentLikeLayout::Fluid
    }

    fn get_right_expression(&self) -> Option<&AstNode<'a, Expression<'a>>> {
        match self {
            AssignmentLike::VariableDeclarator(variable_decorator) => variable_decorator.init(),
            AssignmentLike::AssignmentExpression(assignment) => Some(assignment.right()),
            AssignmentLike::ObjectProperty(property) => Some(property.value()),
            AssignmentLike::PropertyDefinition(property_class_member) => {
                property_class_member.value()
            }
            AssignmentLike::BindingProperty(_) | AssignmentLike::TSTypeAliasDeclaration(_) => None,
        }
    }

    /// Checks that a [AssignmentLike] consists only of the left part
    /// usually, when a [variable declarator](VariableDeclarator) doesn't have initializer
    fn has_only_left_hand_side(&self) -> bool {
        match self {
            Self::AssignmentExpression(_) | Self::TSTypeAliasDeclaration(_) => false,
            Self::VariableDeclarator(declarator) => declarator.init.is_none(),
            Self::PropertyDefinition(property) => property.value().is_none(),
            Self::BindingProperty(property) => {
                property.shorthand
                    && matches!(
                        property.value.kind,
                        BindingPatternKind::BindingIdentifier(_)
                        // Treats binding property has a left-hand side only
                        // when the value is an assignment pattern,
                        // because the `value` includes the `key` part.
                        // e.g., `{ a = 1 }` the `a` is the `key` and `a = 1` is the
                        // `value`, aka AssignmentPattern itself
                        | BindingPatternKind::AssignmentPattern(_)
                    )
            }
            Self::ObjectProperty(property) => property.shorthand,
        }
    }

    /// and if so, it return the layout type
    fn chain_formatting_layout(
        &self,
        right_expression: &Expression,
    ) -> Option<AssignmentLikeLayout> {
        let right_is_tail = !matches!(right_expression, Expression::AssignmentExpression(_));

        // The chain goes up two levels, by checking up to the great parent if all the conditions
        // are correctly met.
        let upper_chain_is_eligible =
            // First, we check if the current node is an assignment expression
            if let Self::AssignmentExpression(assignment) = self {
                // Then we check if the parent is assignment expression or variable declarator
                let parent = assignment.parent;
                // Determine if the chain is eligible based on the following checks:
                // 1. For variable declarators: only continue if this isn't the final assignment in the chain
                (matches!(parent, AstNodes::VariableDeclarator(_)) && !right_is_tail) ||
                // 2. For assignment expressions: continue unless this is the final assignment in an expression statement
                matches!(parent, AstNodes::AssignmentExpression(parent_assignment)
                    if !right_is_tail || !matches!(parent_assignment.parent, AstNodes::ExpressionStatement(_))
                )
            } else {
                false
            };

        if upper_chain_is_eligible {
            if right_is_tail {
                match right_expression {
                    Expression::ArrowFunctionExpression(arrow) => {
                        if arrow.expression {
                            let Statement::ExpressionStatement(stmt) = &arrow.body.statements[0]
                            else {
                                unreachable!()
                            };
                            if matches!(&stmt.expression, Expression::ArrowFunctionExpression(_)) {
                                return Some(AssignmentLikeLayout::ChainTailArrowFunction);
                            }
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
    fn should_break_left_hand_side(&self, left_may_break: bool) -> bool {
        if self.is_complex_destructuring() {
            return true;
        }

        let Self::VariableDeclarator(declarator) = self else {
            return false;
        };

        let type_annotation = declarator.id.type_annotation.as_ref();

        type_annotation.is_some_and(|ann| is_complex_type_annotation(ann))
            || (left_may_break
                && declarator
                    .init
                    .as_ref()
                    .is_some_and(|expr| matches!(expr, Expression::ArrowFunctionExpression(_))))
    }

    /// Checks if the current assignment is eligible for [AssignmentLikeLayout::BreakAfterOperator]
    ///
    /// This function is small wrapper around [should_break_after_operator] because it has to work
    /// for nodes that belong to TypeScript too.
    fn should_break_after_operator(
        &self,
        right_expression: Option<&AstNode<'a, Expression<'a>>>,
        f: &Formatter<'_, 'a>,
    ) -> bool {
        let comments = f.context().comments();
        if let Some(right_expression) = right_expression {
            should_break_after_operator(right_expression, f)
        } else if let AssignmentLike::TSTypeAliasDeclaration(decl) = self {
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
                        || comments.has_comment_before(decl.type_annotation.span().start)
                }
                // `TSUnionType` has its own indentation logic
                TSType::TSUnionType(_) => false,
                _ => {
                    // Check for leading comments on any other type
                    comments.has_comment_before(decl.type_annotation.span().start)
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
                let BindingPatternKind::ObjectPattern(object) = &variable_decorator.id.kind else {
                    return false;
                };

                if object.len() <= 2 {
                    return false;
                }

                object.properties.iter().any(|property| {
                    !property.shorthand || property.value.kind.is_assignment_pattern()
                })
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
            | AssignmentLike::TSTypeAliasDeclaration(_) => false,
        }
    }
}

/// Checks if the function is entitled to be printed with layout [AssignmentLikeLayout::BreakAfterOperator]
fn should_break_after_operator<'a>(
    right: &AstNode<'a, Expression<'a>>,
    f: &Formatter<'_, 'a>,
) -> bool {
    if right.is_jsx() {
        return false;
    }

    let comments = f.context().comments();
    for comment in comments.comments_before_iter(right.span().start) {
        if comment.has_newlines_around() {
            return true;
        }

        // Needs to wrap a parenthesis for the node, so it won't break.
        if comments.is_type_cast_comment(comment) {
            return false;
        }
    }

    match right.as_ref() {
        // head is a long chain, meaning that right -> right are both assignment expressions
        Expression::AssignmentExpression(assignment) => {
            matches!(assignment.right, Expression::AssignmentExpression(_))
        }
        Expression::BinaryExpression(_) | Expression::SequenceExpression(_) => true,
        Expression::LogicalExpression(logical) => {
            !BinaryLikeExpression::can_inline_logical_expr(logical)
        }
        Expression::ConditionalExpression(conditional) => match &conditional.test {
            Expression::BinaryExpression(_) => true,
            Expression::LogicalExpression(logical) => {
                !BinaryLikeExpression::can_inline_logical_expr(logical)
            }
            _ => false,
        },
        Expression::ClassExpression(class) => !class.decorators.is_empty(),

        _ => {
            let argument = match right.as_ast_nodes() {
                AstNodes::AwaitExpression(expression) => Some(expression.argument()),
                AstNodes::YieldExpression(expression) => expression.argument(),
                AstNodes::UnaryExpression(expression) => {
                    let argument = get_last_non_unary_argument(expression);
                    match argument.as_ast_nodes() {
                        AstNodes::AwaitExpression(expression) => Some(expression.argument()),
                        AstNodes::YieldExpression(expression) => expression.argument(),
                        _ => Some(argument),
                    }
                }
                _ => None,
            };

            argument.is_some_and(|argument| {
                argument.is_literal() || is_poorly_breakable_member_or_call_chain(argument, f)
            })
        }
    }
}

/// Iterate over unary expression arguments to get last non-unary
/// Example: void !!(await test()) -> returns await as last argument
fn get_last_non_unary_argument<'a, 'b>(
    unary_expression: &'b AstNode<'a, UnaryExpression<'a>>,
) -> &'b AstNode<'a, Expression<'a>> {
    let mut argument = unary_expression.argument();

    while let AstNodes::UnaryExpression(unary) = argument.as_ast_nodes() {
        argument = unary.argument();
    }

    argument
}

impl<'a> Format<'a> for AssignmentLike<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        // If there's only left hand side, we just write it and return
        if self.has_only_left_hand_side() {
            self.write_left(f);
            return;
        }

        let format_content = format_with(|f| {
            // We create a temporary buffer because the left hand side has to conditionally add
            // a group based on the layout, but the layout can only be computed by knowing the
            // width of the left hand side. The left hand side can be a member, and that has a width
            // can can be known only when it's formatted (it can incur in some transformation,
            // like removing some escapes, etc.).
            //
            // 1. we crate a temporary buffer
            // 2. we write the left hand side into the buffer and retrieve the `is_left_short` info
            // which is computed only when we format it
            // 3. we compute the layout
            // 4. we write the left node inside the main buffer based on the layout
            let mut buffer = VecBuffer::new(f.state_mut());
            let is_left_short = self.write_left(&mut Formatter::new(&mut buffer));
            let formatted_left = buffer.into_vec();
            let left_may_break = formatted_left.may_directly_break();

            let left = format_once(|f| f.write_elements(formatted_left));

            // Compare name only if we are in a position of computing it.
            // If not (for example, left is not an identifier), then let's fallback to false,
            // so we can continue the chain of checks
            let layout = self.layout(is_left_short, left_may_break, f);
            let right = format_with(|f| self.write_right(f, layout));

            let inner_content = format_with(|f| {
                if matches!(&layout, AssignmentLikeLayout::BreakLeftHandSide) {
                    write!(f, [left]);
                } else {
                    write!(f, [group(&left)]);
                }

                if layout != AssignmentLikeLayout::SuppressedInitializer {
                    self.write_operator(f);
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
                        write!(f, [group(&soft_line_indent_or_space(&right))]);
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

impl<'a> Format<'a> for WithAssignmentLayout<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        match self.expression.as_ast_nodes() {
            AstNodes::ArrowFunctionExpression(arrow) => arrow.fmt_with_options(
                FormatJsArrowFunctionExpressionOptions {
                    assignment_layout: self.layout,
                    ..FormatJsArrowFunctionExpressionOptions::default()
                },
                f,
            ),
            _ => self.expression.fmt(f),
        }
    }
}

/// A chain that has no calls at all or all of whose calls have no arguments
/// or have only one which [is_short_argument], except for member call chains
/// [Prettier applies]: <https://github.com/prettier/prettier/blob/a043ac0d733c4d53f980aa73807a63fc914f23bd/src/language-js/print/assignment.js#L329>
fn is_poorly_breakable_member_or_call_chain<'a>(
    expression: &AstNode<'a, Expression<'a>>,
    f: &Formatter<'_, 'a>,
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

        let is_breakable_type_arguments = match &call_expression.type_arguments {
            Some(type_arguments) => is_complex_type_arguments(type_arguments),
            None => false,
        };

        if is_breakable_type_arguments {
            return false;
        }
    }

    !is_member_call_chain(call_expressions[0], f)
}

/// This function checks if `JsAnyCallArgument` is short
/// We need it to decide if `JsCallExpression` with the argument is breakable or not
/// If the argument is short the function call isn't breakable
/// [Prettier applies]: <https://github.com/prettier/prettier/blob/a043ac0d733c4d53f980aa73807a63fc914f23bd/src/language-js/print/assignment.js#L374>
fn is_short_argument(expression: &Expression, threshold: u16, f: &Formatter) -> bool {
    match expression {
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
        Expression::ThisExpression(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NumericLiteral(_) => true,
        _ => false,
    }
}

/// This function checks if `TSTypeArguments` is complex
/// We need it to decide if `CallExpression` with the type arguments is breakable or not
/// If the type arguments is complex the function call is breakable
/// [Prettier applies]: <https://github.com/prettier/prettier/blob/a043ac0d733c4d53f980aa73807a63fc914f23bd/src/language-js/print/assignment.js#L432>
fn is_complex_type_arguments(type_arguments: &TSTypeParameterInstantiation) -> bool {
    let ts_type_argument_list = &type_arguments.params;

    if ts_type_argument_list.len() > 1 {
        return true;
    }

    let is_first_argument_complex = ts_type_argument_list.first().is_some_and(|first_argument| {
        matches!(
            first_argument,
            TSType::TSUnionType(_) | TSType::TSIntersectionType(_) | TSType::TSTypeLiteral(_)
        )
    });

    if is_first_argument_complex {
        return true;
    }

    // TODO: add here will_break logic
    // https://github.com/prettier/prettier/blob/a043ac0d733c4d53f980aa73807a63fc914f23bd/src/language-js/print/assignment.js#L454

    false
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
