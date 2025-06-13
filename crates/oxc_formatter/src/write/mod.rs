mod array_element_list;
mod array_expression;
mod arrow_function_expression;
mod assignment_pattern_property_list;
mod binary_like_expression;
mod binding_property_list;
mod block_statement;
mod call_arguments;
mod class;
mod function;
mod object_like;
mod object_pattern_like;
mod parameter_list;
mod semicolon;
mod type_parameters;
mod utils;
mod variable_declaration;
pub use binary_like_expression::{BinaryLikeExpression, BinaryLikeOperator, should_flatten};

use call_arguments::{FormatAllArgsBrokenOut, FormatCallArgument, is_function_composition_args};
use cow_utils::CowUtils;

use oxc_allocator::{Address, Box, Vec};
use oxc_ast::{AstKind, ast::*};
use oxc_span::{GetSpan, SPAN};
use oxc_syntax::identifier::{ZWNBSP, is_identifier_name, is_line_terminator};

use crate::{
    format_args,
    formatter::{
        Buffer, Format, FormatResult, Formatter,
        prelude::*,
        separated::FormatSeparatedIter,
        token::number::{NumberFormatOptions, format_number_token},
        trivia::FormatLeadingComments,
    },
    generated::ast_nodes::{AstNode, AstNodes},
    options::{FormatTrailingCommas, QuoteProperties, TrailingSeparator},
    parentheses::NeedsParentheses,
    utils::write_arguments_multi_line,
    write,
};

use self::{
    array_expression::FormatArrayExpression,
    arrow_function_expression::FormatJsArrowFunctionExpression,
    object_like::ObjectLike,
    object_pattern_like::ObjectPatternLike,
    parameter_list::{ParameterLayout, ParameterList},
    semicolon::{ClassPropertySemicolon, OptionalSemicolon},
    type_parameters::{FormatTsTypeParameters, FormatTsTypeParametersOptions},
    utils::{
        array::TrailingSeparatorMode,
        statement_body::FormatStatementBody,
        string_utils::{FormatLiteralStringToken, StringLiteralParentKind},
    },
};

pub trait FormatWrite<'ast> {
    fn write(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()>;
}

