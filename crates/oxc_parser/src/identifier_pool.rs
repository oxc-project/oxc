//! Pre-allocated pools for identifier nodes.
//!
//! This module provides a pool for pre-allocating blocks of identifier nodes
//! to reduce individual allocation overhead. Since identifiers are the most
//! commonly allocated nodes in an AST, batching allocations provides a
//! performance improvement.
//!
//! # Safety
//!
//! This module uses `unsafe` to create `Box` from pre-allocated memory.
//! The safety is ensured by:
//! 1. Memory is allocated from the arena allocator which guarantees validity for lifetime `'a`
//! 2. Each slot is only handed out once (index is incremented after each allocation)
//! 3. Raw pointers are used to avoid aliased mutable references
//! 4. The `Box` returned has the same lifetime as the arena

use std::{cell::Cell, ptr::NonNull};

use oxc_allocator::{Allocator, Box};
use oxc_ast::ast::{BindingIdentifier, IdentifierName, IdentifierReference};
use oxc_span::{Atom, Span};

/// Block size for pre-allocated identifier pools.
/// 128 nodes * 32 bytes = 4KB (one memory page for IdentifierReference/BindingIdentifier)
const BLOCK_SIZE: usize = 128;

/// A pool for pre-allocating identifier nodes in blocks.
///
/// Instead of allocating identifiers one at a time, this pool allocates
/// blocks of 128 slots at once and dispenses them individually. This reduces
/// allocation overhead for the most common AST nodes.
pub struct IdentifierPool<'a> {
    allocator: &'a Allocator,

    // IdentifierReference pool - using raw pointers to avoid aliasing issues
    identifier_reference_ptr: Option<NonNull<IdentifierReference<'a>>>,
    identifier_reference_remaining: usize,

    // IdentifierName pool
    identifier_name_ptr: Option<NonNull<IdentifierName<'a>>>,
    identifier_name_remaining: usize,

    // BindingIdentifier pool
    binding_identifier_ptr: Option<NonNull<BindingIdentifier<'a>>>,
    binding_identifier_remaining: usize,
}

