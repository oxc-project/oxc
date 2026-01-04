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
    /// Number of AST nodes. May be overcounted.
    pub nodes: u32,
    /// Number of scopes. May be overcounted since we increment for any node that *may* create a scope.
    pub scopes: u32,
    /// Number of symbols. May be overcounted.
    pub symbols: u32,
    /// Number of references. May be overcounted.
    pub references: u32,
}

impl Stats {
    /// Create new [`Stats`] from specified counts.
    pub fn new(nodes: u32, scopes: u32, symbols: u32, references: u32) -> Self {
        Stats { nodes, scopes, symbols, references }
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

    /// Assert that estimated [`Stats`] match actual.
    ///
    /// # Panics
    /// Panics if stats are not accurate.
    pub fn assert_accurate(self, actual: Self) {
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
