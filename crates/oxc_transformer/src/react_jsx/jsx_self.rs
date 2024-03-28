use crate::{context::TransformerCtx, ReactJsxOptions};
use oxc_ast::ast::*;
use oxc_span::SPAN;

/// @babel/plugin-transform-react-jsx-self
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-react-jsx-self>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-react-jsx-self>
///
pub struct JsxSelf<'a> {
    ctx: TransformerCtx<'a>,
    options: ReactJsxOptions,
}

impl<'a> JsxSelf<'a> {
    pub fn new(ctx: TransformerCtx<'a>) -> Option<Self> {
        let jsx_options = ctx.options.react_jsx.clone()?;
        Some(Self { ctx, options: jsx_options })
    }
    pub fn transform_jsx_opening_element(&mut self, elem: &mut JSXOpeningElement<'a>) {
        if self.options.development != Some(true) {
            return;
        }

        elem.attributes.push(JSXAttributeItem::Attribute(self.ctx.ast.jsx_attribute(
            SPAN,
            JSXAttributeName::Identifier(self.ctx.ast.jsx_identifier(SPAN, "__self".into())),
            Some(JSXAttributeValue::ExpressionContainer(self.ctx.ast.jsx_expression_container(
                SPAN,
                JSXExpression::Expression(self.ctx.ast.this_expression(SPAN)),
            ))),
        )));
    }
}
