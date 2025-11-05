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
    /// **This method is an escape hatch only.**
    /// Prefer using `GetAddress::address` instead, because it is more likely to produce a stable `Address`.
    ///
    /// If the AST node is in a `Box`, the address is guaranteed to be a unique identifier
    /// for the duration of the arena's existence.
    ///
    /// But if the node is in a `Vec`, then the `Address` may not remain accurate if the `Vec`
    /// is resized or has elements added or removed before this node.
    ///
    /// The pointer must point to an AST node in the arena (not on the stack),
    /// or the returned `Address` will be meaningless.
    ///
    /// # Example
    ///
    /// Demonstration of the difference between `Address::from_ptr` and `GetAddress::address`:
    ///
    /// ```ignore
    /// use oxc_allocator::{Address, GetAddress, Vec};
    /// use oxc_span::SPAN;
    ///
    /// // Create a `Vec<Statement>` containing a single `BlockStatement`
    /// let mut stmts = Vec::with_capacity_in(1, &allocator);
    /// stmts.push(ast_builder.statement_block(SPAN, Vec::new_in(&allocator)));
    ///
    /// let block_address = stmts[0].address();
    /// let stmt_address = Address::from_ptr(&stmts[0]);
    ///
    /// // Add another `Statement` to the `Vec`.
    /// // This causes the `Vec` to grow and reallocate.
    /// stmts.push(ast_builder.statement_empty(SPAN));
    ///
    /// let block_address_after_push = stmts[0].address();
    /// let stmt_address_after_push = Address::from_ptr(&stmts[0]);
    ///
    /// // Address of the `BlockStatement` is unchanged
    /// // (because the `Box`'s pointer still points to same memory location)
    /// assert!(block_address_after_push == block_address);
    /// // Address of the `Statement` has changed
    /// // (because the `Vec` reallocated, so its contents have moved in memory)
    /// assert!(stmt_address_after_push != stmt_address);
    ///
    /// // Insert a new `Statement` at start of the `Vec`.
    /// // The `BlockStatement` is now at index 1.
    /// stmts.insert(0, ast_builder.statement_empty(SPAN));
    ///
    /// let block_address_after_insert = stmts[1].address();
    /// let stmt_address_after_insert = Address::from_ptr(&stmts[1]);
    ///
    /// // Address of the `BlockStatement` is still unchanged
    /// assert!(block_address_after_insert == block_address_after_push);
    /// // Address of the `Statement` has changed again
    /// assert!(stmt_address_after_insert != stmt_address_after_push);
    /// ```
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
