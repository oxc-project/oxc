use std::ptr;

use constcat::concat_slices;

use crate::token::TokenKind;

const KW_COUNT_JS: usize = 46;
const KW_COUNT_TS: usize = 81;

/// Keyword spellings and the token kind each rewrites to (the JS set).
/// `get`/`set` map to IDENT (contextual, never keywords at lex time) but
/// stay in the table so the perfect hash keeps its shape.
pub(super) const KEYWORDS_JS: [(&str, TokenKind); KW_COUNT_JS] = [
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
pub(super) static KEYWORDS_TS: [(&str, TokenKind); KW_COUNT_TS] =
    *concat_slices!([(&str, TokenKind)]: &KEYWORDS_JS, &KEYWORDS_TS_EXTRA);

const KW_MAX: usize = KW_COUNT_TS;

/// Slot count of the keyword hash tables - must cover the smallest shift a
/// set may search (JS shift 25 → 128 slots, TS shift 23 → 512).
const KW_SLOTS: usize = 512;

/// Verified first-try hints for the deterministic perfect-hash searches
/// below (checked for injectivity before use, so a word-list edit can never
/// ship a stale constant - it just falls back to the search).
pub(super) const KW_HASH_HINT_JS: (u32, u32) = (0x0058_DC65, 25);
pub(super) const KW_HASH_HINT_TS: (u32, u32) = (0x000B_385B, 23);

/// One keyword-recognition table set: spellings, perfect hash, and the
/// verify patterns `kw_verify_batch` compares against. `Tables` holds two -
/// the JS set and the TS set - and `lex_raw` selects by `LexOptions::ts`.
///
/// The hash key differs per set. JS keys on `(c0, c1, len)`; the TS set
/// keys on `(c0, c1, last, len)` because the wider list has pairs the
/// narrow key cannot separate (static/string, declare/default,
/// interface/intrinsic).
pub struct KwSet {
    pub ts_key: bool,
    pub kw_len: [u8; KW_MAX],
    pub kw_first8: [u64; KW_MAX],
    pub kw_ext: [u16; KW_MAX],
    pub kw_tok: [u8; KW_MAX],
    pub mask_tab: [u64; 9],
    pub kw_hash_mul: u32,
    pub kw_hash_shift: u32,
    pub kw_slot: [u8; KW_SLOTS],
    pub kwh_pat: [u64; KW_SLOTS],
    pub kwh_kind: [u8; KW_SLOTS],
}

#[inline(always)]
fn kw_key(c0: u8, c1: u8, len: u32) -> u32 {
    (c0 as u32) | ((c1 as u32) << 8) | (len << 16)
}

#[inline(always)]
fn kw_key_ts(c0: u8, c1: u8, clast: u8, len: u32) -> u32 {
    (c0 as u32) | ((c1 as u32) << 8) | ((clast as u32) << 16) | (len << 24)
}

impl KwSet {
    pub(super) fn build(
        list: &[(&'static str, TokenKind)],
        ts_key: bool,
        shifts: &[u32],
        hint: (u32, u32),
    ) -> KwSet {
        let n = list.len();
        assert!(n >= 2 && n <= KW_MAX && n < 0xFF, "opmap.rs: bad KwSet word count");
        let mut s = KwSet {
            ts_key,
            kw_len: [0; KW_MAX],
            kw_first8: [0; KW_MAX],
            kw_ext: [0; KW_MAX],
            kw_tok: [0; KW_MAX],
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
            assert!(len >= 2 && len <= 10, "opmap.rs: keyword length out of range");
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
                panic!("opmap.rs: kw perfect-hash search FAILED");
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

    pub(super) fn self_check(&self, list: &[(&'static str, TokenKind)]) {
        for i in 0..list.len() {
            let mut buf = [0u8; 16];
            let bytes = list[i].0.as_bytes();
            buf[..bytes.len()].copy_from_slice(bytes);
            unsafe {
                assert!(
                    self.lookup(buf.as_ptr(), bytes.len()) == list[i].1 as u32,
                    "opmap self-check: kw lookup({}) wrong",
                    list[i].0
                );
                assert!(
                    self.lookup(buf.as_ptr(), bytes.len() + 1) == 0,
                    "opmap self-check: kw lookup({}+1) matched",
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
                assert!(
                    self.lookup(buf.as_ptr(), neg.len()) == 0,
                    "opmap self-check: kw negative {neg} matched"
                );
            }
        }
    }
}
