use rustc_hash::FxHashSet;

use oxc_span::SourceType;
use oxc_syntax::symbol::SymbolId;

use crate::{CompressOptions, symbol_value::SymbolValues};

pub struct MinifierState<'a> {
    pub source_type: SourceType,

    pub options: CompressOptions,

    /// Function declarations that are empty
    pub empty_functions: FxHashSet<SymbolId>,

    pub symbol_values: SymbolValues<'a>,

    pub changed: bool,
}

impl MinifierState<'_> {
    pub fn new(source_type: SourceType, options: CompressOptions) -> Self {
        Self {
            source_type,
            options,
            empty_functions: FxHashSet::default(),
            symbol_values: SymbolValues::default(),
            changed: false,
        }
    }
}
