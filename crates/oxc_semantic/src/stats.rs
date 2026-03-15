use std::cell::Cell;

use oxc_ast::{
    AstBuilderStats, AstKind,
    ast::{BindingIdentifier, IdentifierReference, Program, TSEnumMemberName},
};
use oxc_ast_visit::{Visit, walk::walk_ts_enum_member_name};
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
#[derive(Clone, Copy, Default, Debug)]
pub struct Stats {
    /// Number of AST nodes.
    pub nodes: u32,
    /// Number of lexical scopes.
    pub scopes: u32,
    /// Number of semantic symbols.
    pub symbols: u32,
    /// Number of identifier references.
    pub references: u32,
}

impl From<&AstBuilderStats> for Stats {
    #[expect(clippy::cast_possible_truncation)]
    fn from(stats: &AstBuilderStats) -> Self {
        Stats {
            nodes: stats.nodes() as u32,
            scopes: stats.scopes() as u32,
            symbols: stats.symbols() as u32,
            references: stats.references() as u32,
        }
    }
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

    /// Assert that estimated [`Stats`] are no smaller than `actual`.
    ///
    /// Estimated counts may be larger than actual counts (e.g. symbols may be overcounted
    /// when there are redeclarations like `var x; var x;`). Overestimates are acceptable
    /// because they only result in extra allocated capacity. Underestimates are costly
    /// because they force reallocation and memory copying.
    ///
    /// # Panics
    /// Panics if any estimated count is less than the corresponding actual count.
    pub fn assert_accurate(self, actual: Self) {
        assert_ge!(self.nodes, actual.nodes, "nodes count mismatch");
        assert_ge!(self.scopes, actual.scopes, "scopes count mismatch");
        assert_ge!(self.references, actual.references, "references count mismatch");
        assert_ge!(self.symbols, actual.symbols, "symbols count mismatch");
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
        self.stats.symbols += 1;
        walk_ts_enum_member_name(self, it);
    }
}
