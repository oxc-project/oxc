//! Moving backwards through the token stream.
//!
//! These are the primitives which the scans in `disambiguate` are built from.
//! They move around the token stream without deciding what any construct means.
//!
//! Stepping and inspecting:
//! - [`prev_sig`] steps to the previous token, skipping whitespace and comments.
//! - [`kind_at`] reads a token's kind, treating keywords as plain identifiers,
//!   and escaped identifiers as unescaped ones.
//! - [`ident_is`] and [`word_is_any`] check whether an identifier is a particular word,
//!   like `let`, or any word in a list.
//!
//! Jumping over bracketed groups:
//! - [`match_delim_back`] goes from a `)`, `]` or `}` back to its opener over a bitmap of
//!   the source's bracket bytes ([`Brackets`]), built per lex a word at a time as the matches
//!   reach it. Past [`BRACKET_STEP_CAP`] bracket steps it switches to a table of bracket pairs,
//!   so the answer stays exact.

use std::cell::{Cell, RefCell};

use super::{Tokens, bits};
use crate::{
    pipeline::{
        bytes::{is_word, unicode_ws_len_at},
        disambiguate::BRACKET_STEP_CAP,
        find::bracket_bits,
    },
    token::{KW_KIND_BASE, KW_KIND_MAX, OP_KIND_BASE, is_trivia_byte, matches_tk, tk},
};

/// The source's bracket bytes (`(){}[]`) as a bitmap, built per lex a 64-byte word at a time
/// as the matches reach it: crossing a group costs a step per bracket, not per token, and a file
/// whose queries stay local never has the whole bitmap built. Owned by the lexer and shared with
/// every [`Tokens`] view of the lex, so the words are cells.
#[derive(Default)]
pub(crate) struct Brackets {
    /// Bracket bytes of the source. `st` is applied when a word is read, not stored, because
    /// `carve` is still clearing literal interiors while the matches run.
    bits: Vec<Cell<u64>>,
    /// One bit per word of `bits`: built this lex.
    built: Vec<Cell<u64>>,
    /// The closer-to-opener table a match past [`BRACKET_STEP_CAP`] falls back on.
    pairs: RefCell<Pairs>,
}

/// Bracket pairs of the source up to `built_to`, built once, closers in source order.
#[derive(Default)]
struct Pairs {
    built_to: usize,
    pairs: Vec<(u32, u32)>,
    open: [Vec<u32>; 3],
}

impl Brackets {
    /// A new lex over `n` bytes: size the buffers and forget every word of the previous one.
    pub(crate) fn begin(&mut self, n: usize) {
        let nwords = n.div_ceil(64) + 1;
        self.bits.resize(nwords, Cell::new(0));
        self.built.clear();
        self.built.resize(nwords.div_ceil(64), Cell::new(0));
        let pairs = self.pairs.get_mut();
        pairs.built_to = 0;
        pairs.pairs.clear();
        for stack in &mut pairs.open {
            stack.clear();
        }
    }

    /// The bracket bits of word `w`, built on first use.
    #[inline]
    fn word(&self, src: &[u8], n: usize, w: usize) -> u64 {
        let (i, b) = (w >> 6, 1u64 << (w & 63));
        let built = &self.built[i];
        if built.get() & b == 0 {
            self.bits[w].set(bracket_word(src, w << 6, n));
            built.set(built.get() | b);
        }
        self.bits[w].get()
    }
}

/// Bits of the 64 bytes at `base` that are brackets (bytes at or past `n` are clear). The source
/// carries `PAD` bytes past `n`, so a whole word is readable whenever `base < n`.
fn bracket_word(src: &[u8], base: usize, n: usize) -> u64 {
    if base >= n {
        return 0;
    }
    let mut out = bracket_bits(src, base);
    if base + 64 > n {
        out &= (1u64 << (n - base)) - 1;
    }
    out
}

