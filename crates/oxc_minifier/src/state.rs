use oxc_ast::ast::Function;
use oxc_ecmascript::constant_evaluation::ConstantValue;
use rustc_hash::{FxHashMap, FxHashSet};

use oxc_span::{Atom, SourceType};
use oxc_syntax::symbol::SymbolId;

use crate::{CompressOptions, symbol_value::SymbolValues};

pub struct MinifierState<'a> {
    pub source_type: SourceType,

    pub options: CompressOptions,

    /// The return value of function declarations that are pure
    pub pure_functions: FxHashMap<SymbolId, Option<ConstantValue<'a>>>,

    pub symbol_values: SymbolValues<'a>,

    pub inline_function_declarations: FxHashMap<SymbolId, Function<'a>>,
    pub rename_symbols: FxHashMap<SymbolId, Atom<'a>>,
    pub inlined_function_declarations: FxHashSet<SymbolId>,

    pub changed: bool,
}

impl MinifierState<'_> {
    pub fn new(source_type: SourceType, options: CompressOptions) -> Self {
        Self {
            source_type,
            options,
            pure_functions: FxHashMap::default(),
            symbol_values: SymbolValues::default(),
            inline_function_declarations: FxHashMap::default(),
            rename_symbols: FxHashMap::default(),
            inlined_function_declarations: FxHashSet::default(),
            changed: false,
        }
    }
}
