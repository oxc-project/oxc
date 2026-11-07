use oxc_allocator::{ArenaVec, CloneIn, GetAllocator};
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, Expression, ObjectExpression, ObjectPropertyKind,
    PropertyKey, TSSignature, TSType, TSTypeAnnotation,
};
use oxc_span::{ContentEq, SPAN};
use oxc_syntax::identifier::is_identifier_name;

use crate::IsolatedDeclarations;

impl<'a> IsolatedDeclarations<'a> {
    /// Infer a TypeScript type from a JSON value, matching TypeScript's JSON module
    /// type inference (widened types).
    ///
    /// ```json
    /// { "name": "oxc", "keywords": ["a", "b"], "nested": { "count": 1 } }
    /// ```
    /// becomes
    /// ```ts
    /// {
    ///   name: string;
    ///   keywords: string[];
    ///   nested: {
    ///     count: number;
    ///   };
    /// }
    /// ```
    pub(crate) fn transform_json_expression_to_ts_type(&self, expr: &Expression<'a>) -> TSType<'a> {
        match expr {
            Expression::NullLiteral(_) => TSType::new_ts_null_keyword(SPAN, self),
            Expression::BooleanLiteral(_) => TSType::new_ts_boolean_keyword(SPAN, self),
            Expression::NumericLiteral(_) => TSType::new_ts_number_keyword(SPAN, self),
            Expression::StringLiteral(_) => TSType::new_ts_string_keyword(SPAN, self),
            // JSON allows negative numbers, which are parsed as a unary expression (e.g. `-1`).
            Expression::UnaryExpression(unary) => {
                self.transform_json_expression_to_ts_type(&unary.argument)
            }
            Expression::ObjectExpression(object) => {
                self.transform_json_object_expression_to_ts_type(object)
            }
            Expression::ArrayExpression(array) => {
                self.transform_json_array_expression_to_ts_type(array)
            }
            // Should be unreachable for valid JSON.
            _ => TSType::new_ts_unknown_keyword(SPAN, self),
        }
    }

    fn transform_json_object_expression_to_ts_type(
        &self,
        object: &ObjectExpression<'a>,
    ) -> TSType<'a> {
        let members = ArenaVec::from_iter_in(
            object.properties.iter().filter_map(|property| {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    return None;
                };
                let type_annotation = TSTypeAnnotation::boxed(
                    SPAN,
                    self.transform_json_expression_to_ts_type(&property.value),
                    self,
                );
                let key = self.transform_json_property_key(&property.key);
                Some(TSSignature::new_ts_property_signature(
                    SPAN,
                    false,
                    false,
                    false,
                    key,
                    Some(type_annotation),
                    self,
                ))
            }),
            self,
        );

        TSType::new_ts_type_literal(SPAN, members, self)
    }

    fn transform_json_array_expression_to_ts_type(
        &self,
        array: &ArrayExpression<'a>,
    ) -> TSType<'a> {
        // Collect the distinct element types, preserving first-occurrence order.
        let mut element_types: Vec<TSType<'a>> = Vec::new();
        for element in &array.elements {
            // JSON arrays never contain holes or spreads.
            if matches!(
                element,
                ArrayExpressionElement::SpreadElement(_) | ArrayExpressionElement::Elision(_)
            ) {
                continue;
            }
            let ts_type = self.transform_json_expression_to_ts_type(element.to_expression());
            if !element_types.iter().any(|existing| existing.content_eq(&ts_type)) {
                element_types.push(ts_type);
            }
        }

        let element_type = match element_types.len() {
            0 => TSType::new_ts_never_keyword(SPAN, self),
            1 => element_types.pop().unwrap(),
            _ => TSType::new_ts_union_type(SPAN, ArenaVec::from_iter_in(element_types, self), self),
        };

        TSType::new_ts_array_type(SPAN, element_type, self)
    }

    /// JSON keys are always string literals; emit them as identifiers when possible.
    fn transform_json_property_key(&self, key: &PropertyKey<'a>) -> PropertyKey<'a> {
        match key {
            PropertyKey::StringLiteral(literal) if is_identifier_name(&literal.value) => {
                PropertyKey::new_static_identifier(literal.span, literal.value.as_str(), self)
            }
            _ => key.clone_in(self.allocator()),
        }
    }
}
