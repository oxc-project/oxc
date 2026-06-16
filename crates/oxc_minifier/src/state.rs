use oxc_ecmascript::constant_evaluation::ConstantValue;
use rustc_hash::{FxHashMap, FxHashSet};

use oxc_allocator::{Allocator, BitSet};
use oxc_data_structures::stack::NonEmptyStack;
use oxc_semantic::Scoping;
use oxc_span::SourceType;
use oxc_str::Str;
use oxc_syntax::symbol::SymbolId;

use crate::{CompressOptions, symbol_value::SymbolValues};

/// Dirty data accumulated by the `replace_*` / `drop_*` helper calls between
/// two consumption points. Live from `MinifierState::new` so the pre-loop
/// `Normalize` pass records drops through the same typed helpers as the
/// peephole loop; consumed and re-initialized by `flush_pass_dirty` in the
/// `Compressor` driver after `Normalize` and after every peephole pass.
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

    /// Set when a typed helper mutates the AST. Private by design: the only
    /// writers are the helpers on `MinifierTraverseCtx`; the only reader is
    /// the fixed-point loop driver via `take_mutated()`.
    mutated: bool,

    /// Per-pass dirty accumulator populated by `replace_*` / `drop_*` helpers
    /// as subtrees are removed. Consumed by `flush_pass_dirty` in the
    /// `Compressor` driver (pre-loop and after each mutated pass) to drive
    /// the incremental scoping refresh.
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
            mutated: false,
            dirty: PassDirty::new(scoping.references_len(), allocator),
            concat_scratch: String::new(),
        }
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