impl<'a> FormatWrite<'a> for AstNode<'a, Program<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // Print BOM
        if f.source_text().chars().next().is_some_and(|c| c == ZWNBSP) {
            write!(f, "\u{feff}");
        }
        write!(
            f,
            [
                self.hashbang(),
                format_leading_comments(self.span().start),
                self.directives(),
                self.body(),
                format_leading_comments(self.span().end), // comments before the EOF token
                hard_line_break()
            ]
        )
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, Directive<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.is_empty() {
            return Ok(());
        }
        let source_text = f.context().source_text();
        let mut join = f.join_nodes_with_hardline();
        for directive in self {
            join.entry(directive.span(), source_text, &directive);
        }
        join.finish()?;
        // if next_sibling's first leading_trivia has more than one new_line, we should add an extra empty line at the end of
        // JsDirectiveList, for example:
        //```js
        // "use strict"; <- first leading new_line
        //  			 <- second leading new_line
        // function foo() {

        // }
        //```
        // so we should keep an extra empty line after JsDirectiveList
        let mut chars = f.source_text()[self.last().unwrap().span().end as usize..].chars();
        let mut count = 0;
        for c in chars.by_ref() {
            if is_line_terminator(c) {
                count += 1;
            } else {
                break;
            }
        }
        // Skip printing newlines if the file has only directives.
        if chars.next().is_none() {
            return Ok(());
        }
        let need_extra_empty_line = count > 1;
        write!(f, if need_extra_empty_line { empty_line() } else { hard_line_break() })
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Directive<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                FormatLiteralStringToken::new(
                    self.expression().span().source_text(f.source_text()),
                    self.expression().span(),
                    /* jsx */
                    false,
                    StringLiteralParentKind::Directive,
                ),
                OptionalSemicolon
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, IdentifierName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name().as_ref(), self.span().start))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, IdentifierReference<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name().as_ref(), self.span().start))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, BindingIdentifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name().as_ref(), self.span().start))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, LabelIdentifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name().as_ref(), self.span().start))
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
        let mut join = f.join_nodes_with_soft_line();
        for (element, formatted) in self.iter().zip(
            FormatSeparatedIter::new(self.iter(), ",").with_trailing_separator(trailing_separator),
        ) {
            join.entry(element.span(), source_text, &formatted);
        }
        join.finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ObjectProperty<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let format_key = format_with(|f| {
            if let PropertyKey::StringLiteral(s) = &self.key().as_ref() {
                FormatLiteralStringToken::new(
                    s.span.source_text(f.source_text()),
                    s.span,
                    /* jsx */
                    false,
                    StringLiteralParentKind::Member,
                )
                .fmt(f)
            } else {
                write!(f, self.key())
            }
        });
        if let AstNodes::Function(func) = self.value().as_ast_nodes() {
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
            if self.method() || is_accessor {
                if func.r#async() {
                    write!(f, ["async", space()])?;
                }
                if func.generator() {
                    write!(f, "*")?;
                }
                if self.computed() {
                    write!(f, "[")?;
                }
                write!(f, format_key)?;
                if self.computed() {
                    write!(f, "]")?;
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
                return Ok(());
            }
        }
        if self.computed() {
            write!(f, "[")?;
        }
        write!(f, format_key)?;
        if self.computed() {
            write!(f, "]")?;
        }
        if !self.shorthand() {
            write!(f, [":", space(), self.value()])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TemplateLiteral<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "`")?;
        let mut expressions = self.expressions().iter();

        for quasi in self.quasis() {
            write!(f, quasi);
            if let Some(expr) = expressions.next() {
                write!(f, ["${", expr, "}"]);
            }
        }

        write!(f, "`")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TaggedTemplateExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.tag(), self.type_arguments(), self.quasi()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TemplateElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.value().raw.as_str(), self.span().start))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ComputedMemberExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object(), self.optional().then_some("?"), "[", self.expression(), "]"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, StaticMemberExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object(), self.optional().then_some("?"), ".", self.property()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, PrivateFieldExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object(), self.optional().then_some("?"), ".", self.field()])
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
            // TODO
            write!(f, [callee, optional.then_some("?."), type_arguments, arguments])
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
        write!(f, self.argument())
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
        write!(
            f,
            [
                self.test(),
                space(),
                "?",
                space(),
                self.consequent(),
                space(),
                ":",
                space(),
                self.alternate()
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AssignmentExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.left(), space(), self.operator().as_str(), space(), self.right()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ArrayAssignmentTarget<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // Copy of `utils::array::write_array_node`
        let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());

        // Specifically do not use format_separated as arrays need separators
        // inserted after holes regardless of the formatting since this makes a
        // semantic difference

        let source_text = f.source_text();
        let mut join = f.join_nodes_with_soft_line();
        let last_index = self.elements().len().saturating_sub(1);

        let test = self.elements().iter();
        for (index, element) in self.elements().iter().enumerate() {
            let separator_mode = match element.as_ref() {
                None => TrailingSeparatorMode::Force,
                _ => TrailingSeparatorMode::Auto,
            };

            let is_disallow = matches!(separator_mode, TrailingSeparatorMode::Disallow);
            let is_force = matches!(separator_mode, TrailingSeparatorMode::Force);

            join.entry(
                SPAN,
                source_text,
                &format_with(|f| {
                    if let Some(element) = element.as_ref() {
                        write!(f, group(element))?;
                    }

                    if is_disallow {
                        // Trailing separators are disallowed, replace it with an empty element
                        // if let Some(separator) = element.trailing_separator()? {
                        // write!(f, [format_removed(separator)])?;
                        // }
                    } else if is_force || index != last_index {
                        // In forced separator mode or if this element is not the last in the list, print the separator
                        // match element.trailing_separator()? {
                        // Some(trailing) => write!(f, [trailing.format()])?,
                        // None => text(",").fmt(f)?,
                        // };
                        ",".fmt(f)?;
                    // } else if let Some(separator) = element.trailing_separator()? {
                    // match trailing_separator {
                    // TrailingSeparator::Omit => {
                    // // write!(f, [format_removed(separator)])?;
                    // }
                    // _ => {
                    // write!(f, format_only_if_breaks(SPAN, separator))?;
                    // }
                    // }
                    } else {
                        write!(f, FormatTrailingCommas::ES5)?;
                    }

                    Ok(())
                }),
            );
        }

        join.finish()
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

impl<'a> FormatWrite<'a> for AstNode<'a, SequenceExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let format_inner = format_with(|f| {
            for (i, expr) in self.expressions().iter().enumerate() {
                if i != 0 {
                    write!(f, [",", line_suffix_boundary(), soft_line_break_or_space()])?;
                }
                write!(f, expr)?;
            }
            Ok(())
        });
        write!(f, group(&format_inner))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Super> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "super")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AwaitExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["await", space(), self.argument()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ChainExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.expression().fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ParenthesizedExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.expression().fmt(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, Statement<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let source_text = f.context().source_text();
        let mut join = f.join_nodes_with_hardline();
        for stmt in self {
            join.entry(stmt.span(), source_text, stmt);
        }
        join.finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Hashbang<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["#!", dynamic_text(self.value().as_str(), self.span().start)])?;
        let count = f.source_text()[self.span().end as usize..]
            .chars()
            .take_while(|&c| is_line_terminator(c))
            .count();
        write!(f, if count <= 1 { hard_line_break() } else { empty_line() })
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

impl<'a> FormatWrite<'a> for AstNode<'a, ExpressionStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
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
            return write!(f, group(&format_args!("for", space(), "(;;)", format_body)));
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
        write!(
            f,
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
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ForOfStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
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
        write!(f, group(&format_inner))
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
            let comments = f.context().comments();
            let dangling_comments = comments.dangling_comments(alternate.span());
            let dangling_line_comment =
                dangling_comments.last().is_some_and(|comment| comment.kind().is_line());
            let has_dangling_comments = !dangling_comments.is_empty();

            let trailing_line_comment = comments
                .trailing_comments(consequent.span().end)
                .iter()
                .any(|comment| comment.kind().is_line());

            let else_on_same_line = matches!(consequent.as_ref(), Statement::BlockStatement(_))
                && !trailing_line_comment
                && !dangling_line_comment;

            if else_on_same_line {
                write!(f, space())?;
            } else {
                write!(f, hard_line_break())?;
            }

            if has_dangling_comments {
                write!(f, format_dangling_comments(self.span()))?;

                if trailing_line_comment || dangling_line_comment {
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

impl<'a> FormatWrite<'a> for AstNode<'a, ReturnStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "return")?;
        if let Some(argument) = self.argument() {
            write!(f, [space(), argument])?;
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

impl<'a> FormatWrite<'a> for AstNode<'a, SwitchStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let discriminant = self.discriminant();
        let cases = self.cases();
        let format_cases =
            format_with(|f| if cases.is_empty() { hard_line_break().fmt(f) } else { cases.fmt(f) });
        write!(
            f,
            [
                "switch",
                space(),
                "(",
                group(&soft_block_indent(&discriminant)),
                ")",
                space(),
                "{",
                block_indent(&format_cases),
                "}"
            ]
        )
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, SwitchCase<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let source_text = f.source_text();
        let mut join = f.join_nodes_with_hardline();
        for case in self {
            join.entry(case.span(), source_text, case);
        }
        join.finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, SwitchCase<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if let Some(test) = self.test() {
            write!(f, ["case", space(), test, ":"])?;
        } else {
            write!(f, ["default", ":"])?;
        }

        let consequent = self.consequent();
        // Whether the first statement in the clause is a BlockStatement, and
        // there are no other non-empty statements. Empties may show up when
        // parsing depending on if the input code includes certain newlines.
        let is_single_block_statement =
            matches!(consequent.as_ref().first(), Some(Statement::BlockStatement(_)))
                && consequent
                    .iter()
                    .filter(|statement| !matches!(statement.as_ref(), Statement::EmptyStatement(_)))
                    .count()
                    == 1;
        // When the case block is empty, the case becomes a fallthrough, so it
        // is collapsed directly on top of the next case (just a single
        // hardline).
        // When the block is a single statement _and_ it's a block statement,
        // then the opening brace of the block can hug the same line as the
        // case. But, if there's more than one statement, then the block
        // _cannot_ hug. This distinction helps clarify that the case continues
        // past the end of the block statement, despite the braces making it
        // seem like it might end.
        // Lastly, the default case is just to break and indent the body.
        //
        // switch (key) {
        //   case fallthrough: // trailing comment
        //   case normalBody:
        //     someWork();
        //     break;
        //
        //   case blockBody: {
        //     const a = 1;
        //     break;
        //   }
        //
        //   case separateBlockBody:
        //     {
        //       breakIsNotInsideTheBlock();
        //     }
        //     break;
        //
        //   default:
        //     break;
        // }
        if consequent.is_empty() {
            // Print nothing to ensure that trailing comments on the same line
            // are printed on the same line. The parent list formatter takes
            // care of inserting a hard line break between cases.
            Ok(())
        } else if is_single_block_statement {
            write!(f, [space(), consequent])
        } else {
            // no line break needed after because it is added by the indent in the switch statement
            write!(f, indent(&format_args!(hard_line_break(), consequent)))
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, LabeledStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
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

impl<'a> FormatWrite<'a> for AstNode<'a, ThrowStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["throw ", self.argument(), OptionalSemicolon])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TryStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let block = self.block();
        let handler = self.handler();
        let finalizer = self.finalizer();
        write!(f, ["try", space(), block])?;
        if let Some(handler) = handler {
            write!(f, [space(), handler])?;
        }
        if let Some(finalizer) = finalizer {
            write!(f, [space(), "finally", space(), finalizer])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, CatchClause<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["catch", space(), self.param(), space(), self.body()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, CatchParameter<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["(", self.pattern(), ")"])
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
        write!(f, [self.left(), space(), "=", space(), self.right()])
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
        if self.is_empty() {
            write!(f, [format_dangling_comments(self.span()).with_block_indent()])?;
        } else {
            // write!(f, [group(&soft_block_indent(&self.elements))])?;
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
        let parentheses_not_needed = true; // self
        // .as_arrow_function_expression()
        // .is_some_and(|expression| can_avoid_parentheses(&expression, f));
        let has_any_decorated_parameter = false; // list.has_any_decorated_parameter();
        let can_hug = false;
        // should_hug_function_parameters(self, f.context().comments(), parentheses_not_needed)?
        // && !has_any_decorated_parameter;
        let layout = if !self.has_parameter() {
            ParameterLayout::NoParameters
        } else if can_hug
        /* || self.is_in_test_call()? */
        {
            ParameterLayout::Hug
        } else {
            ParameterLayout::Default
        };

        write!(f, "(")?;

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

        write!(f, ")")
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
            write!(f, [decorators, content])?;
        } else if decorators.is_empty() {
            write!(f, [decorators, group(&content)])?;
        } else {
            write!(f, [group(&decorators), group(&content)])?;
        }

        // write![f, [FormatInitializerClause::new(initializer.as_ref())]]
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ArrowFunctionExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        FormatJsArrowFunctionExpression::new(self).fmt(f)
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

impl<'a> FormatWrite<'a> for AstNode<'a, ClassBody<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["{", block_indent(&self.body()), "}"])
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, ClassElement<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let source_text = f.source_text();
        let mut join = f.join_nodes_with_hardline();
        for (e1, e2) in self.iter().zip(self.iter().skip(1).map(Some).chain(std::iter::once(None)))
        {
            join.entry(e1.span(), source_text, &(e1, e2));
        }
        join.finish()
    }
}

impl<'a> Format<'a> for (&AstNode<'a, ClassElement<'a>>, Option<&AstNode<'a, ClassElement<'a>>>) {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.0, ClassPropertySemicolon::new(self.0, self.1)])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, MethodDefinition<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.decorators()])?;
        if let Some(accessibility) = &self.accessibility() {
            write!(f, [accessibility.as_str(), space()])?;
        }
        if self.r#type().is_abstract() {
            write!(f, ["abstract", space()])?;
        }
        if self.r#static() {
            write!(f, ["static", space()])?;
        }
        match &self.kind() {
            MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => {}
            MethodDefinitionKind::Get => {
                write!(f, ["get", space()])?;
            }
            MethodDefinitionKind::Set => {
                write!(f, ["set", space()])?;
            }
        }
        if self.value().r#async() {
            write!(f, ["async", space()])?;
        }
        if self.value().generator() {
            write!(f, "*")?;
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
        if let Some(type_parameters) = &self.value().type_parameters() {
            write!(f, type_parameters)?;
        }
        write!(f, group(&self.value().params()))?;
        if let Some(return_type) = &self.value().return_type() {
            write!(f, return_type)?;
        }
        if let Some(body) = &self.value().body() {
            write!(f, [space(), body])?;
        }
        if self.r#type().is_abstract()
            || matches!(self.value().r#type(), FunctionType::TSEmptyBodyFunctionExpression)
        {
            write!(f, OptionalSemicolon)?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, PropertyDefinition<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.decorators())?;
        if self.declare() {
            write!(f, ["declare", space()])?;
        }
        if let Some(accessibility) = self.accessibility() {
            write!(f, [accessibility.as_str(), space()])?;
        }
        if self.r#type() == PropertyDefinitionType::TSAbstractPropertyDefinition {
            write!(f, ["abstract", space()])?;
        }
        if self.r#static() {
            write!(f, ["static", space()])?;
        }
        if self.readonly() {
            write!(f, ["readonly", space()])?;
        }
        if self.computed() {
            write!(f, ["[", self.key(), "]"])?;
        } else if let PropertyKey::StringLiteral(s) = self.key().as_ref() {
            FormatLiteralStringToken::new(
                s.span.source_text(f.source_text()),
                s.span,
                /* jsx */
                false,
                StringLiteralParentKind::Member,
            )
            .fmt(f)?;
        } else {
            write!(f, self.key())?;
        }

        if self.optional() {
            write!(f, "?")?;
        }
        if let Some(type_annotation) = &self.type_annotation() {
            write!(f, type_annotation)?;
        }
        if let Some(value) = &self.value() {
            write!(f, [space(), "=", space(), value])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, PrivateIdentifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["#", dynamic_text(self.name().as_str(), self.span().start)])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, StaticBlock<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["static", space(), "{"])?;
        for stmt in self.body() {
            write!(f, stmt);
        }
        write!(f, "}")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AccessorProperty<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.decorators())?;
        if self.r#type().is_abstract() {
            write!(f, ["abstract", space()])?;
        }
        if let Some(accessibility) = self.accessibility() {
            write!(f, [accessibility.as_str(), space()])?;
        }
        if self.r#static() {
            write!(f, ["static", space()])?;
        }
        if self.r#override() {
            write!(f, ["override", space()])?;
        }
        write!(f, ["accessor", space()])?;
        if self.computed() {
            write!(f, "[")?;
        }
        write!(f, self.key())?;
        if self.computed() {
            write!(f, "]")?;
        }
        if let Some(type_annotation) = &self.type_annotation() {
            write!(f, type_annotation)?;
        }
        if let Some(value) = &self.value() {
            write!(f, [space(), "=", space(), value])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["import"])?;
        if let Some(phase) = &self.phase() {
            write!(f, [".", phase.as_str()])?;
        }
        write!(f, ["(", self.source()])?;
        if let Some(options) = &self.options() {
            write!(f, [",", space(), options])?;
        }
        write!(f, ")")
    }
}

