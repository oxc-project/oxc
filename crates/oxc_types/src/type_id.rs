use oxc_index::define_index_type;

define_index_type! {
    /// Unique identifier for a type in the type arena.
    pub struct TypeId = u32;
}
