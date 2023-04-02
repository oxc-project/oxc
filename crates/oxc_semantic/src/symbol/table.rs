use std::ops::{Deref, Index, IndexMut};

use oxc_ast::{Atom, Span};
use rustc_hash::FxHashMap;

use super::{Symbol, SymbolFlags, SymbolId};
use crate::node::AstNodeId;
use crate::ResolvedReference;

#[derive(Debug, Default)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
    symbols_index_span: FxHashMap<Span, SymbolId>,

    resolved_references: Vec<ResolvedReference>,
    reference_index_nodeid: FxHashMap<AstNodeId, usize>,
    reference_index_span: FxHashMap<Span, usize>,
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
    pub fn get_symbol_at_span(&self, span: Span) -> Option<&Symbol> {
        self.symbols_index_span.get(&span).and_then(|&id| self.symbols.get(id.index0()))
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
        self.symbols_index_span.insert(span, symbol_id);
        symbol_id
    }

    #[must_use]
    pub fn get_resolved_reference(&self, id: AstNodeId) -> Option<&ResolvedReference> {
        let reference_id = self.reference_index_nodeid.get(&id);
        reference_id.map(|&id| &self.resolved_references[id])
    }

    #[must_use]
    pub fn get_resolved_reference_at_span(&self, span: Span) -> Option<&ResolvedReference> {
        let reference_id = self.reference_index_span.get(&span);
        reference_id.map(|&id| &self.resolved_references[id])
    }

    pub fn resolve_reference(&mut self, id: AstNodeId, reference: ResolvedReference) {
        let span = reference.span();
        let reference_id = self.resolved_references.len();
        self.resolved_references.push(reference);
        self.reference_index_nodeid.insert(id, reference_id);
        self.reference_index_span.insert(span, reference_id);
    }
}
