use bitflags::bitflags;
use oxc_index::define_index_type;

define_index_type! {
    pub struct AstNodeId = usize;
}

#[cfg_attr(
    all(feature = "serde", feature = "wasm"),
    wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)
)]
#[allow(dead_code)]
const TS_APPEND_CONTENT: &'static str = r#"
export type AstNodeId = number;
export type NodeFlags = {
    JSDoc: 1,
    Class: 2,
    HasYield: 4
};
"#;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NodeFlags: u8 {
        const JSDoc    = 1 << 0; // If the Node has a JSDoc comment attached
        const Class    = 1 << 1; // If Node is inside a class
        const HasYield = 1 << 2; // If function has yield statement

    }
}

impl NodeFlags {
    pub fn has_jsdoc(&self) -> bool {
        self.contains(Self::JSDoc)
    }

    pub fn has_class(&self) -> bool {
        self.contains(Self::Class)
    }

    pub fn has_yield(&self) -> bool {
        self.contains(Self::HasYield)
    }
}
