use crate::{comment_meta, error::diag_code, lanes::Lanes, tables::Tables};

use super::super::{
    HASHBANG, LCOM, TMPL_HEAD, TMPL_MIDDLE, TMPL_NOSUB, TMPL_TAIL,
    bitmap::{bm_clear_range, bm_set1},
    find::{find_line_terminator, find_opener, find_opener6},
};

use super::common::{lex_slash, lex_string, lex_template_segment, skip_unicode_brace_escape};

pub unsafe fn carve(
    t: &Tables,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    opch: *mut u64,
    word: *const u64,
    digit: *const u64,
    ts: bool,
    lanes: &mut Lanes,
) {
    let src = srcs.as_ptr();
    let mut depth: Vec<u32> = Vec::with_capacity(64);
    let mut i = 0usize;
    if n >= 2 && *src == b'#' && *src.add(1) == b'!' {
        let end = find_line_terminator(src, n, 2);
        *kind = HASHBANG;
        bm_clear_range(st, 1, end - 1);
        if end < n {
            bm_set1(st, end);
        }
        i = end;
    }
    loop {
        let nsub = depth.len();
        let s = if nsub != 0 { find_opener6(src, n, i) } else { find_opener(src, n, i) };
        if s >= n {
            break;
        }
        let c = *src.add(s);
        match c {
            b'"' | b'\'' => {
                i = lex_string(src, srcs, n, st, kind, s, c, lanes);
            }
            b'`' => {
                let (end, opened_sub) =
                    lex_template_segment(src, srcs, n, st, kind, s, TMPL_HEAD, TMPL_NOSUB, lanes);
                if opened_sub {
                    depth.push(0);
                }
                i = end;
            }
            b'{' => {
                if s >= 2 && *src.add(s - 1) == b'u' && *src.add(s - 2) == b'\\' {
                    i = skip_unicode_brace_escape(src, n, s);
                } else {
                    let top = depth.len() - 1;
                    depth[top] += 1;
                    i = s + 1;
                }
            }
            b'}' => {
                let top = depth.len() - 1;
                if depth[top] > 0 {
                    depth[top] -= 1;
                    i = s + 1;
                } else {
                    depth.pop();
                    let (end, opened_sub) = lex_template_segment(
                        src,
                        srcs,
                        n,
                        st,
                        kind,
                        s,
                        TMPL_MIDDLE,
                        TMPL_TAIL,
                        lanes,
                    );
                    if opened_sub {
                        depth.push(0);
                    }
                    i = end;
                }
            }
            b'/' => {
                i = lex_slash(t, src, srcs, n, st, kind, opch, word, digit, ts, s, lanes);
            }
            b'<' => {
                let html = s + 3 < n
                    && *src.add(s + 1) == b'!'
                    && *src.add(s + 2) == b'-'
                    && *src.add(s + 3) == b'-';
                if html && (!lanes.module || html_close_at_line_start(srcs, s)) {
                    if lanes.module {
                        lanes.push_diag(s as u32, 4, diag_code::HTML_COMMENT_IN_MODULE);
                    }
                    let end = find_line_terminator(src, n, s + 4);
                    *kind.add(s) = LCOM;
                    if end > s + 1 {
                        bm_clear_range(st, s + 1, end - 1);
                    }
                    if end < n {
                        bm_set1(st, end);
                    }
                    // `<`, `!`, `-` are opchars: clear the span from `opch`
                    // or `coalesce` would re-tokenize `<!--` as operators.
                    bm_clear_range(opch, s, end - 1);
                    // meta_byte_exact skips a 2-byte delimiter; pass s + 2 so
                    // the 4-byte `<!--` is skipped. The record keeps (s, end).
                    let m = comment_meta::meta_byte_exact(
                        &srcs[..n],
                        (s + 2) as u32,
                        end as u32,
                        false,
                    );
                    lanes.comment_meta.push(m);
                    lanes.push_comment_record(srcs, n, s as u32, end as u32, false, m);
                    i = end;
                } else {
                    i = s + 1;
                }
            }
            b'>' => {
                // Annex B B.1.3: `-->` begins a line comment, but only at
                if s >= 2
                    && *src.add(s - 1) == b'-'
                    && *src.add(s - 2) == b'-'
                    && !lanes.module
                    && html_close_at_line_start(srcs, s - 2)
                {
                    let start = s - 2;
                    let end = find_line_terminator(src, n, s + 1);
                    *kind.add(start) = LCOM;
                    bm_set1(st, start);
                    if end > start + 1 {
                        bm_clear_range(st, start + 1, end - 1);
                    }
                    if end < n {
                        bm_set1(st, end);
                    }
                    // Clear the span from `opch` (see `<!--` above).
                    bm_clear_range(opch, start, end - 1);
                    // `-->` is a 3-byte delimiter; pass start + 1 so the
                    // 2-byte-delimiter body resolves to [s + 1, end).
                    let m = comment_meta::meta_byte_exact(
                        &srcs[..n],
                        (start + 1) as u32,
                        end as u32,
                        false,
                    );
                    lanes.comment_meta.push(m);
                    lanes.push_comment_record(srcs, n, start as u32, end as u32, false, m);
                    i = end;
                } else {
                    i = s + 1;
                }
            }
            _ => {
                i = s + 1;
            }
        }
    }
}

/// Annex B B.1.3: a `-->` close-comment counts only at line start — scanning
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
