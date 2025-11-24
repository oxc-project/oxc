// All methods are 1 instruction or less
#![expect(clippy::inline_always)]

use std::ptr::NonNull;

use crate::Box;

/// Memory address of an AST node in arena.
//
// At present, this is a `usize`, but it could be a `NonZeroUsize` instead, so that `Address` gains a niche,
// which would reduce the size of `Option<Address>` from 16 bytes to 8 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Address(usize);

impl Address {
    /// Dummy address.
    ///
    /// Never equal to any real `Address`, but is equal to itself.
    pub const DUMMY: Self = Self(0);

    /// Get the memory address of a pointer to an AST node in arena.
    ///
    /// **This method is an escape hatch only.**
    /// Prefer using [`GetAddress::address`] or [`UnstableAddress::unstable_address`] instead,
    /// because they are more likely to produce a stable [`Address`].
    /// (Yes even `unstable_address` is more likely to produce a stable `Address` than this function!)
    ///
    /// If the AST node is in a [`Box`], the address is guaranteed to be a unique identifier
    /// for the duration of the arena's existence.
    ///
    /// But if the node is in a [`Vec`], then the `Address` may not remain accurate if the `Vec`
    /// is resized or has elements added or removed before this node.
    ///
    /// The pointer must point to an AST node in the arena (not on the stack),
    /// or the returned `Address` will be meaningless.
    ///
    /// If called with a reference, the reference must point to an AST node in the arena (not on the stack),
    /// or the returned `Address` will be meaningless. Be careful not to pass a double-reference to `from_ptr`,
    /// or the resulting `Address` will point to the reference itself, instead of the thing being referenced.
    ///
    /// ```ignore
    /// impl<'a> Visit<'a> for MyVisitor {
    ///     fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
    ///         // Correct - `address` is address of the `IdentifierReference`
    ///         let address = Address::from_ptr(ident);
    ///         // WRONG - `address` is address of `&IdentifierReference` reference itself, which is on the stack
    ///         let address = Address::from_ptr(&ident);
    ///     }
    /// }
    /// ```
    ///
    /// # SAFETY
    ///
    /// Pointer must be non-null.
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
    ///
    /// [`Box`]: crate::Box
    /// [`Vec`]: crate::Vec
    #[inline(always)] // Because it's a no-op
    pub unsafe fn from_ptr<T>(p: *const T) -> Self {
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
    #[inline(always)] // Because it's only 1 instruction
    fn address(&self) -> Address {
        let ptr = Box::as_non_null(self);
        Address(ptr.addr().get())
    }
}

impl GetAddress for Address {
    /// Address of an `Address` is itself.
    #[inline(always)] // Because it's a no-op
    fn address(&self) -> Address {
        *self
    }
}

/// Trait for getting the memory address of an AST node which is not necessarily stable.
///
/// See [`UnstableAddress::unstable_address`] for more details.
///
/// This trait is implemented for all AST struct types.
///
/// *DO NOT* implement this trait on any other type.
pub trait UnstableAddress {
    /// Get the memory [`Address`] of a reference to an AST node in arena, which is not necessarily stable.
    ///
    /// # Stable addresses
    ///
    /// It's ideal to obtain an `Address` for an AST node which is guaranteed to be stable for the life of the AST.
    ///
    /// Then you can reliably compare two `Address`es to determine if they refer to the same node,
    /// without any risk of the result being wrong because one or other of the nodes has moved in memory.
    ///
    /// Types which have a stable address:
    /// * [`Box<T>`]
    /// * AST enums where all variants are `Box`es (e.g. `Statement`, `Expression`)
    ///
    /// Some other types guarantee stability for a shorter period of time:
    /// * `oxc_ast::AstKind` - guaranteed stable address while `Semantic` is alive.
    /// * `oxc_traverse::Ancestor` - guaranteed stable address while traversing descendents of the ancestor.
    ///
    /// The above types all implement [`GetAddress::address`]. If you have access to one of these types,
    /// it's better to use `GetAddress::address` instead of this method.
    ///
    /// # Why this method exists
    ///
    /// Sometimes you only have access to a reference to an AST node, but you know from context that the node
    /// will not move in memory during the time you need the `Address` to remain stable.
    ///
    /// You can use this method in such cases, but you need to be careful. For correct behavior, you must ensure
    /// yourself that the node will not move in memory during the time you need the `Address` to remain accurate.
    ///
    /// When using this method, it is recommended to make a comment at the call site, explaining how you can prove
    /// that the `Address` will remain accurate (the AST node will not move) for the time period that you need it to be.
    ///
    /// # When a type does not move in memory
    ///
    /// If the AST is immutable (e.g. in `Visit` trait), then any node in the AST is statically positioned in memory.
    /// Therefore, in the linter, any reference to an AST node is guaranteed to have a stable `Address`.
    ///
    /// If an AST node is in `Vec`, then it'll remain at same memory address, but only as long as the `Vec` does
    /// not reallocate (e.g. by `Vec::push`, `Vec::extend`).
    ///
    /// This method will return a stable `Address` for any AST node in such circumstances.
    ///
    /// # Common pitfalls
    ///
    /// ## References to AST nodes on the stack
    ///
    /// Ensure that the reference passed to this method is to a node which is in the arena, *not* on the stack.
    ///
    /// ```ignore
    /// let binary_expr: BinaryExpression<'a> = get_owned_binary_expression_somehow();
    /// // WRONG: `binary_expr` is on the stack, so the `Address` will be meaningless
    /// let address = binary_expr.unstable_address();
    /// // More correct: `binary_expr` is in the arena.
    /// // Will have a stable `Address` as long as `vec` does not reallocate.
    /// let mut vec = Vec::new_in(&allocator);
    /// vec.push(binary_expr);
    /// let address = vec[0].unstable_address();
    /// ```
    ///
    /// ## AST nodes in `Vec`s
    ///
    /// ```ignore
    /// let mut vec: &mut Vec<BinaryExpression<'a>> = get_vec_somehow();
    ///
    /// let address = vec[0].unstable_address();
    /// vec.push(get_owned_binary_expression_somehow());
    /// let address_after_push = vec[0].unstable_address();
    ///
    /// // This assertion may or may not pass, depending on whether `push` caused the `Vec` to reallocate.
    /// // This depends on whether `vec` had spare capacity or not, prior to the `push` call.
    /// assert!(address_after_push == address);
    /// ```
    ///
    /// # Guardrails
    ///
    /// This method is less error-prone than [`Address::from_ptr`], because it provides a few guardrails:
    ///
    /// * [`UnstableAddress`] is only implemented on AST struct types, so you can't call it on a type which
    ///   it doesn't make sense to get the `Address` of.
    ///
    /// * You don't need to worry about passing it a double-reference (`&&T`), because this method will automatically
    ///   dereference as required.
    ///
    /// Even with these guardrails, usage of this method still requires care, for the reasons discussed above.
    ///
    /// [`Box<T>`]: crate::Box
    #[inline(always)] // Because it's a no-op
    fn unstable_address(&self) -> Address {
        let p = NonNull::from_ref(self);
        Address(p.addr().get())
    }
}
