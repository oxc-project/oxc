use oxc_ast::{Comment, CommentContent, CommentKind};
use oxc_simd::{BASELINE, Cpu, cpu_multiversion, cpu_version};

use crate::{error::DiagCode, token::tk};

use crate::pipeline::{
    bitmap::Bitmap,
    bytes::unicode_ws_len,
    kinds::Kinds,
    lanes::Lanes,
    scan::{scan_block_comment, scan_line_comment},
    source::Source,
};

use super::common::find_line_terminator;

/// Lex the `//` line comment at `s`. Returns the resume index.
#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn lex_line_comment(
    src: Source,
    token_start: Bitmap,
    kinds: Kinds,
    s: usize,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) -> usize {
    let (end, lic_q) = scan_line_comment(src, s + 2, cpu);
    kinds.set(s, tk!(LineComment));
    if end > s + 1 {
        token_start.clear_range(s + 1, end - 1, cpu);
    }
    if end < src.len() {
        token_start.set(end, cpu);
    }
    let lic = lic_q >= 0 && (lic_q as usize) + 8 < end;
    let body = comment_body(src.bytes(), s as u32, end as u32, false, cpu);
    let m = classify(body, false, lic, cpu);
    debug_assert_eq!(m, classify_exact(body, false, cpu), "LCOM content fused != exact at {s}");
    push_comment_record(src, s as u32, end as u32, CommentKind::Line, true, m, lanes, cpu);
    end
}

/// Lex the `/*` block comment at `s` (unterminated at EOF is diagnosed).
/// Returns the resume index.
#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn lex_block_comment(
    src: Source,
    token_start: Bitmap,
    kinds: Kinds,
    s: usize,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) -> usize {
    let (e, mut saw_nl, lic_q) = scan_block_comment(src, s + 2, cpu);
    let end = if e < src.len() { e + 1 } else { src.len() };
    if e >= src.len() {
        lanes.push_diag(s as u32, (src.len() - s) as u32, DiagCode::UnterminatedBlockComment);
    }
    kinds.set(s, tk!(BlockComment));
    if end > s + 1 {
        token_start.clear_range(s + 1, end - 1, cpu);
    }

    let body = comment_body(src.bytes(), s as u32, end as u32, true, cpu);
    let m = if e < src.len() {
        let lic = lic_q >= 0 && (lic_q as usize) + 8 < e - 1;
        let m = classify(body, true, lic, cpu);
        debug_assert_eq!(m, classify_exact(body, true, cpu), "BCOM content fused != exact at {s}");
        debug_assert_eq!(saw_nl, has_nl_cr(body, cpu), "BCOM saw_nl != exact at {s}");
        m
    } else {
        saw_nl = has_nl_cr(body, cpu);
        classify_exact(body, true, cpu)
    };

    let comment_kind =
        if saw_nl { CommentKind::MultiLineBlock } else { CommentKind::SingleLineBlock };

    let mut q = end;
    let mut followed_by_newline = false;
    while q < src.len() && is_lex_ws(src[q], cpu) {
        if src[q] == b'\n' || src[q] == b'\r' {
            followed_by_newline = true;
            break;
        }
        q += 1;
    }
    if !followed_by_newline && q < src.len() && src[q] >= 0x80 {
        followed_by_newline = followed_by_line_terminator(src, q);
    }

    push_comment_record(
        src,
        s as u32,
        end as u32,
        comment_kind,
        followed_by_newline,
        m,
        lanes,
        cpu,
    );
    end
}

/// Annex B B.1.1: does the `<` at `s` open a `<!--` comment? Always in a script; in a module
/// only at line start, where it is diagnosed rather than read as operators. Shared by `carve`
/// and `carve_jsx` JS mode: the goal, not the JSX setting, decides.
#[cpu_multiversion]
#[inline(always)]
pub(super) fn html_open_comment_at(src: Source, s: usize, module: bool, cpu: impl Cpu) -> bool {
    s + 3 < src.len()
        && src[s + 1] == b'!'
        && src[s + 2] == b'-'
        && src[s + 3] == b'-'
        && (!module || html_close_at_line_start(src, s, cpu))
}

