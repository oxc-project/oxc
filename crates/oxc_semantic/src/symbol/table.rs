use std::ops::{Deref, Index, IndexMut};

use oxc_ast::{Atom, Span};

use super::reference::ResolvedReferenceId;
use super::{Symbol, SymbolFlags, SymbolId};
use crate::node::AstNodeId;
use crate::{Reference, ResolvedReference};

#[derive(Debug, Default)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
    resolved_references: Vec<ResolvedReference>,
}

impl Index<SymbolId> for SymbolTable {
    type Output = Symbol;

    fn index(&self, index: SymbolId) -> &Self::Output {
        &self.symbols[index.index0()]
    }
}

impl IndexMut<SymbolId> for SymbolTable {
    fn index_mut(&mut self, index: SymbolId) -> &mut Self::Output {
        &mut self.symbols[index.index0()]
    }
}

impl Deref for SymbolTable {
    type Target = Vec<Symbol>;

    fn deref(&self) -> &Self::Target {
        &self.symbols
    }
}

impl SymbolTable {
    #[must_use]
    pub fn symbols(&self) -> &Vec<Symbol> {
        &self.symbols
    }

    #[must_use]
    pub fn get(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id.index0())
    }

    #[must_use]
    pub fn create(
        &mut self,
        declaration: AstNodeId,
        name: Atom,
        span: Span,
        flags: SymbolFlags,
    ) -> SymbolId {
        let symbol_id = SymbolId::new(self.symbols.len() + 1);
        let symbol = Symbol::new(symbol_id, declaration, name, span, flags);
        self.symbols.push(symbol);
        symbol_id
    }

    #[must_use]
    pub fn get_resolved_reference(&self, id: ResolvedReferenceId) -> Option<&ResolvedReference> {
        self.resolved_references.get(id.index0())
    }

    /// Resolve `reference` to `symbol_id`
    pub fn resolve_reference(&mut self, reference: Reference, symbol_id: SymbolId) {
        let resolved_reference_id = ResolvedReferenceId::new(self.resolved_references.len() + 1);
        let resolved_reference = reference.resolve_to(symbol_id);
        self.resolved_references.push(resolved_reference);
        self.symbols[symbol_id].add_reference(resolved_reference_id);
    }
}
