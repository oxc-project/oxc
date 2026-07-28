use crate::comment_meta;
use crate::error::diag_code;
use crate::lanes::Lanes;
use crate::opmap::OP_SLASH_EQ;
use crate::tables::{Tables, hex_val, is_digit, is_id_start, is_word, is_ws};

use super::bitmap::{bm_clear_range, bm_next0, bm_set1};
use super::find::{
    find_jsx_tag, find_jsx_tag_ts, find_jsx_text, find_line_terminator, find_opener,
    find_opener_jsx5, find_opener_jsx7, find_opener6, find1, find2, scan_block_comment,
    scan_line_comment, scan_quoted, scan_regex, scan_tmpl_text,
};
use super::regex_div::prev_is_regex;
use super::{
    BCOM, HASHBANG, JEND, JSX_LT, JTEXT, LCOM, REGEX, STR, TMPL_HEAD, TMPL_MIDDLE, TMPL_NOSUB,
    TMPL_TAIL,
};

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

/// Lex the string literal opening at `s`. Returns the resume index. Shared
/// by `carve` and `carve_jsx` JS mode.
#[inline(always)]
unsafe fn lex_string(
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
        lanes.push_diag(s as u32, (n - s) as u32, diag_code::UNTERMINATED_STRING);
    }
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
    lanes.push_string(srcs, s + 1, be);
    end
}

/// Lex the template text segment starting at `s` (a backtick or a
/// substitution-closing `}`): `head_kind` if it ends in `${`, `flat_kind` if
/// it closes or runs to EOF. Returns `(resume index, substitution opened)`;
/// the caller pushes its own nesting frame.
#[inline(always)]
unsafe fn lex_template_segment(
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
        lanes.push_diag(s as u32, (end - s) as u32, diag_code::UNTERMINATED_TEMPLATE);
    }
    *kind.add(s) = if term == 2 { head_kind } else { flat_kind };
    if end > s + 1 {
        bm_clear_range(st, s + 1, end - 1);
    }
    lanes.push_template(srcs, s + 1, end - [0usize, 1, 2][term as usize]);
    (end, term == 2)
}

/// Lex the `//` line comment at `s`. Returns the resume index.
#[inline(always)]
unsafe fn lex_line_comment(
    src: *const u8,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    s: usize,
    lanes: &mut Lanes,
) -> usize {
    let (end, lic_q) = scan_line_comment(src, n, s + 2);
    *kind.add(s) = LCOM;
    if end > s + 1 {
        bm_clear_range(st, s + 1, end - 1);
    }
    if end < n {
        bm_set1(st, end);
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
unsafe fn lex_block_comment(
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
        lanes.push_diag(s as u32, (n - s) as u32, diag_code::UNTERMINATED_BLOCK_COMMENT);
    }
    *kind.add(s) = BCOM;
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
        lanes.push_diag(s as u32, (nl_at + 1 - s) as u32, diag_code::LINE_TERMINATOR_IN_REGEXP);
    } else if e >= n {
        lanes.push_diag(s as u32, (n - s) as u32, diag_code::UNTERMINATED_REGEXP);
    }
    let mut end = fs;
    if end < n && (*word.add(end >> 6) >> (end & 63)) & 1 != 0 {
        end = bm_next0(word, end, n);
    }
    *kind.add(s) = REGEX;
    if end > s + 1 {
        bm_clear_range(st, s + 1, end - 1);
    }
    lanes.push_regex_flags(srcs, fs, end);
    end
}

/// The `/` dispatch shared by `carve` and `carve_jsx` JS mode: line comment,
/// block comment, regex, `/=`, or a bare slash left for `coalesce`.
#[inline(always)]
unsafe fn lex_slash(
    t: &Tables,
    src: *const u8,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    opch: *mut u64,
    word: *const u64,
    digit: *const u64,
    ts: bool,
    s: usize,
    lanes: &mut Lanes,
) -> usize {
    let d = if s + 1 < n { *src.add(s + 1) } else { 0 };
    if d == b'/' {
        lex_line_comment(src, srcs, n, st, kind, s, lanes)
    } else if d == b'*' {
        lex_block_comment(src, srcs, n, st, kind, s, lanes)
    } else if prev_is_regex(t, src, st, kind, word, digit, n, s, ts, lanes.module) {
        lex_regex(src, srcs, n, st, kind, word, s, lanes)
    } else if s + 1 < n && *src.add(s + 1) == b'=' {
        // `/=`: absorb the `=`.
        *kind.add(s) = OP_SLASH_EQ;
        *st.add((s + 1) >> 6) &= !(1u64 << ((s + 1) & 63));
        *opch.add((s + 1) >> 6) &= !(1u64 << ((s + 1) & 63));
        s + 2
    } else {
        s + 1
    }
}

/// Skip the payload of a brace-form unicode escape whose `{` is at `s`: the
/// escape was already joined into its identifier by `misc_pre`, and its
/// braces must not count toward substitution nesting.
#[inline(always)]
unsafe fn skip_unicode_brace_escape(src: *const u8, n: usize, s: usize) -> usize {
    let mut j = s + 1;
    while j < n && hex_val(*src.add(j)) != 255 {
        j += 1;
    }
    if j < n && *src.add(j) == b'}' {
        j += 1;
    }
    j
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
                        jsx_punct(kind, opch, s, crate::token::token_kind::GT);
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
pub(super) unsafe fn carve(
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
