use std::mem;

use oxc_simd::{Cpu, cpu_multiversion};

use crate::token::tk;

use crate::pipeline::{bitmap::Bitmaps, find::find, kinds::Kinds, lanes::Lanes, source::Source};

use super::{
    comments::{
        html_close_comment_at, html_open_comment_at, lex_html_close_comment, lex_html_open_comment,
    },
    common::{
        OPENER_BYTES, OPENER_MODULE_BYTES, OPENER8_BYTES, OPENER8_MODULE_BYTES,
        find_line_terminator, lex_slash, lex_string, lex_template_segment,
        skip_unicode_brace_escape,
    },
};

#[cpu_multiversion]
pub(super) unsafe fn carve_js(
    src: Source,
    bitmaps: &Bitmaps,
    kinds: Kinds,
    ts: bool,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) {
    let Bitmaps { token_start, op_char, .. } = *bitmaps;
    // Reuse the previous lex's allocation, which matters for small inputs
    let mut depth = mem::take(&mut lanes.template_depth);
    depth.clear();
    let mut i = 0usize;
    if src.len() >= 2 && src[0] == b'#' && src[1] == b'!' {
        let end = find_line_terminator(src, 2, cpu);
        kinds.set(0, tk!(Hashbang));
        token_start.clear_range(1, end - 1, cpu);
        if end < src.len() {
            token_start.set(end, cpu);
        }
        i = end;
    }
    loop {
        // HTML close comments only exist in scripts, so modules needn't stop at closing brackets
        let s = match (depth.is_empty(), lanes.module) {
            (true, false) => find(src, i, OPENER_BYTES, cpu),
            (true, true) => find(src, i, OPENER_MODULE_BYTES, cpu),
            (false, false) => find(src, i, OPENER8_BYTES, cpu),
            (false, true) => find(src, i, OPENER8_MODULE_BYTES, cpu),
        };
        if s >= src.len() {
            break;
        }
        let c = src[s];
        match c {
            b'"' | b'\'' => {
                i = lex_string(src, token_start, kinds, s, c, lanes, cpu);
            }
            b'`' => {
                let (end, opened_sub) = lex_template_segment(
                    src,
                    token_start,
                    kinds,
                    s,
                    tk!(TemplateHead),
                    tk!(TemplateNoSub),
                    lanes,
                    cpu,
                );
                if opened_sub {
                    depth.push(0);
                }
                i = end;
            }
            b'{' => {
                if s >= 2 && src[s - 1] == b'u' && src[s - 2] == b'\\' {
                    i = skip_unicode_brace_escape(src, s, cpu);
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
                        token_start,
                        kinds,
                        s,
                        tk!(TemplateMiddle),
                        tk!(TemplateTail),
                        lanes,
                        cpu,
                    );
                    if opened_sub {
                        depth.push(0);
                    }
                    i = end;
                }
            }
            b'/' => {
                i = lex_slash(src, bitmaps, kinds, ts, s, lanes, cpu);
            }
            b'<' => {
                // Annex B B.1.1: `<!--` begins a line comment.
                i = if html_open_comment_at(src, s, lanes.module, cpu) {
                    lex_html_open_comment(src, token_start, kinds, op_char, s, lanes)
                } else {
                    s + 1
                };
            }
            b'>' => {
                // Annex B B.1.3: `-->` begins a line comment, but only at line start.
                i = if html_close_comment_at(src, s, lanes.module, cpu) {
                    lex_html_close_comment(src, token_start, kinds, op_char, s, lanes)
                } else {
                    s + 1
                };
            }
            _ => {
                i = s + 1;
            }
        }
    }
    lanes.template_depth = depth;
}