impl<'a> Format<'a> for ImportOrExportKind {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.is_type() { write!(f, ["type", space()]) } else { Ok(()) }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["import", space(), self.import_kind()])?;

        let should_insert_space_around_brackets = f.options().bracket_spacing.value();

        if let Some(specifiers) = self.specifiers() {
            if specifiers.len() == 1
                && specifiers
                    .as_ref()
                    .first()
                    .is_some_and(|s| matches!(s, ImportDeclarationSpecifier::ImportSpecifier(_)))
            {
                write!(
                    f,
                    [
                        "{",
                        maybe_space(should_insert_space_around_brackets),
                        specifiers.first().unwrap(),
                        maybe_space(should_insert_space_around_brackets),
                        "}",
                        space()
                    ]
                )?;
            } else {
                let mut start_index = 0;
                for specifier in specifiers {
                    if !matches!(specifier.as_ref(), ImportDeclarationSpecifier::ImportSpecifier(_))
                    {
                        start_index += 1;
                    }
                }

                let iter = specifiers.iter().take(start_index);
                for (i, specifier) in iter.enumerate() {
                    if i != 0 {
                        write!(f, [",", space()])?;
                    }
                    write!(f, specifier)?;
                }

                let specifiers = specifiers.iter().skip(start_index).collect::<std::vec::Vec<_>>();
                if specifiers.is_empty() {
                    if start_index == 0 {
                        write!(f, ["{}", space()])?;
                    } else {
                        write!(f, space())?;
                    }
                    // write!(f, [format_dangling_comments(self.span).with_soft_block_indent()])?;
                } else {
                    if start_index != 0 {
                        write!(f, [",", space()])?;
                    }
                    write!(
                        f,
                        [
                            "{",
                            group(&soft_block_indent_with_maybe_space(
                                &specifiers,
                                should_insert_space_around_brackets
                            )),
                            "}",
                            space(),
                        ]
                    )?;
                }
            }
            write!(f, ["from", space()])?;
        }

        write!(f, [self.source(), self.with_clause(), OptionalSemicolon])
    }
}

