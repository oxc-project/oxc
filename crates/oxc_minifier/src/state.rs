use rustc_hash::FxHashSet;

use oxc_allocator::{Allocator, BitSet};
use oxc_data_structures::stack::NonEmptyStack;
use oxc_semantic::Scoping;
use oxc_span::SourceType;
use oxc_str::Str;
use oxc_syntax::scope::ScopeId;

use crate::{CompressOptions, symbol_state::SymbolState};

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

    /// Dense per-pass values, sparse persistent metadata, and optional
    /// reachability data indexed by semantic symbols.
    pub(crate) symbols: SymbolState<'a>,

    /// Private-name usage scoped by nested classes. This tracks `#name`
    /// strings, not semantic `SymbolId`s, so it deliberately stays outside
    /// `SymbolState`.
    pub private_member_usage: PrivateMemberUsageStack<'a>,

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
        let symbols = SymbolState::new(source_type, &options, scoping, allocator);
        Self {
            source_type,
            options,
            dce,
            symbols,
            private_member_usage: PrivateMemberUsageStack::new(),
            body_unsafe_stack: NonEmptyStack::new((scoping.root_scope_id(), false)),
            mutated: false,
            dirty: PassDirty::new(scoping.references_len(), allocator),
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

/// Private member names used in each currently enclosing class.
pub struct PrivateMemberUsageStack<'a> {
    stack: NonEmptyStack<FxHashSet<Str<'a>>>,
}

impl<'a> PrivateMemberUsageStack<'a> {
    pub fn new() -> Self {
        Self { stack: NonEmptyStack::new(FxHashSet::default()) }
    }

    /// Whether all class scopes have been exited.
    pub fn is_at_root(&self) -> bool {
        self.stack.is_exhausted()
    }

    pub fn enter_class(&mut self) {
        self.stack.push(FxHashSet::default());
    }

    /// Exit a class and propagate uses of names declared by an outer class.
    pub fn exit_class(&mut self, declared_private_members: impl Iterator<Item = Str<'a>>) {
        let mut used_private_members = self.stack.pop();
        declared_private_members.for_each(|name| {
            used_private_members.remove(&name);
        });
        self.stack.last_mut().extend(used_private_members);
    }

    pub fn record_use(&mut self, name: Str<'a>) {
        self.stack.last_mut().insert(name);
    }

    pub fn is_used(&self, name: &Str<'a>) -> bool {
        self.stack.last().contains(name)
    }
}
