use crate::{
    cpu::{Avx2, cpu_version},
    simd::Vec256,
    token::tk,
};

use super::{
    luts::{DIGIT_MASK, KWINIT_MASK, MERGED_LUTS, OPCH_MASK, WORD_LUTS, WORD_MASK, WS_MASK},
    punct::{PUNCT_HASH_HI, PUNCT_HASH_KINDS, PUNCT_HASH_LO},
};

#[cpu_version(Avx2)]
#[inline(never)]
pub(super) unsafe fn classify_avx2(
    is_ts: bool,
    src_ptr: *const u8,
    src_len: usize,
    word_bitmap_ptr: *mut u64,
    token_start_bitmap_ptr: *mut u64,
    keyword_initial_bitmap_ptr: *mut u64,
    op_char_bitmap_ptr: *mut u64,
    digit_bitmap_ptr: *mut u64,
    dot_bitmap_ptr: *mut u64,
    misc_bitmap_ptr: *mut u64,
    kinds_ptr: *mut u8,
    cpu: Avx2,
) {
    // Process `ceil(src_len / 64)` blocks.
    // When `src_len` is not a multiple of 64, the final block overreads up to 63 bytes
    // into the caller-guaranteed zeroed-padded region of `PAD` bytes after the source.
    // The bitmap bits corresponding to these padding bytes are masked off at the end.
    // This avoids the need for a byte-at-a-time scalar tail, which cost ~18 cyc per tail byte
    // (up to ~1.1k cyc when `src_len % 64` is near 63), and dominated small-file lexing.
    let block_count = src_len.div_ceil(64);

    // Loop counters.
    // LLVM reworks these counters into:
    // - Increasing counter, starting at 0 and incrementing in steps of 8.
    //   Used for indexing into source, kinds, and bitmaps.
    // - Decreasing counter, starting at `block_count` and decrementing in steps of 1.
    //   Used for the loop end check - loop exits when this counter reaches 0.
    let mut src_idx = 0usize;
    let mut block_idx = 0usize;

    // Load lookup tables.
    // The merged LUT variants differ only in the keyword-initial bits.
    // The selection happens once, outside the loop.
    let punct_hash_lo_vec = Vec256::splat_v128(PUNCT_HASH_LO.read(cpu));
    let punct_hash_hi_vec = Vec256::splat_v128(PUNCT_HASH_HI.read(cpu));
    let punct_kind_lut_halves = PUNCT_HASH_KINDS.as_halves();
    let punct_kind_lut_top_vec = Vec256::splat_v128(punct_kind_lut_halves[0].read(cpu));
    let punct_kind_lut_bot_vec = Vec256::splat_v128(punct_kind_lut_halves[1].read(cpu));

    let merged_lut_lo = if is_ts { MERGED_LUTS.lo_ts } else { MERGED_LUTS.lo };
    let merged_lut_lo_vec = Vec256::splat_v128(merged_lut_lo.read(cpu));
    let merged_lut_hi_vec = Vec256::splat_v128(MERGED_LUTS.hi.read(cpu));

    let word_lut_lo_vec = Vec256::splat_v128(WORD_LUTS.lo.read(cpu));
    let word_lut_hi_vec = Vec256::splat_v128(WORD_LUTS.hi.read(cpu));

    // The last bit of the previous block's `word_bits` and `whitespace_bits` (0 or 1).
    // Whether a byte starts a token depends on the byte before it, so these carry the flags
    // for previous block's last byte over to the next block.
    let mut word_carry: u64 = 0;
    let mut whitespace_carry: u64 = 0;

    while block_idx < block_count {
        // Load 64 bytes of source text into 2 x 256-bit vectors
        let src_vec0 = Vec256::load_unaligned(src_ptr.add(src_idx), cpu);
        let src_vec1 = Vec256::load_unaligned(src_ptr.add(src_idx + 32), cpu);

        // Look up each byte in `WORD_LUTS`
        let hi_nibble_vec0 = src_vec0.shr_u16::<4>() & 0x0f;
        let hi_nibble_vec1 = src_vec1.shr_u16::<4>() & 0x0f;
        let word_flags_vec0 =
            word_lut_lo_vec.shuffle_u8s(src_vec0) & word_lut_hi_vec.shuffle_u8s(hi_nibble_vec0);
        let word_flags_vec1 =
            word_lut_lo_vec.shuffle_u8s(src_vec1) & word_lut_hi_vec.shuffle_u8s(hi_nibble_vec1);

        // Make masks of identifier chars, whitespace and digits.
        // Non-ASCII bytes count as identifier chars in the `word` bitmap, and are folded
        // into the `misc` bitmap, so `misc_pre` can re-classify Unicode whitespace.
        let non_ascii_vec0 = src_vec0.lt_i8(0);
        let non_ascii_vec1 = src_vec1.lt_i8(0);
        let word_vec0 = (word_flags_vec0 & WORD_MASK).gt_i8(0) | non_ascii_vec0;
        let word_vec1 = (word_flags_vec1 & WORD_MASK).gt_i8(0) | non_ascii_vec1;
        let not_whitespace_vec0 = (word_flags_vec0 & WS_MASK).eq_u8(0);
        let not_whitespace_vec1 = (word_flags_vec1 & WS_MASK).eq_u8(0);
        let digit_vec0 = (word_flags_vec0 & DIGIT_MASK).gt_i8(0);
        let digit_vec1 = (word_flags_vec1 & DIGIT_MASK).gt_i8(0);

        // Convert masks to bits, and write `word` and `digit` bitmaps
        let word_bits = (word_vec0.movemask() as u64) | ((word_vec1.movemask() as u64) << 32);
        let whitespace_bits = ((!not_whitespace_vec0.movemask()) as u64)
            | (((!not_whitespace_vec1.movemask()) as u64) << 32);
        *word_bitmap_ptr.add(block_idx) = word_bits;
        *digit_bitmap_ptr.add(block_idx) =
            (digit_vec0.movemask() as u64) | ((digit_vec1.movemask() as u64) << 32);

        // Write `misc` bitmap: `#`, `\` and non-ASCII bytes
        let misc_bits0 =
            (src_vec0.eq_u8(b'#') | src_vec0.eq_u8(b'\\') | non_ascii_vec0).movemask() as u64;
        let misc_bits1 =
            (src_vec1.eq_u8(b'#') | src_vec1.eq_u8(b'\\') | non_ascii_vec1).movemask() as u64;
        *misc_bitmap_ptr.add(block_idx) = misc_bits0 | (misc_bits1 << 32);

        // Look up each byte in `MERGED_LUTS`
        let merged_flags_vec0 =
            merged_lut_lo_vec.shuffle_u8s(src_vec0) & merged_lut_hi_vec.shuffle_u8s(hi_nibble_vec0);
        let merged_flags_vec1 =
            merged_lut_lo_vec.shuffle_u8s(src_vec1) & merged_lut_hi_vec.shuffle_u8s(hi_nibble_vec1);

        // Write `dot`, `keyword_initial` and `op_char` bitmaps
        *dot_bitmap_ptr.add(block_idx) =
            (merged_flags_vec0.movemask() as u64) | ((merged_flags_vec1.movemask() as u64) << 32);
        let keyword_initial_bits0 = (merged_flags_vec0 & KWINIT_MASK).gt_i8(0).movemask() as u64;
        let keyword_initial_bits1 = (merged_flags_vec1 & KWINIT_MASK).gt_i8(0).movemask() as u64;
        *keyword_initial_bitmap_ptr.add(block_idx) =
            keyword_initial_bits0 | (keyword_initial_bits1 << 32);
        let op_char_bits0 = (merged_flags_vec0 & OPCH_MASK).gt_i8(0).movemask() as u64;
        let op_char_bits1 = (merged_flags_vec1 & OPCH_MASK).gt_i8(0).movemask() as u64;
        *op_char_bitmap_ptr.add(block_idx) = op_char_bits0 | (op_char_bits1 << 32);

        // Write `token_start` bitmap.
        // A byte starts a token unless it continues a whitespace or word run.
        // Bit `i` of `prev_word_bits` is the word bit of byte `i - 1`. For the block's first byte,
        // that's the last byte of the previous block, from `word_carry`. Likewise for whitespace.
        let prev_word_bits = (word_bits << 1) | word_carry;
        let prev_whitespace_bits = (whitespace_bits << 1) | whitespace_carry;
        word_carry = word_bits >> 63;
        whitespace_carry = whitespace_bits >> 63;
        *token_start_bitmap_ptr.add(block_idx) =
            !((whitespace_bits & prev_whitespace_bits) | (word_bits & prev_word_bits));

        // Calculate token kinds
        let kind_vec0 = punct_kinds(
            src_vec0,
            hi_nibble_vec0,
            punct_hash_lo_vec,
            punct_hash_hi_vec,
            punct_kind_lut_top_vec,
            punct_kind_lut_bot_vec,
        )
        .blend(tk!(Whitespace), !not_whitespace_vec0)
        .blend(tk!(Ident), word_vec0)
        .blend(tk!(Number), digit_vec0);

        let kind_vec1 = punct_kinds(
            src_vec1,
            hi_nibble_vec1,
            punct_hash_lo_vec,
            punct_hash_hi_vec,
            punct_kind_lut_top_vec,
            punct_kind_lut_bot_vec,
        )
        .blend(tk!(Whitespace), !not_whitespace_vec1)
        .blend(tk!(Ident), word_vec1)
        .blend(tk!(Number), digit_vec1);

        // Write token kinds
        kind_vec0.store_unaligned(kinds_ptr.add(src_idx));
        kind_vec1.store_unaligned(kinds_ptr.add(src_idx + 32));

        // Update loop counters
        src_idx += 64;
        block_idx += 1;
    }

    // Mask the overread positions `src_len..ceil` out of the last block's bitmaps.
    // Zero `PAD` bytes classify as `token_start`=1 (non-word, non-whitespace => token start),
    // which would otherwise mint spurious tokens past `src_len` in `compress`; the
    // other six bitmaps are already 0 for a zero byte, but masking all seven
    // makes the last `u64` of each bitmap bit-identical to the old scalar tail's
    // output (real bits [0, last_block_len), zeros above) regardless of LUT
    // contents. Kinds past `src_len` are never read - `compress` only visits masked
    // `token_start` starts - so they need no fixup.
    let last_block_len = src_len & 63;
    if last_block_len != 0 {
        let last_block_idx = block_count - 1;
        let valid_bits = (1u64 << last_block_len) - 1;
        *word_bitmap_ptr.add(last_block_idx) &= valid_bits;
        *token_start_bitmap_ptr.add(last_block_idx) &= valid_bits;
        *keyword_initial_bitmap_ptr.add(last_block_idx) &= valid_bits;
        *op_char_bitmap_ptr.add(last_block_idx) &= valid_bits;
        *digit_bitmap_ptr.add(last_block_idx) &= valid_bits;
        *dot_bitmap_ptr.add(last_block_idx) &= valid_bits;
        *misc_bitmap_ptr.add(last_block_idx) &= valid_bits;
    }
}

#[cpu_version(Avx2)]
#[inline(always)]
fn punct_kinds(
    src_vec: Vec256<Avx2>,
    hi_nibble_vec: Vec256<Avx2>,
    punct_hash_lo_vec: Vec256<Avx2>,
    punct_hash_hi_vec: Vec256<Avx2>,
    punct_kind_lut_top_vec: Vec256<Avx2>,
    punct_kind_lut_bot_vec: Vec256<Avx2>,
) -> Vec256<Avx2> {
    let hash_vec =
        punct_hash_lo_vec.shuffle_u8s(src_vec) ^ punct_hash_hi_vec.shuffle_u8s(hi_nibble_vec);
    let kind_top_vec = punct_kind_lut_top_vec.shuffle_u8s(hash_vec);
    let kind_bot_vec = punct_kind_lut_bot_vec.shuffle_u8s(hash_vec);
    let kind_vec = kind_top_vec.blend(kind_bot_vec, hash_vec.shl_u16::<3>());
    let control_or_non_ascii_vec = src_vec.lt_i8(0x20);
    kind_vec.blend(tk!(Invalid), control_or_non_ascii_vec)
}
