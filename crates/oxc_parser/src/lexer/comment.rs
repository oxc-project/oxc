use std::ops::Range;

use oxc_ast::CommentKind;
use oxc_data_structures::branch_hints::cold_path;
use oxc_syntax::line_terminator::is_line_terminator;

use crate::{config::LexerConfig as Config, diagnostics};

use super::{Kind, Lexer};

// Irregular line breaks - '\u{2028}' (LS) and '\u{2029}' (PS)
const LS_OR_PS_FIRST: u8 = 0xE2;
const LS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xA8];
const PS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xA9];

/// Returns `true` if `bytes` starts with the 2nd and 3rd bytes of LS or PS.
#[inline]
fn starts_with_ls_or_ps_tail(bytes: &[u8]) -> bool {
    bytes.starts_with(&LS_BYTES_2_AND_3) || bytes.starts_with(&PS_BYTES_2_AND_3)
}

/// Returns `true` if `bytes`, which follow an `@`, start with `license` or `preserve`.
#[inline]
fn starts_with_legal_tag(bytes: &[u8]) -> bool {
    bytes.starts_with(b"license") || bytes.starts_with(b"preserve")
}

/// Looks for the end of a comment, and notes what the comment contains on the way,
/// so that the comment is only read once.
trait CommentScan {
    /// Find the bytes of `chunk` which [`is_end`] has to look at.
    ///
    /// [`is_end`]: CommentScan::is_end
    fn find<const LEN: usize, M: ChunkMatcher<LEN>>(
        &self,
        chunk: &[u8; LEN],
        matcher: M,
    ) -> M::Matches;

    /// Handle a byte which [`find`] found. `byte` is the byte at `index` of `bytes`.
    /// Returns `true` if the comment ends at this byte.
    ///
    /// [`find`]: CommentScan::find
    fn is_end(&mut self, bytes: &[u8], index: usize, byte: u8) -> bool;

    /// Look for the end of the comment in `bytes[range]`, checking `LEN` bytes at a time
    /// with `matcher`. `bytes` is the source from the start of the comment's content.
    ///
    /// Returns the index of the byte which ends the comment, or `None` if it is not in `range`.
    // `#[inline(always)]` so the code is compiled with the CPU features of the function it is in
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn scan<const LEN: usize>(
        &mut self,
        bytes: &[u8],
        range: Range<usize>,
        matcher: impl ChunkMatcher<LEN>,
    ) -> Option<usize> {
        let (chunks, tail) = bytes[range.clone()].as_chunks::<LEN>();
        let mut chunk_start = range.start;
        for chunk in chunks {
            for index_in_chunk in self.find(chunk, matcher) {
                let index = chunk_start + index_in_chunk;
                if self.is_end(bytes, index, chunk[index_in_chunk]) {
                    return Some(index);
                }
            }
            chunk_start += LEN;
        }

        // Less than a chunk remains. Pad it to a whole chunk with zeros, which are never searched for.
        if !tail.is_empty() {
            let mut chunk = [0; LEN];
            chunk[..tail.len()].copy_from_slice(tail);
            for index_in_chunk in self.find(&chunk, matcher) {
                let index = chunk_start + index_in_chunk;
                if self.is_end(bytes, index, chunk[index_in_chunk]) {
                    return Some(index);
                }
            }
        }

        None
    }
}

/// Looks for the line break which ends a line comment.
struct LineCommentScan {
    /// `true` if the comment contains `@license` or `@preserve`.
    has_legal_tag: bool,
}

impl CommentScan for LineCommentScan {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn find<const LEN: usize, M: ChunkMatcher<LEN>>(
        &self,
        chunk: &[u8; LEN],
        matcher: M,
    ) -> M::Matches {
        matcher.find(chunk, [b'\r', b'\n', LS_OR_PS_FIRST, b'@'])
    }

