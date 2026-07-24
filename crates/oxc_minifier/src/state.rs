use rustc_hash::FxHashSet;

use oxc_allocator::{Allocator, BitSet};
use oxc_data_structures::stack::NonEmptyStack;
use oxc_semantic::Scoping;
use oxc_span::SourceType;
use oxc_str::Str;
use oxc_syntax::scope::ScopeId;

use crate::{CompressOptions, symbol_state::SymbolState};

/// Compression pipeline selected for this run.
#[derive(Clone, Copy)]
pub enum CompressionMode {
    /// Remove dead code and apply size-reducing transformations.
    Full,
    /// Remove dead and unused code without otherwise shrinking the output.
    /// This still runs the constant folds needed to expose removable branches,
    /// but skips transforms whose only purpose is smaller syntax. Rolldown uses
    /// this pipeline for tree-shaking with or without output minification.
    TreeShakeOnly,
}

/// Immutable configuration shared by all compression passes in one run.
struct CompressionConfig {
    source_type: SourceType,
    options: CompressOptions,
    mode: CompressionMode,
}

/// Changes accumulated between two pass-completion boundaries. Live from
/// `MinifierState::new` so the pre-loop `Normalize` pass records drops through
/// the same typed helpers as the peephole loop; consumed and reset or resized
/// by [`crate::compression_pass`] after Normalize and every peephole pass.
pub struct PassChanges<'a> {
    /// Whether the completed pass changed facts or AST shape in a way that
    /// requires another peephole traversal. This signal does not itself force
    /// liveness analysis; removed candidate references and dropped direct eval
    /// calls continue to gate that work independently.
    revisit_requested: bool,

    /// `ReferenceId`s whose AST node has been removed and not re-installed
    /// in any later mutation this pass.
    ///
    /// Arena-allocated bitset sized to the program's `references_len()` at
    /// construction / the previous flush. A `BitSet` (rather than an
    /// `FxHashSet`) keeps the per-ident cost on the
    /// `DroppedSubtreeCollector` hot path to
    /// a direct array store instead of a hash + heap insert.
    ///
    /// INVARIANT (the "capacity guard", relied on by `DroppedSubtreeCollector`,
    /// `Scoping::retain_resolved_references_excluding`, and the over-prune
    /// debug assert): references minted MID-pass have indices beyond the
    /// bitset's capacity and are treated as live everywhere — never marked,
    /// never excluded. Conservative: such a reference stays in its symbol's
    /// list until callers rebuild scoping (a missed optimization, never a
    /// correctness issue). `Normalize` mints no references, so a capacity
    /// taken at construction is exact for the first pass.
    pub(crate) removed_references: BitSet<'a>,

    /// At least one direct `eval(...)` call was dropped this pass. Gates
    /// the small `LiveDirectEvalCollector` walk at flush time.
    pub(crate) direct_eval_dropped: bool,
}

impl<'a> PassChanges<'a> {
    pub fn new(references_len: usize, allocator: &'a Allocator) -> Self {
        Self {
            revisit_requested: false,
            removed_references: BitSet::new_in(references_len, allocator),
            direct_eval_dropped: false,
        }
    }
}

/// State associated with one enclosing function body or the program root.
pub struct BodyFrame {
    /// Semantic scope containing the body's top-level statements.
    pub scope_id: ScopeId,

    /// Whether a preceding statement could observe a later hoisted variable
    /// before its initializer runs. The program root also starts unsafe when a
    /// module loader could expose its bindings through a cycle.
    pub hoisted_var_inlining_unsafe: bool,

    /// Source offset after the first unconditional top-level `super()` call,
    /// used to distinguish initialized and uninitialized derived-constructor
    /// `this` accesses.
    pub this_initialized_at: Option<u32>,
}

