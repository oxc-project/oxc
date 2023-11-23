use std::rc::Rc;

use oxc_ast::{ast::*, AstBuilder};
use oxc_span::GetSpan;

use crate::options::{TransformOptions, TransformTarget};

/// ES2015: Function Name
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-function-name>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-function-name>
pub struct FunctionName<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> FunctionName<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>, options: &TransformOptions) -> Option<Self> {
        (options.target < TransformTarget::ES2015 || options.function_name).then(|| Self { ast })
    }

    pub fn transform_function<'b>(&mut self, func: &'b mut Function<'a>) {
        if func.r#type != FunctionType::FunctionExpression {
            return;
        }
    }
}
