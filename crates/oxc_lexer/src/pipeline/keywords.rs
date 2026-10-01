//! Keywords, and matching words against them.
//!
//! [`KEYWORDS_JS`] and [`KEYWORDS_TS`] list the keywords, with the [`TokenKind`] of each.
//! [`kw_match`] and [`kw_match_at`] check whether a word is a keyword.
//!
//! Keywords are found with a perfect hash. JS and TS have different hashes:
//!
//! ```text
//! index = (u16(c0, c1) * DISPS_MUL) >> 26
//! JS:  slot = (DISPS_JS[index] ^ (len << 1)) & 63
//! TS:  slot = (DISPS_TS[index] ^ (len << 1) ^ c_last) & 127
//! ```
//!
//! `c0`, `c1` and `c_last` are the first, second and last bytes of the word, and `len` is its length.
//! `index` depends only on the first 2 bytes, so the CPU can load `DISPS_*[index]` in parallel with
//! calculating `len`. The JS hash does not need the last byte, which saves a load that depends on `len`.
//!
//! [`KW_TEXT`] holds each JS keyword's text in its JS slot, and each keyword's text (JS or TS)
//! in its TS slot, zero-padded to 16 bytes.
//!
//! A JS keyword whose JS slot and TS slot differ is in the table twice.
//! A word is a keyword if the word, zero-padded to 16 bytes, is equal to the entry in its slot.
//! Keyword never contain `\0`, so checking bytes also checks that the lengths are equal.
//!
//! JS mode converts the slot to a [`TokenKind`] with [`KW_KINDS_JS`], and TS mode with [`KW_KINDS_TS`].
//! JS slots can hold TS-only keywords (at their TS slots), so `KW_KINDS_JS` maps every slot which is not
//! a JS keyword's JS slot to `Ident`.
//!
//! [`DISPS_MUL`], [`DISPS_JS`] and [`DISPS_TS`] are hard-coded.
//! Tests check that each hash gives every keyword a different slot,
//! and that no 2 different keywords map to the same slot in [`KW_TEXT`].

use std::{cmp::min, hint, ptr};

use constcat::concat_slices;

use oxc_data_structures::{assert_unchecked, branch_hints::unlikely, str::const_str_eq};

use crate::token::{KW_KIND_BASE, TokenKind};

const KWINIT_LO: [u8; 16] = [0, 1, 3, 3, 3, 1, 3, 3, 0, 3, 0, 0, 1, 0, 1, 1];
const KWINIT_HI: [u8; 16] = [0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0];

/// TS-mode variant: the JS set plus `k`/`m`/`p`/`u` (keyof, module, the
/// p-words, unique/unknown/undefined/using). Same hi-nibble rows.
const KWINIT_TS_LO: [u8; 16] = [2, 1, 3, 3, 3, 3, 3, 3, 0, 3, 0, 1, 1, 1, 1, 1];

#[inline(always)]
pub(super) const fn is_kw_init(c: u8) -> bool {
    (KWINIT_LO[(c & 15) as usize] & KWINIT_HI[(c >> 4) as usize]) != 0
}

#[inline(always)]
pub(super) const fn is_kw_init_ts(c: u8) -> bool {
    (KWINIT_TS_LO[(c & 15) as usize] & KWINIT_HI[(c >> 4) as usize]) != 0
}

const KW_COUNT_JS: usize = 46;
const KW_COUNT_TS: usize = 81;

