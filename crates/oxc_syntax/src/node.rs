use bitflags::bitflags;
use oxc_index::Idx;
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer};

use crate::nonmax::NonMaxU32;

/// AST Node ID
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NodeId(NonMaxU32);

impl NodeId {
    pub const DUMMY: Self = NodeId::new(0);

    /// Create `NodeId` from `u32`.
    ///
    /// # Panics
    /// Panics if `idx` exceeds `NonMaxU32::MAX.get()`.
    pub const fn new(idx: u32) -> Self {
        Self(NonMaxU32::new_checked(idx))
    }

    /// Create `NodeId` from `u32` unchecked.
    ///
    /// # SAFETY
    /// `idx` must not exceed `NonMaxU32::MAX.get()`.
    #[allow(clippy::missing_safety_doc, clippy::unnecessary_safety_comment)]
    pub const unsafe fn new_unchecked(idx: u32) -> Self {
        // SAFETY: Caller must ensure `idx` does not exceed `NonMaxU32::MAX.get()`
        Self(NonMaxU32::new_unchecked(idx))
    }
}

impl Idx for NodeId {
    /// Create `NodeId` from `usize`.
    ///
    /// # Panics
    /// Panics if `idx` exceeds `NonMaxU32::MAX.get()`.
    fn from_usize(idx: usize) -> Self {
        Self(NonMaxU32::from_usize(idx))
    }

    fn index(self) -> usize {
        self.0.get() as usize
    }
}

#[cfg(feature = "serialize")]
impl Serialize for NodeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0.get())
    }
}

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type NodeId = number;
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
    }
}

impl NodeFlags {
    /// Returns `true` if this node has a JSDoc comment attached to it.
    #[inline]
    pub fn has_jsdoc(&self) -> bool {
        self.contains(Self::JSDoc)
    }

    /// Returns `true` if this node is inside a class.
    #[inline]
    pub fn has_class(&self) -> bool {
        self.contains(Self::Class)
    }

    /// Returns `true` if this function has a yield statement.
    #[inline]
    pub fn has_yield(&self) -> bool {
        self.contains(Self::HasYield)
    }
}
