//! Visitor to count nodes, scopes, symbols and references in AST.
//! These counts can be used to pre-allocate sufficient capacity in `AstNodes`,
//! `ScopeTree`, and `SymbolTable` to store info for all these items.

use std::cell::Cell;

use oxc_ast::{
    ast::{
        BindingIdentifier, IdentifierReference, Program, TSEnumMemberName, TSModuleDeclarationName,
    },
    visit::walk::{walk_ts_enum_member_name, walk_ts_module_declaration_name},
    AstKind, Visit,
};
use oxc_syntax::scope::{ScopeFlags, ScopeId};

#[derive(Default, Debug)]
pub(crate) struct Stats {
    pub nodes: usize,
    pub scopes: usize,
    pub symbols: usize,
    pub references: usize,
}

impl Stats {
    pub fn count(program: &Program) -> Self {
        let mut stats = Stats::default();
        stats.visit_program(program);
        stats
    }

    #[cfg_attr(not(debug_assertions), expect(dead_code))]
    pub fn assert_accurate(actual: &Self, estimated: &Self) {
        assert_eq!(actual.nodes, estimated.nodes, "nodes count mismatch");
        assert_eq!(actual.scopes, estimated.scopes, "scopes count mismatch");
        assert_eq!(actual.references, estimated.references, "references count mismatch");
        // `Stats` may overestimate number of symbols, because multiple `BindingIdentifier`s
        // can result in only a single symbol.
        // e.g. `var x; var x;` = 2 x `BindingIdentifier` but 1 x symbol.
        // This is not a big problem - allocating a `Vec` with excess capacity is cheap.
        // It's allocating with *not enough* capacity which is costly, as then the `Vec`
        // will grow and reallocate.
        assert!(
            actual.symbols <= estimated.symbols,
            "symbols count mismatch {} <= {}",
            actual.symbols,
            estimated.symbols
        );
    }
}

impl<'a> Visit<'a> for Stats {
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
