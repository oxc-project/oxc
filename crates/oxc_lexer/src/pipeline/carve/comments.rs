use oxc_ast::{Comment, CommentContent, CommentKind};

use crate::{error::DiagCode, token::tk};

use crate::pipeline::{
    bitmap::{bm_clear_range, bm_set},
    lanes::Lanes,
    scan::{scan_block_comment, scan_line_comment},
};

use super::common::find_line_terminator;

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
    let body = comment_body(&srcs[..n], s as u32, end as u32, false);
    let m = classify(body, false, lic);
    debug_assert_eq!(m, classify_exact(body, false), "LCOM content fused != exact at {s}");
    push_comment_record(srcs, n, s as u32, end as u32, CommentKind::Line, true, m, lanes);
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
    let (e, mut saw_nl, lic_q) = scan_block_comment(src, n, s + 2);
    let end = if e < n { e + 1 } else { n };
    if e >= n {
        lanes.push_diag(s as u32, (n - s) as u32, DiagCode::UnterminatedBlockComment);
    }
    *kind.add(s) = tk!(BlockComment);
    if end > s + 1 {
        bm_clear_range(st, s + 1, end - 1);
    }

    let body = comment_body(&srcs[..n], s as u32, end as u32, true);
    let m = if e < n {
        let lic = lic_q >= 0 && (lic_q as usize) + 8 < e - 1;
        let m = classify(body, true, lic);
        debug_assert_eq!(m, classify_exact(body, true), "BCOM content fused != exact at {s}");
        debug_assert_eq!(saw_nl, has_nl_cr(body), "BCOM saw_nl != exact at {s}");
        m
    } else {
        saw_nl = has_nl_cr(body);
        classify_exact(body, true)
    };

    let comment_kind =
        if saw_nl { CommentKind::MultiLineBlock } else { CommentKind::SingleLineBlock };

    let mut q = end;
    let mut followed_by_newline = false;
    while q < n && is_lex_ws(srcs[q]) {
        if srcs[q] == b'\n' || srcs[q] == b'\r' {
            followed_by_newline = true;
            break;
        }
        q += 1;
    }

    push_comment_record(srcs, n, s as u32, end as u32, comment_kind, followed_by_newline, m, lanes);
    end
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
    // `comment_body` skips a 2-byte delimiter; pass s + 2 so the 4-byte
    // `<!--` is skipped. The record keeps (s, end).
    let body = comment_body(&srcs[..n], (s + 2) as u32, end as u32, false);
    let m = classify_exact(body, false);
    push_comment_record(srcs, n, s as u32, end as u32, CommentKind::HtmlOpen, true, m, lanes);
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
    let body = comment_body(&srcs[..n], (start + 1) as u32, end as u32, false);
    let m = classify_exact(body, false);
    push_comment_record(srcs, n, start as u32, end as u32, CommentKind::HtmlClose, true, m, lanes);
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

#[inline]
fn classify_exact(body: &[u8], is_block: bool) -> CommentContent {
    let lic = contains_license_or_preserve(body);
    classify(body, is_block, lic)
}

#[inline]
fn comment_body(src: &[u8], start: u32, end: u32, is_block: bool) -> &[u8] {
    let (lo, hi) = if is_block {
        (start as usize + 2, (end as usize).saturating_sub(2))
    } else {
        (start as usize + 2, end as usize)
    };
    let lo = lo.min(src.len());
    let hi = hi.clamp(lo, src.len());
    &src[lo..hi]
}

fn contains_license_or_preserve(hay: &[u8]) -> bool {
    if hay.len() < 9 {
        return false;
    }
    let lim = hay.len() - 8;
    let mut i = 0;
    while i < lim {
        if hay[i] == b'@' {
            match hay[i + 1] {
                // spellchecker:disable-next-line
                b'l' if &hay[i + 2..i + 8] == b"icense" => return true,
                b'p' if &hay[i + 2..i + 9] == b"reserve" => return true,
                _ => {}
            }
        }
        i += 1;
    }
    false
}

