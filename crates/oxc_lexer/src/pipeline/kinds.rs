use std::cell::Cell;

use crate::token::TokenKind;

/// One token kind byte per source byte.
#[derive(Clone, Copy)]
pub(super) struct Kinds<'a>(&'a [Cell<u8>]);

impl<'a> Kinds<'a> {
    /// Every byte of kinds must be a declared TokenKind discriminant.
    pub(super) unsafe fn new(kinds: &'a mut [u8]) -> Self {
        Self(Cell::from_mut(kinds).as_slice_of_cells())
    }

    /// For the kernels that read and write through it unchecked.
    pub(super) fn as_ptr(self) -> *mut u8 {
        self.0.as_ptr().cast::<u8>().cast_mut()
    }

    /// The first len kinds, checked once, for code that reads or writes below len.
    #[inline(always)]
    pub(super) fn first(self, len: usize) -> Self {
        Self(&self.0[..len])
    }

    #[inline(always)]
    pub(super) fn get(self, i: usize) -> u8 {
        self.0[i].get()
    }

    #[inline(always)]
    pub(super) fn set(self, i: usize, kind: TokenKind) {
        self.0[i].set(kind as u8);
    }

    /// set without the bounds check, so i must be below the kinds' length.
    #[inline(always)]
    pub(super) unsafe fn set_unchecked(self, i: usize, kind: TokenKind) {
        // SAFETY: the caller keeps i below the kinds' length.
        unsafe { self.0.get_unchecked(i) }.set(kind as u8);
    }
}