impl<'a> IdentifierPool<'a> {
    /// Create a new identifier pool.
    ///
    /// Blocks are allocated lazily on first use.
    #[inline]
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            identifier_reference_ptr: None,
            identifier_reference_remaining: 0,
            identifier_name_ptr: None,
            identifier_name_remaining: 0,
            binding_identifier_ptr: None,
            binding_identifier_remaining: 0,
        }
    }

    /// Allocate an `IdentifierReference` from the pool.
    ///
    /// If the current block is exhausted or not yet allocated, a new block
    /// of 128 slots is allocated.
    #[inline]
    pub fn alloc_identifier_reference(
        &mut self,
        span: Span,
        name: Atom<'a>,
    ) -> Box<'a, IdentifierReference<'a>> {
        if self.identifier_reference_remaining == 0 {
            self.allocate_identifier_reference_block();
        }

        let ptr = self.identifier_reference_ptr.unwrap();
        self.identifier_reference_remaining -= 1;

        if self.identifier_reference_remaining > 0 {
            // SAFETY: ptr + 1 is still within the allocated block since remaining > 0
            self.identifier_reference_ptr =
                Some(unsafe { NonNull::new_unchecked(ptr.as_ptr().add(1)) });
        }

        // SAFETY:
        // 1. `allocate_identifier_reference_block` ensures ptr is valid and points to arena memory
        // 2. We decrement remaining before returning, so each slot is only used once
        // 3. Arena memory is valid for lifetime 'a
        // 4. IdentifierReference has no Drop impl (enforced by Box::new_in compile-time check)
        let slot = unsafe { ptr.as_ptr().as_mut().unwrap_unchecked() };
        slot.span = span;
        slot.name = name.into();
        slot.reference_id = Cell::new(None);

        // SAFETY: ptr is valid, aligned, and points to initialized data in the arena
        unsafe { Box::from_non_null(ptr) }
    }

    /// Allocate an `IdentifierName` from the pool.
    #[inline]
    pub fn alloc_identifier_name(
        &mut self,
        span: Span,
        name: Atom<'a>,
    ) -> Box<'a, IdentifierName<'a>> {
        if self.identifier_name_remaining == 0 {
            self.allocate_identifier_name_block();
        }

        let ptr = self.identifier_name_ptr.unwrap();
        self.identifier_name_remaining -= 1;

        if self.identifier_name_remaining > 0 {
            // SAFETY: ptr + 1 is still within the allocated block
            self.identifier_name_ptr = Some(unsafe { NonNull::new_unchecked(ptr.as_ptr().add(1)) });
        }

        // SAFETY: ptr points to valid, properly aligned memory in the arena
        let slot = unsafe { ptr.as_ptr().as_mut().unwrap_unchecked() };
        slot.span = span;
        slot.name = name.into();

        // SAFETY: ptr is valid, aligned, and points to initialized data in the arena
        unsafe { Box::from_non_null(ptr) }
    }

    /// Allocate a `BindingIdentifier` from the pool.
    #[inline]
    pub fn alloc_binding_identifier(
        &mut self,
        span: Span,
        name: Atom<'a>,
    ) -> Box<'a, BindingIdentifier<'a>> {
        if self.binding_identifier_remaining == 0 {
            self.allocate_binding_identifier_block();
        }

        let ptr = self.binding_identifier_ptr.unwrap();
        self.binding_identifier_remaining -= 1;

        if self.binding_identifier_remaining > 0 {
            // SAFETY: ptr + 1 is still within the allocated block
            self.binding_identifier_ptr =
                Some(unsafe { NonNull::new_unchecked(ptr.as_ptr().add(1)) });
        }

        // SAFETY: ptr points to valid, properly aligned memory in the arena
        let slot = unsafe { ptr.as_ptr().as_mut().unwrap_unchecked() };
        slot.span = span;
        slot.name = name.into();
        slot.symbol_id = Cell::new(None);

        // SAFETY: ptr is valid, aligned, and points to initialized data in the arena
        unsafe { Box::from_non_null(ptr) }
    }

    /// Allocate a new block of `IdentifierReference` slots.
    #[cold]
    #[inline(never)]
    fn allocate_identifier_reference_block(&mut self) {
        let block: &mut [IdentifierReference<'a>] =
            self.allocator.alloc_slice_fill_with(BLOCK_SIZE, |_| IdentifierReference {
                span: Span::default(),
                name: Atom::from("").into(),
                reference_id: Cell::new(None),
            });
        self.identifier_reference_ptr = Some(NonNull::from(&mut block[0]));
        self.identifier_reference_remaining = BLOCK_SIZE;
    }

    /// Allocate a new block of `IdentifierName` slots.
    #[cold]
    #[inline(never)]
    fn allocate_identifier_name_block(&mut self) {
        let block: &mut [IdentifierName<'a>] =
            self.allocator.alloc_slice_fill_with(BLOCK_SIZE, |_| IdentifierName {
                span: Span::default(),
                name: Atom::from("").into(),
            });
        self.identifier_name_ptr = Some(NonNull::from(&mut block[0]));
        self.identifier_name_remaining = BLOCK_SIZE;
    }

    /// Allocate a new block of `BindingIdentifier` slots.
    #[cold]
    #[inline(never)]
    fn allocate_binding_identifier_block(&mut self) {
        let block: &mut [BindingIdentifier<'a>] =
            self.allocator.alloc_slice_fill_with(BLOCK_SIZE, |_| BindingIdentifier {
                span: Span::default(),
                name: Atom::from("").into(),
                symbol_id: Cell::new(None),
            });
        self.binding_identifier_ptr = Some(NonNull::from(&mut block[0]));
        self.binding_identifier_remaining = BLOCK_SIZE;
    }
}
