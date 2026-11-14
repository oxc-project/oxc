use std::cell::Cell;

use oxc_simd::{Cpu, cpu_multiversion};

/// One bit per source byte, a u64 per 64-byte block, shared by the stages that refine it.
#[derive(Clone, Copy)]
pub(super) struct Bitmap<'a>(&'a [Cell<u64>]);

impl<'a> Bitmap<'a> {
    pub(super) fn new(blocks: &'a mut [u64]) -> Self {
        Self(Cell::from_mut(blocks).as_slice_of_cells())
    }

    /// For the kernels that read and write the blocks through it unchecked.
    pub(super) fn as_ptr(self) -> *mut u64 {
        self.0.as_ptr().cast::<u64>().cast_mut()
    }

    /// The first count blocks, checked once, for code that reads or writes below count.
    #[inline(always)]
    pub(super) fn first(self, count: usize) -> Self {
        Self(&self.0[..count])
    }

    #[cpu_multiversion]
    #[inline(always)]
    pub(super) fn block(self, block_idx: usize, _cpu: impl Cpu) -> u64 {
        self.0[block_idx].get()
    }

    #[cpu_multiversion]
    #[inline(always)]
    pub(super) fn get(self, i: usize, _cpu: impl Cpu) -> bool {
        (self.0[i >> 6].get() >> (i & 63)) & 1 != 0
    }

    /// First set bit at or after i, or n.
    #[cpu_multiversion]
    #[inline(always)]
    pub(super) fn next1(self, i: usize, n: usize, _cpu: impl Cpu) -> usize {
        let mut w = i >> 6;
        let x = self.0[w].get() & !((1u64 << (i & 63)).wrapping_sub(1));
        if x != 0 {
            return ((w << 6) + x.trailing_zeros() as usize).min(n);
        }
        w += 1;
        while (w << 6) < n {
            let x = self.0[w].get();
            if x != 0 {
                return ((w << 6) + x.trailing_zeros() as usize).min(n);
            }
            w += 1;
        }
        n
    }

    /// First clear bit at or after i, or n if bits i..n are all set.
    #[cpu_multiversion]
    #[inline(always)]
    pub(super) fn next0(self, i: usize, n: usize, _cpu: impl Cpu) -> usize {
        let mut w = i >> 6;
        // Reads one block past the source when i == n and n is a multiple of 64.
        let mut inv = !self.0[w].get() & !((1u64 << (i & 63)).wrapping_sub(1));
        while inv == 0 {
            w += 1;
            if (w << 6) >= n {
                return n;
            }
            inv = !self.0[w].get();
        }
        let r = (w << 6) + inv.trailing_zeros() as usize;
        if r < n { r } else { n }
    }

    /// Last set bit before p.
    #[cpu_multiversion]
    #[inline(always)]
    pub(super) fn prev1(self, p: usize, _cpu: impl Cpu) -> Option<usize> {
        if p == 0 {
            return None;
        }
        let i = p - 1;
        let mut w = i >> 6;
        let lim = i & 63;
        let mask = if lim == 63 { !0u64 } else { (1u64 << (lim + 1)) - 1 };
        let x = self.0[w].get() & mask;
        if x != 0 {
            return Some((w << 6) + (63 - x.leading_zeros() as usize));
        }
        while w > 0 {
            w -= 1;
            let x = self.0[w].get();
            if x != 0 {
                return Some((w << 6) + (63 - x.leading_zeros() as usize));
            }
        }
        None
    }

    #[cpu_multiversion]
    #[inline(always)]
    pub(super) fn set(self, i: usize, _cpu: impl Cpu) {
        let block = &self.0[i >> 6];
        block.set(block.get() | (1u64 << (i & 63)));
    }

    /// set without the bounds check, so i must be below the bitmap's length in bits.
    #[cpu_multiversion]
    #[inline(always)]
    pub(super) unsafe fn set_unchecked(self, i: usize, _cpu: impl Cpu) {
        // SAFETY: the caller keeps i below the bitmap's length in bits.
        let block = unsafe { self.0.get_unchecked(i >> 6) };
        block.set(block.get() | (1u64 << (i & 63)));
    }

