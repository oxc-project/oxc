// Kernel lint policy (this module tree, `lanes`, `opmap`, `tables`): the
// `unsafe fn` boundary is the reviewed surface, and the pedantic/nursery
// style lints fight the SIMD idiom. API modules keep the full workspace bar.
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::needless_range_loop,
    clippy::manual_range_contains,
    clippy::collapsible_if,
    clippy::collapsible_match
)]

mod bitmap;
mod carve;
mod classify;
mod coalesce;
mod compress;
mod find;
mod keywords;
mod regex_div;
mod replay;

use crate::PAD;
use crate::lanes::Lanes;
use crate::options::LexOptions;
use crate::tables::Tables;

use bitmap::bm_any;
use carve::{carve, carve_jsx};
use classify::{classify, misc_post, misc_pre};
use coalesce::coalesce;
use compress::{compress, lanes_post};
use keywords::KWB;

use crate::token::token_kind;

// Short kind aliases for the pipeline, tied to `token_kind` so they can't drift.
pub(crate) const WS: u8 = token_kind::WHITESPACE;
pub(crate) const IDENT: u8 = token_kind::IDENT;
pub(crate) const NUM: u8 = token_kind::NUMBER;
pub(crate) const BIGINT: u8 = token_kind::BIGINT;
pub(crate) const STR: u8 = token_kind::STRING;
pub(crate) const LCOM: u8 = token_kind::LINE_COMMENT;
pub(crate) const BCOM: u8 = token_kind::BLOCK_COMMENT;
pub(crate) const REGEX: u8 = token_kind::REGEXP;
pub(crate) const TMPL_NOSUB: u8 = token_kind::TEMPLATE_NO_SUB;
pub(crate) const TMPL_HEAD: u8 = token_kind::TEMPLATE_HEAD;
pub(crate) const TMPL_MIDDLE: u8 = token_kind::TEMPLATE_MIDDLE;
pub(crate) const TMPL_TAIL: u8 = token_kind::TEMPLATE_TAIL;
pub(crate) const HASHBANG: u8 = token_kind::HASHBANG;
pub(crate) const IDENT_ESC: u8 = token_kind::IDENT_ESCAPED;
pub(crate) const PRIV_IDENT: u8 = token_kind::PRIVATE_IDENT;
pub(crate) const PRIV_IDENT_ESC: u8 = token_kind::PRIVATE_IDENT_ESCAPED;
pub(crate) const EOF: u8 = token_kind::EOF;

// JSX coarse kinds, written only by `carve_jsx`. `JEND`/`JSX_LT` read as
// values in `prev_is_regex` (after a completed element, `/` is division).
pub(crate) const JTEXT: u8 = token_kind::JSX_TEXT;
pub(crate) const JEND: u8 = token_kind::JSX_TAG_END;
pub(crate) const JSX_LT: u8 = token_kind::JSX_LT;

// `glue_number` computes the kind as `NUM + is_bigint` — keep them adjacent.
const _: () = assert!(BIGINT == NUM + 1);

