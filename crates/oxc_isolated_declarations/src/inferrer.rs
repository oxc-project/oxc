use oxc_allocator::Box;
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPatternKind, Expression, FormalParameter, Function, Statement,
    TSType, TSTypeAnnotation,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, SPAN};

use crate::{
    diagnostics::{array_inferred, inferred_type_of_class_expression},
    return_type::FunctionReturnType,
    IsolatedDeclarations,
};

impl<'a> IsolatedDeclarations<'a> {
    pub fn infer_type_from_expression(&self, expr: &Expression<'a>) -> Option<TSType<'a>> {
        match expr {
            Expression::BooleanLiteral(_) => Some(self.ast.ts_boolean_keyword(SPAN)),
            Expression::NullLiteral(_) => Some(self.ast.ts_null_keyword(SPAN)),
            Expression::NumericLiteral(_) | Expression::BigintLiteral(_) => {
                Some(self.ast.ts_number_keyword(SPAN))
            }
            Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => {
                Some(self.ast.ts_string_keyword(SPAN))
            }
            Expression::Identifier(ident) => match ident.name.as_str() {
                "undefined" => Some(self.ast.ts_undefined_keyword(SPAN)),
                _ => None,
            },
            Expression::FunctionExpression(func) => {
                self.transform_function_to_ts_type(func).map(|x| self.ast.copy(&x))
            }
            Expression::ArrowFunctionExpression(func) => {
                self.transform_arrow_function_to_ts_type(func).map(|x| self.ast.copy(&x))
            }
            Expression::ObjectExpression(expr) => {
                Some(self.transform_object_expression_to_ts_type(expr, false))
            }
            Expression::ArrayExpression(expr) => {
                self.error(array_inferred(expr.span));
                Some(self.ast.ts_unknown_keyword(expr.span))
            }
            Expression::TSAsExpression(expr) => {
                if expr.type_annotation.is_const_type_reference() {
                    Some(self.transform_expression_to_ts_type(&expr.expression))
                } else {
                    Some(self.ast.copy(&expr.type_annotation))
                }
            }
            Expression::ClassExpression(expr) => {
                self.error(inferred_type_of_class_expression(expr.span));
                Some(self.ast.ts_unknown_keyword(SPAN))
            }
            Expression::TSNonNullExpression(expr) => {
                self.infer_type_from_expression(&expr.expression)
            }
            Expression::TSSatisfiesExpression(expr) => {
                self.infer_type_from_expression(&expr.expression)
            }
            Expression::TSInstantiationExpression(_expr) => {
                unreachable!();
                // infer_type_from_expression(ctx, &expr.expression)
            }
            Expression::TSTypeAssertion(expr) => Some(self.ast.copy(&expr.type_annotation)),
            _ => None,
        }
    }

    pub fn infer_type_from_formal_parameter(
        &self,
        param: &FormalParameter<'a>,
    ) -> Option<TSType<'a>> {
        if param.pattern.type_annotation.is_some() {
            param.pattern.type_annotation.as_ref().map(|x| self.ast.copy(&x.type_annotation));
        }
        if let BindingPatternKind::AssignmentPattern(pattern) = &param.pattern.kind {
            if let Some(annotation) = pattern.left.type_annotation.as_ref() {
                Some(self.ast.copy(&annotation.type_annotation))
            } else {
                if let Expression::TSAsExpression(expr) = &pattern.right {
                    if !expr.type_annotation.is_keyword_or_literal() {
                        self.error(
                            OxcDiagnostic::error("Parameter must have an explicit type annotation with --isolatedDeclarations.")
                                .with_label(expr.type_annotation.span())
                        );
                    }
                }

                self.infer_type_from_expression(&pattern.right)
            }
        } else {
            None
        }
    }

    pub fn infer_function_return_type(
        &self,
        function: &Function<'a>,
    ) -> Option<Box<'a, TSTypeAnnotation<'a>>> {
        if function.return_type.is_some() {
            return self.ast.copy(&function.return_type);
        }

        if function.r#async || function.generator {
            return None;
        }

        function.body.as_ref().and_then(|body| {
            FunctionReturnType::infer(self, body)
                .map(|type_annotation| self.ast.ts_type_annotation(SPAN, type_annotation))
        })
    }

    pub fn infer_arrow_function_return_type(
        &self,
        function: &ArrowFunctionExpression<'a>,
    ) -> Option<Box<'a, TSTypeAnnotation<'a>>> {
        if function.return_type.is_some() {
            return self.ast.copy(&function.return_type);
        }

        if function.r#async {
            return None;
        }

        if function.r#async {
            return None;
        }

        if function.expression {
            if let Some(Statement::ExpressionStatement(stmt)) = function.body.statements.first() {
                return self
                    .infer_type_from_expression(&stmt.expression)
                    .map(|type_annotation| self.ast.ts_type_annotation(SPAN, type_annotation));
            }
        }

        FunctionReturnType::infer(self, &function.body)
            .map(|type_annotation| self.ast.ts_type_annotation(SPAN, type_annotation))
    }

    pub fn is_need_to_infer_type_from_expression(expr: &Expression) -> bool {
        !matches!(
            expr,
            Expression::NumericLiteral(_)
                | Expression::BigintLiteral(_)
                | Expression::StringLiteral(_)
                | Expression::TemplateLiteral(_)
        )
    }
}
