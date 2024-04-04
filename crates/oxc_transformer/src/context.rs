use std::rc::Rc;

use oxc_ast::AstBuilder;

use crate::CompilerAssumptions;

pub struct TransformerContext<'a> {
    pub ast: Rc<AstBuilder<'a>>,
    pub assumptions: CompilerAssumptions,
}
