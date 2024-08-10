use bitflags::bitflags;

use nonmax::NonMaxU32;
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer};

use oxc_index::Idx;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AstNodeId(NonMaxU32);

impl AstNodeId {
    pub const DUMMY: Self = AstNodeId::new(0);

    /// Create `AstNodeId` from `u32`.
    ///
    /// # Panics
    /// Panics if `idx` is `u32::MAX`.
    pub const fn new(idx: u32) -> Self {
        // We could use `NonMaxU32::new(idx).unwrap()` but `Option::unwrap` is not a const function
        // and we want this function to be
        assert!(idx != u32::MAX);
        // SAFETY: We have checked that `idx` is not `u32::MAX`
        unsafe { Self::new_unchecked(idx) }
    }

    /// Create `AstNodeId` from `u32` unchecked.
    ///
    /// # SAFETY
    /// `idx` must not be `u32::MAX`.
    #[allow(clippy::missing_safety_doc, clippy::unnecessary_safety_comment)]
    pub const unsafe fn new_unchecked(idx: u32) -> Self {
        // SAFETY: Caller must ensure `idx` is not `u32::MAX`
        Self(NonMaxU32::new_unchecked(idx))
    }
}

impl Idx for AstNodeId {
    #[allow(clippy::cast_possible_truncation)]
    fn from_usize(idx: usize) -> Self {
        assert!(idx < u32::MAX as usize);
        // SAFETY: We just checked `idx` is valid for `NonMaxU32`
        Self(unsafe { NonMaxU32::new_unchecked(idx as u32) })
    }

    fn index(self) -> usize {
        self.0.get() as usize
    }
}

#[cfg(feature = "serialize")]
impl Serialize for AstNodeId {
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
}
