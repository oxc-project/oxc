mod array_element_list;
mod array_expression;
mod arrow_function_expression;
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
mod jsx;
mod member_expression;
mod object_like;
mod object_pattern_like;
mod parameter_list;
mod program;
mod return_or_throw_statement;
mod semicolon;
mod sequence_expression;
mod switch_statement;
mod template;
mod try_statement;
mod type_parameters;
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
    format_args,
    formatter::{
        Buffer, Format, FormatResult, Formatter,
        comments::is_own_line_comment,
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
        member_chain::MemberChain,
        object::format_property_key,
        string_utils::{FormatLiteralStringToken, StringLiteralParentKind},
    },
    write,
    write::parameter_list::{can_avoid_parentheses, should_hug_function_parameters},
};

use self::{
    array_expression::FormatArrayExpression,
    object_like::ObjectLike,
    object_pattern_like::ObjectPatternLike,
    parameter_list::{ParameterLayout, ParameterList},
    semicolon::{ClassPropertySemicolon, OptionalSemicolon},
    type_parameters::{FormatTsTypeParameters, FormatTsTypeParametersOptions},
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
        if f.comments().has_comments_before(start)
            || f.comments().has_comments_between(end, self.span().end)
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
                )
            }
            ExpressionLeftSide::SimpleAssignmentTarget(assignment) => {
                matches!(
                    assignment.as_ref(),
                        | SimpleAssignmentTarget::TSTypeAssertion(_)
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
        let comments = f.context().comments().own_line_comments_before(self.body.span().start);
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
        let comments = f.context().comments().own_line_comments_before(self.body.span().start);

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
                        &f.source_text()[last_comment.span.end as usize..alternate_start as usize];
                    gap_str.contains("else") || gap_str.contains('\n')
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
        }
        if let Some(type_annotation) = &self.type_annotation() {
            write!(f, type_annotation)?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AssignmentPattern<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let comments = f.context().comments().own_line_comments_before(self.right.span().start);
        write!(
            f,
            [
                FormatLeadingComments::Comments(comments),
                self.left(),
                space(),
                "=",
                space(),
                self.right(),
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ObjectPattern<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        ObjectPatternLike::ObjectPattern(self).fmt(f)
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

impl<'a> FormatWrite<'a> for AstNode<'a, ArrayPattern<'a>> {
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

impl<'a> FormatWrite<'a> for AstNode<'a, BindingRestElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["...", self.argument()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, FormalParameters<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let comments = f.context().comments().comments_before(self.span.start);
        if !comments.is_empty() {
            write!(f, [space(), FormatTrailingComments::Comments(comments)])?;
        }

        let parentheses_not_needed = if let AstNodes::ArrowFunctionExpression(arrow) = self.parent {
            can_avoid_parentheses(arrow, f)
        } else {
            false
        };

        let has_any_decorated_parameter =
            self.items.iter().any(|param| !param.decorators.is_empty());

        let can_hug = should_hug_function_parameters(self, parentheses_not_needed, f)
            && !has_any_decorated_parameter;

        // Check if parameters are in a test call
        // After AstKind::Argument removal, parent chain is: FormalParameters -> Function -> CallExpression
        let is_test_call = if let AstNodes::Function(func) = &self.parent {
            if let AstNodes::CallExpression(call) = &func.parent {
                is_test_call_expression(call)
            } else {
                false
            }
        } else {
            false
        };

        let layout = if !self.has_parameter() {
            ParameterLayout::NoParameters
        } else if can_hug || is_test_call {
            ParameterLayout::Hug
        } else {
            ParameterLayout::Default
        };

        if !parentheses_not_needed {
            write!(f, "(")?;
        }

        match layout {
            ParameterLayout::NoParameters => {
                write!(f, format_dangling_comments(self.span()).with_soft_block_indent())?;
            }
            ParameterLayout::Hug => {
                write!(f, ParameterList::with_layout(self, layout))?;
            }
            ParameterLayout::Default => {
                write!(f, soft_block_indent(&ParameterList::with_layout(self, layout)))?;
            }
        }

        if !parentheses_not_needed {
            write!(f, ")")?;
        }

        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, FormalParameter<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let content = format_with(|f| {
            if let Some(accessibility) = self.accessibility() {
                write!(f, [accessibility.as_str(), space()])?;
            }
            if self.r#override() {
                write!(f, ["override", space()])?;
            }
            if self.readonly() {
                write!(f, ["readonly", space()])?;
            }
            write!(f, self.pattern())
        });

        // TODO
        let is_hug_parameter = false;
        // let is_hug_parameter = node
        // .syntax()
        // .grand_parent()
        // .and_then(FormatAnyJsParameters::cast)
        // .is_some_and(|parameters| {
        // should_hug_function_parameters(&parameters, f.comments(), false).unwrap_or(false)
        // });

        let decorators = self.decorators();
        if is_hug_parameter && decorators.is_empty() {
            write!(f, [decorators, content])
        } else if decorators.is_empty() {
            write!(f, [decorators, group(&content)])
        } else {
            write!(f, [group(&decorators), group(&content)])
        }
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
        write!(f, "yield")?;
        if self.delegate() {
            write!(f, "*")?;
        }
        if let Some(argument) = &self.argument() {
            write!(f, [space(), argument])?;
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
            self.span().source_text(f.source_text()),
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
            self.span().source_text(f.source_text()),
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

impl<'a> FormatWrite<'a> for AstNode<'a, TSThisParameter<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "this")?;
        if let Some(type_annotation) = self.type_annotation() {
            type_annotation.fmt(f);
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSEnumDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
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
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeAnnotation<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [":", space(), self.type_annotation()])
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

impl<'a> FormatWrite<'a> for AstNode<'a, TSUnionType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let mut types = self.types().iter();
        if let Some(item) = types.next() {
            write!(f, item)?;

            for item in types {
                write!(f, [" | ", item])?;
            }
            return Ok(());
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSIntersectionType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let mut types = self.types().iter();
        if let Some(item) = types.next() {
            write!(f, item)?;

            for item in types {
                write!(f, [" & ", item])?;
            }
            return Ok(());
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSParenthesizedType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["(", self.type_annotation(), ")"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeOperator<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.operator().to_str(), hard_space(), self.type_annotation()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSArrayType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.element_type(), "[]"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSIndexedAccessType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object_type(), "[", self.index_type(), "]"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTupleType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "[")?;
        for (i, ty) in self.element_types().iter().enumerate() {
            if i != 0 {
                write!(f, [",", space()])?;
            }
            write!(f, ty)?;
        }
        write!(f, "]")
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

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeParameterInstantiation<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "<")?;
        for (i, param) in self.params().iter().enumerate() {
            if i != 0 {
                write!(f, [",", space()])?;
            }
            write!(f, param)?;
        }
        write!(f, ">")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeParameter<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.r#const() {
            write!(f, ["const", space()])?;
        }
        if self.r#in() {
            write!(f, ["in", space()])?;
        }
        if self.out() {
            write!(f, ["out", space()])?;
        }
        write!(f, self.name())?;
        if let Some(constraint) = &self.constraint() {
            write!(f, [space(), "extends", space(), constraint])?;
        }
        if let Some(default) = &self.default() {
            write!(f, [space(), "=", space(), default])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeParameterDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["<", self.params(), ">"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeAliasDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let assignment_like = format_with(|f| {
            write!(
                f,
                [self.id(), self.type_parameters(), space(), "=", space(), self.type_annotation()]
            )
        });
        write!(
            f,
            [
                self.declare().then_some("declare "),
                "type",
                space(),
                group(&assignment_like),
                OptionalSemicolon
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSInterfaceDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let id = self.id();
        let type_parameters = self.type_parameters();
        let extends = self.extends();
        let body = self.body();

        let should_indent_extends_only = type_parameters.as_ref().is_some_and(|params|
                // TODO:
                // !f.comments().has_trailing_line_comment(params.span().end)
                true);

        let type_parameter_group = if should_indent_extends_only && !extends.is_empty() {
            Some(f.group_id("type_parameters"))
        } else {
            None
        };

        let format_id = format_with(|f| {
            write!(f, id)?;

            if let Some(type_parameters) = type_parameters {
                write!(
                    f,
                    FormatTsTypeParameters::new(
                        type_parameters,
                        FormatTsTypeParametersOptions {
                            group_id: type_parameter_group,
                            is_type_or_interface_decl: true
                        }
                    )
                )?;
            }

            Ok(())
        });

        let format_extends = format_with(|f| {
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
                write!(f, "extends")?;
                if extends.len() == 1 {
                    write!(f, extends)?;
                } else {
                    write!(f, indent(&extends))?;
                }
            }

            Ok(())
        });

        let content = format_with(|f| {
            if self.declare() {
                write!(f, ["declare", space()])?;
            }

            write!(f, ["interface", space()])?;

            // TODO:
            // let id_has_trailing_comments = f.comments().has_trailing_comments(id.span().end);
            let id_has_trailing_comments = false;
            if id_has_trailing_comments || !extends.is_empty() {
                if should_indent_extends_only {
                    write!(f, [group(&format_args!(format_id, indent(&format_extends)))])?;
                } else {
                    write!(f, [group(&indent(&format_args!(format_id, format_extends)))])?;
                }
            } else {
                write!(f, [format_id, format_extends])?;
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
        let source_text = f.context().source_text();
        f.join_nodes_with_soft_line().entries(self.body()).finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSPropertySignature<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.readonly() {
            write!(f, "readonly")?;
        }
        if self.computed() {
            write!(f, [space(), "[", self.key(), "]"])?;
        } else {
            write!(f, self.key())?;
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

impl<'a> Format<'a> for AstNode<'a, Vec<'a, TSSignature<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.join_nodes_with_soft_line().entries(self).finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSIndexSignature<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.readonly() {
            write!(f, ["readonly", space()])?;
        }
        // TODO: parameters only have one element for now.
        write!(
            f,
            [
                "[",
                self.parameters().first().unwrap(),
                "]",
                self.type_annotation(),
                OptionalSemicolon
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSCallSignatureDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if let Some(type_parameters) = &self.type_parameters() {
            write!(f, group(type_parameters))?;
        }

        if let Some(this_param) = &self.this_param() {
            write!(f, [this_param, ",", soft_line_break_or_space()])?;
        }
        write!(f, group(&self.params()))?;
        if let Some(return_type) = &self.return_type() {
            write!(f, return_type)?;
        }
        write!(f, OptionalSemicolon)
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
        // TODO:
        // if should_group_function_parameters(
        //         type_parameters.as_ref(),
        //         parameters.len(),
        //         return_type_annotation
        //             .as_ref()
        //             .map(|annotation| annotation.ty()),
        //         &mut format_return_type_annotation,
        //         f,
        //     )? {
        //         write!(f, [group(&parameters)])?;
        //     } else {
        //         write!(f, [parameters])?;
        //     }
        if let Some(type_parameters) = &self.type_parameters() {
            write!(f, [group(&type_parameters)])?;
        }
        write!(f, group(&self.params()))?;
        if let Some(return_type) = &self.return_type() {
            write!(f, return_type)?;
        }
        write!(f, OptionalSemicolon)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSConstructSignatureDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["new", space()])?;
        if let Some(type_parameters) = &self.type_parameters() {
            write!(f, group(&type_parameters))?;
        }
        write!(f, group(&self.params()))?;
        if let Some(return_type) = self.return_type() {
            write!(f, return_type)?;
        }
        write!(f, OptionalSemicolon)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSIndexSignatureName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [dynamic_text(self.name().as_str()), self.type_annotation()])
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, TSInterfaceHeritage<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.join_with(soft_line_break_or_space())
            .entries_with_trailing_separator(self.iter(), ",", TrailingSeparator::Disallowed)
            .finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSInterfaceHeritage<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression(), self.type_arguments()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypePredicate<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSModuleDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.declare() {
            write!(f, ["declare", space()])?;
        }
        write!(f, self.kind().as_str())?;

        if !self.kind().is_global() {
            write!(f, [space(), self.id()])?;
        }

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
        let type_parameters = self.type_parameters();
        let params = self.params();
        let return_type = self.return_type();

        let format_inner = format_with(|f| {
            write!(f, type_parameters)?;

            // TODO
            // let mut format_return_type = return_type.format().memoized();
            let should_group_parameters = false;
            // TODO
            //should_group_function_parameters(
            // type_parameters.as_ref(),
            // parameters.as_ref()?.items().len(),
            // Some(return_type.clone()),
            // &mut format_return_type,
            // f,
            // )?;

            if should_group_parameters {
                write!(f, group(&params))?;
            } else {
                write!(f, params)?;
            }

            write!(f, [space(), "=>", space(), return_type.type_annotation()])
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
            [
                "new",
                space(),
                type_parameters,
                params,
                space(),
                "=>",
                space(),
                return_type.type_annotation()
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSMappedType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let type_parameter = self.type_parameter();
        let name_type = self.name_type();

        let should_expand = false; // TODO has_line_break_before_property_name(node)?;

        let type_annotation_has_leading_comment = false;
        //TODO
        //
        // mapped_type
        // .as_ref()
        // .is_some_and(|annotation| comments.has_leading_comments(annotation.syntax()));

        let format_inner = format_with(|f| {
            // TODO:
            // write!(f, FormatLeadingComments::Comments(comments.dangling_comments(self.span())))?;

            match self.readonly() {
                Some(TSMappedTypeModifierOperator::True) => write!(f, ["readonly", space()])?,
                Some(TSMappedTypeModifierOperator::Plus) => write!(f, ["+readonly", space()])?,
                Some(TSMappedTypeModifierOperator::Minus) => write!(f, ["-readonly", space()])?,
                None => {}
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
                match self.optional() {
                    Some(TSMappedTypeModifierOperator::True) => write!(f, "?"),
                    Some(TSMappedTypeModifierOperator::Plus) => write!(f, "+?"),
                    Some(TSMappedTypeModifierOperator::Minus) => write!(f, "-?"),
                    None => Ok(()),
                }
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

impl<'a> FormatWrite<'a> for AstNode<'a, TSAsExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression(), " as ", self.type_annotation()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSSatisfiesExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression(), " satisfies ", self.type_annotation()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeAssertion<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "<")?;
        // var r = < <T>(x: T) => T > ((x) => { return null; });
        //          ^ make sure space is printed here.
        if matches!(**self.type_annotation(), TSType::TSFunctionType(_)) {
            write!(f, space())?;
        }
        write!(f, [self.type_annotation(), ">", self.expression()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSImportEqualsDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                "import",
                space(),
                self.import_kind(),
                self.id(),
                space(),
                "=",
                space(),
                self.module_reference(),
                OptionalSemicolon
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSExternalModuleReference<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["require(", self.expression(), ")"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSNonNullExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression(), "!"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSExportAssignment<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["export = ", self.expression()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSNamespaceExportDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["export as namespace ", self.id()])
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
