mod array_element_list;
mod array_expression;
mod arrow_function_expression;
mod assignment_pattern_property_list;
mod binary_like_expression;
mod binding_property_list;
mod block_statement;
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

use cow_utils::CowUtils;

use oxc_allocator::{Box, Vec};
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
        write,
    },
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

impl<'a, T: Format<'a>> Format<'a> for Box<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.as_ref().fmt(f)
    }
}

pub trait FormatWrite<'ast> {
    fn write(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()>;
}

impl<'a> FormatWrite<'a> for Program<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // Print BOM
        if f.source_text().chars().next().is_some_and(|c| c == ZWNBSP) {
            write!(f, "\u{feff}");
        }
        write!(
            f,
            [
                self.hashbang,
                format_leading_comments(self.span.start),
                self.directives,
                self.body,
                format_leading_comments(self.span.end), // comments before the EOF token
                hard_line_break()
            ]
        )
    }
}

impl<'a> Format<'a> for Vec<'a, Directive<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.is_empty() {
            return Ok(());
        }
        let source_text = f.context().source_text();
        let mut join = f.join_nodes_with_hardline();
        for directive in self {
            join.entry(directive.span, source_text, directive);
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
        let mut chars = f.source_text()[self.last().unwrap().span.end as usize..].chars();
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

impl<'a> FormatWrite<'a> for Directive<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                FormatLiteralStringToken::new(
                    self.expression.span.source_text(f.source_text()),
                    self.expression.span,
                    /* jsx */
                    false,
                    StringLiteralParentKind::Directive,
                ),
                OptionalSemicolon
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for Expression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Expression::BooleanLiteral(it) => it.fmt(f),
            Expression::NullLiteral(it) => it.fmt(f),
            Expression::NumericLiteral(it) => it.fmt(f),
            Expression::BigIntLiteral(it) => it.fmt(f),
            Expression::RegExpLiteral(it) => it.fmt(f),
            Expression::StringLiteral(it) => it.fmt(f),
            Expression::TemplateLiteral(it) => it.fmt(f),
            Expression::Identifier(it) => it.fmt(f),
            Expression::MetaProperty(it) => it.fmt(f),
            Expression::Super(it) => it.fmt(f),
            Expression::ArrayExpression(it) => it.fmt(f),
            Expression::ArrowFunctionExpression(it) => it.fmt(f),
            Expression::AssignmentExpression(it) => it.fmt(f),
            Expression::AwaitExpression(it) => it.fmt(f),
            Expression::BinaryExpression(it) => it.fmt(f),
            Expression::CallExpression(it) => it.fmt(f),
            Expression::ChainExpression(it) => it.fmt(f),
            Expression::ClassExpression(it) => it.fmt(f),
            Expression::ConditionalExpression(it) => it.fmt(f),
            Expression::FunctionExpression(it) => it.fmt(f),
            Expression::ImportExpression(it) => it.fmt(f),
            Expression::LogicalExpression(it) => it.fmt(f),
            Expression::NewExpression(it) => it.fmt(f),
            Expression::ObjectExpression(it) => it.fmt(f),
            Expression::ParenthesizedExpression(it) => it.fmt(f),
            Expression::SequenceExpression(it) => it.fmt(f),
            Expression::TaggedTemplateExpression(it) => it.fmt(f),
            Expression::ThisExpression(it) => it.fmt(f),
            Expression::UnaryExpression(it) => it.fmt(f),
            Expression::UpdateExpression(it) => it.fmt(f),
            Expression::YieldExpression(it) => it.fmt(f),
            Expression::PrivateInExpression(it) => it.fmt(f),
            Expression::JSXElement(it) => it.fmt(f),
            Expression::JSXFragment(it) => it.fmt(f),
            Expression::TSAsExpression(it) => it.fmt(f),
            Expression::TSSatisfiesExpression(it) => it.fmt(f),
            Expression::TSTypeAssertion(it) => it.fmt(f),
            Expression::TSNonNullExpression(it) => it.fmt(f),
            Expression::TSInstantiationExpression(it) => it.fmt(f),
            Expression::V8IntrinsicExpression(it) => it.fmt(f),
            match_member_expression!(Expression) => self.to_member_expression().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for IdentifierName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name.as_ref(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for IdentifierReference<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name.as_ref(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for BindingIdentifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name.as_ref(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for LabelIdentifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name.as_ref(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for ThisExpression {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "this")
    }
}

impl<'a> FormatWrite<'a> for ArrayExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        FormatArrayExpression::new(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for ArrayExpressionElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::SpreadElement(elem) => elem.fmt(f),
            Self::Elision(elision) => elision.fmt(f),
            _ => self.to_expression().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for Elision {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ObjectExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        ObjectLike::ObjectExpression(self).fmt(f)
    }
}

impl<'a> Format<'a> for Vec<'a, ObjectPropertyKind<'a>> {
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

impl<'a> FormatWrite<'a> for ObjectPropertyKind<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::ObjectProperty(p) => p.fmt(f),
            Self::SpreadProperty(p) => p.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for ObjectProperty<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let format_key = format_with(|f| {
            if let PropertyKey::StringLiteral(s) = &self.key {
                FormatLiteralStringToken::new(
                    s.span.source_text(f.source_text()),
                    s.span,
                    /* jsx */
                    false,
                    StringLiteralParentKind::Member,
                )
                .fmt(f)
            } else {
                write!(f, self.key)
            }
        });
        if let Expression::FunctionExpression(func) = &self.value {
            let is_accessor = match &self.kind {
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
                if func.r#async {
                    write!(f, ["async", space()])?;
                }
                if func.generator {
                    write!(f, "*")?;
                }
                if self.computed {
                    write!(f, "[")?;
                }
                write!(f, format_key)?;
                if self.computed {
                    write!(f, "]")?;
                }
                if let Some(type_parameters) = &func.type_parameters {
                    write!(f, type_parameters)?;
                }
                write!(f, group(&func.params))?;
                if let Some(return_type) = &func.return_type {
                    write!(f, return_type)?;
                }
                if let Some(body) = &func.body {
                    write!(f, [space(), body])?;
                }
                return Ok(());
            }
        }
        if self.computed {
            write!(f, "[")?;
        }
        write!(f, format_key)?;
        if self.computed {
            write!(f, "]")?;
        }
        if !self.shorthand {
            write!(f, [":", space(), self.value])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for PropertyKey<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::StaticIdentifier(key) => key.fmt(f),
            Self::PrivateIdentifier(key) => key.fmt(f),
            match_expression!(Self) => self.to_expression().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TemplateLiteral<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "`")?;
        let mut expressions = self.expressions.iter();

        for quasi in &self.quasis {
            write!(f, quasi);
            if let Some(expr) = expressions.next() {
                write!(f, ["${", expr, "}"]);
            }
        }

        write!(f, "`")
    }
}

impl<'a> FormatWrite<'a> for TaggedTemplateExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.tag, self.type_arguments, self.quasi])
    }
}

impl<'a> FormatWrite<'a> for TemplateElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.value.raw.as_str(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for MemberExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::ComputedMemberExpression(e) => e.fmt(f),
            Self::StaticMemberExpression(e) => e.fmt(f),
            Self::PrivateFieldExpression(e) => e.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for ComputedMemberExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object, self.optional.then_some("?"), "[", self.expression, "]"])
    }
}

impl<'a> FormatWrite<'a> for StaticMemberExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object, self.optional.then_some("?"), ".", self.property])
    }
}

impl<'a> FormatWrite<'a> for PrivateFieldExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object, self.optional.then_some("?"), ".", self.field])
    }
}

