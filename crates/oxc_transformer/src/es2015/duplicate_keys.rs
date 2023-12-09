use std::rc::Rc;

use oxc_ast::{ast::*, AstBuilder};
use oxc_span::GetSpan;

use crate::options::{TransformOptions, TransformTarget};

/// ES2015: Duplicate Keys
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-duplicate-keys>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-duplicate-keys>
pub struct DuplicateKeys<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> DuplicateKeys<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>, options: &TransformOptions) -> Option<Self> {
        (options.target < TransformTarget::ES2015 || options.shorthand_properties)
            .then(|| Self { ast })
    }

    pub fn transform_object_expression<'b>(&mut self, obj_expr: &mut ObjectExpression) {
        
    }
}
