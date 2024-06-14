use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, ArrowFunctionExpression, Expression, Function,
    ObjectExpression, ObjectPropertyKind, TSLiteral, TSMethodSignatureKind, TSTupleElement, TSType,
    TSTypeOperatorOperator,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::SPAN;

use crate::TransformerDts;

impl<'a> TransformerDts<'a> {
    pub fn transform_function_to_ts_type(&self, func: &Function<'a>) -> Option<TSType<'a>> {
        let return_type = self.infer_function_return_type(func);
        let params = self.transform_formal_parameters(&func.params);

        return_type.map(|return_type| {
            self.ctx.ast.ts_function_type(
                func.span,
                self.ctx.ast.copy(&func.this_param),
                params,
                return_type,
                self.ctx.ast.copy(&func.type_parameters),
            )
        })
    }

    pub fn transform_arrow_function_to_ts_type(
        &self,
        func: &ArrowFunctionExpression<'a>,
    ) -> Option<TSType<'a>> {
        let return_type = self.infer_arrow_function_return_type(func);
        let params = self.transform_formal_parameters(&func.params);

        return_type.map(|return_type| {
            self.ctx.ast.ts_function_type(
                func.span,
                None,
                params,
                return_type,
                self.ctx.ast.copy(&func.type_parameters),
            )
        })
    }

    /// Transform object expression to TypeScript type
    /// ```ts
    /// export const obj = {
    ///  doThing<K extends string>(_k: K): Foo<K> {
    ///    return {} as any;
    ///  },
    /// };
    /// // to
    /// export declare const obj: {
    ///   doThing<K extends string>(_k: K): Foo<K>;
    /// };
    /// ```
    pub fn transform_object_expression_to_ts_type(
        &self,
        expr: &ObjectExpression<'a>,
        is_const: bool,
    ) -> TSType<'a> {
        let members =
        self.ctx.ast.new_vec_from_iter(expr.properties.iter().filter_map(|property| match property {
            ObjectPropertyKind::ObjectProperty(object) => {
                if object.computed {
                    self.ctx.error(
                        OxcDiagnostic::error("Computed property names on class or object literals cannot be inferred with --isolatedDeclarations.")
                            .with_label(object.span)
                    );
                    return None;
                }

                if let Expression::FunctionExpression(function) = &object.value {
                    if !is_const && object.method {
                        let return_type = self.infer_function_return_type(function);
                        let params = self.transform_formal_parameters(&function.params);
                        return Some(self.ctx.ast.ts_method_signature(
                            object.span,
                            self.ctx.ast.copy(&object.key),
                            object.computed,
                            false,
                            TSMethodSignatureKind::Method,
                            self.ctx.ast.copy(&function.this_param),
                            params,
                            return_type,
                            self.ctx.ast.copy(&function.type_parameters),
                        ));
                    }
                }

                let type_annotation = self.infer_type_from_expression(&object.value);

                let property_signature = self.ctx.ast.ts_property_signature(
                    object.span,
                    false,
                    false,
                    is_const,
                    self.ctx.ast.copy(&object.key),
                    type_annotation
                        .map(|type_annotation| self.ctx.ast.ts_type_annotation(SPAN, type_annotation)),
                );
                Some(property_signature)
            },
            ObjectPropertyKind::SpreadProperty(spread) => {
                self.ctx.error(OxcDiagnostic::error(
                    "Objects that contain spread assignments can't be inferred with --isolatedDeclarations.",
                ).with_label(spread.span));
                None
            }
        }));
        self.ctx.ast.ts_type_literal(SPAN, members)
    }

    pub fn transform_array_expression_to_ts_type(
        &self,
        expr: &ArrayExpression<'a>,
        is_const: bool,
    ) -> TSType<'a> {
        let element_types =
            self.ctx.ast.new_vec_from_iter(expr.elements.iter().filter_map(|element| {
                 match element {
                    ArrayExpressionElement::SpreadElement(spread) => {
                        self.ctx.error(OxcDiagnostic::error(
                            "Arrays with spread elements can't inferred with --isolatedDeclarations.",
                        ).with_label(spread.span));
                        None
                    },
                    ArrayExpressionElement::Elision(elision) => {
                        Some(TSTupleElement::from(self.ctx.ast.ts_undefined_keyword(elision.span)))
                    },
                    _ => {
                         Some(TSTupleElement::from(self.transform_expression_to_ts_type(element.to_expression())))
                    }
                }
            }));

        let ts_type = self.ctx.ast.ts_tuple_type(SPAN, element_types);
        if is_const {
            self.ctx.ast.ts_type_operator_type(SPAN, TSTypeOperatorOperator::Readonly, ts_type)
        } else {
            ts_type
        }
    }

    // https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3-4.html#const-assertions
    pub fn transform_expression_to_ts_type(&self, expr: &Expression<'a>) -> TSType<'a> {
        match expr {
            Expression::BooleanLiteral(lit) => self
                .ctx
                .ast
                .ts_literal_type(SPAN, TSLiteral::BooleanLiteral(self.ctx.ast.copy(lit))),
            Expression::NumericLiteral(lit) => self
                .ctx
                .ast
                .ts_literal_type(SPAN, TSLiteral::NumericLiteral(self.ctx.ast.copy(lit))),
            Expression::BigintLiteral(lit) => {
                self.ctx.ast.ts_literal_type(SPAN, TSLiteral::BigintLiteral(self.ctx.ast.copy(lit)))
            }
            Expression::StringLiteral(lit) => {
                self.ctx.ast.ts_literal_type(SPAN, TSLiteral::StringLiteral(self.ctx.ast.copy(lit)))
            }
            Expression::TemplateLiteral(lit) => self
                .ctx
                .ast
                .ts_literal_type(SPAN, TSLiteral::TemplateLiteral(self.ctx.ast.copy(lit))),
            Expression::UnaryExpression(expr) => self
                .ctx
                .ast
                .ts_literal_type(SPAN, TSLiteral::UnaryExpression(self.ctx.ast.copy(expr))),
            Expression::ArrayExpression(expr) => {
                self.transform_array_expression_to_ts_type(expr, true)
            }
            Expression::ObjectExpression(expr) => {
                // { readonly a: number }
                self.transform_object_expression_to_ts_type(expr, true)
            }
            _ => {
                unreachable!()
            }
        }
    }
}
