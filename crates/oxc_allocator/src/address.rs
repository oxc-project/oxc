use std::ptr;

use crate::Box;

/// Memory address of an AST node in arena.
///
/// `Address` is generated from a `Box<T>`.
/// AST nodes in a `Box` in an arena are guaranteed to never move in memory,
/// so this address acts as a unique identifier for the duration of the arena's existence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address(usize);

/// Trait for getting the memory address of an AST node.
pub trait GetAddress {
    /// Get the memory address of a value allocated in the arena.
    fn address(&self) -> Address;
}

impl<'a, T> GetAddress for Box<'a, T> {
    /// Get the memory address of a value allocated in the arena.
    #[inline]
    fn address(&self) -> Address {
        Address(ptr::addr_of!(**self) as usize)
    }
}