    // `#[inline(always)]` because otherwise this is not inlined, and every comment pays for a call
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn is_end(&mut self, bytes: &[u8], index: usize, byte: u8) -> bool {
        match byte {
            b'\r' | b'\n' => true,
            b'@' => {
                self.has_legal_tag =
                    self.has_legal_tag || starts_with_legal_tag(&bytes[index + 1..]);
                false
            }
            LS_OR_PS_FIRST => {
                // Could be first byte of LS/PS, or could be some other Unicode char.
                // Either way, Unicode is uncommon, so make this a cold branch.
                cold_path();
                starts_with_ls_or_ps_tail(&bytes[index + 1..])
            }
            _ => false,
        }
    }
}

/// Looks for the `/` of the `*/` which ends a block comment.
struct BlockCommentScan {
    /// `true` if the comment contains a line break.
    has_line_break: bool,
    /// `true` if the comment contains `@license` or `@preserve`.
    has_legal_tag: bool,
}

impl CommentScan for BlockCommentScan {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn find<const LEN: usize, M: ChunkMatcher<LEN>>(
        &self,
        chunk: &[u8; LEN],
        matcher: M,
    ) -> M::Matches {
        // `*` starts every line of a JSDoc comment, whereas `/` is uncommon in comments,
        // so look for `/`, then check whether `*` precedes it.
        // Once the comment is known to contain a line break, there is no need to look for more.
        if self.has_line_break {
            matcher.find(chunk, [b'/', b'@'])
        } else {
            matcher.find(chunk, [b'/', b'@', b'\r', b'\n', LS_OR_PS_FIRST])
        }
    }

    // `#[inline(always)]` because otherwise this is not inlined, and every comment pays for a call
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn is_end(&mut self, bytes: &[u8], index: usize, byte: u8) -> bool {
        match byte {
            // The `*` of the opening `/*` is before `bytes`, so `/` at index 0 is not the end
            b'/' => index > 0 && bytes[index - 1] == b'*',
            b'@' => {
                self.has_legal_tag =
                    self.has_legal_tag || starts_with_legal_tag(&bytes[index + 1..]);
                false
            }
            b'\r' | b'\n' => {
                self.has_line_break = true;
                false
            }
            LS_OR_PS_FIRST => {
                // Could be first byte of LS/PS, or could be some other Unicode char.
                // Either way, Unicode is uncommon, so make this a cold branch.
                cold_path();
                self.has_line_break =
                    self.has_line_break || starts_with_ls_or_ps_tail(&bytes[index + 1..]);
                false
            }
            _ => false,
        }
    }
}

/// Look for the end of a comment. `bytes` is the source from the start of the comment's content.
///
/// Returns the index of the byte which ends the comment, or `None` if the comment continues
/// to the end of the source.
#[inline]
fn scan_comment(scan: &mut impl CommentScan, bytes: &[u8]) -> Option<usize> {
    #[cfg(not(all(target_arch = "x86_64", not(target_feature = "avx2"))))]
    {
        scan.scan(bytes, 0..bytes.len(), BASELINE)
    }

    // This build does not know whether the CPU supports AVX2. Finding out, and calling
    // the code which uses it (which cannot be inlined into code which does not), costs more
    // than AVX2 saves on a short comment, and most comments are short.
    // So read the start of the comment without it.
    #[cfg(all(target_arch = "x86_64", not(target_feature = "avx2")))]
    {
        /// Number of bytes at the start of a comment which are read without AVX2.
        const SHORT_COMMENT_LEN: usize = 32;

        let start = if bytes.len() >= SHORT_COMMENT_LEN {
            let end = scan.scan(bytes, 0..SHORT_COMMENT_LEN, BASELINE);
            if end.is_some() {
                return end;
            }
            SHORT_COMMENT_LEN
        } else {
            0
        };

        match Avx2::detect() {
            // SAFETY: `Avx2` only exists if the CPU supports AVX2
            Some(avx2) => unsafe { scan_long_comment_avx2(scan, bytes, start, avx2) },
            None => scan_long_comment_baseline(scan, bytes, start),
        }
    }
}

