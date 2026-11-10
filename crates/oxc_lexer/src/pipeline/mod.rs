// Kernel lint policy (this module tree and `lanes`):
// the `unsafe fn` boundary is the reviewed surface, and the pedantic/nursery
// style lints fight the SIMD idiom. API modules keep the full workspace bar.
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(clippy::needless_range_loop, clippy::manual_range_contains)]

use std::{mem, slice};

use oxc_simd::{Cpu, DynCpu, cpu_multiversion, cpu_version};
use oxc_span::Span;

use crate::{
    PAD,
    options::LexOptions,
    token::{SPAN_SENTINELS, debug_assert_kind_bytes},
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
mod keywords;
mod lanes;
mod misc;
mod operators;
mod scan;
mod unicode_leads;

pub(crate) use lanes::TokenLanes;

use carve::carve;
use classify::classify;
use coalesce::{KWB, coalesce};
use compress::{STAGE_CAP, compress, init_pair_luts, write_sentinels};
use disambiguate::{Brackets, Closers, Tokens};
use lanes::Lanes;
use misc::{misc_post, misc_pre, retain_code_whitespace};
use unicode_leads::resolve_unicode_leads;

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
    lanes: Lanes,
    cpu: DynCpu,
}

impl Lexer {
    /// Create a new [`Lexer`].
    #[cpu_version(Generic)]
    pub fn new() -> Lexer {
        let cpu = DynCpu::detect();
        Self::new_with_cpu(cpu)
    }

    /// Create a new [`Lexer`] with specified CPU feature set.
    ///
    /// Should only be called directly by tests. Use [`new`] for production code.
    ///
    /// [`new`]: Self::new
    #[cpu_version(Generic)]
    #[inline]
    pub(crate) fn new_with_cpu(cpu: DynCpu) -> Lexer {
        init_pair_luts(cpu);

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
            lanes: Lanes::default(),
            cpu,
        }
    }

    /// # Panics
    ///
    /// Panics if `src` does not extend at least [`PAD`] zeroed bytes past `n`.
    #[cpu_version(Generic)]
    pub fn lex(&mut self, src: &[u8], n: usize, options: LexOptions, tokens: &mut crate::Tokens) {
        assert!(
            src.len() >= n + PAD,
            "lexer: src must have >= {PAD} bytes of padding past len {n} (got {})",
            src.len()
        );
        let capacity = n + PAD + SPAN_SENTINELS;
        tokens.kinds.clear();
        tokens.kinds.reserve(capacity);
        tokens.spans.clear();
        tokens.spans.reserve(capacity);
        // Lend the caller's lane buffers to the stages, so they fill them in place.
        self.lanes.out = mem::take(&mut tokens.lanes);
        // SAFETY: lex_raw writes count + SPAN_SENTINELS valid kinds and spans into reserved space.
        let count = unsafe {
            let count = self.lex_raw(
                src,
                n,
                tokens.kinds.as_mut_ptr().cast::<u8>(),
                tokens.spans.as_mut_ptr(),
                options.jsx,
                options.ts,
                options.source_type_module,
                options.validate_utf8,
            );
            let len = count + SPAN_SENTINELS;
            debug_assert_kind_bytes(slice::from_raw_parts(tokens.kinds.as_ptr().cast::<u8>(), len));
            tokens.kinds.set_len(len);
            tokens.spans.set_len(len);
            count
        };
        if !self.lanes.unicode_leads.is_empty() && count > 0 {
            let (kinds, spans) = (&tokens.kinds[..count], &tokens.spans[..count]);
            resolve_unicode_leads(&mut self.lanes, &src[..n], kinds, spans);
        }
        if !self.lanes.diag_suppress.is_empty() {
            let (diags, sup) = (&mut self.lanes.out.diags, &self.lanes.diag_suppress);
            diags.retain(|d| !sup.iter().any(|&(a, b)| d.off >= a && d.off < b));
        }
        tokens.lanes = mem::take(&mut self.lanes.out);
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
    #[cpu_version(Generic)]
    unsafe fn lex_raw(
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
        self.lex_raw_impl(src, n, out_kinds, out_spans, jsx, ts, module, vutf8, self.cpu)
    }

    #[cpu_multiversion]
    unsafe fn lex_raw_impl(
        &mut self,
        src: &[u8],
        n: usize,
        out_kinds: *mut u8,
        out_spans: *mut Span,
        jsx: bool,
        ts: bool,
        module: bool,
        vutf8: bool,
        cpu: impl Cpu,
    ) -> usize {
        debug_assert!(src.len() >= n + PAD, "source must extend PAD zeroed bytes past n");
        self.ensure(n, cpu);
        self.lanes.clear(cpu);
        self.lanes.module = module;
        self.lanes.disambiguate.begin(n, module, cpu);
        if n == 0 {
            write_sentinels(0, out_spans, out_kinds, cpu);
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

        // Keyword recognition is mode-scoped: the TS set (and its wider
        // kwinit letter class) only ever sees TS input, so JS lexing is
        // byte-identical to a build without it.
        classify(ts, sp, n, nb, word, st, kwinit, opch, digit, dot, misc, kind, cpu);
        let nesc = misc_pre(sp, n, nb, st, word, misc, kind, vutf8, &mut self.lanes, cpu);
        carve(src, n, st, kind, opch, word, digit, dot, kwinit, jsx, ts, &mut self.lanes, cpu);
        coalesce(sp, n, st, opch, word, digit, dot, kwinit, kind, kwpos, ts, &mut self.lanes, cpu);
        misc_post(sp, n, st, word, misc, kind, nesc, cpu);
        if !self.lanes.out.irregular_whitespaces.is_empty() {
            retain_code_whitespace(&self.st, &self.kind, &mut self.lanes.out.irregular_whitespaces);
        }
        compress(
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
            cpu,
        )
    }

    #[cpu_multiversion]
    #[inline(never)]
    fn ensure(&mut self, n: usize, _cpu: impl Cpu) {
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
    }
}

impl Default for Lexer {
    #[cpu_version(Generic)]
    fn default() -> Self {
        Self::new()
    }
}

/// The lex in progress as `disambiguate` reads it, over the buffers [`Lexer::ensure`] sized.
///
/// # SAFETY
///
/// - src must be valid for n + PAD bytes, st and opch for n / 64 + 1 words and
///   `kind` for `(n / 64 + 1) * 64` bytes.
/// - Nothing may write to them while the view is alive: a question reads, answers and returns
///   before the stage writes again.
///
/// `brackets` is the lex's bracket cache, `lanes.disambiguate.brackets`.
#[cpu_multiversion]
#[inline(always)]
unsafe fn token_view<'a>(
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    ts: bool,
    kw_final: usize,
    module: bool,
    brackets: &'a Brackets,
    closers: &'a Closers,
    _cpu: impl Cpu,
) -> Tokens<'a> {
    let nb = n.div_ceil(64) + 1;
    Tokens {
        src: slice::from_raw_parts(src, n + PAD),
        st: slice::from_raw_parts(st, nb),
        opch: slice::from_raw_parts(opch, nb),
        kind: slice::from_raw_parts(kind, nb * 64),
        n,
        ts,
        module,
        kw_final,
        brackets,
        closers,
    }
}