/// Match the close punctuator at `from` back to its opener over the bracket bitmap, counting
/// only punctuator delimiters - template-closing `}`s and cleared literal interiors are
/// invisible. Past [`BRACKET_STEP_CAP`] bracket steps the answer comes from a per-lex
/// closer-to-opener table built once, so it stays exact; None if unbalanced.
#[inline]
pub(crate) fn match_delim_back(tokens: &Tokens, from: usize, open: u8, close: u8) -> Option<usize> {
    let (src, st, kind, n, b) = (tokens.src, tokens.st, tokens.kind, tokens.n, tokens.brackets);
    let mut depth: i32 = 1;
    let mut steps: u32 = 0;
    let mut w = from >> 6;
    let mut bits = b.word(src, n, w) & st[w] & ((1u64 << (from & 63)) - 1);
    loop {
        while bits != 0 {
            let i = 63 - bits.leading_zeros() as usize;
            bits &= !(1u64 << i);
            let pos = (w << 6) | i;
            if kind[pos] >= OP_KIND_BASE {
                let c = src[pos];
                if c == close {
                    depth += 1;
                } else if c == open {
                    depth -= 1;
                    if depth == 0 {
                        return Some(pos);
                    }
                }
            }
            steps += 1;
            if steps > BRACKET_STEP_CAP {
                return delim_memo_opener(tokens, from);
            }
        }
        if w == 0 {
            return None;
        }
        w -= 1;
        bits = b.word(src, n, w) & st[w];
    }
}

#[inline(never)]
fn delim_memo_opener(tokens: &Tokens, from: usize) -> Option<usize> {
    let (src, st, kind, n, b) = (tokens.src, tokens.st, tokens.kind, tokens.n, tokens.brackets);
    let mut m = b.pairs.borrow_mut();
    if from >= m.built_to {
        let first = m.built_to >> 6;
        let last = from >> 6;
        let mut w = first;
        while w <= last {
            let mut bits = b.word(src, n, w) & st[w];
            if w == first {
                bits &= !((1u64 << (m.built_to & 63)) - 1);
            }
            if w == last {
                bits &= u64::MAX >> (63 - (from & 63));
            }
            while bits != 0 {
                let p = (w << 6) | bits.trailing_zeros() as usize;
                bits &= bits - 1;
                if kind[p] >= OP_KIND_BASE {
                    let c = src[p];
                    match c {
                        b'(' | b'[' | b'{' => m.open[delim_slot(c)].push(p as u32),
                        b')' | b']' | b'}' => {
                            if let Some(o) = m.open[delim_slot(c)].pop() {
                                m.pairs.push((p as u32, o));
                            }
                        }
                        _ => {}
                    }
                }
            }
            w += 1;
        }
        m.built_to = from + 1;
    }
    match m.pairs.binary_search_by_key(&(from as u32), |pr| pr.0) {
        Ok(i) => Some(m.pairs[i].1 as usize),
        Err(_) => None,
    }
}

fn delim_slot(c: u8) -> usize {
    match c {
        b'(' | b')' => 0,
        b'[' | b']' => 1,
        _ => 2,
    }
}

pub(crate) fn word_is_any(src: &[u8], w: usize, words: &[&[u8]]) -> bool {
    let len = word_len(src, w);
    let first = src[w];
    words.iter().any(|kw| kw.len() == len && kw[0] == first && ident_is(src, w, kw))
}

#[inline(always)]
pub(crate) fn word_len(src: &[u8], w: usize) -> usize {
    let mut e = w + 1;
    while is_word(src[e]) {
        e += 1;
    }
    e - w
}

/// Does the identifier at `pos` equal exactly `kw`? The following-byte check
/// rejects longer identifiers (the source pad makes it safe at EOF).
#[inline]
pub(crate) fn ident_is(src: &[u8], pos: usize, kw: &[u8]) -> bool {
    src[pos..].starts_with(kw) && {
        let after = pos + kw.len();
        !is_word(src[after]) || unicode_ws_len_at(src, after) != 0
    }
}

/// Previous significant token start before `pos` (skipping trivia), or `None` at start of input.
#[inline]
pub(crate) fn prev_sig(st: &[u64], kind: &[u8], pos: usize) -> Option<usize> {
    let mut q = bits::prev1(st, pos);
    while let Some(p) = q {
        if !is_trivia_byte(kind[p]) {
            break;
        }
        q = bits::prev1(st, p);
    }
    q
}

#[inline(always)]
pub(crate) fn kind_at(kind: &[u8], w: usize) -> u8 {
    let k = kind[w];
    if k >= KW_KIND_BASE && k <= KW_KIND_MAX {
        return tk!(Ident);
    }
    if matches_tk!(k, IdentEscaped | PrivateIdentEscaped) {
        return k & !(tk!(IdentEscaped) ^ tk!(Ident));
    }
    k
}