/// Look for the end of a comment in `bytes[start..]`, with AVX2 instructions.
#[cfg(all(target_arch = "x86_64", not(target_feature = "avx2")))]
#[target_feature(enable = "avx2")]
fn scan_long_comment_avx2(
    scan: &mut impl CommentScan,
    bytes: &[u8],
    start: usize,
    avx2: Avx2,
) -> Option<usize> {
    scan.scan(bytes, start..bytes.len(), avx2)
}

/// Look for the end of a comment in `bytes[start..]`, on a CPU which does not support AVX2.
// Nearly all x86-64 CPUs support AVX2
#[cfg(all(target_arch = "x86_64", not(target_feature = "avx2")))]
#[cold]
#[inline(never)]
fn scan_long_comment_baseline(
    scan: &mut impl CommentScan,
    bytes: &[u8],
    start: usize,
) -> Option<usize> {
    scan.scan(bytes, start..bytes.len(), BASELINE)
}

impl<C: Config> Lexer<'_, C> {
    /// Section 12.4 Single Line Comment
    pub(super) fn skip_single_line_comment(&mut self, kind: CommentKind) -> Kind {
        let start = self.source.position();
        let bytes = self.source.remaining().as_bytes();
        let mut scan = LineCommentScan { has_legal_tag: false };
        let Some(index) = scan_comment(&mut scan, bytes) else {
            self.source.advance_to_end();
            self.trivia_builder.add_line_comment(
                self.token_start,
                self.offset(),
                kind,
                scan.has_legal_tag,
                self.source.whole(),
                &mut self.pending,
            );
            return Kind::Skip;
        };

        // SAFETY: `index` is the index of the line break which ends the comment,
        // so it is in bounds, and on a UTF-8 char boundary
        let end = unsafe { start.add(index) };
        self.trivia_builder.add_line_comment(
            self.token_start,
            self.source.offset_of(end),
            kind,
            scan.has_legal_tag,
            self.source.whole(),
            &mut self.pending,
        );
        // Consume the line break. `\r` and `\n` are 1 byte. LS and PS are 3 bytes.
        let line_break_len = if bytes[index] == LS_OR_PS_FIRST { 3 } else { 1 };
        // SAFETY: The whole of the line break is in bounds, and after it is a UTF-8 char boundary
        self.source.set_position(unsafe { end.add(line_break_len) });
        self.pending.set_is_on_new_line();
        Kind::Skip
    }

    /// Section 12.4 Multi Line Comment
    ///
    /// Position of source must be after the opening `/*`.
    pub(super) fn skip_multi_line_comment(&mut self) -> Kind {
        let start = self.source.position();
        let mut scan = BlockCommentScan { has_line_break: false, has_legal_tag: false };
        let end = scan_comment(&mut scan, self.source.remaining().as_bytes());

        // We need to identify if comment contains line breaks or not
        // (`CommentKind::SingleLineBlock` or `CommentKind::MultiLineBlock`),
        // even if `Token`'s `is_on_new_line` flag is already set.
        let kind = if scan.has_line_break {
            self.pending.set_is_on_new_line();
            CommentKind::MultiLineBlock
        } else {
            CommentKind::SingleLineBlock
        };

        let Some(index) = end else {
            cold_path();
            self.source.advance_to_end();
            self.error(diagnostics::unterminated_multi_line_comment(self.unterminated_range()));
            return Kind::Eof;
        };

        // SAFETY: `index` is the index of the `/` of the closing `*/`, so `index + 1` is in bounds,
        // and on a UTF-8 char boundary
        self.source.set_position(unsafe { start.add(index + 1) });
        self.trivia_builder.add_block_comment(
            self.token_start,
            self.offset(),
            kind,
            scan.has_legal_tag,
            self.source.whole(),
            &mut self.pending,
        );
        Kind::Skip
    }

    /// Section 12.5 Hashbang Comments.
    ///
    /// # SAFETY
    /// Next 2 bytes must be `#!`.
    pub(super) unsafe fn read_hashbang_comment(&mut self) -> Kind {
        debug_assert_eq!(self.peek_2_bytes(), Some([b'#', b'!']));

        // SAFETY: Caller guarantees next 2 bytes are `#!`
        unsafe {
            self.source.next_byte_unchecked();
            self.source.next_byte_unchecked();
        }

        while let Some(c) = self.peek_char() {
            if is_line_terminator(c) {
                break;
            }
            self.consume_char();
        }
        Kind::HashbangComment
    }
}

