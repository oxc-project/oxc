use oxc_ast::{Comment, CommentContent, CommentKind};

use crate::{error::DiagCode, lanes::Lanes, token::tk};

use crate::pipeline::{
    bitmap::{bm_clear_range, bm_set},
    find::find_line_terminator,
    scan::{scan_block_comment, scan_line_comment},
};

const CONTENT_NONE: u8 = 0;
const CONTENT_LEGAL: u8 = 1;
const CONTENT_JSDOC: u8 = 2;
const CONTENT_JSDOC_LEGAL: u8 = 3;
const CONTENT_PURE: u8 = 4;
const CONTENT_NO_SIDE_EFFECTS: u8 = 5;
const CONTENT_WEBPACK: u8 = 6;
const CONTENT_VITE: u8 = 7;
const CONTENT_COVERAGE_IGNORE: u8 = 8;
const CONTENT_COVERAGE_IGNORE_FILE: u8 = 9;
const CONTENT_PROPERTY_KEY: u8 = 10;
const META_MULTILINE: u8 = 0x10;

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
    let m = meta_byte_flags(&srcs[..n], s as u32, end as u32, false, false, lic);
    debug_assert_eq!(
        m,
        meta_byte_exact(&srcs[..n], s as u32, end as u32, false),
        "LCOM meta fused != exact at {s}"
    );
    lanes.comment_meta.push(m);
    push_comment_record(srcs, n, s as u32, end as u32, false, m, lanes);
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
        meta_byte_flags(&srcs[..n], s as u32, end as u32, true, saw_nl, lic)
    } else {
        meta_byte_exact(&srcs[..n], s as u32, end as u32, true)
    };
    debug_assert_eq!(
        m,
        meta_byte_exact(&srcs[..n], s as u32, end as u32, true),
        "BCOM meta fused != exact at {s}"
    );
    lanes.comment_meta.push(m);
    push_comment_record(srcs, n, s as u32, end as u32, true, m, lanes);
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
    // meta_byte_exact skips a 2-byte delimiter; pass s + 2 so the 4-byte
    // `<!--` is skipped. The record keeps (s, end).
    let m = meta_byte_exact(&srcs[..n], (s + 2) as u32, end as u32, false);
    lanes.comment_meta.push(m);
    push_comment_record(srcs, n, s as u32, end as u32, false, m, lanes);
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
    let m = meta_byte_exact(&srcs[..n], (start + 1) as u32, end as u32, false);
    lanes.comment_meta.push(m);
    push_comment_record(srcs, n, start as u32, end as u32, false, m, lanes);
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

#[inline]
fn meta_byte_flags(src: &[u8], start: u32, end: u32, is_block: bool, ml: bool, lic: bool) -> u8 {
    let ord = classify(body(src, start, end, is_block), is_block, lic) & 0x0F;
    ord | if ml && is_block { META_MULTILINE } else { 0 }
}

#[inline]
fn meta_byte_exact(src: &[u8], start: u32, end: u32, is_block: bool) -> u8 {
    let b = body(src, start, end, is_block);
    let lic = contains_license_or_preserve(b);
    let ord = classify(b, is_block, lic) & 0x0F;
    let ml = is_block && has_nl_cr(b);
    ord | if ml { META_MULTILINE } else { 0 }
}