impl<'a> FormatWrite<'a> for CallExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { callee, type_arguments, arguments, optional, .. } = self;

        if callee.as_member_expression().is_some_and(|e| {
            matches!(
                e,
                MemberExpression::StaticMemberExpression(_)
                    | MemberExpression::ComputedMemberExpression(_)
            )
        }) && !callee.needs_parentheses(f.parent_stack())
        {
            // TODO
            write!(f, [callee, optional.then_some("?."), type_arguments, arguments])
        } else {
            let format_inner = format_with(|f| {
                write!(f, [callee, optional.then_some("?."), type_arguments, arguments])
            });
            if matches!(callee, Expression::CallExpression(_)) {
                write!(f, [group(&format_inner)])
            } else {
                write!(f, [format_inner])
            }
        }
    }
}

impl<'a> Format<'a> for Vec<'a, Argument<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.is_empty() {
            return write!(
                f,
                [
                    "(",
                    // format_dangling_comments(node.syntax()).with_soft_block_indent(),
                    ")"
                ]
            );
        }

        write!(
            f,
            [
                "(",
                &group(&soft_block_indent(&format_with(|f| {
                    let separated = FormatSeparatedIter::new(self.iter(), ",")
                        .with_trailing_separator(TrailingSeparator::Omit);
                    write_arguments_multi_line(separated, f)
                }))),
                ")"
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for NewExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { callee, type_arguments, arguments, .. } = self;
        write!(f, ["new", space(), callee, type_arguments, arguments])
    }
}

impl<'a> FormatWrite<'a> for MetaProperty<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.meta, ".", self.property])
    }
}

impl<'a> FormatWrite<'a> for SpreadElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["...", self.argument])
    }
}

impl<'a> FormatWrite<'a> for Argument<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::SpreadElement(e) => e.fmt(f),
            match_expression!(Argument) => self.to_expression().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for UpdateExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.prefix {
            write!(f, self.operator.as_str())?;
        }
        write!(f, self.argument)?;
        if !self.prefix {
            write!(f, self.operator.as_str())?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for UnaryExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.operator.as_str());
        if self.operator.is_keyword() {
            write!(f, space());
        }
        write!(f, self.argument)
    }
}

impl<'a> FormatWrite<'a> for BinaryExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        BinaryLikeExpression::BinaryExpression(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for PrivateInExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.left, space(), "in", space(), self.right])
    }
}

impl<'a> FormatWrite<'a> for LogicalExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        BinaryLikeExpression::LogicalExpression(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for ConditionalExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                self.test,
                space(),
                "?",
                space(),
                self.consequent,
                space(),
                ":",
                space(),
                self.alternate
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AssignmentExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.left, space(), self.operator.as_str(), space(), self.right])
    }
}

impl<'a> FormatWrite<'a> for AssignmentTarget<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            match_simple_assignment_target!(Self) => self.to_simple_assignment_target().fmt(f),
            match_assignment_target_pattern!(Self) => self.to_assignment_target_pattern().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for SimpleAssignmentTarget<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.fmt(f),
            Self::ComputedMemberExpression(e) => e.fmt(f),
            Self::StaticMemberExpression(e) => e.fmt(f),
            Self::PrivateFieldExpression(e) => e.fmt(f),
            Self::TSAsExpression(e) => e.fmt(f),
            Self::TSSatisfiesExpression(e) => e.fmt(f),
            Self::TSNonNullExpression(e) => e.fmt(f),
            Self::TSTypeAssertion(e) => e.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AssignmentTargetPattern<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::ArrayAssignmentTarget(target) => target.fmt(f),
            Self::ObjectAssignmentTarget(target) => target.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for ArrayAssignmentTarget<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // Copy of `utils::array::write_array_node`
        let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());

        // Specifically do not use format_separated as arrays need separators
        // inserted after holes regardless of the formatting since this makes a
        // semantic difference

        let source_text = f.source_text();
        let mut join = f.join_nodes_with_soft_line();
        let last_index = self.elements.len().saturating_sub(1);

        for (index, element) in self.elements.iter().enumerate() {
            let separator_mode = match element {
                None => TrailingSeparatorMode::Force,
                _ => TrailingSeparatorMode::Auto,
            };

            let is_disallow = matches!(separator_mode, TrailingSeparatorMode::Disallow);
            let is_force = matches!(separator_mode, TrailingSeparatorMode::Force);

            join.entry(
                SPAN,
                source_text,
                &format_with(|f| {
                    if let Some(e) = element.as_ref() {
                        write!(f, group(&e))?;
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

impl<'a> FormatWrite<'a> for ObjectAssignmentTarget<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        ObjectPatternLike::ObjectAssignmentTarget(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AssignmentTargetRest<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["...", self.target])
    }
}

impl<'a> FormatWrite<'a> for AssignmentTargetMaybeDefault<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::AssignmentTargetWithDefault(target) => target.fmt(f),
            match_assignment_target!(Self) => self.to_assignment_target().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AssignmentTargetWithDefault<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.binding, space(), "=", space(), self.init])
    }
}

impl<'a> FormatWrite<'a> for AssignmentTargetProperty<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::AssignmentTargetPropertyIdentifier(ident) => ident.fmt(f),
            Self::AssignmentTargetPropertyProperty(prop) => prop.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AssignmentTargetPropertyIdentifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.binding)?;
        if let Some(expr) = &self.init {
            write!(f, [space(), "=", space(), expr])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AssignmentTargetPropertyProperty<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.computed {
            write!(f, "[")?;
        }
        write!(f, self.name)?;
        if self.computed {
            write!(f, "]")?;
        }
        write!(f, [":", space(), self.binding])?;
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for SequenceExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let format_inner = format_with(|f| {
            for (i, expr) in self.expressions.iter().enumerate() {
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

impl<'a> FormatWrite<'a> for Super {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "super")
    }
}

impl<'a> FormatWrite<'a> for AwaitExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["await", space(), self.argument])
    }
}

impl<'a> FormatWrite<'a> for ChainExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.expression.fmt(f)
    }
}

impl<'a> FormatWrite<'a> for ChainElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::CallExpression(expr) => expr.fmt(f),
            Self::TSNonNullExpression(expr) => expr.fmt(f),
            Self::ComputedMemberExpression(expr) => expr.fmt(f),
            Self::StaticMemberExpression(expr) => expr.fmt(f),
            Self::PrivateFieldExpression(expr) => expr.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for ParenthesizedExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.expression.fmt(f)
    }
}

impl<'a> Format<'a> for Vec<'a, Statement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let source_text = f.context().source_text();
        let mut join = f.join_nodes_with_hardline();
        for stmt in self {
            join.entry(stmt.span(), source_text, stmt);
        }
        join.finish()
    }
}

