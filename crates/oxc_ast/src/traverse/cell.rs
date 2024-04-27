#![allow(dead_code)] // just for now
//! Cell type and token for traversing AST.
//!
//! Based on `GhostCell`.
//! All method implementations copied verbatim from original version by paper's authors
//! <https://gitlab.mpi-sws.org/FP/ghostcell/-/blob/master/ghostcell/src/lib.rs>
//! and `ghost_cell` crate `<https://docs.rs/ghost-cell>`.
//!
//! Only difference is that instead of using a lifetime to constrain the life of access tokens,
//! here we provide only an unsafe method `Token::new_unchecked` and the user must maintain
//! the invariant that only one token may be "in play" at same time
//! (see below for exactly what "in play" means).
//!
//! This alteration removes a lifetime, and avoids the unergonomic pattern of all the code that
//! works with a structure containing `GCell`s needing to be within a single closure.

use std::cell::UnsafeCell;

use oxc_allocator::Allocator;

/// Access token for traversing AST.
#[repr(transparent)]
pub struct Token(());

impl Token {
    /// Create new access token for traversing AST.
    ///
    /// It is imperative that any code operating on a single AST does not have access to more
    /// than 1 token. `GCell` uses this guarantee to make it impossible to obtain a `&mut`
    /// reference to any AST node while another reference exists. If more than 1 token is "in play",
    /// this guarantee can be broken, and may lead to undefined behavior.
    ///
    /// This function is used internally by `transform`, but probably should not be used elsewhere.
    ///
    /// It is permissable to create multiple tokens which are never used together on the same AST.
    /// In practice, this means it is possible to transform multiple ASTs on different threads
    /// simultaneously.
    ///
    /// If operating on multiple ASTs together (e.g. concatenating 2 files), then a single token
    /// must be used to access all the ASTs involved in the operation NOT 1 token per AST.
    ///
    /// # SAFETY
    /// Caller must ensure only a single token is used with any AST at one time.
    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn new_unchecked() -> Self {
        Self(())
    }
}

/// A cell type providing interior mutability, with aliasing rules enforced at compile time.
#[repr(transparent)]
pub struct GCell<T: ?Sized> {
    value: UnsafeCell<T>,
}

#[allow(dead_code)]
impl<T> GCell<T> {
    pub const fn new(value: T) -> Self {
        GCell { value: UnsafeCell::new(value) }
    }

    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

#[allow(dead_code, unused_variables)]
impl<T: ?Sized> GCell<T> {
    #[inline]
    #[allow(unsafe_code)]
    pub fn borrow<'a>(&'a self, tk: &'a Token) -> &'a T {
        // SAFETY: At any time there is only a single token for each AST.
        unsafe { &*self.value.get() }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn borrow_mut<'a>(&'a self, tk: &'a mut Token) -> &'a mut T {
        // SAFETY: At any time there is only a single token for each AST.
        unsafe { &mut *self.value.get() }
    }

    #[inline]
    pub const fn as_ptr(&self) -> *mut T {
        self.value.get()
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn get_mut(&mut self) -> &mut T {
        // SAFETY: We have an exclusive mutable access to the cell.
        unsafe { &mut *self.value.get() }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn from_mut(t: &mut T) -> &mut Self {
        // TODO: @overlookmotel make sure this safety documentation is correct.
        // SAFETY: A `GCell` always have the same alignment as its inner value,
        // It is only a compile-time abstraction, So we can safely transmute between them.
        unsafe { &mut *(t as *mut T as *mut Self) }
    }
}

#[allow(dead_code)]
impl<T> GCell<[T]> {
    #[inline]
    #[allow(unsafe_code)]
    pub fn as_slice_of_cells(&self) -> &[GCell<T>] {
        // TODO: @overlookmotel make sure this safety documentation is correct.
        // SAFETY: A `GCell` always have the same alignment as its inner value,
        // It is only a compile-time abstraction, So we can safely transmute between them.
        // There is no difference between slice of `GCell`s and `GCell` of slice.
        unsafe { &*(self as *const GCell<[T]> as *const [GCell<T>]) }
    }
}

#[allow(dead_code)]
impl<T> GCell<T> {
    #[inline]
    pub fn replace(&self, value: T, tk: &mut Token) -> T {
        std::mem::replace(self.borrow_mut(tk), value)
    }

    #[inline]
    pub fn take(&self, tk: &mut Token) -> T
    where
        T: Default,
    {
        self.replace(T::default(), tk)
    }
}

#[allow(dead_code)]
impl<T: Clone> GCell<T> {
    #[inline]
    pub fn clone(&self, tk: &Token) -> Self {
        GCell::new(self.borrow(tk).clone())
    }
}

impl<T: Default> Default for GCell<T> {
    #[inline]
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: ?Sized> AsMut<T> for GCell<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}

impl<T> From<T> for GCell<T> {
    fn from(t: T) -> Self {
        GCell::new(t)
    }
}

#[allow(unsafe_code)]
// SAFETY: `GhostCell` is `Send` + `Sync`, so `GCell` can be too
unsafe impl<T: ?Sized + Send> Send for GCell<T> {}

#[allow(unsafe_code)]
// SAFETY: `GhostCell` is `Send` + `Sync`, so `GCell` can be too
unsafe impl<T: ?Sized + Send + Sync> Sync for GCell<T> {}

/// Type alias for a shared ref to a `GCell`.
/// This is the interior-mutable equivalent to `oxc_allocator::Box`.
pub type SharedBox<'a, T> = &'a GCell<T>;

/// Type alias for a shared Vec
pub type SharedVec<'a, T> = oxc_allocator::Vec<'a, GCell<T>>;

/// Trait to sugar `GCell::from_mut(allocator.alloc(t))` to `allocator.galloc(t)`.
trait GCellAlloc {
    #[allow(clippy::mut_from_ref)]
    fn galloc<T>(&self, value: T) -> &mut GCell<T>;
}

impl GCellAlloc for Allocator {
    /// Allocate `T` into arena and return a `&mut GCell` to it
    #[inline]
    fn galloc<T>(&self, value: T) -> &mut GCell<T> {
        GCell::from_mut(self.alloc(value))
    }
}
