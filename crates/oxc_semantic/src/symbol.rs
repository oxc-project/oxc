use oxc_index::IndexVec;
use oxc_span::{Atom, Span};
pub use oxc_syntax::{
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};

use crate::{
    node::AstNodeId,
    reference::{Reference, ReferenceId},
};

/// Symbol Table
///
/// `SoA` (Struct of Arrays) for memory efficiency.
#[derive(Debug, Default)]
pub struct SymbolTable {
    pub spans: IndexVec<SymbolId, Span>,
    pub names: IndexVec<SymbolId, Atom>,
    pub flags: IndexVec<SymbolId, SymbolFlags>,
    pub scope_ids: IndexVec<SymbolId, ScopeId>,
    /// Pointer to the AST Node where this symbol is declared
    pub declarations: IndexVec<SymbolId, AstNodeId>,
    pub resolved_references: IndexVec<SymbolId, Vec<ReferenceId>>,
    pub references: IndexVec<ReferenceId, Reference>,
}

impl SymbolTable {
    pub fn len(&self) -> usize {
        self.spans.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = SymbolId> + '_ {
        self.spans.iter_enumerated().map(|(symbol_id, _)| symbol_id)
    }

    pub fn get_span(&self, symbol_id: SymbolId) -> Span {
        self.spans[symbol_id]
    }

    pub fn get_name(&self, symbol_id: SymbolId) -> &Atom {
        &self.names[symbol_id]
    }

    pub fn set_name(&mut self, symbol_id: SymbolId, name: Atom) {
        self.names[symbol_id] = name;
    }

    pub fn get_flag(&self, symbol_id: SymbolId) -> SymbolFlags {
        self.flags[symbol_id]
    }
    
    pub fn set_flag(&mut self, symbol_id: SymbolId, includes: SymbolFlags) {
        self.flags[symbol_id] |= includes;
    }

    pub fn get_scope_id(&self, symbol_id: SymbolId) -> ScopeId {
        self.scope_ids[symbol_id]
    }

    pub fn get_declaration(&self, symbol_id: SymbolId) -> AstNodeId {
        self.declarations[symbol_id]
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

    pub fn add_declaration(&mut self, node_id: AstNodeId) {
        self.declarations.push(node_id);
    }

    pub fn create_reference(&mut self, reference: Reference) -> ReferenceId {
        self.references.push(reference)
    }

    pub fn get_reference(&self, reference_id: ReferenceId) -> &Reference {
        &self.references[reference_id]
    }

    pub fn is_global_reference(&self, reference_id: ReferenceId) -> bool {
        self.references[reference_id].symbol_id().is_none()
    }

    pub fn get_resolved_reference_ids(&self, symbol_id: SymbolId) -> &Vec<ReferenceId> {
        &self.resolved_references[symbol_id]
    }

    pub fn get_resolved_references(
        &self,
        symbol_id: SymbolId,
    ) -> impl Iterator<Item = &Reference> + '_ {
        self.resolved_references[symbol_id]
            .iter()
            .map(|reference_id| &self.references[*reference_id])
    }
}