pub struct Lexer {
    word: Vec<u64>,
    st: Vec<u64>,
    kwinit: Vec<u64>,
    opch: Vec<u64>,
    digit: Vec<u64>,
    dot: Vec<u64>,
    misc: Vec<u64>,
    kind: Vec<u8>,
    kwpos: Vec<u32>,
    nb_cap: usize,
    pub starts: Vec<u32>,
    pub kinds: Vec<u8>,
    out_cap: usize,
    pub lanes: Lanes,
    tables: Box<Tables>,
}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            word: Vec::new(),
            st: Vec::new(),
            kwinit: Vec::new(),
            opch: Vec::new(),
            digit: Vec::new(),
            dot: Vec::new(),
            misc: Vec::new(),
            kind: Vec::new(),
            kwpos: Vec::new(),
            nb_cap: 0,
            starts: Vec::new(),
            kinds: Vec::new(),
            out_cap: 0,
            lanes: Lanes::default(),
            tables: Box::new(Tables::new()),
        }
    }

    fn ensure(&mut self, n: usize) {
        let nb = n.div_ceil(64) + 1;
        if self.nb_cap < nb {
            self.word.resize(nb, 0);
            self.st.resize(nb, 0);
            self.kwinit.resize(nb, 0);
            self.opch.resize(nb, 0);
            self.digit.resize(nb, 0);
            self.dot.resize(nb, 0);
            self.misc.resize(nb, 0);
            self.kind.resize(nb * 64, 0);
            self.nb_cap = nb;
        }
        if self.kwpos.is_empty() {
            self.kwpos.resize(KWB * 64 + 8, 0);
        }
        let need = n + PAD;
        if self.out_cap < need {
            self.starts.resize(need, 0);
            self.kinds.resize(need, 0);
            self.out_cap = need;
        }
    }

    /// Run the full pipeline over `src[..n]`, writing the token stream into
    /// `out_kinds`/`out_starts` (EOF sentinel last) and the value lanes and
    /// diagnostics into `self.lanes`. Returns the token count including EOF.
    ///
    /// # Safety
    ///
    /// - `src` must extend at least [`PAD`] zeroed bytes past `n`.
    /// - `out_kinds` must be valid for `n + PAD` byte writes and `out_starts`
    ///   for `n + PAD` u32 writes: `compress` stores full 8-lane groups past
    ///   the last token, and the EOF sentinel follows it.
    pub unsafe fn lex_raw(
        &mut self,
        src: &[u8],
        n: usize,
        out_kinds: *mut u8,
        out_starts: *mut u32,
        jsx: bool,
        ts: bool,
        module: bool,
        vutf8: bool,
    ) -> usize {
        debug_assert!(src.len() >= n + PAD, "source must extend PAD zeroed bytes past n");
        self.ensure(n);
        self.lanes.clear();
        self.lanes.module = module;
        if n == 0 {
            *out_kinds = EOF;
            *out_starts = 0;
            *out_starts.add(1) = 0;
            return 1;
        }
        let nb = n.div_ceil(64);
        let sp = src.as_ptr();
        let word = self.word.as_mut_ptr();
        let st = self.st.as_mut_ptr();
        let kwinit = self.kwinit.as_mut_ptr();
        let opch = self.opch.as_mut_ptr();
        let digit = self.digit.as_mut_ptr();
        let dot = self.dot.as_mut_ptr();
        let misc = self.misc.as_mut_ptr();
        let kind = self.kind.as_mut_ptr();
        let kwpos = self.kwpos.as_mut_ptr();
        let t: &Tables = &self.tables;

        // Keyword recognition is mode-scoped: the TS set (and its wider
        // kwinit letter class) only ever sees TS input, so JS lexing is
        // byte-identical to a build without it.
        let kws = if ts { &t.kwts } else { &t.kwjs };
        classify(t, ts, sp, n, word, st, kwinit, opch, digit, dot, misc, kind);
        *word.add(nb) = 0;
        *st.add(nb) = 0;
        *kwinit.add(nb) = 0;
        *opch.add(nb) = 0;
        *digit.add(nb) = 0;
        *dot.add(nb) = 0;
        *misc.add(nb) = 0;

        let mut nesc = 0usize;
        if bm_any(misc, nb) {
            nesc = if vutf8 {
                misc_pre::<true>(sp, n, st, word, misc, kind, &mut self.lanes)
            } else {
                misc_pre::<false>(sp, n, st, word, misc, kind, &mut self.lanes)
            };
        }
        if jsx {
            carve_jsx(t, src, n, st, kind, opch, word, digit, dot, kwinit, ts, &mut self.lanes);
        } else {
            carve(t, src, n, st, kind, opch, word, digit, ts, &mut self.lanes);
        }
        coalesce(t, kws, sp, n, st, opch, word, digit, dot, kwinit, kind, kwpos, &mut self.lanes);
        if nesc != 0 {
            misc_post(sp, n, st, word, misc, kind);
        }
        let m = compress(t, st, kind, nb, out_starts, out_kinds);
        *out_kinds.add(m) = EOF;
        *out_starts.add(m) = n as u32;
        *out_starts.add(m + 1) = n as u32;
        lanes_post(src, out_kinds, out_starts, m, &mut self.lanes);
        m + 1
    }

    /// Lex `src[..n]` into the internal `starts`/`kinds` buffers (mode from
    /// `options`), returning the token count including EOF. Test/bench
    /// entry; the arena API is [`crate::lex_utf8`].
    ///
    /// # Panics
    ///
    /// Panics if `src` does not extend at least [`PAD`] zeroed bytes past `n`.
    pub fn lex(&mut self, src: &[u8], n: usize, options: LexOptions) -> usize {
        assert!(
            src.len() >= n + PAD,
            "lexer: src must have >= {PAD} bytes of padding past len {n} (got {})",
            src.len()
        );
        self.ensure(n);
        let kinds = self.kinds.as_mut_ptr();
        let starts = self.starts.as_mut_ptr();
        unsafe {
            self.lex_raw(
                src,
                n,
                kinds,
                starts,
                options.jsx,
                options.ts,
                options.source_type_module,
                options.validate_utf8,
            )
        }
    }
}
