//! The token stream as the disambiguation questions read it.

use std::slice;

use oxc_simd::{BASELINE, Cpu, cpu_multiversion, cpu_version};

use crate::token::{KW_KIND_BASE, OP_KIND_BASE, TokenKind, is_trivia_byte, tk};

use crate::pipeline::{
    bitmap::Bitmaps,
    bytes::{is_digit, line_break_in},
    keywords::kw_match_at,
    kinds::Kinds,
    operators::opmap_longest,
    source::Source,
};

use super::{Brackets, Closers, bits, walk};

/// A read-only view of a lex in progress.
///
/// The pipeline builds one at each question, over the buffers the asking stage is filling, and
/// drops it when the question is answered. The bitmaps have one bit per source byte.
#[derive(Clone, Copy)]
pub struct TokensView<'a> {
    /// The source, followed by at least `PAD` zero bytes.
    pub src: &'a [u8],
    /// Token starts.
    pub token_start: &'a [u64],
    /// Operator characters.
    pub op_char: &'a [u64],
    /// The kind at each token start.
    pub kinds: &'a [u8],
    /// Source length.
    pub n: usize,
    pub ts: bool,
    /// The source is a module (`await` and `yield` are reserved at the top level).
    pub module: bool,
    /// Token starts below this position carry their final keyword kind (`coalesce` has flushed
    /// them): a `tk!(Ident)` there is a plain name.
    pub kw_final: usize,
    /// The lex's bracket bitmap, built as the matches reach it.
    pub brackets: &'a Brackets,
    /// Closers the forward scans resolved past their byte cap, kept for the rest of the lex.
    pub closers: &'a Closers,
}

/// The significant token before a position.
#[derive(Clone, Copy)]
pub enum Prev {
    None,
    /// An operator: its start and first byte.
    Op(usize, u8),
    /// A word: its start and keyword code (0 for a plain name).
    Word(usize, u8),
    /// A literal, template part or JSX token.
    Other(usize),
}

#[derive(Clone, Copy)]
pub struct Peek {
    pub pos: usize,
    pub kind: u8,
    pub byte: u8,
}

impl Prev {
    #[cpu_version(Generic)]
    pub fn is_member_dot(self, tokens: &TokensView) -> bool {
        let cpu = BASELINE;

        matches!(self, Prev::Op(q, _) if tokens.member_dot(q, cpu))
    }
}

impl<'a> TokensView<'a> {
    /// Create a [`TokensView`], as a view over the buffers [`Lexer::ensure`] sized.
    ///
    /// `brackets` is the lex's bracket cache, `lanes.disambiguate.brackets`.
    /// `closers` is the lex's closer cache, `lanes.disambiguate.closers`.
    ///
    /// # SAFETY
    ///
    /// - token_start and op_char: block_count() + 1 readable words, kinds: 64 bytes per word.
    /// - Nothing may write to them while the view is alive.
    ///   A question reads, answers and returns before the stage writes again.
    ///
    /// [`Lexer::ensure`]: crate::pipeline::lexer::Lexer::ensure
    #[cpu_multiversion]
    #[inline(always)]
    pub unsafe fn new(
        src: Source<'a>,
        bitmaps: &Bitmaps,
        kinds: Kinds,
        ts: bool,
        kw_final: usize,
        module: bool,
        brackets: &'a Brackets,
        closers: &'a Closers,
        _cpu: impl Cpu,
    ) -> Self {
        let Bitmaps { token_start, op_char, .. } = *bitmaps;
        let blocks = src.block_count() + 1;
        Self {
            src: src.padded(),
            token_start: slice::from_raw_parts(token_start.as_ptr(), blocks),
            op_char: slice::from_raw_parts(op_char.as_ptr(), blocks),
            kinds: slice::from_raw_parts(kinds.as_ptr(), blocks * 64),
            n: src.len(),
            ts,
            module,
            kw_final,
            brackets,
            closers,
        }
    }

