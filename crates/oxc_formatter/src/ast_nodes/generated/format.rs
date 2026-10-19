// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/formatter/format.rs`.

#![expect(clippy::match_same_arms)]
use oxc_ast::ast::*;
use oxc_formatter_core::Format;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{
        JsFormatContext, JsFormatter, JsFormatterExt as _,
        trivia::{
            format_current_trailing_comments, format_leading_comments, format_trailing_comments,
        },
    },
    parentheses::NeedsParentheses,
    print::{FormatWrite, semicolon::write_suppressed_statement},
    utils::{
        suppressed::{FormatSuppressedNode, write_suppressed_expression},
        typecast::{
            TypeCast, classify_type_cast, format_leading_comments_and_open_paren,
            format_outer_leading_comments_and_open_paren, format_type_cast_comment_node,
        },
    },
};

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, Program<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::Program(*self), |f| {
            self.write(f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, Expression<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            Expression::BooleanLiteral(inner) => {
                AstNode::<BooleanLiteral>::new(inner).fmt(f);
            }
            Expression::NullLiteral(inner) => {
                AstNode::<NullLiteral>::new(inner).fmt(f);
            }
            Expression::NumericLiteral(inner) => {
                AstNode::<NumericLiteral>::new(inner).fmt(f);
            }
            Expression::BigIntLiteral(inner) => {
                AstNode::<BigIntLiteral>::new(inner).fmt(f);
            }
            Expression::RegExpLiteral(inner) => {
                AstNode::<RegExpLiteral>::new(inner).fmt(f);
            }
            Expression::StringLiteral(inner) => {
                AstNode::<StringLiteral>::new(inner).fmt(f);
            }
            Expression::TemplateLiteral(inner) => {
                AstNode::<TemplateLiteral>::new(inner).fmt(f);
            }
            Expression::Identifier(inner) => {
                AstNode::<IdentifierReference>::new(inner).fmt(f);
            }
            Expression::Super(inner) => {
                AstNode::<Super>::new(inner).fmt(f);
            }
            Expression::ArrayExpression(inner) => {
                AstNode::<ArrayExpression>::new(inner).fmt(f);
            }
            Expression::ArrowFunctionExpression(inner) => {
                AstNode::<ArrowFunctionExpression>::new(inner).fmt(f);
            }
            Expression::AssignmentExpression(inner) => {
                AstNode::<AssignmentExpression>::new(inner).fmt(f);
            }
            Expression::AwaitExpression(inner) => {
                AstNode::<AwaitExpression>::new(inner).fmt(f);
            }
            Expression::BinaryExpression(inner) => {
                AstNode::<BinaryExpression>::new(inner).fmt(f);
            }
            Expression::CallExpression(inner) => {
                AstNode::<CallExpression>::new(inner).fmt(f);
            }
            Expression::ChainExpression(inner) => {
                AstNode::<ChainExpression>::new(inner).fmt(f);
            }
            Expression::ClassExpression(inner) => {
                AstNode::<Class>::new(inner).fmt(f);
            }
            Expression::ConditionalExpression(inner) => {
                AstNode::<ConditionalExpression>::new(inner).fmt(f);
            }
            Expression::FunctionExpression(inner) => {
                AstNode::<Function>::new(inner).fmt(f);
            }
            Expression::ImportExpression(inner) => {
                AstNode::<ImportExpression>::new(inner).fmt(f);
            }
            Expression::LogicalExpression(inner) => {
                AstNode::<LogicalExpression>::new(inner).fmt(f);
            }
            Expression::NewExpression(inner) => {
                AstNode::<NewExpression>::new(inner).fmt(f);
            }
            Expression::ObjectExpression(inner) => {
                AstNode::<ObjectExpression>::new(inner).fmt(f);
            }
            Expression::ParenthesizedExpression(inner) => {
                AstNode::<ParenthesizedExpression>::new(inner).fmt(f);
            }
            Expression::SequenceExpression(inner) => {
                AstNode::<SequenceExpression>::new(inner).fmt(f);
            }
            Expression::TaggedTemplateExpression(inner) => {
                AstNode::<TaggedTemplateExpression>::new(inner).fmt(f);
            }
            Expression::ThisExpression(inner) => {
                AstNode::<ThisExpression>::new(inner).fmt(f);
            }
            Expression::UnaryExpression(inner) => {
                AstNode::<UnaryExpression>::new(inner).fmt(f);
            }
            Expression::UpdateExpression(inner) => {
                AstNode::<UpdateExpression>::new(inner).fmt(f);
            }
            Expression::YieldExpression(inner) => {
                AstNode::<YieldExpression>::new(inner).fmt(f);
            }
            Expression::PrivateInExpression(inner) => {
                AstNode::<PrivateInExpression>::new(inner).fmt(f);
            }
            Expression::ImportMeta(inner) => {
                AstNode::<ImportMeta>::new(inner).fmt(f);
            }
            Expression::NewTarget(inner) => {
                AstNode::<NewTarget>::new(inner).fmt(f);
            }
            Expression::JSXElement(inner) => {
                AstNode::<JSXElement>::new(inner).fmt(f);
            }
            Expression::JSXFragment(inner) => {
                AstNode::<JSXFragment>::new(inner).fmt(f);
            }
            Expression::TSAsExpression(inner) => {
                AstNode::<TSAsExpression>::new(inner).fmt(f);
            }
            Expression::TSSatisfiesExpression(inner) => {
                AstNode::<TSSatisfiesExpression>::new(inner).fmt(f);
            }
            Expression::TSTypeAssertion(inner) => {
                AstNode::<TSTypeAssertion>::new(inner).fmt(f);
            }
            Expression::TSNonNullExpression(inner) => {
                AstNode::<TSNonNullExpression>::new(inner).fmt(f);
            }
            Expression::TSInstantiationExpression(inner) => {
                AstNode::<TSInstantiationExpression>::new(inner).fmt(f);
            }
            Expression::V8IntrinsicExpression(inner) => {
                AstNode::<V8IntrinsicExpression>::new(inner).fmt(f);
            }
            it @ match_member_expression!(Expression) => {
                AstNode::<'a, MemberExpression>::new(it.to_member_expression()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, IdentifierName<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::IdentifierName(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, IdentifierReference<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::IdentifierReference(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, BindingIdentifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::BindingIdentifier(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, LabelIdentifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::LabelIdentifier(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ThisExpression> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ThisExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ArrayExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ArrayExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, true, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ArrayExpressionElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            ArrayExpressionElement::SpreadElement(inner) => {
                AstNode::<SpreadElement>::new(inner).fmt(f);
            }
            ArrayExpressionElement::Elision(inner) => {
                AstNode::<Elision>::new(inner).fmt(f);
            }
            it @ match_expression!(ArrayExpressionElement) => {
                AstNode::<'a, Expression>::new(it.to_expression()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, Elision> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::Elision(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ObjectExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ObjectExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, true, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ObjectPropertyKind<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            ObjectPropertyKind::ObjectProperty(inner) => {
                AstNode::<ObjectProperty>::new(inner).fmt(f);
            }
            ObjectPropertyKind::SpreadProperty(inner) => {
                AstNode::<SpreadElement>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ObjectProperty<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ObjectProperty(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, PropertyKey<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            PropertyKey::StaticIdentifier(inner) => {
                AstNode::<IdentifierName>::new(inner).fmt(f);
            }
            PropertyKey::PrivateIdentifier(inner) => {
                AstNode::<PrivateIdentifier>::new(inner).fmt(f);
            }
            it @ match_expression!(PropertyKey) => {
                AstNode::<'a, Expression>::new(it.to_expression()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TemplateLiteral<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TemplateLiteral(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TaggedTemplateExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TaggedTemplateExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TemplateElement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TemplateElement(*self), |f| {
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                format_leading_comments(self.span()).fmt(f);
                FormatSuppressedNode(self.span()).fmt(f);
                format_current_trailing_comments(self.span(), f);
            } else {
                self.write(f);
            }
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, MemberExpression<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            MemberExpression::ComputedMemberExpression(inner) => {
                AstNode::<ComputedMemberExpression>::new(inner).fmt(f);
            }
            MemberExpression::StaticMemberExpression(inner) => {
                AstNode::<StaticMemberExpression>::new(inner).fmt(f);
            }
            MemberExpression::PrivateFieldExpression(inner) => {
                AstNode::<PrivateFieldExpression>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ComputedMemberExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ComputedMemberExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, StaticMemberExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::StaticMemberExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, PrivateFieldExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::PrivateFieldExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, CallExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::CallExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, NewExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::NewExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ImportMeta> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ImportMeta(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, NewTarget> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::NewTarget(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, SpreadElement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::SpreadElement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, Argument<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            Argument::SpreadElement(inner) => {
                AstNode::<SpreadElement>::new(inner).fmt(f);
            }
            it @ match_expression!(Argument) => {
                AstNode::<'a, Expression>::new(it.to_expression()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, UpdateExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::UpdateExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, UnaryExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::UnaryExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, BinaryExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::BinaryExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, PrivateInExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::PrivateInExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, LogicalExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::LogicalExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ConditionalExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ConditionalExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, AssignmentExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::AssignmentExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, AssignmentTarget<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            it @ match_simple_assignment_target!(AssignmentTarget) => {
                AstNode::<'a, SimpleAssignmentTarget>::new(it.to_simple_assignment_target()).fmt(f);
            }
            it @ match_assignment_target_pattern!(AssignmentTarget) => {
                AstNode::<'a, AssignmentTargetPattern>::new(it.to_assignment_target_pattern())
                    .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, SimpleAssignmentTarget<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(inner) => {
                AstNode::<IdentifierReference>::new(inner).fmt(f);
            }
            SimpleAssignmentTarget::TSAsExpression(inner) => {
                AstNode::<TSAsExpression>::new(inner).fmt(f);
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(inner) => {
                AstNode::<TSSatisfiesExpression>::new(inner).fmt(f);
            }
            SimpleAssignmentTarget::TSNonNullExpression(inner) => {
                AstNode::<TSNonNullExpression>::new(inner).fmt(f);
            }
            SimpleAssignmentTarget::TSTypeAssertion(inner) => {
                AstNode::<TSTypeAssertion>::new(inner).fmt(f);
            }
            it @ match_member_expression!(SimpleAssignmentTarget) => {
                AstNode::<'a, MemberExpression>::new(it.to_member_expression()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, AssignmentTargetPattern<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            AssignmentTargetPattern::ArrayAssignmentTarget(inner) => {
                AstNode::<ArrayAssignmentTarget>::new(inner).fmt(f);
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(inner) => {
                AstNode::<ObjectAssignmentTarget>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ArrayAssignmentTarget<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ArrayAssignmentTarget(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ObjectAssignmentTarget<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ObjectAssignmentTarget(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, AssignmentTargetRest<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::AssignmentTargetRest(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, AssignmentTargetMaybeDefault<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(inner) => {
                AstNode::<AssignmentTargetWithDefault>::new(inner).fmt(f);
            }
            it @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                AstNode::<'a, AssignmentTarget>::new(it.to_assignment_target()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, AssignmentTargetWithDefault<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::AssignmentTargetWithDefault(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, AssignmentTargetProperty<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(inner) => {
                AstNode::<AssignmentTargetPropertyIdentifier>::new(inner).fmt(f);
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(inner) => {
                AstNode::<AssignmentTargetPropertyProperty>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, AssignmentTargetPropertyIdentifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::AssignmentTargetPropertyIdentifier(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, AssignmentTargetPropertyProperty<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::AssignmentTargetPropertyProperty(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, SequenceExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::SequenceExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, Super> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::Super(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, AwaitExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::AwaitExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ChainExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ChainExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ChainElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            ChainElement::CallExpression(inner) => {
                AstNode::<CallExpression>::new(inner).fmt(f);
            }
            ChainElement::TSNonNullExpression(inner) => {
                AstNode::<TSNonNullExpression>::new(inner).fmt(f);
            }
            it @ match_member_expression!(ChainElement) => {
                AstNode::<'a, MemberExpression>::new(it.to_member_expression()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ParenthesizedExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ParenthesizedExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, Statement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if write_suppressed_statement(*self, f) {
            format_trailing_comments(
                self.parent(f).span(),
                self.inner.span(),
                self.following_span_start(f),
            )
            .fmt(f);
            return;
        }
        match self.inner {
            Statement::BlockStatement(inner) => {
                AstNode::<BlockStatement>::new(inner).fmt(f);
            }
            Statement::BreakStatement(inner) => {
                AstNode::<BreakStatement>::new(inner).fmt(f);
            }
            Statement::ContinueStatement(inner) => {
                AstNode::<ContinueStatement>::new(inner).fmt(f);
            }
            Statement::DebuggerStatement(inner) => {
                AstNode::<DebuggerStatement>::new(inner).fmt(f);
            }
            Statement::DoWhileStatement(inner) => {
                AstNode::<DoWhileStatement>::new(inner).fmt(f);
            }
            Statement::EmptyStatement(inner) => {
                AstNode::<EmptyStatement>::new(inner).fmt(f);
            }
            Statement::ExpressionStatement(inner) => {
                AstNode::<ExpressionStatement>::new(inner).fmt(f);
            }
            Statement::ForInStatement(inner) => {
                AstNode::<ForInStatement>::new(inner).fmt(f);
            }
            Statement::ForOfStatement(inner) => {
                AstNode::<ForOfStatement>::new(inner).fmt(f);
            }
            Statement::ForStatement(inner) => {
                AstNode::<ForStatement>::new(inner).fmt(f);
            }
            Statement::IfStatement(inner) => {
                AstNode::<IfStatement>::new(inner).fmt(f);
            }
            Statement::LabeledStatement(inner) => {
                AstNode::<LabeledStatement>::new(inner).fmt(f);
            }
            Statement::ReturnStatement(inner) => {
                AstNode::<ReturnStatement>::new(inner).fmt(f);
            }
            Statement::SwitchStatement(inner) => {
                AstNode::<SwitchStatement>::new(inner).fmt(f);
            }
            Statement::ThrowStatement(inner) => {
                AstNode::<ThrowStatement>::new(inner).fmt(f);
            }
            Statement::TryStatement(inner) => {
                AstNode::<TryStatement>::new(inner).fmt(f);
            }
            Statement::WhileStatement(inner) => {
                AstNode::<WhileStatement>::new(inner).fmt(f);
            }
            Statement::WithStatement(inner) => {
                AstNode::<WithStatement>::new(inner).fmt(f);
            }
            it @ match_declaration!(Statement) => {
                AstNode::<'a, Declaration>::new(it.to_declaration()).fmt(f);
            }
            it @ match_module_declaration!(Statement) => {
                AstNode::<'a, ModuleDeclaration>::new(it.to_module_declaration()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, Directive<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::Directive(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, Hashbang<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::Hashbang(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, BlockStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::BlockStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, Declaration<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            Declaration::VariableDeclaration(inner) => {
                AstNode::<VariableDeclaration>::new(inner).fmt(f);
            }
            Declaration::FunctionDeclaration(inner) => {
                AstNode::<Function>::new(inner).fmt(f);
            }
            Declaration::ClassDeclaration(inner) => {
                AstNode::<Class>::new(inner).fmt(f);
            }
            Declaration::TSTypeAliasDeclaration(inner) => {
                AstNode::<TSTypeAliasDeclaration>::new(inner).fmt(f);
            }
            Declaration::TSInterfaceDeclaration(inner) => {
                AstNode::<TSInterfaceDeclaration>::new(inner).fmt(f);
            }
            Declaration::TSEnumDeclaration(inner) => {
                AstNode::<TSEnumDeclaration>::new(inner).fmt(f);
            }
            Declaration::TSExternalModuleDeclaration(inner) => {
                AstNode::<TSExternalModuleDeclaration>::new(inner).fmt(f);
            }
            Declaration::TSNamespaceDeclaration(inner) => {
                AstNode::<TSNamespaceDeclaration>::new(inner).fmt(f);
            }
            Declaration::TSGlobalDeclaration(inner) => {
                AstNode::<TSGlobalDeclaration>::new(inner).fmt(f);
            }
            Declaration::TSImportEqualsDeclaration(inner) => {
                AstNode::<TSImportEqualsDeclaration>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, VariableDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::VariableDeclaration(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, VariableDeclarator<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::VariableDeclarator(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, EmptyStatement> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::EmptyStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ExpressionStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ExpressionStatement(*self), |f| {
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                format_leading_comments(self.span()).fmt(f);
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, IfStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::IfStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, DoWhileStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::DoWhileStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, WhileStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::WhileStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ForStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ForStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ForStatementInit<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            ForStatementInit::VariableDeclaration(inner) => {
                AstNode::<VariableDeclaration>::new(inner).fmt(f);
            }
            it @ match_expression!(ForStatementInit) => {
                AstNode::<'a, Expression>::new(it.to_expression()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ForInStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ForInStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ForStatementLeft<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            ForStatementLeft::VariableDeclaration(inner) => {
                AstNode::<VariableDeclaration>::new(inner).fmt(f);
            }
            it @ match_assignment_target!(ForStatementLeft) => {
                AstNode::<'a, AssignmentTarget>::new(it.to_assignment_target()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ForOfStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ForOfStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ContinueStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ContinueStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, BreakStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::BreakStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ReturnStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ReturnStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, WithStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::WithStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, SwitchStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::SwitchStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, SwitchCase<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::SwitchCase(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, LabeledStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::LabeledStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ThrowStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ThrowStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TryStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TryStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, CatchClause<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::CatchClause(*self), |f| {
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                format_leading_comments(self.span()).fmt(f);
                FormatSuppressedNode(self.span()).fmt(f);
                format_current_trailing_comments(self.span(), f);
            } else {
                self.write(f);
            }
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, CatchParameter<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::CatchParameter(*self), |f| {
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                format_leading_comments(self.span()).fmt(f);
                FormatSuppressedNode(self.span()).fmt(f);
                format_current_trailing_comments(self.span(), f);
            } else {
                self.write(f);
            }
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, DebuggerStatement> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::DebuggerStatement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, BindingPattern<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            BindingPattern::BindingIdentifier(inner) => {
                AstNode::<BindingIdentifier>::new(inner).fmt(f);
            }
            BindingPattern::ObjectPattern(inner) => {
                AstNode::<ObjectPattern>::new(inner).fmt(f);
            }
            BindingPattern::ArrayPattern(inner) => {
                AstNode::<ArrayPattern>::new(inner).fmt(f);
            }
            BindingPattern::AssignmentPattern(inner) => {
                AstNode::<AssignmentPattern>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, AssignmentPattern<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::AssignmentPattern(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ObjectPattern<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ObjectPattern(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, BindingProperty<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::BindingProperty(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ArrayPattern<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ArrayPattern(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, BindingRestElement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::BindingRestElement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, Function<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::Function(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, FormalParameters<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::FormalParameters(*self), |f| {
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                format_leading_comments(self.span()).fmt(f);
                FormatSuppressedNode(self.span()).fmt(f);
                format_current_trailing_comments(self.span(), f);
            } else {
                self.write(f);
            }
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, FormalParameter<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::FormalParameter(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, FormalParameterRest<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::FormalParameterRest(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, FunctionBody<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::FunctionBody(*self), |f| {
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                format_leading_comments(self.span()).fmt(f);
                FormatSuppressedNode(self.span()).fmt(f);
                format_current_trailing_comments(self.span(), f);
            } else {
                self.write(f);
            }
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ArrowFunctionBody<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            ArrowFunctionBody::FunctionBody(inner) => {
                AstNode::<FunctionBody>::new(inner).fmt(f);
            }
            it @ match_expression!(ArrowFunctionBody) => {
                AstNode::<'a, Expression>::new(it.to_expression()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ArrowFunctionExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ArrowFunctionExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, YieldExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::YieldExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, Class<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::Class(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ClassHeritage<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent(f).span(),
            || self.following_span_start(f),
            false,
        ) {
            self.write(f);
            return;
        }
        let is_suppressed = f.comments().is_span_suppressed(self.span());
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ClassBody<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ClassBody(*self), |f| {
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                format_leading_comments(self.span()).fmt(f);
                FormatSuppressedNode(self.span()).fmt(f);
                format_current_trailing_comments(self.span(), f);
            } else {
                self.write(f);
            }
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ClassElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            ClassElement::StaticBlock(inner) => {
                AstNode::<StaticBlock>::new(inner).fmt(f);
            }
            ClassElement::MethodDefinition(inner) => {
                AstNode::<MethodDefinition>::new(inner).fmt(f);
            }
            ClassElement::PropertyDefinition(inner) => {
                AstNode::<PropertyDefinition>::new(inner).fmt(f);
            }
            ClassElement::AccessorProperty(inner) => {
                AstNode::<AccessorProperty>::new(inner).fmt(f);
            }
            ClassElement::TSIndexSignature(inner) => {
                AstNode::<TSIndexSignature>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, MethodDefinition<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::MethodDefinition(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, PropertyDefinition<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::PropertyDefinition(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, PrivateIdentifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::PrivateIdentifier(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, StaticBlock<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::StaticBlock(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ModuleDeclaration<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            ModuleDeclaration::ImportDeclaration(inner) => {
                AstNode::<ImportDeclaration>::new(inner).fmt(f);
            }
            ModuleDeclaration::ExportAllDeclaration(inner) => {
                AstNode::<ExportAllDeclaration>::new(inner).fmt(f);
            }
            ModuleDeclaration::ExportDefaultDeclaration(inner) => {
                AstNode::<ExportDefaultDeclaration>::new(inner).fmt(f);
            }
            ModuleDeclaration::ExportDeclaration(inner) => {
                AstNode::<ExportDeclaration>::new(inner).fmt(f);
            }
            ModuleDeclaration::ExportNamedDeclaration(inner) => {
                AstNode::<ExportNamedDeclaration>::new(inner).fmt(f);
            }
            ModuleDeclaration::ExportFromDeclaration(inner) => {
                AstNode::<ExportFromDeclaration>::new(inner).fmt(f);
            }
            ModuleDeclaration::TSExportAssignment(inner) => {
                AstNode::<TSExportAssignment>::new(inner).fmt(f);
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(inner) => {
                AstNode::<TSNamespaceExportDeclaration>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, AccessorProperty<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::AccessorProperty(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ImportExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ImportExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ImportDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ImportDeclaration(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ImportDeclarationSpecifier<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            ImportDeclarationSpecifier::ImportSpecifier(inner) => {
                AstNode::<ImportSpecifier>::new(inner).fmt(f);
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(inner) => {
                AstNode::<ImportDefaultSpecifier>::new(inner).fmt(f);
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(inner) => {
                AstNode::<ImportNamespaceSpecifier>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ImportSpecifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ImportSpecifier(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ImportDefaultSpecifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ImportDefaultSpecifier(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ImportNamespaceSpecifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ImportNamespaceSpecifier(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, WithClause<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::WithClause(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ImportAttribute<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ImportAttribute(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ImportAttributeKey<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            ImportAttributeKey::Identifier(inner) => {
                AstNode::<IdentifierName>::new(inner).fmt(f);
            }
            ImportAttributeKey::StringLiteral(inner) => {
                AstNode::<StringLiteral>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ExportDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ExportDeclaration(*self), |f| {
            self.write(f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ExportNamedDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ExportNamedDeclaration(*self), |f| {
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                format_leading_comments(self.span()).fmt(f);
                FormatSuppressedNode(self.span()).fmt(f);
                format_current_trailing_comments(self.span(), f);
            } else {
                self.write(f);
            }
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ExportFromDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ExportFromDeclaration(*self), |f| {
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                format_leading_comments(self.span()).fmt(f);
                FormatSuppressedNode(self.span()).fmt(f);
                format_current_trailing_comments(self.span(), f);
            } else {
                self.write(f);
            }
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ExportDefaultDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ExportDefaultDeclaration(*self), |f| {
            self.write(f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ExportAllDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ExportAllDeclaration(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ExportSpecifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::ExportSpecifier(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ExportDefaultDeclarationKind<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            ExportDefaultDeclarationKind::FunctionDeclaration(inner) => {
                AstNode::<Function>::new(inner).fmt(f);
            }
            ExportDefaultDeclarationKind::ClassDeclaration(inner) => {
                AstNode::<Class>::new(inner).fmt(f);
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(inner) => {
                AstNode::<TSInterfaceDeclaration>::new(inner).fmt(f);
            }
            it @ match_expression!(ExportDefaultDeclarationKind) => {
                AstNode::<'a, Expression>::new(it.to_expression()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ModuleExportName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            ModuleExportName::IdentifierName(inner) => {
                AstNode::<IdentifierName>::new(inner).fmt(f);
            }
            ModuleExportName::IdentifierReference(inner) => {
                AstNode::<IdentifierReference>::new(inner).fmt(f);
            }
            ModuleExportName::StringLiteral(inner) => {
                AstNode::<StringLiteral>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, V8IntrinsicExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::V8IntrinsicExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, BooleanLiteral> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::BooleanLiteral(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, NullLiteral> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::NullLiteral(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, NumericLiteral<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::NumericLiteral(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, StringLiteral<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::StringLiteral(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, BigIntLiteral<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::BigIntLiteral(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, RegExpLiteral<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::RegExpLiteral(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXElement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSXElement(*self), |f| {
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            if needs_parentheses {
                "(".fmt(f);
            }
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXOpeningElement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSXOpeningElement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXClosingElement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSXClosingElement(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXFragment<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSXFragment(*self), |f| {
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            if needs_parentheses {
                "(".fmt(f);
            }
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXOpeningFragment> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSXOpeningFragment(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXClosingFragment> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSXClosingFragment(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXElementName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            JSXElementName::Identifier(inner) => {
                AstNode::<JSXIdentifier>::new(inner).fmt(f);
            }
            JSXElementName::IdentifierReference(inner) => {
                AstNode::<IdentifierReference>::new(inner).fmt(f);
            }
            JSXElementName::NamespacedName(inner) => {
                AstNode::<JSXNamespacedName>::new(inner).fmt(f);
            }
            JSXElementName::MemberExpression(inner) => {
                AstNode::<JSXMemberExpression>::new(inner).fmt(f);
            }
            JSXElementName::ThisExpression(inner) => {
                AstNode::<ThisExpression>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXNamespacedName<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSXNamespacedName(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXMemberExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSXMemberExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXMemberExpressionObject<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            JSXMemberExpressionObject::IdentifierReference(inner) => {
                AstNode::<IdentifierReference>::new(inner).fmt(f);
            }
            JSXMemberExpressionObject::MemberExpression(inner) => {
                AstNode::<JSXMemberExpression>::new(inner).fmt(f);
            }
            JSXMemberExpressionObject::ThisExpression(inner) => {
                AstNode::<ThisExpression>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXExpressionContainer<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSXExpressionContainer(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXExpression<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            JSXExpression::EmptyExpression(inner) => {
                AstNode::<JSXEmptyExpression>::new(inner).fmt(f);
            }
            it @ match_expression!(JSXExpression) => {
                AstNode::<'a, Expression>::new(it.to_expression()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXEmptyExpression> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSXEmptyExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXAttributeItem<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            JSXAttributeItem::Attribute(inner) => {
                AstNode::<JSXAttribute>::new(inner).fmt(f);
            }
            JSXAttributeItem::SpreadAttribute(inner) => {
                AstNode::<JSXSpreadAttribute>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXAttribute<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSXAttribute(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXSpreadAttribute<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSXSpreadAttribute(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXAttributeName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            JSXAttributeName::Identifier(inner) => {
                AstNode::<JSXIdentifier>::new(inner).fmt(f);
            }
            JSXAttributeName::NamespacedName(inner) => {
                AstNode::<JSXNamespacedName>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXAttributeValue<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            JSXAttributeValue::StringLiteral(inner) => {
                AstNode::<StringLiteral>::new(inner).fmt(f);
            }
            JSXAttributeValue::ExpressionContainer(inner) => {
                AstNode::<JSXExpressionContainer>::new(inner).fmt(f);
            }
            JSXAttributeValue::Element(inner) => {
                AstNode::<JSXElement>::new(inner).fmt(f);
            }
            JSXAttributeValue::Fragment(inner) => {
                AstNode::<JSXFragment>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXIdentifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSXIdentifier(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXChild<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            JSXChild::Text(inner) => {
                AstNode::<JSXText>::new(inner).fmt(f);
            }
            JSXChild::Element(inner) => {
                AstNode::<JSXElement>::new(inner).fmt(f);
            }
            JSXChild::Fragment(inner) => {
                AstNode::<JSXFragment>::new(inner).fmt(f);
            }
            JSXChild::ExpressionContainer(inner) => {
                AstNode::<JSXExpressionContainer>::new(inner).fmt(f);
            }
            JSXChild::Spread(inner) => {
                AstNode::<JSXSpreadChild>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXSpreadChild<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSXSpreadChild(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSXText<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSXText(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSThisParameter<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSThisParameter(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSEnumDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSEnumDeclaration(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSEnumBody<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSEnumBody(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSEnumMember<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSEnumMember(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSEnumMemberName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            TSEnumMemberName::Identifier(inner) => {
                AstNode::<IdentifierName>::new(inner).fmt(f);
            }
            TSEnumMemberName::String(inner) => {
                AstNode::<StringLiteral>::new(inner).fmt(f);
            }
            TSEnumMemberName::ComputedString(inner) => {
                AstNode::<StringLiteral>::new(inner).fmt(f);
            }
            TSEnumMemberName::ComputedTemplateString(inner) => {
                AstNode::<TemplateLiteral>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTypeAnnotation<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSTypeAnnotation(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSLiteralType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSLiteralType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSLiteral<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            TSLiteral::BooleanLiteral(inner) => {
                AstNode::<BooleanLiteral>::new(inner).fmt(f);
            }
            TSLiteral::NumericLiteral(inner) => {
                AstNode::<NumericLiteral>::new(inner).fmt(f);
            }
            TSLiteral::BigIntLiteral(inner) => {
                AstNode::<BigIntLiteral>::new(inner).fmt(f);
            }
            TSLiteral::StringLiteral(inner) => {
                AstNode::<StringLiteral>::new(inner).fmt(f);
            }
            TSLiteral::TemplateLiteral(inner) => {
                AstNode::<TemplateLiteral>::new(inner).fmt(f);
            }
            TSLiteral::UnaryExpression(inner) => {
                AstNode::<UnaryExpression>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSType<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            TSType::TSAnyKeyword(inner) => {
                AstNode::<TSAnyKeyword>::new(inner).fmt(f);
            }
            TSType::TSBigIntKeyword(inner) => {
                AstNode::<TSBigIntKeyword>::new(inner).fmt(f);
            }
            TSType::TSBooleanKeyword(inner) => {
                AstNode::<TSBooleanKeyword>::new(inner).fmt(f);
            }
            TSType::TSIntrinsicKeyword(inner) => {
                AstNode::<TSIntrinsicKeyword>::new(inner).fmt(f);
            }
            TSType::TSNeverKeyword(inner) => {
                AstNode::<TSNeverKeyword>::new(inner).fmt(f);
            }
            TSType::TSNullKeyword(inner) => {
                AstNode::<TSNullKeyword>::new(inner).fmt(f);
            }
            TSType::TSNumberKeyword(inner) => {
                AstNode::<TSNumberKeyword>::new(inner).fmt(f);
            }
            TSType::TSObjectKeyword(inner) => {
                AstNode::<TSObjectKeyword>::new(inner).fmt(f);
            }
            TSType::TSStringKeyword(inner) => {
                AstNode::<TSStringKeyword>::new(inner).fmt(f);
            }
            TSType::TSSymbolKeyword(inner) => {
                AstNode::<TSSymbolKeyword>::new(inner).fmt(f);
            }
            TSType::TSUndefinedKeyword(inner) => {
                AstNode::<TSUndefinedKeyword>::new(inner).fmt(f);
            }
            TSType::TSUnknownKeyword(inner) => {
                AstNode::<TSUnknownKeyword>::new(inner).fmt(f);
            }
            TSType::TSVoidKeyword(inner) => {
                AstNode::<TSVoidKeyword>::new(inner).fmt(f);
            }
            TSType::TSArrayType(inner) => {
                AstNode::<TSArrayType>::new(inner).fmt(f);
            }
            TSType::TSConditionalType(inner) => {
                AstNode::<TSConditionalType>::new(inner).fmt(f);
            }
            TSType::TSConstructorType(inner) => {
                AstNode::<TSConstructorType>::new(inner).fmt(f);
            }
            TSType::TSFunctionType(inner) => {
                AstNode::<TSFunctionType>::new(inner).fmt(f);
            }
            TSType::TSImportType(inner) => {
                AstNode::<TSImportType>::new(inner).fmt(f);
            }
            TSType::TSIndexedAccessType(inner) => {
                AstNode::<TSIndexedAccessType>::new(inner).fmt(f);
            }
            TSType::TSInferType(inner) => {
                AstNode::<TSInferType>::new(inner).fmt(f);
            }
            TSType::TSIntersectionType(inner) => {
                AstNode::<TSIntersectionType>::new(inner).fmt(f);
            }
            TSType::TSLiteralType(inner) => {
                AstNode::<TSLiteralType>::new(inner).fmt(f);
            }
            TSType::TSMappedType(inner) => {
                AstNode::<TSMappedType>::new(inner).fmt(f);
            }
            TSType::TSNamedTupleMember(inner) => {
                AstNode::<TSNamedTupleMember>::new(inner).fmt(f);
            }
            TSType::TSTemplateLiteralType(inner) => {
                AstNode::<TSTemplateLiteralType>::new(inner).fmt(f);
            }
            TSType::TSThisType(inner) => {
                AstNode::<TSThisType>::new(inner).fmt(f);
            }
            TSType::TSTupleType(inner) => {
                AstNode::<TSTupleType>::new(inner).fmt(f);
            }
            TSType::TSTypeLiteral(inner) => {
                AstNode::<TSTypeLiteral>::new(inner).fmt(f);
            }
            TSType::TSTypeOperatorType(inner) => {
                AstNode::<TSTypeOperator>::new(inner).fmt(f);
            }
            TSType::TSTypePredicate(inner) => {
                AstNode::<TSTypePredicate>::new(inner).fmt(f);
            }
            TSType::TSTypeQuery(inner) => {
                AstNode::<TSTypeQuery>::new(inner).fmt(f);
            }
            TSType::TSTypeReference(inner) => {
                AstNode::<TSTypeReference>::new(inner).fmt(f);
            }
            TSType::TSUnionType(inner) => {
                AstNode::<TSUnionType>::new(inner).fmt(f);
            }
            TSType::TSParenthesizedType(inner) => {
                AstNode::<TSParenthesizedType>::new(inner).fmt(f);
            }
            TSType::JSDocNullableType(inner) => {
                AstNode::<JSDocNullableType>::new(inner).fmt(f);
            }
            TSType::JSDocNonNullableType(inner) => {
                AstNode::<JSDocNonNullableType>::new(inner).fmt(f);
            }
            TSType::JSDocUnknownType(inner) => {
                AstNode::<JSDocUnknownType>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSConditionalType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSConditionalType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSUnionType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSUnionType(*self), |f| {
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_outer_leading_comments_and_open_paren(self.span(), needs_parentheses, f);
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSIntersectionType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSIntersectionType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSParenthesizedType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSParenthesizedType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTypeOperator<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSTypeOperator(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSArrayType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSArrayType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSIndexedAccessType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSIndexedAccessType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTupleType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSTupleType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSNamedTupleMember<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSNamedTupleMember(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSOptionalType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSOptionalType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSRestType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSRestType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTupleElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            TSTupleElement::TSOptionalType(inner) => {
                AstNode::<TSOptionalType>::new(inner).fmt(f);
            }
            TSTupleElement::TSRestType(inner) => {
                AstNode::<TSRestType>::new(inner).fmt(f);
            }
            it @ match_ts_type!(TSTupleElement) => {
                AstNode::<'a, TSType>::new(it.to_ts_type()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSAnyKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSAnyKeyword(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSStringKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSStringKeyword(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSBooleanKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSBooleanKeyword(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSNumberKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSNumberKeyword(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSNeverKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSNeverKeyword(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSIntrinsicKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSIntrinsicKeyword(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSUnknownKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSUnknownKeyword(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSNullKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSNullKeyword(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSUndefinedKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSUndefinedKeyword(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSVoidKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSVoidKeyword(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSSymbolKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSSymbolKeyword(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSThisType> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSThisType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSObjectKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSObjectKeyword(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSBigIntKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSBigIntKeyword(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTypeReference<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSTypeReference(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTypeName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            TSTypeName::IdentifierReference(inner) => {
                AstNode::<IdentifierReference>::new(inner).fmt(f);
            }
            TSTypeName::QualifiedName(inner) => {
                AstNode::<TSQualifiedName>::new(inner).fmt(f);
            }
            TSTypeName::ThisExpression(inner) => {
                AstNode::<ThisExpression>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSQualifiedName<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSQualifiedName(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTypeParameterInstantiation<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSTypeParameterInstantiation(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTypeParameter<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSTypeParameter(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTypeParameterDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSTypeParameterDeclaration(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTypeAliasDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSTypeAliasDeclaration(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSClassImplements<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSClassImplements(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSInterfaceDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSInterfaceDeclaration(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSInterfaceBody<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSInterfaceBody(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSPropertySignature<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSPropertySignature(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSSignature<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            TSSignature::TSIndexSignature(inner) => {
                AstNode::<TSIndexSignature>::new(inner).fmt(f);
            }
            TSSignature::TSPropertySignature(inner) => {
                AstNode::<TSPropertySignature>::new(inner).fmt(f);
            }
            TSSignature::TSCallSignatureDeclaration(inner) => {
                AstNode::<TSCallSignatureDeclaration>::new(inner).fmt(f);
            }
            TSSignature::TSConstructSignatureDeclaration(inner) => {
                AstNode::<TSConstructSignatureDeclaration>::new(inner).fmt(f);
            }
            TSSignature::TSMethodSignature(inner) => {
                AstNode::<TSMethodSignature>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSIndexSignature<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSIndexSignature(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSCallSignatureDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSCallSignatureDeclaration(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSMethodSignature<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSMethodSignature(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSConstructSignatureDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSConstructSignatureDeclaration(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSIndexSignatureName<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSIndexSignatureName(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSInterfaceHeritage<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSInterfaceHeritage(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTypePredicate<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSTypePredicate(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTypePredicateName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            TSTypePredicateName::Identifier(inner) => {
                AstNode::<IdentifierName>::new(inner).fmt(f);
            }
            TSTypePredicateName::This(inner) => {
                AstNode::<TSThisType>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSExternalModuleDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSExternalModuleDeclaration(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSNamespaceDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSNamespaceDeclaration(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSNamespaceDeclarationBody<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            TSNamespaceDeclarationBody::TSNamespaceDeclaration(inner) => {
                AstNode::<TSNamespaceDeclaration>::new(inner).fmt(f);
            }
            TSNamespaceDeclarationBody::TSModuleBlock(inner) => {
                AstNode::<TSModuleBlock>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSGlobalDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSGlobalDeclaration(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSModuleBlock<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSModuleBlock(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTypeLiteral<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSTypeLiteral(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSInferType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSInferType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTypeQuery<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSTypeQuery(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTypeQueryExprName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            TSTypeQueryExprName::TSImportType(inner) => {
                AstNode::<TSImportType>::new(inner).fmt(f);
            }
            it @ match_ts_type_name!(TSTypeQueryExprName) => {
                AstNode::<'a, TSTypeName>::new(it.to_ts_type_name()).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSImportType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSImportType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSImportTypeQualifier<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            TSImportTypeQualifier::Identifier(inner) => {
                AstNode::<IdentifierName>::new(inner).fmt(f);
            }
            TSImportTypeQualifier::QualifiedName(inner) => {
                AstNode::<TSImportTypeQualifiedName>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSImportTypeQualifiedName<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSImportTypeQualifiedName(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSFunctionType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSFunctionType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSConstructorType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSConstructorType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSMappedType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSMappedType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTemplateLiteralType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSTemplateLiteralType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSAsExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSAsExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSSatisfiesExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSSatisfiesExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSTypeAssertion<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSTypeAssertion(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSImportEqualsDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSImportEqualsDeclaration(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSModuleReference<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.inner {
            TSModuleReference::ExternalModuleReference(inner) => {
                AstNode::<TSExternalModuleReference>::new(inner).fmt(f);
            }
            TSModuleReference::IdentifierReference(inner) => {
                AstNode::<IdentifierReference>::new(inner).fmt(f);
            }
            TSModuleReference::QualifiedName(inner) => {
                AstNode::<TSQualifiedName>::new(inner).fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSExternalModuleReference<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSExternalModuleReference(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSNonNullExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSNonNullExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, Decorator<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::Decorator(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSExportAssignment<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSExportAssignment(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSNamespaceExportDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSNamespaceExportDeclaration(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, TSInstantiationExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::TSInstantiationExpression(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                true,
            ) {
                debug_assert!(
                    matches!(classify_type_cast(self.span(), f), TypeCast::None),
                    "a quiet node is in a type cast"
                );
                let needs_parentheses = self.needs_parentheses(f);
                if needs_parentheses {
                    "(".fmt(f);
                }
                self.write(f);
                if needs_parentheses {
                    ")".fmt(f);
                }
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            if is_suppressed {
                write_suppressed_expression(
                    self.span(),
                    self.leading_comments_start(),
                    self.needs_parentheses(f),
                    f,
                );
                format_current_trailing_comments(self.span(), f);
                return;
            }
            if format_type_cast_comment_node(self, false, f) {
                return;
            }
            let needs_parentheses = self.needs_parentheses(f);
            format_leading_comments_and_open_paren(
                self.span(),
                self.leading_comments_start(),
                needs_parentheses,
                f,
            );
            self.write(f);
            if needs_parentheses {
                ")".fmt(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSDocNullableType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSDocNullableType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSDocNonNullableType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSDocNonNullableType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, JSDocUnknownType> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.with_node(AstNodes::JSDocUnknownType(*self), |f| {
            if f.comments().is_quiet(
                self.span(),
                || f.context().ancestors().parent_of_current().span(),
                || f.context().ancestors().following_span_start_of_current(),
                false,
            ) {
                self.write(f);
                return;
            }
            let is_suppressed = f.comments().is_span_suppressed(self.span());
            self.format_leading_comments(f);
            if is_suppressed {
                FormatSuppressedNode(self.span()).fmt(f);
            } else {
                self.write(f);
            }
            format_current_trailing_comments(self.span(), f);
        });
    }
}
