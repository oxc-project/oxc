use oxc_index::define_index_type;

define_index_type! {
    pub struct ClassId = u32;
}
define_index_type! {
    pub struct PropertyId = u32;
}
define_index_type! {
    pub struct MethodId = u32;
}