    /// Base kind: keywords back to `tk!(Ident)`, escaped names to their plain kind.
    #[cpu_multiversion]
    #[inline(always)]
    pub fn base_kind(&self, pos: usize, _cpu: impl Cpu) -> u8 {
        let k = self.kinds[pos];
        if k >= KW_KIND_BASE {
            return tk!(Ident);
        }
        if k == tk!(IdentEscaped) {
            return tk!(Ident);
        }
        if k == tk!(PrivateIdentEscaped) {
            return tk!(PrivateIdent);
        }
        k
    }

    /// Keyword code of the word at `pos` of `len` bytes (0 for a plain identifier).
    #[cpu_multiversion]
    #[inline]
    pub fn word_kw(&self, pos: usize, len: usize, cpu: impl Cpu) -> u8 {
        let raw = self.kinds[pos];
        if raw >= KW_KIND_BASE {
            return raw;
        }
        if raw != tk!(Ident) || pos < self.kw_final {
            return 0;
        }
        // Keywords are lowercase words of up to ten letters: skip the hash for the rest.
        if !self.src[pos].is_ascii_lowercase() {
            return 0;
        }
        let k = kw_match_at(self.ts, self.src, pos, len, cpu);
        if k != TokenKind::Ident { k as u8 } else { 0 }
    }

    /// Keyword code of the word at `w` (0 for a plain name).
    #[cpu_multiversion]
    #[inline(always)]
    pub fn ident_kw(&self, w: usize, cpu: impl Cpu) -> u8 {
        let e = self.next_start(w + 1, cpu);
        self.word_kw(w, e - w, cpu)
    }

    /// The token start at or after `i`, or `n`.
    #[cpu_multiversion]
    #[inline(always)]
    pub fn next_start(&self, i: usize, cpu: impl Cpu) -> usize {
        bits::next1(self.token_start, i, self.n, cpu)
    }

    /// The significant token start at or after `i` (skipping trivia), or `n`.
    #[cpu_multiversion]
    #[inline]
    pub fn next_sig(&self, mut i: usize, cpu: impl Cpu) -> usize {
        loop {
            i = self.next_start(i, cpu);
            if i >= self.n {
                return self.n;
            }
            if is_trivia_byte(self.kinds[i], cpu) {
                i += 1;
                continue;
            }
            return i;
        }
    }

    #[cpu_multiversion]
    #[inline]
    pub fn peek(&self, i: usize, cpu: impl Cpu) -> Peek {
        let pos = self.next_sig(i, cpu);
        let kind = if pos < self.n { self.base_kind(pos, cpu) } else { tk!(Eof) };
        Peek { pos, kind, byte: self.src[pos] }
    }

    /// The significant token start before `pos` (skipping trivia), or `None` at the start.
    #[cpu_multiversion]
    #[inline]
    pub fn prev_sig(&self, pos: usize, cpu: impl Cpu) -> Option<usize> {
        walk::prev_sig(self.token_start, self.kinds, pos, cpu)
    }

    /// The significant token before `p`.
    #[cpu_multiversion]
    pub fn prev_token(&self, p: usize, cpu: impl Cpu) -> Prev {
        let Some(q) = self.prev_sig(p, cpu) else {
            return Prev::None;
        };
        let k = self.base_kind(q, cpu);
        if k >= OP_KIND_BASE {
            Prev::Op(q, self.src[q])
        } else if k == tk!(Ident) {
            Prev::Word(q, self.ident_kw(q, cpu))
        } else {
            Prev::Other(q)
        }
    }

    /// Is the word at `w` a property name (`x.if`, `x?.if`)?
    #[cpu_multiversion]
    #[inline]
    pub fn property_name(&self, w: usize, cpu: impl Cpu) -> bool {
        self.prev_sig(w, cpu)
            .is_some_and(|q| self.kinds[q] >= OP_KIND_BASE && self.member_dot(q, cpu))
    }

