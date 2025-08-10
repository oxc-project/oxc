//! HashMap optimization utilities for common usage patterns in oxc
//!
//! This module provides helper functions and optimized constructors for HashMap
//! creation patterns commonly used throughout the oxc codebase. These utilities
//! help reduce allocation overhead by providing reasonable capacity hints based
//! on typical usage patterns.

use crate::{Allocator, HashMap};

/// Capacity estimation constants based on empirical analysis of oxc usage patterns
pub struct CapacityHints;

impl CapacityHints {
    /// Typical number of unique files in a linting session
    pub const DIAGNOSTIC_FILES: usize = 32;

    /// Average number of diagnostics per file
    pub const DIAGNOSTICS_PER_FILE: usize = 8;

    /// Typical number of configuration paths
    pub const CONFIG_PATHS: usize = 16;

    /// Average number of symbols per scope
    pub const SYMBOLS_PER_SCOPE: usize = 12;

    /// Typical number of references per symbol
    pub const REFERENCES_PER_SYMBOL: usize = 4;

    /// Small collection threshold
    pub const SMALL_COLLECTION: usize = 4;

    /// Medium collection threshold  
    pub const MEDIUM_COLLECTION: usize = 16;

    /// Large collection threshold
    pub const LARGE_COLLECTION: usize = 64;
}

/// HashMap creation utilities with optimized capacity hints
pub struct HashMapUtils;

impl HashMapUtils {
    /// Create a HashMap optimized for diagnostic grouping by filename
    ///
    /// This is commonly used in output formatters where diagnostics are grouped
    /// by their source file. Capacity is estimated based on the number of diagnostics.
    /// Note: This is for non-Drop key types. For String keys, use FxHashMap directly.
    pub fn for_diagnostic_grouping<'alloc, K, V>(
        diagnostic_count: usize,
        allocator: &'alloc Allocator,
    ) -> HashMap<'alloc, K, V> {
        // Estimate unique files as sqrt of diagnostic count, with reasonable bounds
        let estimated_files = (diagnostic_count as f64).sqrt().ceil() as usize;
        let capacity =
            estimated_files.clamp(CapacityHints::SMALL_COLLECTION, CapacityHints::DIAGNOSTIC_FILES);
        HashMap::with_capacity_in(capacity, allocator)
    }

    /// Create a HashMap optimized for configuration path storage
    ///
    /// Used for storing configuration files and their resolved paths.
    /// Note: This is for non-Drop key types. For PathBuf keys, use FxHashMap directly.
    pub fn for_config_paths<'alloc, K, V>(
        path_count: usize,
        allocator: &'alloc Allocator,
    ) -> HashMap<'alloc, K, V> {
        // Configuration maps typically have fewer entries than input paths
        let capacity =
            (path_count / 2).clamp(CapacityHints::SMALL_COLLECTION, CapacityHints::CONFIG_PATHS);
        HashMap::with_capacity_in(capacity, allocator)
    }

    /// Create a HashMap optimized for symbol table usage
    ///
    /// Used in semantic analysis for symbol tables and binding maps.
    pub fn for_symbol_table<'alloc, K, V>(
        estimated_symbols: usize,
        allocator: &'alloc Allocator,
    ) -> HashMap<'alloc, K, V> {
        let capacity = estimated_symbols
            .clamp(CapacityHints::SMALL_COLLECTION, CapacityHints::SYMBOLS_PER_SCOPE);
        HashMap::with_capacity_in(capacity, allocator)
    }

    /// Create a HashMap optimized for reference tracking
    ///
    /// Used for tracking symbol references and usage patterns.
    pub fn for_references<'alloc, K, V>(
        estimated_refs: usize,
        allocator: &'alloc Allocator,
    ) -> HashMap<'alloc, K, V> {
        let capacity = estimated_refs
            .clamp(CapacityHints::SMALL_COLLECTION, CapacityHints::REFERENCES_PER_SYMBOL);
        HashMap::with_capacity_in(capacity, allocator)
    }

    /// Create a HashMap with capacity based on input size estimation
    ///
    /// Generic utility for cases where capacity can be estimated from input size.
    pub fn with_estimated_capacity<'alloc, K, V>(
        input_size: usize,
        ratio: f64,
        min_capacity: usize,
        max_capacity: usize,
        allocator: &'alloc Allocator,
    ) -> HashMap<'alloc, K, V> {
        let estimated = (input_size as f64 * ratio).ceil() as usize;
        let capacity = estimated.clamp(min_capacity, max_capacity);
        HashMap::with_capacity_in(capacity, allocator)
    }

    /// Create a small HashMap optimized for few entries
    ///
    /// Use when you expect only a few entries (< 8).
    pub fn small<'alloc, K, V>(allocator: &'alloc Allocator) -> HashMap<'alloc, K, V> {
        HashMap::with_capacity_in(CapacityHints::SMALL_COLLECTION, allocator)
    }

    /// Create a medium HashMap optimized for moderate number of entries
    ///
    /// Use when you expect a moderate number of entries (8-32).
    pub fn medium<'alloc, K, V>(allocator: &'alloc Allocator) -> HashMap<'alloc, K, V> {
        HashMap::with_capacity_in(CapacityHints::MEDIUM_COLLECTION, allocator)
    }

    /// Create a large HashMap optimized for many entries
    ///
    /// Use when you expect many entries (32+).
    pub fn large<'alloc, K, V>(allocator: &'alloc Allocator) -> HashMap<'alloc, K, V> {
        HashMap::with_capacity_in(CapacityHints::LARGE_COLLECTION, allocator)
    }
}