// ---------- Chunk matchers ---------- //

/// Finds which bytes of a chunk of `LEN` bytes of source are any of a few byte values.
///
/// All bytes of the chunk are checked at once, so what it costs does not depend on what the chunk
/// contains, or on how many byte values are searched for. Unlike a search for the first matching
/// byte, every match in the chunk is found, so code which wants to look at each of them
/// does not start a new search after each one.
///
/// Each matcher is only compiled on the platforms which use it.
///
/// | Matcher   | Chunk    | Used on                                                 |
/// |-----------|----------|---------------------------------------------------------|
/// | `Neon`    | 16 bytes | 64-bit ARM                                              |
/// | `Avx2`    | 32 bytes | x86-64, if the CPU supports AVX2, after the first bytes |
/// | `Sse2`    | 16 bytes | x86-64, otherwise                                       |
/// | `Simd128` | 16 bytes | 32-bit WebAssembly built with `simd128`                 |
/// | `Swar`    | 8 bytes  | Everything else                                         |
trait ChunkMatcher<const LEN: usize>: Copy {
    /// Indexes of the bytes of a chunk which matched, in ascending order.
    type Matches: Iterator<Item = usize>;

    /// Find the bytes of `chunk` which are any of `needles`.
    fn find<const N: usize>(self, chunk: &[u8; LEN], needles: [u8; N]) -> Self::Matches;
}

/// Indexes of the bytes of a chunk which matched, in ascending order.
///
/// `mask` has one bit set for each byte which matched.
/// The bit for byte `index` is one of the `1 << SHIFT` bits from `index << SHIFT`.
struct ByteMatches<const SHIFT: u32> {
    mask: u64,
}

impl<const SHIFT: u32> Iterator for ByteMatches<SHIFT> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        if self.mask == 0 {
            return None;
        }
        let index = (self.mask.trailing_zeros() >> SHIFT) as usize;
        // Clear lowest bit
        self.mask &= self.mask - 1;
        Some(index)
    }
}

/// The matcher which this build can use without checking what the CPU supports.
#[cfg(all(target_arch = "aarch64", target_feature = "neon", target_endian = "little"))]
const BASELINE: Neon = Neon(());
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
const BASELINE: Avx2 = Avx2(());
#[cfg(all(target_arch = "x86_64", target_feature = "sse2", not(target_feature = "avx2")))]
const BASELINE: Sse2 = Sse2(());
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
const BASELINE: Simd128 = Simd128(());
#[cfg(not(any(
    all(target_arch = "aarch64", target_feature = "neon", target_endian = "little"),
    all(target_arch = "x86_64", target_feature = "sse2"),
    all(target_arch = "wasm32", target_feature = "simd128"),
)))]
const BASELINE: Swar = Swar;

/// Checks 16 bytes at once with NEON instructions.
#[cfg(all(target_arch = "aarch64", target_feature = "neon", target_endian = "little"))]
#[derive(Clone, Copy)]
struct Neon(());

#[cfg(all(target_arch = "aarch64", target_feature = "neon", target_endian = "little"))]
impl ChunkMatcher<16> for Neon {
    type Matches = ByteMatches<2>;