    /// Is the operator at q a member access dot, not a spread dot or the start of ?.5?
    #[cpu_multiversion]
    #[inline]
    pub fn member_dot(&self, q: usize, cpu: impl Cpu) -> bool {
        let src = self.src;
        match src[q] {
            b'?' => src[q + 1] == b'.' && !is_digit(src[q + 2], cpu),
            // A spread is three token starts before coalesce, ending here, or one after it.
            b'.' => {
                !(src[q + 1] == b'.' && src[q + 2] == b'.')
                    && !(q >= 2 && src[q - 1] == b'.' && src[q - 2] == b'.')
                    && !self.numeric_dot(q, cpu)
            }
            _ => false,
        }
    }

    /// Does the . at q end a decimal integer literal (1.) rather than access a member?
    #[cpu_multiversion]
    pub fn numeric_dot(&self, q: usize, cpu: impl Cpu) -> bool {
        let Some(p) = bits::prev1(self.token_start, q, cpu) else {
            return false;
        };
        let digits = &self.src[p..q];
        // A fraction, a prefixed or exponent literal, or a legacy octal takes no dot.
        self.kinds[p] == tk!(Number)
            && (p == 0 || self.src[p - 1] != b'.')
            && digits.iter().all(|&c| is_digit(c, cpu) || c == b'_')
            && !(digits.len() > 1 && digits[0] == b'0' && digits.iter().all(|&c| c < b'8'))
    }

    /// Number of `c` bytes the token at `p` starts with (a fused `>>>` counts three, a lone `>`
    /// one).
    #[cpu_multiversion]
    pub fn run_len(&self, p: usize, c: u8, cpu: impl Cpu) -> i32 {
        let e = self.next_start(p + 1, cpu);
        let mut i = p;
        while i < e && self.src[i] == c {
            i += 1;
        }
        (i - p) as i32
    }

    /// Length of the fused operator at pos, 1 when no multi-byte operator starts there.
    #[cpu_version(Generic)]
    #[inline]
    pub fn op_len(&self, pos: usize) -> usize {
        let cpu = BASELINE;

        let bytes = <[u8; 4]>::try_from(&self.src[pos..pos + 4]).unwrap();
        // Provide `max_len` as static value 4 to remove branches in `opmap_longest`.
        // When at the very end of source, `bytes` can contain `\0` padding bytes,
        // but these bytes cannot match any operator. This adds redundant work at the very end
        // of the file (very rare case) in return for fewer branches everywhere else (common).
        opmap_longest(bytes, 4, cpu).1 as usize
    }

    /// The TemplateHead of the template whose tail or middle is at tail; None past cap steps.
    #[cpu_version(Generic)]
    pub fn template_head(&self, tail: usize, steps: &mut u32, cap: u32) -> Option<usize> {
        let cpu = BASELINE;

        let mut depth = 1i32;
        let mut t = self.prev_sig(tail, cpu);
        while let Some(tp) = t {
            *steps += 1;
            if *steps > cap {
                return None;
            }
            match self.base_kind(tp, cpu) {
                tk!(TemplateTail) => depth += 1,
                tk!(TemplateHead) => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(tp);
                    }
                }
                _ => {}
            }
            t = self.prev_sig(tp, cpu);
        }
        None
    }

    /// Does a line terminator lie in `a..b`?
    #[cpu_multiversion]
    #[inline]
    pub fn line_break_between(&self, a: usize, b: usize, cpu: impl Cpu) -> bool {
        line_break_in(self.src, a, b, cpu)
    }

    /// Does the identifier at `pos` equal exactly `kw`?
    #[cpu_multiversion]
    #[inline(always)]
    pub fn ident_is(&self, pos: usize, kw: &[u8], cpu: impl Cpu) -> bool {
        walk::ident_is(self.src, pos, kw, cpu)
    }

    /// The opener of the `)`, `]` or `}` at `from`, or `None` if unbalanced
    /// (see [`walk::match_delim_back`]).
    #[cpu_multiversion]
    #[inline]
    pub fn match_delim_back(&self, from: usize, cpu: impl Cpu) -> Option<usize> {
        let (open, close) = match self.src[from] {
            b')' => (b'(', b')'),
            b']' => (b'[', b']'),
            _ => (b'{', b'}'),
        };
        walk::match_delim_back(self, from, open, close, cpu)
    }
}
