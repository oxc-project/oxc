pub mod generated;
pub mod impls;
mod iterator;
mod node;

use std::{cell::Cell, ptr};

use oxc_allocator::Allocator;

pub use generated::ast_nodes::AstNodes;
pub use iterator::AstNodeIterator;
pub use node::AstNode;

thread_local! {
    static ALLOCATOR: Cell<Option<*const Allocator>> = const { Cell::new(None) };
}

/// Sets the thread-local allocator for use during AST node operations.
///
/// # Safety
/// The caller must ensure that the allocator outlives all uses of `allocator()`.
/// Typically this is called at the start of formatting and cleared after.
#[inline]
pub fn set_allocator(allocator: &Allocator) {
    ALLOCATOR.with(|cell| cell.set(Some(ptr::from_ref::<Allocator>(allocator))));
}

/// Gets a reference to the thread-local allocator.
///
/// # Panics
/// Panics if no allocator has been set via `set_allocator`.
#[inline]
pub fn allocator<'a>() -> &'a Allocator {
    ALLOCATOR.with(|cell| {
        // SAFETY: The caller of `set_allocator` guarantees the allocator outlives this access.
        unsafe { &*cell.get().expect("Allocator not set. Call set_allocator first.") }
    })
}