impl<'a> FormatWrite<'a> for Statement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::BlockStatement(s) => s.fmt(f),
            Self::BreakStatement(s) => s.fmt(f),
            Self::ContinueStatement(s) => s.fmt(f),
            Self::DebuggerStatement(s) => s.fmt(f),
            Self::DoWhileStatement(s) => s.fmt(f),
            Self::EmptyStatement(s) => s.fmt(f),
            Self::ExpressionStatement(s) => s.fmt(f),
            Self::ForInStatement(s) => s.fmt(f),
            Self::ForOfStatement(s) => s.fmt(f),
            Self::ForStatement(s) => s.fmt(f),
            Self::IfStatement(s) => s.fmt(f),
            Self::LabeledStatement(s) => s.fmt(f),
            Self::ReturnStatement(s) => s.fmt(f),
            Self::SwitchStatement(s) => s.fmt(f),
            Self::ThrowStatement(s) => s.fmt(f),
            Self::TryStatement(s) => s.fmt(f),
            Self::WhileStatement(s) => s.fmt(f),
            Self::WithStatement(s) => s.fmt(f),
            Self::VariableDeclaration(s) => write!(f, [s, OptionalSemicolon]),
            Self::FunctionDeclaration(s) => s.fmt(f),
            Self::ClassDeclaration(s) => s.fmt(f),
            Self::TSTypeAliasDeclaration(s) => s.fmt(f),
            Self::TSInterfaceDeclaration(s) => s.fmt(f),
            Self::TSEnumDeclaration(s) => s.fmt(f),
            Self::TSModuleDeclaration(s) => s.fmt(f),
            Self::TSImportEqualsDeclaration(s) => s.fmt(f),
            match_module_declaration!(Statement) => self.to_module_declaration().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for Hashbang<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["#!", dynamic_text(self.value.as_str(), self.span.start)])?;
        let count = f.source_text()[self.span.end as usize..]
            .chars()
            .take_while(|&c| is_line_terminator(c))
            .count();
        write!(f, if count <= 1 { hard_line_break() } else { empty_line() })
    }
}

impl<'a> FormatWrite<'a> for Declaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Declaration::VariableDeclaration(d) => d.fmt(f),
            Declaration::FunctionDeclaration(d) => d.fmt(f),
            Declaration::ClassDeclaration(d) => d.fmt(f),
            Declaration::TSTypeAliasDeclaration(d) => d.fmt(f),
            Declaration::TSInterfaceDeclaration(d) => d.fmt(f),
            Declaration::TSEnumDeclaration(d) => d.fmt(f),
            Declaration::TSModuleDeclaration(d) => d.fmt(f),
            Declaration::TSImportEqualsDeclaration(d) => d.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for EmptyStatement {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if matches!(
            f.parent_kind(),
            AstKind::DoWhileStatement(_)
                | AstKind::IfStatement(_)
                | AstKind::WhileStatement(_)
                | AstKind::ForStatement(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_)
                | AstKind::WithStatement(_)
        ) {
            write!(f, ";")
        } else {
            Ok(())
        }
    }
}

impl<'a> FormatWrite<'a> for ExpressionStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression, OptionalSemicolon])
    }
}

impl<'a> FormatWrite<'a> for DoWhileStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { body, test, .. } = self;
        write!(f, group(&format_args!("do", FormatStatementBody::new(body))))?;
        if matches!(body, Statement::BlockStatement(_)) {
            write!(f, space())?;
        } else {
            write!(f, hard_line_break())?;
        }
        write!(f, ["while", space(), "(", group(&soft_block_indent(test)), ")", OptionalSemicolon])
    }
}

impl<'a> FormatWrite<'a> for WhileStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { test, body, .. } = self;
        write!(
            f,
            group(&format_args!(
                "while",
                space(),
                "(",
                group(&soft_block_indent(test)),
                ")",
                FormatStatementBody::new(body)
            ))
        )
    }
}

impl<'a> FormatWrite<'a> for ForStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let ForStatement { init, test, update, body, .. } = self;
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

impl<'a> FormatWrite<'a> for ForStatementInit<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::VariableDeclaration(var) => var.fmt(f),
            _ => self.to_expression().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for ForInStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { left, right, body, .. } = self;
        write!(
            f,
            group(&format_args!(
                "for",
                space(),
                "(",
                left,
                space(),
                "in",
                space(),
                right,
                ")",
                FormatStatementBody::new(body)
            ))
        )
    }
}

impl<'a> FormatWrite<'a> for ForStatementLeft<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            ForStatementLeft::VariableDeclaration(var) => var.fmt(f),
            match_assignment_target!(ForStatementLeft) => self.to_assignment_target().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for ForOfStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { r#await, left, right, body, .. } = self;
        let format_inner = format_with(|f| {
            write!(f, "for")?;
            if *r#await {
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

impl<'a> FormatWrite<'a> for IfStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let IfStatement { test, consequent, alternate, .. } = self;
        write!(
            f,
            group(&format_args!(
                "if",
                space(),
                "(",
                group(&soft_block_indent(test)),
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

            let else_on_same_line = matches!(consequent, Statement::BlockStatement(_))
                && !trailing_line_comment
                && !dangling_line_comment;

            if else_on_same_line {
                write!(f, space())?;
            } else {
                write!(f, hard_line_break())?;
            }

            if has_dangling_comments {
                write!(f, format_dangling_comments(self.span))?;

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
                    group(
                        &FormatStatementBody::new(alternate)
                            .with_forced_space(matches!(alternate, Statement::IfStatement(_)))
                    )
                ]
            )?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ContinueStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "continue")?;
        if let Some(label) = &self.label {
            write!(f, [space(), label])?;
        }
        write!(f, OptionalSemicolon)
    }
}

impl<'a> FormatWrite<'a> for BreakStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "break")?;
        if let Some(label) = &self.label {
            write!(f, [space(), label])?;
        }
        write!(f, OptionalSemicolon)
    }
}

impl<'a> FormatWrite<'a> for ReturnStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "return")?;
        if let Some(argument) = &self.argument {
            write!(f, [space(), argument])?;
        }
        write!(f, OptionalSemicolon)
    }
}

impl<'a> FormatWrite<'a> for WithStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { object, body, .. } = self;
        write!(
            f,
            group(&format_args!("with", space(), "(", object, ")", FormatStatementBody::new(body)))
        )
    }
}

impl<'a> FormatWrite<'a> for SwitchStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { discriminant, cases, .. } = self;
        let format_cases =
            format_with(|f| if cases.is_empty() { hard_line_break().fmt(f) } else { cases.fmt(f) });
        write!(
            f,
            [
                "switch",
                space(),
                "(",
                group(&soft_block_indent(discriminant)),
                ")",
                space(),
                "{",
                block_indent(&format_cases),
                "}"
            ]
        )
    }
}

impl<'a> Format<'a> for Vec<'a, SwitchCase<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let source_text = f.source_text();
        let mut join = f.join_nodes_with_hardline();
        for case in self {
            join.entry(case.span, source_text, case);
        }
        join.finish()
    }
}

impl<'a> FormatWrite<'a> for SwitchCase<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { test, consequent, .. } = self;

        if let Some(test) = test {
            write!(f, ["case", space(), test, ":"])?;
        } else {
            write!(f, ["default", ":"])?;
        }

        // Whether the first statement in the clause is a BlockStatement, and
        // there are no other non-empty statements. Empties may show up when
        // parsing depending on if the input code includes certain newlines.
        let is_single_block_statement =
            matches!(consequent.iter().next(), Some(Statement::BlockStatement(_)))
                && consequent
                    .iter()
                    .filter(|statement| !matches!(statement, Statement::EmptyStatement(_)))
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

impl<'a> FormatWrite<'a> for LabeledStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { label, body, .. } = self;
        write!(f, [label, ":"])?;
        match body {
            Statement::EmptyStatement(_) => {
                // If the body is an empty statement, force semicolon insertion
                write!(f, ";")
            }
            body => {
                write!(f, [space(), body])
            }
        }
    }
}

impl<'a> FormatWrite<'a> for ThrowStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["throw ", self.argument, OptionalSemicolon])
    }
}

impl<'a> FormatWrite<'a> for TryStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { block, handler, finalizer, .. } = self;
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

impl<'a> FormatWrite<'a> for CatchClause<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["catch", space(), self.param, space(), self.body])
    }
}

impl<'a> FormatWrite<'a> for CatchParameter<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["(", self.pattern, ")"])
    }
}

