use crate::{
    error::diag_code,
    lanes::Lanes,
    tables::{Tables, is_digit, is_id_start, is_word, is_ws},
    token::TokenKind,
};

use super::super::{
    HASHBANG, JEND, JSX_LT, JTEXT, STR, TMPL_HEAD, TMPL_MIDDLE, TMPL_NOSUB, TMPL_TAIL,
    bitmap::{bm_clear, bm_clear_range, bm_set},
    disambiguate::{bm_prev_sig, prev_is_regex},
    find::{
        find_jsx_tag, find_jsx_text, find_line_terminator, find_opener, find_opener_jsx5,
        find_opener_jsx7, find_opener6, find1, find2,
    },
    scan::scan_block_comment,
};

use super::common::{
    lex_block_comment, lex_line_comment, lex_slash, lex_string, lex_template_segment,
    skip_unicode_brace_escape,
};

mod angle_brackets;
mod hyphens;
mod names;
use angle_brackets::jsx_over_type_params;
use hyphens::jsx_glue_hyphens;
use names::{jsx_name_end, jsx_names_equal_fast, jsx_skip_trivia};

/// Template substitutions that can nest inside a type-argument list on a JSX
/// element name before the run stops being carved. Overflow falls back to the
/// uncarved skip, so the budget can only leave an exotic shape as it was.
const TYPE_ARG_TMPL_CAP: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq)]
enum JMode {
    Js,
    Tag,
    Text,
}