/// Keyword spellings and the token kind each rewrites to (the JS set).
/// `get`/`set` map to `Ident` (contextual, never keywords at lex time).
/// The keyword tables skip them.
///
/// Does not exist at runtime - only used at build time and in tests.
static KEYWORDS_JS: [(&str, TokenKind); KW_COUNT_JS] = [
    ("await", TokenKind::KwAwait),
    ("break", TokenKind::KwBreak),
    ("case", TokenKind::KwCase),
    ("catch", TokenKind::KwCatch),
    ("class", TokenKind::KwClass),
    ("const", TokenKind::KwConst),
    ("continue", TokenKind::KwContinue),
    ("debugger", TokenKind::KwDebugger),
    ("default", TokenKind::KwDefault),
    ("delete", TokenKind::KwDelete),
    ("do", TokenKind::KwDo),
    ("else", TokenKind::KwElse),
    ("enum", TokenKind::KwEnum),
    ("export", TokenKind::KwExport),
    ("extends", TokenKind::KwExtends),
    ("finally", TokenKind::KwFinally),
    ("for", TokenKind::KwFor),
    ("function", TokenKind::KwFunction),
    ("if", TokenKind::KwIf),
    ("import", TokenKind::KwImport),
    ("in", TokenKind::KwIn),
    ("instanceof", TokenKind::KwInstanceof),
    ("new", TokenKind::KwNew),
    ("of", TokenKind::KwOf),
    ("return", TokenKind::KwReturn),
    ("super", TokenKind::KwSuper),
    ("switch", TokenKind::KwSwitch),
    ("this", TokenKind::KwThis),
    ("throw", TokenKind::KwThrow),
    ("try", TokenKind::KwTry),
    ("typeof", TokenKind::KwTypeof),
    ("var", TokenKind::KwVar),
    ("void", TokenKind::KwVoid),
    ("while", TokenKind::KwWhile),
    ("with", TokenKind::KwWith),
    ("yield", TokenKind::KwYield),
    ("let", TokenKind::KwLet),
    ("static", TokenKind::KwStatic),
    ("async", TokenKind::KwAsync),
    ("get", TokenKind::Ident),
    ("set", TokenKind::Ident),
    ("as", TokenKind::KwAs),
    ("from", TokenKind::KwFrom),
    ("true", TokenKind::KwTrue),
    ("false", TokenKind::KwFalse),
    ("null", TokenKind::KwNull),
];

/// TS-mode additions: TypeScript's contextual keywords plus the strict-mode
/// reserved words. JS mode lexes every one of these spellings as IDENT.
///
/// Does not exist at runtime - only used at build time and in tests.
const KEYWORDS_TS_EXTRA: [(&str, TokenKind); KW_COUNT_TS - KW_COUNT_JS] = [
    ("abstract", TokenKind::KwAbstract),
    ("accessor", TokenKind::KwAccessor),
    ("any", TokenKind::KwAny),
    ("asserts", TokenKind::KwAsserts),
    ("bigint", TokenKind::KwBigInt),
    ("boolean", TokenKind::KwBoolean),
    ("declare", TokenKind::KwDeclare),
    ("global", TokenKind::KwGlobal),
    ("implements", TokenKind::KwImplements),
    ("infer", TokenKind::KwInfer),
    ("interface", TokenKind::KwInterface),
    ("intrinsic", TokenKind::KwIntrinsic),
    ("is", TokenKind::KwIs),
    ("keyof", TokenKind::KwKeyof),
    ("module", TokenKind::KwModule),
    ("namespace", TokenKind::KwNamespace),
    ("never", TokenKind::KwNever),
    ("number", TokenKind::KwNumber),
    ("object", TokenKind::KwObject),
    ("out", TokenKind::KwOut),
    ("override", TokenKind::KwOverride),
    ("package", TokenKind::KwPackage),
    ("private", TokenKind::KwPrivate),
    ("protected", TokenKind::KwProtected),
    ("public", TokenKind::KwPublic),
    ("readonly", TokenKind::KwReadonly),
    ("require", TokenKind::KwRequire),
    ("satisfies", TokenKind::KwSatisfies),
    ("string", TokenKind::KwString),
    ("symbol", TokenKind::KwSymbol),
    ("type", TokenKind::KwType),
    ("undefined", TokenKind::KwUndefined),
    ("unique", TokenKind::KwUnique),
    ("unknown", TokenKind::KwUnknown),
    ("using", TokenKind::KwUsing),
];

/// The TS-mode keyword set: [`KEYWORDS_JS`] followed by [`KEYWORDS_TS_EXTRA`].
///
/// Does not exist at runtime - only used at build time and in tests.
static KEYWORDS_TS: [(&str, TokenKind); KW_COUNT_TS] =
    *concat_slices!([(&str, TokenKind)]: &KEYWORDS_JS, &KEYWORDS_TS_EXTRA);

