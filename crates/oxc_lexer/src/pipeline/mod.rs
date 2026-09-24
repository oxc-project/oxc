// Kernel lint policy (this module tree and `lanes`):
// the `unsafe fn` boundary is the reviewed surface, and the pedantic/nursery
// style lints fight the SIMD idiom. API modules keep the full workspace bar.
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(clippy::needless_range_loop, clippy::manual_range_contains)]

use oxc_span::Span;

use crate::{
    PAD,
    lanes::Lanes,
    options::LexOptions,
    token::{SPAN_SENTINELS, TokenKind, debug_assert_kind_bytes, kinds_from_bytes},
};

mod bitmap;
mod bytes;
mod carve;
mod chunk;
mod classify;
mod coalesce;
mod compress;
mod disambiguate;
mod find;
mod misc;
mod operators;
mod scan;
mod tables;

pub(crate) use disambiguate::State as DisambiguateState;

use carve::carve;
use classify::classify;
use coalesce::{KWB, coalesce};
use compress::{STAGE_CAP, compress, write_sentinels};
use misc::{misc_post, misc_pre};
use tables::Tables;

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

    /// Lex `src[..n]` into the internal `spans`/`sig_kinds` buffers (mode from
    /// `options`), returning the significant token count. Test/bench entry;
    /// the arena API is [`lex_utf8`].
    ///
    /// # Panics
    ///
    /// Panics if `src` does not extend at least [`PAD`] zeroed bytes past `n`.
    ///
    /// [`lex_utf8`]: crate::lex_utf8
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

    /// Run the full pipeline over `src[..n]`, writing the trivia-free token
    /// stream into `out_kinds`/`out_spans` ([`SPAN_SENTINELS`] EOF entries
    /// spanning `(n, n)` follow the last token) and the value lanes and
    /// diagnostics into `self.lanes`. Returns the significant token count,
    /// excluding the sentinels.
    ///
    /// # SAFETY
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
        self.lanes.disambiguate.begin(n, module);
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
        classify(t, ts, sp, n, nb, word, st, kwinit, opch, digit, dot, misc, kind);
        let nesc = misc_pre(sp, n, nb, st, word, misc, kind, vutf8, &mut self.lanes);
        carve(t, src, n, st, kind, opch, word, digit, dot, kwinit, jsx, ts, &mut self.lanes);
        coalesce(t, sp, n, st, opch, word, digit, dot, kwinit, kind, kwpos, ts, &mut self.lanes);
        misc_post(sp, n, st, word, misc, kind, nesc);
        let w = compress(
            t,
            src,
            n,
            nb,
            st,
            kind,
            self.stage_pos.as_mut_ptr(),
            self.stage_kind.as_mut_ptr(),
            out_kinds,
            out_spans,
            &mut self.lanes,
        );
        self.sig_len = w;
        w
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

    /// The kinds written by the last [`Lexer::lex`], including the trailing
    /// [`SPAN_SENTINELS`] EOF entries.
    #[must_use]
    pub fn kinds(&self) -> &[TokenKind] {
        let bytes = &self.sig_kinds[..self.sig_len + SPAN_SENTINELS];
        debug_assert_kind_bytes(bytes);
        // SAFETY: `lex_raw` wrote `sig_len` kinds plus the sentinels, all of
        // them declared discriminants.
        unsafe { kinds_from_bytes(bytes) }
    }
}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}

/// The lex in progress as `disambiguate` reads it, over the buffers [`Lexer::ensure`] sized.
///
/// # SAFETY
///
/// - `src` must be valid for `n + PAD` bytes, `st`, `opch` and `word` for `n / 64 + 1` words and
///   `kind` for `(n / 64 + 1) * 64` bytes.
/// - Nothing may write to them while the view is alive: a question reads, answers and returns
///   before the stage writes again.
///
/// `brackets` is the lex's bracket cache, `lanes.disambiguate.brackets`.
unsafe fn token_view<'a>(
    t: &'a Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    word: *const u64,
    kind: *const u8,
    n: usize,
    ts: bool,
    kw_final: usize,
    module: bool,
    brackets: &'a disambiguate::Brackets,
    closers: &'a disambiguate::Closers,
) -> disambiguate::Tokens<'a> {
    let nb = n.div_ceil(64) + 1;
    disambiguate::Tokens {
        tables: t,
        src: std::slice::from_raw_parts(src, n + PAD),
        st: std::slice::from_raw_parts(st, nb),
        opch: std::slice::from_raw_parts(opch, nb),
        word: std::slice::from_raw_parts(word, nb),
        kind: std::slice::from_raw_parts(kind, nb * 64),
        n,
        ts,
        module,
        kw_final,
        brackets,
        closers,
    }
}
