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
use oxc_ast::{ast::IdentifierReference, AstKind, Trivias};
use oxc_span::SourceType;
pub use oxc_syntax::{
    module_record::ModuleRecord,
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};

pub use crate::{
    node::{AstNode, AstNodeId, AstNodes, NodeFlags},
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
        let AstKind::IdentifierReference(id) = reference_node.kind() else {
            return false;
        };
        self.scopes().root_unresolved_references().contains_key(&id.name)
    }

    /// Find which scope a symbol is declared in
    pub fn symbol_scope(&self, symbol_id: SymbolId) -> ScopeId {
        self.symbols.get_scope_id(symbol_id)
    }

    /// Get all resolved references for a symbol
    pub fn symbol_references(
        &'a self,
        symbol_id: SymbolId,
    ) -> impl Iterator<Item = &'a Reference> + '_ {
        self.symbols.get_resolved_references(symbol_id)
    }

    pub fn symbol_declaration(&self, symbol_id: SymbolId) -> AstNodeId {
        self.symbols.get_declaration(symbol_id)
    }

    pub fn is_reference_to_global_variable(&self, ident: &IdentifierReference) -> bool {
        self.scopes().root_unresolved_references().contains_key(&ident.name)
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_ast::{ast::VariableDeclarationKind, AstKind};
    use oxc_span::{Atom, SourceType};

    use super::*;

    /// Create a [`Semantic`] from source code, assuming there are no syntax/semantic errors.
    fn get_semantic<'s, 'a: 's>(
        allocator: &'a Allocator,
        source: &'s str,
        source_type: SourceType,
    ) -> Semantic<'s> {
        let parse = oxc_parser::Parser::new(allocator, source, source_type).parse();
        assert!(parse.errors.is_empty());
        let program = allocator.alloc(parse.program);
        let semantic = SemanticBuilder::new(source, source_type).build(program);
        assert!(semantic.errors.is_empty());
        semantic.semantic
    }

    #[test]
    fn test_symbols() {
        let source = "
            let a;
            function foo(a) {
                return a + 1;
            }
            let b = a + foo(1);";
        let allocator = Allocator::default();
        let semantic = get_semantic(&allocator, source, SourceType::default());

        let top_level_a = semantic
            .scopes()
            .get_binding(semantic.scopes().root_scope_id(), &Atom::from("a"))
            .unwrap();

        let decl = semantic.symbol_declaration(top_level_a);
        match semantic.nodes().get_node(decl).kind() {
            AstKind::VariableDeclarator(decl) => {
                assert_eq!(decl.kind, VariableDeclarationKind::Let);
            }
            kind => panic!("Expected VariableDeclarator for 'let', got {kind:?}"),
        }

        let references = semantic.symbol_references(top_level_a);
        assert_eq!(references.count(), 1);
    }

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
        let semantic = get_semantic(&allocator, source, SourceType::default());
        for node in semantic.nodes().iter() {
            if let AstKind::IdentifierReference(id) = node.kind() {
                assert!(!semantic.is_reference_to_global_variable(id));
            }
        }
    }
}