    #[cpu_multiversion]
    #[inline(always)]
    pub(super) fn clear(self, i: usize, _cpu: impl Cpu) {
        let block = &self.0[i >> 6];
        block.set(block.get() & !(1u64 << (i & 63)));
    }

    /// clear without the bounds check, so i must be below the bitmap's length in bits.
    #[cpu_multiversion]
    #[inline(always)]
    pub(super) unsafe fn clear_unchecked(self, i: usize, _cpu: impl Cpu) {
        // SAFETY: the caller keeps i below the bitmap's length in bits.
        let block = unsafe { self.0.get_unchecked(i >> 6) };
        block.set(block.get() & !(1u64 << (i & 63)));
    }

    /// Clears bits a..=b, or nothing if a > b.
    #[cpu_multiversion]
    #[inline(always)]
    pub(super) fn clear_range(self, a: usize, b: usize, _cpu: impl Cpu) {
        if a > b {
            return;
        }
        clear_bits(&self.0[a >> 6..=b >> 6], a, b);
    }

    /// clear_range without the bounds check, so b must be below the bitmap's length in bits.
    #[cpu_multiversion]
    #[inline(always)]
    pub(super) unsafe fn clear_range_unchecked(self, a: usize, b: usize, _cpu: impl Cpu) {
        if a > b {
            return;
        }
        // SAFETY: the caller keeps b, and so a, below the bitmap's length in bits.
        clear_bits(unsafe { self.0.get_unchecked(a >> 6..=b >> 6) }, a, b);
    }

    #[cpu_multiversion]
    #[inline(always)]
    pub(super) fn any(self, block_count: usize, _cpu: impl Cpu) -> bool {
        // One test per 16 blocks keeps the scan cheap when no block has a misc byte.
        let (runs, rest) = self.0[..block_count].as_chunks::<16>();
        runs.iter().any(|run| run.iter().fold(0, |acc, block| acc | block.get()) != 0)
            || rest.iter().any(|block| block.get() != 0)
    }
}

/// Clears bits a..=b in blocks, the blocks from the one holding bit a to the one holding bit b.
#[inline(always)]
fn clear_bits(blocks: &[Cell<u64>], a: usize, b: usize) {
    let lo = !0u64 << (a & 63);
    let hi = !0u64 >> (63 - (b & 63));
    match blocks {
        [block] => block.set(block.get() & !(lo & hi)),
        [first, middle @ .., last] => {
            first.set(first.get() & !lo);
            for block in middle {
                block.set(0);
            }
            last.set(last.get() & !hi);
        }
        [] => {}
    }
}

/// The per-byte bitmaps classify fills in and the later stages refine.
#[derive(Clone, Copy)]
pub(super) struct Bitmaps<'a> {
    pub token_start: Bitmap<'a>,
    /// Identifier chars, including every non-ASCII byte.
    pub word: Bitmap<'a>,
    pub keyword_initial: Bitmap<'a>,
    /// Chars a multi-byte operator is made of.
    pub op_char: Bitmap<'a>,
    pub digit: Bitmap<'a>,
    pub dot: Bitmap<'a>,
    /// Bytes the misc passes look at: hash, backslash, non-ASCII, vertical tab and form feed.
    pub misc: Bitmap<'a>,
}

impl Bitmaps<'_> {
    /// The first count blocks of each bitmap.
    #[inline(always)]
    pub(super) fn first(&self, count: usize) -> Self {
        Self {
            token_start: self.token_start.first(count),
            word: self.word.first(count),
            keyword_initial: self.keyword_initial.first(count),
            op_char: self.op_char.first(count),
            digit: self.digit.first(count),
            dot: self.dot.first(count),
            misc: self.misc.first(count),
        }
    }

    #[inline(always)]
    pub(super) fn clear_block(&self, block_idx: usize) {
        let all = [
            self.token_start,
            self.word,
            self.keyword_initial,
            self.op_char,
            self.digit,
            self.dot,
            self.misc,
        ];
        for bitmap in all {
            bitmap.0[block_idx].set(0);
        }
    }
}
