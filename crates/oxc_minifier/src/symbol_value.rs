use rustc_hash::FxHashMap;

use oxc_ecmascript::constant_evaluation::ConstantValue;
use oxc_syntax::{scope::ScopeId, symbol::SymbolId};

/// Information about a symbol's value and usage patterns.
///
/// This struct tracks various properties of symbols that are used
/// during optimization, including constant values, export status,
/// and reference counts.
#[derive(Debug)]
pub struct SymbolValue<'a> {
    /// The constant value of this symbol, if it has been determined to be constant.
    /// `None` indicates that the value is not a compile-time constant.
    pub initialized_constant: Option<ConstantValue<'a>>,

    /// Whether this symbol is exported from the current module.
    /// Exported symbols generally cannot be optimized as aggressively.
    pub exported: bool,

    /// Whether this symbol is declared in a for statement initializer.
    /// This affects how the symbol can be optimized due to scoping rules.
    pub for_statement_init: bool,

    /// Number of read references to this symbol.
    pub read_references_count: u32,

    /// Number of write references to this symbol.
    pub write_references_count: u32,

    /// The scope ID where this symbol is declared.
    #[expect(unused)]
    pub scope_id: ScopeId,
}

impl<'a> SymbolValue<'a> {
    /// Check if this symbol is read-only (has no write references after initialization).
    pub fn is_read_only(&self) -> bool {
        self.write_references_count == 0
    }

    /// Check if this symbol is unused (has no read references).
    pub fn is_unused(&self) -> bool {
        self.read_references_count == 0
    }

    /// Check if this symbol has a constant value and is safe to inline.
    pub fn is_inlinable(&self) -> bool {
        self.initialized_constant.is_some()
            && self.is_read_only()
            && !self.exported
            && !self.for_statement_init
    }
}

/// Container for tracking symbol values during optimization.
///
/// This maintains a mapping from symbol IDs to their associated
/// value information, which is used for constant propagation,
/// dead code elimination, and other optimizations.
#[derive(Debug, Default)]
pub struct SymbolValues<'a> {
    values: FxHashMap<SymbolId, SymbolValue<'a>>,
}

impl<'a> SymbolValues<'a> {
    /// Clear all stored symbol values, typically at the start of a new optimization pass.
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Store information about a symbol.
    pub fn init_value(&mut self, symbol_id: SymbolId, symbol_value: SymbolValue<'a>) {
        self.values.insert(symbol_id, symbol_value);
    }

    /// Retrieve information about a symbol, if available.
    pub fn get_symbol_value(&self, symbol_id: SymbolId) -> Option<&SymbolValue<'a>> {
        self.values.get(&symbol_id)
    }

    /// Check if a symbol has a constant value.
    pub fn has_constant_value(&self, symbol_id: SymbolId) -> bool {
        self.values.get(&symbol_id).map_or(false, |sv| sv.initialized_constant.is_some())
    }

    /// Get the constant value for a symbol, if it has one.
    pub fn get_constant_value(&self, symbol_id: SymbolId) -> Option<&ConstantValue<'a>> {
        self.values.get(&symbol_id).and_then(|sv| sv.initialized_constant.as_ref())
    }

    /// Check if a symbol is safe to inline (has constant value and no side effects).
    pub fn is_inlinable(&self, symbol_id: SymbolId) -> bool {
        self.values.get(&symbol_id).map_or(false, |sv| sv.is_inlinable())
    }
}