const REGEX_KW_MASK: u64 = {
    // `of` is deliberately absent: it precedes a regex only in a for-of
    // head (never written - a RegExp isn't iterable), while `instance/of/g`
    // style division is real code. Matches es-module-lexer/SWC/RESS.
    const RX: [&str; 18] = [
        "in",
        "do",
        "new",
        "case",
        "void",
        "else",
        "yield",
        "await",
        "throw",
        "break",
        "return",
        "typeof",
        "delete",
        "default",
        "extends",
        "continue",
        "debugger",
        "instanceof",
    ];

    // Indexed by kind offset from `KW_KIND_BASE`.
    // Every `RX` word sits in the JS kind block (offsets < 64), so the mask is set-independent.
    let mut mask = 0u64;

    let mut rx_index = 0;
    while rx_index < RX.len() {
        let r = RX[rx_index];

        let mut found: i32 = -1;
        let mut kw_index = 0;
        while kw_index < KEYWORDS_JS.len() {
            let kw = KEYWORDS_JS[kw_index];
            if const_str_eq(kw.0, r) {
                found = (kw.1 as u8 - KW_KIND_BASE) as i32;
                break;
            }
            kw_index += 1;
        }

        assert!(found >= 0 && found < 64, "regex keyword missing from KEYWORDS_JS");
        mask |= 1u64 << found;

        rx_index += 1;
    }

    mask
};

#[inline(always)]
pub fn is_regex_keyword(kind: u8) -> bool {
    let i = kind.wrapping_sub(KW_KIND_BASE);
    i < 64 && (REGEX_KW_MASK >> i) & 1 != 0
}

/// Wrapper to align a table to 32 bytes, so a small table sits in a single cache line.
#[repr(C, align(32))]
struct Aligned32<T>(T);

/// Wrapper to align a table to 64 bytes, so a small table fills a single 64-byte cache line.
#[repr(C, align(64))]
struct Aligned64<T>(T);

/// Wrapper to align a table to 128 bytes, so a table sits in a number of
/// 128-byte cache lines (Apple M), or pairs of 64-byte cache lines (x86).
#[repr(C, align(128))]
struct Aligned128<T>(T);

/// Number of slots in keyword tables.
const SLOT_COUNT: usize = 128;

/// Number of slots which JS keywords hash to.
const JS_SLOT_COUNT: usize = 64;

/// Multiplier for [`disps_index`].
const DISPS_MUL: u32 = 0x7734_D7C1;

/// Shift for [`disps_index`].
const DISPS_SHIFT: u32 = 26;

/// Displacements for the JS keyword hash, indexed by [`disps_index`].
///
/// Every entry must be less than 64, so every JS slot is less than 64.
/// [`kw_match`] relies on that for soundness.
#[rustfmt::skip]
static DISPS_JS: Aligned64<[u8; 64]> = Aligned64([
      5,  21,   0,   0,  21,  37,   0,   1,
     33,   0,   9,   0,   0,   0,   0,   0,
      0,   0,   0,   0,  19,   0,   0,  40,
      0,  16,   0,   0,   0,   0,   0,  34,
      0,  37,   0,  17,   0,   0,   5,   0,
      0,   2,   0,   0,  33,   0,  15,  16,
      0,   0,   0,  12,   0,   0,  32,   0,
     35,  32,   0,  18,  18,   0,  32,  32,
]);

const _: () = {
    let mut i = 0;
    while i < DISPS_JS.0.len() {
        assert!(DISPS_JS.0[i] < 64, "every entry of `DISPS_JS` must be less than 64");
        i += 1;
    }
};

/// Displacements for the TS keyword hash, indexed by [`disps_index`].
#[rustfmt::skip]
static DISPS_TS: Aligned64<[u8; 64]> = Aligned64([
     33,  33,  64,   0,  16,  50,   0,   1,
     33,   0,  34,   0,   0,   0,  80,  65,
     67,   0,   0,  40,   3,   0,   0,  36,
      0,   3,   4,   0,   0,   0,  32,  74,
      0,  80,  66,  33,  80,   0,   0,   0,
     64,  13,  80,  33,  32,   0,  11,  32,
      0,   0,   0,   1,   0,  64,  42,  65,
     54,  33,  64,  22,  35,   0,  37,  35,
]);

