use std::{
    collections::BTreeMap,
    ops::{Index, IndexMut},
};

use oxc_ast::{Atom, Span};

use super::{reference::ResolvedReferenceId, SymbolId};
use crate::{node::AstNodeId, Reference, ResolvedReference, Symbol, SymbolFlags, SymbolTable};

#[derive(Debug, Default)]
pub struct SymbolTableBuilder {
    /// Stores all the `Symbols` indexed by `SymbolId`
    symbols: Vec<Symbol>,
    /// Stores all the resolved references indexed by `ResolvedReferenceId`
    resolved_references: Vec<ResolvedReference>,
    // BTreeMap is empirically a lot faster than FxHashMap for our insertion,
    resolved_references_index: BTreeMap<Span, ResolvedReferenceId>,
}

impl Index<SymbolId> for SymbolTableBuilder {
    type Output = Symbol;

    fn index(&self, index: SymbolId) -> &Self::Output {
        &self.symbols[index.index0()]
    }
}

impl IndexMut<SymbolId> for SymbolTableBuilder {
    fn index_mut(&mut self, index: SymbolId) -> &mut Self::Output {
        &mut self.symbols[index.index0()]
    }
}

impl SymbolTableBuilder {
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

    /// Resolve all `references` to `symbol_id`
    pub fn resolve_reference(&mut self, references: Vec<Reference>, symbol_id: SymbolId) {
        let additional_len = references.len();
        let symbol = &mut self.symbols[symbol_id];

        self.resolved_references.reserve(additional_len);
        symbol.references.reserve(additional_len);

        for reference in references {
            let resolved_reference_id =
                ResolvedReferenceId::new(self.resolved_references.len() + 1);
            self.resolved_references_index.insert(reference.span, resolved_reference_id);

            let resolved_reference = reference.resolve_to(symbol_id);
            self.resolved_references.push(resolved_reference);
            // explicitly push to vector here in correspondence to the previous reserve call
            symbol.references.push(resolved_reference_id);
        }
    }

    pub fn build(self) -> SymbolTable {
        SymbolTable::new(self.symbols, self.resolved_references, self.resolved_references_index)
    }
}