pub struct MinifierState<'a> {
    /// Source semantics, compression options, and pipeline selection fixed for
    /// the lifetime of this run.
    config: CompressionConfig,

    /// Dense per-pass values, sparse persistent metadata, and optional
    /// reachability data indexed by semantic symbols.
    pub(crate) symbols: SymbolState<'a>,

    /// Private-name usage scoped by nested classes. This tracks `#name`
    /// strings, not semantic `SymbolId`s, so it deliberately stays outside
    /// `SymbolState`.
    pub private_member_usage: PrivateMemberUsageStack<'a>,

    /// One frame per enclosing function body (program root at the bottom).
    /// Pushed by `enter_function_body` and popped by `exit_function_body`.
    pub body_frames: NonEmptyStack<BodyFrame>,

    /// Per-pass change accumulator populated by typed mutation helpers and
    /// consumed by `compression_pass` after Normalize and every peephole pass.
    pub(crate) pass_changes: PassChanges<'a>,

    /// Scratch buffer reused by `try_fold_concat` to build template literal
    /// quasis without allocating a fresh `String` per call.
    pub concat_scratch: String,
}

impl<'a> MinifierState<'a> {
    pub(crate) fn new(
        source_type: SourceType,
        options: CompressOptions,
        mode: CompressionMode,
        scoping: &Scoping,
        allocator: &'a Allocator,
    ) -> Self {
        let symbols = SymbolState::new(
            source_type,
            &options,
            scoping,
            allocator,
            matches!(mode, CompressionMode::TreeShakeOnly),
        );
        Self {
            config: CompressionConfig { source_type, options, mode },
            symbols,
            private_member_usage: PrivateMemberUsageStack::new(),
            body_frames: NonEmptyStack::new(BodyFrame {
                scope_id: scoping.root_scope_id(),
                hoisted_var_inlining_unsafe: false,
                this_initialized_at: None,
            }),
            pass_changes: PassChanges::new(scoping.references_len(), allocator),
            concat_scratch: String::new(),
        }
    }

    /// Whether `Normalize`'s member-write scan should seed persistent metadata,
    /// i.e. whether any consumer is live in this configuration. In full
    /// minify the default-mode write-only property drop reads
    /// hazardous-member state and the shared drop predicate reads possible
    /// prototype mutation. In tree-shake-only mode the default path is disabled
    /// and only the `property_write_side_effects: false` opt-in drop reads the
    /// prototype state, so with the knob left on nothing reads the effects and
    /// seeding is skipped.
    pub(crate) fn should_track_member_write_effects(&self) -> bool {
        !self.is_tree_shake_only() || !self.options().treeshake.property_write_side_effects
    }

    pub(crate) fn options(&self) -> &CompressOptions {
        &self.config.options
    }

    pub(crate) fn source_type(&self) -> SourceType {
        self.config.source_type
    }

    pub(crate) fn is_tree_shake_only(&self) -> bool {
        matches!(self.config.mode, CompressionMode::TreeShakeOnly)
    }

    /// Request another peephole traversal after the current pass completes.
    ///
    /// This is deliberately separate from [`Self::record_ast_change`] for
    /// future fact-only callers such as #24060. Such a caller may enable more
    /// transforms without changing the AST, resolved references, or direct-eval
    /// scope flags.
    pub(crate) fn request_revisit(&mut self) {
        self.pass_changes.revisit_requested = true;
    }

    /// Record an AST change and schedule the traversal needed to consume it.
    /// Typed mutation helpers use this as their AST-change entry point.
    pub(crate) fn record_ast_change(&mut self) {
        self.request_revisit();
    }

    /// Return whether the completed pass requested another traversal, then
    /// reset the signal for the next pass.
    pub(crate) fn take_revisit_requested(&mut self) -> bool {
        std::mem::take(&mut self.pass_changes.revisit_requested)
    }

    /// Whether every per-pass change has been consumed.
    pub(crate) fn pass_changes_are_clean(&self) -> bool {
        !self.pass_changes.revisit_requested
            && self.pass_changes.removed_references.is_empty()
            && !self.pass_changes.direct_eval_dropped
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