#[inline]
fn body(src: &[u8], start: u32, end: u32, is_block: bool) -> &[u8] {
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
fn classify(bytes: &[u8], is_block: bool, lic: bool) -> u8 {
    if bytes.is_empty() {
        return CONTENT_NONE;
    }
    match bytes[0] {
        b'!' => return CONTENT_LEGAL,
        b'*' if is_block => {
            if !bytes.iter().all(|&c| c == b'*') {
                return if lic { CONTENT_JSDOC_LEGAL } else { CONTENT_JSDOC };
            }
            return CONTENT_NONE;
        }
        _ => {}
    }

    let mut start = 0;
    while start < bytes.len() && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    if start >= bytes.len() {
        return CONTENT_NONE;
    }

    let rest = &bytes[start..];
    if (rest.starts_with(b"@__KEY__") || rest.starts_with(b"#__KEY__") || !rest[0].is_ascii())
        && is_property_key_annotation(bytes)
    {
        return CONTENT_PROPERTY_KEY;
    }

    match bytes[start] {
        b'@' => {
            start += 1;
            if start >= bytes.len() {
                return CONTENT_NONE;
            }
            if bytes[start..].starts_with(b"vite") {
                return CONTENT_VITE;
            }
            if bytes[start..].starts_with(b"license") || bytes[start..].starts_with(b"preserve") {
                return CONTENT_LEGAL;
            }
        }
        b'#' => start += 1,
        b'w' => {
            if bytes[start..].starts_with(b"webpack")
                && start + 7 < bytes.len()
                && bytes[start + 7].is_ascii_uppercase()
            {
                return CONTENT_WEBPACK;
            }
            return if lic { CONTENT_LEGAL } else { CONTENT_NONE };
        }
        b'v' | b'c' | b'n' | b'i' => {
            let rest = &bytes[start..];
            if rest.starts_with(b"v8 ignore")
                || rest.starts_with(b"c8 ignore")
                || rest.starts_with(b"node:coverage")
                || rest.starts_with(b"istanbul ignore")
            {
                return if is_coverage_ignore_file(rest) {
                    CONTENT_COVERAGE_IGNORE_FILE
                } else {
                    CONTENT_COVERAGE_IGNORE
                };
            }
            return if lic { CONTENT_LEGAL } else { CONTENT_NONE };
        }
        _ => return if lic { CONTENT_LEGAL } else { CONTENT_NONE },
    }

    if start < bytes.len() && bytes[start..].starts_with(b"__") {
        let rest = &bytes[start + 2..];
        if rest.starts_with(b"PURE__") {
            return CONTENT_PURE;
        } else if rest.starts_with(b"NO_SIDE_EFFECTS__") {
            return CONTENT_NO_SIDE_EFFECTS;
        }
    }

    if lic { CONTENT_LEGAL } else { CONTENT_NONE }
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
    blk: bool,
    meta: u8,
    lanes: &mut Lanes,
) {
    let kind = if !blk {
        CommentKind::Line
    } else if meta & META_MULTILINE != 0 {
        CommentKind::MultiLineBlock
    } else {
        CommentKind::SingleLineBlock
    };
    let mut c = Comment::new(start, end.min(sl as u32), kind);
    c.content = content_from_ordinal(meta);

    let s_ = (start as usize).min(sl);
    let mut q = s_;
    let pre = loop {
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
    c.set_preceded_by_newline(pre);

    if !blk {
        c.set_followed_by_newline(true);
    } else {
        let mut e = (end as usize).min(sl);
        let mut post = false;
        while e < sl && is_lex_ws(src[e]) {
            if src[e] == b'\n' || src[e] == b'\r' {
                post = true;
                break;
            }
            e += 1;
        }
        c.set_followed_by_newline(post);
    }
    lanes.comments.push(c);
}

#[inline]
fn content_from_ordinal(o: u8) -> CommentContent {
    match o & 0x0F {
        CONTENT_LEGAL => CommentContent::Legal,
        CONTENT_JSDOC => CommentContent::Jsdoc,
        CONTENT_JSDOC_LEGAL => CommentContent::JsdocLegal,
        CONTENT_PURE => CommentContent::Pure,
        CONTENT_NO_SIDE_EFFECTS => CommentContent::NoSideEffects,
        CONTENT_WEBPACK => CommentContent::Webpack,
        CONTENT_VITE => CommentContent::Vite,
        CONTENT_COVERAGE_IGNORE => CommentContent::CoverageIgnore,
        CONTENT_COVERAGE_IGNORE_FILE => CommentContent::CoverageIgnoreFile,
        CONTENT_PROPERTY_KEY => CommentContent::PropertyKey,
        _ => CommentContent::None,
    }
}

#[inline]
fn is_lex_ws(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

#[cfg(test)]
mod tests {
    use oxc_ast::ast::CommentContent;

    use super::{
        CONTENT_COVERAGE_IGNORE, CONTENT_COVERAGE_IGNORE_FILE, CONTENT_NONE, CONTENT_PROPERTY_KEY,
        classify, content_from_ordinal,
    };

    #[test]
    fn coverage_ignore_file() {
        for (source, is_block) in [
            (b"v8 ignore file".as_slice(), true),
            (b"v8 ignore file".as_slice(), false),
            (b"v8 ignore file -- @preserve".as_slice(), true),
            (b"istanbul ignore file".as_slice(), true),
            (b"istanbul ignore file -- generated".as_slice(), false),
        ] {
            assert_eq!(classify(source, is_block, false), CONTENT_COVERAGE_IGNORE_FILE);
            assert_eq!(
                content_from_ordinal(CONTENT_COVERAGE_IGNORE_FILE),
                CommentContent::CoverageIgnoreFile
            );
        }

        for source in [
            b"v8 ignore next".as_slice(),
            b"v8 ignore filename",
            b"c8 ignore file",
            b"istanbul ignore next",
        ] {
            assert_eq!(classify(source, true, false), CONTENT_COVERAGE_IGNORE);
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
            assert_eq!(classify(source, true, false), CONTENT_PROPERTY_KEY);
            assert_eq!(content_from_ordinal(CONTENT_PROPERTY_KEY), CommentContent::PropertyKey);
        }

        for source in
            [b"__KEY__".as_slice(), b"@ __KEY__", b"@__key__", b"@__KEY___", b"@__KEY__ extra"]
        {
            assert_eq!(classify(source, true, false), CONTENT_NONE);
        }
    }
}
