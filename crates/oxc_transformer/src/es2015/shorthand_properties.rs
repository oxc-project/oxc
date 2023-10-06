use oxc_ast::{ast::*, AstBuilder};
use oxc_span::GetSpan;

use std::rc::Rc;

/// ES2015: Shorthand Properties
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-shorthand-properties>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-shorthand-properties>
pub struct ShorthandProperties<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> ShorthandProperties<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>) -> Self {
        Self { ast }
    }

    pub fn transform_object_property<'b>(&mut self, obj_prop: &'b mut ObjectProperty<'a>) {
        if !obj_prop.shorthand && !obj_prop.method {
            return;
        }

        obj_prop.shorthand = false;
        obj_prop.method = false;

        if !obj_prop.key.is_specific_id("__proto__") || obj_prop.computed {
            return;
        }

        obj_prop.computed = true;

        let proto = StringLiteral { span: obj_prop.key.span(), value: "__proto__".into() };
        let expr = self.ast.literal_string_expression(proto);
        obj_prop.key = PropertyKey::Expression(expr);
    }
}