impl<'a> FormatWrite<'a> for DebuggerStatement {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["debugger", OptionalSemicolon])
    }
}

impl<'a> FormatWrite<'a> for BindingPattern<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.kind)?;
        if self.optional {
            write!(f, "?")?;
        }
        if let Some(type_annotation) = &self.type_annotation {
            write!(f, type_annotation)?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for BindingPatternKind<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::BindingIdentifier(p) => p.fmt(f),
            Self::ObjectPattern(p) => p.fmt(f),
            Self::ArrayPattern(p) => p.fmt(f),
            Self::AssignmentPattern(p) => p.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AssignmentPattern<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.left, space(), "=", space(), self.right])
    }
}

impl<'a> FormatWrite<'a> for ObjectPattern<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        ObjectPatternLike::ObjectPattern(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for BindingProperty<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { key, value, shorthand, computed, .. } = self;
        let group_id = f.group_id("assignment");
        let format_inner = format_with(|f| {
            if *computed {
                write!(f, "[")?;
            }
            if !*shorthand {
                write!(f, key)?;
            }
            if *computed {
                write!(f, "]")?;
            }
            if *shorthand {
                write!(f, value)
            } else {
                write!(
                    f,
                    [
                        ":",
                        group(&indent(&soft_line_break_or_space())).with_group_id(Some(group_id)),
                        line_suffix_boundary(),
                        indent_if_group_breaks(&value, group_id)
                    ]
                )
            }
        });
        write!(f, group(&format_inner))
    }
}

impl<'a> FormatWrite<'a> for ArrayPattern<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "[")?;
        if self.is_empty() {
            write!(f, [format_dangling_comments(self.span).with_block_indent()])?;
        } else {
            // write!(f, [group(&soft_block_indent(&self.elements))])?;
        }
        write!(f, "]")
    }
}

impl<'a> FormatWrite<'a> for BindingRestElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["...", self.argument])
    }
}

impl<'a> FormatWrite<'a> for FormalParameters<'a> {
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
                write!(f, format_dangling_comments(self.span).with_soft_block_indent())?;
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

impl<'a> FormatWrite<'a> for FormalParameter<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { decorators, pattern, accessibility, readonly, r#override, .. } = self;

        let content = format_with(|f| {
            if let Some(accessibility) = accessibility {
                write!(f, [accessibility.as_str(), space()])?;
            }
            if *r#override {
                write!(f, ["override", space()])?;
            }
            if *readonly {
                write!(f, ["readonly", space()])?;
            }
            write!(f, pattern)
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

impl<'a> FormatWrite<'a> for ArrowFunctionExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        FormatJsArrowFunctionExpression::new(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for YieldExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "yield")?;
        if self.delegate {
            write!(f, "*")?;
        }
        if let Some(argument) = &self.argument {
            write!(f, [space(), argument])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ClassBody<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["{", block_indent(&self.body), "}"])
    }
}

impl<'a> Format<'a> for Vec<'a, ClassElement<'a>> {
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

impl<'a> Format<'a> for (&ClassElement<'a>, Option<&ClassElement<'a>>) {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.0, ClassPropertySemicolon::new(self.0, self.1)])
    }
}

impl<'a> FormatWrite<'a> for ClassElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::StaticBlock(e) => e.fmt(f),
            Self::MethodDefinition(e) => e.fmt(f),
            Self::PropertyDefinition(e) => e.fmt(f),
            Self::AccessorProperty(e) => e.fmt(f),
            Self::TSIndexSignature(e) => e.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for MethodDefinition<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.decorators])?;
        if let Some(accessibility) = &self.accessibility {
            write!(f, [accessibility.as_str(), space()])?;
        }
        if self.r#type.is_abstract() {
            write!(f, ["abstract", space()])?;
        }
        if self.r#static {
            write!(f, ["static", space()])?;
        }
        match &self.kind {
            MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => {}
            MethodDefinitionKind::Get => {
                write!(f, ["get", space()])?;
            }
            MethodDefinitionKind::Set => {
                write!(f, ["set", space()])?;
            }
        }
        if self.value.r#async {
            write!(f, ["async", space()])?;
        }
        if self.value.generator {
            write!(f, "*")?;
        }
        if self.computed {
            write!(f, "[")?;
        }
        write!(f, self.key)?;
        if self.computed {
            write!(f, "]")?;
        }
        if self.optional {
            write!(f, "?")?;
        }
        if let Some(type_parameters) = &self.value.type_parameters {
            write!(f, type_parameters)?;
        }
        write!(f, group(&self.value.params))?;
        if let Some(return_type) = &self.value.return_type {
            write!(f, return_type)?;
        }
        if let Some(body) = &self.value.body {
            write!(f, [space(), body])?;
        }
        if self.r#type.is_abstract()
            || matches!(self.value.r#type, FunctionType::TSEmptyBodyFunctionExpression)
        {
            write!(f, OptionalSemicolon)?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for PropertyDefinition<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.decorators)?;
        if self.declare {
            write!(f, ["declare", space()])?;
        }
        if let Some(accessibility) = self.accessibility {
            write!(f, [accessibility.as_str(), space()])?;
        }
        if self.r#type == PropertyDefinitionType::TSAbstractPropertyDefinition {
            write!(f, ["abstract", space()])?;
        }
        if self.r#static {
            write!(f, ["static", space()])?;
        }
        if self.readonly {
            write!(f, ["readonly", space()])?;
        }
        if self.computed {
            write!(f, ["[", self.key, "]"])?;
        } else if let PropertyKey::StringLiteral(s) = &self.key {
            FormatLiteralStringToken::new(
                s.span.source_text(f.source_text()),
                s.span,
                /* jsx */
                false,
                StringLiteralParentKind::Member,
            )
            .fmt(f)?;
        } else {
            write!(f, self.key)?;
        }

        if self.optional {
            write!(f, "?")?;
        }
        if let Some(type_annotation) = &self.type_annotation {
            write!(f, type_annotation)?;
        }
        if let Some(value) = &self.value {
            write!(f, [space(), "=", space(), value])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for PrivateIdentifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["#", dynamic_text(self.name.as_str(), self.span.start)])
    }
}

impl<'a> FormatWrite<'a> for StaticBlock<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["static", space(), "{"])?;
        for stmt in &self.body {
            write!(f, stmt);
        }
        write!(f, "}")
    }
}

impl<'a> FormatWrite<'a> for ModuleDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::ImportDeclaration(decl) => decl.fmt(f),
            Self::ExportAllDeclaration(decl) => decl.fmt(f),
            Self::ExportDefaultDeclaration(decl) => decl.fmt(f),
            Self::ExportNamedDeclaration(decl) => decl.fmt(f),
            Self::TSExportAssignment(decl) => decl.fmt(f),
            Self::TSNamespaceExportDeclaration(decl) => decl.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AccessorProperty<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.decorators)?;
        if self.r#type.is_abstract() {
            write!(f, ["abstract", space()])?;
        }
        if let Some(accessibility) = self.accessibility {
            write!(f, [accessibility.as_str(), space()])?;
        }
        if self.r#static {
            write!(f, ["static", space()])?;
        }
        if self.r#override {
            write!(f, ["override", space()])?;
        }
        write!(f, ["accessor", space()])?;
        if self.computed {
            write!(f, "[")?;
        }
        write!(f, self.key)?;
        if self.computed {
            write!(f, "]")?;
        }
        if let Some(type_annotation) = &self.type_annotation {
            write!(f, type_annotation)?;
        }
        if let Some(value) = &self.value {
            write!(f, [space(), "=", space(), value])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ImportExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["import"])?;
        if let Some(phase) = &self.phase {
            write!(f, [".", phase.as_str()])?;
        }
        write!(f, ["(", self.source])?;
        if let Some(options) = &self.options {
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

impl<'a> FormatWrite<'a> for ImportDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["import", space(), self.import_kind])?;

        let should_insert_space_around_brackets = f.options().bracket_spacing.value();

        if let Some(specifiers) = &self.specifiers {
            if specifiers.len() == 1
                && specifiers
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
                    if !matches!(specifier, ImportDeclarationSpecifier::ImportSpecifier(_)) {
                        start_index += 1;
                    }
                }

                let (before, after) = specifiers.split_at(start_index);

                for (i, specifier) in before.iter().enumerate() {
                    if i != 0 {
                        write!(f, [",", space()])?;
                    }
                    write!(f, specifier)?;
                }

                let specifiers = after;
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

        write!(f, [self.source, self.with_clause, OptionalSemicolon])
    }
}