/// Lex the `<!--` comment opening at `s` (see [`html_open_comment_at`]). Returns the resume
/// index. Cold: a literal `<!--`.
///
/// As it's cold code, it's compiled only once, not separately for each CPU feature set,
/// which keeps the binary smaller. `#[inline(never)]` stops it being inlined into
/// the separate copies of its callers. It uses [`BASELINE`] for functions it calls.
#[cpu_version(Generic)]
#[cold]
#[inline(never)]
pub(super) unsafe fn lex_html_open_comment(
    src: Source,
    token_start: Bitmap,
    kinds: Kinds,
    op_char: Bitmap,
    s: usize,
    lanes: &mut Lanes,
) -> usize {
    let cpu = BASELINE;

    if lanes.module {
        lanes.push_diag(s as u32, 4, DiagCode::HtmlCommentInModule);
    }
    let end = find_line_terminator(src, s + 4, cpu);
    kinds.set(s, tk!(LineComment));
    if end > s + 1 {
        token_start.clear_range(s + 1, end - 1, cpu);
    }
    if end < src.len() {
        token_start.set(end, cpu);
    }
    // Clear the comment from op_char, or coalesce would read it as operators.
    op_char.clear_range(s, end - 1, cpu);
    // `comment_body` skips a 2-byte delimiter; pass s + 2 so the 4-byte
    // `<!--` is skipped. The record keeps (s, end).
    let body = comment_body(src.bytes(), (s + 2) as u32, end as u32, false, cpu);
    let m = classify_exact(body, false, cpu);
    push_comment_record(src, s as u32, end as u32, CommentKind::HtmlOpen, true, m, lanes, cpu);
    end
}

/// Annex B B.1.3: does the `>` at `s` end a `-->` that opens a comment? Only in a script, and
/// only at line start.
#[cpu_multiversion]
#[inline(always)]
pub(super) fn html_close_comment_at(src: Source, s: usize, module: bool, cpu: impl Cpu) -> bool {
    !module
        && s >= 2
        && src[s - 1] == b'-'
        && src[s - 2] == b'-'
        && html_close_at_line_start(src, s - 2, cpu)
}

/// Lex the `-->` comment whose `>` is at `s` (see [`html_close_comment_at`]). Returns the
/// resume index. Cold: a literal `-->` at line start.
///
/// As it's cold code, it's compiled only once, not separately for each CPU feature set,
/// which keeps the binary smaller. `#[inline(never)]` stops it being inlined into
/// the separate copies of its callers. It uses [`BASELINE`] for functions it calls.
#[cpu_version(Generic)]
#[cold]
#[inline(never)]
pub(super) unsafe fn lex_html_close_comment(
    src: Source,
    token_start: Bitmap,
    kinds: Kinds,
    op_char: Bitmap,
    s: usize,
    lanes: &mut Lanes,
) -> usize {
    let cpu = BASELINE;

    let start = s - 2;
    let end = find_line_terminator(src, s + 1, cpu);
    kinds.set(start, tk!(LineComment));
    token_start.set(start, cpu);
    if end > start + 1 {
        token_start.clear_range(start + 1, end - 1, cpu);
    }
    if end < src.len() {
        token_start.set(end, cpu);
    }
    // Clear the comment from op_char, as for the html open comment.
    op_char.clear_range(start, end - 1, cpu);
    // `-->` is a 3-byte delimiter; pass start + 1 so the 2-byte-delimiter
    // body resolves to [s + 1, end).
    let body = comment_body(src.bytes(), (start + 1) as u32, end as u32, false, cpu);
    let m = classify_exact(body, false, cpu);
    push_comment_record(src, start as u32, end as u32, CommentKind::HtmlClose, true, m, lanes, cpu);
    end
}

