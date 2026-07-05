// Kernel lint policy — see the note in `pipeline/mod.rs`.
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::needless_range_loop,
    clippy::manual_range_contains,
    clippy::collapsible_if,
    clippy::collapsible_match
)]

use crate::token::token_kind;

pub const KW_COUNT_JS: usize = 46;
pub const KW_COUNT_TS: usize = 81;
pub const KW_KIND_BASE: u8 = token_kind::KW_BASE;

/// Keyword spellings and the token kind each rewrites to (the JS set).
/// `get`/`set` map to IDENT (contextual, never keywords at lex time) but
/// stay in the table so the perfect hash keeps its shape.
pub const KEYWORDS: [(&str, u8); KW_COUNT_JS] = [
    ("await", token_kind::KW_AWAIT),
    ("break", token_kind::KW_BREAK),
    ("case", token_kind::KW_CASE),
    ("catch", token_kind::KW_CATCH),
    ("class", token_kind::KW_CLASS),
    ("const", token_kind::KW_CONST),
    ("continue", token_kind::KW_CONTINUE),
    ("debugger", token_kind::KW_DEBUGGER),
    ("default", token_kind::KW_DEFAULT),
    ("delete", token_kind::KW_DELETE),
    ("do", token_kind::KW_DO),
    ("else", token_kind::KW_ELSE),
    ("enum", token_kind::KW_ENUM),
    ("export", token_kind::KW_EXPORT),
    ("extends", token_kind::KW_EXTENDS),
    ("finally", token_kind::KW_FINALLY),
    ("for", token_kind::KW_FOR),
    ("function", token_kind::KW_FUNCTION),
    ("if", token_kind::KW_IF),
    ("import", token_kind::KW_IMPORT),
    ("in", token_kind::KW_IN),
    ("instanceof", token_kind::KW_INSTANCEOF),
    ("new", token_kind::KW_NEW),
    ("of", token_kind::KW_OF),
    ("return", token_kind::KW_RETURN),
    ("super", token_kind::KW_SUPER),
    ("switch", token_kind::KW_SWITCH),
    ("this", token_kind::KW_THIS),
    ("throw", token_kind::KW_THROW),
    ("try", token_kind::KW_TRY),
    ("typeof", token_kind::KW_TYPEOF),
    ("var", token_kind::KW_VAR),
    ("void", token_kind::KW_VOID),
    ("while", token_kind::KW_WHILE),
    ("with", token_kind::KW_WITH),
    ("yield", token_kind::KW_YIELD),
    ("let", token_kind::KW_LET),
    ("static", token_kind::KW_STATIC),
    ("async", token_kind::KW_ASYNC),
    ("get", token_kind::IDENT),
    ("set", token_kind::IDENT),
    ("as", token_kind::KW_AS),
    ("from", token_kind::KW_FROM),
    ("true", token_kind::KW_TRUE),
    ("false", token_kind::KW_FALSE),
    ("null", token_kind::KW_NULL),
];

