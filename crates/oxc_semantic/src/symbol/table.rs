use std::collections::BTreeMap;
use std::ops::{Deref, Index, IndexMut};

use oxc_ast::ast::IdentifierReference;
use oxc_index::Idx;
use oxc_span::Span;

use super::{
    reference::{ResolvedReference, ResolvedReferenceId},
    Mangler, Symbol, SymbolId,
};

/// `SymbolTable` is a storage of all the symbols (related to `BindingIdentifiers`)
/// and references (related to `IdentifierReferences`) of the program. It supports two
/// kinds of queries: indexing by `SymbolId` retrieves the corresponding `Symbol` and
/// indexing by `ResolvedReferenceId` retrieves the correspodning `ResolvedReference`
///
#[derive(Debug, Default)]
pub struct SymbolTable {
    /// Stores all the `Symbols` indexed by `SymbolId`
    symbols: Vec<Symbol>,

    mangler: Mangler,

    /// Stores all the resolved references indexed by `ResolvedReferenceId`
    resolved_references: Vec<ResolvedReference>,

    resolved_references_index: BTreeMap<Span, ResolvedReferenceId>,

    symbol_index: BTreeMap<Span, SymbolId>,
}

impl Index<SymbolId> for SymbolTable {
    type Output = Symbol;

    fn index(&self, index: SymbolId) -> &Self::Output {
        &self.symbols[index.index()]
    }
}

impl IndexMut<SymbolId> for SymbolTable {
    fn index_mut(&mut self, index: SymbolId) -> &mut Self::Output {
        &mut self.symbols[index.index()]
    }
}

impl Index<ResolvedReferenceId> for SymbolTable {
    type Output = ResolvedReference;

    fn index(&self, index: ResolvedReferenceId) -> &Self::Output {
        &self.resolved_references[index.index()]
    }
}

impl IndexMut<ResolvedReferenceId> for SymbolTable {
    fn index_mut(&mut self, index: ResolvedReferenceId) -> &mut Self::Output {
        &mut self.resolved_references[index.index()]
    }
}

impl Deref for SymbolTable {
    type Target = Vec<Symbol>;

    fn deref(&self) -> &Self::Target {
        &self.symbols
    }
}

impl SymbolTable {
    pub fn new(
        symbols: Vec<Symbol>,
        mangler: Mangler,
        resolved_references: Vec<ResolvedReference>,
        resolved_references_index: BTreeMap<Span, ResolvedReferenceId>,
        symbol_index: BTreeMap<Span, SymbolId>,
    ) -> Self {
        Self { symbols, mangler, resolved_references, resolved_references_index, symbol_index }
    }

    pub fn mangle(&self) {
        self.mangler.compute_slot_frequency(&self.symbols);
    }

    pub fn symbols(&self) -> &Vec<Symbol> {
        &self.symbols
    }

    pub fn mangler(&self) -> &Mangler {
        &self.mangler
    }

    pub fn get_symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id.index())
    }

    pub fn resolved_references(&self) -> &Vec<ResolvedReference> {
        &self.resolved_references
    }

    pub fn get_resolved_reference(&self, id: ResolvedReferenceId) -> Option<&ResolvedReference> {
        self.resolved_references.get(id.index())
    }

    pub fn get_resolved_reference_for_id(
        &self,
        id: &IdentifierReference,
    ) -> Option<&ResolvedReference> {
        self.resolved_references_index
            .get(&id.span)
            .map(|ref_id| &self.resolved_references[ref_id.index()])
    }

    pub fn get_symbol_by_span(&self, span: Span) -> Option<&Symbol> {
        self.symbol_index.get(&span).map(|symbol_id| &self.symbols[symbol_id.index()])
    }
}
