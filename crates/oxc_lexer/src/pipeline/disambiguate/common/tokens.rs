//! The token stream as the disambiguation questions read it.

use crate::token::{KW_KIND_BASE, OP_KIND_BASE, is_trivia_byte, tk};

use crate::pipeline::{bytes::line_break_in, operators::opmap_longest, tables::Tables};

use super::{Brackets, Closers, bits, walk};

/// A read-only view of a lex in progress.
///
/// The pipeline builds one at each question, over the buffers the asking stage is filling, and
/// drops it when the question is answered. The bitmaps have one bit per source byte.
#[derive(Clone, Copy)]
pub(crate) struct Tokens<'a> {
    pub(crate) tables: &'a Tables,
    /// The source, followed by at least `PAD` zero bytes.
    pub(crate) src: &'a [u8],
    /// Token starts.
    pub(crate) st: &'a [u64],
    /// Operator characters.
    pub(crate) opch: &'a [u64],
    /// Identifier characters.
    pub(crate) word: &'a [u64],
    /// The kind at each token start.
    pub(crate) kind: &'a [u8],
    /// Source length.
    pub(crate) n: usize,
    pub(crate) ts: bool,
    /// The source is a module (`await` and `yield` are reserved at the top level).
    pub(crate) module: bool,
    /// Token starts below this position carry their final keyword kind (`coalesce` has flushed
    /// them): a `tk!(Ident)` there is a plain name.
    pub(crate) kw_final: usize,
    /// The lex's bracket bitmap, built as the matches reach it.
    pub(crate) brackets: &'a Brackets,
    /// Closers the forward scans resolved past their byte cap, kept for the rest of the lex.
    pub(crate) closers: &'a Closers,
}

/// The significant token before a position.
#[derive(Clone, Copy)]
pub(crate) enum Prev {
    None,
    /// An operator: its start and first byte.
    Op(usize, u8),
    /// A word: its start and keyword code (0 for a plain name).
    Word(usize, u8),
    /// A literal, template part or JSX token.
    Other(usize),
}

#[derive(Clone, Copy)]
pub(crate) struct Peek {
    pub(crate) pos: usize,
    pub(crate) kind: u8,
    pub(crate) byte: u8,
}

impl Prev {
    pub(crate) fn is_member_dot(self, src: &[u8]) -> bool {
        matches!(self, Prev::Op(q, c) if c == b'.' || (c == b'?' && src[q + 1] == b'.'))
    }
}

impl Tokens<'_> {
    /// Base kind: keywords back to `tk!(Ident)`, escaped names to their plain kind.
    #[inline]
    pub(crate) fn base_kind(&self, pos: usize) -> u8 {
        let k = self.kind[pos];
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
    #[inline]
    pub(crate) fn word_kw(&self, pos: usize, len: usize) -> u8 {
        let raw = self.kind[pos];
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
        let set = if self.ts { &self.tables.keywords.kwts } else { &self.tables.keywords.kwjs };
        let k = set.lookup_at(self.src, pos, len) as u8;
        if k >= KW_KIND_BASE { k } else { 0 }
    }

    /// Keyword code of the word at `w` (0 for a plain name).
    #[inline]
    pub(crate) fn ident_kw(&self, w: usize) -> u8 {
        let e = self.next_start(w + 1);
        self.word_kw(w, e - w)
    }

    /// The token start at or after `i`, or `n`.
    #[inline]
    pub(crate) fn next_start(&self, i: usize) -> usize {
        bits::next1(self.st, i, self.n)
    }

    /// The significant token start at or after `i` (skipping trivia), or `n`.
    #[inline]
    pub(crate) fn next_sig(&self, mut i: usize) -> usize {
        loop {
            i = self.next_start(i);
            if i >= self.n {
                return self.n;
            }
            if is_trivia_byte(self.kind[i]) {
                i += 1;
                continue;
            }
            return i;
        }
    }

    #[inline]
    pub(crate) fn peek(&self, i: usize) -> Peek {
        let pos = self.next_sig(i);
        let kind = if pos < self.n { self.base_kind(pos) } else { tk!(Eof) };
        Peek { pos, kind, byte: self.src[pos] }
    }

    /// The significant token start before `pos` (skipping trivia), or `None` at the start.
    #[inline]
    pub(crate) fn prev_sig(&self, pos: usize) -> Option<usize> {
        walk::prev_sig(self.st, self.kind, pos)
    }

    /// The significant token before `p`.
    pub(crate) fn prev_token(&self, p: usize) -> Prev {
        let Some(q) = self.prev_sig(p) else {
            return Prev::None;
        };
        let k = self.base_kind(q);
        if k >= OP_KIND_BASE {
            Prev::Op(q, self.src[q])
        } else if k == tk!(Ident) {
            Prev::Word(q, self.ident_kw(q))
        } else {
            Prev::Other(q)
        }
    }

    /// Is the word at `w` a property name (`x.if`, `x?.if`)?
    pub(crate) fn property_name(&self, w: usize) -> bool {
        self.prev_token(w).is_member_dot(self.src)
    }

    /// Number of `c` bytes the token at `p` starts with (a fused `>>>` counts three, a lone `>`
    /// one).
    pub(crate) fn run_len(&self, p: usize, c: u8) -> i32 {
        let e = self.next_start(p + 1);
        let mut i = p;
        while i < e && self.src[i] == c {
            i += 1;
        }
        (i - p) as i32
    }

    /// Length of the fused operator at pos, 1 when no multi-byte operator starts there.
    #[inline]
    pub(crate) fn op_len(&self, pos: usize) -> usize {
        let lmax = (self.n - pos).min(4) as u32;
        let bytes = <[u8; 4]>::try_from(&self.src[pos..pos + 4]).unwrap();
        opmap_longest(bytes, lmax).1 as usize
    }

    /// The TemplateHead of the template whose tail or middle is at tail; None past cap steps.
    pub(crate) fn template_head(&self, tail: usize, steps: &mut u32, cap: u32) -> Option<usize> {
        let mut depth = 1i32;
        let mut t = self.prev_sig(tail);
        while let Some(tp) = t {
            *steps += 1;
            if *steps > cap {
                return None;
            }
            match self.base_kind(tp) {
                tk!(TemplateTail) => depth += 1,
                tk!(TemplateHead) => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(tp);
                    }
                }
                _ => {}
            }
            t = self.prev_sig(tp);
        }
        None
    }

    /// Does a line terminator lie in `a..b`?
    #[inline]
    pub(crate) fn line_break_between(&self, a: usize, b: usize) -> bool {
        line_break_in(self.src, a, b)
    }

    /// Does the identifier at `pos` equal exactly `kw`?
    #[inline]
    pub(crate) fn ident_is(&self, pos: usize, kw: &[u8]) -> bool {
        walk::ident_is(self.src, pos, kw)
    }

    /// The opener of the `)`, `]` or `}` at `from`, or `None` if unbalanced
    /// (see [`walk::match_delim_back`]).
    #[inline]
    pub(crate) fn match_delim_back(&self, from: usize, open: u8, close: u8) -> Option<usize> {
        walk::match_delim_back(self, from, open, close)
    }
}
