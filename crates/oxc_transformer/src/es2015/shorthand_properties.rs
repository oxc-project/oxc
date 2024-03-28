use std::rc::Rc;

use oxc_ast::{ast::*, AstBuilder};
use oxc_span::GetSpan;

use crate::{context::TransformerCtx, options::TransformTarget};

/// ES2015: Shorthand Properties
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-shorthand-properties>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-shorthand-properties>
pub struct ShorthandProperties<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> ShorthandProperties<'a> {
    pub fn new(ctx: TransformerCtx<'a>) -> Option<Self> {
        (ctx.options.target < TransformTarget::ES2015 || ctx.options.shorthand_properties)
            .then_some(Self { ast: ctx.ast })
    }

    pub fn transform_object_property<'b>(&mut self, obj_prop: &'b mut ObjectProperty<'a>) {
        if !obj_prop.shorthand && !obj_prop.method {
            return;
        }

        obj_prop.shorthand = false;
        obj_prop.method = false;

        if obj_prop.computed {
            // all computed key can never be transformed to `__proto__` setter unexpectedly
            return;
        }

        // We should handle the edge case of `__proto__` property.
        // All shorthand properties with key `__proto__` can never be `__proto__` setter.
        // But the transformed result can be `__proto__` setter unexpectedly.
        // It's easy to fix it by using computed property.

        let is_proto_string = obj_prop.key.is_specific_string_literal("__proto__");

        if !is_proto_string && !obj_prop.key.is_specific_id("__proto__") {
            return;
        }
        // We reach here, it means that the key is `__proto__` or `"__proto__"`.

        // Transform `__proto__`/`"__proto__"` to computed property.
        obj_prop.computed = true;

        if is_proto_string {
            // After the transformation, the string literal `"__proto__"` is already expected result.
            //
            // input:
            // "__proto__"() {}
            // output:
            // ["__proto__"]: function() {}
            return;
        }

        // Convert `[__proto__]` to `["__proto__"]`

        let proto = StringLiteral { span: obj_prop.key.span(), value: "__proto__".into() };
        let expr = self.ast.literal_string_expression(proto);
        obj_prop.key = PropertyKey::Expression(expr);
    }
}
