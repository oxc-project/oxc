//! Minifier-owned symbol data, grouped by storage lifetime and density.

use std::{collections::hash_map::Entry, num::NonZeroU32};

use rustc_hash::FxHashMap;

use oxc_allocator::{Allocator, BitSet};
use oxc_index::IndexVec;
use oxc_semantic::Scoping;
use oxc_span::SourceType;
use oxc_syntax::symbol::SymbolId;

use crate::{
    CompressOptions,
    symbol_liveness::SymbolLiveness,
    symbol_metadata::{FunctionSummary, MemberWriteEffect, PersistentSymbolMetadata},
    symbol_value::SymbolValue,
};

#[derive(Default)]
struct DeadArgumentMetadata {
    // Stored as `prefix + 1` so `None` remains a niche and this field stays 4
    // bytes. Parameter counts fit in `u32` because source spans use `u32`.
    prefix: Option<NonZeroU32>,
    missed_call: bool,
}

impl DeadArgumentMetadata {
    fn set_prefix(&mut self, prefix: usize) {
        let prefix = u32::try_from(prefix).expect("function parameter count must fit in u32");
        let encoded =
            prefix.checked_add(1).expect("function parameter count must be below u32::MAX");
        self.prefix = NonZeroU32::new(encoded);
    }

    fn prefix(&self) -> Option<usize> {
        self.prefix.map(|prefix| (prefix.get() - 1) as usize)
    }
}

/// Symbol-indexed data owned by the minifier.
///
/// The fields stay separate because they have different storage and reset
/// contracts:
/// - `values` is dense scratch rebuilt for each peephole pass;
/// - `persistent` is sparse metadata retained across passes;
/// - `liveness` is stable observability metadata plus optional graph results.
pub struct SymbolState<'a> {
    /// Per-pass scratch indexed directly by dense semantic symbol IDs.
    ///
    /// Sized once from `Scoping::symbols_len()`; no minifier pass mints new
    /// symbols, so indexed writes intentionally panic if that contract changes.
    values: IndexVec<SymbolId, Option<SymbolValue<'a>>>,
    /// Symbols seen as direct callees with arguments that may be removable.
    /// Seeded once by Normalize before the fixed-point loop.
    dead_argument_candidates: Option<BitSet<'a>>,
    /// DCE-only facts kept separate so full compression retains the compact
    /// shared persistent-metadata layout and its cache behavior.
    dead_arguments: Option<FxHashMap<SymbolId, DeadArgumentMetadata>>,
    persistent: FxHashMap<SymbolId, PersistentSymbolMetadata>,
    liveness: Option<SymbolLiveness<'a>>,
}

impl<'a> SymbolState<'a> {
    pub fn new(
        source_type: SourceType,
        options: &CompressOptions,
        scoping: &Scoping,
        allocator: &'a Allocator,
        track_dead_arguments: bool,
    ) -> Self {
        let mut values = IndexVec::with_capacity(scoping.symbols_len());
        values.resize_with(scoping.symbols_len(), || None);
        Self {
            values,
            dead_argument_candidates: track_dead_arguments
                .then(|| BitSet::new_in(scoping.symbols_len(), allocator)),
            dead_arguments: track_dead_arguments.then(FxHashMap::default),
            persistent: FxHashMap::default(),
            liveness: SymbolLiveness::new_if_enabled(source_type, options, scoping, allocator),
        }
    }

    /// Clear values without releasing the buffer so the next pass stays on
    /// the indexed-write fast path.
    pub fn reset_values(&mut self) {
        for value in &mut self.values {
            *value = None;
        }
    }

    #[inline]
    pub fn init_value(&mut self, symbol_id: SymbolId, value: SymbolValue<'a>) {
        self.values[symbol_id] = Some(value);
    }

