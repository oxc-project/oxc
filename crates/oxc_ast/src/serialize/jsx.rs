use oxc_ast_macros::ast_meta;
use oxc_estree::{ESTree, JsonSafeString, Serializer, StructSerializer};
use oxc_syntax::node::NodeId;

use crate::ast::*;

/// Serializer for `opening_element` field of `JSXElement`.
///
/// `selfClosing` field of `JSXOpeningElement` depends on whether `JSXElement` has a `closing_element`.
#[ast_meta]
#[estree(
    ts_type = "JSXOpeningElement",
    raw_deser = "
        const openingElement = DESER[Box<JSXOpeningElement>](POS_OFFSET.opening_element);
        if (THIS.closingElement === null) openingElement.selfClosing = true;
        openingElement
    "
)]
pub struct JSXElementOpeningElement<'a, 'b>(pub &'b JSXElement<'a>);

impl ESTree for JSXElementOpeningElement<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let element = self.0;
        let opening_element = element.opening_element.as_ref();

        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("JSXOpeningElement"));
        state.serialize_field("name", &opening_element.name);
        state.serialize_ts_field("typeArguments", &opening_element.type_arguments);
        state.serialize_field("attributes", &opening_element.attributes);
        state.serialize_field("selfClosing", &element.closing_element.is_none());
        state.serialize_span(opening_element.span);
        state.end();
    }
}

/// Converter for `selfClosing` field of `JSXOpeningElement`.
///
/// This converter is not used for serialization - `JSXElementOpening` above handles serialization.
/// This type is only required to add `selfClosing: boolean` to TS type def,
/// and provide default value of `false` for raw transfer deserializer.
#[ast_meta]
#[estree(ts_type = "boolean", raw_deser = "false")]
pub struct JSXOpeningElementSelfClosing<'a, 'b>(#[expect(dead_code)] pub &'b JSXOpeningElement<'a>);

impl ESTree for JSXOpeningElementSelfClosing<'_, '_> {
    fn serialize<S: Serializer>(&self, _serializer: S) {
        unreachable!();
    }
}

/// Serializer for `IdentifierReference` variant of `JSXElementName` and `JSXMemberExpressionObject`.
///
/// Convert to `JSXIdentifier`.
#[ast_meta]
#[estree(
    ts_type = "JSXIdentifier",
    raw_deser = "
        const ident = DESER[Box<IdentifierReference>](POS);
        { type: 'JSXIdentifier', name: ident.name, start: ident.start, end: ident.end, ...(RANGE && { range: ident.range }), ...(PARENT && { parent }) }
    "
)]
pub struct JSXElementIdentifierReference<'a, 'b>(pub &'b IdentifierReference<'a>);

impl ESTree for JSXElementIdentifierReference<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JSXIdentifier { span: self.0.span, node_id: NodeId::DUMMY, name: self.0.name }
            .serialize(serializer);
    }
}

/// Serializer for `ThisExpression` variant of `JSXElementName` and `JSXMemberExpressionObject`.
///
/// Convert to `JSXIdentifier`.
#[ast_meta]
#[estree(
    ts_type = "JSXIdentifier",
    raw_deser = "
        const thisExpr = DESER[Box<ThisExpression>](POS);
        { type: 'JSXIdentifier', name: 'this', start: thisExpr.start, end: thisExpr.end, ...(RANGE && { range: thisExpr.range }), ...(PARENT && { parent }) }
    "
)]
pub struct JSXElementThisExpression<'b>(pub &'b ThisExpression);

impl ESTree for JSXElementThisExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JSXIdentifier { span: self.0.span, node_id: NodeId::DUMMY, name: Atom::from("this") }
            .serialize(serializer);
    }
}
