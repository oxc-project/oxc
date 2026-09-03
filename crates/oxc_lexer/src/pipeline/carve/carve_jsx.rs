use crate::{
    lanes::Lanes,
    tables::{Tables, is_digit, is_id_start, is_word, is_ws},
};

use super::super::{
    HASHBANG, JEND, JSX_LT, JTEXT, STR, TMPL_HEAD, TMPL_MIDDLE, TMPL_NOSUB, TMPL_TAIL,
    bitmap::{bm_clear_range, bm_set1},
    find::{
        find_jsx_tag, find_jsx_tag_ts, find_jsx_text, find_line_terminator, find_opener_jsx5,
        find_opener_jsx7, find1, find2,
    },
    regex_div::prev_is_regex,
};

use super::common::{
    lex_block_comment, lex_line_comment, lex_slash, lex_string, lex_template_segment,
    skip_unicode_brace_escape,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum JMode {
    Js,
    Tag,
    Text,
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

#[derive(Clone, Copy)]
struct JFrame {
    kind: JFrameKind,
    /// Mode to restore when this frame pops.
    parent: JMode,
    /// Nested-brace counter (TemplateSub / JsxCont only).
    depth: u32,
}

/// JSX-aware carve: a 3-mode (JS / TAG / TEXT) pushdown over a single frame
/// stack, emitting JSX_LT / JEND / JTEXT and raw (no-escape) attribute
/// strings. All JSX logic lives here; `classify` is untouched.
///
/// Unlike `carve`, takes `digit`/`dot`/`kwinit` as `*mut`: a JTEXT run's
/// start byte keeps its `st` bit yet may be a digit/keyword/operator char,
/// so those bits are cleared there to keep `coalesce` and `keywords` from
/// re-interpreting it.
pub unsafe fn carve_jsx(
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
            bm_set1(st, end);
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
                            while tpos < n && is_ws(*src.add(tpos)) {
                                tpos += 1;
                            }
                            let tc = if tpos < n { *src.add(tpos) } else { 0 };
                            if tc == b'>' {
                                // fragment `<>`
                                jsx_punct(kind, opch, s, JSX_LT);
                                stack.push(JFrame {
                                    kind: JFrameKind::JsxTag,
                                    parent: JMode::Js,
                                    depth: 0,
                                });
                                mode = JMode::Tag;
                            } else if is_id_start(tc) && !(ts && ts_is_type_params(srcs, n, tpos)) {
                                // Element — unless `.tsx` says this is a
                                // type-parameter list, which stays a less-than.
                                jsx_punct(kind, opch, s, JSX_LT);
                                stack.push(JFrame {
                                    kind: JFrameKind::JsxTag,
                                    parent: JMode::Js,
                                    depth: 0,
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
                let s = if ts { find_jsx_tag_ts(src, n, i) } else { find_jsx_tag(src, n, i) };
                if s >= n {
                    break;
                }
                // Tag/attr names that spell reserved words must stay IDENT:
                // clear their `kwinit` so the `keywords` pass skips them.
                if s > i {
                    bm_clear_range(kwinit, i, s - 1);
                }
                let c = *src.add(s);
                // `.tsx`: a type-argument list on the element
                // (`<Box<number> ...>`) puts a balanced `<...>` run inside
                // the opening tag; skip it so its inner `>` cannot close the
                // tag. The skip is content-blind — a string type-arg is not
                // carved (wrong kinds, still monotonic), and a literal
                // `<`/`>` inside one desyncs the depth count. Content-aware
                // skipping belongs to a TS type-aware round.
                if ts && c == b'<' {
                    let mut depth = 1i32;
                    let mut p = s + 1;
                    while p < n && depth != 0 {
                        let q = find2(src, n, p, b'<', b'>');
                        if q >= n {
                            p = n;
                            break;
                        }
                        depth += if *src.add(q) == b'<' { 1 } else { -1 };
                        p = q + 1;
                    }
                    // Later passes can emit spurious diagnostics from the
                    // uncarved interior; record the span so drain-time
                    // filtering drops them.
                    lanes.diag_suppress.push((s as u32, p as u32));
                    i = p;
                    continue;
                }
                match c {
                    b'"' | b'\'' => {
                        // JSX attribute string: no escapes, ends at next quote.
                        let e = find1(src, n, s + 1, c);
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
                        } else if d == b'>' {
                            // self-close `/>`
                            let gp = s + 1;
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
                    b'>' => {
                        // opening tag ends; children begin
                        if let Some(f @ JFrame { kind: JFrameKind::JsxTag, .. }) = stack.last_mut()
                        {
                            f.kind = JFrameKind::JsxElem;
                        }
                        jsx_punct(kind, opch, s, crate::token::TokenKind::Gt as u8);
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
                    bm_set1(st, text_start);
                    *kind.add(text_start) = JTEXT;
                    let w = text_start >> 6;
                    let bit = 1u64 << (text_start & 63);
                    *opch.add(w) &= !bit;
                    *digit.add(w) &= !bit;
                    *dot.add(w) &= !bit;
                    *kwinit.add(w) &= !bit;
                    if runend > text_start + 1 {
                        bm_clear_range(st, text_start + 1, runend - 1);
                    }
                }
                if s >= n {
                    break;
                }
                let c = *src.add(s);
                if c == b'{' {
                    stack.push(JFrame { kind: JFrameKind::JsxCont, parent: JMode::Text, depth: 0 });
                    mode = JMode::Js;
                    i = s + 1;
                } else if c == b'>' || c == b'}' {
                    // A stray `>`/`}` ends the run; clear its opch so
                    // coalesce can't fuse adjacent strays into `>>`.
                    *opch.add(s >> 6) &= !(1u64 << (s & 63));
                    text_start = s + 1;
                    i = s + 1;
                } else {
                    // c == '<'
                    let c1 = if s + 1 < n { *src.add(s + 1) } else { 0 };
                    if c1 == b'/' {
                        // closing tag `</name>` or `</>`; the name stays IDENT
                        let gp = find1(src, n, s + 2, b'>');
                        if gp > s + 2 {
                            bm_clear_range(kwinit, s + 2, gp - 1);
                        }
                        jsx_punct(kind, opch, s, JSX_LT);
                        if gp < n {
                            jsx_punct(kind, opch, gp, JEND);
                        }
                        let parent = stack.last().map_or(JMode::Js, |f| f.parent);
                        stack.pop();
                        mode = parent;
                        let after = if gp < n { gp + 1 } else { n };
                        if mode == JMode::Text {
                            text_start = after;
                        }
                        i = after;
                    } else if c1 == b'>' || is_id_start(c1) {
                        // child element / fragment
                        jsx_punct(kind, opch, s, JSX_LT);
                        stack.push(JFrame {
                            kind: JFrameKind::JsxTag,
                            parent: JMode::Text,
                            depth: 0,
                        });
                        mode = JMode::Tag;
                        i = s + 1;
                    } else {
                        // malformed lone `<` in text — clear opch, no `<<` fusion
                        *opch.add(s >> 6) &= !(1u64 << (s & 63));
                        text_start = s + 1;
                        i = s + 1;
                    }
                }
            }
        }
    }
}

/// Stamp a single-byte JSX-structural punct: set its final `kind` and clear
/// its `opch` bit so `coalesce` cannot re-fuse it (`<div>=` into `>=`).
#[inline(always)]
unsafe fn jsx_punct(kind: *mut u8, opch: *mut u64, off: usize, k: u8) {
    *kind.add(off) = k;
    *opch.add(off >> 6) &= !(1u64 << (off & 63));
}

/// `.tsx` disambiguation: at an operand-position `<IDENT...`, is this a TS
/// type-parameter list rather than a JSX element? In `.tsx` a bare `<T>` is
/// JSX (a generic arrow must be written `<T,>`), so the signals are a
/// trailing `,`, a default `=`, or an `extends` constraint. Bounded forward
/// peek; the source pad makes the look-aheads safe past `n`.
#[inline]
fn ts_is_type_params(src: &[u8], n: usize, t: usize) -> bool {
    let mut p = t;
    // optional `const` type-parameter modifier: `<const T,>`
    if n - p >= 6 && &src[p..p + 5] == b"const" && !is_word(src[p + 5]) {
        let mut qq = p + 5;
        while qq < n && is_ws(src[qq]) {
            qq += 1;
        }
        if qq < n && is_id_start(src[qq]) {
            p = qq; // `const` was a modifier; advance to the real param
        }
    }
    while p < n && is_word(src[p]) {
        p += 1; // first type-parameter identifier
    }
    while p < n && is_ws(src[p]) {
        p += 1;
    }
    if p >= n {
        return false;
    }
    let c = src[p];
    if c == b',' || c == b'=' {
        return true; // `<T,>`  `<T,U>`  `<T = D>`
    }
    // `extends` is also a legal JSX attribute name; it signals a generic only
    // as a full word not followed by `=` (attr value) or `>` (boolean attr).
    if n - p >= 7 && &src[p..p + 7] == b"extends" && !is_word(src[p + 7]) {
        let mut qq = p + 7;
        while qq < n && is_ws(src[qq]) {
            qq += 1;
        }
        let d = if qq < n { src[qq] } else { 0 };
        return !(d == b'=' || d == b'>');
    }
    false
}