impl<'a> Format<'a> for std::vec::Vec<&AstNode<'a, ImportDeclarationSpecifier<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());
        f.join_with(&soft_line_break_or_space())
            .entries(
                FormatSeparatedIter::new(self.iter(), ",")
                    .with_trailing_separator(trailing_separator),
            )
            .finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.import_kind(), self.imported()])?;
        if self.local().span() != self.imported().span() {
            write!(f, [space(), "as", space(), self.local()])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportDefaultSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.local().fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportNamespaceSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["*", space(), "as", space(), self.local()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, WithClause<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let should_insert_space_around_brackets = f.options().bracket_spacing.value();
        let format_comment = format_with(|f| {
            if self.with_entries().is_empty()
                && f.comments().has_leading_comments(self.span().end - 1)
            {
                write!(f, [space(), format_leading_comments(self.span().end - 1)])
            } else {
                Ok(())
            }
        });
        write!(
            f,
            [
                space(),
                format_comment,
                self.attributes_keyword(),
                space(),
                "{",
                group(&soft_block_indent_with_maybe_space(
                    self.with_entries(),
                    should_insert_space_around_brackets,
                )),
                "}"
            ]
        )
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, ImportAttribute<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());
        f.join_with(&soft_line_break_or_space())
            .entries(
                FormatSeparatedIter::new(self.iter(), ",")
                    .with_trailing_separator(trailing_separator),
            )
            .finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportAttribute<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if let AstNodes::StringLiteral(s) = self.key().as_ast_nodes() {
            if f.options().quote_properties == QuoteProperties::AsNeeded
                && is_identifier_name(s.value().as_str())
            {
                dynamic_text(s.value().as_str(), s.span().start).fmt(f)?;
            } else {
                s.fmt(f)?;
            }
        } else {
            write!(f, self.key())?;
        }
        write!(f, [":", space(), self.value()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ExportNamedDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let declaration = self.declaration();
        let export_kind = self.export_kind();
        let specifiers = self.specifiers();
        let source = self.source();
        let with_clause = self.with_clause();
        let should_insert_space_around_brackets = f.options().bracket_spacing.value();

        if let Some(AstNodes::Class(class)) = declaration.map(AstNode::<Declaration>::as_ast_nodes)
        {
            if !class.decorators().is_empty() {
                write!(f, class.decorators())?;
            }
        }

        write!(f, ["export", space()])?;

        if let Some(decl) = declaration {
            write!(f, decl)?;
        } else {
            write!(f, [export_kind, "{"])?;
            // TODO
            if specifiers.is_empty() {
                // write!(f, [format_dangling_comments(self.span).with_block_indent()])?;
            } else {
                let should_insert_space_around_brackets = f.options().bracket_spacing.value();
                write!(
                    f,
                    group(&soft_block_indent_with_maybe_space(
                        &specifiers,
                        should_insert_space_around_brackets
                    ))
                )?;
            }
            write!(f, [export_kind, "}"])?;
        }

        if let Some(source) = source {
            write!(f, [space(), "from", space(), source])?;
        }

        if let Some(with_clause) = with_clause {
            write!(f, [space(), with_clause])?;
        }
        if declaration.is_none_or(|d| matches!(d.as_ref(), Declaration::VariableDeclaration(_))) {
            write!(f, OptionalSemicolon)?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ExportDefaultDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if let AstNodes::Class(class) = self.declaration().as_ast_nodes() {
            if !class.decorators().is_empty() {
                write!(f, class.decorators())?;
            }
        }
        write!(f, ["export", space(), "default", space(), self.declaration()])?;
        if self.declaration().is_expression() {
            write!(f, OptionalSemicolon)?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ExportAllDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["export", space(), self.export_kind(), "*", space()])?;
        if let Some(name) = &self.exported() {
            write!(f, ["as", space(), name, space()])?;
        }
        write!(f, ["from", space(), self.source(), self.with_clause(), OptionalSemicolon])
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, ExportSpecifier<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());
        f.join_with(&soft_line_break_or_space())
            .entries(
                FormatSeparatedIter::new(self.iter(), ",")
                    .with_trailing_separator(trailing_separator),
            )
            .finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ExportSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.export_kind());
        if self.local().span() == self.exported().span() {
            write!(f, self.local())
        } else {
            write!(f, [self.local(), space(), "as", space(), self.exported()])
        }
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
        FormatLiteralStringToken::new(
            self.span().source_text(f.source_text()),
            self.span(),
            /* jsx */
            false,
            StringLiteralParentKind::Expression,
        )
        .fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, BigIntLiteral<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(&self.raw().unwrap().cow_to_ascii_lowercase(), self.span().start))
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
        let s = format!("{pattern}/{flags}");
        write!(f, dynamic_text(&s, self.span().start))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["<", self.opening_element().name()])?;
        for attr in self.opening_element().attributes() {
            match attr.as_ref() {
                JSXAttributeItem::Attribute(_) => {
                    write!(f, hard_space())?;
                }
                JSXAttributeItem::SpreadAttribute(_) => {
                    write!(f, space())?;
                }
            }
            write!(f, attr)?;
        }
        if self.closing_element().is_none() {
            write!(f, [space(), "/"])?;
        }
        write!(f, ">")?;

        for child in self.children() {
            write!(f, child)?;
        }
        write!(f, self.closing_element())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXOpeningElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // Implemented in JSXElement above due to no access to
        // no `self_closing`.
        unreachable!()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXClosingElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["</", self.name(), ">"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXFragment<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.opening_fragment())?;
        for child in self.children() {
            write!(f, child)?;
        }
        write!(f, self.closing_fragment())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXOpeningFragment> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "<>")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXClosingFragment> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "</>")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXNamespacedName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.namespace(), ":", self.name()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXMemberExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object(), ".", self.property()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXExpressionContainer<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["{", self.expression(), "}"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXEmptyExpression> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXAttribute<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.name())?;
        if let Some(value) = &self.value() {
            write!(f, ["=", value])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXSpreadAttribute<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["{...", self.argument(), "}"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXIdentifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name().as_str(), self.span().start))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXSpreadChild<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let expression = self.expression();
        write!(f, ["{...", expression, "}"])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXText<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.value().as_str(), self.span().start))
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
        let source_text = f.source_text();
        let mut join = f.join_nodes_with_soft_line();
        for (element, formatted) in self.iter().zip(
            FormatSeparatedIter::new(self.iter(), ",")
                .with_trailing_separator(trailing_separator)
                .nodes_grouped(),
        ) {
            join.entry(element.span(), source_text, &formatted);
        }
        join.finish()
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
        write!(
            f,
            [
                self.check_type(),
                " extends ",
                self.extends_type(),
                " ? ",
                self.true_type(),
                " : ",
                self.false_type()
            ]
        )
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

