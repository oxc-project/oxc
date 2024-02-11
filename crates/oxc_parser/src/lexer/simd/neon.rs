use crate::lexer::source::Source;
use core::arch::aarch64::*;

pub(crate) const ALIGNMENT: usize = 16;

pub struct LookupTable<const N: usize> {}

impl<const N: usize> LookupTable<N> {
    pub fn new(delimiters: [u8; N]) -> Self {
        todo!()
    }

    #[inline]
    pub fn match_vectored(&self, source: &Source) -> (Option<usize>, usize) {
        todo!()
    }
}

const fn build_bitmap<const N: usize>(delimiters: [u8; N]) -> ([u8; 16], [u8; 16]) {
    todo!()
}
