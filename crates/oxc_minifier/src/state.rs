use oxc_ecmascript::constant_evaluation::ConstantValue;
use rustc_hash::{FxHashMap, FxHashSet};

use oxc_data_structures::stack::NonEmptyStack;
use oxc_semantic::Scoping;
use oxc_span::SourceType;
use oxc_str::{Ident, Str};
use oxc_syntax::{reference::ReferenceId, symbol::SymbolId};

use crate::{CompressOptions, symbol_value::SymbolValues};

/// Per-pass dirty data accumulated by walking-helper calls. Consumed by
/// `exit_program` (in commit 5) and reset there.
pub struct PassDirty<'a> {
    /// `ReferenceId`s whose AST node has been removed and not re-installed
    /// in any later mutation this pass.
    pub(crate) dead_refs: FxHashSet<ReferenceId>,

    /// Names of unresolved references whose last AST occurrence has been
    /// removed. Pruning `Scoping::root_unresolved_references` is name-keyed
    /// (and a name can have many references); confirming the prune is safe
    /// requires a small walk in `exit_program`.
    pub(crate) dead_unresolved: FxHashSet<Ident<'a>>,

    /// At least one direct `eval(...)` call was dropped this pass. Gates
    /// the small `LiveDirectEvalCollector` walk at `exit_program`.
    pub(crate) eval_dropped: bool,
}

impl PassDirty<'_> {
    pub fn new() -> Self {
        Self {
            dead_refs: FxHashSet::default(),
            dead_unresolved: FxHashSet::default(),
            eval_dropped: false,
        }
    }

    pub fn reset(&mut self) {
        self.dead_refs.clear();
        self.dead_unresolved.clear();
        self.eval_dropped = false;
    }

    /// Future API: commit 5's `exit_program` will short-circuit when the
    /// per-pass accumulator is empty (no AST mutation observed).
    #[expect(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.dead_refs.is_empty() && self.dead_unresolved.is_empty() && !self.eval_dropped
    }
}

impl Default for PassDirty<'_> {
    fn default() -> Self {
        Self::new()
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

    pub(crate) changed: bool,

    /// Monotonic mutation counter. Bumped by every helper call.
    /// Together with `changed: bool` during this transition commit;
    /// `changed` is removed in commit 5.
    pub(crate) mutations: u64,

    /// Per-pass dirty accumulator populated by `replace_*` / `drop_*` helpers.
    /// Will be consumed by `exit_program` in commit 5 to drive incremental
    /// scoping refresh; this commit only builds the data (no-op observable).
    pub(crate) dirty: PassDirty<'a>,

    /// Scratch buffer reused by `try_fold_concat` to build template literal
    /// quasis without allocating a fresh `String` per call.
    pub concat_scratch: String,
}

impl MinifierState<'_> {
    pub fn new(
        source_type: SourceType,
        options: CompressOptions,
        dce: bool,
        scoping: &Scoping,
    ) -> Self {
        Self {
            source_type,
            options,
            dce,
            pure_functions: FxHashMap::default(),
            symbol_values: SymbolValues::new(scoping.symbols_len()),
            class_symbols_stack: ClassSymbolsStack::new(),
            proto_write_symbols: FxHashSet::default(),
            changed: false,
            mutations: 0,
            dirty: PassDirty::new(),
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
