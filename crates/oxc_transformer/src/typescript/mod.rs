use oxc_ast::ast::*;
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

    #[allow(clippy::unused_self)]
    pub fn transform_formal_parameters(&self, params: &mut FormalParameters<'a>) {
        if params.items.get(0).is_some_and(|param| matches!(&param.pattern.kind, BindingPatternKind::BindingIdentifier(ident) if ident.name =="this")) {
            params.items.remove(0);
        }
    }
}
