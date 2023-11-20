use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, Span, SPAN};
use std::{mem, rc::Rc};

use crate::utils::{is_valid_es3_identifier, is_valid_identifier};
use crate::{TransformOptions, TransformTarget};

/// ES2015: Template Literals
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-template-literals>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-template-literals>
pub struct PropertyLiteral<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> PropertyLiteral<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>, options: &TransformOptions) -> Option<Self> {
        (options.target <= TransformTarget::ES3 || options.property_literals).then(|| Self { ast })
    }

    pub fn transform_object_property<'b>(&mut self, expr: &'b mut ObjectProperty<'a>) {
        if expr.computed {
            return;
        }
        if let PropertyKey::Identifier(ident) = &expr.key {
            if !is_valid_es3_identifier(&ident.name) {
                let string_lit = self.ast.literal_string_expression(StringLiteral::new(
                    SPAN,
                    Atom::from(ident.name.clone()),
                ));
                expr.key = PropertyKey::Expression(string_lit);
            }
        }
    }
}