    #[inline]
    pub fn value(&self, symbol_id: SymbolId) -> Option<&SymbolValue<'a>> {
        self.values.get(symbol_id)?.as_ref()
    }

    pub fn set_function_summary(&mut self, symbol_id: SymbolId, summary: FunctionSummary) {
        self.persistent.entry(symbol_id).or_default().set_function_summary(summary);
    }

    pub fn clear_function_summary(&mut self, symbol_id: SymbolId) {
        if let Some(metadata) = self.persistent.get_mut(&symbol_id) {
            // Clear in place: removing this shared entry would also erase its
            // monotone member-write effect.
            metadata.set_function_summary(FunctionSummary::Unknown);
        }
    }

    pub fn function_summary(&self, symbol_id: SymbolId) -> FunctionSummary {
        self.persistent
            .get(&symbol_id)
            .map_or(FunctionSummary::Unknown, PersistentSymbolMetadata::function_summary)
    }

    /// Record that a direct call may consume a dead-argument prefix.
    pub fn mark_dead_argument_candidate(&mut self, symbol_id: SymbolId) {
        let Some(candidates) = &mut self.dead_argument_candidates else { return };
        candidates.set_bit(symbol_id.index());
    }

    pub fn is_dead_argument_candidate(&self, symbol_id: SymbolId) -> bool {
        self.dead_argument_candidates
            .as_ref()
            .is_some_and(|candidates| candidates.contains(symbol_id.index()))
    }

    /// Record the first argument index whose value the function cannot observe.
    /// Returns `true` when an existing fact became stronger, or when an earlier
    /// call in this traversal missed a newly inserted fact.
    pub fn set_dead_argument_prefix(&mut self, symbol_id: SymbolId, prefix: usize) -> bool {
        let metadata = self.dead_arguments.as_mut().expect("dead-argument tracking is disabled");
        match metadata.entry(symbol_id) {
            Entry::Occupied(mut entry) => {
                let metadata = entry.get_mut();
                let previous = metadata.prefix();
                metadata.set_prefix(prefix);
                let changed = previous.is_some_and(|previous| previous != prefix);
                changed || std::mem::take(&mut metadata.missed_call)
            }
            Entry::Vacant(entry) => {
                let mut metadata = DeadArgumentMetadata::default();
                metadata.set_prefix(prefix);
                entry.insert(metadata);
                false
            }
        }
    }

    pub fn record_missed_dead_argument_call(&mut self, symbol_id: SymbolId) {
        self.dead_arguments
            .as_mut()
            .expect("dead-argument tracking is disabled")
            .entry(symbol_id)
            .or_default()
            .missed_call = true;
    }

    pub fn clear_dead_argument_prefix(&mut self, symbol_id: SymbolId) -> bool {
        let Some(metadata) =
            self.dead_arguments.as_mut().and_then(|facts| facts.get_mut(&symbol_id))
        else {
            return false;
        };
        let had_prefix = metadata.prefix.is_some();
        metadata.prefix = None;
        had_prefix
    }

    pub fn dead_argument_prefix(&self, symbol_id: SymbolId) -> Option<usize> {
        self.dead_arguments.as_ref()?.get(&symbol_id)?.prefix()
    }

    pub fn record_member_write_effect(&mut self, symbol_id: SymbolId, effect: MemberWriteEffect) {
        self.persistent.entry(symbol_id).or_default().record_member_write_effect(effect);
    }

    pub fn member_write_effect(&self, symbol_id: SymbolId) -> MemberWriteEffect {
        self.persistent
            .get(&symbol_id)
            .map_or(MemberWriteEffect::None, PersistentSymbolMetadata::member_write_effect)
    }

    /// Whether runtime semantics have an implicit observation channel that
    /// remains even if every resolved reference disappears from the current
    /// AST. Returns `false` when liveness state is absent; optimization
    /// consumers may interpret that result only in configurations where
    /// absence is safe.
    pub fn is_implicitly_observable(&self, symbol_id: SymbolId) -> bool {
        self.liveness.as_ref().is_some_and(|liveness| liveness.is_implicitly_observable(symbol_id))
    }

    /// Whether post-flush graph analysis proved a function declaration
    /// unreachable from executing code. Returns `false` when liveness state is
    /// absent because deadness was not proved.
    pub fn function_is_dead(&self, symbol_id: SymbolId) -> bool {
        self.liveness.as_ref().is_some_and(|liveness| liveness.function_is_dead(symbol_id))
    }

    pub fn liveness(&self) -> Option<&SymbolLiveness<'a>> {
        self.liveness.as_ref()
    }

    pub fn liveness_mut(&mut self) -> Option<&mut SymbolLiveness<'a>> {
        self.liveness.as_mut()
    }

    pub fn ensure_liveness(
        &mut self,
        create: impl FnOnce() -> SymbolLiveness<'a>,
    ) -> &mut SymbolLiveness<'a> {
        self.liveness.get_or_insert_with(create)
    }
}
