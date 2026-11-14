use std::ops::{Index, Range};

use oxc_simd::{Cpu, cpu_multiversion};

use crate::{PAD, pipeline::bytes::unicode_ws_len};

/// Source text followed by PAD readable bytes, zero unless the source was cut short with until.
#[derive(Clone, Copy)]
pub(super) struct Source<'a> {
    padded: &'a [u8],
}

impl<'a> Source<'a> {
    /// The first len bytes of src, which must extend at least PAD bytes past len.
    pub(super) fn new(src: &'a [u8], len: usize) -> Self {
        Self { padded: &src[..len + PAD] }
    }

    // checked_sub tells LLVM that padded.len() >= PAD, so reads guarded by i < len skip the check.
    pub(super) fn len(self) -> usize {
        self.padded.len().checked_sub(PAD).unwrap()
    }

    pub(super) fn block_count(self) -> usize {
        self.len().div_ceil(64)
    }

    /// The source without its padding.
    pub(super) fn bytes(self) -> &'a [u8] {
        &self.padded[..self.len()]
    }

    pub(super) fn padded(self) -> &'a [u8] {
        self.padded
    }

    pub(super) fn as_ptr(self) -> *const u8 {
        self.padded.as_ptr()
    }

    /// The source cut short at end: the padding is then the source bytes that follow.
    pub(super) fn until(self, end: usize) -> Self {
        Self { padded: &self.padded[..end + PAD] }
    }

    /// Byte length (2 or 3) of the multi-byte WhiteSpace or LineTerminator at i, or 0.
    #[cpu_multiversion]
    #[inline(always)]
    pub(super) fn unicode_ws_len_at(self, i: usize, cpu: impl Cpu) -> usize {
        let c = &self.padded[i..i + 3];
        unicode_ws_len(c[0], c[1], c[2], cpu)
    }

    /// The 4 bytes at i as a little-endian u32.
    pub(super) fn u32_at(self, i: usize) -> u32 {
        u32::from_le_bytes(self.padded[i..i + 4].try_into().unwrap())
    }

    /// The 8 bytes at i as a little-endian u64.
    pub(super) fn u64_at(self, i: usize) -> u64 {
        u64::from_le_bytes(self.padded[i..i + 8].try_into().unwrap())
    }
}

impl Index<usize> for Source<'_> {
    type Output = u8;

    fn index(&self, i: usize) -> &u8 {
        &self.padded[i]
    }
}

impl Index<Range<usize>> for Source<'_> {
    type Output = [u8];

    fn index(&self, range: Range<usize>) -> &[u8] {
        &self.padded[range]
    }
}
