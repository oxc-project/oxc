use crate::comment_meta;
use crate::error::diag_code;
use crate::lanes::Lanes;
use crate::opmap::OP_SLASH_EQ;
use crate::tables::{Tables, hex_val, is_digit, is_id_start, is_word, is_ws};

use super::bitmap::{bm_clear_range, bm_next0, bm_set1};
use super::find::{
    find_jsx_tag, find_jsx_text, find_line_terminator, find_opener, find_opener_jsx5,
    find_opener_jsx7, find_opener6, find1, find2, scan_block_comment, scan_line_comment,
    scan_quoted, scan_regex, scan_tmpl_text,
};
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
use super::find::{load256, mm, veq};
use super::regex_div::{bm_prev_sig, prev_is_regex};
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
    start: u32,
    name_s: u32,
}
/// Stamp a single-byte JSX-structural punct: set its final `kind` and clear
/// its `opch` bit so `coalesce` cannot re-fuse it (`<div>=` into `>=`).
#[inline(always)]
unsafe fn jsx_punct(kind: *mut u8, opch: *mut u64, off: usize, k: u8) {
    *kind.add(off) = k;
    *opch.add(off >> 6) &= !(1u64 << (off & 63));
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum AngleVerdict {
    TypeParams,
    Jsx,
    Ambiguous { gt: usize, lp: usize },
}

/// JSXIdentifier admits `-`, so `data-x` and `aria-label` are one name token
/// where JS would read three. Fuse every hyphen in `[a, b)` into the run
/// before it: drop its token start and its `opch` bit (so `coalesce` cannot
/// read it as an operator), plus the token start of the run that follows.
///
/// The caller only ever passes name/attribute regions: strings and `{}`
/// containers are consumed whole before the next region begins, so a hyphen
/// reached here is never a minus.
#[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
#[inline(always)]
unsafe fn jsx_glue_hyphens(
    src: *const u8,
    n: usize,
    st: *mut u64,
    opch: *mut u64,
    word: *const u64,
    a: usize,
    b: usize,
) {
    let mut i = a;
    let mut last = usize::MAX;
    while i < b {
        let mut m = mm(veq(load256(src, i), b'-'));
        let rem = b - i;
        if rem < 32 {
            m &= (1u32 << rem) - 1;
        }
        while m != 0 {
            let h = i + m.trailing_zeros() as usize;
            m &= m - 1;
            glue_hyphen_at(n, st, opch, word, h, &mut last);
        }
        i += 32;
    }
}
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2")))]
#[inline(always)]
unsafe fn jsx_glue_hyphens(
    src: *const u8,
    n: usize,
    st: *mut u64,
    opch: *mut u64,
    word: *const u64,
    a: usize,
    b: usize,
) {
    let mut last = usize::MAX;
    let mut h = a;
    while h < b {
        if *src.add(h) == b'-' {
            glue_hyphen_at(n, st, opch, word, h, &mut last);
        }
        h += 1;
    }
}
#[inline(always)]
unsafe fn glue_hyphen_at(
    n: usize,
    st: *mut u64,
    opch: *mut u64,
    word: *const u64,
    h: usize,
    last: &mut usize,
) {
    if h == 0 || !(wordbit(word, h - 1) || *last == h - 1) {
        return;
    }
    *st.add(h >> 6) &= !(1u64 << (h & 63));
    *opch.add(h >> 6) &= !(1u64 << (h & 63));
    if h + 1 < n && wordbit(word, h + 1) {
        *st.add((h + 1) >> 6) &= !(1u64 << ((h + 1) & 63));
    }
    *last = h;
}

/// Template substitutions that can nest inside a type-argument list on a JSX
/// element name before the run stops being carved. Overflow falls back to the
/// uncarved skip, so the budget can only leave an exotic shape as it was.
const TYPE_ARG_TMPL_CAP: usize = 32;

/// `.tsx` disambiguation: at an operand-position `<IDENT...`, is this a TS
/// type-parameter list rather than a JSX element? In `.tsx` a bare `<T>` is
/// JSX (a generic arrow must be written `<T,>`), so the signals are a
/// trailing `,`, a default `=`, or an `extends` constraint. Bounded forward
/// peek; the source pad makes the look-aheads safe past `n`.
///
/// Identifier runs come off the `word` bitmap, not `is_word` on the raw
/// byte. `is_word` accepts every byte >= 0x80, so non-ASCII whitespace in
/// the head glued into the first identifier and hid the signal; `misc_pre`
/// has already cleared `word` across Unicode whitespace by the time
/// `carve_jsx` runs, so the corrected run is free to read and no byte has to
/// be re-scanned for a leading 0x80.
#[inline(always)]
unsafe fn wordbit(word: *const u64, p: usize) -> bool {
    (*word.add(p >> 6) >> (p & 63)) & 1 != 0
}
/// Byte length of the whitespace at `p` — ASCII, or the multi-byte
/// ECMAScript whitespace `misc_pre` marked as a token boundary — else 0.
#[inline(always)]
unsafe fn head_ws_len(src: &[u8], p: usize) -> usize {
    let c = src[p];
    if is_ws(c) {
        return 1;
    }
    if c >= 0x80 {
        return super::classify::unicode_ws_len(src.as_ptr(), p);
    }
    0
}
#[inline]
unsafe fn ts_angle_verdict(src: &[u8], n: usize, t: usize, word: *const u64) -> AngleVerdict {
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
    while p < n && wordbit(word, p) {
        p += 1; // first type-parameter identifier
    }
    while p < n {
        let w = head_ws_len(src, p);
        if w == 0 {
            break;
        }
        p += w;
    }
    if p >= n {
        return AngleVerdict::Jsx;
    }
    let c = src[p];
    if c == b',' || c == b'=' {
        return AngleVerdict::TypeParams; // `<T,>`  `<T,U>`  `<T = D>`
    }
    if c == b'>' {
        // `<T>(` and `<T> (` are the same type-parameter list, so the `(`
        // has to be found across trivia. Bounded: a JSX element open walks
        // its own indent run and stops at the first non-space, and running
        // out of budget yields `Jsx`, which is what this arm answered
        // before, so the bound can only leave a rare shape unresolved.
        let mut q = p + 1;
        while q < n && is_ws(src[q]) {
            q += 1;
        }
        let v = if q < n && src[q] == b'(' {
            AngleVerdict::Ambiguous { gt: p, lp: q }
        } else {
            AngleVerdict::Jsx
        };
        return v;
    }
    // `extends` is also a legal JSX attribute name; it signals a generic only
    // as a full word not followed by `=` (attr value) or `>` (boolean attr).
    if n - p >= 7 && &src[p..p + 7] == b"extends" && !wordbit(word, p + 7) {
        let mut qq = p + 7;
        while qq < n && is_ws(src[qq]) {
            qq += 1;
        }
        let d = if qq < n { src[qq] } else { 0 };
        if d == b'/' && !matches!(if qq + 1 < n { src[qq + 1] } else { 0 }, b'*' | b'/') {
            return AngleVerdict::Jsx;
        }
        let v = if d == b'=' || d == b'>' { AngleVerdict::Jsx } else { AngleVerdict::TypeParams };
        return v;
    }
    AngleVerdict::Jsx
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
                    jsx_punct(kind, opch, s, crate::token::TokenKind::Lt as u8);
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
                                jsx_punct(kind, opch, q, crate::token::TokenKind::Lt as u8);
                                p = q + 1;
                            }
                            b'>' => {
                                if !(q > 0 && *src.add(q - 1) == b'=') {
                                    depth -= 1;
                                    jsx_punct(kind, opch, q, crate::token::TokenKind::Gt as u8);
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
                    *opch.add(s >> 6) &= !(1u64 << (s & 63));
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
                        *opch.add(s >> 6) &= !(1u64 << (s & 63));
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
unsafe fn jsx_name_end(src: *const u8, n: usize, mut i: usize) -> usize {
    while i < n && is_jsx_name_byte(*src.add(i)) {
        i += 1;
    }
    i
}

unsafe fn jsx_skip_trivia(src: *const u8, n: usize, mut i: usize) -> usize {
    loop {
        if i >= n {
            return n;
        }
        let c = *src.add(i);
        if is_ws(c) {
            i += 1;
            continue;
        }
        if c == b'/' && i + 1 < n {
            match *src.add(i + 1) {
                b'*' => {
                    let e = scan_block_comment(src, n, i + 2).0;
                    if e >= n {
                        return n;
                    }
                    i = e + 1;
                    continue;
                }
                b'/' => {
                    i = find_line_terminator(src, n, i + 2);
                    continue;
                }
                _ => {}
            }
        }
        if c >= 0x80 {
            let w = super::classify::unicode_ws_len(src, i);
            if w != 0 {
                i += w;
                continue;
            }
        }
        return i;
    }
}

#[inline(always)]
fn is_jsx_name_byte(c: u8) -> bool {
    is_word(c) || matches!(c, b'.' | b':' | b'-')
}

#[inline]
unsafe fn jsx_names_equal_fast(
    src: *const u8,
    word: *const u64,
    n: usize,
    a: usize,
    b: usize,
    lim_b: usize,
) -> bool {
    let e = bm_next0(word, a, n);
    let len = e - a;
    let ce = *src.add(e);
    if len <= 8
        && !matches!(ce, b'.' | b':' | b'-')
        && !(is_ws(ce) && !is_word(*src.add(e + 1)) && jsx_name_continues_after(src, n, e))
    {
        let x = core::ptr::read_unaligned(src.add(a) as *const u64);
        let y = core::ptr::read_unaligned(src.add(b) as *const u64);
        let mask = if len == 8 { !0u64 } else { (1u64 << (len * 8)) - 1 };
        if (x ^ y) & mask != 0 {
            return false;
        }
        return b + len >= lim_b || !is_jsx_name_byte(*src.add(b + len));
    }
    jsx_names_equal(src, n, a, b, lim_b)
}

unsafe fn jsx_name_continues_after(src: *const u8, lim: usize, i: usize) -> bool {
    let t = jsx_skip_trivia(src, lim, i);
    t < lim && matches!(*src.add(t), b'.' | b':')
}

unsafe fn jsx_name_next(src: *const u8, i: usize, lim: usize, after_sep: bool) -> usize {
    if i >= lim {
        return lim;
    }
    let c = *src.add(i);
    if is_jsx_name_byte(c) || !(is_ws(c) || c == b'/') {
        return i;
    }
    let t = jsx_skip_trivia(src, lim, i);
    if after_sep || (t < lim && matches!(*src.add(t), b'.' | b':')) { t } else { i }
}

unsafe fn jsx_names_equal(src: *const u8, n: usize, a: usize, b: usize, lim_b: usize) -> bool {
    let mut i = a;
    let mut j = b;
    let mut after_sep = false;
    loop {
        i = jsx_name_next(src, i, n, after_sep);
        j = jsx_name_next(src, j, lim_b, after_sep);
        let x = if i < n { *src.add(i) } else { 0 };
        let y = if j < lim_b { *src.add(j) } else { 0 };
        let xn = is_jsx_name_byte(x);
        let yn = is_jsx_name_byte(y);
        if !xn && !yn {
            return true;
        }
        if x != y {
            return false;
        }
        after_sep = x == b'.' || x == b':';
        i += 1;
        j += 1;
    }
}

const FN_TYPE_SCAN_CAP: usize = 1 << 16;
const FN_TYPE_TMPL_DEPTH: u32 = 8;

unsafe fn skip_quoted_type(src: *const u8, lim: usize, i: usize, q: u8) -> Option<usize> {
    let mut j = i + 1;
    loop {
        if j >= lim {
            return None;
        }
        let d = *src.add(j);
        if d == b'\\' {
            j += 2;
            continue;
        }
        if d == q {
            return Some(j + 1);
        }
        if d == b'\n' || d == b'\r' {
            return None;
        }
        j += 1;
    }
}

unsafe fn skip_template_type(src: *const u8, lim: usize, i: usize, depth: u32) -> Option<usize> {
    if depth > FN_TYPE_TMPL_DEPTH {
        return None;
    }
    let mut j = i + 1;
    let mut subs = [0u32; 8];
    let mut nsub = 0usize;
    loop {
        if j >= lim {
            return None;
        }
        let d = *src.add(j);
        if nsub == 0 {
            match d {
                b'\\' => j += 2,
                b'`' => return Some(j + 1),
                b'$' if *src.add(j + 1) == b'{' => {
                    if nsub == subs.len() {
                        return None;
                    }
                    subs[nsub] = 0;
                    nsub += 1;
                    j += 2;
                }
                _ => j += 1,
            }
        } else {
            match d {
                b'{' => {
                    subs[nsub - 1] += 1;
                    j += 1;
                }
                b'}' => {
                    if subs[nsub - 1] == 0 {
                        nsub -= 1;
                    } else {
                        subs[nsub - 1] -= 1;
                    }
                    j += 1;
                }
                b'"' | b'\'' => j = skip_quoted_type(src, lim, j, d)?,
                b'`' => j = skip_template_type(src, lim, j, depth + 1)?,
                _ => j += 1,
            }
        }
    }
}

unsafe fn generic_fn_type_after(src: *const u8, n: usize, lp: usize) -> bool {
    let lim = (lp + FN_TYPE_SCAN_CAP).min(n);
    let mut depth: i32 = 0;
    let mut i = lp;
    loop {
        if i >= lim {
            return false;
        }
        let c = *src.add(i);
        match c {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    i += 1;
                    break;
                }
            }
            b'"' | b'\'' => {
                let Some(e) = skip_quoted_type(src, lim, i, c) else {
                    return false;
                };
                i = e;
                continue;
            }
            b'`' => {
                let Some(e) = skip_template_type(src, lim, i, 0) else {
                    return false;
                };
                i = e;
                continue;
            }
            b'/' => match *src.add(i + 1) {
                b'*' => {
                    let e = scan_block_comment(src, n, i + 2).0;
                    if e >= n {
                        return false;
                    }
                    i = e;
                }
                b'/' => {
                    i = find_line_terminator(src, n, i + 2);
                    continue;
                }
                _ => return false,
            },
            _ => {}
        }
        i += 1;
    }
    loop {
        if i >= lim {
            return false;
        }
        let c = *src.add(i);
        if is_ws(c) {
            i += 1;
            continue;
        }
        if c == b'/' {
            match *src.add(i + 1) {
                b'*' => {
                    let e = scan_block_comment(src, n, i + 2).0;
                    if e >= n {
                        return false;
                    }
                    i = e + 1;
                    continue;
                }
                b'/' => {
                    i = find_line_terminator(src, n, i + 2);
                    continue;
                }
                _ => return false,
            }
        }
        if c >= 0x80 {
            let w = super::classify::unicode_ws_len(src, i);
            if w != 0 {
                i += w;
                continue;
            }
        }
        return c == b'=' && *src.add(i + 1) == b'>';
    }
}

#[inline]
unsafe fn jsx_over_type_params(
    t: &Tables,
    src: *const u8,
    srcs: &[u8],
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    word: *const u64,
    n: usize,
    lt: usize,
    tpos: usize,
    ts: bool,
    lanes: &mut Lanes,
) -> bool {
    if !ts {
        return true;
    }
    match ts_angle_verdict(srcs, n, tpos, word) {
        AngleVerdict::TypeParams => false,
        AngleVerdict::Jsx => true,
        AngleVerdict::Ambiguous { gt, lp } => {
            jsx_ambiguous_site(t, src, st, opch, kind, n, lt, gt, lp, lanes)
        }
    }
}

#[inline(never)]
unsafe fn jsx_ambiguous_site(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    lt: usize,
    gt: usize,
    lp: usize,
    lanes: &mut Lanes,
) -> bool {
    if super::regex_div::ts_type_region_open(t, src, st, opch, kind, n, lt) {
        return false;
    }
    if super::regex_div::type_parameter_list_head(t, src, st, opch, kind, n, lt) {
        return false;
    }
    if generic_fn_type_after(src, n, lp) {
        if super::regex_div::jsx_site_is_expression(t, src, st, opch, kind, n, lt) {
            lanes.push_diag(lt as u32, (gt + 1 - lt) as u32, diag_code::UNTERMINATED_JSX_ELEMENT);
        }
        return false;
    }
    true
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
