//! AST Node ID and flags.
use std::fmt;

use bitflags::bitflags;
use oxc_index::Idx;
#[cfg(feature = "serialize")]
use serde::Serialize;

/// AST Node ID with interior mutability.
///
/// This type is `Copy` and provides interior mutability through unsafe code.
/// Safe to use as long as the AST is only accessed from a single thread.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[repr(transparent)]
pub struct NodeId {
    value: u32,
}

impl NodeId {
    /// Dummy node id for synthetic AST nodes.
    pub const DUMMY: Self = Self { value: 0 };

    /// Node id of the Program node.
    pub const ROOT: Self = Self { value: 0 };

    #[inline]
    pub const fn new(v: u32) -> Self {
        Self { value: v }
    }

    #[inline]
    pub const fn get(self) -> u32 {
        self.value
    }

    /// Set the node ID value.
    ///
    /// # Safety
    /// Uses interior mutability via unsafe code. Only safe when the AST
    /// is accessed from a single thread.
    #[inline]
    pub fn set(&self, v: u32) {
        // SAFETY: AST nodes are single-threaded. This pattern is used
        // to allow setting node IDs after AST construction.
        unsafe {
            let ptr = std::ptr::addr_of!(self.value).cast_mut();
            std::ptr::write(ptr, v);
        }
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({})", self.value)
    }
}

impl Idx for NodeId {
    const MAX: usize = u32::MAX as usize;

    #[inline]
    unsafe fn from_usize_unchecked(idx: usize) -> Self {
        Self::new(idx as u32)
    }

    #[inline]
    fn index(self) -> usize {
        self.value as usize
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