/// Text of keywords, zero-padded to 16 bytes, indexed by slot.
///
/// Each JS keyword is in its JS slot, and each keyword (JS or TS) is in its TS slot.
/// Each entry is 2 x `u64`s holding bytes 0-7 and 8-15. Empty slots are all zeros.
static KW_TEXT: Aligned128<[[u64; 2]; SLOT_COUNT]> = {
    let mut text = [[0; 2]; SLOT_COUNT];

    let mut i = 0;
    while i < KEYWORDS_JS.len() {
        let (keyword, kind) = KEYWORDS_JS[i];
        if !matches!(kind, TokenKind::Ident) {
            let keyword = keyword.as_bytes();
            text[keyword_slot::<false>(keyword)] = keyword_text(keyword);
        }
        i += 1;
    }

    let mut i = 0;
    while i < KEYWORDS_TS.len() {
        let (keyword, kind) = KEYWORDS_TS[i];
        if !matches!(kind, TokenKind::Ident) {
            let keyword = keyword.as_bytes();
            text[keyword_slot::<true>(keyword)] = keyword_text(keyword);
        }
        i += 1;
    }

    Aligned128(text)
};

/// [`TokenKind`] of each JS keyword, indexed by JS slot. Other slots are `Ident`.
static KW_KINDS_JS: Aligned64<[TokenKind; JS_SLOT_COUNT]> =
    Aligned64(kinds_table::<JS_SLOT_COUNT, false>(&KEYWORDS_JS));

/// [`TokenKind`] of each JS or TS keyword, indexed by TS slot. Other slots are `Ident`.
static KW_KINDS_TS: Aligned128<[TokenKind; SLOT_COUNT]> =
    Aligned128(kinds_table::<SLOT_COUNT, true>(&KEYWORDS_TS));

/// Build a table mapping JS slots (`IS_TS == false`) or TS slots (`IS_TS == true`) to [`TokenKind`]s
/// for the keywords in `keywords`.
///
/// Keywords whose kind is `Ident` are skipped, so they don't overwrite the kind of another keyword
/// with the same slot.
const fn kinds_table<const N: usize, const IS_TS: bool>(
    keywords: &[(&str, TokenKind)],
) -> [TokenKind; N] {
    let mut kinds = [TokenKind::Ident; N];

    let mut i = 0;
    while i < keywords.len() {
        let (keyword, kind) = keywords[i];
        if !matches!(kind, TokenKind::Ident) {
            kinds[keyword_slot::<IS_TS>(keyword.as_bytes())] = kind;
        }
        i += 1;
    }

    kinds
}

/// Get a keyword's JS slot (`IS_TS == false`) or TS slot (`IS_TS == true`), same as [`kw_match`] does.
const fn keyword_slot<const IS_TS: bool>(keyword: &[u8]) -> usize {
    let index = disps_index(keyword[0] as u16 | ((keyword[1] as u16) << 8));
    let len = keyword.len();
    if IS_TS {
        keyword_hash_ts(DISPS_TS.0[index], len, keyword[len - 1])
    } else {
        keyword_hash_js(DISPS_JS.0[index], len)
    }
}

/// Get a keyword's text, zero-padded to 16 bytes, as 2 x `u64`s.
const fn keyword_text(keyword: &[u8]) -> [u64; 2] {
    let mut text = [0; 2];

    let mut i = 0;
    while i < keyword.len() {
        text[i / 8] |= (keyword[i] as u64) << ((i % 8) * 8);
        i += 1;
    }

    text
}

/// Get index into [`DISPS_JS`] or [`DISPS_TS`] for a word, from its first 2 bytes as a little-endian `u16`.
#[inline(always)]
const fn disps_index(first_2: u16) -> usize {
    ((first_2 as u32).wrapping_mul(DISPS_MUL) >> DISPS_SHIFT) as usize
}

/// Get a word's JS slot.
///
/// `disp` is the entry in [`DISPS_JS`] for the word.
///
/// Every entry in `DISPS_JS` is less than 64, so if `len` is at most 16, the result is less than 64.
/// Therefore, result does not need masking to the table size.
#[inline(always)]
const fn keyword_hash_js(disp: u8, len: usize) -> usize {
    (disp as usize) ^ (len << 1)
}