/// TS-mode additions: TypeScript's contextual keywords plus the strict-mode
/// reserved words. JS mode lexes every one of these spellings as IDENT.
pub const KEYWORDS_TS_EXTRA: [(&str, u8); KW_COUNT_TS - KW_COUNT_JS] = [
    ("abstract", token_kind::KW_ABSTRACT),
    ("accessor", token_kind::KW_ACCESSOR),
    ("any", token_kind::KW_ANY),
    ("asserts", token_kind::KW_ASSERTS),
    ("bigint", token_kind::KW_BIGINT),
    ("boolean", token_kind::KW_BOOLEAN),
    ("declare", token_kind::KW_DECLARE),
    ("global", token_kind::KW_GLOBAL),
    ("implements", token_kind::KW_IMPLEMENTS),
    ("infer", token_kind::KW_INFER),
    ("interface", token_kind::KW_INTERFACE),
    ("intrinsic", token_kind::KW_INTRINSIC),
    ("is", token_kind::KW_IS),
    ("keyof", token_kind::KW_KEYOF),
    ("module", token_kind::KW_MODULE),
    ("namespace", token_kind::KW_NAMESPACE),
    ("never", token_kind::KW_NEVER),
    ("number", token_kind::KW_NUMBER),
    ("object", token_kind::KW_OBJECT),
    ("out", token_kind::KW_OUT),
    ("override", token_kind::KW_OVERRIDE),
    ("package", token_kind::KW_PACKAGE),
    ("private", token_kind::KW_PRIVATE),
    ("protected", token_kind::KW_PROTECTED),
    ("public", token_kind::KW_PUBLIC),
    ("readonly", token_kind::KW_READONLY),
    ("require", token_kind::KW_REQUIRE),
    ("satisfies", token_kind::KW_SATISFIES),
    ("string", token_kind::KW_STRING),
    ("symbol", token_kind::KW_SYMBOL),
    ("type", token_kind::KW_TYPE),
    ("undefined", token_kind::KW_UNDEFINED),
    ("unique", token_kind::KW_UNIQUE),
    ("unknown", token_kind::KW_UNKNOWN),
    ("using", token_kind::KW_USING),
];

