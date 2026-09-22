use crate::{
    pipeline::{bytes::is_digit, tables::Tables},
    token::{KW_KIND_BASE, OP_KIND_BASE, tk},
};

use super::{Brackets, Closers, bits, walk};

#[derive(Clone, Copy)]
pub(crate) struct Tokens<'a> {
    pub(crate) tables: &'a Tables,
    pub(crate) src: &'a [u8],
    pub(crate) st: &'a [u64],
    pub(crate) opch: &'a [u64],
    pub(crate) word: &'a [u64],
    pub(crate) kind: &'a [u8],
    pub(crate) n: usize,
    pub(crate) ts: bool,
    pub(crate) module: bool,
    /// Below this, keyword kinds are final (coalesce has flushed them).
    pub(crate) kw_final: usize,
    pub(crate) brackets: &'a Brackets,
    pub(crate) closers: &'a Closers,
}

#[derive(Clone, Copy)]
pub(crate) enum Prev {
    None,
    Op(usize, u8),
    Word(usize, u8),
    Other(usize),
}

impl Tokens<'_> {
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

    #[inline]
    pub(crate) fn word_kw(&self, pos: usize, len: usize) -> u8 {
        let raw = self.kind[pos];
        if raw >= KW_KIND_BASE {
            return raw;
        }
        if raw != tk!(Ident) || pos < self.kw_final {
            return 0;
        }
        let set = if self.ts { &self.tables.keywords.kwts } else { &self.tables.keywords.kwjs };
        let k = set.lookup_at(self.src, pos, len) as u8;
        if k >= KW_KIND_BASE { k } else { 0 }
    }

    #[inline]
    pub(crate) fn ident_kw(&self, w: usize) -> u8 {
        let e = self.next_start(w + 1);
        self.word_kw(w, e - w)
    }

    #[inline]
    pub(crate) fn next_start(&self, i: usize) -> usize {
        bits::next1(self.st, i, self.n)
    }

    #[inline]
    pub(crate) fn next_sig(&self, mut i: usize) -> usize {
        loop {
            i = self.next_start(i);
            if i >= self.n {
                return self.n;
            }
            let k = self.kind[i];
            if k == tk!(Whitespace)
                || k == tk!(LineComment)
                || k == tk!(BlockComment)
                || k == tk!(Hashbang)
            {
                i += 1;
                continue;
            }
            return i;
        }
    }

    #[inline]
    pub(crate) fn prev_sig(&self, pos: usize) -> Option<usize> {
        walk::prev_sig(self.st, self.kind, pos)
    }

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

    pub(crate) fn property_name(&self, w: usize) -> bool {
        matches!(self.prev_token(w), Prev::Op(q, c) if c == b'.' || (c == b'?' && self.src[q + 1] == b'.'))
    }

    /// A fused >>> counts three, a lone > one.
    pub(crate) fn run_len(&self, p: usize, c: u8) -> i32 {
        let e = self.next_start(p + 1);
        let mut i = p;
        while i < e && self.src[i] == c {
            i += 1;
        }
        (i - p) as i32
    }

    #[inline]
    pub(crate) fn munch(&self, pos: usize) -> (usize, u32) {
        let b0 = self.src[pos];
        let b1 = self.src[pos + 1];
        let b2 = self.src[pos + 2];
        let b3 = self.src[pos + 3];
        let mut l = 4u32;
        while l >= 2 {
            if pos + l as usize <= self.n {
                let k = self.tables.op.opmap_lookup(b0, b1, b2, b3, l);
                if k != 0 && !(k == tk!(OptionalChain) as u32 && is_digit(b2)) {
                    return (l as usize, k);
                }
            }
            l -= 1;
        }
        (1, 0)
    }

    #[inline]
    pub(crate) fn line_break_between(&self, a: usize, b: usize) -> bool {
        walk::lt_in_range(self.src, a, b)
    }

    #[inline]
    pub(crate) fn ident_is(&self, pos: usize, kw: &[u8]) -> bool {
        walk::ident_is(self.src, pos, kw)
    }

    #[inline]
    pub(crate) fn match_delim_back(&self, from: usize, open: u8, close: u8) -> Option<usize> {
        walk::match_delim_back(self, from, open, close)
    }
}