impl<'a> Format<'a> for &[ImportDeclarationSpecifier<'a>] {
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

impl<'a> FormatWrite<'a> for ImportDeclarationSpecifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::ImportSpecifier(s) => s.fmt(f),
            Self::ImportDefaultSpecifier(s) => s.fmt(f),
            Self::ImportNamespaceSpecifier(s) => s.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for ImportSpecifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { imported, local, import_kind, .. } = self;
        write!(f, [import_kind, imported])?;
        if local.span != imported.span() {
            write!(f, [space(), "as", space(), local])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ImportDefaultSpecifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.local.fmt(f)
    }
}

impl<'a> FormatWrite<'a> for ImportNamespaceSpecifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["*", space(), "as", space(), self.local])
    }
}

impl<'a> FormatWrite<'a> for WithClause<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let should_insert_space_around_brackets = f.options().bracket_spacing.value();
        let format_comment = format_with(|f| {
            if self.with_entries.is_empty() && f.comments().has_leading_comments(self.span.end - 1)
            {
                write!(f, [space(), format_leading_comments(self.span.end - 1)])
            } else {
                Ok(())
            }
        });
        write!(
            f,
            [
                space(),
                format_comment,
                self.attributes_keyword,
                space(),
                "{",
                group(&soft_block_indent_with_maybe_space(
                    &self.with_entries,
                    should_insert_space_around_brackets,
                )),
                "}"
            ]
        )
    }
}

impl<'a> Format<'a> for Vec<'a, ImportAttribute<'a>> {
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

impl<'a> FormatWrite<'a> for ImportAttribute<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.key, ":", space(), self.value])
    }
}

impl<'a> FormatWrite<'a> for ImportAttributeKey<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::Identifier(ident) => ident.fmt(f),
            Self::StringLiteral(s) => {
                if f.options().quote_properties == QuoteProperties::AsNeeded
                    && is_identifier_name(s.value.as_str())
                {
                    dynamic_text(s.value.as_str(), s.span.start).fmt(f)
                } else {
                    s.fmt(f)
                }
            }
        }
    }
}

impl<'a> FormatWrite<'a> for ExportNamedDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { declaration, export_kind, specifiers, source, with_clause, .. } = self;
        let should_insert_space_around_brackets = f.options().bracket_spacing.value();

        if let Some(Declaration::ClassDeclaration(class)) = declaration {
            if !class.decorators.is_empty() {
                write!(f, class.decorators)?;
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
                        specifiers,
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
        if declaration.is_none()
            || declaration
                .as_ref()
                .is_some_and(|d| matches!(d, Declaration::VariableDeclaration(_)))
        {
            write!(f, OptionalSemicolon)?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ExportDefaultDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if let ExportDefaultDeclarationKind::ClassDeclaration(class) = &self.declaration {
            if !class.decorators.is_empty() {
                write!(f, class.decorators)?;
            }
        }
        write!(f, ["export", space(), "default", space(), self.declaration])?;
        if self.declaration.is_expression() {
            write!(f, OptionalSemicolon)?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ExportAllDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["export", space(), self.export_kind, "*", space()])?;
        if let Some(name) = &self.exported {
            write!(f, ["as", space(), name, space()])?;
        }
        write!(f, ["from", space(), self.source, self.with_clause, OptionalSemicolon])
    }
}

impl<'a> Format<'a> for Vec<'a, ExportSpecifier<'a>> {
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

impl<'a> FormatWrite<'a> for ExportSpecifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.export_kind);
        if self.local.span() == self.exported.span() {
            write!(f, self.local)
        } else {
            write!(f, [self.local, space(), "as", space(), self.exported])
        }
    }
}

impl<'a> FormatWrite<'a> for ExportDefaultDeclarationKind<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::FunctionDeclaration(decl) => decl.fmt(f),
            Self::ClassDeclaration(decl) => decl.fmt(f),
            Self::TSInterfaceDeclaration(decl) => decl.fmt(f),
            match_expression!(Self) => self.to_expression().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for ModuleExportName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::IdentifierName(ident) => ident.fmt(f),
            Self::IdentifierReference(ident) => ident.fmt(f),
            Self::StringLiteral(literal) => literal.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for V8IntrinsicExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["%", self.name, self.arguments])
    }
}

impl<'a> FormatWrite<'a> for BooleanLiteral {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, if self.value { "true" } else { "false" })
    }
}

impl<'a> FormatWrite<'a> for NullLiteral {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "null")
    }
}

impl<'a> FormatWrite<'a> for NumericLiteral<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_number_token(
            self.span.source_text(f.source_text()),
            self.span,
            NumberFormatOptions::default().keep_one_trailing_decimal_zero(),
        )
        .fmt(f)
    }
}

impl<'a> FormatWrite<'a> for StringLiteral<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        FormatLiteralStringToken::new(
            self.span.source_text(f.source_text()),
            self.span,
            /* jsx */
            false,
            StringLiteralParentKind::Expression,
        )
        .fmt(f)
    }
}

impl<'a> FormatWrite<'a> for BigIntLiteral<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(&self.raw.as_str().cow_to_ascii_lowercase(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for RegExpLiteral<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let raw = self.raw.unwrap().as_str();
        let (pattern, flags) = raw.rsplit_once('/').unwrap();
        // TODO: print the flags without allocation.
        let mut flags = flags.chars().collect::<std::vec::Vec<_>>();
        flags.sort_unstable();
        let flags = flags.into_iter().collect::<String>();
        let s = format!("{pattern}/{flags}");
        write!(f, dynamic_text(&s, self.span.start))
    }
}

impl<'a> FormatWrite<'a> for JSXElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["<", self.opening_element.name])?;
        for attr in &self.opening_element.attributes {
            match attr {
                JSXAttributeItem::Attribute(_) => {
                    write!(f, hard_space())?;
                }
                JSXAttributeItem::SpreadAttribute(_) => {
                    write!(f, space())?;
                }
            }
            write!(f, attr)?;
        }
        if self.closing_element.is_none() {
            write!(f, [space(), "/"])?;
        }
        write!(f, ">")?;

        for child in &self.children {
            write!(f, child)?;
        }
        write!(f, self.closing_element)
    }
}

impl<'a> FormatWrite<'a> for JSXOpeningElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // Implemented in JSXElement above due to no access to
        // no `self_closing`.
        unreachable!()
    }
}

impl<'a> FormatWrite<'a> for JSXClosingElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["</", self.name, ">"])
    }
}

impl<'a> FormatWrite<'a> for JSXFragment<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.opening_fragment)?;
        for child in &self.children {
            write!(f, child)?;
        }
        write!(f, self.closing_fragment)
    }
}

impl<'a> FormatWrite<'a> for JSXOpeningFragment {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "<>")
    }
}