    #[inline(always)]
    fn find<const N: usize>(self, chunk: &[u8; 16], needles: [u8; N]) -> Self::Matches {
        use std::arch::aarch64::{
            vceqzq_u8, vdupq_n_u8, veorq_u8, vget_lane_u64, vld1q_u8, vminq_u8,
            vreinterpret_u64_u8, vreinterpretq_u16_u8, vshrn_n_u16,
        };

        // SAFETY: `Neon` is only compiled if NEON is enabled, so the intrinsics are available.
        // `chunk` is 16 bytes, which is what `vld1q_u8` reads.
        let mask = unsafe {
            let bytes = vld1q_u8(chunk.as_ptr());
            // XOR of a byte with a needle is 0 if they are equal, so the smallest of the XORs
            // is 0 if the byte is any of the needles.
            // Comparing with each needle and ORing the results is the obvious way to do this,
            // but LLVM compiles that to 2 more instructions per chunk than this
            // (it loses track of each byte of the result being `0xFF` or 0, and recomputes that).
            let mut smallest_diff = vdupq_n_u8(0xFF);
            for needle in needles {
                smallest_diff = vminq_u8(smallest_diff, veorq_u8(bytes, vdupq_n_u8(needle)));
            }
            let matches = vceqzq_u8(smallest_diff);
            // Each byte of `matches` is `0xFF` or 0. Narrow each to 4 bits, to fit them all
            // in a `u64`, then keep 1 bit of each.
            let nibbles = vshrn_n_u16::<4>(vreinterpretq_u16_u8(matches));
            vget_lane_u64::<0>(vreinterpret_u64_u8(nibbles)) & 0x1111_1111_1111_1111
        };
        ByteMatches { mask }
    }
}

/// Checks 16 bytes at once with SSE2 instructions.
#[cfg(all(target_arch = "x86_64", target_feature = "sse2", not(target_feature = "avx2")))]
#[derive(Clone, Copy)]
struct Sse2(());

#[cfg(all(target_arch = "x86_64", target_feature = "sse2", not(target_feature = "avx2")))]
impl ChunkMatcher<16> for Sse2 {
    type Matches = ByteMatches<0>;

    #[inline(always)]
    fn find<const N: usize>(self, chunk: &[u8; 16], needles: [u8; N]) -> Self::Matches {
        use std::arch::x86_64::{
            __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128,
            _mm_set1_epi8, _mm_setzero_si128,
        };

        // SAFETY: `Sse2` is only compiled if SSE2 is enabled, so the intrinsics are available.
        // `chunk` is 16 bytes, which is what `_mm_loadu_si128` reads. It does not require
        // the pointer to be aligned.
        let mask = unsafe {
            #[expect(clippy::cast_ptr_alignment)]
            let bytes = _mm_loadu_si128(chunk.as_ptr().cast::<__m128i>());
            let mut matches = _mm_setzero_si128();
            for needle in needles {
                #[expect(clippy::cast_possible_wrap)]
                let needle = _mm_set1_epi8(needle as i8);
                matches = _mm_or_si128(matches, _mm_cmpeq_epi8(bytes, needle));
            }
            // Top bit of each byte of `matches`. The mask is 16 bits, so is not negative.
            #[expect(clippy::cast_sign_loss)]
            let mask = _mm_movemask_epi8(matches) as u32;
            mask
        };
        ByteMatches { mask: u64::from(mask) }
    }
}

/// Checks 32 bytes at once with AVX2 instructions.
///
/// An `Avx2` only exists if the CPU supports AVX2. Code which uses it must be compiled
/// with AVX2 enabled, or it is very slow, as the intrinsics are not inlined.
#[cfg(target_arch = "x86_64")]
#[derive(Clone, Copy)]
struct Avx2(());

#[cfg(target_arch = "x86_64")]
impl Avx2 {
    /// Returns an `Avx2` if the CPU supports AVX2.
    /// What the CPU supports is only looked up once. After that, this is a load and a test.
    #[cfg(not(target_feature = "avx2"))]
    #[inline]
    fn detect() -> Option<Self> {
        if std::arch::is_x86_feature_detected!("avx2") { Some(Self(())) } else { None }
    }
}

#[cfg(target_arch = "x86_64")]
impl ChunkMatcher<32> for Avx2 {
    type Matches = ByteMatches<0>;

