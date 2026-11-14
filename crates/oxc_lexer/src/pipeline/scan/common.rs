use std::slice;

use oxc_simd::{Cpu, cpu_multiversion};

use crate::pipeline::{
    bytes::{hex_digit, is_digit, is_ls_ps_tail, is_word},
    find::find,
    source::Source,
};

/// Regex-body scan. LF, CR and the 0xE2 lead are watched so line terminators in the body can be diagnosed.
///
/// Ordered for speed, pairing `\` with `]` (see `any_of` in `find/avx2.rs`).
/// `[` is kept out of the first half, away from `\` and `]`, otherwise LLVM turns `[`, `\` and `]` (0x5B-0x5D)
/// into a range check, which is 1 op shorter but makes the chain to the exit branch 1 op longer.
const REGEX_BYTES: [u8; 7] = [b'\\', b']', b'\n', b'\r', b'/', b'[', 0xE2];

#[cpu_multiversion]
#[inline]
pub fn scan_quoted(src: Source, mut i: usize, q: u8, saw_nl: &mut bool, cpu: impl Cpu) -> usize {
    loop {
        // LF/CR are watched too: raw line terminators in strings are diagnosed.
        let p = find(src, i, [q, b'\\', b'\n', b'\r'], cpu);
        if p >= src.len() {
            return src.len();
        }
        let ch = src[p];
        if ch == b'\\' {
            // Escape, including line continuations. `\<CR><LF>` is one
            // LineTerminatorSequence: skip all three bytes, or the next find
            // lands on the LF and flags a legal continuation.
            let crlf = src[p + 1] == b'\r' && src[p + 2] == b'\n';
            i = p + 2 + usize::from(crlf);
            continue;
        }
        if ch == b'\n' || ch == b'\r' {
            *saw_nl = true;
            i = p + 1; // keep scanning so the token end is unchanged
            continue;
        }
        return p;
    }
}

#[cpu_multiversion]
#[inline]
pub fn scan_regex(src: Source, mut i: usize, cpu: impl Cpu) -> (usize, Option<usize>) {
    let mut incl = false;
    let mut nl_at = None;
    loop {
        let p = find(src, i, REGEX_BYTES, cpu);
        if p >= src.len() {
            return (src.len(), nl_at);
        }
        let c = src[p];
        if c == b'\\' {
            // An escaped line terminator is still invalid in a regex (the
            // grammar requires a NonTerminator); record it here or the p+2
            // skip would sail past it. The p + 1 < src.len() guard keeps garbage pad
            // bytes after a `\` at EOF out of the diagnostics.
            if p + 1 < src.len() {
                let c1 = src[p + 1];
                if c1 == b'\n' || c1 == b'\r' {
                    nl_at.get_or_insert(p + 1);
                } else if c1 == 0xE2
                    && p + 3 < src.len()
                    && is_ls_ps_tail(src[p + 2], src[p + 3], cpu)
                {
                    // Escaped LS/PS: span ends past the whole char; the p+2
                    // skip lands mid-sequence (unwatched 0x80) and sails on.
                    nl_at.get_or_insert(p + 3);
                }
            }
            i = p + 2;
            continue;
        }
        if c == b'\n' || c == b'\r' {
            // Raw line terminator: invalid anywhere in the body, `[...]`
            // included. Record the first; keep scanning so the token end
            // (and every downstream span) is unchanged.
            nl_at.get_or_insert(p);
            i = p + 1;
            continue;
        }
        if c == 0xE2 {
            // LS/PS are line terminators too; other E2-led chars (em dash,
            // arrows) fall through untouched.
            if p + 2 < src.len() && is_ls_ps_tail(src[p + 1], src[p + 2], cpu) {
                nl_at.get_or_insert(p + 2); // span ends past the whole char
            }
            i = p + 1;
            continue;
        }
        if incl {
            if c == b']' {
                incl = false;
            }
            i = p + 1;
            continue;
        }
        if c == b'[' {
            incl = true;
            i = p + 1;
            continue;
        }
        if c == b'/' {
            return (p, nl_at);
        }
        i = p + 1;
    }
}

/// What ends a template text: the end of the source, a backtick, or a substitution.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TemplateEnd {
    Unterminated,
    Backtick,
    Substitution,
}

impl TemplateEnd {
    /// Bytes of the closing delimiter, which the text excludes.
    pub fn delimiter_len(self) -> usize {
        match self {
            Self::Unterminated => 0,
            Self::Backtick => 1,
            Self::Substitution => 2,
        }
    }
}

