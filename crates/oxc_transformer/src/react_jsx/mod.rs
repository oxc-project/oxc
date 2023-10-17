use std::rc::Rc;

use oxc_ast::AstBuilder;

#[derive(Debug, Default, Clone, Copy)]
pub struct ReactJsxOptions {
    _runtime: ReactJsxRuntime,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ReactJsxRuntime {
    #[default]
    Classic,
    #[allow(unused)]
    Automatic,
}

/// Transform React JSX
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-react-jsx>
/// * <https://github.com/babel/babel/tree/main/packages/babel-helper-builder-react-jsx>
pub struct ReactJsx<'a> {
    _ast: Rc<AstBuilder<'a>>,
    _options: ReactJsxOptions,
}

impl<'a> ReactJsx<'a> {
    pub fn new(_ast: Rc<AstBuilder<'a>>, _options: ReactJsxOptions) -> Self {
        Self { _ast, _options }
    }
}
