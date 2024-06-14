//! This module is just for documentation purposes, and is hidden behind the
//! `example_generated` feature, which is off by default.
//!
//! Note that a `cargo expand`ed version of this module (with some slight
//! cleanup -- e.g. removing all the code that comes from builtin derives) is
//! checked in to the [repository](https://github.com/thomcc/index_vec), and may
//! be easier/better to look at.

define_index_type! {
    /// I'm a doc comment on the type.
    pub struct CoolIndex = u32;

    DEFAULT = CoolIndex::new(0);

    MAX_INDEX = i32::MAX as usize;

    DISABLE_MAX_INDEX_CHECK = cfg!(not(debug_assertions));

    DISPLAY_FORMAT = "{} is a ~Cool Index~";

    DEBUG_FORMAT = "CI({:?})";

    IMPL_RAW_CONVERSIONS = true;
}
