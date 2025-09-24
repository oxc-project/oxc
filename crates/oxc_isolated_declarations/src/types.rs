use oxc_allocator::CloneIn;
use oxc_ast::{
    NONE,
    ast::{
        ArrayExpression, ArrayExpressionElement, ArrowFunctionExpression, Expression, Function,
        ObjectExpression, ObjectPropertyKind, PropertyKey, PropertyKind, TSLiteral,
        TSMethodSignatureKind, TSTupleElement, TSType, TSTypeOperatorOperator,
    },
};
use oxc_span::{ContentEq, GetSpan, SPAN, Span};
use oxc_syntax::identifier::is_identifier_name;

use crate::{
    IsolatedDeclarations,
    diagnostics::{
        arrays_with_spread_elements, function_must_have_explicit_return_type,
        inferred_type_of_expression, method_must_have_explicit_return_type,
        object_with_spread_assignments, shorthand_property,
    },
    function::get_function_span,
};

impl<'a> IsolatedDeclarations<'a> {
    pub(crate) fn transform_function_to_ts_type(&self, func: &Function<'a>) -> Option<TSType<'a>> {
        let return_type = self.infer_function_return_type(func);
        if return_type.is_none() {
            self.error(function_must_have_explicit_return_type(get_function_span(func)));
        }

        let params = self.transform_formal_parameters(&func.params, false);

        return_type.map(|return_type| {
            self.ast.ts_type_function_type(
                func.span,
                func.type_parameters.clone_in(self.ast.allocator),
                func.this_param.clone_in(self.ast.allocator),
                params,
                return_type,
            )
        })
    }

    pub(crate) fn transform_arrow_function_to_ts_type(
        &self,
        func: &ArrowFunctionExpression<'a>,
    ) -> Option<TSType<'a>> {
        let return_type = self.infer_arrow_function_return_type(func);

        if return_type.is_none() {
            self.error(function_must_have_explicit_return_type(Span::new(
                func.params.span.start,
                func.body.span.start + 1,
            )));
        }

        let params = self.transform_formal_parameters(&func.params, false);

        return_type.map(|return_type| {
            self.ast.ts_type_function_type(
                func.span,
                func.type_parameters.clone_in(self.ast.allocator),
                NONE,
                params,
                return_type,
            )
        })
    }

    /// Convert a computed property key to a static property key when possible
    fn transform_property_key(&self, key: &PropertyKey<'a>) -> PropertyKey<'a> {
        match key {
            // ["string"] -> string
            PropertyKey::StringLiteral(literal) if is_identifier_name(&literal.value) => {
                self.ast.property_key_static_identifier(literal.span, literal.value.as_str())
            }
            // [`string`] -> string
            PropertyKey::TemplateLiteral(literal)
                if is_identifier_name(&literal.quasis[0].value.raw) =>
            {
                self.ast.property_key_static_identifier(literal.span, literal.quasis[0].value.raw)
            }
            // [100] -> 100
            // number literal will be cloned as-is
            _ => key.clone_in(self.ast.allocator),
        }
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
        // The span of accessors that cannot infer the type.
        let mut accessor_spans = Vec::new();
        // If either a setter or getter is inferred, the PropertyKey will be added.
        // Use `Vec` rather than `HashSet` because the `PropertyKey` doesn't support
        // `Hash` trait, fortunately, the number of accessors is small.
        let mut accessor_inferred: Vec<&PropertyKey<'a>> = Vec::new();

        let members =
            self.ast.vec_from_iter(expr.properties.iter().filter_map(|property| match property {
                ObjectPropertyKind::ObjectProperty(object) => {
                    if object.computed && self.report_property_key(&object.key) {
                        return None;
                    }

                    if object.shorthand {
                        self.error(shorthand_property(object.span));
                        return None;
                    }

                    let key = &object.key;

                    if !is_const && object.method {
                        let Expression::FunctionExpression(function) = &object.value else {
                            unreachable!(
                                "`object.kind` being `Method` guarantees that it is a function"
                            );
                        };
                        let return_type = self.infer_function_return_type(function);
                        if return_type.is_none() {
                            self.error(method_must_have_explicit_return_type(object.key.span()));
                        }
                        let params = self.transform_formal_parameters(&function.params, false);
                        let key = self.transform_property_key(key);
                        let computed = key
                            .as_expression()
                            .is_some_and(|k| !k.is_string_literal() && !k.is_number_literal());

                        return Some(self.ast.ts_signature_method_signature(
                            object.span,
                            key,
                            computed,
                            false,
                            TSMethodSignatureKind::Method,
                            function.type_parameters.clone_in(self.ast.allocator),
                            function.this_param.clone_in(self.ast.allocator),
                            params,
                            return_type,
                        ));
                    }

                    let type_annotation = match object.kind {
                        PropertyKind::Get => {
                            if accessor_inferred.iter().any(|k| k.content_eq(key)) {
                                return None;
                            }

                            let Expression::FunctionExpression(function) = &object.value else {
                                unreachable!(
                                    "`object.kind` being `Get` guarantees that it is a function"
                                );
                            };

                            let annotation = self.infer_function_return_type(function);
                            if annotation.is_none() {
                                accessor_spans.push((key, key.span()));
                                return None;
                            }

                            accessor_inferred.push(key);
                            annotation
                        }
                        PropertyKind::Set => {
                            if accessor_inferred.iter().any(|k| k.content_eq(key)) {
                                return None;
                            }

                            let Expression::FunctionExpression(function) = &object.value else {
                                unreachable!(
                                    "`object.kind` being `Set` guarantees that it is a function"
                                );
                            };
                            let annotation = function.params.items.first().and_then(|param| {
                                param.pattern.type_annotation.clone_in(self.ast.allocator)
                            });
                            if annotation.is_none() {
                                accessor_spans.push((key, function.params.span));
                                return None;
                            }

                            accessor_inferred.push(key);
                            annotation
                        }
                        PropertyKind::Init => {
                            let type_annotation = if is_const {
                                self.transform_expression_to_ts_type(&object.value)
                            } else {
                                self.infer_type_from_expression(&object.value)
                            };

                            if type_annotation.is_none() {
                                self.error(inferred_type_of_expression(object.value.span()));
                                return None;
                            }

                            type_annotation.map(|type_annotation| {
                                self.ast.alloc_ts_type_annotation(SPAN, type_annotation)
                            })
                        }
                    };

                    let key = self.transform_property_key(key);
                    let property_signature = self.ast.ts_signature_property_signature(
                        object.span,
                        key.as_expression()
                            .is_some_and(|k| !k.is_string_literal() && !k.is_number_literal()),
                        false,
                        is_const,
                        key,
                        type_annotation,
                    );
                    Some(property_signature)
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    self.error(object_with_spread_assignments(spread.span));
                    None
                }
            }));

        // Report an error if the type of neither the setter nor the getter is inferred.
        for (key, span) in accessor_spans {
            if !accessor_inferred.iter().any(|k| k.content_eq(key)) {
                self.error(inferred_type_of_expression(span));
            }
        }

        self.ast.ts_type_type_literal(SPAN, members)
    }

    pub(crate) fn transform_array_expression_to_ts_type(
        &self,
        expr: &ArrayExpression<'a>,
        is_const: bool,
    ) -> TSType<'a> {
        let element_types = self.ast.vec_from_iter(expr.elements.iter().filter_map(|element| {
            match element {
                ArrayExpressionElement::SpreadElement(spread) => {
                    self.error(arrays_with_spread_elements(spread.span));
                    None
                }
                ArrayExpressionElement::Elision(elision) => {
                    Some(TSTupleElement::from(self.ast.ts_type_undefined_keyword(elision.span)))
                }
                _ => self
                    .transform_expression_to_ts_type(element.to_expression())
                    .map(TSTupleElement::from)
                    .or_else(|| {
                        self.error(inferred_type_of_expression(element.span()));
                        None
                    }),
            }
        }));

        let ts_type = self.ast.ts_type_tuple_type(SPAN, element_types);
        if is_const {
            self.ast.ts_type_type_operator_type(SPAN, TSTypeOperatorOperator::Readonly, ts_type)
        } else {
            ts_type
        }
    }

