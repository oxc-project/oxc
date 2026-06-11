use oxc_allocator::{Box as ArenaBox, CloneIn};
use oxc_ast::ast::{
    ArrowFunctionExpression, Expression, FormalParameter, Function, Statement, TSType,
    TSTypeAnnotation, UnaryExpression,
};
use oxc_span::{GetSpan, SPAN};

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
            Expression::BooleanLiteral(_) => Some(self.ast.ts_type_boolean_keyword(SPAN)),
            Expression::NullLiteral(_) => Some(self.ast.ts_type_null_keyword(SPAN)),
            Expression::NumericLiteral(_) => Some(self.ast.ts_type_number_keyword(SPAN)),
            Expression::BigIntLiteral(_) => Some(self.ast.ts_type_big_int_keyword(SPAN)),
            Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => {
                Some(self.ast.ts_type_string_keyword(SPAN))
            }
            Expression::Identifier(ident) => match ident.name.as_str() {
                "undefined" => Some(self.ast.ts_type_undefined_keyword(SPAN)),
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
                Some(self.ast.ts_type_unknown_keyword(expr.span))
            }
            Expression::TSAsExpression(expr) => {
                if expr.type_annotation.is_const_type_reference() {
                    self.transform_const_expression_to_ts_type(&expr.expression)
                } else {
                    Some(expr.type_annotation.clone_in(self.ast.allocator))
                }
            }
            Expression::TSTypeAssertion(expr) => {
                if expr.type_annotation.is_const_type_reference() {
                    self.transform_const_expression_to_ts_type(&expr.expression)
                } else {
                    Some(expr.type_annotation.clone_in(self.ast.allocator))
                }
            }
            // `expr satisfies T` has the type of `expr` (tsc accepts these
            // under isolated declarations whenever `expr` itself is inferable).
            Expression::TSSatisfiesExpression(expr) => {
                self.infer_type_from_expression(&expr.expression)
            }
            Expression::ClassExpression(expr) => {
                self.error(inferred_type_of_class_expression(expr.span));
                Some(self.ast.ts_type_unknown_keyword(SPAN))
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
            && let Some(Statement::ExpressionStatement(stmt)) = function.body.statements.first()
        {
            let ty = self.infer_type_from_expression(&stmt.expression).or_else(|| {
                if stmt.expression.is_function() {
                    // The nested function reports its own errors.
                    Some(self.ast.ts_type_unknown_keyword(SPAN))
                } else {
                    // tsc reports TS9013 at the uninferable body expression
                    // rather than TS9007 at the arrow.
                    self.error(crate::diagnostics::inferred_type_of_expression(
                        stmt.expression.span(),
                    ));
                    Some(self.ast.ts_type_unknown_keyword(SPAN))
                }
            });
            return ty
                .map(|type_annotation| self.ast.alloc_ts_type_annotation(SPAN, type_annotation));
        }

        FunctionReturnType::infer(self, &function.body)
            .map(|type_annotation| self.ast.alloc_ts_type_annotation(SPAN, type_annotation))
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
