use oxc_ecmascript::constant_evaluation::ConstantValue;
use rustc_hash::{FxHashMap, FxHashSet};

use oxc_allocator::{Allocator, BitSet};
use oxc_data_structures::stack::NonEmptyStack;
use oxc_semantic::Scoping;
use oxc_span::SourceType;
use oxc_str::Str;
use oxc_syntax::symbol::SymbolId;

use crate::{CompressOptions, symbol_value::SymbolValues};

/// Per-pass dirty data accumulated by walking-helper calls. Consumed by
/// `exit_program` and reset there.
pub struct PassDirty<'a> {
    /// `ReferenceId`s whose AST node has been removed and not re-installed
    /// in any later mutation this pass.
    ///
    /// Arena-allocated bitset sized to the program's `references_len()` at
    /// `enter_program`. Switched from `FxHashSet` to `BitSet` (spec §6.3
    /// Tier 1 mitigation) to drop per-ident `insert`/`remove` cost on the
    /// `DropDiff` hot path from ~25 cycles (hash + heap) to ~5 cycles
    /// (direct array store).
    ///
    /// Invariant: every `ReferenceId` that `DropDiff` visits has an index
    /// `< capacity`. Refs minted MID-pass (via `create_reference` /
    /// `clone_in_with_semantic_ids`) would have indices beyond capacity,
    /// but a debug-mode probe (panic on `idx >= capacity`) confirmed this
    /// case is unreachable in both the test corpus (506 tests) and the
    /// `just minsize` corpus, so the hot path elides the bounds check.
    pub(crate) dead_refs: BitSet<'a>,

    /// At least one direct `eval(...)` call was dropped this pass. Gates
    /// the small `LiveDirectEvalCollector` walk at `exit_program`.
    pub(crate) eval_dropped: bool,
}

impl<'a> PassDirty<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            // Empty bitset; replaced with a properly-sized one at `enter_program`.
            dead_refs: BitSet::new_in(0, allocator),
            eval_dropped: false,
        }
    }

    /// Re-allocate `dead_refs` sized to the program's current
    /// `references_len()`, and reset all other accumulator fields.
    ///
    /// Called at every `enter_program`. The prior bitset is dropped; the
    /// arena reclaims its memory at program end. We re-allocate (rather
    /// than `clear()`) because `references_len()` can grow between passes
    /// as helpers mint fresh references.
    pub fn init(&mut self, references_len: usize, allocator: &'a Allocator) {
        self.dead_refs = BitSet::new_in(references_len, allocator);
        self.eval_dropped = false;
    }

    /// Reset everything except `dead_refs` allocation, which is re-sized
    /// by `init`. Used at `exit_program` to clear cross-pass leakage of
    /// `eval_dropped`; `dead_refs` is already consumed by then so its state
    /// doesn't matter until the next `init`.
    pub fn reset(&mut self) {
        self.eval_dropped = false;
    }

    /// Future API: short-circuit when the per-pass accumulator is empty
    /// (no AST mutation observed).
    #[expect(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.dead_refs.is_empty() && !self.eval_dropped
    }
}

pub struct MinifierState<'a> {
    pub source_type: SourceType,

    pub options: CompressOptions,

    /// When true, only run dead code elimination passes (subset of full peephole optimizations).
    pub dce: bool,

    /// The return value of function declarations that are pure
    pub pure_functions: FxHashMap<SymbolId, Option<ConstantValue<'a>>>,

    pub symbol_values: SymbolValues<'a>,

    /// Private member usage for classes
    pub class_symbols_stack: ClassSymbolsStack<'a>,

    /// Symbols that have `__proto__` member writes.
    /// Writing to `__proto__` changes the prototype chain, potentially installing
    /// setters that make subsequent property writes side-effectful.
    pub proto_write_symbols: FxHashSet<SymbolId>,

    /// Monotonic mutation counter. Bumped by every helper call (`replace_*`,
    /// `drop_*`, `notice_change`). The fixed-point loop driver snapshots this
    /// value, runs a pass, and re-runs while the counter advanced.
    pub(crate) mutations: u64,

    /// Per-pass dirty accumulator populated by `replace_*` / `drop_*` helpers.
    /// Will be consumed by `exit_program` in commit 5 to drive incremental
    /// scoping refresh; this commit only builds the data (no-op observable).
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
        Self {
            source_type,
            options,
            dce,
            pure_functions: FxHashMap::default(),
            symbol_values: SymbolValues::new(scoping.symbols_len()),
            class_symbols_stack: ClassSymbolsStack::new(),
            proto_write_symbols: FxHashSet::default(),
            mutations: 0,
            dirty: PassDirty::new(allocator),
            concat_scratch: String::new(),
        }
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