    // https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3-4.html#const-assertions
    pub(crate) fn transform_expression_to_ts_type(
        &self,
        expr: &Expression<'a>,
    ) -> Option<TSType<'a>> {
        match expr {
            Expression::BooleanLiteral(lit) => Some(self.ast.ts_type_literal_type(
                SPAN,
                TSLiteral::BooleanLiteral(lit.clone_in(self.ast.allocator)),
            )),
            Expression::NumericLiteral(lit) => Some(self.ast.ts_type_literal_type(
                SPAN,
                TSLiteral::NumericLiteral(lit.clone_in(self.ast.allocator)),
            )),
            Expression::BigIntLiteral(lit) => Some(self.ast.ts_type_literal_type(
                SPAN,
                TSLiteral::BigIntLiteral(lit.clone_in(self.ast.allocator)),
            )),
            Expression::StringLiteral(lit) => Some(self.ast.ts_type_literal_type(
                SPAN,
                TSLiteral::StringLiteral(lit.clone_in(self.ast.allocator)),
            )),
            Expression::NullLiteral(lit) => Some(self.ast.ts_type_null_keyword(lit.span)),
            Expression::Identifier(ident) => match ident.name.as_str() {
                "undefined" => Some(self.ast.ts_type_undefined_keyword(ident.span)),
                _ => None,
            },
            Expression::TemplateLiteral(lit) => {
                self.transform_template_to_string(lit).map(|string| {
                    self.ast.ts_type_literal_type(lit.span, TSLiteral::StringLiteral(string))
                })
            }
            Expression::UnaryExpression(expr) => {
                if Self::can_infer_unary_expression(expr) {
                    Some(self.ast.ts_type_literal_type(
                        SPAN,
                        TSLiteral::UnaryExpression(expr.clone_in(self.ast.allocator)),
                    ))
                } else {
                    None
                }
            }
            Expression::ArrayExpression(expr) => {
                Some(self.transform_array_expression_to_ts_type(expr, true))
            }
            Expression::ObjectExpression(expr) => {
                Some(self.transform_object_expression_to_ts_type(expr, true))
            }
            Expression::FunctionExpression(func) => self.transform_function_to_ts_type(func),
            Expression::ArrowFunctionExpression(func) => {
                self.transform_arrow_function_to_ts_type(func)
            }
            Expression::TSAsExpression(expr) => {
                if expr.type_annotation.is_const_type_reference() {
                    self.transform_expression_to_ts_type(&expr.expression)
                } else {
                    Some(expr.type_annotation.clone_in(self.ast.allocator))
                }
            }
            _ => None,
        }
    }
}
