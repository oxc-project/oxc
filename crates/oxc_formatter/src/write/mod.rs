mod array_element_list;
mod array_expression;
mod array_pattern;
mod arrow_function_expression;
mod as_or_satisfies_expression;
mod assignment_pattern_property_list;
mod binary_like_expression;
mod binding_property_list;
mod block_statement;
mod call_arguments;
mod class;
mod decorators;
mod export_declarations;
mod function;
mod import_declaration;
mod import_expression;
mod intersection_type;
mod jsx;
mod mapped_type;
mod member_expression;
mod object_like;
mod object_pattern_like;
mod parameters;
mod program;
mod return_or_throw_statement;
mod semicolon;
mod sequence_expression;
mod switch_statement;
mod template;
mod try_statement;
mod tuple_type;
mod type_parameters;
mod union_type;
mod utils;
mod variable_declaration;

pub use arrow_function_expression::{
    ExpressionLeftSide, FormatJsArrowFunctionExpression, FormatJsArrowFunctionExpressionOptions,
};
pub use binary_like_expression::{BinaryLikeExpression, BinaryLikeOperator, should_flatten};
pub use function::FormatFunctionOptions;

use call_arguments::is_function_composition_args;
use cow_utils::CowUtils;

use oxc_allocator::{Address, Box, FromIn, StringBuilder, Vec};
use oxc_ast::{AstKind, ast::*};
use oxc_span::GetSpan;

use crate::{
    Expand, best_fitting, format_args,
    formatter::{
        Buffer, Format, FormatResult, Formatter,
        prelude::*,
        separated::FormatSeparatedIter,
        token::number::{NumberFormatOptions, format_number_token},
        trivia::{
            DanglingIndentMode, FormatDanglingComments, FormatLeadingComments,
            FormatTrailingComments,
        },
    },
    generated::ast_nodes::{AstNode, AstNodes},
    options::{FormatTrailingCommas, QuoteProperties, Semicolons, TrailingSeparator},
    parentheses::NeedsParentheses,
    utils::{
        assignment_like::AssignmentLike,
        call_expression::{contains_a_test_pattern, is_test_call_expression, is_test_each_pattern},
        conditional::ConditionalLike,
        format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
        member_chain::MemberChain,
        object::format_property_key,
        string_utils::{FormatLiteralStringToken, StringLiteralParentKind},
        suppressed::FormatSuppressedNode,
    },
    write,
    write::parameters::{can_avoid_parentheses, should_hug_function_parameters},
};

use self::{
    array_expression::FormatArrayExpression,
    class::format_grouped_parameters_with_return_type,
    function::should_group_function_parameters,
    object_like::ObjectLike,
    object_pattern_like::ObjectPatternLike,
    parameters::{ParameterLayout, ParameterList},
    return_or_throw_statement::FormatAdjacentArgument,
    semicolon::OptionalSemicolon,
    type_parameters::{FormatTSTypeParameters, FormatTSTypeParametersOptions},
    utils::{
        array::{TrailingSeparatorMode, write_array_node},
        statement_body::FormatStatementBody,
    },
};

