use std::hint;

use constcat::concat_slices;

use oxc_data_structures::str::const_str_eq;
use oxc_simd::{Avx2, BASELINE, Cpu, DynCpu, Neon, Vec128, cpu_multiversion, cpu_version};

use crate::token::{KW_KIND_BASE, TokenKind};

const KWINIT_LO: [u8; 16] = [0, 1, 3, 3, 3, 1, 3, 3, 0, 3, 0, 0, 1, 0, 1, 1];
const KWINIT_HI: [u8; 16] = [0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0];

/// TS-mode variant: the JS set plus `k`/`m`/`p`/`u` (keyof, module, the
/// p-words, unique/unknown/undefined/using). Same hi-nibble rows.
const KWINIT_TS_LO: [u8; 16] = [2, 1, 3, 3, 3, 3, 3, 3, 0, 3, 0, 1, 1, 1, 1, 1];

/// Only called at compile time and in tests.
pub(super) const fn is_kw_init(c: u8) -> bool {
    (KWINIT_LO[(c & 15) as usize] & KWINIT_HI[(c >> 4) as usize]) != 0
}

/// Only called at compile time and in tests.
pub(super) const fn is_kw_init_ts(c: u8) -> bool {
    (KWINIT_TS_LO[(c & 15) as usize] & KWINIT_HI[(c >> 4) as usize]) != 0
}

const KW_COUNT_JS: usize = 46;
const KW_COUNT_TS: usize = 81;

/// A keyword and its corresponding [`TokenKind`].
///
/// Only exists at build time and in tests, not referenced from any runtime code.
#[derive(Clone, Copy)]
struct KeywordDef {
    word: &'static str,
    kind: TokenKind,
}

impl KeywordDef {
    /// Create new [`KeywordDef`].
    const fn new(word: &'static str, kind: TokenKind) -> Self {
        Self { word, kind }
    }

    /// Get this keyword's text, zero-padded to 16 bytes, as 2 x `u64`s.
    const fn padded(&self) -> [u64; 2] {
        let mut text = [0; 2];

        let bytes = self.word.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            text[i / 8] |= (bytes[i] as u64) << ((i % 8) * 8);
            i += 1;
        }

        text
    }

    /// Get a keyword's JS slot (`is_ts == false`) or TS slot (`is_ts == true`), same as [`kw_match`] does.
    const fn slot(&self, is_ts: bool) -> usize {
        let cpu = BASELINE;

        let word = self.word.as_bytes();
        if is_ts {
            keyword_hash_ts(self.padded()[0], cpu)
        } else {
            keyword_hash_js(word[0] as u16 | ((word[1] as u16) << 8), word.len(), cpu)
        }
    }
}

