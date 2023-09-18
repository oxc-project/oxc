use std::rc::Rc;

use oxc_ast::AstBuilder;

use crate::TransformReactOptions;

/// Transform React JSX
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-react-jsx>
/// * <https://github.com/babel/babel/tree/main/packages/babel-helper-builder-react-jsx>
pub struct ReactJsx<'a> {
    _ast: Rc<AstBuilder<'a>>,
    _options: TransformReactOptions,
}

impl<'a> ReactJsx<'a> {
    pub fn new(_ast: Rc<AstBuilder<'a>>, _options: TransformReactOptions) -> Self {
        Self { _ast, _options }
    }
}