pub trait FormatWrite<'ast, T = ()> {
    fn write(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()>;
    fn write_with_options(&self, options: T, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        unreachable!("Please implement it first.");
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, IdentifierName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name().as_str()))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, IdentifierReference<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name().as_str()))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, BindingIdentifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name().as_str()))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, LabelIdentifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name().as_str()))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ThisExpression> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "this")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ArrayExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        FormatArrayExpression::new(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Elision> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ObjectExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        ObjectLike::ObjectExpression(self).fmt(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, ObjectPropertyKind<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());
        let source_text = f.context().source_text();
        f.join_nodes_with_soft_line()
            .entries_with_trailing_separator(self.iter(), ",", trailing_separator)
            .finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ObjectProperty<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let is_accessor = match &self.kind() {
            PropertyKind::Init => false,
            PropertyKind::Get => {
                write!(f, ["get", space()])?;
                true
            }
            PropertyKind::Set => {
                write!(f, ["set", space()])?;
                true
            }
        };

        if self.method || is_accessor {
            let AstNodes::Function(func) = self.value().as_ast_nodes() else {
                unreachable!(
                    "The `value` always be a function node if `method` or `accessor` is true"
                )
            };

            if func.r#async() {
                write!(f, ["async", space()])?;
            }
            if func.generator() {
                write!(f, "*")?;
            }
            if self.computed {
                write!(f, ["[", self.key(), "]"])?;
            } else {
                format_property_key(self.key(), f)?;
            }

            if let Some(type_parameters) = &func.type_parameters() {
                write!(f, type_parameters)?;
            }
            write!(f, group(&func.params()))?;
            if let Some(return_type) = &func.return_type() {
                write!(f, return_type)?;
            }
            if let Some(body) = &func.body() {
                write!(f, [space(), body])?;
            }

            Ok(())
        } else {
            write!(f, AssignmentLike::ObjectProperty(self))
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, CallExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let callee = self.callee();
        let type_arguments = self.type_arguments();
        let arguments = self.arguments();
        let optional = self.optional();

        if callee.as_member_expression().is_some_and(|e| {
            matches!(
                e,
                MemberExpression::StaticMemberExpression(_)
                    | MemberExpression::ComputedMemberExpression(_)
            )
        }) && !callee.needs_parentheses(f)
        {
            MemberChain::from_call_expression(self, f).fmt(f)
        } else {
            let format_inner = format_with(|f| {
                write!(f, [callee, optional.then_some("?."), type_arguments, arguments])
            });
            if matches!(callee.as_ref(), Expression::CallExpression(_)) {
                write!(f, [group(&format_inner)])
            } else {
                write!(f, [format_inner])
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, NewExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["new", space(), self.callee(), self.type_arguments(), self.arguments()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, MetaProperty<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.meta(), ".", self.property()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, SpreadElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["...", self.argument()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, UpdateExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.prefix() {
            write!(f, self.operator().as_str())?;
        }
        write!(f, self.argument())?;
        if !self.prefix() {
            write!(f, self.operator().as_str())?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, UnaryExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.operator().as_str());
        if self.operator().is_keyword() {
            write!(f, space());
        }
        let Span { start, end, .. } = self.argument.span();
        if f.comments().has_comment_before(start)
            || f.comments().has_comment_in_range(end, self.span().end)
        {
            write!(
                f,
                [group(&format_args!(text("("), soft_block_indent(self.argument()), text(")")))]
            )
        } else {
            write!(f, self.argument())
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, BinaryExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        BinaryLikeExpression::BinaryExpression(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, PrivateInExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.left(), space(), "in", space(), self.right()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, LogicalExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        BinaryLikeExpression::LogicalExpression(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ConditionalExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        ConditionalLike::ConditionalExpression(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AssignmentExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        AssignmentLike::AssignmentExpression(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ArrayAssignmentTarget<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "[")?;

        if self.elements.is_empty() && self.rest.is_none() {
            write!(f, [format_dangling_comments(self.span()).with_block_indent()])?;
        } else {
            write!(
                f,
                group(&soft_block_indent(&format_once(|f| {
                    if !self.elements.is_empty() {
                        write_array_node(
                            self.elements.len() + usize::from(self.rest.is_some()),
                            self.elements().iter().map(AstNode::as_ref),
                            f,
                        )?;
                    }
                    if let Some(rest) = self.rest() {
                        write!(f, [soft_line_break_or_space(), rest]);
                    }
                    Ok(())
                })))
            )?;
        }

        write!(f, "]")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ObjectAssignmentTarget<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        ObjectPatternLike::ObjectAssignmentTarget(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AssignmentTargetRest<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["...", self.target()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AssignmentTargetWithDefault<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.binding(), space(), "=", space(), self.init()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AssignmentTargetPropertyIdentifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.binding())?;
        if let Some(expr) = &self.init() {
            write!(f, [space(), "=", space(), expr])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AssignmentTargetPropertyProperty<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.computed() {
            write!(f, "[")?;
        }
        write!(f, self.name())?;
        if self.computed() {
            write!(f, "]")?;
        }
        write!(f, [":", space(), self.binding()])?;
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Super> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "super")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AwaitExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let format_inner = format_with(|f| write!(f, ["await", space(), self.argument()]));

        let is_callee_or_object = match self.parent {
            AstNodes::CallExpression(_)
            | AstNodes::NewExpression(_)
            | AstNodes::StaticMemberExpression(_) => true,
            AstNodes::ComputedMemberExpression(member) => member.object.span() == self.span(),
            _ => false,
        };

        if is_callee_or_object {
            let mut parent = self.parent.parent();
            let mut ancestor_is_await = false;
            loop {
                match parent {
                    AstNodes::AwaitExpression(_)
                    | AstNodes::BlockStatement(_)
                    | AstNodes::FunctionBody(_)
                    | AstNodes::SwitchCase(_)
                    | AstNodes::Program(_)
                    | AstNodes::TSModuleBlock(_) => break,
                    _ => parent = parent.parent(),
                }
            }

            let indented = format_with(|f| write!(f, [soft_block_indent(&format_inner)]));

            return if let AstNodes::AwaitExpression(expr) = parent {
                if !expr.needs_parentheses(f) {
                    return write!(f, [group(&indented)]);
                }

                write!(f, [indented])
            } else {
                write!(f, [group(&indented)])
            };
        }

        write!(f, [format_inner])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ChainExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.expression().fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ParenthesizedExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        unreachable!("No `ParenthesizedExpression` as we disabled `preserve_parens` in the parser")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, EmptyStatement> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if matches!(
            self.parent,
            AstNodes::DoWhileStatement(_)
                | AstNodes::IfStatement(_)
                | AstNodes::WhileStatement(_)
                | AstNodes::ForStatement(_)
                | AstNodes::ForInStatement(_)
                | AstNodes::ForOfStatement(_)
                | AstNodes::WithStatement(_)
        ) {
            write!(f, ";")
        } else {
            Ok(())
        }
    }
}

/// Returns `true` if the expression needs a leading semicolon to prevent ASI issues
fn expression_statement_needs_semicolon<'a>(
    stmt: &AstNode<'a, ExpressionStatement<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> bool {
    if matches!(
        stmt.parent,
        // `if (true) (() => {})`
        AstNodes::IfStatement(_)
        // `do ({} => {}) while (true)`
        | AstNodes::DoWhileStatement(_)
        // `while (true) (() => {})`
        | AstNodes::WhileStatement(_)
        // `for (;;) (() => {})`
        | AstNodes::ForStatement(_)
        // `for (i in o) (() => {})`
        | AstNodes::ForInStatement(_)
        // `for (i of o) (() => {})`
        | AstNodes::ForOfStatement(_)
        // `with(true) (() => {})`
        | AstNodes::WithStatement(_)
        // `label: (() => {})`
        | AstNodes::LabeledStatement(_)
    ) {
        return false;
    }
    // Arrow functions need semicolon only if they will have parentheses
    // e.g., `(a) => {}` needs `;(a) => {}` but `a => {}` doesn't need semicolon
    if let Expression::ArrowFunctionExpression(arrow) = &stmt.expression {
        return !can_avoid_parentheses(arrow, f);
    }

    // First check if the expression itself needs protection
    let expr = stmt.expression();

    // Get the leftmost expression to check what the line starts with
    let mut current = ExpressionLeftSide::Expression(expr);
    loop {
        let needs_semi = match current {
            ExpressionLeftSide::Expression(expr) => {
                expr.needs_parentheses(f)
                    || match expr.as_ref() {
                        Expression::ArrayExpression(_)
                        | Expression::RegExpLiteral(_)
                        | Expression::TSTypeAssertion(_)
                        | Expression::ArrowFunctionExpression(_)
                        | Expression::JSXElement(_) => true,

                        Expression::TemplateLiteral(template) => true,
                        Expression::UnaryExpression(unary) => {
                            matches!(
                                unary.operator,
                                UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation
                            )
                        }
                        _ => false,
                    }
            }
            ExpressionLeftSide::AssignmentTarget(assignment) => {
                matches!(
                    assignment.as_ref(),
                    AssignmentTarget::ArrayAssignmentTarget(_)
                        | AssignmentTarget::TSTypeAssertion(_)
                        | AssignmentTarget::TSAsExpression(_)
                        | AssignmentTarget::TSSatisfiesExpression(_)
                        | AssignmentTarget::TSNonNullExpression(_)
                )
            }
            ExpressionLeftSide::SimpleAssignmentTarget(assignment) => {
                matches!(
                    assignment.as_ref(),
                    SimpleAssignmentTarget::TSTypeAssertion(_)
                        | SimpleAssignmentTarget::TSAsExpression(_)
                        | SimpleAssignmentTarget::TSNonNullExpression(_)
                )
            }
            _ => false,
        };

        if needs_semi {
            return true;
        }

        if let Some(next) = current.left_expression() { current = next } else { return false }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ExpressionStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // Check if we need a leading semicolon to prevent ASI issues
        if f.options().semicolons == Semicolons::AsNeeded
            && expression_statement_needs_semicolon(self, f)
        {
            write!(f, ";")?;
        }

        write!(f, [self.expression(), OptionalSemicolon])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, DoWhileStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let body = self.body();
        write!(f, group(&format_args!("do", FormatStatementBody::new(body))))?;
        if matches!(body.as_ref(), Statement::BlockStatement(_)) {
            write!(f, space())?;
        } else {
            write!(f, hard_line_break())?;
        }
        write!(
            f,
            [
                "while",
                space(),
                "(",
                group(&soft_block_indent(&self.test())),
                ")",
                OptionalSemicolon
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, WhileStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            group(&format_args!(
                "while",
                space(),
                "(",
                group(&soft_block_indent(self.test())),
                ")",
                FormatStatementBody::new(self.body())
            ))
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ForStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let init = self.init();
        let test = self.test();
        let update = self.update();
        let body = self.body();
        let format_body = FormatStatementBody::new(body);
        if init.is_none() && test.is_none() && update.is_none() {
            let comments = f.context().comments().comments_before(body.span().start);
            if !comments.is_empty() {
                write!(
                    f,
                    [
                        FormatDanglingComments::Comments {
                            comments,
                            indent: DanglingIndentMode::None
                        },
                        soft_line_break_or_space()
                    ]
                )?;
            }
            return write!(f, [group(&format_args!("for", space(), "(;;)", format_body))]);
        }

        let format_inner = format_with(|f| {
            write!(
                f,
                [
                    "for",
                    space(),
                    "(",
                    group(&soft_block_indent(&format_args!(
                        init,
                        ";",
                        soft_line_break_or_space(),
                        test,
                        ";",
                        soft_line_break_or_space(),
                        update
                    ))),
                    ")",
                    format_body
                ]
            )
        });
        write!(f, group(&format_inner))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ForInStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let comments = f.context().comments().own_line_comments_before(self.right.span().start);
        write!(
            f,
            [
                FormatLeadingComments::Comments(comments),
                group(&format_args!(
                    "for",
                    space(),
                    "(",
                    self.left(),
                    space(),
                    "in",
                    space(),
                    self.right(),
                    ")",
                    FormatStatementBody::new(self.body())
                ))
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ForOfStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let comments = f.context().comments().own_line_comments_before(self.right.span().start);

        let r#await = self.r#await();
        let left = self.left();
        let right = self.right();
        let body = self.body();
        let format_inner = format_with(|f| {
            write!(f, "for")?;
            if r#await {
                write!(f, [space(), "await"])?;
            }
            write!(
                f,
                [
                    space(),
                    "(",
                    left,
                    space(),
                    "of",
                    space(),
                    right,
                    ")",
                    FormatStatementBody::new(body)
                ]
            )
        });
        write!(f, [FormatLeadingComments::Comments(comments), group(&format_inner)])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, IfStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let test = self.test();
        let consequent = self.consequent();
        let alternate = self.alternate();
        write!(
            f,
            group(&format_args!(
                "if",
                space(),
                "(",
                group(&soft_block_indent(&test)),
                ")",
                FormatStatementBody::new(consequent),
            ))
        )?;
        if let Some(alternate) = alternate {
            let alternate_start = alternate.span().start;
            let comments = f.context().comments().comments_before(alternate_start);

            let has_line_comment = comments.iter().any(|comment| comment.kind == CommentKind::Line);
            let has_dangling_comments = has_line_comment
                || comments.last().is_some_and(|last_comment| {
                    // Ensure the comments are placed before the else keyword or on a new line
                    let gap_str =
                        f.source_text().slice_range(last_comment.span.end, alternate_start);
                    gap_str.contains("else")
                        || f.source_text()
                            .contains_newline_between(last_comment.span.end, alternate_start)
                });

            let else_on_same_line =
                matches!(consequent.as_ref(), Statement::BlockStatement(_)) && !has_line_comment;

            if else_on_same_line {
                write!(f, space())?;
            } else {
                write!(f, hard_line_break())?;
            }

            if has_dangling_comments {
                write!(
                    f,
                    FormatDanglingComments::Comments { comments, indent: DanglingIndentMode::None }
                )?;

                if has_line_comment {
                    write!(f, hard_line_break())?;
                } else {
                    write!(f, space())?;
                }
            }

            write!(
                f,
                [
                    "else",
                    group(&FormatStatementBody::new(alternate).with_forced_space(matches!(
                        alternate.as_ref(),
                        Statement::IfStatement(_)
                    )))
                ]
            )?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ContinueStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "continue")?;
        if let Some(label) = self.label() {
            write!(f, [space(), label])?;
        }
        write!(f, OptionalSemicolon)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, BreakStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "break")?;
        if let Some(label) = self.label() {
            write!(f, [space(), label])?;
        }
        write!(f, OptionalSemicolon)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, WithStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            group(&format_args!(
                "with",
                space(),
                "(",
                self.object(),
                ")",
                FormatStatementBody::new(self.body())
            ))
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, LabeledStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let comments = f.context().comments().comments_before(self.body.span().start);
        FormatLeadingComments::Comments(comments).fmt(f)?;

        let label = self.label();
        let body = self.body();
        write!(f, [label, ":"])?;
        if matches!(body.as_ref(), Statement::EmptyStatement(_)) {
            // If the body is an empty statement, force semicolon insertion
            write!(f, ";")
        } else {
            write!(f, [space(), body])
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, DebuggerStatement> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["debugger", OptionalSemicolon])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, BindingPattern<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.kind())?;
        if self.optional() {
            write!(f, "?")?;
        } else if let AstNodes::VariableDeclarator(declarator) = self.parent {
            write!(f, declarator.definite.then_some("!"))?;
        }
        if let Some(type_annotation) = &self.type_annotation() {
            write!(f, type_annotation)?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AssignmentPattern<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let mut left = self.left().memoized();
        // Format `left` early before writing leading comments, so that comments
        // inside `left` are not treated as leading comments of `= right`
        left.inspect(f)?;
        let comments = f.context().comments().own_line_comments_before(self.right.span().start);
        write!(
            f,
            [FormatLeadingComments::Comments(comments), left, space(), "=", space(), self.right()]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ObjectPattern<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if matches!(self.parent, AstNodes::FormalParameter(param) if param.pattern.type_annotation.is_some())
        {
            FormatNodeWithoutTrailingComments(&ObjectPatternLike::ObjectPattern(self)).fmt(f)
        } else {
            ObjectPatternLike::ObjectPattern(self).fmt(f)
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, BindingProperty<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let group_id = f.group_id("assignment");
        let format_inner = format_with(|f| {
            if self.computed() {
                write!(f, "[")?;
            }
            if !self.shorthand() {
                write!(f, self.key())?;
            }
            if self.computed() {
                write!(f, "]")?;
            }
            if self.shorthand() {
                write!(f, self.value())
            } else {
                write!(
                    f,
                    [
                        ":",
                        group(&indent(&soft_line_break_or_space())).with_group_id(Some(group_id)),
                        line_suffix_boundary(),
                        indent_if_group_breaks(&self.value(), group_id)
                    ]
                )
            }
        });
        write!(f, group(&format_inner))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, BindingRestElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["...", self.argument()])
    }
}

impl<'a> FormatWrite<'a, FormatJsArrowFunctionExpressionOptions>
    for AstNode<'a, ArrowFunctionExpression<'a>>
{
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        FormatJsArrowFunctionExpression::new(self).fmt(f)
    }

    fn write_with_options(
        &self,
        options: FormatJsArrowFunctionExpressionOptions,
        f: &mut Formatter<'_, 'a>,
    ) -> FormatResult<()> {
        FormatJsArrowFunctionExpression::new_with_options(self, options).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, YieldExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["yield", self.delegate().then_some("*")])?;
        if let Some(argument) = &self.argument() {
            write!(f, [space(), FormatAdjacentArgument(argument)])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, V8IntrinsicExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["%", self.name(), self.arguments()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, BooleanLiteral> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, if self.value() { "true" } else { "false" })
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, NullLiteral> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "null")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, NumericLiteral<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_number_token(
            f.source_text().text_for(self),
            self.span(),
            NumberFormatOptions::default().keep_one_trailing_decimal_zero(),
        )
        .fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, StringLiteral<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let is_jsx = matches!(self.parent, AstNodes::JSXAttribute(_));
        FormatLiteralStringToken::new(
            f.source_text().text_for(self),
            self.span(),
            /* jsx */
            is_jsx,
            StringLiteralParentKind::Expression,
        )
        .fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, BigIntLiteral<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            dynamic_text(
                f.context().allocator().alloc_str(&self.raw().unwrap().cow_to_ascii_lowercase())
            )
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, RegExpLiteral<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let raw = self.raw().unwrap().as_str();
        let (pattern, flags) = raw.rsplit_once('/').unwrap();
        // TODO: print the flags without allocation.
        let mut flags = flags.chars().collect::<std::vec::Vec<_>>();
        flags.sort_unstable();
        let flags = flags.iter().collect::<String>();
        let s = StringBuilder::from_strs_array_in([pattern, "/", &flags], f.context().allocator());
        write!(f, dynamic_text(s.into_str(),))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSEnumDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.declare() {
            write!(f, ["declare", space()])?;
        }
        if self.r#const() {
            write!(f, ["const", space()])?;
        }
        write!(f, ["enum", space(), self.id(), space(), "{", self.body(), "}"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSEnumBody<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.members().is_empty() {
            write!(
                f,
                group(&format_args!(format_dangling_comments(self.span()), soft_line_break()))
            )
        } else {
            write!(f, block_indent(self.members()))
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, TSEnumMember<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());
        f.join_nodes_with_soft_line()
            .entries_with_trailing_separator(self.iter(), ",", trailing_separator)
            .finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSEnumMember<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let id = self.id();
        let is_computed = matches!(id.as_ref(), TSEnumMemberName::ComputedTemplateString(_));

        if is_computed {
            write!(f, "[")?;
        }

        write!(f, [id])?;

        if is_computed {
            write!(f, "]")?;
        }

        if let Some(init) = self.initializer() {
            write!(f, [space(), "=", space(), init])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeAnnotation<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self.parent {
            AstNodes::TSFunctionType(_) | AstNodes::TSConstructorType(_) => {
                write!(f, ["=>", space(), self.type_annotation()])
            }
            AstNodes::TSTypePredicate(_) => {
                write!(f, [self.type_annotation()])
            }
            _ => {
                write!(f, [":", space(), self.type_annotation()])
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSLiteralType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.literal().fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSConditionalType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        ConditionalLike::TSConditionalType(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSParenthesizedType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["(", self.type_annotation(), ")"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeOperator<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            write!(f, "(")?;
        }

        write!(f, [self.operator().to_str(), hard_space(), self.type_annotation()])?;

        if needs_parentheses {
            write!(f, ")")?;
        }

        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSArrayType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if let AstNodes::TSUnionType(union) = self.element_type().as_ast_nodes() {
            // `TSUnionType` has special logic for comments, so we need to delegate to it.
            union.fmt(f)?;
        } else {
            FormatNodeWithoutTrailingComments(self.element_type()).fmt(f)?;
        }
        let comments =
            f.context().comments().comments_before_character(self.element_type.span().end, b'[');
        FormatTrailingComments::Comments(comments).fmt(f)?;
        write!(f, ["[]"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSIndexedAccessType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object_type(), "[", self.index_type(), "]"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSNamedTupleMember<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.label())?;
        if self.optional() {
            write!(f, "?")?;
        }
        write!(f, [":", space(), self.element_type()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSOptionalType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.type_annotation(), "?"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSRestType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["...", self.type_annotation()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSAnyKeyword> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "any")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSStringKeyword> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "string")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSBooleanKeyword> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "boolean")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSNumberKeyword> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "number")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSNeverKeyword> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "never")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSIntrinsicKeyword> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "intrinsic")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSUnknownKeyword> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "unknown")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSNullKeyword> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "null")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSUndefinedKeyword> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "undefined")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSVoidKeyword> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "void")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSSymbolKeyword> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "symbol")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSThisType> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "this")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSObjectKeyword> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "object")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSBigIntKeyword> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "bigint")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeReference<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.type_name(), self.type_arguments()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSQualifiedName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.left(), ".", self.right()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeParameterDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        FormatTSTypeParameters::new(self, FormatTSTypeParametersOptions::default()).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeAliasDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [AssignmentLike::TSTypeAliasDeclaration(self), OptionalSemicolon])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSInterfaceDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let id = self.id();
        let type_parameters = self.type_parameters();
        let extends = self.extends();
        let body = self.body();

        let should_indent_extends_only = type_parameters.as_ref().is_some_and(|params| {
            !extends.as_ref().first().is_some_and(|first| {
                f.comments()
                    .comments_in_range(params.span().end, first.span().start)
                    .iter()
                    .any(|c| c.is_line())
            })
        });

        let type_parameter_group = if should_indent_extends_only && !extends.is_empty() {
            Some(f.group_id("type_parameters"))
        } else {
            None
        };

        let format_id = format_with(|f| {
            if type_parameters.is_none() && extends.is_empty() {
                FormatNodeWithoutTrailingComments(id).fmt(f)?;
            } else {
                write!(f, [id])?;
            }

            if let Some(type_parameters) = type_parameters {
                write!(
                    f,
                    FormatTSTypeParameters::new(
                        type_parameters,
                        FormatTSTypeParametersOptions {
                            group_id: type_parameter_group,
                            is_type_or_interface_decl: true
                        }
                    )
                )?;
            }

            Ok(())
        });

        let format_extends = format_with(|f| {
            let Some(first_extend) = extends.as_ref().first() else {
                return Ok(());
            };

            let has_leading_own_line_comment =
                f.context().comments().has_leading_own_line_comment(first_extend.span().start);
            if has_leading_own_line_comment {
                write!(
                    f,
                    FormatTrailingComments::Comments(
                        f.context().comments().comments_before(first_extend.span().start)
                    )
                )?;
            }

            if !extends.is_empty() {
                if should_indent_extends_only {
                    write!(
                        f,
                        [
                            if_group_breaks(&space()).with_group_id(type_parameter_group),
                            if_group_fits_on_line(&soft_line_break_or_space())
                                .with_group_id(type_parameter_group),
                        ]
                    )?;
                } else {
                    write!(f, soft_line_break_or_space())?;
                }

                write!(f, [line_suffix_boundary(), "extends", space()])?;

                if extends.len() == 1 {
                    write!(f, extends)?;
                } else {
                    write!(f, indent(&extends))?;
                }

                let has_leading_own_line_comment =
                    f.context().comments().has_leading_own_line_comment(self.body.span().start);

                if !has_leading_own_line_comment {
                    write!(f, [space()])?;
                    body.format_leading_comments(f)?;
                }
            }

            Ok(())
        });

        let content = format_with(|f| {
            if self.declare() {
                write!(f, ["declare", space()])?;
            }

            write!(f, ["interface", space()])?;

            if extends.is_empty() {
                write!(f, [format_id, format_extends])?;
            } else if should_indent_extends_only {
                write!(f, [group(&format_args!(format_id, indent(&format_extends)))])?;
            } else {
                write!(f, [group(&indent(&format_args!(format_id, format_extends)))])?;
            }

            write!(f, [space(), "{"])?;

            if body.body().is_empty() {
                write!(f, format_dangling_comments(body.span()).with_block_indent())?;
            } else {
                write!(f, block_indent(&body))?;
            }

            write!(f, "}")
        });

        write!(f, group(&content))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSInterfaceBody<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.body().fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSPropertySignature<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.readonly() {
            write!(f, ["readonly", space()])?;
        }
        if self.computed() {
            write!(f, ["[", self.key(), "]"])?;
        } else {
            format_property_key(self.key(), f)?;
        }
        if self.optional() {
            write!(f, "?")?;
        }
        if let Some(type_annotation) = &self.type_annotation() {
            write!(f, type_annotation)?;
        }
        Ok(())
    }
}

struct FormatTSSignature<'a, 'b> {
    last: bool,
    signature: &'b AstNode<'a, TSSignature<'a>>,
}

impl GetSpan for FormatTSSignature<'_, '_> {
    fn span(&self) -> Span {
        self.signature.span()
    }
}

impl<'a> Format<'a> for FormatTSSignature<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [group(&self.signature)])?;

        match f.options().semicolons {
            Semicolons::Always => {
                if self.last {
                    write!(f, [if_group_breaks(&text(";"))])?;
                } else {
                    text(";").fmt(f)?;
                }
            }
            Semicolons::AsNeeded => {
                if !self.last {
                    write!(f, [if_group_fits_on_line(&text(";"))])?;
                }
            }
        }

        Ok(())
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, TSSignature<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let last_index = self.len().saturating_sub(1);
        f.join_nodes_with_soft_line()
            .entries(
                self.iter()
                    .enumerate()
                    .map(|(i, signature)| FormatTSSignature { last: i == last_index, signature }),
            )
            .finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSCallSignatureDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, group(&format_args!(self.type_parameters(), self.params(), self.return_type())))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSMethodSignature<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self.kind() {
            TSMethodSignatureKind::Method => {}
            TSMethodSignatureKind::Get => {
                write!(f, ["get", space()])?;
            }
            TSMethodSignatureKind::Set => {
                write!(f, ["set", space()])?;
            }
        }
        if self.computed() {
            write!(f, "[")?;
        }
        write!(f, self.key())?;
        if self.computed() {
            write!(f, "]")?;
        }
        if self.optional() {
            write!(f, "?")?;
        }

        format_grouped_parameters_with_return_type(
            self.type_parameters(),
            self.this_param.as_deref(),
            self.params(),
            self.return_type(),
            f,
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSConstructSignatureDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["new", space()])?;
        write!(f, group(&format_args!(self.type_parameters(), self.params(), self.return_type())))
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, TSInterfaceHeritage<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let last_index = self.len().saturating_sub(1);
        let mut joiner = f.join_with(soft_line_break_or_space());

        for (i, heritage) in FormatSeparatedIter::new(self.into_iter(), ",")
            .with_trailing_separator(TrailingSeparator::Disallowed)
            .enumerate()
        {
            if i == last_index {
                joiner.entry(&FormatNodeWithoutTrailingComments(&heritage));
            } else {
                joiner.entry(&heritage);
            }
        }

        joiner.finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSInterfaceHeritage<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression(), self.type_arguments()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypePredicate<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.asserts() {
            write!(f, ["asserts", space()])?;
        }
        write!(f, [self.parameter_name()])?;
        if let Some(type_annotation) = self.type_annotation() {
            write!(f, [space(), "is", space(), type_annotation])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSModuleDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.declare() {
            write!(f, ["declare", space()])?;
        }

        if !self.kind.is_global() {
            write!(f, self.kind().as_str())?;
        }

        write!(f, [space(), self.id()])?;

        if let Some(body) = self.body() {
            let mut body = body;
            loop {
                match body.as_ast_nodes() {
                    AstNodes::TSModuleDeclaration(b) => {
                        write!(f, [".", b.id()])?;
                        if let Some(b) = &b.body() {
                            body = b;
                        } else {
                            break;
                        }
                    }
                    AstNodes::TSModuleBlock(body) => {
                        write!(f, [space(), body]);
                        break;
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
        } else {
            write!(f, OptionalSemicolon)?;
        }

        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSModuleBlock<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let directives = self.directives();
        let body = self.body();
        let span = self.span();

        write!(f, "{")?;
        if body.is_empty() && directives.is_empty() {
            write!(f, [format_dangling_comments(span).with_block_indent()])?;
        } else {
            write!(f, [block_indent(&format_args!(directives, body))])?;
        }
        write!(f, "}")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeLiteral<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        ObjectLike::TSTypeLiteral(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSInferType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["infer ", self.type_parameter()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeQuery<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["typeof ", self.expr_name(), self.type_arguments()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSImportType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["import(", self.argument()])?;
        if let Some(options) = &self.options() {
            write!(f, [",", space(), options])?;
        }
        write!(f, ")")?;
        if let Some(qualified_name) = &self.qualifier() {
            write!(f, [".", qualified_name])?;
        }
        write!(f, self.type_arguments())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSImportTypeQualifiedName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.left(), ".", self.right()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSFunctionType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let format_inner = format_with(|f| {
            let type_parameters = self.type_parameters();
            write!(f, type_parameters)?;

            let params = self.params();
            let return_type = self.return_type();
            let mut format_parameters = params.memoized();
            let mut format_return_type = return_type.memoized();

            // Inspect early, in case the `return_type` is formatted before `parameters`
            // in `should_group_function_parameters`.
            format_parameters.inspect(f)?;

            let group_parameters = should_group_function_parameters(
                type_parameters.map(AsRef::as_ref),
                params.items.len()
                    + usize::from(params.rest.is_some())
                    + usize::from(self.this_param.is_some()),
                Some(&self.return_type),
                &mut format_return_type,
                f,
            )?;

            if group_parameters {
                write!(f, [group(&format_parameters)])
            } else {
                write!(f, [format_parameters])
            }?;

            write!(f, [space(), format_return_type])
        });

        write!(f, group(&format_inner))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSConstructorType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let r#abstract = self.r#abstract();
        let type_parameters = self.type_parameters();
        let params = self.params();
        let return_type = self.return_type();

        if r#abstract {
            write!(f, ["abstract", space()])?;
        }
        write!(
            f,
            [group(&format_args!("new", space(), type_parameters, params, space(), return_type))]
        );
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeAssertion<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let break_after_cast = !matches!(
            self.expression,
            Expression::ArrayExpression(_) | Expression::ObjectExpression(_)
        );

        let format_cast = format_with(|f| {
            write!(f, ["<", group(&soft_block_indent(&self.type_annotation())), ">",])
        });

        if break_after_cast {
            let format_cast = format_cast.memoized();
            let format_expression = self.expression().memoized();

            write!(
                f,
                [best_fitting![
                    format_args!(format_cast, format_expression),
                    format_args!(
                        format_cast,
                        group(&format_args!(
                            text("("),
                            block_indent(&format_expression),
                            text(")")
                        ))
                    ),
                    format_args!(format_cast, format_expression)
                ]]
            )
        } else {
            write!(f, [format_cast, self.expression()])
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSNonNullExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression(), "!"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSInstantiationExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression(), self.type_arguments()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSDocNullableType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.postfix() {
            write!(f, [self.type_annotation(), "?"])
        } else {
            write!(f, ["?", self.type_annotation()])
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSDocNonNullableType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.postfix() {
            write!(f, [self.type_annotation(), "!"])
        } else {
            write!(f, ["!", self.type_annotation()])
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSDocUnknownType> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}