#[inline]
fn classify(bytes: &[u8], is_block: bool, lic: bool) -> CommentContent {
    if bytes.is_empty() {
        return CommentContent::None;
    }
    match bytes[0] {
        b'!' => return CommentContent::Legal,
        b'*' if is_block => {
            if !bytes.iter().all(|&c| c == b'*') {
                return if lic { CommentContent::JsdocLegal } else { CommentContent::Jsdoc };
            }
            return CommentContent::None;
        }
        _ => {}
    }

    let mut start = 0;
    while start < bytes.len() && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    if start >= bytes.len() {
        return CommentContent::None;
    }

    let rest = &bytes[start..];
    if (rest.starts_with(b"@__KEY__") || rest.starts_with(b"#__KEY__") || !rest[0].is_ascii())
        && is_property_key_annotation(bytes)
    {
        return CommentContent::PropertyKey;
    }

    match bytes[start] {
        b'@' => {
            start += 1;
            if start >= bytes.len() {
                return CommentContent::None;
            }
            if bytes[start..].starts_with(b"vite") {
                return CommentContent::Vite;
            }
            if bytes[start..].starts_with(b"license") || bytes[start..].starts_with(b"preserve") {
                return CommentContent::Legal;
            }
        }
        b'#' => start += 1,
        b'w' => {
            if bytes[start..].starts_with(b"webpack")
                && start + 7 < bytes.len()
                && bytes[start + 7].is_ascii_uppercase()
            {
                return CommentContent::Webpack;
            }
            return if lic { CommentContent::Legal } else { CommentContent::None };
        }
        b'v' | b'c' | b'n' | b'i' => {
            let rest = &bytes[start..];
            if rest.starts_with(b"v8 ignore")
                || rest.starts_with(b"c8 ignore")
                || rest.starts_with(b"node:coverage")
                || rest.starts_with(b"istanbul ignore")
            {
                return if is_coverage_ignore_file(rest) {
                    CommentContent::CoverageIgnoreFile
                } else {
                    CommentContent::CoverageIgnore
                };
            }
            return if lic { CommentContent::Legal } else { CommentContent::None };
        }
        _ => return if lic { CommentContent::Legal } else { CommentContent::None },
    }

    if start < bytes.len() && bytes[start..].starts_with(b"__") {
        let rest = &bytes[start + 2..];
        if rest.starts_with(b"PURE__") {
            return CommentContent::Pure;
        } else if rest.starts_with(b"NO_SIDE_EFFECTS__") {
            return CommentContent::NoSideEffects;
        }
    }

    if lic { CommentContent::Legal } else { CommentContent::None }
}

fn is_property_key_annotation(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .is_ok_and(|source| matches!(source.trim().strip_prefix(['@', '#']), Some("__KEY__")))
}

fn is_coverage_ignore_file(source: &[u8]) -> bool {
    fn starts_with_directive(source: &[u8], directive: &[u8]) -> bool {
        source
            .strip_prefix(directive)
            .is_some_and(|rest| rest.first().is_none_or(u8::is_ascii_whitespace))
    }

    starts_with_directive(source, b"v8 ignore file")
        || starts_with_directive(source, b"istanbul ignore file")
}

fn has_nl_cr(bytes: &[u8]) -> bool {
    bytes.iter().any(|&b| b == b'\n' || b == b'\r')
}

#[inline]
fn push_comment_record(
    src: &[u8],
    sl: usize,
    start: u32,
    end: u32,
    kind: CommentKind,
    followed_by_newline: bool,
    content: CommentContent,
    lanes: &mut Lanes,
) {
    let mut c = Comment::new(start, end.min(sl as u32), kind);
    c.content = content;

    let mut q = (start as usize).min(sl);
    let preceded_by_newline = loop {
        if q == 0 {
            break true;
        }
        let b = src[q - 1];
        if b == b'\n' || b == b'\r' {
            break true;
        }
        if !is_lex_ws(b) {
            break false;
        }
        q -= 1;
    };
    c.set_preceded_by_newline(preceded_by_newline);
    c.set_followed_by_newline(followed_by_newline);

    lanes.comments.push(c);
}

#[inline]
fn is_lex_ws(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

#[cfg(test)]
mod tests {
    use oxc_ast::ast::CommentContent;

    use super::classify;

    #[test]
    fn coverage_ignore_file() {
        for (source, is_block) in [
            (b"v8 ignore file".as_slice(), true),
            (b"v8 ignore file".as_slice(), false),
            (b"v8 ignore file -- @preserve".as_slice(), true),
            (b"istanbul ignore file".as_slice(), true),
            (b"istanbul ignore file -- generated".as_slice(), false),
        ] {
            assert_eq!(classify(source, is_block, false), CommentContent::CoverageIgnoreFile);
        }

        for source in [
            b"v8 ignore next".as_slice(),
            b"v8 ignore filename",
            b"c8 ignore file",
            b"istanbul ignore next",
        ] {
            assert_eq!(classify(source, true, false), CommentContent::CoverageIgnore);
        }
    }

    #[test]
    fn property_key_annotation() {
        for source in [
            b"@__KEY__".as_slice(),
            b" #__KEY__ ",
            b"\t@__KEY__\n",
            "\u{a0}@__KEY__\u{a0}".as_bytes(),
        ] {
            assert_eq!(classify(source, true, false), CommentContent::PropertyKey);
        }

        for source in
            [b"__KEY__".as_slice(), b"@ __KEY__", b"@__key__", b"@__KEY___", b"@__KEY__ extra"]
        {
            assert_eq!(classify(source, true, false), CommentContent::None);
        }
    }
}