/// Annex B B.1.3: a `-->` close-comment counts only at line start - scanning
/// back must reach a LineTerminator (or start of input) crossing nothing but
/// whitespace and block comments; a newline inside a crossed block comment
/// also qualifies. Cold: called only on a literal `-->`.
#[cpu_multiversion]
fn html_close_at_line_start(src: Source, mut q: usize, _cpu: impl Cpu) -> bool {
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

#[cpu_multiversion]
#[inline]
fn classify_exact(body: &[u8], is_block: bool, cpu: impl Cpu) -> CommentContent {
    let lic = contains_license_or_preserve(body, cpu);
    classify(body, is_block, lic, cpu)
}

#[cpu_multiversion]
#[inline]
fn comment_body(src: &[u8], start: u32, end: u32, is_block: bool, _cpu: impl Cpu) -> &[u8] {
    let (lo, hi) = if is_block {
        (start as usize + 2, (end as usize).saturating_sub(2))
    } else {
        (start as usize + 2, end as usize)
    };
    let lo = lo.min(src.len());
    let hi = hi.clamp(lo, src.len());
    &src[lo..hi]
}

#[cpu_multiversion]
fn contains_license_or_preserve(hay: &[u8], _cpu: impl Cpu) -> bool {
    if hay.len() < 9 {
        return false;
    }
    let lim = hay.len() - 8;
    let mut i = 0;
    while i < lim {
        if hay[i] == b'@' {
            match hay[i + 1] {
                b'l' if &hay[i + 2..i + 8] == b"icense" => return true,
                b'p' if &hay[i + 2..i + 9] == b"reserve" => return true,
                _ => {}
            }
        }
        i += 1;
    }
    false
}

#[cpu_multiversion]
#[inline]
fn classify(bytes: &[u8], is_block: bool, lic: bool, cpu: impl Cpu) -> CommentContent {
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
        && is_property_key_annotation(bytes, cpu)
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
        b't' => {
            if bytes[start..].starts_with(b"turbopack")
                && start + 9 < bytes.len()
                && bytes[start + 9].is_ascii_uppercase()
            {
                return CommentContent::Turbopack;
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
                return if is_coverage_ignore_file(rest, cpu) {
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

#[cpu_multiversion]
fn is_property_key_annotation(bytes: &[u8], _cpu: impl Cpu) -> bool {
    std::str::from_utf8(bytes)
        .is_ok_and(|source| matches!(source.trim().strip_prefix(['@', '#']), Some("__KEY__")))
}

#[cpu_multiversion]
fn is_coverage_ignore_file(source: &[u8], cpu: impl Cpu) -> bool {
    #[cpu_multiversion]
    fn starts_with_directive(source: &[u8], directive: &[u8], _cpu: impl Cpu) -> bool {
        source
            .strip_prefix(directive)
            .is_some_and(|rest| rest.first().is_none_or(u8::is_ascii_whitespace))
    }

    starts_with_directive(source, b"v8 ignore file", cpu)
        || starts_with_directive(source, b"istanbul ignore file", cpu)
}

#[cpu_multiversion]
fn has_nl_cr(bytes: &[u8], _cpu: impl Cpu) -> bool {
    bytes.iter().any(|&b| b == b'\n' || b == b'\r')
}

#[cpu_multiversion]
#[inline]
fn push_comment_record(
    src: Source,
    start: u32,
    end: u32,
    kind: CommentKind,
    followed_by_newline: bool,
    content: CommentContent,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) {
    let mut c = Comment::new(start, end.min(src.len() as u32), kind);
    c.content = content;

    let mut q = (start as usize).min(src.len());
    let preceded_by_newline = loop {
        if q == 0 {
            break true;
        }
        let b = src[q - 1];
        if b == b'\n' || b == b'\r' {
            break true;
        }
        if !is_lex_ws(b, cpu) {
            break b >= 0x80 && preceded_by_line_terminator(src, q);
        }
        q -= 1;
    };
    c.set_preceded_by_newline(preceded_by_newline);
    c.set_followed_by_newline(followed_by_newline);

    lanes.out.comments.push(c);
}

#[cpu_multiversion]
#[inline]
fn is_lex_ws(c: u8, _cpu: impl Cpu) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

/// LS (U+2028) or PS (U+2029).
#[inline]
fn is_ls_ps(c0: u8, c1: u8, c2: u8) -> bool {
    c0 == 0xE2 && c1 == 0x80 && (c2 == 0xA8 || c2 == 0xA9)
}

/// Cold: scan forward from q over WhiteSpace, including non-ASCII.
#[cpu_version(Generic)]
#[cold]
#[inline(never)]
fn followed_by_line_terminator(src: Source, mut q: usize) -> bool {
    let cpu = BASELINE;
    while q < src.len() {
        let b = src[q];
        if b == b'\n' || b == b'\r' {
            return true;
        }
        if is_lex_ws(b, cpu) {
            q += 1;
            continue;
        }
        if b < 0x80 {
            return false;
        }
        let len = src.unicode_ws_len_at(q, cpu);
        if len == 0 {
            return false;
        }
        if is_ls_ps(src[q], src[q + 1], src[q + 2]) {
            return true;
        }
        q += len;
    }
    false
}

/// Cold: scan backward from q over WhiteSpace, including non-ASCII. Start of input is a newline.
#[cpu_version(Generic)]
#[cold]
#[inline(never)]
fn preceded_by_line_terminator(src: Source, mut q: usize) -> bool {
    let cpu = BASELINE;
    loop {
        if q == 0 {
            return true;
        }
        let b = src[q - 1];
        if b == b'\n' || b == b'\r' {
            return true;
        }
        if is_lex_ws(b, cpu) {
            q -= 1;
            continue;
        }
        if b < 0x80 {
            return false;
        }
        if q >= 3 {
            let (c0, c1, c2) = (src[q - 3], src[q - 2], src[q - 1]);
            if unicode_ws_len(c0, c1, c2, cpu) == 3 {
                if is_ls_ps(c0, c1, c2) {
                    return true;
                }
                q -= 3;
                continue;
            }
        }
        if q >= 2 {
            let (c0, c1) = (src[q - 2], src[q - 1]);
            if unicode_ws_len(c0, c1, 0, cpu) == 2 {
                q -= 2;
                continue;
            }
        }
        return false;
    }
}

#[cfg(test)]
mod tests {
    use oxc_ast::ast::CommentContent;
    use oxc_simd::{Cpu, test_cpus};

    use super::classify;

    #[test_cpus]
    fn coverage_ignore_file(cpu: impl Cpu) {
        for (source, is_block) in [
            (b"v8 ignore file".as_slice(), true),
            (b"v8 ignore file".as_slice(), false),
            (b"v8 ignore file -- @preserve".as_slice(), true),
            (b"istanbul ignore file".as_slice(), true),
            (b"istanbul ignore file -- generated".as_slice(), false),
        ] {
            assert_eq!(classify(source, is_block, false, cpu), CommentContent::CoverageIgnoreFile);
        }

        for source in [
            b"v8 ignore next".as_slice(),
            b"v8 ignore filename",
            b"c8 ignore file",
            b"istanbul ignore next",
        ] {
            assert_eq!(classify(source, true, false, cpu), CommentContent::CoverageIgnore);
        }
    }

    #[test_cpus]
    fn property_key_annotation(cpu: impl Cpu) {
        for source in [
            b"@__KEY__".as_slice(),
            b" #__KEY__ ",
            b"\t@__KEY__\n",
            "\u{a0}@__KEY__\u{a0}".as_bytes(),
        ] {
            assert_eq!(classify(source, true, false, cpu), CommentContent::PropertyKey);
        }

        for source in
            [b"__KEY__".as_slice(), b"@ __KEY__", b"@__key__", b"@__KEY___", b"@__KEY__ extra"]
        {
            assert_eq!(classify(source, true, false, cpu), CommentContent::None);
        }
    }

    #[test_cpus]
    fn turbopack(cpu: impl Cpu) {
        for (source, is_block) in [
            (b" turbopackChunkName: \"x\" ".as_slice(), true),
            (b"turbopackOptional: true".as_slice(), true),
            (b"\tturbopackIgnore: true\n".as_slice(), true),
            (b" turbopackIgnore: true".as_slice(), false),
        ] {
            assert_eq!(classify(source, is_block, false, cpu), CommentContent::Turbopack);
        }

        for source in [b" turbopack ".as_slice(), b" turbopack", b" turbopackignore: true"] {
            assert_eq!(classify(source, true, false, cpu), CommentContent::None);
        }

        // Without a magic comment, @license or @preserve still makes it legal
        assert_eq!(classify(b" turbopack @license ", true, true, cpu), CommentContent::Legal);
    }
}
