use std::rc::Rc;

use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_syntax::NumberBase;

use crate::context::Ctx;

/// [plugin-transform-react-jsx-source](https://babeljs.io/docs/babel-plugin-transform-react-jsx-source)
///
/// This plugin is included in `preset-react` and only enabled in development mode.
///
/// ## Example
///
/// In: `<sometag />`
/// Out: `<sometag __source={ { fileName: 'this/file.js', lineNumber: 10, columnNumber: 1 } } />`
///
/// TODO: get lineNumber and columnNumber from somewhere
pub struct ReactJsxSource<'a> {
    ctx: Ctx<'a>,
}

impl<'a> ReactJsxSource<'a> {
    pub fn new(ctx: &Ctx<'a>) -> Self {
        Self { ctx: Rc::clone(ctx) }
    }

    pub fn transform_jsx_opening_element(&self, elem: &mut JSXOpeningElement<'a>) {
        self.add_source_attribute(elem);
    }
}

impl<'a> ReactJsxSource<'a> {
    /// `<sometag __source={ { fileName: 'this/file.js', lineNumber: 10, columnNumber: 1 } } />`
    ///           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    fn add_source_attribute(&self, elem: &mut JSXOpeningElement<'a>) {
        let kind = PropertyKind::Init;

        let filename = {
            let name = IdentifierName::new(SPAN, "fileName".into());
            let key = self.ctx.ast.property_key_identifier(name);
            let string = StringLiteral::new(SPAN, self.ctx.ast.new_atom(self.ctx.filename()));
            let value = self.ctx.ast.literal_string_expression(string);
            self.ctx.ast.object_property(SPAN, kind, key, value, None, false, false, false)
        };

        let line_number = {
            let ident = IdentifierName::new(SPAN, "lineNumber".into());
            let key = self.ctx.ast.property_key_identifier(ident);
            let number = self.ctx.ast.number_literal(SPAN, 1.0, "1", NumberBase::Decimal);
            let value = self.ctx.ast.literal_number_expression(number);
            self.ctx.ast.object_property(SPAN, kind, key, value, None, false, false, false)
        };

        let column_number = {
            let ident = IdentifierName::new(SPAN, "columnNumber".into());
            let key = self.ctx.ast.property_key_identifier(ident);
            let number = self.ctx.ast.number_literal(SPAN, 1.0, "1", NumberBase::Decimal);
            let value = self.ctx.ast.literal_number_expression(number);
            self.ctx.ast.object_property(SPAN, kind, key, value, None, false, false, false)
        };

        let mut properties = self.ctx.ast.new_vec();
        properties.push(ObjectPropertyKind::ObjectProperty(filename));
        properties.push(ObjectPropertyKind::ObjectProperty(line_number));
        properties.push(ObjectPropertyKind::ObjectProperty(column_number));

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
