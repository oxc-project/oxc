//! Counter to estimate counts of nodes, scopes, symbols and references.
//!
//! These counts can be used to pre-allocate sufficient capacity in `AstNodes`,
//! `ScopeTree`, and `SymbolTable`, to store data for all these items.
//! If sufficient capacity is not reserved in advance, these structures can grow and reallocate
//! during their construction, which involves copying large chunks of memory.
//! This produces a large performance cost - around 30% on our benchmarks for large source files.
//!
//! `Counts` has 2 implementations.
//! * `standard` - preferred version, for 64-bit platforms with virtual memory.
//! * `visitor` - fallback version, for 32-bit platforms, and platforms with no virtual memory (e.g. WASM).
//!
//! Please see docs in each module for the differences between them.

#[cfg(all(target_pointer_width = "64", not(target_arch = "wasm32")))]
mod standard;
#[cfg(all(target_pointer_width = "64", not(target_arch = "wasm32")))]
pub use standard::Counts;

#[cfg(not(all(target_pointer_width = "64", not(target_arch = "wasm32"))))]
mod fallback;
#[cfg(not(all(target_pointer_width = "64", not(target_arch = "wasm32"))))]
pub use fallback::Counts;

/// Macro to assert that `left <= right`
macro_rules! assert_le {
    ($left:expr, $right:expr, $($msg_args:tt)+) => {
        match (&$left, &$right) {
            (left, right) => if !(left <= right) {
                panic!(
                    "assertion failed: `(left <= right)`\n  left: `{:?}`,\n right: `{:?}`: {}",
                    left, right,
                    ::std::format_args!($($msg_args)+),
                );
            }
        }
    };

    ($left:expr, $right:expr) => {
        match (&$left, &$right) {
            (left, right) => if !(left <= right) {
                panic!(
                    "assertion failed: `(left <= right)`\n  left: `{:?}`,\n right: `{:?}`",
                    left, right,
                );
            }
        }
    };

    ($lhs:expr, $rhs:expr,) => {
        assert_le!($lhs, $rhs);
    };
}
use assert_le;
