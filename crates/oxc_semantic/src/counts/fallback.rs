//! Counter to estimate counts of nodes, scopes, symbols and references.
//!
//! Produces an accurate count, by visiting AST and counting these items.
//!
//! Doing a full traverse of AST has a sizeable performance cost, but is necessary on platforms
//! which are 32-bit or do not have virtual memory (e.g. WASM) and so the "standard" version of
//! `Counts` is not suitable.

use std::cell::Cell;

use oxc_ast::{
    ast::{
        BindingIdentifier, IdentifierReference, Program, TSEnumMemberName, TSModuleDeclarationName,
    },
    visit::walk::{walk_ts_enum_member_name, walk_ts_module_declaration_name},
    AstKind, Visit,
};
use oxc_syntax::scope::{ScopeFlags, ScopeId};

use super::assert_le;

#[derive(Default, Debug)]
pub struct Counts {
    pub nodes: u32,
    pub scopes: u32,
    pub symbols: u32,
    pub references: u32,
}

impl Counts {
    /// Calculate counts by visiting AST
    pub fn count(program: &Program, _source_text: &str) -> Self {
        let mut counts = Self::default();
        counts.visit_program(program);
        counts
    }

    /// Assert that estimated counts were accurate
    #[cfg_attr(not(debug_assertions), expect(dead_code))]
    pub fn assert_accurate(actual: &Self, estimated: &Self) {
        assert_eq!(actual.nodes, estimated.nodes, "nodes count mismatch");
        assert_eq!(actual.scopes, estimated.scopes, "scopes count mismatch");
        // `Counts` may overestimate number of symbols, because multiple `BindingIdentifier`s
        // can result in only a single symbol.
        // e.g. `var x; var x;` = 2 x `BindingIdentifier` but 1 x symbol.
        // This is not a big problem - allocating a `Vec` with excess capacity is fairly cheap.
        // It's allocating with *not enough* capacity which is costly, as then the `Vec`
        // will grow and reallocate.
        assert_le!(actual.symbols, estimated.symbols, "symbols count mismatch");
        assert_eq!(actual.references, estimated.references, "references count mismatch");
    }
}

impl<'a> Visit<'a> for Counts {
    #[inline]
    fn enter_node(&mut self, _: AstKind<'a>) {
        self.nodes += 1;
    }

    #[inline]
    fn enter_scope(&mut self, _: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        self.scopes += 1;
    }

    #[inline]
    fn visit_binding_identifier(&mut self, _: &BindingIdentifier<'a>) {
        self.nodes += 1;
        self.symbols += 1;
    }

    #[inline]
    fn visit_identifier_reference(&mut self, _: &IdentifierReference<'a>) {
        self.nodes += 1;
        self.references += 1;
    }

    #[inline]
    fn visit_ts_enum_member_name(&mut self, it: &TSEnumMemberName<'a>) {
        if !it.is_expression() {
            self.symbols += 1;
        }
        walk_ts_enum_member_name(self, it);
    }

    #[inline]
    fn visit_ts_module_declaration_name(&mut self, it: &TSModuleDeclarationName<'a>) {
        self.symbols += 1;
        walk_ts_module_declaration_name(self, it);
    }
}
