use oxc_ast::{ast::*, AstBuilder};
use oxc_span::SPAN;
use std::rc::Rc;

use crate::context::TransformerCtx;
use crate::utils::is_valid_es3_identifier;
use crate::TransformTarget;

/// ES3: PropertyLiteral
///
/// References:
/// * <https://github.com/babel/babel/blob/master/packages/babel-plugin-transform-property-literals/src/index.js>
pub struct PropertyLiteral<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> PropertyLiteral<'a> {
    pub fn new(ctx: TransformerCtx<'a>) -> Option<Self> {
        (ctx.options.target <= TransformTarget::ES3 || ctx.options.property_literals)
            .then_some(Self { ast: ctx.ast })
    }

    pub fn transform_object_property<'b>(&mut self, expr: &'b mut ObjectProperty<'a>) {
        if expr.computed {
            return;
        }
        if let PropertyKey::Identifier(ident) = &expr.key {
            if !is_valid_es3_identifier(&ident.name) {
                let string_lit = self
                    .ast
                    .literal_string_expression(StringLiteral::new(SPAN, ident.name.clone()));
                expr.key = PropertyKey::Expression(string_lit);
            }
        }
    }
}