/// Keyword spellings and the token kind each rewrites to (the JS set).
/// `get`/`set` map to `Ident` (contextual, never keywords at lex time).
/// The keyword tables skip them.
///
/// Does not exist at runtime - only used at build time and in tests.
static KEYWORDS_JS: [KeywordDef; KW_COUNT_JS] = [
    KeywordDef::new("await", TokenKind::KwAwait),
    KeywordDef::new("break", TokenKind::KwBreak),
    KeywordDef::new("case", TokenKind::KwCase),
    KeywordDef::new("catch", TokenKind::KwCatch),
    KeywordDef::new("class", TokenKind::KwClass),
    KeywordDef::new("const", TokenKind::KwConst),
    KeywordDef::new("continue", TokenKind::KwContinue),
    KeywordDef::new("debugger", TokenKind::KwDebugger),
    KeywordDef::new("default", TokenKind::KwDefault),
    KeywordDef::new("delete", TokenKind::KwDelete),
    KeywordDef::new("do", TokenKind::KwDo),
    KeywordDef::new("else", TokenKind::KwElse),
    KeywordDef::new("enum", TokenKind::KwEnum),
    KeywordDef::new("export", TokenKind::KwExport),
    KeywordDef::new("extends", TokenKind::KwExtends),
    KeywordDef::new("finally", TokenKind::KwFinally),
    KeywordDef::new("for", TokenKind::KwFor),
    KeywordDef::new("function", TokenKind::KwFunction),
    KeywordDef::new("if", TokenKind::KwIf),
    KeywordDef::new("import", TokenKind::KwImport),
    KeywordDef::new("in", TokenKind::KwIn),
    KeywordDef::new("instanceof", TokenKind::KwInstanceof),
    KeywordDef::new("new", TokenKind::KwNew),
    KeywordDef::new("of", TokenKind::KwOf),
    KeywordDef::new("return", TokenKind::KwReturn),
    KeywordDef::new("super", TokenKind::KwSuper),
    KeywordDef::new("switch", TokenKind::KwSwitch),
    KeywordDef::new("this", TokenKind::KwThis),
    KeywordDef::new("throw", TokenKind::KwThrow),
    KeywordDef::new("try", TokenKind::KwTry),
    KeywordDef::new("typeof", TokenKind::KwTypeof),
    KeywordDef::new("var", TokenKind::KwVar),
    KeywordDef::new("void", TokenKind::KwVoid),
    KeywordDef::new("while", TokenKind::KwWhile),
    KeywordDef::new("with", TokenKind::KwWith),
    KeywordDef::new("yield", TokenKind::KwYield),
    KeywordDef::new("let", TokenKind::KwLet),
    KeywordDef::new("static", TokenKind::KwStatic),
    KeywordDef::new("async", TokenKind::KwAsync),
    KeywordDef::new("get", TokenKind::Ident),
    KeywordDef::new("set", TokenKind::Ident),
    KeywordDef::new("as", TokenKind::KwAs),
    KeywordDef::new("from", TokenKind::KwFrom),
    KeywordDef::new("true", TokenKind::KwTrue),
    KeywordDef::new("false", TokenKind::KwFalse),
    KeywordDef::new("null", TokenKind::KwNull),
];

/// TS-mode additions: TypeScript's contextual keywords plus the strict-mode
/// reserved words. JS mode lexes every one of these spellings as IDENT.
///
/// Does not exist at runtime - only used at build time and in tests.
const KEYWORDS_TS_EXTRA: [KeywordDef; KW_COUNT_TS - KW_COUNT_JS] = [
    KeywordDef::new("abstract", TokenKind::KwAbstract),
    KeywordDef::new("accessor", TokenKind::KwAccessor),
    KeywordDef::new("any", TokenKind::KwAny),
    KeywordDef::new("asserts", TokenKind::KwAsserts),
    KeywordDef::new("bigint", TokenKind::KwBigInt),
    KeywordDef::new("boolean", TokenKind::KwBoolean),
    KeywordDef::new("declare", TokenKind::KwDeclare),
    KeywordDef::new("global", TokenKind::KwGlobal),
    KeywordDef::new("implements", TokenKind::KwImplements),
    KeywordDef::new("infer", TokenKind::KwInfer),
    KeywordDef::new("interface", TokenKind::KwInterface),
    KeywordDef::new("intrinsic", TokenKind::KwIntrinsic),
    KeywordDef::new("is", TokenKind::KwIs),
    KeywordDef::new("keyof", TokenKind::KwKeyof),
    KeywordDef::new("module", TokenKind::KwModule),
    KeywordDef::new("namespace", TokenKind::KwNamespace),
    KeywordDef::new("never", TokenKind::KwNever),
    KeywordDef::new("number", TokenKind::KwNumber),
    KeywordDef::new("object", TokenKind::KwObject),
    KeywordDef::new("out", TokenKind::KwOut),
    KeywordDef::new("override", TokenKind::KwOverride),
    KeywordDef::new("package", TokenKind::KwPackage),
    KeywordDef::new("private", TokenKind::KwPrivate),
    KeywordDef::new("protected", TokenKind::KwProtected),
    KeywordDef::new("public", TokenKind::KwPublic),
    KeywordDef::new("readonly", TokenKind::KwReadonly),
    KeywordDef::new("require", TokenKind::KwRequire),
    KeywordDef::new("satisfies", TokenKind::KwSatisfies),
    KeywordDef::new("string", TokenKind::KwString),
    KeywordDef::new("symbol", TokenKind::KwSymbol),
    KeywordDef::new("type", TokenKind::KwType),
    KeywordDef::new("undefined", TokenKind::KwUndefined),
    KeywordDef::new("unique", TokenKind::KwUnique),
    KeywordDef::new("unknown", TokenKind::KwUnknown),
    KeywordDef::new("using", TokenKind::KwUsing),
];

