use std::cell::Cell;

use oxc_ast::{
    ast::{
        BindingIdentifier, IdentifierReference, Program, TSEnumMemberName, TSModuleDeclarationName,
    },
    visit::walk::{walk_ts_enum_member_name, walk_ts_module_declaration_name},
    AstKind, Visit,
};
use oxc_syntax::scope::{ScopeFlags, ScopeId};

/// Statistics about data held in [`Semantic`].
///
/// Comprises number of AST nodes, scopes, symbols, and references.
///
/// These counts can be used to pre-allocate sufficient capacity in `AstNodes`,
/// `ScopeTree`, and `SymbolTable` to store info for all these items.
///
/// * Obtain `Stats` from an existing [`Semantic`] with [`Semantic::stats`].
/// * Use [`Stats::count`] to visit AST and obtain accurate counts.
///
/// # Example
/// ```
/// use oxc_ast::ast::Program;
/// use oxc_semantic::{Semantic, Stats};
///
/// fn print_stats_from_semantic(semantic: &Semantic) {
///     dbg!(semantic.stats());
/// }
///
/// fn print_stats_from_ast(program: &Program) {
///     dbg!(Stats::count(program));
/// }
/// ```
///
/// [`Semantic`]: super::Semantic
/// [`Semantic::stats`]: super::Semantic::stats
#[derive(Default, Debug)]
pub struct Stats {
    pub nodes: u32,
    pub scopes: u32,
    pub symbols: u32,
    pub references: u32,
}

impl Stats {
    /// Create new [`Stats`] from specified counts.
    pub fn new(nodes: u32, scopes: u32, symbols: u32, references: u32) -> Self {
        Stats { nodes, scopes, symbols, references }
    }

    /// Gather [`Stats`] by visiting AST and counting nodes, scopes, symbols, and references.
    ///
    /// Nodes, scopes and references counts will be exactly accurate.
    /// Symbols count may be an over-estimate if there are multiple declarations for a single symbol.
    /// e.g. `var x; var x;` will produce a count of 2 symbols, but this is actually only 1 symbol.
    ///
    /// If semantic analysis has already been run on AST, prefer getting counts with [`Semantic::stats`].
    /// They will be 100% accurate, and very cheap to obtain, whereas this method performs a complete
    /// AST traversal.
    ///
    /// [`Semantic::stats`]: super::Semantic::stats
    pub fn count(program: &Program) -> Self {
        let mut counter = Counter::default();
        counter.visit_program(program);
        counter.stats
    }

    /// Check that estimated [`Stats`] match actual.
    ///
    /// # Panics
    /// Panics if stats are not accurate.
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

#[derive(Default)]
struct Counter {
    stats: Stats,
}

/// Visitor to count nodes, scopes, symbols and references in AST
impl<'a> Visit<'a> for Counter {
    #[inline]
    fn enter_node(&mut self, _: AstKind<'a>) {
        self.stats.nodes += 1;
    }

    #[inline]
    fn enter_scope(&mut self, _: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        self.stats.scopes += 1;
    }

    #[inline]
    fn visit_binding_identifier(&mut self, _: &BindingIdentifier<'a>) {
        self.stats.nodes += 1;
        self.stats.symbols += 1;
    }

    #[inline]
    fn visit_identifier_reference(&mut self, _: &IdentifierReference<'a>) {
        self.stats.nodes += 1;
        self.stats.references += 1;
    }

    #[inline]
    fn visit_ts_enum_member_name(&mut self, it: &TSEnumMemberName<'a>) {
        if !it.is_expression() {
            self.stats.symbols += 1;
        }
        walk_ts_enum_member_name(self, it);
    }

    #[inline]
    fn visit_ts_module_declaration_name(&mut self, it: &TSModuleDeclarationName<'a>) {
        self.stats.symbols += 1;
        walk_ts_module_declaration_name(self, it);
    }
}
