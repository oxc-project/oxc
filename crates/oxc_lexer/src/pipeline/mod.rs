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

use oxc_span::Span;

use crate::PAD;
use crate::lanes::Lanes;
use crate::options::LexOptions;
use crate::tables::Tables;
use crate::token::SPAN_SENTINELS;

use bitmap::bm_any;
use carve::{carve, carve_jsx};
use classify::{classify, misc_post, misc_pre};
use coalesce::coalesce;
use compress::{build_spans, compress, lanes_post, write_sentinels};
use keywords::KWB;

use crate::token::TokenKind;

const STAGE_BLOCKS: usize = 32;
const STAGE_CAP: usize = STAGE_BLOCKS * 64 + 128;

// Short kind aliases for the pipeline, tied to `token_kind` so they can't drift.
pub(crate) const WS: u8 = TokenKind::Whitespace as u8;
pub(crate) const IDENT: u8 = TokenKind::Ident as u8;
pub(crate) const NUM: u8 = TokenKind::Number as u8;
pub(crate) const BIGINT: u8 = TokenKind::BigInt as u8;
pub(crate) const STR: u8 = TokenKind::String as u8;
pub(crate) const LCOM: u8 = TokenKind::LineComment as u8;
pub(crate) const BCOM: u8 = TokenKind::BlockComment as u8;
pub(crate) const REGEX: u8 = TokenKind::RegExp as u8;
pub(crate) const TMPL_NOSUB: u8 = TokenKind::TemplateNoSub as u8;
pub(crate) const TMPL_HEAD: u8 = TokenKind::TemplateHead as u8;
pub(crate) const TMPL_MIDDLE: u8 = TokenKind::TemplateMiddle as u8;
pub(crate) const TMPL_TAIL: u8 = TokenKind::TemplateTail as u8;
pub(crate) const HASHBANG: u8 = TokenKind::Hashbang as u8;
pub(crate) const IDENT_ESC: u8 = TokenKind::IdentEscaped as u8;
pub(crate) const PRIV_IDENT: u8 = TokenKind::PrivateIdent as u8;
pub(crate) const PRIV_IDENT_ESC: u8 = TokenKind::PrivateIdentEscaped as u8;
pub(crate) const EOF: u8 = TokenKind::Eof as u8;

// JSX coarse kinds, written only by `carve_jsx`. `JEND`/`JSX_LT` read as
// values in `prev_is_regex` (after a completed element, `/` is division).
pub(crate) const JTEXT: u8 = TokenKind::JsxText as u8;
pub(crate) const JEND: u8 = TokenKind::JsxTagEnd as u8;
pub(crate) const JSX_LT: u8 = TokenKind::JsxLt as u8;

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
    stage_pos: Vec<u32>,
    stage_kind: Vec<u8>,
    pub spans: Vec<Span>,
    sig_kinds: Vec<u8>,
    pub sig_len: usize,
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
            stage_pos: vec![0; STAGE_CAP],
            stage_kind: vec![0; STAGE_CAP],
            spans: Vec::new(),
            sig_kinds: Vec::new(),
            sig_len: 0,
            out_cap: 0,
            lanes: Lanes::default(),
            tables: Box::new(Tables::new()),
        }
    }

    /// The kinds written by the last [`Lexer::lex`], including the trailing
    /// [`SPAN_SENTINELS`] EOF entries.
    #[must_use]
    pub fn kinds(&self) -> &[TokenKind] {
        let bytes = &self.sig_kinds[..self.sig_len + SPAN_SENTINELS];
        crate::token::debug_assert_kind_bytes(bytes);
        // SAFETY: `lex_raw` wrote `sig_len` kinds plus the sentinels, all of
        // them declared discriminants.
        unsafe { crate::token::kinds_from_bytes(bytes) }
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
            self.spans.resize(need + SPAN_SENTINELS, Span::new(0, 0));
            self.sig_kinds.resize(need + SPAN_SENTINELS, 0);
            self.out_cap = need;
        }
    }

    /// Run the full pipeline over `src[..n]`, writing the trivia-free token
    /// stream into `out_kinds`/`out_spans` ([`SPAN_SENTINELS`] EOF entries
    /// spanning `(n, n)` follow the last token) and the value lanes and
    /// diagnostics into `self.lanes`. Returns the significant token count,
    /// excluding the sentinels.
    ///
    /// # Safety
    ///
    /// - `src` must extend at least [`PAD`] zeroed bytes past `n`.
    /// - `out_kinds` must be valid for `n + PAD + SPAN_SENTINELS` byte writes
    ///   and `out_spans` for the same number of [`Span`] writes: `build_spans`
    ///   stores full 4-lane groups past the last token, and the sentinels
    ///   follow it.
    pub unsafe fn lex_raw(
        &mut self,
        src: &[u8],
        n: usize,
        out_kinds: *mut u8,
        out_spans: *mut Span,
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
            write_sentinels(0, out_spans, out_kinds);
            self.sig_len = 0;
            return 0;
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
        let stage_pos = self.stage_pos.as_mut_ptr();
        let stage_kind = self.stage_kind.as_mut_ptr();
        let mut c = 0usize;
        let mut w = 0usize;
        let mut b = 0usize;
        while b < nb {
            let b1 = (b + STAGE_BLOCKS).min(nb);
            c += compress(t, st, kind, b, b1, stage_pos.add(c), stage_kind.add(c));
            b = b1;
            if c > 1 {
                w += build_spans(stage_kind, stage_pos, c - 1, out_spans.add(w), out_kinds.add(w));
                *stage_pos = *stage_pos.add(c - 1);
                *stage_kind = *stage_kind.add(c - 1);
                c = 1;
            }
        }
        if c == 1 {
            *stage_pos.add(1) = n as u32;
            w += build_spans(stage_kind, stage_pos, 1, out_spans.add(w), out_kinds.add(w));
        }
        write_sentinels(n as u32, out_spans.add(w), out_kinds.add(w));
        lanes_post(src, out_kinds, out_spans, w, n as u32, &mut self.lanes);
        self.sig_len = w;
        w
    }

    /// Lex `src[..n]` into the internal `spans`/`sig_kinds` buffers (mode from
    /// `options`), returning the significant token count. Test/bench entry;
    /// the arena API is [`crate::lex_utf8`].
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
        let kinds = self.sig_kinds.as_mut_ptr();
        let spans = self.spans.as_mut_ptr();
        unsafe {
            self.lex_raw(
                src,
                n,
                kinds,
                spans,
                options.jsx,
                options.ts,
                options.source_type_module,
                options.validate_utf8,
            )
        }
    }
}