    #[inline(always)]
    fn find<const N: usize>(self, chunk: &[u8; 32], needles: [u8; N]) -> Self::Matches {
        use std::arch::x86_64::{
            __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_or_si256,
            _mm256_set1_epi8, _mm256_setzero_si256,
        };

        // SAFETY: `Avx2` only exists if the CPU supports AVX2, so the intrinsics are available.
        // `chunk` is 32 bytes, which is what `_mm256_loadu_si256` reads. It does not require
        // the pointer to be aligned.
        let mask = unsafe {
            #[expect(clippy::cast_ptr_alignment)]
            let bytes = _mm256_loadu_si256(chunk.as_ptr().cast::<__m256i>());
            let mut matches = _mm256_setzero_si256();
            for needle in needles {
                #[expect(clippy::cast_possible_wrap)]
                let needle = _mm256_set1_epi8(needle as i8);
                matches = _mm256_or_si256(matches, _mm256_cmpeq_epi8(bytes, needle));
            }
            // Top bit of each byte of `matches`. Cast to `u32` keeps all 32 bits.
            #[expect(clippy::cast_sign_loss)]
            let mask = _mm256_movemask_epi8(matches) as u32;
            mask
        };
        ByteMatches { mask: u64::from(mask) }
    }
}

/// Checks 16 bytes at once with WebAssembly SIMD instructions.
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
#[derive(Clone, Copy)]
struct Simd128(());

#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
impl ChunkMatcher<16> for Simd128 {
    type Matches = ByteMatches<0>;

    #[inline(always)]
    fn find<const N: usize>(self, chunk: &[u8; 16], needles: [u8; N]) -> Self::Matches {
        use std::arch::wasm32::{u8x16_bitmask, u8x16_eq, u8x16_splat, v128, v128_load, v128_or};

        // SAFETY: `Simd128` is only compiled if `simd128` is enabled, so the intrinsics
        // are available. `chunk` is 16 bytes, which is what `v128_load` reads. It does not
        // require the pointer to be aligned.
        let mask = unsafe {
            #[expect(clippy::cast_ptr_alignment)]
            let bytes = v128_load(chunk.as_ptr().cast::<v128>());
            let mut matches = u8x16_splat(0);
            for needle in needles {
                matches = v128_or(matches, u8x16_eq(bytes, u8x16_splat(needle)));
            }
            // Top bit of each byte of `matches`
            u8x16_bitmask(matches)
        };
        ByteMatches { mask: u64::from(mask) }
    }
}

/// Checks 8 bytes at once with integer arithmetic.
#[cfg(not(any(
    all(target_arch = "aarch64", target_feature = "neon", target_endian = "little"),
    all(target_arch = "x86_64", target_feature = "sse2"),
    all(target_arch = "wasm32", target_feature = "simd128"),
)))]
#[derive(Clone, Copy)]
struct Swar;

#[cfg(not(any(
    all(target_arch = "aarch64", target_feature = "neon", target_endian = "little"),
    all(target_arch = "x86_64", target_feature = "sse2"),
    all(target_arch = "wasm32", target_feature = "simd128"),
)))]
impl ChunkMatcher<8> for Swar {
    type Matches = ByteMatches<3>;

    #[inline]
    fn find<const N: usize>(self, chunk: &[u8; 8], needles: [u8; N]) -> Self::Matches {
        /// The lower 7 bits of each byte of a chunk.
        const LOW_7_BITS: u64 = u64::from_ne_bytes([0x7F; 8]);
        /// The top bit of each byte of a chunk.
        const TOP_BITS: u64 = u64::from_ne_bytes([0x80; 8]);

        // Little-endian, so that lower bits are for earlier bytes
        let bytes = u64::from_le_bytes(*chunk);
        // Top bit of each byte is set if that byte of `chunk` is none of `needles`
        let mut differs = TOP_BITS;
        for needle in needles {
            let diff = bytes ^ u64::from_ne_bytes([needle; 8]);
            // Adding `0x7F` to the lower 7 bits of a byte sets its top bit if any of them is set,
            // and does not carry into the next byte
            differs &= ((diff & LOW_7_BITS) + LOW_7_BITS) | diff;
        }
        ByteMatches { mask: !differs & TOP_BITS }
    }
}