const fn keywords_ts() -> [(&'static str, u8); KW_COUNT_TS] {
    let mut out = [("", 0u8); KW_COUNT_TS];
    let mut i = 0;
    while i < KW_COUNT_JS {
        out[i] = KEYWORDS[i];
        i += 1;
    }
    while i < KW_COUNT_TS {
        out[i] = KEYWORDS_TS_EXTRA[i - KW_COUNT_JS];
        i += 1;
    }
    out
}

/// The TS-mode keyword set: [`KEYWORDS`] followed by [`KEYWORDS_TS_EXTRA`].
pub static KEYWORDS_TS: [(&str, u8); KW_COUNT_TS] = keywords_ts();

/// First punctuator kind — the token-kind space reserves [32, 128) for them.
pub const OP_KIND_BASE: u8 = token_kind::LBRACE;
pub const OPMAP_NOPS: usize = 33;

pub struct OpDef {
    pub txt: &'static [u8],
    pub len: u8,
    pub kind: u8,
}

macro_rules! op {
    ($t:literal, $k:expr) => {
        OpDef { txt: $t, len: $t.len() as u8, kind: $k }
    };
}

pub static OPMAP_OPS: [OpDef; OPMAP_NOPS] = [
    op!(b"<=", token_kind::LE),
    op!(b">=", token_kind::GE),
    op!(b"==", token_kind::EQ_EQ),
    op!(b"!=", token_kind::BANG_EQ),
    op!(b"===", token_kind::EQ_EQ_EQ),
    op!(b"!==", token_kind::BANG_EQ_EQ),
    op!(b"**", token_kind::STAR_STAR),
    op!(b"++", token_kind::PLUS_PLUS),
    op!(b"--", token_kind::MINUS_MINUS),
    op!(b"<<", token_kind::LSHIFT),
    op!(b">>", token_kind::RSHIFT),
    op!(b">>>", token_kind::URSHIFT),
    op!(b"&&", token_kind::AMP_AMP),
    op!(b"||", token_kind::PIPE_PIPE),
    op!(b"??", token_kind::NULLISH),
    op!(b"?.", token_kind::OPTIONAL_CHAIN),
    op!(b"=>", token_kind::ARROW),
    op!(b"+=", token_kind::PLUS_EQ),
    op!(b"-=", token_kind::MINUS_EQ),
    op!(b"*=", token_kind::STAR_EQ),
    op!(b"%=", token_kind::PERCENT_EQ),
    op!(b"<<=", token_kind::LSHIFT_EQ),
    op!(b">>=", token_kind::RSHIFT_EQ),
    op!(b">>>=", token_kind::URSHIFT_EQ),
    op!(b"&=", token_kind::AMP_EQ),
    op!(b"|=", token_kind::PIPE_EQ),
    op!(b"^=", token_kind::CARET_EQ),
    op!(b"&&=", token_kind::AMP_AMP_EQ),
    op!(b"||=", token_kind::PIPE_PIPE_EQ),
    op!(b"??=", token_kind::NULLISH_EQ),
    op!(b"**=", token_kind::STAR_STAR_EQ),
    op!(b"...", token_kind::ELLIPSIS),
    op!(b"/=", token_kind::SLASH_EQ),
];

pub const OP_QDOT: u8 = token_kind::OPTIONAL_CHAIN;
pub const OP_SLASH_EQ: u8 = token_kind::SLASH_EQ;

pub const PUNCT1_KIND_UNKNOWN: u8 = 255;
pub const PUNCT1_NKNOWN: usize = 26;

/// Single-char punctuators and their kinds. `#` maps to UNKNOWN: a bare `#`
/// is invalid on its own (private names and hashbangs are resolved earlier).
pub const PUNCT1: [(u8, u8); PUNCT1_NKNOWN] = [
    (b'(', token_kind::LPAREN),
    (b')', token_kind::RPAREN),
    (b'[', token_kind::LBRACKET),
    (b']', token_kind::RBRACKET),
    (b'{', token_kind::LBRACE),
    (b'}', token_kind::RBRACE),
    (b';', token_kind::SEMI),
    (b',', token_kind::COMMA),
    (b'.', token_kind::DOT),
    (b'<', token_kind::LT),
    (b'>', token_kind::GT),
    (b'+', token_kind::PLUS),
    (b'-', token_kind::MINUS),
    (b'*', token_kind::STAR),
    (b'/', token_kind::SLASH),
    (b'%', token_kind::PERCENT),
    (b'&', token_kind::AMP),
    (b'|', token_kind::PIPE),
    (b'^', token_kind::CARET),
    (b'!', token_kind::BANG),
    (b'~', token_kind::TILDE),
    (b'?', token_kind::QUESTION),
    (b':', token_kind::COLON),
    (b'=', token_kind::EQ),
    (b'@', token_kind::AT),
    (b'#', PUNCT1_KIND_UNKNOWN),
];

const fn punct1_list() -> [u8; PUNCT1_NKNOWN] {
    let mut out = [0u8; PUNCT1_NKNOWN];
    let mut i = 0;
    while i < PUNCT1_NKNOWN {
        out[i] = PUNCT1[i].0;
        i += 1;
    }
    out
}

const fn punct1_tok() -> [u8; PUNCT1_NKNOWN] {
    let mut out = [0u8; PUNCT1_NKNOWN];
    let mut i = 0;
    while i < PUNCT1_NKNOWN {
        out[i] = PUNCT1[i].1;
        i += 1;
    }
    out
}

/// Column views of [`PUNCT1`].
pub static PUNCT1_LIST: [u8; PUNCT1_NKNOWN] = punct1_list();
pub static PUNCT1_TOK: [u8; PUNCT1_NKNOWN] = punct1_tok();

pub struct OpMap {
    pub opmap_mul: u32,
    pub opmap_slot: [u8; 256],

    pub punct1_ord: [u8; 256],
}

pub const KW_MAX: usize = KW_COUNT_TS;
/// Slot count of the keyword hash tables — must cover the smallest shift a
/// set may search (JS shift 25 → 128 slots, TS shift 23 → 512).
pub const KW_SLOTS: usize = 512;

/// Verified first-try hints for the deterministic perfect-hash searches
/// below (checked for injectivity before use, so a word-list edit can never
/// ship a stale constant — it just falls back to the search).
pub const KW_HASH_HINT_JS: (u32, u32) = (0x0058_DC65, 25);
pub const KW_HASH_HINT_TS: (u32, u32) = (0x000B_385B, 23);

/// One keyword-recognition table set: spellings, perfect hash, and the
/// verify patterns `kw_verify_batch` compares against. `Tables` holds two —
/// the JS set and the TS set — and `lex_raw` selects by `LexOptions::ts`.
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
pub fn kw_key(c0: u8, c1: u8, len: u32) -> u32 {
    (c0 as u32) | ((c1 as u32) << 8) | (len << 16)
}

#[inline(always)]
pub fn kw_key_ts(c0: u8, c1: u8, clast: u8, len: u32) -> u32 {
    (c0 as u32) | ((c1 as u32) << 8) | ((clast as u32) << 16) | (len << 24)
}

#[inline(always)]
pub fn op_key(c0: u8, c1: u8, c2: u8, len: u32) -> u32 {
    (c0 as u32) | ((c1 as u32) << 8) | ((c2 as u32) << 16) | (len << 24)
}

impl KwSet {
    pub fn build(
        list: &[(&'static str, u8)],
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
            s.kw_tok[i] = list[i].1;
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
        let w = core::ptr::read_unaligned(p as *const u64);
        if (w & self.mask_tab[if len < 8 { len } else { 8 }]) != self.kw_first8[idx] {
            return 0;
        }
        if len > 8 {
            let e = core::ptr::read_unaligned(p.add(8) as *const u16);
            let emask: u16 = if len == 9 { 0x00FF } else { 0xFFFF };
            if (e & emask) != self.kw_ext[idx] {
                return 0;
            }
        }
        self.kw_tok[idx] as u32
    }

    pub fn self_check(&self, list: &[(&'static str, u8)]) {
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

impl OpMap {
    pub fn new() -> OpMap {
        let mut m =
            OpMap { opmap_mul: 0, opmap_slot: [0xFF; 256], punct1_ord: [PUNCT1_KIND_UNKNOWN; 256] };
        m.opmap_init();
        m.punct1_init();
        m.self_check();
        m
    }

    fn opmap_init(&mut self) {
        for i in 0..OPMAP_NOPS {
            let a = &OPMAP_OPS[i];
            assert!(
                a.len as usize == a.txt.len() && a.kind >= OP_KIND_BASE && a.kind <= 89,
                "opmap.rs: bad OpDef {i}"
            );
            for j in (i + 1)..OPMAP_NOPS {
                let b = &OPMAP_OPS[j];
                let a2 = if a.len >= 3 { a.txt[2] } else { 0 };
                let b2 = if b.len >= 3 { b.txt[2] } else { 0 };
                assert!(
                    !(a.len == b.len && a.txt[0] == b.txt[0] && a.txt[1] == b.txt[1] && a2 == b2),
                    "opmap.rs: (c0,c1,c2,len) collision"
                );
            }
        }
        let mut m: u64 = (1u64 << 24) | 1;
        while m < (1u64 << 28) {
            let mut used = [0u8; 256];
            let mut ok = true;
            for i in 0..OPMAP_NOPS {
                let o = &OPMAP_OPS[i];
                let c2 = if o.len >= 3 { o.txt[2] } else { 0 };
                let key = op_key(o.txt[0], o.txt[1], c2, o.len as u32);
                let slot = (key.wrapping_mul(m as u32) >> 24) as usize;
                if used[slot] != 0 {
                    ok = false;
                    break;
                }
                used[slot] = 1;
            }
            if ok {
                self.opmap_mul = m as u32;
                self.opmap_slot = [0xFF; 256];
                for i in 0..OPMAP_NOPS {
                    let o = &OPMAP_OPS[i];
                    let c2 = if o.len >= 3 { o.txt[2] } else { 0 };
                    let key = op_key(o.txt[0], o.txt[1], c2, o.len as u32);
                    let slot = (key.wrapping_mul(self.opmap_mul) >> 24) as usize;
                    self.opmap_slot[slot] = i as u8;
                }
                return;
            }
            m += 2;
        }
        panic!("opmap.rs: opmap perfect-hash search FAILED");
    }

    fn punct1_init(&mut self) {
        self.punct1_ord = [PUNCT1_KIND_UNKNOWN; 256];
        for i in 0..PUNCT1_NKNOWN {
            self.punct1_ord[PUNCT1_LIST[i] as usize] = PUNCT1_TOK[i];
        }
    }

    #[inline(always)]
    pub fn opmap_lookup(&self, b0: u8, b1: u8, b2: u8, b3: u8, len: u32) -> u32 {
        let c2 = if len >= 3 { b2 } else { 0 };
        let key = op_key(b0, b1, c2, len);
        let idx = self.opmap_slot[(key.wrapping_mul(self.opmap_mul) >> 24) as usize];
        if idx == 0xFF {
            return 0;
        }
        let o = &OPMAP_OPS[idx as usize];
        if o.len as u32 != len {
            return 0;
        }
        if o.txt[0] != b0 || o.txt[1] != b1 {
            return 0;
        }
        if len >= 3 && o.txt[2] != b2 {
            return 0;
        }
        if len >= 4 && o.txt[3] != b3 {
            return 0;
        }
        o.kind as u32
    }

    fn self_check(&self) {
        let mut seen_kind = [0u8; 256];
        for i in 0..OPMAP_NOPS {
            let o = &OPMAP_OPS[i];
            let mut b = [0u8; 4];
            b[..o.len as usize].copy_from_slice(&o.txt[..o.len as usize]);
            assert!(
                self.opmap_lookup(b[0], b[1], b[2], b[3], o.len as u32) == o.kind as u32,
                "opmap self-check: opmap_lookup wrong"
            );
            assert!(seen_kind[o.kind as usize] == 0, "opmap self-check: duplicate ordinal");
            seen_kind[o.kind as usize] = 1;
        }
        assert!(
            self.opmap_lookup(b'.', b'.', 0, 0, 2) == 0
                && self.opmap_lookup(b'<', b'<', 0, 0, 2) == 83
                && self.opmap_lookup(b'<', b'=', 0, 0, 2) == 49
                && self.opmap_lookup(b'>', b'>', b'>', 0, 3) == 87
                && self.opmap_lookup(b'>', b'>', b'=', 0, 3) == 86
                && self.opmap_lookup(b'=', b'=', 0, 0, 2) == 53
                && self.opmap_lookup(b'=', b'/', 0, 0, 2) == 0,
            "opmap self-check: op spot-checks failed"
        );
        let mut seen = [0u8; 256];
        for i in 0..PUNCT1_NKNOWN {
            let ord = self.punct1_ord[PUNCT1_LIST[i] as usize];
            assert!(
                ord == PUNCT1_TOK[i] && seen[ord as usize] == 0,
                "opmap self-check: PUNCT1 ordinal wrong/dup"
            );
            seen[ord as usize] = 1;
        }
        for b in 0..256usize {
            let ord = self.punct1_ord[b];
            let is_known = PUNCT1_LIST.contains(&(b as u8));
            assert!(
                is_known || ord == PUNCT1_KIND_UNKNOWN,
                "opmap self-check: PUNCT1_ORD should be unknown"
            );
        }
        assert!(
            self.punct1_ord[b'(' as usize] == 34
                && self.punct1_ord[b'#' as usize] == 255
                && self.punct1_ord[b'a' as usize] == 255
                && self.punct1_ord[b'"' as usize] == 255
                && self.punct1_ord[b'`' as usize] == 255
                && self.punct1_ord[b'\\' as usize] == 255
                && self.punct1_ord[b'$' as usize] == 255
                && self.punct1_ord[b' ' as usize] == 255
                && self.punct1_ord[0] == 255,
            "opmap self-check: PUNCT1 spot-checks failed"
        );
    }
}
