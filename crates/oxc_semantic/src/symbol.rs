#![allow(non_snake_case)] // Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`

use oxc_ast::ast::Expression;
use oxc_index::IndexVec;
use oxc_span::{CompactStr, Span};
pub use oxc_syntax::{
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};
#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

use crate::{
    node::AstNodeId,
    reference::{Reference, ReferenceId},
};

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type IndexVec<I, T> = Array<T>;
"#;

#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify), serde(rename_all = "camelCase"))]
pub struct Symbol {
    pub span: Span,
    pub name: CompactStr,
    pub flag: SymbolFlags,
    pub scope_id: ScopeId,
    /// Pointer to the AST Node where this symbol is declared
    pub declaration: AstNodeId,
    pub resolved_references: Vec<ReferenceId>,
    pub redeclare_variables: Vec<Span>,
}

impl Symbol {
    pub fn new(span: Span, name: CompactStr, flag: SymbolFlags, scope_id: ScopeId) -> Self {
        Self {
            span,
            name,
            flag,
            scope_id,
            declaration: AstNodeId::dummy(),
            resolved_references: Vec::new(),
            redeclare_variables: Vec::new(),
        }
    }

    pub fn extend_reference_ids(&mut self, reference_ids: Vec<ReferenceId>) {
        self.resolved_references.extend(reference_ids);
    }

    pub fn add_redeclare_variable(&mut self, span: Span) {
        self.redeclare_variables.push(span);
    }
}

/// Symbol Table
///
/// `SoA` (Struct of Arrays) for memory efficiency.
#[derive(Debug, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify), serde(rename_all = "camelCase"))]
pub struct SymbolTable {
    symbols: IndexVec<SymbolId, Symbol>,
    pub references: IndexVec<ReferenceId, Reference>,
}

impl SymbolTable {
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = SymbolId> + '_ {
        self.symbols.iter_enumerated().map(|(symbol_id, _)| symbol_id)
    }

    pub fn iter_rev(&self) -> impl Iterator<Item = SymbolId> + '_ {
        self.symbols.iter_enumerated().rev().map(|(symbol_id, _)| symbol_id)
    }

    pub fn names(&self) -> impl Iterator<Item = &CompactStr> {
        self.symbols.iter().map(|symbol| &symbol.name)
    }

    pub fn name(&self, symbol_id: SymbolId) -> &CompactStr {
        &self.symbols[symbol_id].name
    }

    pub fn get_symbol_id_from_span(&self, span: &Span) -> Option<SymbolId> {
        self.symbols.iter_enumerated().find_map(|(symbol_id, symbol)| {
            if symbol.span == *span {
                Some(symbol_id)
            } else {
                None
            }
        })
    }

    pub fn get_symbol_id_from_name(&self, name: &str) -> Option<SymbolId> {
        self.symbols.iter_enumerated().find_map(|(symbol_id, symbol)| {
            if symbol.name.as_str() == name {
                Some(symbol_id)
            } else {
                None
            }
        })
    }

    pub fn get_span(&self, symbol_id: SymbolId) -> Span {
        self.symbols[symbol_id].span
    }

    pub fn get_name(&self, symbol_id: SymbolId) -> &str {
        &self.symbols[symbol_id].name
    }

    pub fn set_name(&mut self, symbol_id: SymbolId, name: CompactStr) {
        self.symbols[symbol_id].name = name;
    }

    pub fn get_flag(&self, symbol_id: SymbolId) -> SymbolFlags {
        self.symbols[symbol_id].flag
    }

    pub fn get_redeclare_variables(&self, symbol_id: SymbolId) -> &Vec<Span> {
        &self.symbols[symbol_id].redeclare_variables
    }

    pub fn union_flag(&mut self, symbol_id: SymbolId, includes: SymbolFlags) {
        self.symbols[symbol_id].flag |= includes;
    }

    pub fn get_scope_id(&self, symbol_id: SymbolId) -> ScopeId {
        self.symbols[symbol_id].scope_id
    }

    pub fn get_scope_id_from_span(&self, span: &Span) -> Option<ScopeId> {
        self.get_symbol_id_from_span(span).map(|symbol_id| self.get_scope_id(symbol_id))
    }

    pub fn get_scope_id_from_name(&self, name: &str) -> Option<ScopeId> {
        self.get_symbol_id_from_name(name).map(|symbol_id| self.get_scope_id(symbol_id))
    }

    pub fn get_declaration(&self, symbol_id: SymbolId) -> AstNodeId {
        self.symbols[symbol_id].declaration
    }

    pub fn create_symbol(
        &mut self,
        span: Span,
        name: CompactStr,
        flag: SymbolFlags,
        scope_id: ScopeId,
    ) -> SymbolId {
        self.symbols.push(Symbol::new(span, name, flag, scope_id))
    }

    pub fn add_declaration(&mut self, node_id: AstNodeId) {
        if let Some(symbol) = self.symbols.last_mut() {
            symbol.declaration = node_id;
        }
    }

    pub fn add_redeclare_variable(&mut self, symbol_id: SymbolId, span: Span) {
        self.symbols[symbol_id].add_redeclare_variable(span);
    }

    pub fn create_reference(&mut self, reference: Reference) -> ReferenceId {
        self.references.push(reference)
    }

    pub fn get_reference(&self, reference_id: ReferenceId) -> &Reference {
        &self.references[reference_id]
    }

    pub fn get_reference_mut(&mut self, reference_id: ReferenceId) -> &mut Reference {
        &mut self.references[reference_id]
    }

    pub fn has_binding(&self, reference_id: ReferenceId) -> bool {
        self.references[reference_id].symbol_id().is_some()
    }

    pub fn is_global_reference(&self, reference_id: ReferenceId) -> bool {
        self.references[reference_id].symbol_id().is_none()
    }

    pub fn get_resolved_reference_ids(&self, symbol_id: SymbolId) -> &Vec<ReferenceId> {
        &self.symbols[symbol_id].resolved_references
    }
    pub fn get_resolved_reference_ids_mut(&mut self, symbol_id: SymbolId) -> &mut Vec<ReferenceId> {
        &mut self.symbols[symbol_id].resolved_references
    }

    pub fn extend_reference_ids(&mut self, symbol_id: SymbolId, reference_ids: Vec<ReferenceId>) {
        self.symbols[symbol_id].extend_reference_ids(reference_ids);
    }

    pub fn get_resolved_references(
        &self,
        symbol_id: SymbolId,
    ) -> impl Iterator<Item = &Reference> + '_ {
        self.symbols[symbol_id]
            .resolved_references
            .iter()
            .map(|reference_id| &self.references[*reference_id])
    }

    /// Determine whether evaluating the specific input `node` is a consequenceless reference. ie.
    /// evaluating it won't result in potentially arbitrary code from being ran. The following are
    /// allowed and determined not to cause side effects:
    ///
    ///  - `this` expressions
    ///  - `super` expressions
    ///  - Bound identifiers
    ///
    /// Reference:
    /// <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L557>
    pub fn is_static(&self, expr: &Expression) -> bool {
        match expr {
            Expression::ThisExpression(_) | Expression::Super(_) => true,
            Expression::Identifier(ident) => {
                ident.reference_id.get().map_or(false, |reference_id| {
                    self.get_reference(reference_id).symbol_id().map_or_else(
                        || self.has_binding(reference_id),
                        |symbol_id| self.get_resolved_references(symbol_id).all(|r| !r.is_write()),
                    )
                })
            }
            _ => false,
        }
    }
}