#[derive(Clone, Copy)]
struct JFrame {
    kind: JFrameKind,
    /// Mode to restore when this frame pops.
    parent: JMode,
    /// Nested-brace counter (TemplateSub / JsxCont only).
    depth: u32,
    start: u32,
    name_s: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum JFrameKind {
    /// `${...}` of a split template literal (mode JS; `depth` counts nested `{}`).
    TemplateSub,
    /// `{...}` JSX expression container (mode JS; `depth` counts nested `{}`).
    JsxCont,
    /// A `<...>` opening tag being lexed (mode TAG).
    JsxTag,
    /// An opened element whose children are being lexed (mode TEXT).
    JsxElem,
}

/// JSX-aware carve: a 3-mode (JS / TAG / TEXT) pushdown over a single frame
/// stack, emitting JSX_LT / JEND / JTEXT and raw (no-escape) attribute
/// strings. All JSX logic lives here; `classify` is untouched.
///
/// Unlike `carve`, takes `digit`/`dot`/`kwinit` as `*mut`: a JTEXT run's
/// start byte keeps its `st` bit yet may be a digit/keyword/operator char,
/// so those bits are cleared there to keep `coalesce` and `keywords` from
/// re-interpreting it.
pub(super) unsafe fn carve_jsx(
    t: &Tables,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    opch: *mut u64,
    word: *const u64,
    digit: *mut u64,
    dot: *mut u64,
    kwinit: *mut u64,
    ts: bool,
    lanes: &mut Lanes,
) {
    let src = srcs.as_ptr();
    let mut stack: Vec<JFrame> = Vec::with_capacity(64);
    let mut mode = JMode::Js;
    let mut text_start = 0usize;
    let mut i = 0usize;
    if n >= 2 && *src == b'#' && *src.add(1) == b'!' {
        let end = find_line_terminator(src, n, 2);
        *kind = HASHBANG;
        bm_clear_range(st, 1, end - 1);
        if end < n {
            bm_set(st, end);
        }
        i = end;
    }
    loop {
        match mode {
            JMode::Js => {
                let in_brace = stack.last().is_some_and(|f| {
                    matches!(f.kind, JFrameKind::TemplateSub | JFrameKind::JsxCont)
                });
                let s = if in_brace {
                    find_opener_jsx7(src, n, i)
                } else {
                    find_opener_jsx5(src, n, i)
                };
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
                            src, srcs, n, st, kind, s, TMPL_HEAD, TMPL_NOSUB, lanes,
                        );
                        if opened_sub {
                            stack.push(JFrame {
                                kind: JFrameKind::TemplateSub,
                                parent: JMode::Js,
                                depth: 0,
                                start: s as u32,
                                name_s: 0,
                            });
                        }
                        i = end;
                    }
                    b'{' => {
                        if s >= 2 && *src.add(s - 1) == b'u' && *src.add(s - 2) == b'\\' {
                            i = skip_unicode_brace_escape(src, n, s);
                        } else {
                            if let Some(f) = stack.last_mut() {
                                f.depth += 1;
                            }
                            i = s + 1;
                        }
                    }
                    b'}' => {
                        let top_kind = stack.last().map(|f| f.kind);
                        let top_depth = stack.last().map_or(0, |f| f.depth);
                        if top_depth > 0 {
                            if let Some(f) = stack.last_mut() {
                                f.depth -= 1;
                            }
                            i = s + 1;
                        } else if top_kind == Some(JFrameKind::TemplateSub) {
                            stack.pop();
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
                                stack.push(JFrame {
                                    kind: JFrameKind::TemplateSub,
                                    parent: JMode::Js,
                                    depth: 0,
                                    start: s as u32,
                                    name_s: 0,
                                });
                            }
                            i = end;
                        } else if top_kind == Some(JFrameKind::JsxCont) {
                            let parent = stack.last().map_or(JMode::Js, |f| f.parent);
                            stack.pop();
                            mode = parent;
                            if mode == JMode::Text {
                                text_start = s + 1;
                            }
                            i = s + 1;
                        } else {
                            i = s + 1;
                        }
                    }
                    b'/' => {
                        i = lex_slash(t, src, srcs, n, st, kind, opch, word, digit, ts, s, lanes);
                    }
                    b'<' => {
                        let c1 = if s + 1 < n { *src.add(s + 1) } else { 0 };
                        if c1 == b'<' {
                            // `<<` shift: skip both, or the second `<` would
                            // read the first as an operand preceder.
                            i = s + 2;
                        } else if c1 == b'=' || is_digit(c1) {
                            // `<=` / `a<5`: leave for coalesce.
                            i = s + 1;
                        } else if prev_is_regex(
                            t,
                            src,
                            st,
                            kind,
                            word,
                            digit,
                            n,
                            s,
                            ts,
                            lanes.module,
                        ) {
                            // Operand position: candidate JSX.
                            let mut tpos = s + 1;
                            loop {
                                while tpos < n && is_ws(*src.add(tpos)) {
                                    tpos += 1;
                                }
                                if tpos + 1 >= n || *src.add(tpos) != b'/' {
                                    break;
                                }
                                match *src.add(tpos + 1) {
                                    b'*' => {
                                        let e = scan_block_comment(src, n, tpos + 2).0;
                                        if e >= n {
                                            break;
                                        }
                                        tpos = e + 1;
                                    }
                                    b'/' => tpos = find_line_terminator(src, n, tpos + 2),
                                    _ => break,
                                }
                            }
                            let tc = if tpos < n { *src.add(tpos) } else { 0 };
                            if tc == b'>' {
                                // fragment `<>`
                                jsx_punct(kind, opch, s, JSX_LT);
                                stack.push(JFrame {
                                    kind: JFrameKind::JsxTag,
                                    parent: JMode::Js,
                                    depth: 0,
                                    start: s as u32,
                                    name_s: tpos as u32,
                                });
                                mode = JMode::Tag;
                            } else if is_id_start(tc)
                                && jsx_over_type_params(
                                    t, src, srcs, st, opch, kind, word, n, s, tpos, ts, lanes,
                                )
                            {
                                // Element — unless `.tsx` says this is a
                                // type-parameter list, which stays a less-than.
                                jsx_punct(kind, opch, s, JSX_LT);
                                stack.push(JFrame {
                                    kind: JFrameKind::JsxTag,
                                    parent: JMode::Js,
                                    depth: 0,
                                    start: s as u32,
                                    name_s: tpos as u32,
                                });
                                mode = JMode::Tag;
                            }
                            i = s + 1;
                        } else {
                            // Operator position: less-than.
                            i = s + 1;
                        }
                    }
                    _ => {
                        i = s + 1;
                    }
                }
            }
            JMode::Tag => {
                let s = find_jsx_tag(src, n, i);
                if s >= n {
                    break;
                }
                // Tag/attr names that spell reserved words must stay IDENT:
                // clear their `kwinit` so the `keywords` pass skips them.
                if s > i {
                    bm_clear_range(kwinit, i, s - 1);
                    jsx_glue_hyphens(src, n, st, opch, word, i, s);
                }
                let c = *src.add(s);
                if c == b'<' {
                    let q = bm_prev_sig(st, kind, s);
                    if q >= 0 && *src.add(q as usize) == b'=' {
                        let tpos = jsx_skip_trivia(src, n, s + 1);
                        jsx_punct(kind, opch, s, JSX_LT);
                        stack.push(JFrame {
                            kind: JFrameKind::JsxTag,
                            parent: JMode::Tag,
                            depth: 0,
                            start: s as u32,
                            name_s: tpos as u32,
                        });
                        i = s + 1;
                        continue;
                    }
                }
                // `.tsx`: a type-argument list on the element
                // (`<Box<number> ...>`) puts a balanced `<...>` run inside
                // the opening tag, and its inner `>` must not close the tag.
                // Literals are carved as the run is crossed rather than
                // skipped over, so a string or template type argument gets
                // its own token and a `<`/`>` written inside one cannot
                // desync the depth count. Cold: a type-argument list on an
                // element name.
                if ts && c == b'<' {
                    let mut depth = 1i32;
                    let mut p = s + 1;
                    jsx_punct(kind, opch, s, TokenKind::Lt as u8);
                    // Open template substitutions, each counting its own
                    // nested braces — the same shape `carve` keeps in its
                    // `depth` vector, sized so this path allocates nothing.
                    let mut sub = [0u32; TYPE_ARG_TMPL_CAP];
                    let mut nsub = 0usize;
                    while p < n && depth != 0 {
                        let q = if nsub != 0 {
                            find_opener6(src, n, p)
                        } else {
                            find_opener(src, n, p)
                        };
                        if q >= n {
                            p = n;
                            break;
                        }
                        match *src.add(q) {
                            b'<' => {
                                depth += 1;
                                jsx_punct(kind, opch, q, TokenKind::Lt as u8);
                                p = q + 1;
                            }
                            b'>' => {
                                if !(q > 0 && *src.add(q - 1) == b'=') {
                                    depth -= 1;
                                    jsx_punct(kind, opch, q, TokenKind::Gt as u8);
                                }
                                p = q + 1;
                            }
                            b'"' | b'\'' => {
                                p = lex_string(src, srcs, n, st, kind, q, *src.add(q), lanes);
                            }
                            b'`' => {
                                let (end, opened) = lex_template_segment(
                                    src, srcs, n, st, kind, q, TMPL_HEAD, TMPL_NOSUB, lanes,
                                );
                                p = end;
                                if opened {
                                    // Past the nesting budget the rest of the
                                    // run stays uncarved, which is what this
                                    // whole arm used to do.
                                    if nsub == TYPE_ARG_TMPL_CAP {
                                        break;
                                    }
                                    sub[nsub] = 0;
                                    nsub += 1;
                                }
                            }
                            // Braces only reach here through `find_opener6`,
                            // which is only selected while a substitution is
                            // open — the guards say so rather than leaving it
                            // to the finder choice.
                            b'{' if nsub != 0 => {
                                sub[nsub - 1] += 1;
                                p = q + 1;
                            }
                            b'}' if nsub != 0 => {
                                if sub[nsub - 1] != 0 {
                                    sub[nsub - 1] -= 1;
                                    p = q + 1;
                                    continue;
                                }
                                nsub -= 1;
                                let (end, opened) = lex_template_segment(
                                    src,
                                    srcs,
                                    n,
                                    st,
                                    kind,
                                    q,
                                    TMPL_MIDDLE,
                                    TMPL_TAIL,
                                    lanes,
                                );
                                p = end;
                                if opened {
                                    if nsub == TYPE_ARG_TMPL_CAP {
                                        break;
                                    }
                                    sub[nsub] = 0;
                                    nsub += 1;
                                }
                            }
                            // A comment inside a type-argument list is
                            // trivia; a lone `/` cannot start a type, so it
                            // is left alone.
                            b'/' => {
                                p = match *src.add(q + 1) {
                                    b'*' => lex_block_comment(src, srcs, n, st, kind, q, lanes),
                                    b'/' => lex_line_comment(src, srcs, n, st, kind, q, lanes),
                                    _ => q + 1,
                                };
                            }
                            _ => p = q + 1,
                        }
                    }
                    // Later passes can still emit spurious diagnostics from
                    // the run; record the span so drain-time filtering drops
                    // them.
                    lanes.diag_suppress.push((s as u32, p as u32));
                    i = p;
                    continue;
                }
                match c {
                    b'"' | b'\'' => {
                        // JSX attribute string: no escapes, ends at next quote.
                        let e = find1(src, n, s + 1, c);
                        if e >= n {
                            lanes.push_diag(
                                s as u32,
                                (n - s) as u32,
                                diag_code::UNTERMINATED_STRING,
                            );
                        }
                        let end = if e < n { e + 1 } else { n };
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
                        lanes.push_string_raw(srcs, s + 1, be);
                        i = end;
                    }
                    b'{' => {
                        stack.push(JFrame {
                            kind: JFrameKind::JsxCont,
                            parent: JMode::Tag,
                            depth: 0,
                            start: s as u32,
                            name_s: 0,
                        });
                        mode = JMode::Js;
                        i = s + 1;
                    }
                    b'/' => {
                        let d = if s + 1 < n { *src.add(s + 1) } else { 0 };
                        if d == b'*' {
                            // comment inside a tag = whitespace
                            i = lex_block_comment(src, srcs, n, st, kind, s, lanes);
                        } else if d == b'/' {
                            i = lex_line_comment(src, srcs, n, st, kind, s, lanes);
                        } else {
                            let gp = if d == b'>' {
                                Some(s + 1)
                            } else if is_ws(d) {
                                let mut w = s + 2;
                                while w < n && is_ws(*src.add(w)) {
                                    w += 1;
                                }
                                (w < n && *src.add(w) == b'>').then_some(w)
                            } else {
                                None
                            };
                            if let Some(gp) = gp {
                                jsx_punct(kind, opch, gp, JEND);
                                let parent = stack.last().map_or(JMode::Js, |f| f.parent);
                                stack.pop();
                                mode = parent;
                                if mode == JMode::Text {
                                    text_start = gp + 1;
                                }
                                i = gp + 1;
                            } else {
                                // lone `/` (malformed) — stays a slash.
                                i = s + 1;
                            }
                        }
                    }
                    b'>' => {
                        // opening tag ends; children begin
                        if let Some(f @ JFrame { kind: JFrameKind::JsxTag, .. }) = stack.last_mut()
                        {
                            f.kind = JFrameKind::JsxElem;
                        }
                        jsx_punct(kind, opch, s, TokenKind::Gt as u8);
                        mode = JMode::Text;
                        text_start = s + 1;
                        i = s + 1;
                    }
                    _ => {
                        i = s + 1;
                    }
                }
            }
            JMode::Text => {
                let s = find_jsx_text(src, n, i);
                let runend = if s < n { s } else { n };
                if runend > text_start {
                    // One JTEXT token for the run; neutralize its start byte
                    // against coalesce/keywords and clear the interior.
                    bm_set(st, text_start);
                    *kind.add(text_start) = JTEXT;
                    bm_clear(opch, text_start);
                    bm_clear(digit, text_start);
                    bm_clear(dot, text_start);
                    bm_clear(kwinit, text_start);
                    if runend > text_start + 1 {
                        bm_clear_range(st, text_start + 1, runend - 1);
                    }
                }
                if s >= n {
                    break;
                }
                let c = *src.add(s);
                if c == b'{' {
                    stack.push(JFrame {
                        kind: JFrameKind::JsxCont,
                        parent: JMode::Text,
                        depth: 0,
                        start: s as u32,
                        name_s: 0,
                    });
                    mode = JMode::Js;
                    i = s + 1;
                } else if c == b'>' || c == b'}' {
                    // A stray `>`/`}` ends the run; clear its opch so
                    // coalesce can't fuse adjacent strays into `>>`.
                    bm_clear(opch, s);
                    lanes.push_diag(s as u32, 1, diag_code::JSX_TEXT_INVALID_CHARACTER);
                    text_start = s + 1;
                    i = s + 1;
                } else {
                    // c == '<'
                    let c1 = if s + 1 < n { *src.add(s + 1) } else { 0 };
                    let tpos = if c1 == b'/' || c1 == b'>' || is_id_start(c1) {
                        s + 1
                    } else {
                        jsx_skip_trivia(src, n, s + 1)
                    };
                    let tc = if tpos < n { *src.add(tpos) } else { 0 };
                    if tc == b'/' {
                        // closing tag `</name>` or `</>`; the name stays IDENT
                        let mut gp = tpos + 1;
                        loop {
                            gp = find2(src, n, gp, b'>', b'/');
                            if gp >= n || *src.add(gp) == b'>' {
                                break;
                            }
                            gp = match *src.add(gp + 1) {
                                b'*' => lex_block_comment(src, srcs, n, st, kind, gp, lanes),
                                b'/' => lex_line_comment(src, srcs, n, st, kind, gp, lanes),
                                _ => gp + 1,
                            };
                        }
                        if gp > tpos + 1 {
                            bm_clear_range(kwinit, tpos + 1, gp - 1);
                            // `</data-x>`: the name region holds no
                            // expression container, so hyphens in it are
                            // always part of the JSXIdentifier.
                            jsx_glue_hyphens(src, n, st, opch, word, tpos + 1, gp);
                        }
                        jsx_punct(kind, opch, s, JSX_LT);
                        if gp < n {
                            jsx_punct(kind, opch, gp, JEND);
                        }
                        let after = if gp < n { gp + 1 } else { n };
                        if gp >= n {
                            lanes.push_diag(
                                s as u32,
                                (n - s) as u32,
                                diag_code::UNTERMINATED_JSX_TAG,
                            );
                        } else if let Some(f) = stack.last() {
                            let c2 = *src.add(tpos + 1);
                            let cs = if is_word(c2) || c2 == b'>' {
                                tpos + 1
                            } else {
                                jsx_skip_trivia(src, gp, tpos + 1)
                            };
                            if !jsx_names_equal_fast(src, word, n, f.name_s as usize, cs, gp) {
                                lanes.push_diag(
                                    s as u32,
                                    (after - s) as u32,
                                    diag_code::JSX_CLOSING_TAG_MISMATCH,
                                );
                            }
                        }
                        let parent = stack.last().map_or(JMode::Js, |f| f.parent);
                        stack.pop();
                        mode = parent;
                        if mode == JMode::Text {
                            text_start = after;
                        }
                        i = after;
                    } else if tc == b'>' || is_id_start(tc) {
                        // child element / fragment
                        jsx_punct(kind, opch, s, JSX_LT);
                        stack.push(JFrame {
                            kind: JFrameKind::JsxTag,
                            parent: JMode::Text,
                            depth: 0,
                            start: s as u32,
                            name_s: tpos as u32,
                        });
                        mode = JMode::Tag;
                        i = s + 1;
                    } else {
                        // malformed lone `<` in text — clear opch, no `<<` fusion
                        bm_clear(opch, s);
                        text_start = s + 1;
                        i = s + 1;
                    }
                }
            }
        }
    }
    if let Some(f) = stack.last() {
        match f.kind {
            JFrameKind::JsxTag => {
                lanes.push_diag(f.start, n as u32 - f.start, diag_code::UNTERMINATED_JSX_TAG);
            }
            JFrameKind::JsxElem => {
                let ne = jsx_name_end(src, n, f.name_s as usize) as u32;
                lanes.push_diag(f.start, ne - f.start, diag_code::UNTERMINATED_JSX_ELEMENT);
            }
            JFrameKind::JsxCont => {
                lanes.push_diag(f.start, n as u32 - f.start, diag_code::UNTERMINATED_JSX_CONTAINER);
            }
            JFrameKind::TemplateSub => {}
        }
    }
}

/// Stamp a single-byte JSX-structural punct: set its final `kind` and clear
/// its `opch` bit so `coalesce` cannot re-fuse it (`<div>=` into `>=`).
#[inline(always)]
unsafe fn jsx_punct(kind: *mut u8, opch: *mut u64, off: usize, k: u8) {
    *kind.add(off) = k;
    bm_clear(opch, off);
}
