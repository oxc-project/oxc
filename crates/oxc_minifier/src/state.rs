use oxc_ecmascript::constant_evaluation::ConstantValue;
use rustc_hash::FxHashMap;

use oxc_span::SourceType;
use oxc_syntax::symbol::SymbolId;

use crate::{CompressOptions, symbol_value::SymbolInformationMap};

pub struct MinifierState<'a> {
    pub source_type: SourceType,

    pub options: CompressOptions,

    /// The return value of function declarations that are pure
    pub pure_functions: FxHashMap<SymbolId, Option<ConstantValue<'a>>>,

    pub symbol_values: SymbolInformationMap<'a>,

    pub changed: bool,
}

impl MinifierState<'_> {
    pub fn new(source_type: SourceType, options: CompressOptions) -> Self {
        Self {
            source_type,
            options,
            pure_functions: FxHashMap::default(),
            symbol_values: SymbolInformationMap::default(),
            changed: false,
        }
    }
}
