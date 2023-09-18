use oxc_ast::AstBuilder;

use std::rc::Rc;

/// Transform TypeScript
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-typescript>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-typescript>
pub struct TypeScript<'a> {
    _ast: Rc<AstBuilder<'a>>,
}

impl<'a> TypeScript<'a> {
    pub fn new(_ast: Rc<AstBuilder<'a>>) -> Self {
        Self { _ast }
    }
}
