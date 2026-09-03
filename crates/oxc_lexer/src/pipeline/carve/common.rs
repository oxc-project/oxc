use crate::{
    comment_meta,
    error::diag_code,
    lanes::Lanes,
    opmap::OP_SLASH_EQ,
    tables::{Tables, hex_val},
};

use super::super::{
    BCOM, LCOM, REGEX, STR,
    bitmap::{bm_clear_range, bm_next0, bm_set1},
    find::{scan_block_comment, scan_line_comment, scan_quoted, scan_regex, scan_tmpl_text},
    regex_div::prev_is_regex,
};

/// Lex the string literal opening at `s`. Returns the resume index. Shared
/// by `carve` and `carve_jsx` JS mode.
#[inline(always)]
pub unsafe fn lex_string(
    src: *const u8,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    s: usize,
    c: u8,
    lanes: &mut Lanes,
) -> usize {
    let mut saw_nl = false;
    let e = scan_quoted(src, n, s + 1, c, &mut saw_nl);
    let end = if e < n { e + 1 } else { n };
    if saw_nl {
        // The terminator wins over unterminated-at-EOF, same as oxc_parser.
        lanes.push_line_terminator_in_string(srcs, s, end);
    } else if e >= n {
        lanes.push_diag(s as u32, (n - s) as u32, diag_code::UNTERMINATED_STRING);
    }
    *kind.add(s) = STR;
    if end > s + 1 {
        bm_clear_range(st, s + 1, end - 1);
    }
    let be = if e < n {
        e
    } else if n > s + 1 && *src.add(n - 1) == c {
        n - 1
    } else {
        n
    };
    lanes.push_string(srcs, s + 1, be);
    end
}

/// Lex the template text segment starting at `s` (a backtick or a
/// substitution-closing `}`): `head_kind` if it ends in `${`, `flat_kind` if
/// it closes or runs to EOF. Returns `(resume index, substitution opened)`;
/// the caller pushes its own nesting frame.
#[inline(always)]
pub unsafe fn lex_template_segment(
    src: *const u8,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    s: usize,
    head_kind: u8,
    flat_kind: u8,
    lanes: &mut Lanes,
) -> (usize, bool) {
    let mut term = 0i32;
    let end = scan_tmpl_text(src, n, s + 1, &mut term);
    if term == 0 {
        lanes.push_diag(s as u32, (end - s) as u32, diag_code::UNTERMINATED_TEMPLATE);
    }
    *kind.add(s) = if term == 2 { head_kind } else { flat_kind };
    if end > s + 1 {
        bm_clear_range(st, s + 1, end - 1);
    }
    lanes.push_template(srcs, s + 1, end - [0usize, 1, 2][term as usize]);
    (end, term == 2)
}

/// The `/` dispatch shared by `carve` and `carve_jsx` JS mode: line comment,
/// block comment, regex, `/=`, or a bare slash left for `coalesce`.
#[inline(always)]
pub unsafe fn lex_slash(
    t: &Tables,
    src: *const u8,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    opch: *mut u64,
    word: *const u64,
    digit: *const u64,
    ts: bool,
    s: usize,
    lanes: &mut Lanes,
) -> usize {
    let d = if s + 1 < n { *src.add(s + 1) } else { 0 };
    if d == b'/' {
        lex_line_comment(src, srcs, n, st, kind, s, lanes)
    } else if d == b'*' {
        lex_block_comment(src, srcs, n, st, kind, s, lanes)
    } else if prev_is_regex(t, src, st, kind, word, digit, n, s, ts, lanes.module) {
        lex_regex(src, srcs, n, st, kind, word, s, lanes)
    } else if s + 1 < n && *src.add(s + 1) == b'=' {
        // `/=`: absorb the `=`.
        *kind.add(s) = OP_SLASH_EQ;
        *st.add((s + 1) >> 6) &= !(1u64 << ((s + 1) & 63));
        *opch.add((s + 1) >> 6) &= !(1u64 << ((s + 1) & 63));
        s + 2
    } else {
        s + 1
    }
}

