use std::mem;

use oxc_simd::{Cpu, DynCpu, cpu_multiversion, cpu_version};
use oxc_span::Span;

use crate::{
    PAD,
    options::LexOptions,
    token::{TokenKind, kind_bytes},
    tokens::Tokens,
};

use crate::pipeline::{
    bitmap::{Bitmap, Bitmaps},
    carve::carve,
    classify::classify,
    coalesce::{BATCH_SLOTS, coalesce},
    compress::{STAGE_CAP, compress, init_pair_luts, lanes_post},
    kinds::Kinds,
    lex::{Lex, LexState},
    misc::{misc_post, misc_pre, retain_code_whitespace},
    source::Source,
};

pub struct Lexer {
    token_start: Vec<u64>,
    word: Vec<u64>,
    keyword_initial: Vec<u64>,
    op_char: Vec<u64>,
    digit: Vec<u64>,
    dot: Vec<u64>,
    misc: Vec<u64>,
    kinds: Vec<u8>,
    kw_candidates: Vec<u32>,
    events: Vec<u32>,
    block_capacity: usize,
    stage_pos: Box<[u32; STAGE_CAP]>,
    stage_kind: Box<[u8; STAGE_CAP]>,
    pub(super) state: LexState,
    cpu: DynCpu,
}

impl Lexer {
    /// Create a new [`Lexer`].
    #[cpu_version(Generic)]
    pub fn new() -> Lexer {
        // Tests and conformance must use `Lexer::new_with_cpu` so that they test both AVX2
        // and generic implementations on x86_64, using the `DynCpu` provided by `#[test_cpus]`
        if cfg!(any(test, feature = "testing")) {
            panic!(
                "Tests must use `Lexer::new_with_cpu` or `testing::lex_with_cpu`, not `Lexer::new`"
            );
        }

        #[expect(clippy::disallowed_methods, reason = "tests call `new_with_cpu` instead")]
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
            token_start: Vec::new(),
            word: Vec::new(),
            keyword_initial: Vec::new(),
            op_char: Vec::new(),
            digit: Vec::new(),
            dot: Vec::new(),
            misc: Vec::new(),
            kinds: Vec::new(),
            kw_candidates: Vec::new(),
            events: Vec::new(),
            block_capacity: 0,
            stage_pos: Box::new([0; STAGE_CAP]),
            stage_kind: Box::new([0; STAGE_CAP]),
            state: LexState::default(),
            cpu,
        }
    }

    /// # Panics
    ///
    /// Panics if `src` does not extend at least [`PAD`] zeroed bytes past `n`.
    #[cpu_version(Generic)]
    pub fn lex(&mut self, src: &[u8], n: usize, options: LexOptions, tokens: &mut Tokens) {
        // `n.saturating_add(PAD)` not `n + PAD` to ensure this assertion catches
        // e.g. `src.len() == 100, n == usize::MAX`, where `n + PAD` would wrap around to `PAD - 1`
        // so `src.len() >= n + PAD` would be true.
        // Slices are at most `isize::MAX` bytes, so cannot be >= a saturated `usize::MAX`.
        assert!(
            src.len() >= n.saturating_add(PAD),
            "lexer: src must have >= {PAD} bytes of padding past len {n}"
        );
        let src = Source::new(src, n);
        // Lend the caller's lane buffers to the stages, so they fill them in place.
        self.state.begin(mem::take(&mut tokens.lanes), src.len(), &options, self.cpu);
        let count = self.lex_raw(src, &mut tokens.kinds, &mut tokens.spans, options);
        tokens.lanes =
            self.state.finish(src.bytes(), &tokens.kinds[..count], &tokens.spans[..count]);
    }

    #[cpu_version(Generic)]
    fn lex_raw(
        &mut self,
        src: Source,
        out_kinds: &mut Vec<TokenKind>,
        out_spans: &mut Vec<Span>,
        options: LexOptions,
    ) -> usize {
        self.lex_raw_impl(src, out_kinds, out_spans, options, self.cpu)
    }

    #[cpu_multiversion]
    fn lex_raw_impl(
        &mut self,
        src: Source,
        out_kinds: &mut Vec<TokenKind>,
        out_spans: &mut Vec<Span>,
        options: LexOptions,
        cpu: impl Cpu,
    ) -> usize {
        self.ensure(src, cpu);

        let bitmaps = &Bitmaps {
            token_start: Bitmap::new(&mut self.token_start),
            word: Bitmap::new(&mut self.word),
            keyword_initial: Bitmap::new(&mut self.keyword_initial),
            op_char: Bitmap::new(&mut self.op_char),
            digit: Bitmap::new(&mut self.digit),
            dot: Bitmap::new(&mut self.dot),
            misc: Bitmap::new(&mut self.misc),
        };
        // SAFETY: the buffer starts zeroed, which is Eof, and the stages write only declared kinds.
        let kinds = unsafe { Kinds::new(&mut self.kinds) };
        let lex = Lex { src, bitmaps, kinds, options: &options };
        if src.len() == 0 {
            return compress(
                lex,
                &mut self.stage_pos,
                &mut self.stage_kind,
                out_kinds,
                out_spans,
                cpu,
            );
        }
        // Keyword recognition is mode-scoped: the TS set (and its wider
        // keyword_initial letter class) only ever sees TS input, so JS lexing is
        // byte-identical to a build without it.
        classify(lex, cpu);
        let escaped_ident_count = misc_pre(lex, &mut self.state, cpu);
        carve(lex, &mut self.state, cpu);
        coalesce(lex, &mut self.kw_candidates, &mut self.events, &mut self.state, cpu);
        misc_post(lex, escaped_ident_count, cpu);
        if !self.state.lanes.irregular_whitespaces.is_empty() {
            let spans = &mut self.state.lanes.irregular_whitespaces;
            retain_code_whitespace(bitmaps.token_start, kinds, spans);
        }
        let count =
            compress(lex, &mut self.stage_pos, &mut self.stage_kind, out_kinds, out_spans, cpu);
        let (values, spans) = (kind_bytes(&out_kinds[..count]), &out_spans[..count]);
        lanes_post(src.padded(), values, spans, src.len() as u32, &mut self.state.lanes, cpu);
        count
    }

    #[cpu_multiversion]
    #[inline(never)]
    fn ensure(&mut self, src: Source, _cpu: impl Cpu) {
        // One spare block past the source, which the bit scans may read.
        let blocks = src.block_count() + 1;
        if self.block_capacity < blocks {
            self.token_start.resize(blocks, 0);
            self.word.resize(blocks, 0);
            self.keyword_initial.resize(blocks, 0);
            self.op_char.resize(blocks, 0);
            self.digit.resize(blocks, 0);
            self.dot.resize(blocks, 0);
            self.misc.resize(blocks, 0);
            self.kinds.resize(blocks * 64, 0);
            self.block_capacity = blocks;
        }
        if self.kw_candidates.is_empty() {
            self.kw_candidates.resize(BATCH_SLOTS, 0);
            self.events.resize(BATCH_SLOTS, 0);
        }
    }
}

impl Default for Lexer {
    #[cpu_version(Generic)]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use oxc_simd::{DynCpu, test_cpus};

    use crate::{LexOptions, PAD, Tokens};

    use super::Lexer;

    #[test_cpus]
    #[should_panic(expected = "bytes of padding past len")]
    fn lex_panics_if_n_plus_pad_overflows(cpu: DynCpu) {
        let src = [0; PAD];
        let mut tokens = Tokens::new();
        Lexer::new_with_cpu(cpu).lex(&src, usize::MAX, LexOptions::default(), &mut tokens);
    }
}
