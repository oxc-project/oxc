use std::ptr;

use crate::Box;

/// Memory address of an AST node in arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address(usize);

impl Address {
    /// Dummy address.
    ///
    /// Never equal to any real `Address`, but is equal to itself.
    pub const DUMMY: Self = Self(0);

    /// Get the memory address of a pointer to an AST node in arena.
    ///
    /// The pointer must point to an AST node in the arena (not on the stack),
    /// or the returned `Address` will be meaningless.
    ///
    /// If the AST node is in a `Box`, the address is guaranteed to be a unique identifier
    /// for the duration of the arena's existence.
    /// If the node is in a `Vec`, then the `Address` may not remain accurate if the `Vec`
    /// is resized or has elements added or removed before this node.
    #[inline]
    pub fn from_ptr<T>(p: *const T) -> Self {
        Self(p as usize)
    }
}

/// Trait for getting the memory address of an AST node.
pub trait GetAddress {
    /// Get the memory address of a value allocated in the arena.
    fn address(&self) -> Address;
}

impl<T> GetAddress for Box<'_, T> {
    /// Get the memory address of a value allocated in the arena.
    ///
    /// AST nodes in a `Box` in an arena are guaranteed to never move in memory,
    /// so this address acts as a unique identifier for the duration of the arena's existence.
    #[inline]
    fn address(&self) -> Address {
        Address::from_ptr(ptr::addr_of!(**self))
    }
}

impl GetAddress for Address {
    /// Address of an `Address` is itself.
    #[inline]
    fn address(&self) -> Address {
        *self
    }
}
