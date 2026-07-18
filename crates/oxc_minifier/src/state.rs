use rustc_hash::{FxHashMap, FxHashSet};

use oxc_allocator::{Allocator, BitSet};
use oxc_data_structures::stack::NonEmptyStack;
use oxc_semantic::Scoping;
use oxc_span::SourceType;
use oxc_str::Str;
use oxc_syntax::{scope::ScopeId, symbol::SymbolId};

use crate::{
    CompressOptions,
    symbol_liveness::SymbolLiveness,
    symbol_metadata::{FunctionSummary, MemberWriteEffect, PersistentSymbolMetadata},
    symbol_value::SymbolValues,
};

/// Dirty data accumulated by the `replace_*` / `drop_*` helper calls between
/// two consumption points. Live from `MinifierState::new` so the pre-loop
/// `Normalize` pass records drops through the same typed helpers as the
/// peephole loop; consumed and reset or resized by the end-of-pass sequence
/// after `Normalize` and after every peephole pass.
pub struct PassDirty<'a> {
    /// `ReferenceId`s whose AST node has been removed and not re-installed
    /// in any later mutation this pass.
    ///
    /// Arena-allocated bitset sized to the program's `references_len()` at
    /// construction / the previous flush. A `BitSet` (rather than an
    /// `FxHashSet`) keeps the per-ident cost on the `DropDiff` hot path to
    /// a direct array store instead of a hash + heap insert.
    ///
    /// INVARIANT (the "capacity guard", relied on by `DropDiff`,
    /// `Scoping::retain_resolved_references_excluding`, and the over-prune
    /// debug assert): references minted MID-pass have indices beyond the
    /// bitset's capacity and are treated as live everywhere — never marked,
    /// never excluded. Conservative: such a reference stays in its symbol's
    /// list until callers rebuild scoping (a missed optimization, never a
    /// correctness issue). `Normalize` mints no references, so a capacity
    /// taken at construction is exact for the first pass.
    pub(crate) dead_refs: BitSet<'a>,

    /// At least one direct `eval(...)` call was dropped this pass. Gates
    /// the small `LiveDirectEvalCollector` walk at flush time.
    pub(crate) eval_dropped: bool,
}

impl<'a> PassDirty<'a> {
    pub fn new(references_len: usize, allocator: &'a Allocator) -> Self {
        Self { dead_refs: BitSet::new_in(references_len, allocator), eval_dropped: false }
    }
}

pub struct MinifierState<'a> {
    pub source_type: SourceType,

    pub options: CompressOptions,

    /// Two modes: tree-shaking only (`true`), or full minify (`false`).
    /// `Compressor::dead_code_elimination` uses `true`; `Compressor::build`
    /// uses `false`.
    ///
    /// "DCE" here does not mean "removes dead code" (full minify does that
    /// too). It means tree-shaking: remove code that nothing imports, without
    /// making the rest smaller. Rolldown runs this on every tree-shaking build,
    /// with or without `minify`, so users can see this output directly.
    ///
    /// In this mode `exit_*` only runs the passes that remove code, plus the
    /// constant folds those removals need. For example, `fold_binary_expr`
    /// folds `'production' === 'production'` to `true` so the dead `else`
    /// branch can be dropped (this is how `define` values remove branches). The
    /// passes that only shrink code (`substitute_*`, `minimize_*`) are left
    /// out. See the `if ctx.state.dce` branch in `peephole/mod.rs`.
    pub dce: bool,

    /// Sparse metadata that remains valid across peephole iterations.
    persistent_symbols: FxHashMap<SymbolId, PersistentSymbolMetadata>,

    pub symbol_values: SymbolValues<'a>,

    /// Private member usage for classes
    pub class_symbols_stack: ClassSymbolsStack<'a>,

    /// One frame per enclosing function body (program root at the bottom).
    /// `(body_scope, body_unsafe)`. While `body_unsafe` is false, the next
    /// `var x = <literal>;` whose declarator sits at `body_scope` is safe to
    /// inline despite hoisting. A preceding non-declarative statement sets it;
    /// the program root additionally starts unsafe when the module has any
    /// loader (`import` / `export … from` / `export * from`), since a cyclic
    /// importer could observe a not-yet-assigned binding our exports close over.
    /// Pushed by `enter_function_body`, popped by `exit_function_body`. See
    /// `init_symbol_value`.
    pub body_unsafe_stack: NonEmptyStack<(ScopeId, bool)>,

    /// Set when a typed helper mutates the AST. Private by design: the only
    /// writers are the helpers on `MinifierTraverseCtx`; the only reader is
    /// the fixed-point loop driver via `take_mutated()`.
    mutated: bool,

    /// Per-pass dirty accumulator populated by `replace_*` / `drop_*` helpers
    /// as subtrees are removed. Consumed by the end-of-pass sequence after
    /// Normalize and every peephole pass to drive the incremental scoping
    /// refresh.
    pub(crate) dirty: PassDirty<'a>,

    /// Implicitly observable bindings plus the optional recursive-function
    /// graph. Present for modules or when unused removal is enabled; a `using`
    /// declaration can also create it lazily. Stable metadata is seeded from
    /// scoping and Normalize; post-flush analysis derives reachability from the
    /// current semantic reference lists.
    pub(crate) symbol_liveness: Option<SymbolLiveness<'a>>,

    /// Scratch buffer reused by `try_fold_concat` to build template literal
    /// quasis without allocating a fresh `String` per call.
    pub concat_scratch: String,
}