/// Get a word's TS slot.
///
/// `disp` is the entry in [`DISPS_TS`] for the word, and `last` is the last byte of the word.
/// A non-ASCII last byte can take the hash up to 255, so it's masked to the table size.
#[inline(always)]
const fn keyword_hash_ts(disp: u8, len: usize, last: u8) -> usize {
    ((disp as usize) ^ (len << 1) ^ (last as usize)) & (SLOT_COUNT - 1)
}

/// Check if the word of `len` bytes at `pos` in `src` is a keyword.
///
/// Returns the keyword's [`TokenKind`], or `Ident` if it's not a keyword.
/// If `ts` is `false`, TS-only keywords are not treated as keywords (`Ident`).
///
/// The `len` bytes at `pos` must be word chars, otherwise the result may be wrong.
///
/// # Panics
///
/// Panics if `src` does not extend 16 bytes past `pos`.
/// The source pad guarantees that for any word in the source.
#[inline]
pub(super) fn kw_match_at(ts: bool, src: &[u8], pos: usize, len: usize) -> TokenKind {
    let word = &src[pos..pos + 16];

    // An empty word is not a keyword. Callers never pass 0, but `kw_match` requires `len`
    // of at least 1 for soundness, so we have to check.
    if unlikely(len == 0) {
        return TokenKind::Ident;
    }

    // A word of 16 bytes or more is not a keyword, and with length 16, it doesn't match one either.
    // Branchless select here rather than an early exit branch because identifiers longer than
    // 16 bytes are not uncommon.
    let len = min(len, 16);

    // SAFETY: `word` is 16 bytes, and `len` is between 1 and 16
    unsafe {
        if ts {
            kw_match::<true>(word.as_ptr(), len)
        } else {
            kw_match::<false>(word.as_ptr(), len)
        }
    }
}

/// Check if the word of `len` bytes at `ptr` is a keyword.
///
/// Returns the keyword's [`TokenKind`], or `Ident` if it's not a keyword.
/// If `IS_TS` is `false`, TS-only keywords are not treated as keywords (`Ident`).
///
/// The first `len` bytes at `ptr` must be word chars, otherwise the result may be wrong.
/// Word chars are never 0, which the comparison relies on to check the lengths are equal.
///
/// # SAFETY
///
/// * `ptr` must be valid for reading 16 bytes.
/// * `len` must be between 1 and 16 (inclusive).
#[inline(always)]
pub(super) unsafe fn kw_match<const IS_TS: bool>(ptr: *const u8, len: usize) -> TokenKind {
    // Getting `disp` does not depend on `len`, so the CPU can load it in parallel with calculating `len`
    let first_2 = ptr::read_unaligned(ptr as *const u16);
    let index = disps_index(first_2);

    let slot = if IS_TS {
        let disp = DISPS_TS.0[index];
        keyword_hash_ts(disp, len, *ptr.add(len - 1))
    } else {
        let disp = DISPS_JS.0[index];
        let slot = keyword_hash_js(disp, len);
        // SAFETY: Every entry in `DISPS_JS` is less than 64 (checked at compile time).
        // Caller guarantees `len` is at most 16, so `len << 1` is at most 32.
        // XOR of 2 values less than 64 is less than 64.
        // This lets the compiler skip bounds checks when indexing `KW_TEXT` and `KW_KINDS_JS`.
        unsafe { assert_unchecked!(slot < JS_SLOT_COUNT) };
        slot
    };

    let is_match = text_matches(ptr, len, &KW_TEXT.0[slot]);
    let kind = if IS_TS { KW_KINDS_TS.0[slot] } else { KW_KINDS_JS.0[slot] };
    // Whether a candidate is a keyword is unpredictable, so tell LLVM to select without a branch.
    // Without the hint, it branches to skip loading `kind` when there's no match.
    hint::select_unpredictable(is_match, kind, TokenKind::Ident)
}

