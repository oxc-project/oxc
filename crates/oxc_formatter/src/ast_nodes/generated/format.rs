// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/formatter/format.rs`.

#![expect(clippy::match_same_arms)]
use oxc_ast::ast::*;
use oxc_formatter_core::Format;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::AstNode,
    formatter::{
        JsFormatContext, JsFormatter, JsFormatterExt as _,
        trivia::{format_leading_comments, format_trailing_comments},
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, Program<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        self.write(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, Expression<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            Expression::BooleanLiteral(inner) => {
                AstNode::<BooleanLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::NullLiteral(inner) => {
                AstNode::<NullLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::NumericLiteral(inner) => {
                AstNode::<NumericLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::BigIntLiteral(inner) => {
                AstNode::<BigIntLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::RegExpLiteral(inner) => {
                AstNode::<RegExpLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::StringLiteral(inner) => {
                AstNode::<StringLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TemplateLiteral(inner) => {
                AstNode::<TemplateLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::Identifier(inner) => {
                AstNode::<IdentifierReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::Super(inner) => {
                AstNode::<Super> { inner, parent, following_span_start: self.following_span_start }
                    .fmt(f);
            }
            Expression::ArrayExpression(inner) => {
                AstNode::<ArrayExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ArrowFunctionExpression(inner) => {
                AstNode::<ArrowFunctionExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::AssignmentExpression(inner) => {
                AstNode::<AssignmentExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::AwaitExpression(inner) => {
                AstNode::<AwaitExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::BinaryExpression(inner) => {
                AstNode::<BinaryExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::CallExpression(inner) => {
                AstNode::<CallExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ChainExpression(inner) => {
                AstNode::<ChainExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ClassExpression(inner) => {
                AstNode::<Class> { inner, parent, following_span_start: self.following_span_start }
                    .fmt(f);
            }
            Expression::ConditionalExpression(inner) => {
                AstNode::<ConditionalExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::FunctionExpression(inner) => {
                AstNode::<Function> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ImportExpression(inner) => {
                AstNode::<ImportExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::LogicalExpression(inner) => {
                AstNode::<LogicalExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::NewExpression(inner) => {
                AstNode::<NewExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ObjectExpression(inner) => {
                AstNode::<ObjectExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ParenthesizedExpression(inner) => {
                AstNode::<ParenthesizedExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::SequenceExpression(inner) => {
                AstNode::<SequenceExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TaggedTemplateExpression(inner) => {
                AstNode::<TaggedTemplateExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ThisExpression(inner) => {
                AstNode::<ThisExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::UnaryExpression(inner) => {
                AstNode::<UnaryExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::UpdateExpression(inner) => {
                AstNode::<UpdateExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::YieldExpression(inner) => {
                AstNode::<YieldExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::PrivateInExpression(inner) => {
                AstNode::<PrivateInExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ImportMeta(inner) => {
                AstNode::<ImportMeta> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::NewTarget(inner) => {
                AstNode::<NewTarget> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::JSXElement(inner) => {
                AstNode::<JSXElement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::JSXFragment(inner) => {
                AstNode::<JSXFragment> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TSAsExpression(inner) => {
                AstNode::<TSAsExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TSSatisfiesExpression(inner) => {
                AstNode::<TSSatisfiesExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TSTypeAssertion(inner) => {
                AstNode::<TSTypeAssertion> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TSNonNullExpression(inner) => {
                AstNode::<TSNonNullExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TSInstantiationExpression(inner) => {
                AstNode::<TSInstantiationExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::V8IntrinsicExpression(inner) => {
                AstNode::<V8IntrinsicExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_member_expression!(Expression) => {
                let inner = it.to_member_expression();
                AstNode::<MemberExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'_, 'a, Expression<'a>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, 'a>) -> bool {
        match self.inner {
            Expression::BooleanLiteral(inner) => AstNode::<BooleanLiteral> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::NullLiteral(inner) => AstNode::<NullLiteral> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::NumericLiteral(inner) => AstNode::<NumericLiteral> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::BigIntLiteral(inner) => AstNode::<BigIntLiteral> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::RegExpLiteral(inner) => AstNode::<RegExpLiteral> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::StringLiteral(inner) => AstNode::<StringLiteral> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::TemplateLiteral(inner) => AstNode::<TemplateLiteral> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::Identifier(inner) => AstNode::<IdentifierReference> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::Super(inner) => AstNode::<Super> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::ArrayExpression(inner) => AstNode::<ArrayExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::ArrowFunctionExpression(inner) => AstNode::<ArrowFunctionExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::AssignmentExpression(inner) => AstNode::<AssignmentExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::AwaitExpression(inner) => AstNode::<AwaitExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::BinaryExpression(inner) => AstNode::<BinaryExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::CallExpression(inner) => AstNode::<CallExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::ChainExpression(inner) => AstNode::<ChainExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::ClassExpression(inner) => AstNode::<Class> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::ConditionalExpression(inner) => AstNode::<ConditionalExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::FunctionExpression(inner) => AstNode::<Function> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::ImportExpression(inner) => AstNode::<ImportExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::LogicalExpression(inner) => AstNode::<LogicalExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::NewExpression(inner) => AstNode::<NewExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::ObjectExpression(inner) => AstNode::<ObjectExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::ParenthesizedExpression(inner) => AstNode::<ParenthesizedExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::SequenceExpression(inner) => AstNode::<SequenceExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::TaggedTemplateExpression(inner) => AstNode::<TaggedTemplateExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::ThisExpression(inner) => AstNode::<ThisExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::UnaryExpression(inner) => AstNode::<UnaryExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::UpdateExpression(inner) => AstNode::<UpdateExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::YieldExpression(inner) => AstNode::<YieldExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::PrivateInExpression(inner) => AstNode::<PrivateInExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::ImportMeta(inner) => AstNode::<ImportMeta> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::NewTarget(inner) => AstNode::<NewTarget> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::JSXElement(inner) => AstNode::<JSXElement> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::JSXFragment(inner) => AstNode::<JSXFragment> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::TSAsExpression(inner) => AstNode::<TSAsExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::TSSatisfiesExpression(inner) => AstNode::<TSSatisfiesExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::TSTypeAssertion(inner) => AstNode::<TSTypeAssertion> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::TSNonNullExpression(inner) => AstNode::<TSNonNullExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::TSInstantiationExpression(inner) => AstNode::<TSInstantiationExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::V8IntrinsicExpression(inner) => AstNode::<V8IntrinsicExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::ComputedMemberExpression(inner) => AstNode::<ComputedMemberExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::StaticMemberExpression(inner) => AstNode::<StaticMemberExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
            Expression::PrivateFieldExpression(inner) => AstNode::<PrivateFieldExpression> {
                inner,
                parent: self.parent,
                following_span_start: self.following_span_start,
            }
            .needs_parentheses(f),
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, IdentifierName<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, IdentifierReference<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, BindingIdentifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, LabelIdentifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ThisExpression> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ArrayExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ArrayExpressionElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ArrayExpressionElement::SpreadElement(inner) => {
                AstNode::<SpreadElement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ArrayExpressionElement::Elision(inner) => {
                AstNode::<Elision> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(ArrayExpressionElement) => {
                let inner = it.to_expression();
                AstNode::<Expression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, Elision> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ObjectExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ObjectPropertyKind<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ObjectPropertyKind::ObjectProperty(inner) => {
                AstNode::<ObjectProperty> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ObjectPropertyKind::SpreadProperty(inner) => {
                AstNode::<SpreadElement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ObjectProperty<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, PropertyKey<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            PropertyKey::StaticIdentifier(inner) => {
                AstNode::<IdentifierName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            PropertyKey::PrivateIdentifier(inner) => {
                AstNode::<PrivateIdentifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(PropertyKey) => {
                let inner = it.to_expression();
                AstNode::<Expression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TemplateLiteral<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TaggedTemplateExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TemplateElement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let is_suppressed = f.comments().is_span_suppressed(self.span());
        if is_suppressed {
            format_leading_comments(self.span()).fmt(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, MemberExpression<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            MemberExpression::ComputedMemberExpression(inner) => {
                AstNode::<ComputedMemberExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            MemberExpression::StaticMemberExpression(inner) => {
                AstNode::<StaticMemberExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            MemberExpression::PrivateFieldExpression(inner) => {
                AstNode::<PrivateFieldExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ComputedMemberExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, StaticMemberExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, PrivateFieldExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, CallExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, NewExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ImportMeta> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, NewTarget> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, SpreadElement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, Argument<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            Argument::SpreadElement(inner) => {
                AstNode::<SpreadElement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(Argument) => {
                let inner = it.to_expression();
                AstNode::<Expression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, UpdateExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, UnaryExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, BinaryExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, PrivateInExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, LogicalExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ConditionalExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, AssignmentExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, AssignmentTarget<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            it @ match_simple_assignment_target!(AssignmentTarget) => {
                let inner = it.to_simple_assignment_target();
                AstNode::<SimpleAssignmentTarget> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_assignment_target_pattern!(AssignmentTarget) => {
                let inner = it.to_assignment_target_pattern();
                AstNode::<AssignmentTargetPattern> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, SimpleAssignmentTarget<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(inner) => {
                AstNode::<IdentifierReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            SimpleAssignmentTarget::TSAsExpression(inner) => {
                AstNode::<TSAsExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(inner) => {
                AstNode::<TSSatisfiesExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            SimpleAssignmentTarget::TSNonNullExpression(inner) => {
                AstNode::<TSNonNullExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            SimpleAssignmentTarget::TSTypeAssertion(inner) => {
                AstNode::<TSTypeAssertion> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_member_expression!(SimpleAssignmentTarget) => {
                let inner = it.to_member_expression();
                AstNode::<MemberExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, AssignmentTargetPattern<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            AssignmentTargetPattern::ArrayAssignmentTarget(inner) => {
                AstNode::<ArrayAssignmentTarget> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(inner) => {
                AstNode::<ObjectAssignmentTarget> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ArrayAssignmentTarget<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ObjectAssignmentTarget<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, AssignmentTargetRest<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, AssignmentTargetMaybeDefault<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(inner) => {
                AstNode::<AssignmentTargetWithDefault> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                let inner = it.to_assignment_target();
                AstNode::<AssignmentTarget> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, AssignmentTargetWithDefault<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, AssignmentTargetProperty<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(inner) => {
                AstNode::<AssignmentTargetPropertyIdentifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(inner) => {
                AstNode::<AssignmentTargetPropertyProperty> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>>
    for AstNode<'_, 'a, AssignmentTargetPropertyIdentifier<'a>>
{
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, AssignmentTargetPropertyProperty<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, SequenceExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, Super> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, AwaitExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ChainExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ChainElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ChainElement::CallExpression(inner) => {
                AstNode::<CallExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ChainElement::TSNonNullExpression(inner) => {
                AstNode::<TSNonNullExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_member_expression!(ChainElement) => {
                let inner = it.to_member_expression();
                AstNode::<MemberExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ParenthesizedExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, Statement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if write_suppressed_statement(self, f) {
            format_trailing_comments(
                self.parent.span(),
                self.inner.span(),
                self.following_span_start,
            )
            .fmt(f);
            return;
        }
        let parent = self.parent;
        match self.inner {
            Statement::BlockStatement(inner) => {
                AstNode::<BlockStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::BreakStatement(inner) => {
                AstNode::<BreakStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ContinueStatement(inner) => {
                AstNode::<ContinueStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::DebuggerStatement(inner) => {
                AstNode::<DebuggerStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::DoWhileStatement(inner) => {
                AstNode::<DoWhileStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::EmptyStatement(inner) => {
                AstNode::<EmptyStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ExpressionStatement(inner) => {
                AstNode::<ExpressionStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ForInStatement(inner) => {
                AstNode::<ForInStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ForOfStatement(inner) => {
                AstNode::<ForOfStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ForStatement(inner) => {
                AstNode::<ForStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::IfStatement(inner) => {
                AstNode::<IfStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::LabeledStatement(inner) => {
                AstNode::<LabeledStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ReturnStatement(inner) => {
                AstNode::<ReturnStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::SwitchStatement(inner) => {
                AstNode::<SwitchStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ThrowStatement(inner) => {
                AstNode::<ThrowStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::TryStatement(inner) => {
                AstNode::<TryStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::WhileStatement(inner) => {
                AstNode::<WhileStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::WithStatement(inner) => {
                AstNode::<WithStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_declaration!(Statement) => {
                let inner = it.to_declaration();
                AstNode::<Declaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_module_declaration!(Statement) => {
                let inner = it.to_module_declaration();
                AstNode::<ModuleDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, Directive<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, Hashbang<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, BlockStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, Declaration<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            Declaration::VariableDeclaration(inner) => {
                AstNode::<VariableDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::FunctionDeclaration(inner) => {
                AstNode::<Function> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::ClassDeclaration(inner) => {
                AstNode::<Class> { inner, parent, following_span_start: self.following_span_start }
                    .fmt(f);
            }
            Declaration::TSTypeAliasDeclaration(inner) => {
                AstNode::<TSTypeAliasDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSInterfaceDeclaration(inner) => {
                AstNode::<TSInterfaceDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSEnumDeclaration(inner) => {
                AstNode::<TSEnumDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSExternalModuleDeclaration(inner) => {
                AstNode::<TSExternalModuleDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSNamespaceDeclaration(inner) => {
                AstNode::<TSNamespaceDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSGlobalDeclaration(inner) => {
                AstNode::<TSGlobalDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSImportEqualsDeclaration(inner) => {
                AstNode::<TSImportEqualsDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, VariableDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, VariableDeclarator<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, EmptyStatement> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ExpressionStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let is_suppressed = f.comments().is_span_suppressed(self.span());
        if is_suppressed {
            format_leading_comments(self.span()).fmt(f);
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, IfStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, DoWhileStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, WhileStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ForStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ForStatementInit<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ForStatementInit::VariableDeclaration(inner) => {
                AstNode::<VariableDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(ForStatementInit) => {
                let inner = it.to_expression();
                AstNode::<Expression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ForInStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ForStatementLeft<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ForStatementLeft::VariableDeclaration(inner) => {
                AstNode::<VariableDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_assignment_target!(ForStatementLeft) => {
                let inner = it.to_assignment_target();
                AstNode::<AssignmentTarget> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ForOfStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ContinueStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, BreakStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ReturnStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, WithStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, SwitchStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, SwitchCase<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, LabeledStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ThrowStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TryStatement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, CatchClause<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let is_suppressed = f.comments().is_span_suppressed(self.span());
        if is_suppressed {
            format_leading_comments(self.span()).fmt(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, CatchParameter<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let is_suppressed = f.comments().is_span_suppressed(self.span());
        if is_suppressed {
            format_leading_comments(self.span()).fmt(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, DebuggerStatement> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, BindingPattern<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            BindingPattern::BindingIdentifier(inner) => {
                AstNode::<BindingIdentifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            BindingPattern::ObjectPattern(inner) => {
                AstNode::<ObjectPattern> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            BindingPattern::ArrayPattern(inner) => {
                AstNode::<ArrayPattern> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            BindingPattern::AssignmentPattern(inner) => {
                AstNode::<AssignmentPattern> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, AssignmentPattern<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ObjectPattern<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, BindingProperty<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ArrayPattern<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, BindingRestElement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, Function<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, FormalParameters<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let is_suppressed = f.comments().is_span_suppressed(self.span());
        if is_suppressed {
            format_leading_comments(self.span()).fmt(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, FormalParameter<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, FormalParameterRest<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, FunctionBody<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let is_suppressed = f.comments().is_span_suppressed(self.span());
        if is_suppressed {
            format_leading_comments(self.span()).fmt(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ArrowFunctionBody<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ArrowFunctionBody::FunctionBody(inner) => {
                AstNode::<FunctionBody> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(ArrowFunctionBody) => {
                let inner = it.to_expression();
                AstNode::<Expression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ArrowFunctionExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, YieldExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, Class<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ClassHeritage<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ClassBody<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let is_suppressed = f.comments().is_span_suppressed(self.span());
        if is_suppressed {
            format_leading_comments(self.span()).fmt(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ClassElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ClassElement::StaticBlock(inner) => {
                AstNode::<StaticBlock> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ClassElement::MethodDefinition(inner) => {
                AstNode::<MethodDefinition> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ClassElement::PropertyDefinition(inner) => {
                AstNode::<PropertyDefinition> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ClassElement::AccessorProperty(inner) => {
                AstNode::<AccessorProperty> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ClassElement::TSIndexSignature(inner) => {
                AstNode::<TSIndexSignature> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, MethodDefinition<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, PropertyDefinition<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, PrivateIdentifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, StaticBlock<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ModuleDeclaration<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ModuleDeclaration::ImportDeclaration(inner) => {
                AstNode::<ImportDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::ExportAllDeclaration(inner) => {
                AstNode::<ExportAllDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::ExportDefaultDeclaration(inner) => {
                AstNode::<ExportDefaultDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::ExportDeclaration(inner) => {
                AstNode::<ExportDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::ExportNamedDeclaration(inner) => {
                AstNode::<ExportNamedDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::ExportFromDeclaration(inner) => {
                AstNode::<ExportFromDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::TSExportAssignment(inner) => {
                AstNode::<TSExportAssignment> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(inner) => {
                AstNode::<TSNamespaceExportDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, AccessorProperty<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ImportExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ImportDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ImportDeclarationSpecifier<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ImportDeclarationSpecifier::ImportSpecifier(inner) => {
                AstNode::<ImportSpecifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(inner) => {
                AstNode::<ImportDefaultSpecifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(inner) => {
                AstNode::<ImportNamespaceSpecifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ImportSpecifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ImportDefaultSpecifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ImportNamespaceSpecifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, WithClause<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ImportAttribute<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ImportAttributeKey<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ImportAttributeKey::Identifier(inner) => {
                AstNode::<IdentifierName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ImportAttributeKey::StringLiteral(inner) => {
                AstNode::<StringLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ExportDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        self.write(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ExportNamedDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let is_suppressed = f.comments().is_span_suppressed(self.span());
        if is_suppressed {
            format_leading_comments(self.span()).fmt(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ExportFromDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let is_suppressed = f.comments().is_span_suppressed(self.span());
        if is_suppressed {
            format_leading_comments(self.span()).fmt(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ExportDefaultDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        self.write(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ExportAllDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ExportSpecifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ExportDefaultDeclarationKind<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ExportDefaultDeclarationKind::FunctionDeclaration(inner) => {
                AstNode::<Function> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ExportDefaultDeclarationKind::ClassDeclaration(inner) => {
                AstNode::<Class> { inner, parent, following_span_start: self.following_span_start }
                    .fmt(f);
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(inner) => {
                AstNode::<TSInterfaceDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(ExportDefaultDeclarationKind) => {
                let inner = it.to_expression();
                AstNode::<Expression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, ModuleExportName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ModuleExportName::IdentifierName(inner) => {
                AstNode::<IdentifierName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleExportName::IdentifierReference(inner) => {
                AstNode::<IdentifierReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleExportName::StringLiteral(inner) => {
                AstNode::<StringLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, V8IntrinsicExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, BooleanLiteral> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, NullLiteral> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, NumericLiteral<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, StringLiteral<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, BigIntLiteral<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, RegExpLiteral<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXElement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
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
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXOpeningElement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXClosingElement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXFragment<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
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
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXOpeningFragment> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXClosingFragment> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXElementName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXElementName::Identifier(inner) => {
                AstNode::<JSXIdentifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXElementName::IdentifierReference(inner) => {
                AstNode::<IdentifierReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXElementName::NamespacedName(inner) => {
                AstNode::<JSXNamespacedName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXElementName::MemberExpression(inner) => {
                AstNode::<JSXMemberExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXElementName::ThisExpression(inner) => {
                AstNode::<ThisExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXNamespacedName<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXMemberExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXMemberExpressionObject<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXMemberExpressionObject::IdentifierReference(inner) => {
                AstNode::<IdentifierReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXMemberExpressionObject::MemberExpression(inner) => {
                AstNode::<JSXMemberExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXMemberExpressionObject::ThisExpression(inner) => {
                AstNode::<ThisExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXExpressionContainer<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXExpression<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXExpression::EmptyExpression(inner) => {
                AstNode::<JSXEmptyExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(JSXExpression) => {
                let inner = it.to_expression();
                AstNode::<Expression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXEmptyExpression> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXAttributeItem<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXAttributeItem::Attribute(inner) => {
                AstNode::<JSXAttribute> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXAttributeItem::SpreadAttribute(inner) => {
                AstNode::<JSXSpreadAttribute> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXAttribute<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXSpreadAttribute<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXAttributeName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXAttributeName::Identifier(inner) => {
                AstNode::<JSXIdentifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXAttributeName::NamespacedName(inner) => {
                AstNode::<JSXNamespacedName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXAttributeValue<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXAttributeValue::StringLiteral(inner) => {
                AstNode::<StringLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXAttributeValue::ExpressionContainer(inner) => {
                AstNode::<JSXExpressionContainer> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXAttributeValue::Element(inner) => {
                AstNode::<JSXElement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXAttributeValue::Fragment(inner) => {
                AstNode::<JSXFragment> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXIdentifier<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXChild<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXChild::Text(inner) => {
                AstNode::<JSXText> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXChild::Element(inner) => {
                AstNode::<JSXElement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXChild::Fragment(inner) => {
                AstNode::<JSXFragment> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXChild::ExpressionContainer(inner) => {
                AstNode::<JSXExpressionContainer> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXChild::Spread(inner) => {
                AstNode::<JSXSpreadChild> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXSpreadChild<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSXText<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSThisParameter<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSEnumDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSEnumBody<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSEnumMember<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSEnumMemberName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSEnumMemberName::Identifier(inner) => {
                AstNode::<IdentifierName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSEnumMemberName::String(inner) => {
                AstNode::<StringLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSEnumMemberName::ComputedString(inner) => {
                AstNode::<StringLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSEnumMemberName::ComputedTemplateString(inner) => {
                AstNode::<TemplateLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTypeAnnotation<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSLiteralType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSLiteral<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSLiteral::BooleanLiteral(inner) => {
                AstNode::<BooleanLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSLiteral::NumericLiteral(inner) => {
                AstNode::<NumericLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSLiteral::BigIntLiteral(inner) => {
                AstNode::<BigIntLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSLiteral::StringLiteral(inner) => {
                AstNode::<StringLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSLiteral::TemplateLiteral(inner) => {
                AstNode::<TemplateLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSLiteral::UnaryExpression(inner) => {
                AstNode::<UnaryExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSType<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSType::TSAnyKeyword(inner) => {
                AstNode::<TSAnyKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSBigIntKeyword(inner) => {
                AstNode::<TSBigIntKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSBooleanKeyword(inner) => {
                AstNode::<TSBooleanKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSIntrinsicKeyword(inner) => {
                AstNode::<TSIntrinsicKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSNeverKeyword(inner) => {
                AstNode::<TSNeverKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSNullKeyword(inner) => {
                AstNode::<TSNullKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSNumberKeyword(inner) => {
                AstNode::<TSNumberKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSObjectKeyword(inner) => {
                AstNode::<TSObjectKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSStringKeyword(inner) => {
                AstNode::<TSStringKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSSymbolKeyword(inner) => {
                AstNode::<TSSymbolKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSUndefinedKeyword(inner) => {
                AstNode::<TSUndefinedKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSUnknownKeyword(inner) => {
                AstNode::<TSUnknownKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSVoidKeyword(inner) => {
                AstNode::<TSVoidKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSArrayType(inner) => {
                AstNode::<TSArrayType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSConditionalType(inner) => {
                AstNode::<TSConditionalType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSConstructorType(inner) => {
                AstNode::<TSConstructorType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSFunctionType(inner) => {
                AstNode::<TSFunctionType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSImportType(inner) => {
                AstNode::<TSImportType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSIndexedAccessType(inner) => {
                AstNode::<TSIndexedAccessType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSInferType(inner) => {
                AstNode::<TSInferType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSIntersectionType(inner) => {
                AstNode::<TSIntersectionType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSLiteralType(inner) => {
                AstNode::<TSLiteralType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSMappedType(inner) => {
                AstNode::<TSMappedType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSNamedTupleMember(inner) => {
                AstNode::<TSNamedTupleMember> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTemplateLiteralType(inner) => {
                AstNode::<TSTemplateLiteralType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSThisType(inner) => {
                AstNode::<TSThisType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTupleType(inner) => {
                AstNode::<TSTupleType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTypeLiteral(inner) => {
                AstNode::<TSTypeLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTypeOperatorType(inner) => {
                AstNode::<TSTypeOperator> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTypePredicate(inner) => {
                AstNode::<TSTypePredicate> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTypeQuery(inner) => {
                AstNode::<TSTypeQuery> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTypeReference(inner) => {
                AstNode::<TSTypeReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSUnionType(inner) => {
                AstNode::<TSUnionType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSParenthesizedType(inner) => {
                AstNode::<TSParenthesizedType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::JSDocNullableType(inner) => {
                AstNode::<JSDocNullableType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::JSDocNonNullableType(inner) => {
                AstNode::<JSDocNonNullableType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::JSDocUnknownType(inner) => {
                AstNode::<JSDocUnknownType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSConditionalType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSUnionType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let is_suppressed = f.comments().is_span_suppressed(self.span());
        if is_suppressed {
            write_suppressed_expression(
                self.span(),
                self.leading_comments_start(),
                self.needs_parentheses(f),
                f,
            );
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSIntersectionType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSParenthesizedType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTypeOperator<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSArrayType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSIndexedAccessType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTupleType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSNamedTupleMember<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSOptionalType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSRestType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTupleElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSTupleElement::TSOptionalType(inner) => {
                AstNode::<TSOptionalType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSTupleElement::TSRestType(inner) => {
                AstNode::<TSRestType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_ts_type!(TSTupleElement) => {
                let inner = it.to_ts_type();
                AstNode::<TSType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSAnyKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSStringKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSBooleanKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSNumberKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSNeverKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSIntrinsicKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSUnknownKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSNullKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSUndefinedKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSVoidKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSSymbolKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSThisType> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSObjectKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSBigIntKeyword> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTypeReference<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTypeName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSTypeName::IdentifierReference(inner) => {
                AstNode::<IdentifierReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSTypeName::QualifiedName(inner) => {
                AstNode::<TSQualifiedName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSTypeName::ThisExpression(inner) => {
                AstNode::<ThisExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSQualifiedName<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTypeParameterInstantiation<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTypeParameter<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTypeParameterDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTypeAliasDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSClassImplements<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSInterfaceDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSInterfaceBody<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSPropertySignature<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSSignature<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSSignature::TSIndexSignature(inner) => {
                AstNode::<TSIndexSignature> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSSignature::TSPropertySignature(inner) => {
                AstNode::<TSPropertySignature> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSSignature::TSCallSignatureDeclaration(inner) => {
                AstNode::<TSCallSignatureDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSSignature::TSConstructSignatureDeclaration(inner) => {
                AstNode::<TSConstructSignatureDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSSignature::TSMethodSignature(inner) => {
                AstNode::<TSMethodSignature> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSIndexSignature<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSCallSignatureDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSMethodSignature<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSConstructSignatureDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSIndexSignatureName<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSInterfaceHeritage<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTypePredicate<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTypePredicateName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSTypePredicateName::Identifier(inner) => {
                AstNode::<IdentifierName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSTypePredicateName::This(inner) => {
                AstNode::<TSThisType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSExternalModuleDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSNamespaceDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSNamespaceDeclarationBody<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSNamespaceDeclarationBody::TSNamespaceDeclaration(inner) => {
                AstNode::<TSNamespaceDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSNamespaceDeclarationBody::TSModuleBlock(inner) => {
                AstNode::<TSModuleBlock> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSGlobalDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSModuleBlock<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTypeLiteral<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSInferType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTypeQuery<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTypeQueryExprName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSTypeQueryExprName::TSImportType(inner) => {
                AstNode::<TSImportType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_ts_type_name!(TSTypeQueryExprName) => {
                let inner = it.to_ts_type_name();
                AstNode::<TSTypeName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSImportType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSImportTypeQualifier<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSImportTypeQualifier::Identifier(inner) => {
                AstNode::<IdentifierName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSImportTypeQualifier::QualifiedName(inner) => {
                AstNode::<TSImportTypeQualifiedName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSImportTypeQualifiedName<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSFunctionType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSConstructorType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSMappedType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTemplateLiteralType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSAsExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSSatisfiesExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSTypeAssertion<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSImportEqualsDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSModuleReference<'a>> {
    #[inline]
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSModuleReference::ExternalModuleReference(inner) => {
                AstNode::<TSExternalModuleReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSModuleReference::IdentifierReference(inner) => {
                AstNode::<IdentifierReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSModuleReference::QualifiedName(inner) => {
                AstNode::<TSQualifiedName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSExternalModuleReference<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSNonNullExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, Decorator<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSExportAssignment<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSNamespaceExportDeclaration<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, TSInstantiationExpression<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
            self.format_trailing_comments(f);
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
        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSDocNullableType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSDocNonNullableType<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'_, 'a, JSDocUnknownType> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        if f.comments().is_quiet(
            self.span(),
            || self.parent.span(),
            self.following_span_start,
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