impl<'a> FormatWrite<'a> for JSXClosingFragment {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "</>")
    }
}

impl<'a> FormatWrite<'a> for JSXElementName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::Identifier(identifier) => identifier.fmt(f),
            Self::IdentifierReference(identifier) => identifier.fmt(f),
            Self::NamespacedName(namespaced_name) => namespaced_name.fmt(f),
            Self::MemberExpression(member_expr) => member_expr.fmt(f),
            Self::ThisExpression(expr) => expr.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for JSXNamespacedName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.namespace, ":", self.name])
    }
}

impl<'a> FormatWrite<'a> for JSXMemberExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object, ".", self.property])
    }
}

impl<'a> FormatWrite<'a> for JSXMemberExpressionObject<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::IdentifierReference(ident) => ident.fmt(f),
            Self::MemberExpression(member_expr) => member_expr.fmt(f),
            Self::ThisExpression(expr) => expr.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for JSXExpressionContainer<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["{", self.expression, "}"])
    }
}

impl<'a> FormatWrite<'a> for JSXExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::EmptyExpression(expr) => expr.fmt(f),
            _ => self.to_expression().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for JSXEmptyExpression {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for JSXAttributeItem<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::Attribute(attr) => attr.fmt(f),
            Self::SpreadAttribute(spread_attr) => spread_attr.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for JSXAttribute<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.name)?;
        if let Some(value) = &self.value {
            write!(f, ["=", value])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for JSXSpreadAttribute<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["{...", self.argument, "}"])
    }
}

impl<'a> FormatWrite<'a> for JSXAttributeName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::Identifier(ident) => ident.fmt(f),
            Self::NamespacedName(namespaced_name) => namespaced_name.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for JSXAttributeValue<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::Fragment(fragment) => fragment.fmt(f),
            Self::Element(el) => el.fmt(f),
            Self::StringLiteral(lit) => {
                let quote = if lit.value.contains('"') { "'" } else { "\"" };
                write!(f, [quote, dynamic_text(lit.value.as_str(), lit.span.start), quote])
            }
            Self::ExpressionContainer(expr_container) => expr_container.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for JSXIdentifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name.as_str(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for JSXChild<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::Fragment(fragment) => fragment.fmt(f),
            Self::Element(el) => el.fmt(f),
            Self::Spread(spread) => spread.fmt(f),
            Self::ExpressionContainer(expr_container) => expr_container.fmt(f),
            Self::Text(text) => text.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for JSXSpreadChild<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["{...", self.expression, "}"])
    }
}

impl<'a> FormatWrite<'a> for JSXText<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.value.as_str(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for TSThisParameter<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "this")?;
        if let Some(type_annotation) = &self.type_annotation {
            type_annotation.fmt(f);
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSEnumDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { id, r#const, .. } = self;
        if *r#const {
            write!(f, ["const", space()])?;
        }
        write!(f, ["enum", space(), id, space(), "{", self.body, "}"])
    }
}

impl<'a> FormatWrite<'a> for TSEnumBody<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.members.is_empty() {
            write!(f, group(&format_args!(format_dangling_comments(self.span), soft_line_break())))
        } else {
            write!(f, block_indent(&self.members))
        }
    }
}

impl<'a> Format<'a> for Vec<'a, TSEnumMember<'a>> {
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

impl<'a> FormatWrite<'a> for TSEnumMember<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSEnumMemberName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSTypeAnnotation<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [":", space(), self.type_annotation])
    }
}

impl<'a> FormatWrite<'a> for TSLiteralType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.literal.fmt(f)
    }
}

impl<'a> FormatWrite<'a> for TSLiteral<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::BooleanLiteral(l) => l.fmt(f),
            Self::NumericLiteral(l) => l.fmt(f),
            Self::BigIntLiteral(l) => l.fmt(f),
            Self::StringLiteral(l) => l.fmt(f),
            Self::TemplateLiteral(l) => l.fmt(f),
            Self::UnaryExpression(e) => e.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TSType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::TSFunctionType(ty) => ty.fmt(f),
            Self::TSConstructorType(ty) => ty.fmt(f),
            Self::TSArrayType(ty) => ty.fmt(f),
            Self::TSTupleType(ty) => ty.fmt(f),
            Self::TSUnionType(ty) => ty.fmt(f),
            Self::TSParenthesizedType(ty) => ty.fmt(f),
            Self::TSIntersectionType(ty) => ty.fmt(f),
            Self::TSConditionalType(ty) => ty.fmt(f),
            Self::TSInferType(ty) => ty.fmt(f),
            Self::TSIndexedAccessType(ty) => ty.fmt(f),
            Self::TSMappedType(ty) => ty.fmt(f),
            Self::TSNamedTupleMember(ty) => ty.fmt(f),
            Self::TSLiteralType(ty) => ty.fmt(f),
            Self::TSImportType(ty) => ty.fmt(f),
            Self::TSAnyKeyword(ty) => ty.fmt(f),
            Self::TSBigIntKeyword(ty) => ty.fmt(f),
            Self::TSBooleanKeyword(ty) => ty.fmt(f),
            Self::TSIntrinsicKeyword(ty) => ty.fmt(f),
            Self::TSNeverKeyword(ty) => ty.fmt(f),
            Self::TSNullKeyword(ty) => ty.fmt(f),
            Self::TSNumberKeyword(ty) => ty.fmt(f),
            Self::TSObjectKeyword(ty) => ty.fmt(f),
            Self::TSStringKeyword(ty) => ty.fmt(f),
            Self::TSSymbolKeyword(ty) => ty.fmt(f),
            Self::TSThisType(ty) => ty.fmt(f),
            Self::TSUndefinedKeyword(ty) => ty.fmt(f),
            Self::TSUnknownKeyword(ty) => ty.fmt(f),
            Self::TSVoidKeyword(ty) => ty.fmt(f),
            Self::TSTemplateLiteralType(ty) => ty.fmt(f),
            Self::TSTypeLiteral(ty) => ty.fmt(f),
            Self::TSTypeOperatorType(ty) => ty.fmt(f),
            Self::TSTypePredicate(ty) => ty.fmt(f),
            Self::TSTypeQuery(ty) => ty.fmt(f),
            Self::TSTypeReference(ty) => ty.fmt(f),
            Self::JSDocNullableType(ty) => ty.fmt(f),
            Self::JSDocNonNullableType(ty) => ty.fmt(f),
            Self::JSDocUnknownType(ty) => ty.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TSConditionalType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                self.check_type,
                " extends ",
                self.extends_type,
                " ? ",
                self.true_type,
                " : ",
                self.false_type
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for TSUnionType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Some((first, rest)) = self.types.split_first() else {
            return Ok(());
        };
        write!(f, first);
        for item in rest {
            write!(f, [" | ", item])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSIntersectionType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Some((first, rest)) = self.types.split_first() else {
            return Ok(());
        };
        write!(f, first);
        for item in rest {
            write!(f, [" & ", item])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSParenthesizedType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["(", self.type_annotation, ")"])
    }
}

impl<'a> FormatWrite<'a> for TSTypeOperator<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.operator.to_str(), hard_space(), self.type_annotation])
    }
}

impl<'a> FormatWrite<'a> for TSArrayType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.element_type, "[]"])
    }
}

impl<'a> FormatWrite<'a> for TSIndexedAccessType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object_type, "[", self.index_type, "]"])
    }
}

impl<'a> FormatWrite<'a> for TSTupleType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "[")?;
        for (i, ty) in self.element_types.iter().enumerate() {
            if i != 0 {
                write!(f, [",", space()])?;
            }
            write!(f, ty)?;
        }
        write!(f, "]")
    }
}