/// The TS-mode keyword set: [`KEYWORDS_JS`] followed by [`KEYWORDS_TS_EXTRA`].
///
/// Does not exist at runtime - only used at build time and in tests.
static KEYWORDS_TS: [KeywordDef; KW_COUNT_TS] =
    *concat_slices!([KeywordDef]: &KEYWORDS_JS, &KEYWORDS_TS_EXTRA);

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
            let kw = &KEYWORDS_JS[kw_index];
            if const_str_eq(kw.word, r) {
                found = (kw.kind as u8 - KW_KIND_BASE) as i32;
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

#[cpu_multiversion]
#[inline(always)]
pub fn is_regex_keyword(kind: u8, _cpu: impl Cpu) -> bool {
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

/// Number of JS slots, as the JS hash is at most 63 + 16.
const JS_SLOT_COUNT: usize = 80;

/// Number of TS slots.
const SLOT_COUNT: usize = 256;

/// Multiplier for the JS keyword hash.
const JS_MUL: u32 = 0x14D9_7F90;

/// Multiplier for the TS keyword hash.
const TS_MUL: u64 = 0x686A_C588_6DD9_FA39;

/// JS keyword text by JS slot in 8-byte cells, which a keyword over 8 bytes runs past.
static KW_TEXT_JS: Aligned64<[u64; JS_SLOT_COUNT + 1]> = {
    let mut text = [0; JS_SLOT_COUNT + 1];

    let mut i = 0;
    while i < KEYWORDS_JS.len() {
        let kw = &KEYWORDS_JS[i];
        if !matches!(kw.kind, TokenKind::Ident) {
            let slot = kw.slot(false);
            let padded = kw.padded();
            assert!(
                text[slot] == 0 && (padded[1] == 0 || text[slot + 1] == 0),
                "JS keywords overlap"
            );
            text[slot] = padded[0];
            text[slot + 1] |= padded[1];
        }
        i += 1;
    }

    Aligned64(text)
};

/// Bit N is set if the JS keyword in the slot has a byte N.
static KW_BITS_JS: Aligned64<[u32; JS_SLOT_COUNT]> = {
    let mut bits = [0; JS_SLOT_COUNT];

    let mut i = 0;
    while i < KEYWORDS_JS.len() {
        let kw = &KEYWORDS_JS[i];
        if !matches!(kw.kind, TokenKind::Ident) {
            bits[kw.slot(false)] = (1 << kw.word.len()) - 1;
        }
        i += 1;
    }

    Aligned64(bits)
};

/// Byte N is 0xFF if the JS keyword in the slot has a byte N.
static KW_MASKS_JS: Aligned64<[[u64; 2]; JS_SLOT_COUNT]> = {
    let mut masks = [[0; 2]; JS_SLOT_COUNT];

    let mut i = 0;
    while i < KEYWORDS_JS.len() {
        let kw = &KEYWORDS_JS[i];
        if !matches!(kw.kind, TokenKind::Ident) {
            let slot = kw.slot(false);
            let mut j = 0;
            while j < kw.word.len() {
                masks[slot][j / 8] |= 0xFF << ((j % 8) * 8);
                j += 1;
            }
        }
        i += 1;
    }

    Aligned64(masks)
};

/// Keyword text by TS slot, zero-padded to 16 bytes.
static KW_TEXT_TS: Aligned64<[[u64; 2]; SLOT_COUNT]> = {
    let mut text = [[0; 2]; SLOT_COUNT];

    let mut i = 0;
    while i < KEYWORDS_TS.len() {
        let kw = &KEYWORDS_TS[i];
        if !matches!(kw.kind, TokenKind::Ident) {
            let slot = kw.slot(true);
            assert!(text[slot][0] == 0, "TS hash is not perfect");
            text[slot] = kw.padded();
        }
        i += 1;
    }

    Aligned64(text)
};

/// [`TokenKind`] of each JS keyword, indexed by JS slot. Other slots are `Ident`.
static KW_KINDS_JS: Aligned64<[TokenKind; JS_SLOT_COUNT]> =
    Aligned64(kinds_table::<JS_SLOT_COUNT>(&KEYWORDS_JS, false));

/// [`TokenKind`] of each JS or TS keyword, indexed by TS slot. Other slots are `Ident`.
static KW_KINDS_TS: Aligned128<[TokenKind; SLOT_COUNT]> =
    Aligned128(kinds_table::<SLOT_COUNT>(&KEYWORDS_TS, true));

/// Build a table mapping JS slots (`is_ts == false`) or TS slots (`is_ts == true`) to [`TokenKind`]s
/// for the keywords in `keywords`.
///
/// Keywords whose kind is `Ident` are skipped, so they don't overwrite the kind of another keyword
/// with the same slot.
///
/// Only called at compile time.
const fn kinds_table<const N: usize>(keywords: &[KeywordDef], is_ts: bool) -> [TokenKind; N] {
    let mut kinds = [TokenKind::Ident; N];

    let mut i = 0;
    while i < keywords.len() {
        let kw = &keywords[i];
        if !matches!(kw.kind, TokenKind::Ident) {
            kinds[kw.slot(is_ts)] = kw.kind;
        }
        i += 1;
    }

    kinds
}

/// Get a word's JS slot from its first 2 bytes and its length.
#[cpu_multiversion]
#[inline(always)]
const fn keyword_hash_js(first_2: u16, len: usize, _cpu: impl Cpu) -> usize {
    ((first_2 as u32).wrapping_mul(JS_MUL) >> 26) as usize + len
}

/// Get a word's TS slot from its first 8 bytes, with any bytes after the word zeroed.
#[cpu_multiversion]
#[inline(always)]
const fn keyword_hash_ts(first_8: u64, _cpu: impl Cpu) -> usize {
    (first_8.wrapping_mul(TS_MUL) >> 56) as usize
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
#[cpu_multiversion]
#[inline]
pub(super) fn kw_match_at(
    is_ts: bool,
    src: &[u8],
    pos: usize,
    len: usize,
    cpu: impl Cpu,
) -> TokenKind {
    let word = src[pos..pos + 16].try_into().unwrap();
    if is_ts { kw_match::<true>(word, len, cpu) } else { kw_match::<false>(word, len, cpu) }
}

/// Check if the first len bytes of word are a keyword.
///
/// Returns the keyword's [`TokenKind`], or `Ident` if it's not a keyword.
/// If `IS_TS` is `false`, TS-only keywords are not treated as keywords (`Ident`).
///
/// The first len bytes of word must be word chars, otherwise the result may be wrong.
/// Word chars are never 0, which the comparison relies on to check the lengths are equal.
#[cpu_multiversion]
#[inline(always)]
pub(super) fn kw_match<const IS_TS: bool>(word: &[u8; 16], len: usize, cpu: impl Cpu) -> TokenKind {
    // No keyword is 16 bytes long, so the cap changes no match and keeps every slot in range.
    let len = len.min(16);
    let (slot, is_match) = if IS_TS {
        match DynCpu::from(cpu) {
            DynCpu::Avx2(cpu) => ts_match_avx2(word, len, cpu),
            DynCpu::Neon(cpu) => ts_match_neon(word, len, cpu),
            DynCpu::Generic(_) => ts_match_generic(word, len),
        }
    } else {
        let slot = keyword_hash_js(u16::from_le_bytes([word[0], word[1]]), len, cpu);
        let is_match = match DynCpu::from(cpu) {
            DynCpu::Avx2(cpu) => js_match_avx2(word, slot, cpu),
            DynCpu::Neon(cpu) => js_match_neon(word, slot, cpu),
            DynCpu::Generic(_) => js_match_generic(word, slot),
        };
        (slot, is_match)
    };

    let kind = if IS_TS { KW_KINDS_TS.0[slot] } else { KW_KINDS_JS.0[slot] };
    // Whether a candidate is a keyword is unpredictable, so tell LLVM to select without a branch.
    // Without the hint, it branches to skip loading `kind` when there's no match.
    hint::select_unpredictable(is_match, kind, TokenKind::Ident)
}

// The JS slot grows with the length, so matching the keyword's own bytes also matches the length.
#[cpu_version(Avx2)]
#[inline(always)]
fn js_match_avx2(word: &[u8; 16], slot: usize, cpu: Avx2) -> bool {
    let word = Vec128::read_unaligned(word, cpu);
    let text = Vec128::read_unaligned(KW_TEXT_JS.0[slot..].first_chunk::<2>().unwrap(), cpu);
    (!word.eq_u8(text).movemask() & KW_BITS_JS.0[slot]) == 0
}

#[cpu_version(Neon)]
#[inline(always)]
fn js_match_neon(word: &[u8; 16], slot: usize, cpu: Neon) -> bool {
    let word = Vec128::read_unaligned(word, cpu);
    let text = Vec128::read_unaligned(KW_TEXT_JS.0[slot..].first_chunk::<2>().unwrap(), cpu);
    ((word ^ text) & Vec128::read_unaligned(&KW_MASKS_JS.0[slot], cpu)).is_zero()
}

#[cpu_version(Generic)]
#[inline(always)]
fn js_match_generic(word: &[u8; 16], slot: usize) -> bool {
    let [lo, hi] = halves(word);
    let text = KW_TEXT_JS.0[slot..].first_chunk::<2>().unwrap();
    let mask = &KW_MASKS_JS.0[slot];
    let first_8 = (lo ^ text[0]) & mask[0];
    let second_8 = (hi ^ text[1]) & mask[1];
    (first_8 | second_8) == 0
}

// The masked word's first 8 bytes are the hash key, and all 16 are compared with the keyword.
#[cpu_version(Avx2)]
#[inline(always)]
fn ts_match_avx2(word: &[u8; 16], len: usize, cpu: Avx2) -> (usize, bool) {
    let word = Vec128::read_unaligned(word, cpu) & Vec128::read_unaligned(len_mask(len, cpu), cpu);
    let slot = keyword_hash_ts(word.get_u64::<0>(), cpu);
    (slot, word.all_eq_u8(Vec128::read_unaligned(&KW_TEXT_TS.0[slot], cpu)))
}

#[cpu_version(Neon)]
#[inline(always)]
fn ts_match_neon(word: &[u8; 16], len: usize, cpu: Neon) -> (usize, bool) {
    let word = Vec128::read_unaligned(word, cpu) & Vec128::read_unaligned(len_mask(len, cpu), cpu);
    let slot = keyword_hash_ts(word.get_u64::<0>(), cpu);
    (slot, word.all_eq_u8(Vec128::read_unaligned(&KW_TEXT_TS.0[slot], cpu)))
}

#[cpu_version(Generic)]
#[inline(always)]
fn ts_match_generic(word: &[u8; 16], len: usize) -> (usize, bool) {
    let cpu = BASELINE;

    let ([lo, hi], [mask_lo, mask_hi]) = (halves(word), halves(len_mask(len, cpu)));
    let masked_first_8 = lo & mask_lo;
    let masked_second_8 = hi & mask_hi;
    let slot = keyword_hash_ts(masked_first_8, cpu);
    let text = &KW_TEXT_TS.0[slot];
    (slot, ((masked_first_8 ^ text[0]) | (masked_second_8 ^ text[1])) == 0)
}

/// The 16 bytes as two little-endian u64s.
#[cpu_version(Generic)]
#[inline(always)]
fn halves(bytes: &[u8; 16]) -> [u64; 2] {
    let (lo, hi) = bytes.split_at(8);
    [u64::from_le_bytes(lo.try_into().unwrap()), u64::from_le_bytes(hi.try_into().unwrap())]
}

/// Get 16 bytes which are a mask with the first `len` bytes set to `0xFF`, and the rest 0.
///
/// # Panics
///
/// Panics if `len` is greater than 16.
#[cpu_multiversion]
#[inline(always)]
fn len_mask(len: usize, _cpu: impl Cpu) -> &'static [u8; 16] {
    LEN_MASK_WINDOW.0[16 - len..].first_chunk::<16>().unwrap()
}

/// 16 x `0xFF` bytes followed by 16 x 0 bytes, for [`len_mask`].
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
    use oxc_simd::test_cpus;

    use super::*;

    /// Bytes which can follow a word in tests. Real source has a non-word char after a word,
    /// but the match must ignore whatever follows.
    const AFTERS: [&[u8]; 4] =
        [b" ", b"\0\0\0\0", b"abcdefghijklmnop", b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF"];

    /// Keywords from `list`, excluding placeholders whose kind is `Ident`.
    fn keywords(list: &'static [KeywordDef]) -> impl Iterator<Item = &'static KeywordDef> {
        list.iter().filter(|kw| kw.kind != TokenKind::Ident)
    }

    /// Run [`kw_match`] on `word`, followed by `after` (which is not part of the word),
    /// with CPU feature set `cpu`.
    #[cpu_multiversion]
    fn kw_match_str(word: &[u8], after: &[u8], is_ts: bool, cpu: impl Cpu) -> TokenKind {
        assert!((1..=16).contains(&word.len()));

        let mut buf = [0u8; 48];
        buf[..word.len()].copy_from_slice(word);
        buf[word.len()..word.len() + after.len()].copy_from_slice(after);

        let window = buf.first_chunk::<16>().unwrap();
        if is_ts {
            kw_match::<true>(window, word.len(), cpu)
        } else {
            kw_match::<false>(window, word.len(), cpu)
        }
    }

    #[test]
    fn test_is_kw_init() {
        let mut in_set = [false; 256];
        for kw in &KEYWORDS_JS {
            in_set[kw.word.as_bytes()[0] as usize] = true;
        }

        for b in 0..=255 {
            assert!(is_kw_init(b) == in_set[b as usize], "KWINIT_LO/HI wrong at byte {b:#04x}");
        }
    }

    #[test]
    fn test_is_kw_init_ts() {
        let mut in_set_ts = [false; 256];
        for kw in &KEYWORDS_TS {
            in_set_ts[kw.word.as_bytes()[0] as usize] = true;
        }

        for b in 0..=255 {
            assert!(
                is_kw_init_ts(b) == in_set_ts[b as usize],
                "KWINIT_TS_LO wrong at byte {b:#04x}"
            );
        }
    }

    #[test]
    fn test_keyword_lengths() {
        // `coalesce`'s candidate filter drops most words longer than 10 bytes,
        // and the word length `kw_verify_batch` passes to `kw_match` is capped at 16
        for kw in keywords(&KEYWORDS_TS) {
            let word = kw.word;
            assert!((2..=10).contains(&word.len()), "keyword `{word}` length out of range");
        }
    }

    #[test]
    fn test_js_hash_is_perfect() {
        let mut used: [Option<&str>; JS_SLOT_COUNT] = [None; JS_SLOT_COUNT];
        for kw in keywords(&KEYWORDS_JS) {
            let word = kw.word;
            let slot = kw.slot(false);
            if let Some(other) = used[slot] {
                panic!("JS hash is not perfect: `{word}` and `{other}` both hash to slot {slot}");
            }
            used[slot] = Some(word);
        }
    }

    #[test]
    fn test_ts_hash_is_perfect() {
        let mut used: [Option<&str>; SLOT_COUNT] = [None; SLOT_COUNT];
        for kw in keywords(&KEYWORDS_TS) {
            let word = kw.word;
            let slot = kw.slot(true);
            if let Some(other) = used[slot] {
                panic!("TS hash is not perfect: `{word}` and `{other}` both hash to slot {slot}");
            }
            used[slot] = Some(word);
        }
    }

    #[test_cpus]
    fn test_kw_match_keywords(cpu: impl Cpu) {
        for &KeywordDef { word, kind } in keywords(&KEYWORDS_TS) {
            let is_js_keyword = keywords(&KEYWORDS_JS).any(|kw| kw.word == word);
            let js_kind = if is_js_keyword { kind } else { TokenKind::Ident };
            for after in AFTERS {
                let bytes = word.as_bytes();
                assert_eq!(kw_match_str(bytes, after, true, cpu), kind, "TS `{word}`");
                assert_eq!(kw_match_str(bytes, after, false, cpu), js_kind, "JS `{word}`");
            }
        }
    }

    #[test_cpus]
    fn test_kw_match_non_keywords(cpu: impl Cpu) {
        let words: &[&[u8]] = &[
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

        for kw in keywords(&KEYWORDS_TS) {
            let word = kw.word.as_bytes();
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
                    kw_match_str(word, after, true, cpu),
                    TokenKind::Ident,
                    "TS `{}`",
                    String::from_utf8_lossy(word)
                );
                assert_eq!(
                    kw_match_str(word, after, false, cpu),
                    TokenKind::Ident,
                    "JS `{}`",
                    String::from_utf8_lossy(word)
                );
            }
        }
    }

    /// Check `kw_match` gives the same result as a linear search of the keyword lists,
    /// for all 1-3 letter lowercase words, and for every keyword with each byte replaced.
    #[test_cpus]
    fn test_kw_match_same_as_linear_search(cpu: impl Cpu) {
        let linear_search = |word: &[u8], is_ts: bool| -> TokenKind {
            let list: &'static [KeywordDef] = if is_ts { &KEYWORDS_TS } else { &KEYWORDS_JS };
            keywords(list)
                .find(|kw| kw.word.as_bytes() == word)
                .map_or(TokenKind::Ident, |kw| kw.kind)
        };

        let mut words: Vec<Vec<u8>> = vec![];
        for b1 in b'a'..=b'z' {
            words.push(vec![b1]);
            for b2 in b'a'..=b'z' {
                words.push(vec![b1, b2]);
                for b3 in b'a'..=b'z' {
                    words.push(vec![b1, b2, b3]);
                }
            }
        }

        let replacements =
            (b'a'..=b'z').chain([b'A', b'Z', b'0', b'9', b'_', b'$', 0x80, 0xC3, 0xFF]);
        for kw in &KEYWORDS_TS {
            let word = kw.word.as_bytes();
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
                    kw_match_str(word, after, true, cpu),
                    linear_search(word, true),
                    "TS `{}`",
                    String::from_utf8_lossy(word)
                );
                assert_eq!(
                    kw_match_str(word, after, false, cpu),
                    linear_search(word, false),
                    "JS `{}`",
                    String::from_utf8_lossy(word)
                );
            }
        }
    }

    #[test_cpus]
    fn test_kw_match_at(cpu: impl Cpu) {
        let mut src = [0u8; 64];
        src[..36].copy_from_slice(b"instanceofabcdefghijklmnopqrstuvwxyz");
        // Length taken from the caller, not the source
        assert_eq!(kw_match_at(false, &src, 0, 10, cpu), TokenKind::KwInstanceof);
        assert_eq!(kw_match_at(false, &src, 0, 2, cpu), TokenKind::KwIn);
        assert_eq!(kw_match_at(true, &src, 0, 9, cpu), TokenKind::Ident);
        // Longer than 16 bytes, and 0 bytes
        assert_eq!(kw_match_at(false, &src, 0, 36, cpu), TokenKind::Ident);
        assert_eq!(kw_match_at(true, &src, 0, 36, cpu), TokenKind::Ident);
        assert_eq!(kw_match_at(false, &src, 0, 0, cpu), TokenKind::Ident);
        assert_eq!(kw_match_at(true, &src, 0, 0, cpu), TokenKind::Ident);
        // Mode
        src[..6].copy_from_slice(b"string");
        assert_eq!(kw_match_at(true, &src, 0, 6, cpu), TokenKind::KwString);
        assert_eq!(kw_match_at(false, &src, 0, 6, cpu), TokenKind::Ident);
    }
}
