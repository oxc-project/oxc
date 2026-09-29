use std::ptr;

use constcat::concat_slices;

use oxc_data_structures::str::const_str_eq;

use crate::token::{KW_KIND_BASE, TokenKind};

const KWINIT_LO: [u8; 16] = [0, 1, 3, 3, 3, 1, 3, 3, 0, 3, 0, 0, 1, 0, 1, 1];
const KWINIT_HI: [u8; 16] = [0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0];

/// TS-mode variant: the JS set plus `k`/`m`/`p`/`u` (keyof, module, the
/// p-words, unique/unknown/undefined/using). Same hi-nibble rows.
const KWINIT_TS_LO: [u8; 16] = [2, 1, 3, 3, 3, 3, 3, 3, 0, 3, 0, 1, 1, 1, 1, 1];

#[inline(always)]
pub const fn is_kw_init(c: u8) -> bool {
    (KWINIT_LO[(c & 15) as usize] & KWINIT_HI[(c >> 4) as usize]) != 0
}

#[inline(always)]
pub const fn is_kw_init_ts(c: u8) -> bool {
    (KWINIT_TS_LO[(c & 15) as usize] & KWINIT_HI[(c >> 4) as usize]) != 0
}

const KW_COUNT_JS: usize = 46;
const KW_COUNT_TS: usize = 81;

