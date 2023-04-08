#![feature(is_some_and)]
#![feature(let_chains)]

mod binder;
mod builder;
mod module_record;
mod node;
mod scope;
mod symbol;

use std::rc::Rc;

pub use builder::SemanticBuilder;
use node::AstNodeId;
pub use node::{AstNode, AstNodes, SemanticNode};
use oxc_ast::{
    ast::IdentifierReference, module_record::ModuleRecord, AstKind, SourceType, Trivias,
};
use scope::ScopeId;
pub use scope::{Scope, ScopeFlags, ScopeTree};
use symbol::SymbolId;
pub use symbol::{Reference, ResolvedReference, Symbol, SymbolFlags, SymbolTable};

pub struct Semantic<'a> {
    source_text: &'a str,

    source_type: SourceType,

    nodes: AstNodes<'a>,

    scopes: ScopeTree,

    symbols: SymbolTable,

    trivias: Rc<Trivias>,

    module_record: ModuleRecord,
}

impl<'a> Semantic<'a> {
    #[must_use]
    pub fn source_text(&self) -> &'a str {
        self.source_text
    }

    #[must_use]
    pub fn source_type(&self) -> &SourceType {
        &self.source_type
    }

    #[must_use]
    pub fn nodes(&self) -> &AstNodes<'a> {
        &self.nodes
    }

    #[must_use]
    pub fn scopes(&self) -> &ScopeTree {
        &self.scopes
    }

    #[must_use]
    pub fn trivias(&self) -> &Trivias {
        &self.trivias
    }

    #[must_use]
    pub fn module_record(&self) -> &ModuleRecord {
        &self.module_record
    }

    #[must_use]
    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    #[must_use]
    pub fn is_unresolved_reference(&self, node_id: AstNodeId) -> bool {
        let reference_node = &self.nodes()[node_id];
        let AstKind::IdentifierReference(id) = reference_node.kind() else { return false; };
        let scope = &self.scopes()[reference_node.scope_id()];
        scope.unresolved_references.contains_key(&id.name)
    }

    #[must_use]
    pub fn symbol_scope(&self, symbol_id: SymbolId) -> ScopeId {
        let symbol = &self.symbols[symbol_id];
        let declaration = symbol.declaration();
        self.nodes[declaration].scope_id()
    }

    #[must_use]
    pub fn is_reference_to_global_variables(&self, id: &IdentifierReference) -> bool {
        // unresolved references are treated as reference to global.
        self.symbols.get_resolved_reference_for_id(id).map_or(true, |reference_id| {
            let referred_symbol = reference_id.resolved_symbol_id;
            let symbol_scope = self.symbol_scope(referred_symbol);
            // Symbol declared in top level
            symbol_scope == self.scopes().root_scope_id()
        })
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_ast::{AstKind, SourceType};

    use crate::SemanticBuilder;

    #[test]
    fn test_is_global() {
        let source = "
        var a = 0;
        function foo() {
          a += 1;
        }

        var b = a + 2;

        console.log(b);
      ";
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let parse =
            oxc_parser::Parser::new(&allocator, source, oxc_ast::SourceType::default()).parse();
        assert!(parse.errors.is_empty());
        let program = allocator.alloc(parse.program);

        {
            let semantic = SemanticBuilder::new(source, source_type, &parse.trivias).build(program);
            assert!(semantic.errors.is_empty());
            let semantic = semantic.semantic;
            for node in semantic.nodes().iter() {
                if let AstKind::IdentifierReference(id) = node.get().kind() {
                    assert!(semantic.is_reference_to_global_variables(id));
                }
            }
        }
    }
}
