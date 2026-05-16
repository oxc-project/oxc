use oxc_ecmascript::constant_evaluation::ConstantValue;
use oxc_index::IndexVec;
use oxc_syntax::{scope::ScopeId, symbol::SymbolId};

#[derive(Debug)]
pub struct SymbolValue<'a> {
    /// Initialized constant value evaluated from expressions.
    /// `None` when the value is not a constant evaluated value.
    pub initialized_constant: Option<ConstantValue<'a>>,

    /// Symbol is exported.
    pub exported: bool,

    pub read_references_count: u32,
    pub write_references_count: u32,

    /// Number of read references that are member write targets (e.g. `a` in `a.foo = 1`).
    /// These reads exist only to access the object for a property write, not to use the value.
    /// Always <= `read_references_count`.
    pub member_write_target_read_count: u32,

    /// Whether the symbol's value is guaranteed fresh (cannot alias another binding).
    /// True for function/class declarations and variable declarations initialized
    /// with object/array/function/class literals.
    pub is_fresh_value: bool,

    pub scope_id: ScopeId,
}

/// Per-symbol scratch store indexed by `SymbolId`.
///
/// Symbol IDs are dense `u32`s, so an indexed `IndexVec` lookup beats a
/// `FxHashMap` (hash + probe) on every hot path in the peephole pass.
///
/// Sized once from `Scoping::symbols_len()`; no minifier pass mints new
/// symbols, so `init_value` panics on out-of-range — that's the signal to
/// add a grow path.
#[derive(Debug)]
pub struct SymbolValues<'a> {
    values: IndexVec<SymbolId, Option<SymbolValue<'a>>>,
}

impl<'a> SymbolValues<'a> {
    pub(crate) fn new(len: usize) -> Self {
        let mut values = IndexVec::with_capacity(len);
        values.resize_with(len, || None);
        Self { values }
    }

    /// Reset slots to `None` without releasing the buffer, so the next peephole
    /// iteration's `init_value` stays on the indexed-write fast path.
    pub fn reset(&mut self) {
        for slot in &mut self.values {
            *slot = None;
        }
    }

    #[inline]
    pub fn init_value(&mut self, symbol_id: SymbolId, symbol_value: SymbolValue<'a>) {
        self.values[symbol_id] = Some(symbol_value);
    }

    #[inline]
    pub fn get_symbol_value(&self, symbol_id: SymbolId) -> Option<&SymbolValue<'a>> {
        self.values.get(symbol_id)?.as_ref()
    }
}
