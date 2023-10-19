use std::{cell::RefCell, rc::Rc};

use oxc_ast::AstBuilder;
use oxc_semantic::{ScopeTree, SymbolTable};

#[derive(Clone)]
pub struct TransformerCtx<'a> {
    pub ast: Rc<AstBuilder<'a>>,
    pub symbols: Rc<RefCell<SymbolTable>>,
    pub scopes: Rc<RefCell<ScopeTree>>,
}
