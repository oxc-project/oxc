use crate::{lanes::Lanes, token::tk};

use crate::pipeline::{
    bitmap::{bm_clear_range, bm_set},
    find::{find_line_terminator, find_opener, find_opener6},
    tables::Tables,
};

use super::common::{
    html_close_comment_at, html_open_comment_at, lex_html_close_comment, lex_html_open_comment,
    lex_slash, lex_string, lex_template_segment, skip_unicode_brace_escape,
};

pub(super) unsafe fn carve_js(
    t: &Tables,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    opch: *mut u64,
    word: *const u64,
    ts: bool,
    lanes: &mut Lanes,
) {
    let src = srcs.as_ptr();
    let mut depth: Vec<u32> = Vec::with_capacity(64);
    let mut i = 0usize;
    if n >= 2 && *src == b'#' && *src.add(1) == b'!' {
        let end = find_line_terminator(src, n, 2);
        *kind = tk!(Hashbang);
        bm_clear_range(st, 1, end - 1);
        if end < n {
            bm_set(st, end);
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
                let (end, opened_sub) = lex_template_segment(
                    src,
                    srcs,
                    n,
                    st,
                    kind,
                    s,
                    tk!(TemplateHead),
                    tk!(TemplateNoSub),
                    lanes,
                );
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
                        tk!(TemplateMiddle),
                        tk!(TemplateTail),
                        lanes,
                    );
                    if opened_sub {
                        depth.push(0);
                    }
                    i = end;
                }
            }
            b'/' => {
                i = lex_slash(t, src, srcs, n, st, kind, opch, word, ts, s, lanes);
            }
            b'<' => {
                // Annex B B.1.1: `<!--` begins a line comment.
                i = if html_open_comment_at(srcs, n, s, lanes.module) {
                    lex_html_open_comment(src, srcs, n, st, kind, opch, s, lanes)
                } else {
                    s + 1
                };
            }
            b'>' => {
                // Annex B B.1.3: `-->` begins a line comment, but only at line start.
                i = if html_close_comment_at(srcs, s, lanes.module) {
                    lex_html_close_comment(src, srcs, n, st, kind, opch, s, lanes)
                } else {
                    s + 1
                };
            }
            _ => {
                i = s + 1;
            }
        }
    }
}
