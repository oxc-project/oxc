use std::iter;

use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::formatter::prelude::{FormatElements, format_once, line_suffix_boundary};
use crate::formatter::{BufferExtensions, VecBuffer};
use crate::write::utils::string_utils::{FormatLiteralStringToken, StringLiteralParentKind};
use crate::write::{BinaryLikeExpression, call_arguments, write};
use crate::{
    format_args,
    formatter::{
        Buffer, Format, FormatResult, Formatter, prelude::*, trivia::format_dangling_comments,
    },
    generated::ast_nodes::{AstNode, AstNodes},
    options::Expand,
    write::arrow_function_expression::{
        AssignmentLikeLayout, FormatJsArrowFunctionExpression,
        FormatJsArrowFunctionExpressionOptions,
    },
};

#[derive(Clone, Copy)]
pub enum AssignmentLike<'a, 'b> {
    VariableDeclarator(&'b AstNode<'a, VariableDeclarator<'a>>),
    AssignmentExpression(&'b AstNode<'a, AssignmentExpression<'a>>),
}

impl<'a> AssignmentLike<'a, '_> {
    fn write_left(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<bool> {
        match self {
            AssignmentLike::VariableDeclarator(variable_declarator) => {
                write!(f, variable_declarator.id())?;
                Ok(false)
            }
            AssignmentLike::AssignmentExpression(assignment) => {
                let left = assignment.left();
                write!(f, [&left]);
                Ok(false)
            }
        }
    }

    fn write_operator(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        #[expect(clippy::match_wildcard_for_single_variants)]
        match self {
            Self::VariableDeclarator(variable_declarator) if variable_declarator.init.is_some() => {
                write!(f, [space(), "="])
            }
            Self::AssignmentExpression(assignment) => {
                let operator = assignment.operator.as_str();
                write!(f, [space(), operator])
            }
            _ => Ok(()),
        }
    }

    fn write_right(
        &self,
        f: &mut Formatter<'_, 'a>,
        layout: AssignmentLikeLayout,
    ) -> FormatResult<()> {
        match self {
            Self::VariableDeclarator(variable_declarator) => {
                if let Some(init) = variable_declarator.init() {
                    write!(f, [space(), with_assignment_layout(init, Some(layout)),])
                } else {
                    Ok(())
                }
            }
            Self::AssignmentExpression(assignment) => {
                let right = assignment.right();
                write!(f, [space(), with_assignment_layout(right, Some(layout))])
            }
        }
    }

    /// Returns the layout variant for an assignment like depending on right expression and left part length
    /// [Prettier applies]: https://github.com/prettier/prettier/blob/main/src/language-js/print/assignment.js
    fn layout(
        &self,
        is_left_short: bool,
        left_may_break: bool,
        f: &mut Formatter<'_, 'a>,
    ) -> AssignmentLikeLayout {
        if self.has_only_left_hand_side() {
            return AssignmentLikeLayout::OnlyLeft;
        }

        // if let RightAssignmentLike::JsInitializerClause(initializer) = &right {
        //     if f.context().comments().is_suppressed(initializer.syntax()) {
        //         return Ok(AssignmentLikeLayout::SuppressedInitializer);
        //     }
        // }
        let right_expression = self.get_right_expression();

        if let Some(layout) = right_expression.and_then(|expr| self.chain_formatting_layout(expr)) {
            return layout;
        }

        if let Some(Expression::CallExpression(call_expression)) = &right_expression {
            if call_expression
                .callee
                .get_identifier_reference()
                .is_some_and(|ident| ident.name == "require")
            {
                return AssignmentLikeLayout::NeverBreakAfterOperator;
            }
        }

        if self.should_break_left_hand_side() {
            return AssignmentLikeLayout::BreakLeftHandSide;
        }

        if self.should_break_after_operator(right_expression, f) {
            return AssignmentLikeLayout::BreakAfterOperator;
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
        let right_expression = iter::successors(right_expression, |expression| match expression {
            Expression::UnaryExpression(unary) => Some(&unary.argument),
            Expression::TSNonNullExpression(assertion) => Some(&assertion.expression),
            _ => None,
        })
        .last();

        if matches!(right_expression, Some(Expression::StringLiteral(_))) {
            return AssignmentLikeLayout::BreakAfterOperator;
        }

        let is_poorly_breakable = match &right_expression {
            Some(expression) => is_poorly_breakable_member_or_call_chain(expression, f),
            None => false,
        };

        if is_poorly_breakable {
            return AssignmentLikeLayout::BreakAfterOperator;
        }

        if !left_may_break
            && matches!(
                right_expression,
                Some(
                    Expression::ClassExpression(_)
                        | Expression::TemplateLiteral(_)
                        | Expression::BooleanLiteral(_)
                        | Expression::NumericLiteral(_)
                )
            )
        {
            return AssignmentLikeLayout::NeverBreakAfterOperator;
        }

        AssignmentLikeLayout::Fluid
    }

    fn get_right_expression(&self) -> Option<&Expression<'a>> {
        match self {
            AssignmentLike::VariableDeclarator(variable_decorator) => {
                variable_decorator.init.as_ref()
            }
            AssignmentLike::AssignmentExpression(assignment) => Some(&assignment.right),
        }
    }

    /// Checks that a [JsAnyAssignmentLike] consists only of the left part
    /// usually, when a [variable declarator](JsVariableDeclarator) doesn't have initializer
    fn has_only_left_hand_side(&self) -> bool {
        match self {
            Self::VariableDeclarator(declarator) => declarator.init.is_none(),
            Self::AssignmentExpression(_) => false,
        }
        // TODO:
        // if let Self::JsPropertyClassMember(class_member) = self {
        //     class_member.value().is_none()
        // } else {
        //     matches!(self, Self::TsPropertySignatureClassMember(_))
        // }
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

    /// Particular function that checks if the left hand side of a [JsAnyAssignmentLike] should
    /// be broken on multiple lines
    fn should_break_left_hand_side(&self) -> bool {
        let is_complex_destructuring = self.is_complex_destructuring();

        // let has_complex_type_annotation = self
        //     .annotation()
        //     .and_then(|annotation| is_complex_type_annotation(annotation).ok())
        let has_complex_type_annotation = false;

        // let is_complex_type_alias = self.is_complex_type_alias()?;
        let is_complex_type_alias = false;

        // let is_right_arrow_func = self.right().is_ok_and(|right| match right {
        //     RightAssignmentLike::JsInitializerClause(init) => {
        //         init.expression().is_ok_and(|expression| {
        //             matches!(expression, Expression::JsArrowFunctionExpression(_))
        //         })
        //     }
        //     _ => false,
        // });
        let is_right_arrow_func = false;

        // let is_breakable = self
        //     .annotation()
        //     .and_then(|annotation| is_annotation_breakable(annotation).ok())
        //     .unwrap_or(false);

        let is_breakable = false;

        is_complex_destructuring
            || has_complex_type_annotation
            || is_complex_type_alias
            || (is_right_arrow_func && is_breakable)
    }

    /// Checks if the current assignment is eligible for [AssignmentLikeLayout::BreakAfterOperator]
    ///
    /// This function is small wrapper around [should_break_after_operator] because it has to work
    /// for nodes that belong to TypeScript too.
    fn should_break_after_operator(
        &self,
        right_expression: Option<&Expression<'a>>,
        f: &Formatter<'_, 'a>,
    ) -> bool {
        let comments = f.context().comments();
        if let Some(right_expression) = right_expression {
            should_break_after_operator(right_expression, f)
        } else {
            // RightAssignmentLike::AnyTsType(AnyTsType::TsUnionType(ty)) => {
            //     // Recursively checks if the union type is nested and identifies the innermost union type.
            //     // If a leading comment is found while navigating to the inner union type,
            //     // it is considered as having leading comments.
            //     let mut union_type = ty.clone();
            //     let mut has_leading_comments = comments.has_leading_comments(union_type.syntax());
            //     while is_nested_union_type(&union_type)? && !has_leading_comments {
            //         if let Some(Ok(inner_union_type)) = union_type.types().last() {
            //             let inner_union_type = TsUnionType::cast(inner_union_type.into_syntax());
            //             if let Some(inner_union_type) = inner_union_type {
            //                 has_leading_comments =
            //                     comments.has_leading_comments(inner_union_type.syntax());
            //                 union_type = inner_union_type;
            //             } else {
            //                 break;
            //             }
            //         } else {
            //             break;
            //         }
            //     }
            //     has_leading_comments
            // }
            false
        }
    }

    fn is_complex_destructuring(&self) -> bool {
        match self {
            AssignmentLike::VariableDeclarator(variable_decorator) => {
                let BindingPatternKind::ObjectPattern(object) = &variable_decorator.id.kind else {
                    return false;
                };

                let properties = &object.properties;
                if properties.len() <= 2 {
                    return false;
                }

                properties.iter().any(|property| !property.value.kind.is_binding_identifier())
            }
            AssignmentLike::AssignmentExpression(assignment) => {
                let AssignmentTarget::ObjectAssignmentTarget(object) = &assignment.left else {
                    return false;
                };

                let properties = &object.properties;
                properties.iter().any(|property| match property {
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(
                        property_identifier,
                    ) => property_identifier.init.is_some(),
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(_) => true,
                })
            }
        }
    }
}

/// Checks if the function is entitled to be printed with layout [AssignmentLikeLayout::BreakAfterOperator]
fn should_break_after_operator(right: &Expression, f: &Formatter<'_, '_>) -> bool {
    if f.comments().has_leading_own_line_comments(right.span().start)
        && !matches!(right, Expression::JSXElement(_) | Expression::JSXFragment(_))
    {
        return true;
    }

    match right {
        // head is a long chain, meaning that right -> right are both assignment expressions
        Expression::AssignmentExpression(assignment) => {
            matches!(assignment.right, Expression::AssignmentExpression(_))
        }
        Expression::BinaryExpression(_) | Expression::SequenceExpression(_) => true,
        Expression::LogicalExpression(logical) => {
            !BinaryLikeExpression::can_inline_logical_expr(logical)
        }
        Expression::ConditionalExpression(conditional) => {
            !matches!(&conditional.test, Expression::LogicalExpression(logical) if BinaryLikeExpression::can_inline_logical_expr(logical))
        }
        Expression::ClassExpression(class) => !class.decorators.is_empty(),

        _ => {
            let argument = match right {
                Expression::AwaitExpression(expression) => Some(&expression.argument),
                Expression::YieldExpression(expression) => expression.argument.as_ref(),
                Expression::UnaryExpression(expression) => {
                    match get_last_non_unary_argument(expression) {
                        Expression::AwaitExpression(expression) => Some(&expression.argument),
                        Expression::YieldExpression(expression) => expression.argument.as_ref(),
                        argument => Some(argument),
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
    unary_expression: &'b UnaryExpression<'a>,
) -> &'b Expression<'a> {
    let mut argument = &unary_expression.argument;

    while let Expression::UnaryExpression(unary) = argument {
        argument = &unary.argument;
    }

    argument
}

impl<'a> Format<'a> for AssignmentLike<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
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
            let is_left_short = self.write_left(&mut Formatter::new(&mut buffer))?;
            let formatted_left = buffer.into_vec();
            let left_may_break = formatted_left.may_directly_break();

            let left = format_once(|f| f.write_elements(formatted_left));

            // Compare name only if we are in a position of computing it.
            // If not (for example, left is not an identifier), then let's fallback to false,
            // so we can continue the chain of checks
            let layout = self.layout(is_left_short, left_may_break, f);
            let right = format_with(|f| self.write_right(f, layout));

            let inner_content = format_with(|f| {
                if matches!(
                    &layout,
                    AssignmentLikeLayout::BreakLeftHandSide | AssignmentLikeLayout::OnlyLeft
                ) {
                    write!(f, [left])?;
                } else {
                    write!(f, [group(&left)])?;
                }

                if layout != AssignmentLikeLayout::SuppressedInitializer {
                    self.write_operator(f)?;
                }

                #[expect(clippy::match_same_arms)]
                match layout {
                    AssignmentLikeLayout::OnlyLeft => Ok(()),
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
                        )
                    }
                    AssignmentLikeLayout::BreakAfterOperator => {
                        write!(f, [group(&soft_line_indent_or_space(&right))])
                    }
                    AssignmentLikeLayout::NeverBreakAfterOperator => {
                        write!(f, [space(), right])
                    }
                    AssignmentLikeLayout::BreakLeftHandSide => {
                        write!(f, [space(), group(&right)])
                    }
                    AssignmentLikeLayout::Chain => {
                        write!(f, [soft_line_break_or_space(), right])
                    }
                    AssignmentLikeLayout::ChainTail => {
                        write!(f, [&indent(&format_args!(soft_line_break_or_space(), right))])
                    }
                    AssignmentLikeLayout::ChainTailArrowFunction => {
                        write!(f, [space(), right])
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
                | AssignmentLikeLayout::SuppressedInitializer
                | AssignmentLikeLayout::OnlyLeft => {
                    write!(f, [&inner_content])
                }
                _ => {
                    write!(f, [group(&inner_content)])
                }
            }
        });

        write!(f, [format_content])
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
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self.expression.as_ast_nodes() {
            AstNodes::ArrowFunctionExpression(arrow) => {
                FormatJsArrowFunctionExpression::new_with_options(
                    arrow,
                    FormatJsArrowFunctionExpressionOptions {
                        assignment_layout: self.layout,
                        ..FormatJsArrowFunctionExpressionOptions::default()
                    },
                )
                .fmt(f)
            }
            _ => self.expression.fmt(f),
        }
    }
}

/// A chain that has no calls at all or all of whose calls have no arguments
/// or have only one which [is_short_argument], except for member call chains
/// [Prettier applies]: https://github.com/prettier/prettier/blob/a043ac0d733c4d53f980aa73807a63fc914f23bd/src/language-js/print/assignment.js#L329
fn is_poorly_breakable_member_or_call_chain(
    expression: &Expression,
    f: &Formatter<'_, '_>,
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

    let mut expression = expression;

    loop {
        expression = match expression {
            Expression::TSNonNullExpression(assertion) => &assertion.expression,
            Expression::CallExpression(call_expression) => {
                is_chain = true;
                let callee = &call_expression.callee;
                call_expressions.push(call_expression);
                callee
            }
            Expression::StaticMemberExpression(node) => {
                is_chain = true;
                &node.object
            }
            Expression::ComputedMemberExpression(node) => {
                is_chain = true;
                &node.object
            }
            Expression::Identifier(_) | Expression::ThisExpression(_) => {
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

    for call_expression in call_expressions {
        // if is_member_call_chain(call_expression.clone(), f.comments(), f.options().tab_width())? {
        //     return Ok(false);
        // }
        // TODO: It looks like `is_member_call_chain` is used for checking comments,
        // but not sure if the following code is equivalent to the above check.
        if f.comments().has_comments_in_span(call_expression.span) {
            return false;
        }

        let args = &call_expression.arguments;

        let is_breakable_call = match args.len() {
            0 => false,
            1 => match args.iter().next() {
                Some(first_argument) => !is_short_argument(first_argument, threshold),
                None => false,
            },
            _ => true,
        };

        if is_breakable_call {
            return false;
        }

        let is_breakable_type_arguments = match &call_expression.type_arguments {
            Some(type_arguments) => false, // is_complex_type_arguments(type_arguments)?,
            None => false,
        };

        if is_breakable_type_arguments {
            return false;
        }
    }

    true
}

/// This function checks if `JsAnyCallArgument` is short
/// We need it to decide if `JsCallExpression` with the argument is breakable or not
/// If the argument is short the function call isn't breakable
/// [Prettier applies]: https://github.com/prettier/prettier/blob/a043ac0d733c4d53f980aa73807a63fc914f23bd/src/language-js/print/assignment.js#L374
fn is_short_argument(argument: &Argument, threshold: u16) -> bool {
    match argument {
        Argument::Identifier(identifier) => identifier.name.len() <= threshold as usize,
        Argument::UnaryExpression(unary_expression) => {
            unary_expression.operator.is_arithmetic()
                && matches!(unary_expression.argument, Expression::NumericLiteral(_))
        }
        Argument::RegExpLiteral(regex) => regex.regex.pattern.text.len() <= threshold as usize,
        Argument::StringLiteral(literal) => {
            // let formatter = FormatLiteralStringToken::new(
            //     &literal.value,
            //     literal.span,
            //     false,
            //     StringLiteralParentKind::Expression,
            // );

            // formatter.clean_text(f).width() <= threshold as usize
            literal.raw.is_some_and(|text| text.len() <= threshold as usize)
        }
        Argument::TemplateLiteral(literal) => {
            let elements = &literal.expressions;

            // Besides checking length exceed we also need to check that the template doesn't have any expressions.
            // It means that the elements of the template are empty or have only one `JsTemplateChunkElement` element
            // Prettier: https://github.com/prettier/prettier/blob/a043ac0d733c4d53f980aa73807a63fc914f23bd/src/language-js/print/assignment.js#L402-L405
            literal.quasis.len() == 1 && {
                let raw = literal.quasis[0].value.raw;
                raw.len() <= threshold as usize && !raw.contains('\n')
            }
        }
        Argument::ThisExpression(_)
        | Argument::NullLiteral(_)
        | Argument::BigIntLiteral(_)
        | Argument::BooleanLiteral(_)
        | Argument::NumericLiteral(_) => true,
        _ => false,
    }
}
