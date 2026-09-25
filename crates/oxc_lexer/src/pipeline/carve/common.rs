use oxc_ast::CommentKind;

use crate::{comment_meta, error::DiagCode, lanes::Lanes, token::tk};

use crate::pipeline::{
    bitmap::{bm_clear, bm_clear_range, bm_get, bm_next0, bm_set},
    bytes::hex_val,
    disambiguate::not_operator_position,
    find::find_line_terminator,
    scan::{scan_block_comment, scan_line_comment, scan_quoted, scan_regex, scan_tmpl_text},
    tables::Tables,
    token_view,
};

/// Lex the string literal opening at `s`. Returns the resume index. Shared
/// by `carve` and `carve_jsx` JS mode.
#[inline(always)]
pub(super) unsafe fn lex_string(
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
        lanes.push_diag(s as u32, (n - s) as u32, DiagCode::UnterminatedString);
    }
    *kind.add(s) = tk!(String);
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
pub(super) unsafe fn lex_template_segment(
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
        lanes.push_diag(s as u32, (end - s) as u32, DiagCode::UnterminatedTemplate);
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
pub(super) unsafe fn lex_slash(
    t: &Tables,
    src: *const u8,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    opch: *mut u64,
    word: *const u64,
    ts: bool,
    s: usize,
    lanes: &mut Lanes,
) -> usize {
    let d = if s + 1 < n { *src.add(s + 1) } else { 0 };
    if d == b'/' {
        lex_line_comment(src, srcs, n, st, kind, s, lanes)
    } else if d == b'*' {
        lex_block_comment(src, srcs, n, st, kind, s, lanes)
    } else if not_operator_position(
        &token_view(
            t,
            src,
            st,
            opch,
            word,
            kind,
            n,
            ts,
            0,
            lanes.module,
            &lanes.disambiguate.brackets,
            &lanes.disambiguate.closers,
        ),
        &mut lanes.disambiguate.walks,
        s,
    ) {
        lex_regex(src, srcs, n, st, kind, word, s, lanes)
    } else if s + 1 < n && *src.add(s + 1) == b'=' {
        // `/=`: absorb the `=`.
        *kind.add(s) = tk!(SlashEq);
        bm_clear(st, s + 1);
        bm_clear(opch, s + 1);
        s + 2
    } else {
        s + 1
    }
}

/// Lex the `//` line comment at `s`. Returns the resume index.
#[inline(always)]
pub(super) unsafe fn lex_line_comment(
    src: *const u8,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    s: usize,
    lanes: &mut Lanes,
) -> usize {
    let (end, lic_q) = scan_line_comment(src, n, s + 2);
    *kind.add(s) = tk!(LineComment);
    if end > s + 1 {
        bm_clear_range(st, s + 1, end - 1);
    }
    if end < n {
        bm_set(st, end);
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
pub(super) unsafe fn lex_block_comment(
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
        lanes.push_diag(s as u32, (n - s) as u32, DiagCode::UnterminatedBlockComment);
    }
    *kind.add(s) = tk!(BlockComment);
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
        lanes.push_diag(s as u32, (nl_at + 1 - s) as u32, DiagCode::LineTerminatorInRegexp);
    } else if e >= n {
        lanes.push_diag(s as u32, (n - s) as u32, DiagCode::UnterminatedRegexp);
    }
    let mut end = fs;
    if end < n && bm_get(word, end) {
        end = bm_next0(word, end, n);
    }
    *kind.add(s) = tk!(RegExp);
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
pub(super) unsafe fn skip_unicode_brace_escape(src: *const u8, n: usize, s: usize) -> usize {
    let mut j = s + 1;
    while j < n && hex_val(*src.add(j)) != 255 {
        j += 1;
    }
    if j < n && *src.add(j) == b'}' {
        j += 1;
    }
    j
}

/// Annex B B.1.1: does the `<` at `s` open a `<!--` comment? Always in a script; in a module
/// only at line start, where it is diagnosed rather than read as operators. Shared by `carve`
/// and `carve_jsx` JS mode: the goal, not the JSX setting, decides.
#[inline(always)]
pub(super) fn html_open_comment_at(srcs: &[u8], n: usize, s: usize, module: bool) -> bool {
    s + 3 < n
        && srcs[s + 1] == b'!'
        && srcs[s + 2] == b'-'
        && srcs[s + 3] == b'-'
        && (!module || html_close_at_line_start(srcs, s))
}

/// Lex the `<!--` comment opening at `s` (see [`html_open_comment_at`]). Returns the resume
/// index. Cold: a literal `<!--`.
#[cold]
pub(super) unsafe fn lex_html_open_comment(
    src: *const u8,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    opch: *mut u64,
    s: usize,
    lanes: &mut Lanes,
) -> usize {
    if lanes.module {
        lanes.push_diag(s as u32, 4, DiagCode::HtmlCommentInModule);
    }
    let end = find_line_terminator(src, n, s + 4);
    *kind.add(s) = tk!(LineComment);
    if end > s + 1 {
        bm_clear_range(st, s + 1, end - 1);
    }
    if end < n {
        bm_set(st, end);
    }
    // `<`, `!`, `-` are opchars: clear the span from `opch` or `coalesce`
    // would re-tokenize `<!--` as operators.
    bm_clear_range(opch, s, end - 1);
    // meta_byte_exact skips a 2-byte delimiter; pass s + 2 so the 4-byte
    // `<!--` is skipped. The record keeps (s, end).
    let m = comment_meta::meta_byte_exact(&srcs[..n], (s + 2) as u32, end as u32, false);
    lanes.comment_meta.push(m);
    lanes.push_comment_record(srcs, n, s as u32, end as u32, false, m);
    lanes.comments.last_mut().unwrap().kind = CommentKind::HtmlOpen;
    end
}

/// Annex B B.1.3: does the `>` at `s` end a `-->` that opens a comment? Only in a script, and
/// only at line start.
#[inline(always)]
pub(super) fn html_close_comment_at(srcs: &[u8], s: usize, module: bool) -> bool {
    !module
        && s >= 2
        && srcs[s - 1] == b'-'
        && srcs[s - 2] == b'-'
        && html_close_at_line_start(srcs, s - 2)
}

/// Lex the `-->` comment whose `>` is at `s` (see [`html_close_comment_at`]). Returns the
/// resume index. Cold: a literal `-->` at line start.
#[cold]
pub(super) unsafe fn lex_html_close_comment(
    src: *const u8,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    opch: *mut u64,
    s: usize,
    lanes: &mut Lanes,
) -> usize {
    let start = s - 2;
    let end = find_line_terminator(src, n, s + 1);
    *kind.add(start) = tk!(LineComment);
    bm_set(st, start);
    if end > start + 1 {
        bm_clear_range(st, start + 1, end - 1);
    }
    if end < n {
        bm_set(st, end);
    }
    // Clear the span from `opch` (see `<!--` above).
    bm_clear_range(opch, start, end - 1);
    // `-->` is a 3-byte delimiter; pass start + 1 so the 2-byte-delimiter
    // body resolves to [s + 1, end).
    let m = comment_meta::meta_byte_exact(&srcs[..n], (start + 1) as u32, end as u32, false);
    lanes.comment_meta.push(m);
    lanes.push_comment_record(srcs, n, start as u32, end as u32, false, m);
    lanes.comments.last_mut().unwrap().kind = CommentKind::HtmlClose;
    end
}

/// Annex B B.1.3: a `-->` close-comment counts only at line start - scanning
/// back must reach a LineTerminator (or start of input) crossing nothing but
/// whitespace and block comments; a newline inside a crossed block comment
/// also qualifies. Cold: called only on a literal `-->`.
fn html_close_at_line_start(src: &[u8], mut q: usize) -> bool {
    loop {
        if q == 0 {
            return true; // start of input
        }
        let c = src[q - 1];
        match c {
            b' ' | b'\t' | 0x0b | 0x0c => q -= 1,
            b'\n' | b'\r' => return true,
            // LS/PS ending at q-1.
            0xA8 | 0xA9 => {
                return q >= 3 && src[q - 2] == 0x80 && src[q - 3] == 0xE2;
            }
            // `*/` at (q-2, q-1): skip back to its `/*`; a newline inside the
            // comment body satisfies the rule.
            b'/' if q >= 2 && src[q - 2] == b'*' => {
                let mut m = q - 2;
                let mut saw_nl = false;
                loop {
                    if m < 2 {
                        return saw_nl; // unbalanced `*/`
                    }
                    if src[m - 2] == b'/' && src[m - 1] == b'*' {
                        q = m - 2;
                        break;
                    }
                    let b = src[m - 1];
                    if b == b'\n' || b == b'\r' {
                        saw_nl = true;
                    }
                    m -= 1;
                }
                if saw_nl {
                    return true;
                }
                // single-line block comment skipped; keep scanning
            }
            _ => return false,
        }
    }
}
