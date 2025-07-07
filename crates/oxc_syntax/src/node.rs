//! AST Node ID and flags.
use bitflags::bitflags;
use nonmax::NonMaxU32;
use oxc_index::Idx;
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer};

/// AST Node ID
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NodeId(NonMaxU32);

impl NodeId {
    /// Mock node id.
    ///
    /// This is used for synthetically-created AST nodes, among other things.
    pub const DUMMY: Self = NodeId::new(0);

    /// Node id of the Program node.
    pub const ROOT: Self = NodeId::new(0);

    /// Create `NodeId` from `u32`.
    ///
    /// # Panics
    /// Panics if `idx` is `u32::MAX`.
    pub const fn new(idx: u32) -> Self {
        if let Some(idx) = NonMaxU32::new(idx) {
            return Self(idx);
        }
        panic!();
    }

    /// Create `NodeId` from `u32` unchecked.
    ///
    /// # SAFETY
    /// `idx` must not be `u32::MAX`.
    pub const unsafe fn new_unchecked(idx: u32) -> Self {
        // SAFETY: Caller must ensure `idx` is not `u32::MAX`
        unsafe { Self(NonMaxU32::new_unchecked(idx)) }
    }
}

impl Idx for NodeId {
    #[expect(clippy::cast_possible_truncation)]
    fn from_usize(idx: usize) -> Self {
        assert!(idx < u32::MAX as usize);
        // SAFETY: We just checked `idx` is a legal value for `NonMaxU32`
        Self(unsafe { NonMaxU32::new_unchecked(idx as u32) })
    }

    fn index(self) -> usize {
        self.0.get() as usize
    }
}

#[cfg(feature = "serialize")]
impl Serialize for NodeId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.0.get())
    }
}

bitflags! {
    /// Contains additional information about an AST node.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NodeFlags: u8 {
        /// Set if the Node has a JSDoc comment attached
        const JSDoc     = 1 << 0;
        /// Set on Nodes inside classes
        const Class     = 1 << 1;
        /// Set functions containing yield statements
        const HasYield  = 1 << 2;
        /// Set for `export { specifier }`
        const ExportSpecifier  = 1 << 3;
    }
}

impl NodeFlags {
    /// Returns `true` if this node has a JSDoc comment attached to it.
    #[inline]
    pub fn has_jsdoc(self) -> bool {
        self.contains(Self::JSDoc)
    }

    /// Returns `true` if this node is inside a class.
    #[inline]
    pub fn has_class(self) -> bool {
        self.contains(Self::Class)
    }

    /// Returns `true` if this function has a yield statement.
    #[inline]
    pub fn has_yield(self) -> bool {
        self.contains(Self::HasYield)
    }

    /// Returns `true` if this function has an export specifier.
    #[inline]
    pub fn has_export_specifier(self) -> bool {
        self.contains(Self::ExportSpecifier)
    }
}
