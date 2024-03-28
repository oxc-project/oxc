use crate::{context::TransformerCtx, ReactJsxOptions};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_syntax::*;

/// @babel/plugin-transform-react-jsx-source
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-react-jsx-source>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-react-jsx-source>
pub struct JsxSrc<'a> {
    ctx: TransformerCtx<'a>,
    options: ReactJsxOptions,
}

impl<'a> JsxSrc<'a> {
    pub fn new(ctx: TransformerCtx<'a>) -> Option<Self> {
        let jsx_options = ctx.options.react_jsx.clone()?.with_comments(&ctx.semantic());
        Some(Self { ctx, options: jsx_options })
    }
    pub fn transform_jsx_opening_element(&mut self, elem: &mut JSXOpeningElement<'a>) {
        if self.options.development != Some(true) {
            return;
        }
        let mut properties = self.ctx.ast.new_vec();
        let kind = PropertyKind::Init;
        // fileName
        properties.push(ObjectPropertyKind::ObjectProperty(self.ctx.ast.object_property(
            SPAN,
            kind,
            self.ctx.ast.property_key_identifier(IdentifierName::new(SPAN, "fileName".into())),
            self.ctx.ast.literal_string_expression(StringLiteral {
                span: SPAN,
                value: self.ctx.ast.new_atom(&self.ctx.file_name),
            }),
            None,
            false,
            false,
            false,
        )));
        // lineNumber
        properties.push(ObjectPropertyKind::ObjectProperty(self.ctx.ast.object_property(
            SPAN,
            kind,
            self.ctx.ast.property_key_identifier(IdentifierName::new(SPAN, "lineNumber".into())),
            // TODO: i am not sure it there any way for me to know the lineNumber.. maybe by SPAN
            self.ctx.ast.literal_number_expression(self.ctx.ast.number_literal(
                SPAN,
                10.0,
                "10",
                NumberBase::Decimal,
            )),
            None,
            false,
            false,
            false,
        )));
        // columnNumber
        properties.push(ObjectPropertyKind::ObjectProperty(self.ctx.ast.object_property(
            SPAN,
            kind,
            self.ctx.ast.property_key_identifier(IdentifierName::new(SPAN, "columnNumber".into())),
            // TODO: i am not sure it there any way for me to know the lineNumber.. maybe by SPAN
            self.ctx.ast.literal_number_expression(self.ctx.ast.number_literal(
                SPAN,
                1.0,
                "1",
                NumberBase::Decimal,
            )),
            None,
            false,
            false,
            false,
        )));

        let attribute_item = self.ctx.ast.jsx_attribute(
            SPAN,
            JSXAttributeName::Identifier(self.ctx.ast.jsx_identifier(SPAN, "__source".into())),
            Some(JSXAttributeValue::ExpressionContainer(self.ctx.ast.jsx_expression_container(
                SPAN,
                JSXExpression::Expression(self.ctx.ast.object_expression(SPAN, properties, None)),
            ))),
        );

        elem.attributes.push(JSXAttributeItem::Attribute(attribute_item));
    }
}
