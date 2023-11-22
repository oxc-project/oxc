//! Direct port of needs-parens for adding or removing parentheses.
//!
//! See <https://github.com/prettier/prettier/blob/main/src/language-js/needs-parens.js>

#![allow(
    clippy::unused_self,
    clippy::match_same_arms,
    clippy::match_like_matches_macro,
    clippy::single_match
)]
use oxc_ast::{
    ast::{
        AssignmentTarget, AssignmentTargetPattern, ChainElement, ExportDefaultDeclarationKind,
        Expression, ModuleDeclaration, SimpleAssignmentTarget,
    },
    AstKind,
};
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator, UpdateOperator};

use crate::{array, doc::Doc, ss, Prettier};

impl<'a> Prettier<'a> {
    pub(crate) fn wrap_parens(&self, doc: Doc<'a>, kind: AstKind<'a>) -> Doc<'a> {
        if self.need_parens(kind) {
            array![self, ss!("("), doc, ss!(")")]
        } else {
            doc
        }
    }

    fn need_parens(&self, kind: AstKind<'a>) -> bool {
        if matches!(kind, AstKind::Program(_)) {
            return false;
        }

        if kind.is_statement() || kind.is_declaration() {
            return false;
        }

        let parent_kind = self.parent_kind();

        if self.check_parent_kind(kind, parent_kind) {
            return true;
        }

        match kind {
            AstKind::NumberLiteral(literal) => {
                matches!(parent_kind, AstKind::MemberExpression(e) if e.object().span() == literal.span)
            }
            AstKind::SequenceExpression(_) => !matches!(parent_kind, AstKind::Program(_)),
            AstKind::ObjectExpression(e) => self.check_object_function_class(e.span),
            AstKind::Function(f) if f.is_expression() => {
                if self.check_object_function_class(f.span) {
                    return true;
                }
                match parent_kind {
                    AstKind::CallExpression(call_expr) => call_expr.callee.span() == f.span,
                    AstKind::NewExpression(new_expr) => new_expr.callee.span() == f.span,
                    AstKind::TaggedTemplateExpression(_) => true,
                    _ => false,
                }
            }
            AstKind::Class(c) if c.is_expression() => self.check_object_function_class(c.span),
            AstKind::AssignmentExpression(assign_expr) => match parent_kind {
                AstKind::ArrowExpression(arrow_expr)
                    if arrow_expr.expression
                        && arrow_expr.body.statements[0].span() == assign_expr.span =>
                {
                    true
                }
                AstKind::AssignmentExpression(_) => false,
                AstKind::ExpressionStatement(_) => matches!(
                    assign_expr.left,
                    AssignmentTarget::AssignmentTargetPattern(
                        AssignmentTargetPattern::ObjectAssignmentTarget(_)
                    )
                ),
                _ => false,
            },
            AstKind::UpdateExpression(update_expr) => match parent_kind {
                AstKind::UnaryExpression(unary_expr) => {
                    update_expr.prefix
                        && ((update_expr.operator == UpdateOperator::Increment
                            && unary_expr.operator == UnaryOperator::UnaryPlus)
                            || (update_expr.operator == UpdateOperator::Decrement
                                && unary_expr.operator == UnaryOperator::UnaryNegation))
                }
                _ => self.check_update_unary(update_expr.span),
            },
            AstKind::UnaryExpression(unary_expr) => match parent_kind {
                AstKind::UnaryExpression(parent_expr) => {
                    let u_op = unary_expr.operator;
                    u_op == parent_expr.operator
                        && (matches!(u_op, UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation))
                }
                _ => self.check_update_unary(unary_expr.span),
            },
            AstKind::YieldExpression(e) => match parent_kind {
                AstKind::AwaitExpression(_) => true,
                _ => self.check_yield_await(e.span),
            },
            AstKind::AwaitExpression(e) => self.check_yield_await(e.span),
            AstKind::TSTypeAssertion(e) => self.check_binarish(e.span),
            AstKind::TSAsExpression(e) => self.check_binarish(e.span),
            AstKind::TSSatisfiesExpression(e) => self.check_binarish(e.span),
            AstKind::LogicalExpression(e) => self.check_binarish(e.span),
            AstKind::BinaryExpression(e) => match parent_kind {
                AstKind::UpdateExpression(_) => true,
                _ => self.check_binarish(e.span),
            },
            AstKind::MemberExpression(e) => self.check_member_call(e.span()),
            AstKind::CallExpression(e) => self.check_member_call(e.span),
            AstKind::TaggedTemplateExpression(e) => {
                self.check_member_call_tagged_template_ts_non_null(e.span)
            }
            AstKind::TSNonNullExpression(e) => {
                self.check_member_call_tagged_template_ts_non_null(e.span)
            }
            AstKind::Function(e) if e.is_expression() => match parent_kind {
                AstKind::CallExpression(call_expr) => call_expr.callee.span() == e.span,
                AstKind::NewExpression(new_expr) => new_expr.callee.span() == e.span,
                AstKind::TaggedTemplateExpression(_) => true,
                _ => false,
            },
            AstKind::ArrowExpression(e) => match parent_kind {
                AstKind::CallExpression(call_expr) => call_expr.callee.span() == e.span,
                AstKind::NewExpression(new_expr) => new_expr.callee.span() == e.span,
                AstKind::MemberExpression(member_expr) => member_expr.object().span() == e.span,
                AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TaggedTemplateExpression(_)
                | AstKind::UnaryExpression(_)
                | AstKind::LogicalExpression(_)
                | AstKind::AwaitExpression(_)
                | AstKind::TSTypeAssertion(_) => true,
                AstKind::ConditionalExpression(cond_expr) => cond_expr.test.span() == e.span,
                _ => false,
            },
            AstKind::Class(class) if class.is_expression() => match parent_kind {
                AstKind::NewExpression(new_expr) => new_expr.callee.span() == class.span,
                _ => false,
            },
            _ => false,
        }
    }

    fn check_parent_kind(&self, kind: AstKind<'a>, parent_kind: AstKind<'a>) -> bool {
        match parent_kind {
            AstKind::Class(class) => {
                if let Some(h) = &class.super_class {
                    match kind {
                        AstKind::ArrowExpression(e) if e.span == h.span() => return true,
                        AstKind::AssignmentExpression(e) if e.span == h.span() => return true,
                        AstKind::AwaitExpression(e) if e.span == h.span() => return true,
                        AstKind::BinaryExpression(e) if e.span == h.span() => return true,
                        AstKind::ConditionalExpression(e) if e.span == h.span() => return true,
                        AstKind::LogicalExpression(e) if e.span == h.span() => return true,
                        AstKind::NewExpression(e) if e.span == h.span() => return true,
                        AstKind::ObjectExpression(e) if e.span == h.span() => return true,
                        AstKind::SequenceExpression(e) if e.span == h.span() => return true,
                        AstKind::TaggedTemplateExpression(e) if e.span == h.span() => return true,
                        AstKind::UnaryExpression(e) if e.span == h.span() => return true,
                        AstKind::UpdateExpression(e) if e.span == h.span() => return true,
                        AstKind::YieldExpression(e) if e.span == h.span() => return true,
                        AstKind::TSNonNullExpression(e) if e.span == h.span() => return true,
                        AstKind::Class(e)
                            if e.is_expression()
                                && !e.decorators.is_empty()
                                && e.span == h.span() =>
                        {
                            return true
                        }
                        _ => {}
                    }
                }
            }
            AstKind::ModuleDeclaration(ModuleDeclaration::ExportDefaultDeclaration(decl)) => {
                if let ExportDefaultDeclarationKind::Expression(e) = &decl.declaration {
                    return matches!(e, Expression::SequenceExpression(_))
                        || Self::should_wrap_function_for_export_default(e);
                }
            }
            _ => {}
        }
        false
    }

    fn check_object_function_class(&self, span: Span) -> bool {
        for ast_kind in self.nodes.iter().rev() {
            if let AstKind::ExpressionStatement(e) = ast_kind {
                if Self::starts_with_no_lookahead_token(&e.expression, span) {
                    return true;
                }
            }
        }
        false
    }

    fn check_update_unary(&self, span: Span) -> bool {
        match self.parent_kind() {
            AstKind::MemberExpression(member_expr) => member_expr.object().span() == span,
            AstKind::TaggedTemplateExpression(_) => true,
            AstKind::CallExpression(call_expr) => call_expr.callee.span() == span,
            AstKind::NewExpression(new_expr) => new_expr.callee.span() == span,
            AstKind::BinaryExpression(bin_expr) => {
                bin_expr.left.span() == span && bin_expr.operator == BinaryOperator::Exponential
            }
            AstKind::TSNonNullExpression(_) => true,
            _ => false,
        }
    }

    fn check_yield_await(&self, span: Span) -> bool {
        match self.parent_kind() {
            AstKind::TaggedTemplateExpression(_)
            | AstKind::UnaryExpression(_)
            | AstKind::LogicalExpression(_)
            | AstKind::SpreadElement(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::BinaryExpression(_) => true,
            AstKind::MemberExpression(member_expr) => member_expr.object().span() == span,
            AstKind::NewExpression(new_expr) => new_expr.callee.span() == span,
            AstKind::CallExpression(new_expr) => new_expr.callee.span() == span,
            AstKind::ConditionalExpression(con_expr) => con_expr.test.span() == span,
            _ => false,
        }
    }

    fn check_binarish(&self, span: Span) -> bool {
        match self.parent_kind() {
            AstKind::TSAsExpression(_) => !self.is_binary_cast_expression(span),
            AstKind::TSSatisfiesExpression(_) => !self.is_binary_cast_expression(span),
            AstKind::ConditionalExpression(_) => self.is_binary_cast_expression(span),
            AstKind::NewExpression(new_expr) => new_expr.callee.span() == span,
            AstKind::CallExpression(new_expr) => new_expr.callee.span() == span,
            AstKind::Class(class) => class.super_class.as_ref().is_some_and(|e| e.span() == span),
            AstKind::TSTypeAssertion(_)
            | AstKind::TaggedTemplateExpression(_)
            | AstKind::UnaryExpression(_)
            | AstKind::JSXSpreadAttribute(_)
            | AstKind::SpreadElement(_)
            | AstKind::AwaitExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::UpdateExpression(_) => true,
            AstKind::MemberExpression(member_expr) => member_expr.object().span() == span,
            AstKind::AssignmentExpression(assign_expr) => {
                assign_expr.left.span() == span && self.is_binary_cast_expression(span)
            }
            AstKind::AssignmentPattern(assign_pat) => {
                assign_pat.left.span() == span && self.is_binary_cast_expression(span)
            }
            _ => false,
        }
    }

    fn check_member_call(&self, span: Span) -> bool {
        // if (shouldAddParenthesesToChainElement(path)) {
        // return true;
        // }
        self.check_member_call_tagged_template_ts_non_null(span)
    }

    fn check_member_call_tagged_template_ts_non_null(&self, span: Span) -> bool {
        match self.parent_kind() {
            AstKind::NewExpression(new_expr) if new_expr.callee.span() == span => true,
            _ => false,
        }
    }

    // This differs from the prettier implementation, which may be wrong.
    fn should_wrap_function_for_export_default(e: &Expression<'a>) -> bool {
        match e {
            Expression::FunctionExpression(_) | Expression::ClassExpression(_) => true,
            Expression::CallExpression(e) => {
                Self::should_wrap_function_for_export_default(&e.callee)
            }
            Expression::MemberExpression(e) => {
                Self::should_wrap_function_for_export_default(e.object())
            }
            Expression::AssignmentExpression(e) => match &e.left {
                AssignmentTarget::SimpleAssignmentTarget(t) => match t {
                    SimpleAssignmentTarget::AssignmentTargetIdentifier(_) => false,
                    SimpleAssignmentTarget::MemberAssignmentTarget(e) => {
                        Self::should_wrap_function_for_export_default(e.object())
                    }
                    SimpleAssignmentTarget::TSAsExpression(e) => {
                        Self::should_wrap_function_for_export_default(&e.expression)
                    }
                    SimpleAssignmentTarget::TSSatisfiesExpression(e) => {
                        Self::should_wrap_function_for_export_default(&e.expression)
                    }
                    SimpleAssignmentTarget::TSNonNullExpression(e) => {
                        Self::should_wrap_function_for_export_default(&e.expression)
                    }
                    SimpleAssignmentTarget::TSTypeAssertion(e) => {
                        Self::should_wrap_function_for_export_default(&e.expression)
                    }
                },
                AssignmentTarget::AssignmentTargetPattern(_) => false,
            },
            _ => false,
        }
    }

    fn is_binary_cast_expression(&self, _span: Span) -> bool {
        false
    }

    fn starts_with_no_lookahead_token(e: &Expression<'a>, span: Span) -> bool {
        match e {
            Expression::BinaryExpression(e) => Self::starts_with_no_lookahead_token(&e.left, span),
            Expression::LogicalExpression(e) => Self::starts_with_no_lookahead_token(&e.left, span),
            Expression::AssignmentExpression(e) => match &e.left {
                AssignmentTarget::SimpleAssignmentTarget(t) => match t {
                    SimpleAssignmentTarget::AssignmentTargetIdentifier(_) => false,
                    SimpleAssignmentTarget::MemberAssignmentTarget(e) => {
                        Self::starts_with_no_lookahead_token(e.object(), span)
                    }
                    SimpleAssignmentTarget::TSAsExpression(e) => {
                        Self::starts_with_no_lookahead_token(&e.expression, span)
                    }
                    SimpleAssignmentTarget::TSSatisfiesExpression(e) => {
                        Self::starts_with_no_lookahead_token(&e.expression, span)
                    }
                    SimpleAssignmentTarget::TSNonNullExpression(e) => {
                        Self::starts_with_no_lookahead_token(&e.expression, span)
                    }
                    SimpleAssignmentTarget::TSTypeAssertion(e) => {
                        Self::starts_with_no_lookahead_token(&e.expression, span)
                    }
                },
                AssignmentTarget::AssignmentTargetPattern(_) => false,
            },
            Expression::MemberExpression(e) => {
                Self::starts_with_no_lookahead_token(e.object(), span)
            }
            Expression::TaggedTemplateExpression(e) => {
                if matches!(e.tag, Expression::FunctionExpression(_)) {
                    return false;
                }
                Self::starts_with_no_lookahead_token(&e.tag, span)
            }
            Expression::CallExpression(e) => {
                if matches!(e.callee, Expression::FunctionExpression(_)) {
                    return false;
                }
                Self::starts_with_no_lookahead_token(&e.callee, span)
            }
            Expression::ConditionalExpression(e) => {
                Self::starts_with_no_lookahead_token(&e.test, span)
            }
            Expression::UpdateExpression(e) => {
                !e.prefix
                    && match &e.argument {
                        SimpleAssignmentTarget::AssignmentTargetIdentifier(_) => false,
                        SimpleAssignmentTarget::MemberAssignmentTarget(e) => {
                            Self::starts_with_no_lookahead_token(e.object(), span)
                        }
                        SimpleAssignmentTarget::TSAsExpression(e) => {
                            Self::starts_with_no_lookahead_token(&e.expression, span)
                        }
                        SimpleAssignmentTarget::TSSatisfiesExpression(e) => {
                            Self::starts_with_no_lookahead_token(&e.expression, span)
                        }
                        SimpleAssignmentTarget::TSNonNullExpression(e) => {
                            Self::starts_with_no_lookahead_token(&e.expression, span)
                        }
                        SimpleAssignmentTarget::TSTypeAssertion(e) => {
                            Self::starts_with_no_lookahead_token(&e.expression, span)
                        }
                    }
            }
            Expression::SequenceExpression(e) => e
                .expressions
                .get(0)
                .map_or(false, |e| Self::starts_with_no_lookahead_token(e, span)),
            Expression::ChainExpression(e) => match &e.expression {
                ChainElement::CallExpression(e) => {
                    Self::starts_with_no_lookahead_token(&e.callee, span)
                }
                ChainElement::MemberExpression(e) => {
                    Self::starts_with_no_lookahead_token(e.object(), span)
                }
            },
            Expression::TSSatisfiesExpression(e) => {
                Self::starts_with_no_lookahead_token(&e.expression, span)
            }
            Expression::TSAsExpression(e) => {
                Self::starts_with_no_lookahead_token(&e.expression, span)
            }
            Expression::TSNonNullExpression(e) => {
                Self::starts_with_no_lookahead_token(&e.expression, span)
            }
            _ => e.span() == span,
        }
    }
}
