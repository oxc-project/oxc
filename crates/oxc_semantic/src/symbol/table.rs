use std::collections::BTreeMap;
use std::ops::{Deref, Index, IndexMut};

use oxc_ast::ast::IdentifierReference;
use oxc_ast::Span;

use super::reference::ResolvedReferenceId;
use super::{Symbol, SymbolId};
use crate::ResolvedReference;

/// `SymbolTable` is a storage of all the symbols (related to `BindingIdentifiers`)
/// and references (related to `IdentifierReferences`) of the program. It supports two
/// kinds of queries: indexing by `SymbolId` retrieves the corresponding `Symbol` and
/// indexing by `ResolvedReferenceId` retrieves the correspodning `ResolvedReference`
///
#[derive(Debug, Default)]
pub struct SymbolTable {
    /// Stores all the `Symbols` indexed by `SymbolId`
    symbols: Vec<Symbol>,
    /// Stores all the resolved references indexed by `ResolvedReferenceId`
    resolved_references: Vec<ResolvedReference>,
    resolved_references_index: BTreeMap<Span, ResolvedReferenceId>,
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

impl Index<ResolvedReferenceId> for SymbolTable {
    type Output = ResolvedReference;

    fn index(&self, index: ResolvedReferenceId) -> &Self::Output {
        &self.resolved_references[index.index0()]
    }
}

impl IndexMut<ResolvedReferenceId> for SymbolTable {
    fn index_mut(&mut self, index: ResolvedReferenceId) -> &mut Self::Output {
        &mut self.resolved_references[index.index0()]
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
    pub fn new(
        symbols: Vec<Symbol>,
        resolved_references: Vec<ResolvedReference>,
        resolved_references_index: BTreeMap<Span, ResolvedReferenceId>,
    ) -> Self {
        Self { symbols, resolved_references, resolved_references_index }
    }

    #[must_use]
    pub fn symbols(&self) -> &Vec<Symbol> {
        &self.symbols
    }

    #[must_use]
    pub fn get_symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id.index0())
    }

    #[must_use]
    pub fn resolved_references(&self) -> &Vec<ResolvedReference> {
        &self.resolved_references
    }

    #[must_use]
    pub fn get_resolved_reference(&self, id: ResolvedReferenceId) -> Option<&ResolvedReference> {
        self.resolved_references.get(id.index0())
    }

    #[must_use]
    pub fn get_resolved_reference_for_id(
        &self,
        id: &IdentifierReference,
    ) -> Option<&ResolvedReference> {
        self.resolved_references_index
            .get(&id.span)
            .map(|ref_id| &self.resolved_references[ref_id.index0()])
    }
}
