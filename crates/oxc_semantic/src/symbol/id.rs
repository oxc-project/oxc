use std::hash::Hash;

use oxc_index::define_index_type;

define_index_type! {
    pub struct SymbolId = u32;
}
