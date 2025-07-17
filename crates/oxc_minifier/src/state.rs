use rustc_hash::FxHashMap;

use oxc_ecmascript::constant_evaluation::ConstantValue;
use oxc_semantic::SymbolId;
use oxc_span::SourceType;

use crate::CompressOptions;

pub struct MinifierState<'a> {
    pub source_type: SourceType,

    pub options: CompressOptions,

    /// Constant values evaluated from expressions.
    ///
    /// Values are saved during constant evaluation phase.
    /// Values are read during [oxc_ecmascript::is_global_reference::IsGlobalReference::get_constant_value_for_reference_id].
    pub constant_values: FxHashMap<SymbolId, ConstantValue<'a>>,
}

impl MinifierState<'_> {
    pub fn new(source_type: SourceType, options: CompressOptions) -> Self {
        Self { source_type, options, constant_values: FxHashMap::default() }
    }
}