impl<'a> Format<'a> for AstNode<'a, Vec<'a, TSClassImplements<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSClassImplements<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["implements", self.expression(), self.type_arguments()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSInterfaceDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let id = self.id();
        let type_parameters = self.type_parameters();
        let extends = self.extends();
        let body = self.body();

        let should_indent_extends_only = type_parameters
            .as_ref()
            .is_some_and(|params| !f.comments().has_trailing_line_comment(params.span().end));

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

            let id_has_trailing_comments = f.comments().has_trailing_comments(id.span().end);
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
        // let last_index = self.body.len().saturating_sub(1);
        let source_text = f.context().source_text();
        let mut joiner = f.join_nodes_with_soft_line();
        for (index, sig) in self.body().iter().enumerate() {
            joiner.entry(sig.span(), source_text, sig);
        }
        joiner.finish()
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
        let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());
        let source_text = f.source_text();
        let mut join = f.join_nodes_with_soft_line();
        for element in self {
            join.entry(element.span(), source_text, element);
        }
        join.finish()
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
        write!(f, [dynamic_text(self.name().as_str(), self.span().start), self.type_annotation()])
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, TSInterfaceHeritage<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.join_with(&soft_line_break_or_space())
            .entries(
                FormatSeparatedIter::new(self.iter(), ",")
                    .with_trailing_separator(TrailingSeparator::Disallowed),
            )
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

        let comments = f.comments().clone();
        let type_annotation_has_leading_comment = false;
        //TODO
        //
        // mapped_type
        // .as_ref()
        // .is_some_and(|annotation| comments.has_leading_comments(annotation.syntax()));

        let format_inner = format_with(|f| {
            write!(f, FormatLeadingComments::Comments(comments.dangling_comments(self.span())))?;

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

impl<'a> FormatWrite<'a> for AstNode<'a, TSTemplateLiteralType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "`")?;
        let mut quasis = self.quasis().iter();
        let quasi = quasis.next().unwrap();
        write!(f, dynamic_text(quasi.value().raw.as_str(), quasi.span().start));

        for (index, (quasi, types)) in quasis.zip(self.types().iter()).enumerate() {
            write!(f, ["${", types, "}"])?;
            write!(f, dynamic_text(quasi.value().raw.as_str(), quasi.span().start));
        }
        write!(f, "`")
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

impl<'a> Format<'a> for AstNode<'a, Vec<'a, Decorator<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.is_empty() {
            return Ok(());
        }
        for declarator in self {
            write!(f, declarator)?;
        }
        write!(f, hard_line_break())?;
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Decorator<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["@", self.expression()])
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