/// Extension trait for FxHashMap to add capacity optimization utilities
///
/// This trait provides optimized constructors for rustc_hash::FxHashMap
/// used throughout the codebase outside of the allocator context.
pub trait FxHashMapExt<K, V> {
    /// Create a FxHashMap optimized for diagnostic grouping
    fn for_diagnostic_grouping(diagnostic_count: usize) -> Self;

    /// Create a FxHashMap optimized for configuration storage
    fn for_config_storage(config_count: usize) -> Self;

    /// Create a FxHashMap with estimated capacity
    fn with_estimated_capacity(input_size: usize, ratio: f64) -> Self;

    /// Create a small FxHashMap
    fn small() -> Self;

    /// Create a medium FxHashMap
    fn medium() -> Self;

    /// Create a large FxHashMap
    fn large() -> Self;
}

impl<K, V> FxHashMapExt<K, V> for rustc_hash::FxHashMap<K, V> {
    fn for_diagnostic_grouping(diagnostic_count: usize) -> Self {
        let estimated_files = (diagnostic_count as f64).sqrt().ceil() as usize;
        let capacity =
            estimated_files.clamp(CapacityHints::SMALL_COLLECTION, CapacityHints::DIAGNOSTIC_FILES);
        Self::with_capacity_and_hasher(capacity, Default::default())
    }

    fn for_config_storage(config_count: usize) -> Self {
        let capacity =
            (config_count / 2).clamp(CapacityHints::SMALL_COLLECTION, CapacityHints::CONFIG_PATHS);
        Self::with_capacity_and_hasher(capacity, Default::default())
    }

    fn with_estimated_capacity(input_size: usize, ratio: f64) -> Self {
        let capacity = (input_size as f64 * ratio).ceil() as usize;
        Self::with_capacity_and_hasher(capacity, Default::default())
    }

    fn small() -> Self {
        Self::with_capacity_and_hasher(CapacityHints::SMALL_COLLECTION, Default::default())
    }

    fn medium() -> Self {
        Self::with_capacity_and_hasher(CapacityHints::MEDIUM_COLLECTION, Default::default())
    }

    fn large() -> Self {
        Self::with_capacity_and_hasher(CapacityHints::LARGE_COLLECTION, Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Allocator;

    #[test]
    fn test_diagnostic_grouping_capacity() {
        let allocator = Allocator::default();

        // Small number of diagnostics
        let map = HashMapUtils::for_diagnostic_grouping::<u32, u32>(4, &allocator);
        assert!(map.capacity() >= CapacityHints::SMALL_COLLECTION);

        // Large number of diagnostics
        let map = HashMapUtils::for_diagnostic_grouping::<u32, u32>(1000, &allocator);
        assert!(map.capacity() >= CapacityHints::DIAGNOSTIC_FILES);
    }

    #[test]
    fn test_config_paths_capacity() {
        let allocator = Allocator::default();

        let map = HashMapUtils::for_config_paths::<u32, u32>(20, &allocator);
        assert!(map.capacity() >= CapacityHints::SMALL_COLLECTION);
        assert!(map.capacity() <= CapacityHints::CONFIG_PATHS);
    }

    #[test]
    fn test_fx_hashmap_ext() {
        let map = rustc_hash::FxHashMap::<u32, u32>::for_diagnostic_grouping(100);
        assert!(map.capacity() > 0);

        let small_map = rustc_hash::FxHashMap::<u32, u32>::small();
        assert!(small_map.capacity() >= CapacityHints::SMALL_COLLECTION);
    }
}
