use oxc_simd::{Cpu, cpu_multiversion};

use crate::{error::DiagCode, token::tk};

use crate::pipeline::{
    bitmap::{Bitmap, Bitmaps},
    bytes::hex_val,
    disambiguate::DisambiguateState,
    find::find,
    kinds::Kinds,
    lanes::Lanes,
    lex::Lex,
    scan::{scan_quoted, scan_regex, scan_tmpl_text},
    source::Source,
};

use super::comments::{lex_block_comment, lex_line_comment};

/// Ordered for speed, pairing `'` with `/` and `<` with `>` (see `any_of` in `find/avx2.rs`).
pub(super) const OPENER_BYTES: [u8; 6] = [b'"', b'`', b'\'', b'/', b'<', b'>'];

/// Ordered for speed, pairing `'` with `/` and `<` with `>` (see `any_of` in `find/avx2.rs`).
pub(super) const OPENER8_BYTES: [u8; 8] = [b'"', b'`', b'\'', b'/', b'{', b'}', b'<', b'>'];

/// [`OPENER_BYTES`] without the closing angle bracket, which only scripts need.
pub(super) const OPENER_MODULE_BYTES: [u8; 5] = [b'"', b'`', b'\'', b'/', b'<'];

/// [`OPENER8_BYTES`] without the closing angle bracket, which only scripts need.
pub(super) const OPENER8_MODULE_BYTES: [u8; 7] = [b'"', b'`', b'\'', b'/', b'{', b'}', b'<'];

/// Lex the string literal opening at `s`. Returns the resume index. Shared
/// by `carve` and `carve_jsx` JS mode.
#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn lex_string(
    src: Source,
    token_start: Bitmap,
    kinds: Kinds,
    s: usize,
    c: u8,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) -> usize {
    let mut saw_nl = false;
    let e = scan_quoted(src, s + 1, c, &mut saw_nl, cpu);
    let end = if e < src.len() { e + 1 } else { src.len() };
    if saw_nl {
        // The terminator wins over unterminated-at-EOF, same as oxc_parser.
        lanes.push_line_terminator_in_string(src.padded(), s, end);
    } else if e >= src.len() {
        lanes.push_diag(s as u32, (src.len() - s) as u32, DiagCode::UnterminatedString);
    }
    kinds.set(s, tk!(String));
    if end > s + 1 {
        token_start.clear_range(s + 1, end - 1, cpu);
    }
    let be = if e < src.len() {
        e
    } else if src.len() > s + 1 && src[src.len() - 1] == c {
        src.len() - 1
    } else {
        src.len()
    };
    lanes.push_string(src.padded(), s + 1, be, cpu);
    end
}

/// Lex the template text segment starting at `s` (a backtick or a
/// substitution-closing `}`): `head_kind` if it ends in `${`, `flat_kind` if
/// it closes or runs to EOF. Returns `(resume index, substitution opened)`;
/// the caller pushes its own nesting frame.
#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn lex_template_segment(
    src: Source,
    token_start: Bitmap,
    kinds: Kinds,
    s: usize,
    head_kind: u8,
    flat_kind: u8,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) -> (usize, bool) {
    let mut term = 0i32;
    let end = scan_tmpl_text(src, s + 1, &mut term, cpu);
    if term == 0 {
        lanes.push_diag(s as u32, (end - s) as u32, DiagCode::UnterminatedTemplate);
    }
    kinds.set(s, if term == 2 { head_kind } else { flat_kind });
    if end > s + 1 {
        token_start.clear_range(s + 1, end - 1, cpu);
    }
    lanes.push_template(src.padded(), s + 1, end - [0usize, 1, 2][term as usize], cpu);
    (end, term == 2)
}