impl<'a> MinifierState<'a> {
    pub fn new(
        source_type: SourceType,
        options: CompressOptions,
        dce: bool,
        scoping: &Scoping,
        allocator: &'a Allocator,
    ) -> Self {
        let symbol_liveness =
            SymbolLiveness::new_if_enabled(source_type, &options, scoping, allocator);
        Self {
            source_type,
            options,
            dce,
            persistent_symbols: FxHashMap::default(),
            symbol_values: SymbolValues::new(scoping.symbols_len()),
            class_symbols_stack: ClassSymbolsStack::new(),
            body_unsafe_stack: NonEmptyStack::new((scoping.root_scope_id(), false)),
            mutated: false,
            dirty: PassDirty::new(scoping.references_len(), allocator),
            symbol_liveness,
            concat_scratch: String::new(),
        }
    }

    /// Whether `Normalize`'s member-write scan should seed persistent metadata,
    /// i.e. whether any consumer is live in this configuration. In full
    /// minify the default-mode write-only property drop reads
    /// hazardous-member state and the shared drop predicate reads possible
    /// prototype mutation. In DCE mode the default path is disabled and only
    /// the `property_write_side_effects: false` opt-in drop reads the prototype
    /// state, so with the knob left on nothing reads the effects and seeding is
    /// skipped.
    pub fn should_track_member_write_effects(&self) -> bool {
        !self.dce || !self.options.treeshake.property_write_side_effects
    }

    pub(crate) fn set_function_summary(&mut self, symbol_id: SymbolId, summary: FunctionSummary) {
        self.persistent_symbols.entry(symbol_id).or_default().set_function_summary(summary);
    }

    pub(crate) fn clear_function_summary(&mut self, symbol_id: SymbolId) {
        if let Some(metadata) = self.persistent_symbols.get_mut(&symbol_id) {
            // Clear in place: removing this shared entry would also erase its
            // monotone member-write effect.
            metadata.set_function_summary(FunctionSummary::Unknown);
        }
    }

    pub(crate) fn function_summary(&self, symbol_id: SymbolId) -> FunctionSummary {
        self.persistent_symbols
            .get(&symbol_id)
            .map_or(FunctionSummary::Unknown, PersistentSymbolMetadata::function_summary)
    }

    pub(crate) fn record_member_write_effect(
        &mut self,
        symbol_id: SymbolId,
        effect: MemberWriteEffect,
    ) {
        self.persistent_symbols.entry(symbol_id).or_default().record_member_write_effect(effect);
    }

    pub(crate) fn member_write_effect(&self, symbol_id: SymbolId) -> MemberWriteEffect {
        self.persistent_symbols
            .get(&symbol_id)
            .map_or(MemberWriteEffect::None, PersistentSymbolMetadata::member_write_effect)
    }

    /// Whether runtime semantics have an implicit observation channel that
    /// remains even if every resolved reference disappears from the current
    /// AST.
    pub(crate) fn symbol_is_implicitly_observable(&self, symbol_id: SymbolId) -> bool {
        self.symbol_liveness
            .as_ref()
            .is_some_and(|liveness| liveness.is_implicitly_observable(symbol_id))
    }

    /// Whether post-flush graph analysis proved a function declaration
    /// unreachable from executing code.
    pub(crate) fn function_is_dead(&self, symbol_id: SymbolId) -> bool {
        self.symbol_liveness.as_ref().is_some_and(|liveness| liveness.function_is_dead(symbol_id))
    }

    /// Returns whether the AST was mutated since the last call, and resets.
    /// Read and reset are one operation so the signal cannot be cleared
    /// anywhere except at its single consumption point.
    pub(crate) fn take_mutated(&mut self) -> bool {
        std::mem::take(&mut self.mutated)
    }

    /// Record that a typed helper mutated the AST.
    pub(crate) fn record_mutation(&mut self) {
        self.mutated = true;
    }
}

/// Stack to track class symbol information
pub struct ClassSymbolsStack<'a> {
    stack: NonEmptyStack<FxHashSet<Str<'a>>>,
}

impl<'a> ClassSymbolsStack<'a> {
    pub fn new() -> Self {
        Self { stack: NonEmptyStack::new(FxHashSet::default()) }
    }

    /// Check if the stack is exhausted
    pub fn is_exhausted(&self) -> bool {
        self.stack.is_exhausted()
    }

    /// Enter a new class scope
    pub fn push_class_scope(&mut self) {
        self.stack.push(FxHashSet::default());
    }

    /// Exit the current class scope
    pub fn pop_class_scope(&mut self, declared_private_symbols: impl Iterator<Item = Str<'a>>) {
        let mut used_private_symbols = self.stack.pop();
        declared_private_symbols.for_each(|name| {
            used_private_symbols.remove(&name);
        });
        // if the symbol was not declared in this class, that is declared in the class outside the current class
        self.stack.last_mut().extend(used_private_symbols);
    }

    /// Add a private member to the current class scope
    pub fn push_private_member_to_current_class(&mut self, name: Str<'a>) {
        self.stack.last_mut().insert(name);
    }

    /// Check if a private member is used in the current class scope
    pub fn is_private_member_used_in_current_class(&self, name: &Str<'a>) -> bool {
        self.stack.last().contains(name)
    }
}
