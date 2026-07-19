use oxc_allocator::{ArenaBox, ArenaVec, CloneIn, GetAllocator};
use oxc_ast::{
    ast::{
        ArrayExpression, ArrayExpressionElement, ArrowFunctionExpression, Expression,
        ExpressionKind, ExpressionTag, Function, ObjectExpression, ObjectPropertyKind, PropertyKey,
        PropertyKind, TSLiteral, TSMethodSignatureKind, TSSignature, TSTupleElement, TSType,
        TSTypeAnnotation, TSTypeOperatorOperator,
    },
    builder::NONE,
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
            TSType::new_ts_function_type(
                func.span,
                func.type_parameters.clone_in(self.allocator()),
                func.this_param.clone_in(self.allocator()),
                params,
                return_type,
                self,
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
            TSType::new_ts_function_type(
                func.span,
                func.type_parameters.clone_in(self.allocator()),
                NONE,
                params,
                return_type,
                self,
            )
        })
    }

    /// Convert a computed property key to a static property key when possible
    fn transform_property_key(&self, key: &PropertyKey<'a>) -> PropertyKey<'a> {
        match key.as_expression().map(Expression::kind) {
            // ["string"] -> string
            Some(ExpressionKind::StringLiteral(literal)) if is_identifier_name(&literal.value) => {
                PropertyKey::new_static_identifier(literal.span, literal.value.as_str(), self)
            }
            // [`string`] -> string
            Some(ExpressionKind::TemplateLiteral(literal))
                if is_identifier_name(&literal.quasis[0].value.raw) =>
            {
                PropertyKey::new_static_identifier(literal.span, literal.quasis[0].value.raw, self)
            }
            // [100] -> 100
            // number literal will be cloned as-is
            _ => key.clone_in(self.allocator()),
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

        let members = ArenaVec::from_iter_in(
            expr.properties.iter().filter_map(|property| match property {
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
                        let Some(function) = object.value.as_function_expression() else {
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
                        // PROTOTYPE: the old hand-written `Expression::is_string_literal` matched
                        // `StringLiteral | TemplateLiteral`; the generated one is strict
                        // (`StringLiteral` only), so match tags to preserve the old semantics.
                        let computed = key.as_expression().is_some_and(|k| {
                            !matches!(
                                k.tag(),
                                ExpressionTag::StringLiteral | ExpressionTag::TemplateLiteral
                            ) && !k.is_number_literal()
                        });

                        return Some(TSSignature::new_ts_method_signature(
                            object.span,
                            key,
                            computed,
                            false,
                            TSMethodSignatureKind::Method,
                            function.type_parameters.clone_in(self.allocator()),
                            function.this_param.clone_in(self.allocator()),
                            params,
                            return_type,
                            self,
                        ));
                    }

                    let type_annotation = match object.kind {
                        PropertyKind::Get => {
                            if accessor_inferred.iter().any(|k| k.content_eq(key)) {
                                return None;
                            }

                            let Some(function) = object.value.as_function_expression() else {
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

                            let Some(function) = object.value.as_function_expression() else {
                                unreachable!(
                                    "`object.kind` being `Set` guarantees that it is a function"
                                );
                            };
                            let annotation =
                                function.params.items.first().and_then(|param| {
                                    param.type_annotation.clone_in(self.allocator())
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
                                self.transform_const_expression_to_ts_type(&object.value)
                            } else {
                                self.infer_type_from_expression(&object.value)
                            };

                            if type_annotation.is_none() {
                                self.error(inferred_type_of_expression(object.value.span()));
                                return None;
                            }

                            type_annotation.map(|type_annotation| {
                                TSTypeAnnotation::boxed(SPAN, type_annotation, self)
                            })
                        }
                    };

                    let key = self.transform_property_key(key);
                    // PROTOTYPE: preserve the old `StringLiteral | TemplateLiteral` semantics of
                    // the previous hand-written `is_string_literal` (see above).
                    let property_signature = TSSignature::new_ts_property_signature(
                        object.span,
                        key.as_expression().is_some_and(|k| {
                            !matches!(
                                k.tag(),
                                ExpressionTag::StringLiteral | ExpressionTag::TemplateLiteral
                            ) && !k.is_number_literal()
                        }),
                        false,
                        is_const,
                        key,
                        type_annotation,
                        self,
                    );
                    Some(property_signature)
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    self.error(object_with_spread_assignments(spread.span));
                    None
                }
            }),
            self,
        );

        // Report an error if the type of neither the setter nor the getter is inferred.
        for (key, span) in accessor_spans {
            if !accessor_inferred.iter().any(|k| k.content_eq(key)) {
                self.error(inferred_type_of_expression(span));
            }
        }

        TSType::new_ts_type_literal(SPAN, members, self)
    }

    pub(crate) fn transform_array_expression_to_ts_type(
        &self,
        expr: &ArrayExpression<'a>,
        is_const: bool,
    ) -> TSType<'a> {
        let element_types = ArenaVec::from_iter_in(
            expr.elements.iter().filter_map(|element| match element {
                ArrayExpressionElement::SpreadElement(spread) => {
                    self.error(arrays_with_spread_elements(spread.span));
                    None
                }
                ArrayExpressionElement::Elision(elision) => {
                    Some(TSTupleElement::new_ts_undefined_keyword(elision.span, self))
                }
                ArrayExpressionElement::Expression(expression) => self
                    .transform_expression_to_ts_type_with_const_context(expression, is_const)
                    .map(TSTupleElement::from)
                    .or_else(|| {
                        self.error(inferred_type_of_expression(element.span()));
                        None
                    }),
            }),
            self,
        );

        let ts_type = TSType::new_ts_tuple_type(SPAN, element_types, self);
        if is_const {
            TSType::new_ts_type_operator_type(SPAN, TSTypeOperatorOperator::Readonly, ts_type, self)
        } else {
            ts_type
        }
    }

    // https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3-4.html#const-assertions
    pub(crate) fn transform_expression_to_ts_type(
        &self,
        expr: &Expression<'a>,
    ) -> Option<TSType<'a>> {
        self.transform_expression_to_ts_type_with_const_context(expr, false)
    }

    pub(crate) fn transform_const_expression_to_ts_type(
        &self,
        expr: &Expression<'a>,
    ) -> Option<TSType<'a>> {
        self.transform_expression_to_ts_type_with_const_context(expr, true)
    }

    fn transform_expression_to_ts_type_with_const_context(
        &self,
        expr: &Expression<'a>,
        is_const: bool,
    ) -> Option<TSType<'a>> {
        match expr.kind() {
            ExpressionKind::BooleanLiteral(lit) => Some(TSType::new_ts_literal_type(
                SPAN,
                TSLiteral::BooleanLiteral(ArenaBox::new_in(lit.clone_in(self.allocator()), self)),
                self,
            )),
            ExpressionKind::NumericLiteral(lit) => Some(TSType::new_ts_literal_type(
                SPAN,
                TSLiteral::NumericLiteral(ArenaBox::new_in(lit.clone_in(self.allocator()), self)),
                self,
            )),
            ExpressionKind::BigIntLiteral(lit) => Some(TSType::new_ts_literal_type(
                SPAN,
                TSLiteral::BigIntLiteral(ArenaBox::new_in(lit.clone_in(self.allocator()), self)),
                self,
            )),
            ExpressionKind::StringLiteral(lit) => Some(TSType::new_ts_literal_type(
                SPAN,
                TSLiteral::StringLiteral(ArenaBox::new_in(lit.clone_in(self.allocator()), self)),
                self,
            )),
            ExpressionKind::NullLiteral(lit) => Some(TSType::new_ts_null_keyword(lit.span, self)),
            ExpressionKind::Identifier(ident) => match ident.name.as_str() {
                "undefined" => Some(TSType::new_ts_undefined_keyword(ident.span, self)),
                _ => None,
            },
            ExpressionKind::TemplateLiteral(lit) => {
                self.transform_template_to_string(lit).map(|string| {
                    TSType::new_ts_literal_type(lit.span, TSLiteral::StringLiteral(string), self)
                })
            }
            ExpressionKind::UnaryExpression(expr) => {
                if Self::can_infer_unary_expression(expr) {
                    Some(TSType::new_ts_literal_type(
                        SPAN,
                        TSLiteral::UnaryExpression(ArenaBox::new_in(
                            expr.clone_in(self.allocator()),
                            self,
                        )),
                        self,
                    ))
                } else {
                    None
                }
            }
            ExpressionKind::ArrayExpression(expr) => {
                Some(self.transform_array_expression_to_ts_type(expr, is_const))
            }
            ExpressionKind::ObjectExpression(expr) => {
                Some(self.transform_object_expression_to_ts_type(expr, is_const))
            }
            ExpressionKind::FunctionExpression(func) => self.transform_function_to_ts_type(func),
            ExpressionKind::ArrowFunctionExpression(func) => {
                self.transform_arrow_function_to_ts_type(func)
            }
            ExpressionKind::TSAsExpression(expr) => {
                if expr.type_annotation.is_const_type_reference() {
                    self.transform_const_expression_to_ts_type(&expr.expression)
                } else {
                    Some(expr.type_annotation.clone_in(self.allocator()))
                }
            }
            ExpressionKind::TSTypeAssertion(expr) => {
                if expr.type_annotation.is_const_type_reference() {
                    self.transform_const_expression_to_ts_type(&expr.expression)
                } else {
                    Some(expr.type_annotation.clone_in(self.allocator()))
                }
            }
            ExpressionKind::ParenthesizedExpression(expr) => {
                self.transform_expression_to_ts_type_with_const_context(&expr.expression, is_const)
            }
            _ => None,
        }
    }
}