impl<'a> FormatWrite<'a> for TSNamedTupleMember<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.label)?;
        if self.optional {
            write!(f, "?")?;
        }
        write!(f, [":", space(), self.element_type])
    }
}

impl<'a> FormatWrite<'a> for TSOptionalType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.type_annotation, "?"])
    }
}

impl<'a> FormatWrite<'a> for TSRestType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["...", self.type_annotation])
    }
}

impl<'a> FormatWrite<'a> for TSTupleElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::TSOptionalType(ty) => ty.fmt(f),
            Self::TSRestType(ty) => ty.fmt(f),
            match_ts_type!(TSTupleElement) => self.to_ts_type().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TSAnyKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "any")
    }
}

impl<'a> FormatWrite<'a> for TSStringKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "string")
    }
}

impl<'a> FormatWrite<'a> for TSBooleanKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "boolean")
    }
}

impl<'a> FormatWrite<'a> for TSNumberKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "number")
    }
}

impl<'a> FormatWrite<'a> for TSNeverKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "never")
    }
}

impl<'a> FormatWrite<'a> for TSIntrinsicKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "intrinsic")
    }
}

impl<'a> FormatWrite<'a> for TSUnknownKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "unknown")
    }
}

impl<'a> FormatWrite<'a> for TSNullKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "null")
    }
}

impl<'a> FormatWrite<'a> for TSUndefinedKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "undefined")
    }
}

impl<'a> FormatWrite<'a> for TSVoidKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "void")
    }
}

impl<'a> FormatWrite<'a> for TSSymbolKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "symbol")
    }
}

impl<'a> FormatWrite<'a> for TSThisType {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "this")
    }
}

impl<'a> FormatWrite<'a> for TSObjectKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "object")
    }
}

impl<'a> FormatWrite<'a> for TSBigIntKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "bigint")
    }
}

impl<'a> FormatWrite<'a> for TSTypeReference<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.type_name, self.type_arguments])
    }
}

impl<'a> FormatWrite<'a> for TSTypeName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::IdentifierReference(ident) => ident.fmt(f),
            Self::QualifiedName(name) => name.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TSQualifiedName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.left, ".", self.right])
    }
}

impl<'a> FormatWrite<'a> for TSTypeParameterInstantiation<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "<")?;
        for (i, param) in self.params.iter().enumerate() {
            if i != 0 {
                write!(f, [",", space()])?;
            }
            write!(f, param)?;
        }
        write!(f, ">")
    }
}

impl<'a> FormatWrite<'a> for TSTypeParameter<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.r#const {
            write!(f, ["const", space()])?;
        }
        if self.r#in {
            write!(f, ["in", space()])?;
        }
        if self.out {
            write!(f, ["out", space()])?;
        }
        write!(f, self.name)?;
        if let Some(constraint) = &self.constraint {
            write!(f, [space(), "extends", space(), constraint])?;
        }
        if let Some(default) = &self.default {
            write!(f, [space(), "=", space(), default])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSTypeParameterDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["<", self.params, ">"])
    }
}

impl<'a> FormatWrite<'a> for TSTypeAliasDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let assignment_like = format_with(|f| {
            write!(f, [self.id, self.type_parameters, space(), "=", space(), self.type_annotation])
        });
        write!(
            f,
            [
                self.declare.then_some("declare "),
                "type",
                space(),
                group(&assignment_like),
                OptionalSemicolon
            ]
        )
    }
}

impl<'a> Format<'a> for &Vec<'a, TSClassImplements<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSClassImplements<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["implements", self.expression, self.type_arguments])
    }
}

impl<'a> FormatWrite<'a> for TSInterfaceDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { id, extends, type_parameters, body, .. } = self;

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

            if let Some(type_parameters) = &type_parameters {
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
            if self.declare {
                write!(f, ["declare", space()])?;
            }

            write!(f, ["interface", space()])?;

            let id_has_trailing_comments = f.comments().has_trailing_comments(id.span.end);
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

            if body.body.is_empty() {
                write!(f, format_dangling_comments(body.span()).with_block_indent())?;
            } else {
                write!(f, block_indent(body))?;
            }

            write!(f, "}")
        });

        write!(f, group(&content))
    }
}

impl<'a> FormatWrite<'a> for TSInterfaceBody<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // let last_index = self.body.len().saturating_sub(1);
        let source_text = f.context().source_text();
        let mut joiner = f.join_nodes_with_soft_line();
        for (index, sig) in self.body.iter().enumerate() {
            joiner.entry(sig.span(), source_text, sig);
        }
        joiner.finish()
    }
}

impl<'a> FormatWrite<'a> for TSPropertySignature<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.readonly {
            write!(f, "readonly")?;
        }
        if self.computed {
            write!(f, [space(), "[", self.key, "]"])?;
        } else {
            match &self.key {
                PropertyKey::StaticIdentifier(key) => {
                    write!(f, self.key)?;
                }
                PropertyKey::PrivateIdentifier(key) => {
                    write!(f, self.key)?;
                }
                PropertyKey::StringLiteral(key) => {
                    write!(f, self.key)?;
                }
                key => {
                    write!(f, key)?;
                }
            }
        }
        if self.optional {
            write!(f, "?")?;
        }
        if let Some(type_annotation) = &self.type_annotation {
            write!(f, type_annotation)?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSSignature<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::TSIndexSignature(o) => o.fmt(f),
            Self::TSPropertySignature(o) => o.fmt(f),
            Self::TSCallSignatureDeclaration(o) => o.fmt(f),
            Self::TSConstructSignatureDeclaration(o) => o.fmt(f),
            Self::TSMethodSignature(o) => o.fmt(f),
        }
    }
}

impl<'a> Format<'a> for Vec<'a, TSSignature<'a>> {
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

impl<'a> FormatWrite<'a> for TSIndexSignature<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.readonly {
            write!(f, ["readonly", space()])?;
        }
        // TODO: parameters only have one element for now.
        write!(
            f,
            ["[", self.parameters.first().unwrap(), "]", self.type_annotation, OptionalSemicolon]
        )
    }
}

impl<'a> FormatWrite<'a> for TSCallSignatureDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if let Some(type_parameters) = &self.type_parameters {
            write!(f, group(&type_parameters))?;
        }

        if let Some(this_param) = &self.this_param {
            write!(f, [this_param, ",", soft_line_break_or_space()])?;
        }
        write!(f, group(&self.params))?;
        if let Some(return_type) = &self.return_type {
            write!(f, return_type)?;
        }
        write!(f, OptionalSemicolon)
    }
}

impl<'a> FormatWrite<'a> for TSMethodSignature<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self.kind {
            TSMethodSignatureKind::Method => {}
            TSMethodSignatureKind::Get => {
                write!(f, ["get", space()])?;
            }
            TSMethodSignatureKind::Set => {
                write!(f, ["set", space()])?;
            }
        }
        if self.computed {
            write!(f, "[")?;
        }
        write!(f, self.key)?;
        if self.computed {
            write!(f, "]")?;
        }
        if self.optional {
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
        if let Some(type_parameters) = &self.type_parameters {
            write!(f, [group(&type_parameters)])?;
        }
        write!(f, group(&self.params))?;
        if let Some(return_type) = &self.return_type {
            write!(f, return_type)?;
        }
        write!(f, OptionalSemicolon)
    }
}

impl<'a> FormatWrite<'a> for TSConstructSignatureDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["new", space()])?;
        if let Some(type_parameters) = &self.type_parameters {
            write!(f, group(&type_parameters))?;
        }
        write!(f, group(&self.params))?;
        if let Some(return_type) = &self.return_type {
            write!(f, return_type)?;
        }
        write!(f, OptionalSemicolon)
    }
}

