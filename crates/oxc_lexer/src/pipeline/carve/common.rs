use oxc_simd::{Cpu, cpu_multiversion};

use crate::{error::DiagCode, token::tk};

use crate::pipeline::{
    bitmap::{Bitmap, Bitmaps},
    bytes::{hex_digit, is_ls_ps_tail},
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
    if saw_nl {
        // The terminator wins over unterminated-at-EOF, same as oxc_parser.
        let end = if e < src.len() { e + 1 } else { src.len() };
        lanes.push_line_terminator_in_string(src.padded(), s, end);
    } else if e >= src.len() {
        lanes.push_diag(s as u32, (src.len() - s) as u32, DiagCode::UnterminatedString);
    }
    let (end, be) = mark_string(src, token_start, kinds, s, c, e, cpu);
    lanes.push_string(src.padded(), s + 1, be, cpu);
    end
}

/// Lex the JSX attribute string at s, which has no escapes and ends at the next matching quote.
#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn lex_jsx_attribute_string(
    src: Source,
    token_start: Bitmap,
    kinds: Kinds,
    s: usize,
    c: u8,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) -> usize {
    let e = find(src, s + 1, [c], cpu);
    if e >= src.len() {
        lanes.push_diag(s as u32, (src.len() - s) as u32, DiagCode::UnterminatedString);
    }
    let (end, be) = mark_string(src, token_start, kinds, s, c, e, cpu);
    lanes.push_string_raw(src.padded(), s + 1, be, cpu);
    end
}

/// Mark the string at s, closed by the quote at e unless e is the end; returns end and body end.
#[cpu_multiversion]
#[inline(always)]
unsafe fn mark_string(
    src: Source,
    token_start: Bitmap,
    kinds: Kinds,
    s: usize,
    c: u8,
    e: usize,
    cpu: impl Cpu,
) -> (usize, usize) {
    let end = if e < src.len() { e + 1 } else { src.len() };
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
    (end, be)
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

/// The template substitutions open at the carve position, innermost last, with their open braces.
#[derive(Default)]
pub(in crate::pipeline) struct Substitutions(Vec<u32>);

impl Substitutions {
    pub(super) fn clear(&mut self) {
        self.0.clear();
    }

    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Lex the template at the backtick at s, opening a substitution if it ends in a dollar brace.
    #[cpu_multiversion]
    #[inline(always)]
    pub(super) unsafe fn lex_template(
        &mut self,
        src: Source,
        token_start: Bitmap,
        kinds: Kinds,
        s: usize,
        lanes: &mut Lanes,
        cpu: impl Cpu,
    ) -> usize {
        let (head, flat) = (tk!(TemplateHead), tk!(TemplateNoSub));
        let (end, opened) =
            lex_template_segment(src, token_start, kinds, s, head, flat, lanes, cpu);
        if opened {
            self.0.push(0);
        }
        end
    }

    pub(super) fn open_brace(&mut self) {
        let top = self.0.len() - 1;
        self.0[top] += 1;
    }

    /// Close a nested brace, or else the substitution and lex the template text after it.
    #[cpu_multiversion]
    #[inline(always)]
    pub(super) unsafe fn close_brace(
        &mut self,
        src: Source,
        token_start: Bitmap,
        kinds: Kinds,
        s: usize,
        lanes: &mut Lanes,
        cpu: impl Cpu,
    ) -> usize {
        let top = self.0.len() - 1;
        if self.0[top] > 0 {
            self.0[top] -= 1;
            return s + 1;
        }
        self.0.pop();
        let (middle, tail) = (tk!(TemplateMiddle), tk!(TemplateTail));
        let (end, opened) =
            lex_template_segment(src, token_start, kinds, s, middle, tail, lanes, cpu);
        if opened {
            self.0.push(0);
        }
        end
    }
}

/// Lex a hashbang line at the start of the source; returns where carving resumes, 0 without one.
#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn lex_hashbang(
    src: Source,
    token_start: Bitmap,
    kinds: Kinds,
    cpu: impl Cpu,
) -> usize {
    if !(src.len() >= 2 && src[0] == b'#' && src[1] == b'!') {
        return 0;
    }
    let end = find_line_terminator(src, 2, cpu);
    kinds.set(0, tk!(Hashbang));
    token_start.clear_range(1, end - 1, cpu);
    if end < src.len() {
        token_start.set(end, cpu);
    }
    end
}

/// Whether the brace at s opens the code point of a backslash-u escape.
#[cpu_multiversion]
#[inline(always)]
pub(super) fn is_unicode_escape_brace(src: Source, s: usize, _cpu: impl Cpu) -> bool {
    s >= 2 && src[s - 1] == b'u' && src[s - 2] == b'\\'
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
    while j < src.len() && hex_digit(src[j], cpu).is_some() {
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
        if src[p] != 0xE2 || is_ls_ps_tail(src[p + 1], src[p + 2], cpu) {
            return p;
        }
        i = p + 1;
    }
}
