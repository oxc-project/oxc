use rustc_hash::FxHashSet;

use oxc_span::SourceType;
use oxc_syntax::symbol::SymbolId;

use crate::{CompressOptions, symbol_value::SymbolValues};

/// State maintained during minification.
///
/// This struct holds all the mutable state that needs to be tracked
/// during the minification process, including optimization options,
/// tracking of empty functions, symbol values, and change tracking.
pub struct MinifierState<'a> {
    /// The source type being processed (module, script, etc.).
    pub source_type: SourceType,

    /// Compression options that control which optimizations are applied.
    pub options: CompressOptions,

    /// Set of function symbol IDs that have been determined to be empty.
    /// This is used to optimize calls to these functions.
    pub empty_functions: FxHashSet<SymbolId>,

    /// Map of symbol IDs to their constant values and metadata.
    /// This is used for constant propagation and other value-based optimizations.
    pub symbol_values: SymbolValues<'a>,

    /// Flag indicating whether any changes were made during the current optimization pass.
    /// This is used to determine when the fixed-point iteration should stop.
    pub changed: bool,
}

impl MinifierState<'_> {
    /// Create a new minifier state with the given source type and options.
    pub fn new(source_type: SourceType, options: CompressOptions) -> Self {
        Self {
            source_type,
            options,
            empty_functions: FxHashSet::default(),
            symbol_values: SymbolValues::default(),
            changed: false,
        }
    }

    /// Mark the state as changed, indicating that optimizations were applied.
    pub fn mark_changed(&mut self) {
        self.changed = true;
    }

    /// Check if any changes were made since the last reset.
    pub fn has_changed(&self) -> bool {
        self.changed
    }

    /// Reset the changed flag, typically at the start of a new optimization pass.
    pub fn reset_changed(&mut self) {
        self.changed = false;
    }

    /// Mark a function as empty.
    pub fn mark_function_empty(&mut self, symbol_id: SymbolId) {
        self.empty_functions.insert(symbol_id);
    }

    /// Check if a function is marked as empty.
    pub fn is_function_empty(&self, symbol_id: SymbolId) -> bool {
        self.empty_functions.contains(&symbol_id)
    }
}
