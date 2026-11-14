use std::mem;

use oxc_simd::{Cpu, cpu_multiversion};

use crate::pipeline::{
    bitmap::Bitmaps,
    find::find,
    lex::{Lex, LexState},
};

use super::{
    comments::{
        html_close_comment_at, html_open_comment_at, lex_html_close_comment, lex_html_open_comment,
    },
    common::{
        OPENER_BYTES, OPENER_MODULE_BYTES, OPENER8_BYTES, OPENER8_MODULE_BYTES,
        is_unicode_escape_brace, lex_hashbang, lex_slash, lex_string, skip_unicode_brace_escape,
    },
};

#[cpu_multiversion]
pub(super) unsafe fn carve_js(lex: Lex, state: &mut LexState, cpu: impl Cpu) {
    let Lex { src, bitmaps, kinds, options } = lex;
    let Bitmaps { token_start, op_char, .. } = *bitmaps;
    let LexState { lanes, disambiguate, substitutions, .. } = state;
    let module = options.source_type_module;
    // Reuse the previous lex's allocation, which matters for small inputs
    let mut subs = mem::take(substitutions);
    subs.clear();
    let mut i = lex_hashbang(src, token_start, kinds, cpu);
    loop {
        // HTML close comments only exist in scripts, so modules needn't stop at closing brackets
        let s = match (subs.is_empty(), module) {
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
                i = subs.lex_template(src, token_start, kinds, s, lanes, cpu);
            }
            b'{' => {
                if is_unicode_escape_brace(src, s, cpu) {
                    i = skip_unicode_brace_escape(src, s, cpu);
                } else {
                    subs.open_brace();
                    i = s + 1;
                }
            }
            b'}' => {
                i = subs.close_brace(src, token_start, kinds, s, lanes, cpu);
            }
            b'/' => {
                i = lex_slash(lex, s, lanes, disambiguate, cpu);
            }
            b'<' => {
                // Annex B B.1.1: `<!--` begins a line comment.
                i = if html_open_comment_at(src, s, module, cpu) {
                    lex_html_open_comment(src, token_start, kinds, op_char, s, module, lanes)
                } else {
                    s + 1
                };
            }
            b'>' => {
                // Annex B B.1.3: `-->` begins a line comment, but only at line start.
                i = if html_close_comment_at(src, s, module, cpu) {
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
    *substitutions = subs;
}
