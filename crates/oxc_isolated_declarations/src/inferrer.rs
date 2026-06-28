use oxc_allocator::{ArenaBox, CloneIn, GetAllocator};
use oxc_ast::ast::{
    ArrowFunctionExpression, Expression, FormalParameter, Function, Statement, TSType,
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
        match expr {
            Expression::BooleanLiteral(_) => Some(TSType::new_ts_boolean_keyword(SPAN, self)),
            Expression::NullLiteral(_) => Some(TSType::new_ts_null_keyword(SPAN, self)),
            Expression::NumericLiteral(_) => Some(TSType::new_ts_number_keyword(SPAN, self)),
            Expression::BigIntLiteral(_) => Some(TSType::new_ts_big_int_keyword(SPAN, self)),
            Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => {
                Some(TSType::new_ts_string_keyword(SPAN, self))
            }
            Expression::Identifier(ident) => match ident.name.as_str() {
                "undefined" => Some(TSType::new_ts_undefined_keyword(SPAN, self)),
                _ => None,
            },
            Expression::FunctionExpression(func) => self.transform_function_to_ts_type(func),
            Expression::ArrowFunctionExpression(func) => {
                self.transform_arrow_function_to_ts_type(func)
            }
            Expression::ObjectExpression(expr) => {
                Some(self.transform_object_expression_to_ts_type(expr, false))
            }
            Expression::ArrayExpression(expr) => {
                self.error(array_inferred(expr.span));
                Some(TSType::new_ts_unknown_keyword(expr.span, self))
            }
            Expression::TSAsExpression(expr) => {
                if expr.type_annotation.is_const_type_reference() {
                    self.transform_const_expression_to_ts_type(&expr.expression)
                } else {
                    Some(expr.type_annotation.clone_in(self.allocator()))
                }
            }
            Expression::TSTypeAssertion(expr) => {
                if expr.type_annotation.is_const_type_reference() {
                    self.transform_const_expression_to_ts_type(&expr.expression)
                } else {
                    Some(expr.type_annotation.clone_in(self.allocator()))
                }
            }
            Expression::ClassExpression(expr) => {
                self.error(inferred_type_of_class_expression(expr.span));
                Some(TSType::new_ts_unknown_keyword(SPAN, self))
            }
            Expression::ParenthesizedExpression(expr) => {
                self.infer_type_from_expression(&expr.expression)
            }
            Expression::TSNonNullExpression(expr) => {
                self.infer_type_from_expression(&expr.expression)
            }
            Expression::UnaryExpression(expr) => {
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
            return function.return_type.clone_in(self.allocator());
        }

        if function.r#async || function.generator {
            return None;
        }

        function.body.as_ref().and_then(|body| {
            FunctionReturnType::infer(self, body)
                .map(|type_annotation| TSTypeAnnotation::boxed(SPAN, type_annotation, self))
        })
    }

    pub(crate) fn infer_arrow_function_return_type(
        &self,
        function: &ArrowFunctionExpression<'a>,
    ) -> Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        if function.return_type.is_some() {
            return function.return_type.clone_in(self.allocator());
        }

        if function.r#async {
            return None;
        }

        if function.expression
            && let Some(Statement::ExpressionStatement(stmt)) = function.body.statements.first()
        {
            return self
                .infer_type_from_expression(&stmt.expression)
                .map(|type_annotation| TSTypeAnnotation::boxed(SPAN, type_annotation, self));
        }

        FunctionReturnType::infer(self, &function.body)
            .map(|type_annotation| TSTypeAnnotation::boxed(SPAN, type_annotation, self))
    }

    pub(crate) fn is_need_to_infer_type_from_expression(expr: &Expression<'a>) -> bool {
        match expr {
            Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::StringLiteral(_) => false,
            Expression::TemplateLiteral(lit) => !lit.expressions.is_empty(),
            Expression::UnaryExpression(expr) => !Self::can_infer_unary_expression(expr),
            _ => true,
        }
    }
}
