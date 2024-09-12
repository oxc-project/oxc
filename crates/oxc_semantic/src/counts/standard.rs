//! Counter to estimate counts of nodes, scopes, symbols and references.
//!
//! Estimate counts based on size of source text.
//!
//! These will almost always be a large over-estimate, but will never be an under-estimate.
//! Not under-estimating is the most important thing, as the `Vec`s in `AstNodes`, `ScopeTree`
//! and `SymbolTable` will not need to resize during building `Semantic`, which avoids expensive
//! memory copying.
//!
//! This implementation of `Counts` can be used on 64-bit platforms with virtual memory, where it's
//! not a big problem to reserve excessively large blocks of virtual memory, because:
//! 1. The memory space is so large that it's almost impossible to exhaust.
//! 2. Allocating memory only consumes virtual memory, not physical memory.
//!
//! Note: Ideally, we'd shrink the allocations to fit once the actual size required is known.
//! But found that shrinking caused memory to be reallocated, which had a large perf cost
//! (~10% on semantic benchmarks).

use std::cmp::max;

use oxc_ast::ast::Program;

use super::assert_le;

#[derive(Default, Debug)]
pub struct Counts {
    pub nodes: u32,
    pub scopes: u32,
    pub symbols: u32,
    pub references: u32,
}

impl Counts {
    /// Calculate counts as probable over-estimates based on size of source text
    pub fn count(_program: &Program, source_text: &str) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let source_len = source_text.len() as u32;

        // Calculate maximum number of nodes, scopes, symbols and references that's possible
        // for given length of source code.
        // These will almost always be a large over-estimate, but will never be an under-estimate.

        // The most node-intensive code is:
        // ``      = 0 bytes, 1 nodes
        // `x`     = 1 bytes, 3 nodes
        // `x=x`   = 3 bytes, 7 nodes
        // `x=x=x` = 5 bytes, 11 nodes
        #[allow(clippy::cast_lossless)]
        let nodes = u32::try_from(source_len as u64 * 2 + 1).unwrap_or(u32::MAX);

        // The most scope-intensive code is:
        // ``       = 0 bytes, 1 scopes
        // `{}`     = 2 bytes, 2 scopes
        // `{{}}`   = 4 bytes, 3 scopes
        // `{{{}}}` = 6 bytes, 4 scopes
        let scopes = source_len / 2 + 1;

        // The most symbol-intensive code is:
        // ``         = 0 bytes, 0 symbols
        // `a=>0`     = 4 bytes, 1 symbols
        // `(a,a)=>0` = 8 bytes, 2 symbols
        // `var a`    = 5 bytes, 1 symbols
        // `var a,a`  = 7 bytes, 2 symbols
        let symbols = max(source_len / 2, 1) - 1;

        // The most reference-intensive code is:
        // `a`       = 1 bytes, 1 references
        // `a,a`     = 3 bytes, 2 references
        // `a,a,a`   = 5 bytes, 3 references
        let references = source_len / 2 + 1;

        Self { nodes, scopes, symbols, references }
    }

    /// Assert that estimated counts were not an under-estimate
    #[cfg_attr(not(debug_assertions), expect(dead_code))]
    pub fn assert_accurate(actual: &Self, estimated: &Self) {
        assert_le!(actual.nodes, estimated.nodes, "nodes count mismatch");
        assert_le!(actual.scopes, estimated.scopes, "scopes count mismatch");
        assert_le!(actual.symbols, estimated.symbols, "symbols count mismatch");
        assert_le!(actual.references, estimated.references, "references count mismatch");
    }
}
