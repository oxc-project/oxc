//! Statistics about AST nodes, scopes, symbols, and references.
//!
//! These counts can be used to pre-allocate sufficient capacity in `AstNodes`,
//! `ScopeTree`, and `SymbolTable` to store info for all these items.
//!
//! The `Stats` struct is shared between the parser (which collects the counts)
//! and the semantic analyzer (which uses them for pre-allocation).

#[cfg(debug_assertions)]
use std::cell::Cell;

#[cfg(debug_assertions)]
use oxc_ast::{
    AstKind,
    ast::{BindingIdentifier, IdentifierReference, Program, TSEnumMemberName},
};
#[cfg(debug_assertions)]
use oxc_ast_visit::{Visit, walk::walk_ts_enum_member_name};
#[cfg(debug_assertions)]
use oxc_syntax::scope::{ScopeFlags, ScopeId};

/// Macro to assert that `left >= right`
macro_rules! assert_ge {
    ($left:expr, $right:expr, $($msg_args:tt)+) => {
        match (&$left, &$right) {
            (left, right) => if !(left >= right) {
                panic!(
                    "assertion failed: `(left >= right)`\n  left: `{:?}`,\n right: `{:?}`\n  {}",
                    left, right,
                    ::std::format_args!($($msg_args)+),
                );
            }
        }
    };

    ($left:expr, $right:expr) => {
        match (&$left, &$right) {
            (left, right) => if !(left >= right) {
                panic!(
                    "assertion failed: `(left >= right)`\n  left: `{:?}`,\n right: `{:?}`",
                    left, right,
                );
            }
        }
    };

    ($lhs:expr, $rhs:expr,) => {
        assert_le!($lhs, $rhs);
    };
}

/// Statistics about AST nodes, scopes, symbols, and references.
///
/// These counts can be used to pre-allocate sufficient capacity in `AstNodes`,
/// `ScopeTree`, and `SymbolTable` to store info for all these items.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Stats {
    /// Number of AST nodes.
    pub nodes: u32,
    /// Number of scopes (may be an overestimate).
    pub scopes: u32,
    /// Number of symbols (may be an overestimate due to multiple declarations
    /// for a single symbol, e.g., `var x; var x;`).
    pub symbols: u32,
    /// Number of identifier references.
    pub references: u32,
}

impl Stats {
    /// Create new [`Stats`] from specified counts.
    #[inline]
    pub const fn new(nodes: u32, scopes: u32, symbols: u32, references: u32) -> Self {
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
    #[cfg(debug_assertions)]
    pub fn count_accurate(program: &Program) -> Self {
        let mut counter = Counter::default();
        counter.visit_program(program);
        counter.stats
    }

    /// Increase scope, symbol, and reference counts by provided `excess`.
    ///
    /// `excess` is provided as a fraction.
    /// e.g. to over-allocate by 20%, pass `0.2` as `excess`.
    #[must_use]
    pub fn increase_by(mut self, excess: f64) -> Self {
        let factor = excess + 1.0;
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_lossless)]
        let increase = |n: u32| (n as f64 * factor) as u32;

        self.scopes = increase(self.scopes);
        self.symbols = increase(self.symbols);
        self.references = increase(self.references);

        self
    }

    /// Increment the node count by 1.
    #[inline]
    pub fn add_node(&mut self) {
        self.nodes += 1;
    }

    /// Increment the scope count by 1.
    #[inline]
    pub fn add_scope(&mut self) {
        self.scopes += 1;
    }

    /// Increment the symbol count by 1.
    #[inline]
    pub fn add_symbol(&mut self) {
        self.symbols += 1;
    }

    /// Increment the reference count by 1.
    #[inline]
    pub fn add_reference(&mut self) {
        self.references += 1;
    }

    /// Decrement the node count by 1.
    ///
    /// Used when a speculatively parsed node is discarded without using a checkpoint.
    /// For example, when parsing `{ type foo }` in an import, the `type` keyword is first
    /// parsed as an IdentifierName, but if it's determined to be a type modifier, that
    /// node is discarded and this method should be called to correct the count.
    #[inline]
    pub fn subtract_node(&mut self) {
        debug_assert!(self.nodes > 0, "Cannot subtract node when count is 0");
        self.nodes -= 1;
    }

    /// Assert that estimated [`Stats`] match actual.
    ///
    /// # Panics
    /// Panics if stats are not accurate.
    #[cfg(debug_assertions)]
    pub fn assert_accurate(self, actual: Self, source: &str) {
        // if self != actual {
        //     eprintln!("Source: {source}");
        //     eprintln!(
        //         "Parser stats: nodes={}, scopes={}, symbols={}, references={}",
        //         self.nodes, self.scopes, self.symbols, self.references
        //     );
        //     eprintln!(
        //         "Actual stats: nodes={}, scopes={}, symbols={}, references={}",
        //         actual.nodes, actual.scopes, actual.symbols, actual.references
        //     );
        // }
        assert_ge!(self.nodes, actual.nodes, "nodes count mismatch");
        assert_ge!(self.scopes, actual.scopes, "scopes count mismatch");
        assert_ge!(self.references, actual.references, "references count mismatch");
        // `Counter` may overestimate number of symbols, because multiple `BindingIdentifier`s
        // can result in only a single symbol.
        // e.g. `var x; var x;` = 2 x `BindingIdentifier` but 1 x symbol.
        // This is not a big problem - allocating a `Vec` with excess capacity is cheap.
        // It's allocating with *not enough* capacity which is costly, as then the `Vec`
        // will grow and reallocate.
        assert_ge!(self.symbols, actual.symbols, "symbols count mismatch");
    }
}

#[cfg(debug_assertions)]
#[derive(Default)]
struct Counter {
    stats: Stats,
}

/// Visitor to count nodes, scopes, symbols and references in AST
#[cfg(debug_assertions)]
impl<'a> Visit<'a> for Counter {
    #[inline]
    fn enter_node(&mut self, kind: AstKind<'a>) {
        self.stats.nodes += 1;
        let _ = kind; // silence unused variable warning
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
        self.stats.symbols += 1;
        walk_ts_enum_member_name(self, it);
    }
}
