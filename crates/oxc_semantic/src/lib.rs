#![feature(let_chains)]

mod binder;
mod builder;
mod checker;
mod diagnostics;
mod jsdoc;
mod module_record;
mod node;
mod reference;
mod scope;
mod symbol;

use std::rc::Rc;

pub use builder::{SemanticBuilder, SemanticBuilderReturn};
pub use jsdoc::{JSDoc, JSDocComment, JSDocTag};
use oxc_ast::{ast::IdentifierReference, module_record::ModuleRecord, AstKind, Trivias};
use oxc_span::SourceType;
pub use oxc_syntax::{
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};

pub use crate::{
    node::{AstNode, AstNodeId, AstNodes},
    reference::{Reference, ReferenceFlag, ReferenceId},
    scope::ScopeTree,
    symbol::SymbolTable,
};

pub struct Semantic<'a> {
    source_text: &'a str,

    source_type: SourceType,

    nodes: AstNodes<'a>,

    scopes: ScopeTree,

    symbols: SymbolTable,

    trivias: Rc<Trivias>,

    module_record: ModuleRecord,

    jsdoc: JSDoc<'a>,

    unused_labels: Vec<AstNodeId>,
}

impl<'a> Semantic<'a> {
    pub fn into_symbol_table_and_scope_tree(self) -> (SymbolTable, ScopeTree) {
        (self.symbols, self.scopes)
    }

    pub fn source_text(&self) -> &'a str {
        self.source_text
    }

    pub fn source_type(&self) -> &SourceType {
        &self.source_type
    }

    pub fn nodes(&self) -> &AstNodes<'a> {
        &self.nodes
    }

    pub fn scopes(&self) -> &ScopeTree {
        &self.scopes
    }

    pub fn trivias(&self) -> &Trivias {
        &self.trivias
    }

    pub fn jsdoc(&self) -> &JSDoc<'a> {
        &self.jsdoc
    }

    pub fn module_record(&self) -> &ModuleRecord {
        &self.module_record
    }

    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    pub fn unused_labels(&self) -> &Vec<AstNodeId> {
        &self.unused_labels
    }

    pub fn is_unresolved_reference(&self, node_id: AstNodeId) -> bool {
        let reference_node = self.nodes.get_node(node_id);
        let AstKind::IdentifierReference(id) = reference_node.kind() else { return false; };
        self.scopes().root_unresolved_references().contains_key(&id.name)
    }

    pub fn symbol_scope(&self, symbol_id: SymbolId) -> ScopeId {
        self.symbols.get_scope_id(symbol_id)
    }

    pub fn is_reference_to_global_variable(&self, ident: &IdentifierReference) -> bool {
        self.scopes().root_unresolved_references().contains_key(&ident.name)
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_ast::AstKind;
    use oxc_span::SourceType;

    use crate::SemanticBuilder;

    #[test]
    fn test_is_global() {
        let source = "
        var a = 0;
        function foo() {
          a += 1;
        }

        var b = a + 2;
      ";
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let parse =
            oxc_parser::Parser::new(&allocator, source, oxc_span::SourceType::default()).parse();
        assert!(parse.errors.is_empty());
        let program = allocator.alloc(parse.program);

        {
            let semantic = SemanticBuilder::new(source, source_type).build(program);
            assert!(semantic.errors.is_empty());
            let semantic = semantic.semantic;
            for node in semantic.nodes().iter() {
                if let AstKind::IdentifierReference(id) = node.kind() {
                    assert!(!semantic.is_reference_to_global_variable(id));
                }
            }
        }
    }
}
