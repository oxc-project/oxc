use oxc_index::IndexVec;
use oxc_span::{Atom, Span};
pub use oxc_syntax::{
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};

use crate::reference::{Reference, ReferenceId};

/// Symbol Table
///
/// `SoA` (Struct of Arrays) for memory efficiency.
#[derive(Debug)]
pub struct SymbolTable {
    pub(crate) spans: IndexVec<SymbolId, Span>,
    pub(crate) names: IndexVec<SymbolId, Atom>,
    pub(crate) flags: IndexVec<SymbolId, SymbolFlags>,
    pub(crate) scope_ids: IndexVec<SymbolId, ScopeId>,
    pub(crate) resolved_references: IndexVec<SymbolId, Vec<ReferenceId>>,
    pub(crate) references: IndexVec<ReferenceId, Reference>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            spans: IndexVec::new(),
            names: IndexVec::new(),
            flags: IndexVec::new(),
            scope_ids: IndexVec::new(),
            resolved_references: IndexVec::new(),
            references: IndexVec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.spans.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_name(&self, symbol_id: SymbolId) -> &Atom {
        &self.names[symbol_id]
    }

    pub fn get_flag(&self, symbol_id: SymbolId) -> SymbolFlags {
        self.flags[symbol_id]
    }

    pub fn get_scope_id(&self, symbol_id: SymbolId) -> ScopeId {
        self.scope_ids[symbol_id]
    }

    pub fn create_symbol(
        &mut self,
        span: Span,
        name: Atom,
        flag: SymbolFlags,
        scope_id: ScopeId,
    ) -> SymbolId {
        _ = self.spans.push(span);
        _ = self.names.push(name);
        _ = self.flags.push(flag);
        _ = self.scope_ids.push(scope_id);
        self.resolved_references.push(vec![])
    }

    pub fn create_reference(&mut self, _span: Span, name: Atom) -> ReferenceId {
        self.references.push(Reference::new(name))
    }

    pub fn get_reference(&self, reference_id: ReferenceId) -> &Reference {
        &self.references[reference_id]
    }

    pub fn is_global_reference(&self, reference_id: ReferenceId) -> bool {
        self.references[reference_id].symbol_id.is_none()
    }

    pub fn get_resolved_references(&self, symbol_id: SymbolId) -> &Vec<ReferenceId> {
        &self.resolved_references[symbol_id]
    }
}
