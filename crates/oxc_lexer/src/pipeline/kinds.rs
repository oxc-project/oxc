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

    /// The number of kinds.
    #[inline(always)]
    pub(super) fn len(self) -> usize {
        self.0.len()
    }

    #[inline(always)]
    pub(super) fn get(self, i: usize) -> u8 {
        self.0[i].get()
    }

    #[inline(always)]
    pub(super) fn set(self, i: usize, kind: TokenKind) {
        self.0[i].set(kind as u8);
    }

    /// Set the kind at i if cond, without a branch.
    #[inline(always)]
    pub(super) fn set_if(self, i: usize, cond: bool, kind: TokenKind) {
        let cell = &self.0[i];
        let mask = 0u8.wrapping_sub(u8::from(cond));
        cell.set((cell.get() & !mask) | (kind as u8 & mask));
    }
}