impl<'a> FormatWrite<'a> for TSIndexSignatureName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [dynamic_text(self.name.as_str(), self.span.start), self.type_annotation])
    }
}

impl<'a> Format<'a> for Vec<'a, TSInterfaceHeritage<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.join_with(&soft_line_break_or_space())
            .entries(
                FormatSeparatedIter::new(self.iter(), ",")
                    .with_trailing_separator(TrailingSeparator::Disallowed),
            )
            .finish()
    }
}

impl<'a> FormatWrite<'a> for TSInterfaceHeritage<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression, self.type_arguments])
    }
}

impl<'a> FormatWrite<'a> for TSTypePredicate<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSTypePredicateName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSModuleDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.declare {
            write!(f, ["declare", space()])?;
        }
        write!(f, self.kind.as_str())?;

        if !self.kind.is_global() {
            write!(f, [space(), self.id])?;
        }

        if let Some(body) = &self.body {
            let mut body = body;
            loop {
                match body {
                    TSModuleDeclarationBody::TSModuleDeclaration(b) => {
                        write!(f, [".", b.id])?;
                        if let Some(b) = &b.body {
                            body = b;
                        } else {
                            break;
                        }
                    }
                    TSModuleDeclarationBody::TSModuleBlock(body) => {
                        write!(f, [space(), body]);
                        break;
                    }
                }
            }
        } else {
            write!(f, OptionalSemicolon)?;
        }

        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSModuleDeclarationName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::Identifier(ident) => ident.fmt(f),
            Self::StringLiteral(s) => s.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TSModuleDeclarationBody<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSModuleBlock<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { body, directives, span, .. } = self;
        write!(f, "{")?;
        if body.is_empty() && directives.is_empty() {
            write!(f, [format_dangling_comments(*span).with_block_indent()])?;
        } else {
            write!(f, [block_indent(&format_args!(directives, body))])?;
        }
        write!(f, "}")
    }
}

impl<'a> FormatWrite<'a> for TSTypeLiteral<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        ObjectLike::TSTypeLiteral(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for TSInferType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["infer ", self.type_parameter])
    }
}

impl<'a> FormatWrite<'a> for TSTypeQuery<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["typeof ", self.expr_name, self.type_arguments])
    }
}

impl<'a> FormatWrite<'a> for TSTypeQueryExprName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            match_ts_type_name!(Self) => self.to_ts_type_name().fmt(f),
            Self::TSImportType(decl) => decl.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TSImportType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["import(", self.argument])?;
        if let Some(options) = &self.options {
            write!(f, [",", space(), options])?;
        }
        write!(f, ")")?;
        if let Some(qualified_name) = &self.qualifier {
            write!(f, [".", qualified_name])?;
        }
        write!(f, self.type_arguments)
    }
}

impl<'a> FormatWrite<'a> for TSFunctionType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { type_parameters, params, return_type, .. } = self;

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

            write!(f, [space(), "=>", space(), return_type.type_annotation])
        });

        write!(f, group(&format_inner))
    }
}

impl<'a> FormatWrite<'a> for TSConstructorType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { r#abstract, type_parameters, params, return_type, .. } = self;
        if *r#abstract {
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
                return_type.type_annotation
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for TSMappedType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Self { type_parameter, name_type, .. } = self;

        let should_expand = false; // TODO has_line_break_before_property_name(node)?;

        let comments = f.comments().clone();
        let type_annotation_has_leading_comment = false;
        //TODO
        //
        // mapped_type
        // .as_ref()
        // .is_some_and(|annotation| comments.has_leading_comments(annotation.syntax()));

        let format_inner = format_with(|f| {
            write!(f, FormatLeadingComments::Comments(comments.dangling_comments(self.span)))?;

            match self.readonly {
                Some(TSMappedTypeModifierOperator::True) => write!(f, ["readonly", space()])?,
                Some(TSMappedTypeModifierOperator::Plus) => write!(f, ["+readonly", space()])?,
                Some(TSMappedTypeModifierOperator::Minus) => write!(f, ["-readonly", space()])?,
                None => {}
            }

            let format_inner_inner = format_with(|f| {
                write!(f, "[")?;
                write!(f, type_parameter.name)?;
                if let Some(constraint) = &type_parameter.constraint {
                    write!(f, [space(), "in", space(), constraint])?;
                }
                if let Some(default) = &type_parameter.default {
                    write!(f, [space(), "=", space(), default])?;
                }
                if let Some(name_type) = &name_type {
                    write!(f, [space(), "as", space(), name_type])?;
                }
                write!(f, "]")?;
                match self.optional {
                    Some(TSMappedTypeModifierOperator::True) => write!(f, "?"),
                    Some(TSMappedTypeModifierOperator::Plus) => write!(f, "+?"),
                    Some(TSMappedTypeModifierOperator::Minus) => write!(f, "-?"),
                    None => Ok(()),
                }
            });

            write!(f, [space(), group(&format_inner_inner)])?;
            if let Some(type_annotation) = &self.type_annotation {
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

impl<'a> FormatWrite<'a> for TSTemplateLiteralType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "`")?;
        for (index, item) in self.quasis.iter().enumerate() {
            if index != 0 {
                if let Some(types) = self.types.get(index - 1) {
                    write!(f, ["${", types, "}"])?;
                }
            }
            write!(f, dynamic_text(item.value.raw.as_str(), item.span.start));
        }
        write!(f, "`")
    }
}

impl<'a> FormatWrite<'a> for TSAsExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression, " as ", self.type_annotation])
    }
}

impl<'a> FormatWrite<'a> for TSSatisfiesExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression, " satisfies ", self.type_annotation])
    }
}

impl<'a> FormatWrite<'a> for TSTypeAssertion<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "<")?;
        // var r = < <T>(x: T) => T > ((x) => { return null; });
        //          ^ make sure space is printed here.
        if matches!(self.type_annotation, TSType::TSFunctionType(_)) {
            write!(f, space())?;
        }
        write!(f, [self.type_annotation, ">", self.expression])
    }
}

impl<'a> FormatWrite<'a> for TSImportEqualsDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                "import",
                space(),
                self.import_kind,
                self.id,
                space(),
                "=",
                space(),
                self.module_reference,
                OptionalSemicolon
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for TSModuleReference<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::ExternalModuleReference(decl) => decl.fmt(f),
            match_ts_type_name!(Self) => self.to_ts_type_name().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TSExternalModuleReference<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["require(", self.expression, ")"])
    }
}

impl<'a> FormatWrite<'a> for TSNonNullExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression, "!"])
    }
}

impl<'a> Format<'a> for Vec<'a, Decorator<'a>> {
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

impl<'a> FormatWrite<'a> for Decorator<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["@", self.expression])
    }
}

impl<'a> FormatWrite<'a> for TSExportAssignment<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["export = ", self.expression])
    }
}

impl<'a> FormatWrite<'a> for TSNamespaceExportDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["export as namespace ", self.id])
    }
}

impl<'a> FormatWrite<'a> for TSInstantiationExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression, self.type_arguments])
    }
}

impl<'a> FormatWrite<'a> for JSDocNullableType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.postfix {
            write!(f, [self.type_annotation, "?"])
        } else {
            write!(f, ["?", self.type_annotation])
        }
    }
}

impl<'a> FormatWrite<'a> for JSDocNonNullableType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.postfix {
            write!(f, [self.type_annotation, "!"])
        } else {
            write!(f, ["!", self.type_annotation])
        }
    }
}

impl<'a> FormatWrite<'a> for JSDocUnknownType {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}