/// Check if the word of `len` bytes at `ptr`, zero-padded to 16 bytes, is equal to `text`.
///
/// # SAFETY
///
/// `ptr` must be valid for reading 16 bytes, and `len` must be between 0 and 16 (inclusive).
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
))]
#[inline(always)]
unsafe fn text_matches(ptr: *const u8, len: usize, text: &[u64; 2]) -> bool {
    use std::arch::x86_64::{_mm_and_si128, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8};

    let word = _mm_loadu_si128(ptr.cast());
    let mask = _mm_loadu_si128(len_mask_ptr(len).cast());
    let text = _mm_loadu_si128(text.as_ptr().cast());
    let masked_word = _mm_and_si128(word, mask);
    let eq = _mm_cmpeq_epi8(masked_word, text);
    _mm_movemask_epi8(eq) == 0xFFFF
}

/// Check if the word of `len` bytes at `ptr`, zero-padded to 16 bytes, is equal to `text`.
///
/// # SAFETY
///
/// `ptr` must be valid for reading 16 bytes, and `len` must be between 0 and 16 (inclusive).
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
unsafe fn text_matches(ptr: *const u8, len: usize, text: &[u64; 2]) -> bool {
    use std::arch::aarch64::{vandq_u8, veorq_u8, vgetq_lane_u64, vld1q_u8, vreinterpretq_u64_u8};

    let word = vld1q_u8(ptr);
    let mask = vld1q_u8(len_mask_ptr(len));
    let text = vld1q_u8(text.as_ptr().cast());
    let masked_word = vandq_u8(word, mask);
    let diff = vreinterpretq_u64_u8(veorq_u8(masked_word, text));
    // NEON has no equivalent of x86's `ptest`, so combine the 2 halves, and test the result for 0
    (vgetq_lane_u64::<0>(diff) | vgetq_lane_u64::<1>(diff)) == 0
}

/// Check if the word of `len` bytes at `ptr`, zero-padded to 16 bytes, is equal to `text`.
///
/// # SAFETY
///
/// `ptr` must be valid for reading 16 bytes, and `len` must be between 0 and 16 (inclusive).
#[cfg(not(any(
    all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ),
    all(target_arch = "aarch64", target_feature = "neon")
)))]
#[inline(always)]
unsafe fn text_matches(ptr: *const u8, len: usize, text: &[u64; 2]) -> bool {
    use ptr::read_unaligned;

    let ptr = ptr as *const u64;
    let mask_ptr = len_mask_ptr(len) as *const u64;
    let masked_first_8 = read_unaligned(ptr) & read_unaligned(mask_ptr);
    let masked_second_8 = read_unaligned(ptr.add(1)) & read_unaligned(mask_ptr.add(1));
    ((masked_first_8 ^ text[0]) | (masked_second_8 ^ text[1])) == 0
}

/// Get a pointer to 16 bytes which are a mask with the first `len` bytes set to `0xFF`, and the rest 0.
///
/// # SAFETY
///
/// `len` must be between 0 and 16 (inclusive).
#[inline(always)]
unsafe fn len_mask_ptr(len: usize) -> *const u8 {
    LEN_MASK_WINDOW.0.as_ptr().add(16 - len)
}