#[cpu_multiversion]
#[inline]
pub fn scan_tmpl_text(src: Source, mut i: usize, cpu: impl Cpu) -> (usize, TemplateEnd) {
    loop {
        let p = find(src, i, [b'`', b'\\', b'$'], cpu);
        if p >= src.len() {
            return (src.len(), TemplateEnd::Unterminated);
        }
        let c = src[p];
        if c == b'\\' {
            i = p + 2;
            continue;
        }
        if c == b'`' {
            return (p + 1, TemplateEnd::Backtick);
        }
        if p + 1 < src.len() && src[p + 1] == b'{' {
            return (p + 2, TemplateEnd::Substitution);
        }
        i = p + 1;
    }
}

#[cpu_multiversion]
#[inline]
pub fn scan_ident_esc(src: Source, p: usize, cpu: impl Cpu) -> usize {
    let mut i = p;
    while i < src.len() {
        if src[i] == b'\\' && i + 1 < src.len() && src[i + 1] == b'u' {
            i += 2;
            if i < src.len() && src[i] == b'{' {
                i += 1;
                while i < src.len() && hex_digit(src[i], cpu).is_some() {
                    i += 1;
                }
                if i < src.len() && src[i] == b'}' {
                    i += 1;
                }
            } else {
                let mut k = 0;
                while k < 4 && i < src.len() && hex_digit(src[i], cpu).is_some() {
                    i += 1;
                    k += 1;
                }
            }
            while i < src.len() && is_word(src[i], cpu) && src.unicode_ws_len_at(i, cpu) == 0 {
                i += 1;
            }
            continue;
        }
        break;
    }
    i
}

#[cpu_multiversion]
#[inline(never)]
pub fn scan_number(src: Source, pos: usize, cpu: impl Cpu) -> usize {
    if src[pos] == b'0' && pos + 1 < src.len() {
        let c1 = src[pos + 1];
        let c = c1 | 0x20;
        if c == b'x' || c == b'o' || c == b'b' {
            let radix = if c == b'x' {
                16
            } else if c == b'o' {
                8
            } else {
                2
            };
            let mut i = pos + 2;
            while i < src.len() {
                let d = src[i];
                if d == b'_' {
                    i += 1;
                    continue;
                }
                if hex_digit(d, cpu).is_some_and(|v| v < radix) {
                    i += 1;
                } else {
                    break;
                }
            }
            if i < src.len() && src[i] == b'n' {
                return i + 1;
            }
            return i;
        }
        if is_digit(c1, cpu) {
            let mut i = pos + 1;
            let mut octal = true;
            while i < src.len() {
                let d = src[i];
                if is_digit(d, cpu) {
                    octal &= d < b'8';
                } else if d != b'_' {
                    break;
                }
                i += 1;
            }
            if octal {
                return i;
            }
            return scan_fraction_exponent(src, i, cpu).0;
        }
    }
    let mut i = pos;
    while i < src.len() && (is_digit(src[i], cpu) || src[i] == b'_') {
        i += 1;
    }
    let (i, is_float) = scan_fraction_exponent(src, i, cpu);
    if !is_float && i < src.len() && src[i] == b'n' {
        return i + 1;
    }
    i
}

#[cpu_multiversion]
#[inline]
fn scan_fraction_exponent(src: Source, mut i: usize, cpu: impl Cpu) -> (usize, bool) {
    let mut is_float = false;
    if i < src.len() && src[i] == b'.' {
        is_float = true;
        i += 1;
        while i < src.len() && (is_digit(src[i], cpu) || src[i] == b'_') {
            i += 1;
        }
    }
    if i < src.len() && (src[i] | 0x20) == b'e' {
        is_float = true;
        i += 1;
        if i < src.len() && (src[i] == b'+' || src[i] == b'-') {
            i += 1;
        }
        while i < src.len() && (is_digit(src[i], cpu) || src[i] == b'_') {
            i += 1;
        }
    }
    (i, is_float)
}

#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn is_license_at(src: *const u8, q: usize, _cpu: impl Cpu) -> bool {
    let c1 = *src.add(q + 1);
    (c1 == b'l' && slice::from_raw_parts(src.add(q + 2), 6) == b"icense")
        || (c1 == b'p' && slice::from_raw_parts(src.add(q + 2), 7) == b"reserve")
}

/// First `@license` or `@preserve` among the `@`s in `am`, where bit `i` is for byte `base + i`.
#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn first_license_at(
    src: *const u8,
    base: usize,
    mut am: u32,
    cpu: impl Cpu,
) -> Option<u32> {
    while am != 0 {
        let q = base + am.trailing_zeros() as usize;
        am &= am - 1;
        if is_license_at(src, q, cpu) {
            return Some(q as u32);
        }
    }
    None
}
