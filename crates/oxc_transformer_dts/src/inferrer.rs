use oxc_allocator::Box;
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPatternKind, Expression, FormalParameter, Function, Statement,
    TSType, TSTypeAnnotation,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, SPAN};

use crate::{return_type::FunctionReturnType, TransformerDts};

impl<'a> TransformerDts<'a> {
    pub fn infer_type_from_expression(&self, expr: &Expression<'a>) -> Option<TSType<'a>> {
        match expr {
            Expression::BooleanLiteral(_) => Some(self.ctx.ast.ts_boolean_keyword(SPAN)),
            Expression::NullLiteral(_) => Some(self.ctx.ast.ts_null_keyword(SPAN)),
            Expression::NumericLiteral(_) | Expression::BigintLiteral(_) => {
                Some(self.ctx.ast.ts_number_keyword(SPAN))
            }
            Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => {
                Some(self.ctx.ast.ts_string_keyword(SPAN))
            }
            Expression::Identifier(ident) => match ident.name.as_str() {
                "undefined" => Some(self.ctx.ast.ts_undefined_keyword(SPAN)),
                _ => None,
            },
            Expression::FunctionExpression(func) => {
                self.transform_function_to_ts_type(func).map(|x| self.ctx.ast.copy(&x))
            }
            Expression::ArrowFunctionExpression(func) => {
                self.transform_arrow_function_to_ts_type(func).map(|x| self.ctx.ast.copy(&x))
            }
            Expression::ObjectExpression(expr) => {
                Some(self.transform_object_expression_to_ts_type(expr, false))
            }
            Expression::TSAsExpression(expr) => {
                if expr.type_annotation.is_const_type_reference() {
                    Some(self.transform_expression_to_ts_type(&expr.expression))
                } else {
                    Some(self.ctx.ast.copy(&expr.type_annotation))
                }
            }
            Expression::ClassExpression(expr) => {
                self.ctx.error(
                    OxcDiagnostic::error(
                        "
                        Inference from class expressions is not supported with --isolatedDeclarations.
                    ",
                    )
                    .with_label(expr.span),
                );
                Some(self.ctx.ast.ts_unknown_keyword(SPAN))
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
            Expression::TSTypeAssertion(expr) => Some(self.ctx.ast.copy(&expr.type_annotation)),
            _ => None,
        }
    }

    pub fn infer_type_from_formal_parameter(
        &self,
        param: &FormalParameter<'a>,
    ) -> Option<TSType<'a>> {
        if param.pattern.type_annotation.is_some() {
            param.pattern.type_annotation.as_ref().map(|x| self.ctx.ast.copy(&x.type_annotation));
        }
        if let BindingPatternKind::AssignmentPattern(pattern) = &param.pattern.kind {
            if let Some(annotation) = pattern.left.type_annotation.as_ref() {
                Some(self.ctx.ast.copy(&annotation.type_annotation))
            } else {
                if let Expression::TSAsExpression(expr) = &pattern.right {
                    if !expr.type_annotation.is_keyword_or_literal() {
                        self.ctx.error(
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
            return self.ctx.ast.copy(&function.return_type);
        }

        let return_type = FunctionReturnType::infer(
            self,
            function
                .body
                .as_ref()
                .unwrap_or_else(|| unreachable!("declare function can not have body")),
        )
        .map(|type_annotation| self.ctx.ast.ts_type_annotation(SPAN, type_annotation));

        if return_type.is_none() {
            self.ctx.error(OxcDiagnostic::error(
                "Function must have an explicit return type annotation with --isolatedDeclarations.",
            ).with_label(function.span));

            Some(self.ctx.ast.ts_type_annotation(SPAN, self.ctx.ast.ts_unknown_keyword(SPAN)))
        } else {
            return_type
        }
    }

    pub fn infer_arrow_function_return_type(
        &self,
        function: &ArrowFunctionExpression<'a>,
    ) -> Option<Box<'a, TSTypeAnnotation<'a>>> {
        if function.return_type.is_some() {
            return self.ctx.ast.copy(&function.return_type);
        }

        if function.expression {
            if let Some(Statement::ExpressionStatement(stmt)) = function.body.statements.first() {
                return self
                    .infer_type_from_expression(&stmt.expression)
                    .map(|type_annotation| self.ctx.ast.ts_type_annotation(SPAN, type_annotation));
            }
        }
        FunctionReturnType::infer(self, &function.body)
            .map(|type_annotation| self.ctx.ast.ts_type_annotation(SPAN, type_annotation))
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
