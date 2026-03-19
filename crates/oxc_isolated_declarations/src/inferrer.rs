use oxc_allocator::{Box as ArenaBox, CloneIn};
use oxc_ast::ast::{
    ArrowFunctionExpression, Expression, ExpressionKind, FormalParameter, Function, TSType,
    TSTypeAnnotation, UnaryExpression,
};
use oxc_span::SPAN;

use crate::{
    IsolatedDeclarations,
    diagnostics::{array_inferred, inferred_type_of_class_expression},
    return_type::FunctionReturnType,
};

impl<'a> IsolatedDeclarations<'a> {
    pub(crate) fn can_infer_unary_expression(expr: &UnaryExpression<'a>) -> bool {
        expr.operator.is_arithmetic() && expr.argument.is_number_literal()
    }

    pub(crate) fn infer_type_from_expression(&self, expr: &Expression<'a>) -> Option<TSType<'a>> {
        match expr.kind() {
            ExpressionKind::BooleanLiteral(_) => Some(self.ast.ts_type_boolean_keyword(SPAN)),
            ExpressionKind::NullLiteral(_) => Some(self.ast.ts_type_null_keyword(SPAN)),
            ExpressionKind::NumericLiteral(_) => Some(self.ast.ts_type_number_keyword(SPAN)),
            ExpressionKind::BigIntLiteral(_) => Some(self.ast.ts_type_big_int_keyword(SPAN)),
            ExpressionKind::StringLiteral(_) | ExpressionKind::TemplateLiteral(_) => {
                Some(self.ast.ts_type_string_keyword(SPAN))
            }
            ExpressionKind::Identifier(ident) => match ident.name.as_str() {
                "undefined" => Some(self.ast.ts_type_undefined_keyword(SPAN)),
                _ => None,
            },
            ExpressionKind::FunctionExpression(func) => self.transform_function_to_ts_type(func),
            ExpressionKind::ArrowFunctionExpression(func) => {
                self.transform_arrow_function_to_ts_type(func)
            }
            ExpressionKind::ObjectExpression(expr) => {
                Some(self.transform_object_expression_to_ts_type(expr, false))
            }
            ExpressionKind::ArrayExpression(expr) => {
                self.error(array_inferred(expr.span));
                Some(self.ast.ts_type_unknown_keyword(expr.span))
            }
            ExpressionKind::TSAsExpression(expr) => {
                if expr.type_annotation.is_const_type_reference() {
                    self.transform_const_expression_to_ts_type(&expr.expression)
                } else {
                    Some(expr.type_annotation.clone_in(self.ast.allocator))
                }
            }
            ExpressionKind::TSTypeAssertion(expr) => {
                if expr.type_annotation.is_const_type_reference() {
                    self.transform_const_expression_to_ts_type(&expr.expression)
                } else {
                    Some(expr.type_annotation.clone_in(self.ast.allocator))
                }
            }
            ExpressionKind::ClassExpression(expr) => {
                self.error(inferred_type_of_class_expression(expr.span));
                Some(self.ast.ts_type_unknown_keyword(SPAN))
            }
            ExpressionKind::ParenthesizedExpression(expr) => {
                self.infer_type_from_expression(&expr.expression)
            }
            ExpressionKind::TSNonNullExpression(expr) => {
                self.infer_type_from_expression(&expr.expression)
            }
            ExpressionKind::TSSatisfiesExpression(expr) => {
                self.infer_type_from_expression(&expr.expression)
            }
            ExpressionKind::UnaryExpression(expr) => {
                if Self::can_infer_unary_expression(expr) {
                    self.infer_type_from_expression(&expr.argument)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(crate) fn infer_type_from_formal_parameter(
        &self,
        param: &FormalParameter<'a>,
    ) -> Option<TSType<'a>> {
        if let Some(init) = &param.initializer {
            self.infer_type_from_expression(init)
        } else {
            None
        }
    }

    pub(crate) fn infer_function_return_type(
        &self,
        function: &Function<'a>,
    ) -> Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        if function.return_type.is_some() {
            return function.return_type.clone_in(self.ast.allocator);
        }

        if function.r#async || function.generator {
            return None;
        }

        function.body.as_ref().and_then(|body| {
            FunctionReturnType::infer(self, body)
                .map(|type_annotation| self.ast.alloc_ts_type_annotation(SPAN, type_annotation))
        })
    }

    pub(crate) fn infer_arrow_function_return_type(
        &self,
        function: &ArrowFunctionExpression<'a>,
    ) -> Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        if function.return_type.is_some() {
            return function.return_type.clone_in(self.ast.allocator);
        }

        if function.r#async {
            return None;
        }

        if function.expression
            && let Some(stmt) =
                function.body.statements.first().and_then(|s| s.as_expression_statement())
        {
            return self
                .infer_type_from_expression(&stmt.expression)
                .map(|type_annotation| self.ast.alloc_ts_type_annotation(SPAN, type_annotation));
        }

        FunctionReturnType::infer(self, &function.body)
            .map(|type_annotation| self.ast.alloc_ts_type_annotation(SPAN, type_annotation))
    }

    pub(crate) fn is_need_to_infer_type_from_expression(expr: &Expression<'a>) -> bool {
        match expr.kind() {
            ExpressionKind::NumericLiteral(_)
            | ExpressionKind::BigIntLiteral(_)
            | ExpressionKind::StringLiteral(_) => false,
            ExpressionKind::TemplateLiteral(lit) => !lit.expressions.is_empty(),
            ExpressionKind::UnaryExpression(expr) => !Self::can_infer_unary_expression(expr),
            _ => true,
        }
    }
}
