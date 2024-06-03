use bitflags::bitflags;
use oxc_index::define_index_type;

define_index_type! {
    pub struct AstNodeId = usize;
}

impl AstNodeId {
    #[inline]
    pub fn dummy() -> Self {
        Self::new(0)
    }
}

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type AstNodeId = number;
export type NodeFlags = {
    JSDoc: 1,
    Class: 2,
    HasYield: 4
    Parameter: 8
};
"#;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NodeFlags: u8 {
        const JSDoc     = 1 << 0; // If the Node has a JSDoc comment attached
        const Class     = 1 << 1; // If Node is inside a class
        const HasYield  = 1 << 2; // If function has yield statement
        const Parameter = 1 << 3; // If Node is inside a parameter
    }
}

impl NodeFlags {
    #[inline]
    pub fn has_jsdoc(&self) -> bool {
        self.contains(Self::JSDoc)
    }

    #[inline]
    pub fn has_class(&self) -> bool {
        self.contains(Self::Class)
    }

    #[inline]
    pub fn has_yield(&self) -> bool {
        self.contains(Self::HasYield)
    }

    #[inline]
    pub fn has_parameter(&self) -> bool {
        self.contains(Self::Parameter)
    }
}