/// Lex the `//` line comment at `s`. Returns the resume index.
#[inline(always)]
pub unsafe fn lex_line_comment(
    src: *const u8,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    s: usize,
    lanes: &mut Lanes,
) -> usize {
    let (end, lic_q) = scan_line_comment(src, n, s + 2);
    *kind.add(s) = LCOM;
    if end > s + 1 {
        bm_clear_range(st, s + 1, end - 1);
    }
    if end < n {
        bm_set1(st, end);
    }
    let lic = lic_q >= 0 && (lic_q as usize) + 8 < end;
    let m = comment_meta::meta_byte_flags(&srcs[..n], s as u32, end as u32, false, false, lic);
    debug_assert_eq!(
        m,
        comment_meta::meta_byte_exact(&srcs[..n], s as u32, end as u32, false),
        "LCOM meta fused != exact at {s}"
    );
    lanes.comment_meta.push(m);
    lanes.push_comment_record(srcs, n, s as u32, end as u32, false, m);
    end
}

/// Lex the `/*` block comment at `s` (unterminated at EOF is diagnosed).
/// Returns the resume index.
#[inline(always)]
pub unsafe fn lex_block_comment(
    src: *const u8,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    s: usize,
    lanes: &mut Lanes,
) -> usize {
    let (e, saw_nl, lic_q) = scan_block_comment(src, n, s + 2);
    let end = if e < n { e + 1 } else { n };
    if e >= n {
        lanes.push_diag(s as u32, (n - s) as u32, diag_code::UNTERMINATED_BLOCK_COMMENT);
    }
    *kind.add(s) = BCOM;
    if end > s + 1 {
        bm_clear_range(st, s + 1, end - 1);
    }
    let m = if e < n {
        let lic = lic_q >= 0 && (lic_q as usize) + 8 < e - 1;
        comment_meta::meta_byte_flags(&srcs[..n], s as u32, end as u32, true, saw_nl, lic)
    } else {
        comment_meta::meta_byte_exact(&srcs[..n], s as u32, end as u32, true)
    };
    debug_assert_eq!(
        m,
        comment_meta::meta_byte_exact(&srcs[..n], s as u32, end as u32, true),
        "BCOM meta fused != exact at {s}"
    );
    lanes.comment_meta.push(m);
    lanes.push_comment_record(srcs, n, s as u32, end as u32, true, m);
    end
}

/// Lex the regex literal at `s` (the regex-vs-division decision is already
/// made): body, flag run, diagnostics. Returns the resume index.
#[inline(always)]
unsafe fn lex_regex(
    src: *const u8,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    word: *const u64,
    s: usize,
    lanes: &mut Lanes,
) -> usize {
    let mut nl_at = usize::MAX;
    let e = scan_regex(src, n, s + 1, &mut nl_at);
    let fs = if e < n { e + 1 } else { n };
    if nl_at != usize::MAX {
        // oxc_parser reports a line terminator in the body as "unterminated"
        // with a span ending just past the first one, even when a later `/`
        // closes our token.
        lanes.push_diag(s as u32, (nl_at + 1 - s) as u32, diag_code::LINE_TERMINATOR_IN_REGEXP);
    } else if e >= n {
        lanes.push_diag(s as u32, (n - s) as u32, diag_code::UNTERMINATED_REGEXP);
    }
    let mut end = fs;
    if end < n && (*word.add(end >> 6) >> (end & 63)) & 1 != 0 {
        end = bm_next0(word, end, n);
    }
    *kind.add(s) = REGEX;
    if end > s + 1 {
        bm_clear_range(st, s + 1, end - 1);
    }
    lanes.push_regex_flags(srcs, fs, end);
    end
}

/// Skip the payload of a brace-form unicode escape whose `{` is at `s`: the
/// escape was already joined into its identifier by `misc_pre`, and its
/// braces must not count toward substitution nesting.
#[inline(always)]
pub unsafe fn skip_unicode_brace_escape(src: *const u8, n: usize, s: usize) -> usize {
    let mut j = s + 1;
    while j < n && hex_val(*src.add(j)) != 255 {
        j += 1;
    }
    if j < n && *src.add(j) == b'}' {
        j += 1;
    }
    j
}