/// 16 x `0xFF` bytes followed by 16 x 0 bytes, for [`len_mask_ptr`].
static LEN_MASK_WINDOW: Aligned32<[u8; 32]> = {
    let mut window = [0; 32];

    let mut i = 0;
    while i < 16 {
        window[i] = 0xFF;
        i += 1;
    }

    Aligned32(window)
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Bytes which can follow a word in tests. Real source has a non-word char after a word,
    /// but the match must ignore whatever follows.
    const AFTERS: [&[u8]; 4] =
        [b" ", b"\0\0\0\0", b"abcdefghijklmnop", b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF"];

    /// Keywords from `list`, excluding placeholders whose kind is `Ident`.
    fn keywords(
        list: &'static [(&'static str, TokenKind)],
    ) -> impl Iterator<Item = (&'static str, TokenKind)> {
        list.iter().copied().filter(|&(_, kind)| kind != TokenKind::Ident)
    }

    /// Run [`kw_match`] on `word`, followed by `after` (which is not part of the word).
    fn kw_match_str(word: &[u8], after: &[u8], is_ts: bool) -> TokenKind {
        let mut buf = [0u8; 48];
        buf[..word.len()].copy_from_slice(word);
        buf[word.len()..word.len() + after.len()].copy_from_slice(after);

        assert!((1usize..=16).contains(&word.len()));

        // SAFETY: `buf` is 48 bytes. `word.len()` is between 1-16 bytes.
        unsafe {
            if is_ts {
                kw_match::<true>(buf.as_ptr(), word.len())
            } else {
                kw_match::<false>(buf.as_ptr(), word.len())
            }
        }
    }

    #[test]
    fn test_is_kw_init() {
        let mut in_set = [false; 256];
        for kw in KEYWORDS_JS.iter() {
            in_set[kw.0.as_bytes()[0] as usize] = true;
        }
        for c in 0..256usize {
            assert!(is_kw_init(c as u8) == in_set[c], "KWINIT_LO/HI wrong at byte {c:#04x}");
        }
    }

    #[test]
    fn test_is_kw_init_ts() {
        let mut in_set_ts = [false; 256];
        for kw in KEYWORDS_TS.iter() {
            in_set_ts[kw.0.as_bytes()[0] as usize] = true;
        }
        for c in 0..256usize {
            assert!(is_kw_init_ts(c as u8) == in_set_ts[c], "KWINIT_TS_LO wrong at byte {c:#04x}");
        }
    }

    #[test]
    fn test_keyword_lengths() {
        // `coalesce`'s candidate filter drops most words longer than 10 bytes,
        // and the word length `kw_verify_batch` passes to `kw_match` is capped at 16
        for (keyword, _) in keywords(&KEYWORDS_TS) {
            assert!((2..=10).contains(&keyword.len()), "keyword `{keyword}` length out of range");
        }
    }

    #[test]
    fn test_js_hash_is_perfect() {
        let mut used: [Option<&str>; JS_SLOT_COUNT] = [None; JS_SLOT_COUNT];
        for (keyword, _) in keywords(&KEYWORDS_JS) {
            let slot = keyword_slot::<false>(keyword.as_bytes());
            if let Some(other) = used[slot] {
                panic!(
                    "JS hash is not perfect: `{keyword}` and `{other}` both hash to slot {slot}"
                );
            }
            used[slot] = Some(keyword);
        }
    }

    #[test]
    fn test_ts_hash_is_perfect() {
        let mut used: [Option<&str>; SLOT_COUNT] = [None; SLOT_COUNT];
        for (keyword, _) in keywords(&KEYWORDS_TS) {
            let slot = keyword_slot::<true>(keyword.as_bytes());
            if let Some(other) = used[slot] {
                panic!(
                    "TS hash is not perfect: `{keyword}` and `{other}` both hash to slot {slot}"
                );
            }
            used[slot] = Some(keyword);
        }
    }

    /// A JS keyword's JS slot must not be the TS slot of a different keyword, as they share `KW_TEXT`
    #[test]
    fn test_kw_text_no_conflicts() {
        for (js_keyword, _) in keywords(&KEYWORDS_JS) {
            let slot = keyword_slot::<false>(js_keyword.as_bytes());
            for (keyword, _) in keywords(&KEYWORDS_TS) {
                if keyword != js_keyword {
                    assert!(
                        keyword_slot::<true>(keyword.as_bytes()) != slot,
                        "JS slot of `{js_keyword}` is TS slot of `{keyword}` (slot {slot})"
                    );
                }
            }
        }
    }

    #[test]
    fn test_kw_match_keywords() {
        for (keyword, kind) in keywords(&KEYWORDS_TS) {
            let is_js_keyword = keywords(&KEYWORDS_JS).any(|(word, _)| word == keyword);
            let js_kind = if is_js_keyword { kind } else { TokenKind::Ident };
            for after in AFTERS {
                let word = keyword.as_bytes();
                assert_eq!(kw_match_str(word, after, true), kind, "TS `{keyword}`");
                assert_eq!(kw_match_str(word, after, false), js_kind, "JS `{keyword}`");
            }
        }
    }

    #[test]
    fn test_kw_match_non_keywords() {
        let words: &[&[u8]] = &[
            // spellchecker:off
            b"lets",
            b"iff",
            b"i",
            b"nul",
            b"nulll",
            b"truee",
            b"types",
            b"strin",
            b"interfac",
            b"intrinsics",
            b"undefine",
            b"satisfiess",
            // spellchecker:on
            b"get",
            b"set",
            b"Class",
            b"iF",
            b"instanceofx",
            b"interfacee",
            b"x",
            b"var\xC3\xA9",
            b"i\xFF",
        ];

        let mut words: Vec<Vec<u8>> = words.iter().map(|bytes| bytes.to_vec()).collect();

        for (keyword, _) in keywords(&KEYWORDS_TS) {
            let word = keyword.as_bytes();
            // Prefix
            words.push(word[..word.len() - 1].to_vec());
            // Extended
            words.push([word, b"s"].concat());
            // Last byte changed, including to non-ASCII
            for last in [b'_', b'0', 0x80, 0xFF] {
                let mut changed = word.to_vec();
                *changed.last_mut().unwrap() = last;
                words.push(changed);
            }
        }

        for word in &words {
            for after in AFTERS {
                assert_eq!(
                    kw_match_str(word, after, true),
                    TokenKind::Ident,
                    "TS `{}`",
                    String::from_utf8_lossy(word)
                );
                assert_eq!(
                    kw_match_str(word, after, false),
                    TokenKind::Ident,
                    "JS `{}`",
                    String::from_utf8_lossy(word)
                );
            }
        }
    }

    /// Check `kw_match` gives the same result as a linear search of the keyword lists,
    /// for all 1-3 letter lowercase words, and for every keyword with each byte replaced.
    #[test]
    fn test_kw_match_same_as_linear_search() {
        let linear_search = |word: &[u8], is_ts: bool| -> TokenKind {
            let list: &'static [(&'static str, TokenKind)] =
                if is_ts { &KEYWORDS_TS } else { &KEYWORDS_JS };
            keywords(list)
                .find(|(keyword, _)| keyword.as_bytes() == word)
                .map_or(TokenKind::Ident, |(_, kind)| kind)
        };

        let mut words: Vec<Vec<u8>> = vec![];
        for a in b'a'..=b'z' {
            words.push(vec![a]);
            for b in b'a'..=b'z' {
                words.push(vec![a, b]);
                for c in b'a'..=b'z' {
                    words.push(vec![a, b, c]);
                }
            }
        }
        let replacements =
            (b'a'..=b'z').chain([b'A', b'Z', b'0', b'9', b'_', b'$', 0x80, 0xC3, 0xFF]);
        for (keyword, _) in KEYWORDS_TS {
            let word = keyword.as_bytes();
            for i in 0..word.len() {
                for replacement in replacements.clone() {
                    let mut changed = word.to_vec();
                    changed[i] = replacement;
                    words.push(changed);
                }
            }
        }

        for word in &words {
            for after in AFTERS {
                assert_eq!(
                    kw_match_str(word, after, true),
                    linear_search(word, true),
                    "TS `{}`",
                    String::from_utf8_lossy(word)
                );
                assert_eq!(
                    kw_match_str(word, after, false),
                    linear_search(word, false),
                    "JS `{}`",
                    String::from_utf8_lossy(word)
                );
            }
        }
    }

    #[test]
    fn test_kw_match_at() {
        let mut src = [0u8; 64];
        src[..36].copy_from_slice(b"instanceofabcdefghijklmnopqrstuvwxyz");
        // Length taken from the caller, not the source
        assert_eq!(kw_match_at(false, &src, 0, 10), TokenKind::KwInstanceof);
        assert_eq!(kw_match_at(false, &src, 0, 2), TokenKind::KwIn);
        assert_eq!(kw_match_at(true, &src, 0, 9), TokenKind::Ident);
        // Longer than 16 bytes, and 0 bytes
        assert_eq!(kw_match_at(false, &src, 0, 36), TokenKind::Ident);
        assert_eq!(kw_match_at(true, &src, 0, 36), TokenKind::Ident);
        assert_eq!(kw_match_at(false, &src, 0, 0), TokenKind::Ident);
        assert_eq!(kw_match_at(true, &src, 0, 0), TokenKind::Ident);
        // Mode
        src[..6].copy_from_slice(b"string");
        assert_eq!(kw_match_at(true, &src, 0, 6), TokenKind::KwString);
        assert_eq!(kw_match_at(false, &src, 0, 6), TokenKind::Ident);
    }
}
