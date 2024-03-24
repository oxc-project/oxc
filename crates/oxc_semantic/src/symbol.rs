// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use oxc_ast::ast::Expression;
use oxc_index::IndexVec;
use oxc_span::{Atom, CompactStr, Span};
pub use oxc_syntax::{
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};

use crate::{
    node::AstNodeId,
    reference::{Reference, ReferenceId},
};

#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type IndexVec<I, T> = Array<T>;
"#;

/// Symbol Table
///
/// `SoA` (Struct of Arrays) for memory efficiency.
#[derive(Debug, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify), serde(rename_all = "camelCase"))]
pub struct SymbolTable {
    pub spans: IndexVec<SymbolId, Span>,
    pub names: IndexVec<SymbolId, CompactStr>,
    pub flags: IndexVec<SymbolId, SymbolFlags>,
    pub scope_ids: IndexVec<SymbolId, ScopeId>,
    /// Pointer to the AST Node where this symbol is declared
    pub declarations: IndexVec<SymbolId, AstNodeId>,
    pub resolved_references: IndexVec<SymbolId, Vec<ReferenceId>>,
    pub references: IndexVec<ReferenceId, Reference>,
    pub redeclare_variables: IndexVec<SymbolId, Vec<Span>>,
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

    pub fn iter_rev(&self) -> impl Iterator<Item = SymbolId> + '_ {
        self.spans.iter_enumerated().rev().map(|(symbol_id, _)| symbol_id)
    }

    pub fn get_symbol_id_from_span(&self, span: &Span) -> Option<SymbolId> {
        self.spans
            .iter_enumerated()
            .find_map(|(symbol, inner_span)| if inner_span == span { Some(symbol) } else { None })
    }

    pub fn get_symbol_id_from_name(&self, name: &str) -> Option<SymbolId> {
        self.names.iter_enumerated().find_map(|(symbol, inner_name)| {
            if inner_name.as_str() == name {
                Some(symbol)
            } else {
                None
            }
        })
    }

    pub fn get_span(&self, symbol_id: SymbolId) -> Span {
        self.spans[symbol_id]
    }

    pub fn get_name(&self, symbol_id: SymbolId) -> &str {
        &self.names[symbol_id]
    }

    pub fn set_name(&mut self, symbol_id: SymbolId, name: CompactStr) {
        self.names[symbol_id] = name;
    }

    pub fn get_flag(&self, symbol_id: SymbolId) -> SymbolFlags {
        self.flags[symbol_id]
    }

    pub fn get_redeclare_variables(&self, symbol_id: SymbolId) -> &Vec<Span> {
        &self.redeclare_variables[symbol_id]
    }

    pub fn union_flag(&mut self, symbol_id: SymbolId, includes: SymbolFlags) {
        self.flags[symbol_id] |= includes;
    }

    pub fn get_scope_id(&self, symbol_id: SymbolId) -> ScopeId {
        self.scope_ids[symbol_id]
    }

    pub fn get_scope_id_from_span(&self, span: &Span) -> Option<ScopeId> {
        self.get_symbol_id_from_span(span).map(|symbol_id| self.get_scope_id(symbol_id))
    }

    pub fn get_scope_id_from_name(&self, name: &Atom) -> Option<ScopeId> {
        self.get_symbol_id_from_name(name).map(|symbol_id| self.get_scope_id(symbol_id))
    }

    pub fn get_declaration(&self, symbol_id: SymbolId) -> AstNodeId {
        self.declarations[symbol_id]
    }

    pub fn create_symbol(
        &mut self,
        span: Span,
        name: &str,
        flag: SymbolFlags,
        scope_id: ScopeId,
    ) -> SymbolId {
        _ = self.spans.push(span);
        _ = self.names.push(CompactStr::from(name));
        _ = self.flags.push(flag);
        _ = self.scope_ids.push(scope_id);
        _ = self.resolved_references.push(vec![]);
        self.redeclare_variables.push(vec![])
    }

    pub fn add_declaration(&mut self, node_id: AstNodeId) {
        self.declarations.push(node_id);
    }

    pub fn add_redeclare_variable(&mut self, symbol_id: SymbolId, span: Span) {
        self.redeclare_variables[symbol_id].push(span);
    }

    pub fn create_reference(&mut self, reference: Reference) -> ReferenceId {
        self.references.push(reference)
    }

    pub fn get_reference(&self, reference_id: ReferenceId) -> &Reference {
        &self.references[reference_id]
    }

    pub fn has_binding(&self, reference_id: ReferenceId) -> bool {
        self.references[reference_id].symbol_id().is_some()
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