/// Keyword spellings and the token kind each rewrites to (the JS set).
/// `get`/`set` map to IDENT (contextual, never keywords at lex time) but
/// stay in the table so the perfect hash keeps its shape.
const KEYWORDS_JS: [(&str, TokenKind); KW_COUNT_JS] = [
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

pub struct Keywords {
    pub kwjs: KwSet,
    pub kwts: KwSet,
    pub regex_kw_mask: u64,
}

impl Keywords {
    pub(super) fn new() -> Self {
        let kwjs = KwSet::build(&KEYWORDS_JS, false, &[25, 24], KW_HASH_HINT_JS);
        let kwts = KwSet::build(&KEYWORDS_TS, true, &[23], KW_HASH_HINT_TS);

        Self { kwjs, kwts, regex_kw_mask: REGEX_KW_MASK }
    }

    #[inline(always)]
    pub fn is_regex_keyword(&self, kind: u8) -> bool {
        let i = kind.wrapping_sub(KW_KIND_BASE);
        i < 64 && (self.regex_kw_mask >> i) & 1 != 0
    }
}

/// Slot count of the keyword hash tables - must cover the smallest shift a
/// set may search (JS shift 25 → 128 slots, TS shift 23 → 512).
const KW_SLOTS: usize = 512;

/// Verified first-try hints for the deterministic perfect-hash searches
/// below (checked for injectivity before use, so a word-list edit can never
/// ship a stale constant - it just falls back to the search).
const KW_HASH_HINT_JS: (u32, u32) = (0x0058_DC65, 25);
const KW_HASH_HINT_TS: (u32, u32) = (0x000B_385B, 23);

/// One keyword-recognition table set: spellings, perfect hash, and the
/// verify patterns `kw_verify_batch` compares against. `Keywords` holds two -
/// the JS set and the TS set - and `lex_raw` selects by `LexOptions::ts`.
///
/// The hash key differs per set. JS keys on `(c0, c1, len)`; the TS set
/// keys on `(c0, c1, last, len)` because the wider list has pairs the
/// narrow key cannot separate (static/string, declare/default,
/// interface/intrinsic).
pub struct KwSet {
    pub ts_key: bool,
    pub kw_len: [u8; KW_COUNT_TS],
    pub kw_first8: [u64; KW_COUNT_TS],
    pub kw_ext: [u16; KW_COUNT_TS],
    pub kw_tok: [u8; KW_COUNT_TS],
    pub mask_tab: [u64; 9],
    pub kw_hash_mul: u32,
    pub kw_hash_shift: u32,
    pub kw_slot: [u8; KW_SLOTS],
    pub kwh_pat: [u64; KW_SLOTS],
    pub kwh_kind: [u8; KW_SLOTS],
}

impl KwSet {
    fn build(
        list: &[(&'static str, TokenKind)],
        ts_key: bool,
        shifts: &[u32],
        hint: (u32, u32),
    ) -> KwSet {
        let n = list.len();
        assert!(n >= 2 && n <= KW_COUNT_TS && n < 0xFF, "bad KwSet word count");
        let mut s = KwSet {
            ts_key,
            kw_len: [0; KW_COUNT_TS],
            kw_first8: [0; KW_COUNT_TS],
            kw_ext: [0; KW_COUNT_TS],
            kw_tok: [0; KW_COUNT_TS],
            mask_tab: [0; 9],
            kw_hash_mul: 0,
            kw_hash_shift: 0,
            kw_slot: [0xFF; KW_SLOTS],
            kwh_pat: [!0u64; KW_SLOTS],
            kwh_kind: [0; KW_SLOTS],
        };
        for i in 0..n {
            let bytes = list[i].0.as_bytes();
            let len = bytes.len();
            assert!(len >= 2 && len <= 10, "keyword length out of range");
            s.kw_len[i] = len as u8;
            s.kw_tok[i] = list[i].1 as u8;
            let mut w: u64 = 0;
            let m = if len < 8 { len } else { 8 };
            for k in 0..m {
                w |= (bytes[k] as u64) << (8 * k);
            }
            s.kw_first8[i] = w;
            let mut e: u16 = 0;
            if len > 8 {
                for k in 8..len {
                    e |= (bytes[k] as u16) << (8 * (k - 8));
                }
            }
            s.kw_ext[i] = e;
        }
        for l in 0..=8usize {
            s.mask_tab[l] = if l >= 8 { !0u64 } else { (1u64 << (8 * l)) - 1 };
        }
        let keyof = |i: usize| -> u32 {
            let b = list[i].0.as_bytes();
            if ts_key {
                kw_key_ts(b[0], b[1], b[b.len() - 1], b.len() as u32)
            } else {
                kw_key(b[0], b[1], b.len() as u32)
            }
        };
        let injective = |mul: u32, shift: u32| -> bool {
            if (1usize << (32 - shift)) > KW_SLOTS {
                return false;
            }
            let mut used = [false; KW_SLOTS];
            for i in 0..n {
                let slot = (keyof(i).wrapping_mul(mul) >> shift) as usize;
                if used[slot] {
                    return false;
                }
                used[slot] = true;
            }
            true
        };
        if hint.0 != 0 && injective(hint.0, hint.1) {
            s.kw_hash_mul = hint.0;
            s.kw_hash_shift = hint.1;
        } else {
            'search: {
                for &shift in shifts {
                    let mut m: u64 = 1;
                    while m < (1u64 << 23) {
                        if injective(m as u32, shift) {
                            s.kw_hash_mul = m as u32;
                            s.kw_hash_shift = shift;
                            break 'search;
                        }
                        m += 2;
                    }
                }
                panic!("kw perfect-hash search FAILED");
            }
        }
        for i in 0..n {
            let slot = (keyof(i).wrapping_mul(s.kw_hash_mul) >> s.kw_hash_shift) as usize;
            s.kw_slot[slot] = i as u8;
            if s.kw_len[i] <= 8 {
                s.kwh_pat[slot] = s.kw_first8[i];
                s.kwh_kind[slot] = s.kw_tok[i];
            }
        }
        s
    }

    /// [`lookup`](Self::lookup) of the word at `pos` of `len` bytes, read through a slice.
    ///
    /// `lookup` reads at most ten bytes from the word's start, so the slice must extend ten bytes
    /// past `pos`; the source pad guarantees that for any word in the source.
    #[inline(always)]
    pub fn lookup_at(&self, src: &[u8], pos: usize, len: usize) -> u32 {
        let word = &src[pos..pos + 10];
        // SAFETY: `lookup` reads bytes 0..len-1 (len <= 10), 0..8 and, for len > 8, 8..10 of the
        // word, all inside the ten-byte slice.
        unsafe { self.lookup(word.as_ptr(), len) }
    }

    /// Exact keyword match of the `len` bytes at `p`: the token kind on a
    /// hit (IDENT for the get/set placeholders), 0 otherwise.
    #[inline(always)]
    pub unsafe fn lookup(&self, p: *const u8, len: usize) -> u32 {
        if len < 2 || len > 10 {
            return 0;
        }
        let key = if self.ts_key {
            kw_key_ts(*p, *p.add(1), *p.add(len - 1), len as u32)
        } else {
            kw_key(*p, *p.add(1), len as u32)
        };
        let idx = self.kw_slot[(key.wrapping_mul(self.kw_hash_mul) >> self.kw_hash_shift) as usize];
        if idx == 0xFF {
            return 0;
        }
        let idx = idx as usize;
        if self.kw_len[idx] as usize != len {
            return 0;
        }
        let w = ptr::read_unaligned(p as *const u64);
        if (w & self.mask_tab[if len < 8 { len } else { 8 }]) != self.kw_first8[idx] {
            return 0;
        }
        if len > 8 {
            let e = ptr::read_unaligned(p.add(8) as *const u16);
            let emask: u16 = if len == 9 { 0x00FF } else { 0xFFFF };
            if (e & emask) != self.kw_ext[idx] {
                return 0;
            }
        }
        self.kw_tok[idx] as u32
    }
}

#[inline(always)]
fn kw_key(c0: u8, c1: u8, len: u32) -> u32 {
    (c0 as u32) | ((c1 as u32) << 8) | (len << 16)
}

#[inline(always)]
fn kw_key_ts(c0: u8, c1: u8, clast: u8, len: u32) -> u32 {
    (c0 as u32) | ((c1 as u32) << 8) | ((clast as u32) << 16) | (len << 24)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_kw_set_js() {
        let keywords = Keywords::new();
        check_kwset(&keywords.kwjs, &KEYWORDS_JS);
    }

    #[test]
    fn test_kw_set_ts() {
        let keywords = Keywords::new();
        check_kwset(&keywords.kwts, &KEYWORDS_TS);
    }

    fn check_kwset(kwset: &KwSet, list: &[(&'static str, TokenKind)]) {
        for i in 0..list.len() {
            let mut buf = [0u8; 16];
            let bytes = list[i].0.as_bytes();
            buf[..bytes.len()].copy_from_slice(bytes);
            unsafe {
                assert!(
                    kwset.lookup(buf.as_ptr(), bytes.len()) == list[i].1 as u32,
                    "kw lookup({}) wrong",
                    list[i].0
                );
                assert!(
                    kwset.lookup(buf.as_ptr(), bytes.len() + 1) == 0,
                    "kw lookup({}+1) matched",
                    list[i].0
                );
            }
        }

        for neg in [
            // spellchecker:off
            "lets",
            "iff",
            "i",
            "instanceofx",
            "Class",
            "nul",
            "nulll",
            "truee",
            "types",
            "strin",
            "interfac",
            "interfacee",
            "intrinsics",
            "undefine",
            "satisfiess",
            // spellchecker:on
        ] {
            let mut buf = [0u8; 16];
            buf[..neg.len()].copy_from_slice(neg.as_bytes());
            unsafe {
                assert!(kwset.lookup(buf.as_ptr(), neg.len()) == 0, "kw negative {neg} matched");
            }
        }
    }

    /// The 2 sets agree on the keywords they share.
    /// TS set matches TS-only keywords, and JS set doesn't.
    #[test]
    fn test_kw_sets_cross_check() {
        let keywords = Keywords::new();

        let mut buf = [0u8; 16];
        for (w, tok) in KEYWORDS_TS {
            let bytes = w.as_bytes();
            buf.fill(0);
            buf[..bytes.len()].copy_from_slice(bytes);

            let ts = unsafe { keywords.kwts.lookup(buf.as_ptr(), bytes.len()) };
            assert!(ts == tok as u32, "kwts lookup({w}) wrong");

            let js = unsafe { keywords.kwjs.lookup(buf.as_ptr(), bytes.len()) };
            let in_js = KEYWORDS_JS.iter().any(|k| k.0 == w);
            assert!(js == if in_js { tok as u32 } else { 0 }, "kwjs lookup({w}) wrong");
        }
    }
}
