//! RWU (Read-Write-Use) table for tracking variable liveness in control flow analysis.
//!
//! This is adapted from the Rust compiler's liveness analysis but with modifications
//! for JavaScript/TypeScript semantics.
//!
//! ## Difference from rustc
//!
//! The rustc implementation uses abstract `LiveNode` and `Variable` types that represent
//! program points and variables within a specific function context. This implementation
//! directly uses `BasicBlockId` and `SymbolId` from oxc's existing infrastructure.
//!
//! **Important**: This means each RWU entry represents liveness at the basic block level,
//! not at individual program points within a block. This is coarser-grained than rustc's
//! approach but simpler to implement given oxc's current CFG structure.

use std::iter;

use oxc_cfg::BasicBlockId;
use oxc_syntax::symbol::SymbolId;

/// Represents the Read, Write, and Use flags for a variable at a specific program point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadWriteUseData {
    /// Whether the variable is read at this point
    pub reader: bool,
    /// Whether the variable is written at this point
    pub writer: bool,
    /// Whether the variable is used (meaningful read, not just for generating a new value)
    pub used: bool,
}

/// A table tracking Read/Write/Use information for variables across basic blocks.
///
/// This is conceptually like a `Vec<Vec<RWU>>` but uses a compact representation.
/// Each word represents 2 different `RWU`s packed together. Each packed RWU
/// is stored in 4 bits: a reader bit, a writer bit, a used bit and a padding bit.
///
/// ## Memory Layout
///
/// The table stores `blocks * vars` RWU entries. Each RWU takes 4 bits (half a byte).
/// The data for each block is contiguous and starts at a word boundary.
///
/// Example for 3 blocks and 5 variables:
/// ```text
/// Block 0: [var0|var1] [var2|var3] [var4|pad]
/// Block 1: [var0|var1] [var2|var3] [var4|pad]
/// Block 2: [var0|var1] [var2|var3] [var4|pad]
/// ```
pub struct RWUTable {
    /// Total number of basic blocks
    blocks: usize,
    /// Total number of variables (symbols)
    vars: usize,
    /// Compressed representation of RWUs
    words: Vec<u8>,
    /// Number of words per each basic block
    block_words: usize,
}

impl RWUTable {
    const RWU_READER: u8 = 0b0001;
    const RWU_WRITER: u8 = 0b0010;
    const RWU_USED: u8 = 0b0100;
    const RWU_MASK: u8 = 0b1111;

    /// Size of packed RWU in bits
    const RWU_BITS: usize = 4;
    /// Size of a word in bits
    const WORD_BITS: usize = u8::BITS as usize;
    /// Number of packed RWUs that fit into a single word
    const WORD_RWU_COUNT: usize = Self::WORD_BITS / Self::RWU_BITS;

    /// Create a new RWU table for the given number of basic blocks and variables
    pub fn new(blocks: usize, vars: usize) -> Self {
        let block_words = vars.div_ceil(Self::WORD_RWU_COUNT);
        Self { blocks, vars, block_words, words: vec![0u8; block_words * blocks] }
    }

    /// Get the word index and bit shift for a given block and variable
    fn word_and_shift(&self, block: BasicBlockId, var: SymbolId) -> (usize, u32) {
        let block_idx = block.index();
        let var_idx = var.index();

        assert!(block_idx < self.blocks, "block index out of bounds");
        assert!(var_idx < self.vars, "variable index out of bounds");

        let word = var_idx / Self::WORD_RWU_COUNT;
        let shift = Self::RWU_BITS * (var_idx % Self::WORD_RWU_COUNT);
        (block_idx * self.block_words + word, u32::try_from(shift).unwrap())
    }

    /// Get mutable references to two different block rows
    ///
    /// # Safety
    ///
    /// This function uses unsafe code to create two mutable slices from the same vector.
    /// This is sound because:
    ///
    /// 1. The precondition `a != b` ensures the slices don't overlap
    /// 2. Both indices are bounds-checked via assertions
    /// 3. Each slice is exactly `block_words` elements, and blocks are stored contiguously
    /// 4. The memory layout ensures `a_start..a_start+block_words` and
    ///    `b_start..b_start+block_words` are disjoint when `a != b`
    ///
    /// # Panics
    ///
    /// Panics if `a` or `b` are out of bounds, or if `a == b`.
    fn pick2_rows_mut(&mut self, a: BasicBlockId, b: BasicBlockId) -> (&mut [u8], &mut [u8]) {
        let a_idx = a.index();
        let b_idx = b.index();

        assert!(a_idx < self.blocks, "block a index out of bounds");
        assert!(b_idx < self.blocks, "block b index out of bounds");
        assert_ne!(a, b, "cannot pick same block twice");

        let a_start = a_idx * self.block_words;
        let b_start = b_idx * self.block_words;

        // SAFETY: See function-level safety comment
        unsafe {
            let ptr = self.words.as_mut_ptr();
            (
                std::slice::from_raw_parts_mut(ptr.add(a_start), self.block_words),
                std::slice::from_raw_parts_mut(ptr.add(b_start), self.block_words),
            )
        }
    }

    /// Copy RWU data from src block to dst block
    pub fn copy(&mut self, dst: BasicBlockId, src: BasicBlockId) {
        if dst == src {
            return;
        }

        let (dst_row, src_row) = self.pick2_rows_mut(dst, src);
        dst_row.copy_from_slice(src_row);
    }

    /// Set dst to the union of dst and src, returns true if dst was changed
    pub fn union(&mut self, dst: BasicBlockId, src: BasicBlockId) -> bool {
        if dst == src {
            return false;
        }

        let mut changed = false;
        let (dst_row, src_row) = self.pick2_rows_mut(dst, src);
        for (dst_word, src_word) in iter::zip(dst_row, &*src_row) {
            let old = *dst_word;
            let new = *dst_word | src_word;
            *dst_word = new;
            changed |= old != new;
        }
        changed
    }

    /// Get whether the variable is read at the given block
    pub fn get_reader(&self, block: BasicBlockId, var: SymbolId) -> bool {
        let (word, shift) = self.word_and_shift(block, var);
        (self.words[word] >> shift) & Self::RWU_READER != 0
    }

    /// Get whether the variable is used at the given block
    pub fn get_used(&self, block: BasicBlockId, var: SymbolId) -> bool {
        let (word, shift) = self.word_and_shift(block, var);
        (self.words[word] >> shift) & Self::RWU_USED != 0
    }

    /// Get the full RWU state for a variable at a given block
    pub fn get(&self, block: BasicBlockId, var: SymbolId) -> ReadWriteUseData {
        let (word, shift) = self.word_and_shift(block, var);
        let rwu_packed = self.words[word] >> shift;
        ReadWriteUseData {
            reader: rwu_packed & Self::RWU_READER != 0,
            writer: rwu_packed & Self::RWU_WRITER != 0,
            used: rwu_packed & Self::RWU_USED != 0,
        }
    }

    /// Set the RWU state for a variable at a given block
    pub fn set(&mut self, block: BasicBlockId, var: SymbolId, rwu: ReadWriteUseData) {
        let mut packed = 0;
        if rwu.reader {
            packed |= Self::RWU_READER;
        }
        if rwu.writer {
            packed |= Self::RWU_WRITER;
        }
        if rwu.used {
            packed |= Self::RWU_USED;
        }

        let (word, shift) = self.word_and_shift(block, var);
        let word_ref = &mut self.words[word];
        *word_ref = (*word_ref & !(Self::RWU_MASK << shift)) | (packed << shift);
    }
}