/// The `/` dispatch shared by `carve` and `carve_jsx` JS mode: line comment,
/// block comment, regex, `/=`, or a bare slash left for `coalesce`.
#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn lex_slash(
    lex: Lex,
    s: usize,
    lanes: &mut Lanes,
    disambiguate: &mut DisambiguateState,
    cpu: impl Cpu,
) -> usize {
    let Lex { src, bitmaps, kinds, .. } = lex;
    let Bitmaps { token_start, word, op_char, .. } = *bitmaps;
    let d = if s + 1 < src.len() { src[s + 1] } else { 0 };
    if d == b'/' {
        lex_line_comment(src, token_start, kinds, s, lanes, cpu)
    } else if d == b'*' {
        lex_block_comment(src, token_start, kinds, s, lanes, cpu)
    } else if disambiguate.not_operator_position(lex, s, cpu) {
        lex_regex(src, token_start, kinds, word, s, lanes, cpu)
    } else if s + 1 < src.len() && src[s + 1] == b'=' {
        // `/=`: absorb the `=`.
        kinds.set(s, tk!(SlashEq));
        token_start.clear(s + 1, cpu);
        op_char.clear(s + 1, cpu);
        s + 2
    } else {
        s + 1
    }
}

/// Lex the regex literal at `s` (the regex-vs-division decision is already
/// made): body, flag run, diagnostics. Returns the resume index.
#[cpu_multiversion]
#[inline(always)]
unsafe fn lex_regex(
    src: Source,
    token_start: Bitmap,
    kinds: Kinds,
    word: Bitmap,
    s: usize,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) -> usize {
    let mut nl_at = usize::MAX;
    let e = scan_regex(src, s + 1, &mut nl_at, cpu);
    let fs = if e < src.len() { e + 1 } else { src.len() };
    if nl_at != usize::MAX {
        // oxc_parser reports a line terminator in the body as "unterminated"
        // with a span ending just past the first one, even when a later `/`
        // closes our token.
        lanes.push_diag(s as u32, (nl_at + 1 - s) as u32, DiagCode::LineTerminatorInRegexp);
    } else if e >= src.len() {
        lanes.push_diag(s as u32, (src.len() - s) as u32, DiagCode::UnterminatedRegexp);
    }
    let mut end = fs;
    if end < src.len() && word.get(end, cpu) {
        end = word.next0(end, src.len(), cpu);
    }
    kinds.set(s, tk!(RegExp));
    if end > s + 1 {
        token_start.clear_range(s + 1, end - 1, cpu);
    }
    lanes.push_regex_flags(src.padded(), fs, end, cpu);
    end
}

/// Skip the payload of a brace-form unicode escape whose `{` is at `s`: the
/// escape was already joined into its identifier by `misc_pre`, and its
/// braces must not count toward substitution nesting.
#[cpu_multiversion]
#[inline(always)]
pub(super) fn skip_unicode_brace_escape(src: Source, s: usize, cpu: impl Cpu) -> usize {
    let mut j = s + 1;
    while j < src.len() && hex_val(src[j], cpu) != 255 {
        j += 1;
    }
    if j < src.len() && src[j] == b'}' {
        j += 1;
    }
    j
}

/// First ECMAScript LineTerminator at/after `i`: LF, CR, or the 3-byte LS/PS
/// (U+2028/U+2029). Stops at LS/PS too, so the hashbang scan cannot run
/// through one. A 0xE2 that isn't LS/PS keeps scanning; the 2-byte confirm
/// past a trailing 0xE2 reads the pad, which can never match 0x80.
#[cpu_multiversion]
#[inline]
pub(super) unsafe fn find_line_terminator(src: Source, mut i: usize, cpu: impl Cpu) -> usize {
    loop {
        let p = find(src, i, [b'\n', b'\r', 0xE2], cpu);
        if p >= src.len() {
            return src.len();
        }
        if src[p] != 0xE2 {
            return p;
        }
        if src[p + 1] == 0x80 && (src[p + 2] == 0xA8 || src[p + 2] == 0xA9) {
            return p;
        }
        i = p + 1;
    }
}
