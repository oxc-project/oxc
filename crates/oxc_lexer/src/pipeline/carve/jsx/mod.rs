use std::mem;

use oxc_simd::{BASELINE, Cpu, cpu_multiversion, cpu_version};

use crate::{error::DiagCode, token::tk};

use crate::pipeline::{
    bitmap::{Bitmap, Bitmaps},
    bytes::{is_digit, is_id_start, is_word},
    disambiguate::{TokensView, not_operator_position, prev_sig},
    find::find,
    kinds::Kinds,
    lanes::Lanes,
    scan::scan_ident_esc,
    source::Source,
};

use super::{
    comments::{
        html_close_comment_at, html_open_comment_at, lex_block_comment, lex_html_close_comment,
        lex_html_open_comment, lex_line_comment,
    },
    common::{
        OPENER_BYTES, OPENER8_BYTES, find_line_terminator, lex_slash, lex_string,
        lex_template_segment, skip_unicode_brace_escape,
    },
};

mod angle_brackets;
mod hyphens;
mod names;

use angle_brackets::jsx_over_type_params;
use hyphens::jsx_glue_hyphens;
use names::{jsx_name_end, jsx_names_equal_fast, jsx_skip_trivia, jsx_skip_trivia_fast};

/// Ordered for speed, pairing `'` with `/` (see `any_of` in `find/avx2.rs`).
const OPENER_JSX5_BYTES: [u8; 5] = [b'"', b'`', b'\'', b'/', b'<'];

/// Ordered for speed, pairing `'` with `/` (see `any_of` in `find/avx2.rs`).
const OPENER_JSX7_BYTES: [u8; 7] = [b'"', b'`', b'\'', b'/', b'{', b'}', b'<'];

/// Ordered for speed, pairing `'` with `/` and `<` with `>` (see `any_of` in `find/avx2.rs`).
const JSX_TAG_BYTES: [u8; 6] = [b'"', b'{', b'\'', b'/', b'<', b'>'];

/// Ordered for speed, pairing `<` with `>` (see `any_of` in `find/avx2.rs`).
const JSX_TEXT_BYTES: [u8; 4] = [b'<', b'>', b'{', b'}'];

#[derive(Clone, Copy, PartialEq, Eq)]
enum JMode {
    Js,
    Tag,
    Text,
}

#[derive(Clone, Copy)]
pub(in crate::pipeline) struct JFrame {
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
#[cpu_multiversion]
pub(super) unsafe fn carve_jsx(
    src: Source,
    bitmaps: &Bitmaps,
    kinds: Kinds,
    ts: bool,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) {
    let Bitmaps { token_start, word, keyword_initial, op_char, digit, dot, .. } = *bitmaps;
    // Annex B B.1.3 (`-->` at line start) needs JS mode to stop at `>`, which JSX is full of.
    // The finders that do so are switched on only after a `<!--` has been lexed (the pair the
    // legacy comments come in, and `<!--` is caught free on the `<` path), so a module or a
    // JSX script without HTML comments never pays; a `-->` before any `<!--` stays operators.
    let mut html_close = false;
    // Reuse the previous lex's allocation, which matters for small inputs
    let mut stack = mem::take(&mut lanes.jsx_stack);
    stack.clear();
    let mut mode = JMode::Js;
    let mut text_start = 0usize;
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
        match mode {
            JMode::Js => {
                let in_brace = stack.last().is_some_and(|f| {
                    matches!(f.kind, JFrameKind::TemplateSub | JFrameKind::JsxCont)
                });
                // The finders that also stop at `>` are only switched on by a `<!--` in a
                // script (see `html_close`).
                let s = match (in_brace, html_close) {
                    (true, false) => find(src, i, OPENER_JSX7_BYTES, cpu),
                    (true, true) => find(src, i, OPENER8_BYTES, cpu),
                    (false, false) => find(src, i, OPENER_JSX5_BYTES, cpu),
                    (false, true) => find(src, i, OPENER_BYTES, cpu),
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
                        if s >= 2 && src[s - 1] == b'u' && src[s - 2] == b'\\' {
                            i = skip_unicode_brace_escape(src, s, cpu);
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
                                token_start,
                                kinds,
                                s,
                                tk!(TemplateMiddle),
                                tk!(TemplateTail),
                                lanes,
                                cpu,
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
                        i = lex_slash(src, bitmaps, kinds, ts, s, lanes, cpu);
                    }
                    b'<' => {
                        let c1 = if s + 1 < src.len() { src[s + 1] } else { 0 };
                        if c1 == b'!' && html_open_comment_at(src, s, lanes.module, cpu) {
                            // Annex B B.1.1: the goal, not the JSX setting, decides.
                            i = lex_html_open_comment(src, token_start, kinds, op_char, s, lanes);
                            html_close = !lanes.module;
                            continue;
                        }
                        if c1 == b'<' {
                            // `<<` shift: skip both, or the second `<` would
                            // read the first as an operand preceder.
                            i = s + 2;
                        } else if c1 == b'=' || is_digit(c1, cpu) {
                            // `<=` / `a<5`: leave for coalesce.
                            i = s + 1;
                        } else if not_operator_position(
                            &TokensView::new(
                                src,
                                bitmaps,
                                kinds,
                                ts,
                                0,
                                lanes.module,
                                &lanes.disambiguate.brackets,
                                &lanes.disambiguate.closers,
                                cpu,
                            ),
                            &mut lanes.disambiguate.walks,
                            s,
                            cpu,
                        ) {
                            // Operand position: candidate JSX. Whitespace (Unicode too) and
                            // comments may separate `<` from the name.
                            let tpos = jsx_skip_trivia_fast(src, s + 1, cpu);
                            let tc = if tpos < src.len() { src[tpos] } else { 0 };
                            if tc == b'>' {
                                // fragment `<>`
                                jsx_punct(kinds, op_char, s, tk!(JsxLt), cpu);
                                stack.push(JFrame {
                                    kind: JFrameKind::JsxTag,
                                    parent: JMode::Js,
                                    depth: 0,
                                    start: s as u32,
                                    name_s: tpos as u32,
                                });
                                mode = JMode::Tag;
                            } else if is_id_start(tc, cpu)
                                && jsx_over_type_params(
                                    src, bitmaps, kinds, s, tpos, ts, lanes, cpu,
                                )
                            {
                                // Element - unless `.tsx` says this is a
                                // type-parameter list, which stays a less-than.
                                jsx_punct(kinds, op_char, s, tk!(JsxLt), cpu);
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
                    b'>' => {
                        // Only a script's finder stops here: Annex B B.1.3 `-->` at line start.
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
            JMode::Tag => {
                let s = find(src, i, JSX_TAG_BYTES, cpu);
                if s >= src.len() {
                    break;
                }
                // Tag and attribute names that spell keywords stay identifiers.
                if s > i {
                    keyword_initial.clear_range(i, s - 1, cpu);
                    jsx_glue_hyphens(src, bitmaps, i, s, cpu);
                }
                let c = src[s];
                if c == b'<' {
                    let tokens = TokensView::new(
                        src,
                        bitmaps,
                        kinds,
                        ts,
                        0,
                        lanes.module,
                        &lanes.disambiguate.brackets,
                        &lanes.disambiguate.closers,
                        cpu,
                    );
                    if prev_sig(tokens.token_start, tokens.kinds, s, cpu)
                        .is_some_and(|q| src[q] == b'=')
                    {
                        let tpos = jsx_skip_trivia(src, s + 1, cpu);
                        jsx_punct(kinds, op_char, s, tk!(JsxLt), cpu);
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
                    jsx_punct(kinds, op_char, s, tk!(Lt), cpu);
                    // Open template substitutions, each counting its own
                    // nested braces - the same shape `carve` keeps in its
                    // depth vector.
                    let mut sub: Vec<u32> = Vec::new();
                    while p < src.len() && depth != 0 {
                        let q = if !sub.is_empty() {
                            find(src, p, OPENER8_BYTES, cpu)
                        } else {
                            find(src, p, OPENER_BYTES, cpu)
                        };
                        if q >= src.len() {
                            p = src.len();
                            break;
                        }
                        match src[q] {
                            b'<' => {
                                depth += 1;
                                jsx_punct(kinds, op_char, q, tk!(Lt), cpu);
                                p = q + 1;
                            }
                            b'>' => {
                                if !(q > 0 && src[q - 1] == b'=') {
                                    depth -= 1;
                                    jsx_punct(kinds, op_char, q, tk!(Gt), cpu);
                                }
                                p = q + 1;
                            }
                            b'"' | b'\'' => {
                                p = lex_string(src, token_start, kinds, q, src[q], lanes, cpu);
                            }
                            b'`' => {
                                let (end, opened) = lex_template_segment(
                                    src,
                                    token_start,
                                    kinds,
                                    q,
                                    tk!(TemplateHead),
                                    tk!(TemplateNoSub),
                                    lanes,
                                    cpu,
                                );
                                p = end;
                                if opened {
                                    sub.push(0);
                                }
                            }
                            // Braces only reach here through the OPENER8_BYTES search,
                            // which is only selected while a substitution is
                            // open - the guards say so rather than leaving it
                            // to the finder choice.
                            b'{' if !sub.is_empty() => {
                                if let Some(top) = sub.last_mut() {
                                    *top += 1;
                                }
                                p = q + 1;
                            }
                            b'}' if !sub.is_empty() => {
                                if let Some(top) = sub.last_mut()
                                    && *top != 0
                                {
                                    *top -= 1;
                                    p = q + 1;
                                    continue;
                                }
                                sub.pop();
                                let (end, opened) = lex_template_segment(
                                    src,
                                    token_start,
                                    kinds,
                                    q,
                                    tk!(TemplateMiddle),
                                    tk!(TemplateTail),
                                    lanes,
                                    cpu,
                                );
                                p = end;
                                if opened {
                                    sub.push(0);
                                }
                            }
                            // A comment inside a type-argument list is
                            // trivia; a lone `/` cannot start a type, so it
                            // is left alone.
                            b'/' => {
                                p = match src[q + 1] {
                                    b'*' => {
                                        lex_block_comment(src, token_start, kinds, q, lanes, cpu)
                                    }
                                    b'/' => {
                                        lex_line_comment(src, token_start, kinds, q, lanes, cpu)
                                    }
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
                        let e = find(src, s + 1, [c], cpu);
                        if e >= src.len() {
                            lanes.push_diag(
                                s as u32,
                                (src.len() - s) as u32,
                                DiagCode::UnterminatedString,
                            );
                        }
                        let end = if e < src.len() { e + 1 } else { src.len() };
                        kinds.set(s, tk!(String));
                        if end > s + 1 {
                            token_start.clear_range(s + 1, end - 1, cpu);
                        }
                        let be = if e < src.len() {
                            e
                        } else if src.len() > s + 1 && src[src.len() - 1] == c {
                            src.len() - 1
                        } else {
                            src.len()
                        };
                        lanes.push_string_raw(src.padded(), s + 1, be, cpu);
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
                        let d = if s + 1 < src.len() { src[s + 1] } else { 0 };
                        if d == b'*' {
                            // comment inside a tag = whitespace
                            i = lex_block_comment(src, token_start, kinds, s, lanes, cpu);
                        } else if d == b'/' {
                            i = lex_line_comment(src, token_start, kinds, s, lanes, cpu);
                        } else {
                            // `/>` closes the tag, with any trivia (comments, Unicode
                            // whitespace) between the two.
                            let gp = if d == b'>' {
                                Some(s + 1)
                            } else {
                                let w = jsx_skip_trivia(src, s + 1, cpu);
                                (w < src.len() && src[w] == b'>').then_some(w)
                            };
                            if let Some(gp) = gp {
                                // Carve any comment between `/` and `>`: the tag resumes past
                                // the `>`.
                                let mut q = s + 1;
                                while q < gp {
                                    q = match (src[q], src[q + 1]) {
                                        (b'/', b'*') => lex_block_comment(
                                            src,
                                            token_start,
                                            kinds,
                                            q,
                                            lanes,
                                            cpu,
                                        ),
                                        (b'/', b'/') => {
                                            lex_line_comment(src, token_start, kinds, q, lanes, cpu)
                                        }
                                        _ => q + 1,
                                    };
                                }
                                jsx_punct(kinds, op_char, gp, tk!(JsxTagEnd), cpu);
                                let parent = stack.last().map_or(JMode::Js, |f| f.parent);
                                stack.pop();
                                mode = parent;
                                if mode == JMode::Text {
                                    text_start = gp + 1;
                                }
                                i = gp + 1;
                            } else {
                                // lone `/` (malformed) - stays a slash.
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
                        jsx_punct(kinds, op_char, s, tk!(Gt), cpu);
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
                let s = find(src, i, JSX_TEXT_BYTES, cpu);
                let runend = if s < src.len() { s } else { src.len() };
                if runend > text_start {
                    // One JTEXT token for the run; neutralize its start byte
                    // against coalesce/keywords and clear the interior.
                    token_start.set(text_start, cpu);
                    kinds.set(text_start, tk!(JsxText));
                    op_char.clear(text_start, cpu);
                    digit.clear(text_start, cpu);
                    dot.clear(text_start, cpu);
                    keyword_initial.clear(text_start, cpu);
                    if runend > text_start + 1 {
                        token_start.clear_range(text_start + 1, runend - 1, cpu);
                    }
                }
                if s >= src.len() {
                    break;
                }
                let c = src[s];
                if c == b'{' {
                    if s >= 2 && src[s - 1] == b'u' && src[s - 2] == b'\\' {
                        restore_escaped_container(src, token_start, word, kinds, s);
                    }
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
                    // A stray > or } ends the run; clearing op_char stops coalesce fusing strays.
                    op_char.clear(s, cpu);
                    lanes.push_diag(s as u32, 1, DiagCode::JsxTextInvalidCharacter);
                    text_start = s + 1;
                    i = s + 1;
                } else {
                    // c == '<'. Trivia may follow it: a comment (`</*c*//a>` closes `a`) or
                    // Unicode whitespace, whose lead byte would pass for a name start.
                    let c1 = if s + 1 < src.len() { src[s + 1] } else { 0 };
                    let c2 = if s + 2 < src.len() { src[s + 2] } else { 0 };
                    let direct = c1 == b'>'
                        || (c1 == b'/' && c2 != b'*' && c2 != b'/')
                        || (c1 < 0x80 && is_id_start(c1, cpu));
                    let tpos = if direct { s + 1 } else { jsx_skip_trivia(src, s + 1, cpu) };
                    let tc = if tpos < src.len() { src[tpos] } else { 0 };
                    if tc == b'/' {
                        // closing tag `</name>` or `</>`; the name stays IDENT
                        // A comment between `<` and `/` is trivia to carve: the branch resumes
                        // past the `>` and would otherwise leave its bytes as operators.
                        let mut q = s + 1;
                        while q < tpos {
                            q = match (src[q], src[q + 1]) {
                                (b'/', b'*') => {
                                    lex_block_comment(src, token_start, kinds, q, lanes, cpu)
                                }
                                (b'/', b'/') => {
                                    lex_line_comment(src, token_start, kinds, q, lanes, cpu)
                                }
                                _ => q + 1,
                            };
                        }
                        let mut gp = tpos + 1;
                        loop {
                            gp = find(src, gp, [b'>', b'/'], cpu);
                            if gp >= src.len() || src[gp] == b'>' {
                                break;
                            }
                            gp = match src[gp + 1] {
                                b'*' => lex_block_comment(src, token_start, kinds, gp, lanes, cpu),
                                b'/' => lex_line_comment(src, token_start, kinds, gp, lanes, cpu),
                                _ => gp + 1,
                            };
                        }
                        if gp > tpos + 1 {
                            keyword_initial.clear_range(tpos + 1, gp - 1, cpu);
                            // `</data-x>`: the name region holds no
                            // expression container, so hyphens in it are
                            // always part of the JSXIdentifier.
                            jsx_glue_hyphens(src, bitmaps, tpos + 1, gp, cpu);
                        }
                        jsx_punct(kinds, op_char, s, tk!(JsxLt), cpu);
                        if gp < src.len() {
                            jsx_punct(kinds, op_char, gp, tk!(JsxTagEnd), cpu);
                        }
                        let after = if gp < src.len() { gp + 1 } else { src.len() };
                        if gp >= src.len() {
                            lanes.push_diag(
                                s as u32,
                                (src.len() - s) as u32,
                                DiagCode::UnterminatedJsxTag,
                            );
                        } else if let Some(f) = stack.last() {
                            let c2 = src[tpos + 1];
                            let cs = if (c2 < 0x80 && is_word(c2, cpu)) || c2 == b'>' {
                                tpos + 1
                            } else {
                                jsx_skip_trivia(src.until(gp), tpos + 1, cpu)
                            };
                            if !jsx_names_equal_fast(src, word, f.name_s as usize, cs, gp, cpu) {
                                lanes.push_diag(
                                    s as u32,
                                    (after - s) as u32,
                                    DiagCode::JsxClosingTagMismatch,
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
                    } else if tc == b'>' || is_id_start(tc, cpu) {
                        // child element / fragment
                        jsx_punct(kinds, op_char, s, tk!(JsxLt), cpu);
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
                        // Malformed lone < in text: clear op_char so no << is fused.
                        op_char.clear(s, cpu);
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
                lanes.push_diag(f.start, src.len() as u32 - f.start, DiagCode::UnterminatedJsxTag);
            }
            JFrameKind::JsxElem => {
                let ne = jsx_name_end(src, f.name_s as usize, cpu) as u32;
                lanes.push_diag(f.start, ne - f.start, DiagCode::UnterminatedJsxElement);
            }
            JFrameKind::JsxCont => {
                lanes.push_diag(
                    f.start,
                    src.len() as u32 - f.start,
                    DiagCode::UnterminatedJsxContainer,
                );
            }
            JFrameKind::TemplateSub => {}
        }
    }
    lanes.jsx_stack = stack;
}

/// Cold: misc_pre read a backslash-u in JSX text as an escape running into the container at s.
#[cpu_version(Generic)]
#[cold]
#[inline(never)]
unsafe fn restore_escaped_container(
    src: Source,
    token_start: Bitmap,
    word: Bitmap,
    kinds: Kinds,
    s: usize,
) {
    let cpu = BASELINE;
    token_start.set(s, cpu);
    let mut i = s + 1;
    if word.get(i, cpu) {
        token_start.set(i, cpu);
        i = word.next0(i, src.len(), cpu);
    }
    if src[i] == b'\\' && src[i + 1] == b'u' {
        token_start.set(i, cpu);
        kinds.set(i, tk!(IdentEscaped));
        i = scan_ident_esc(src, i, cpu);
    }
    if src[i] == b'}' {
        token_start.set(i, cpu);
    }
}

/// Set a JSX punctuator's kind and clear its op_char bit so coalesce cannot fuse it.
#[cpu_multiversion]
#[inline(always)]
unsafe fn jsx_punct(kinds: Kinds, op_char: Bitmap, off: usize, k: u8, cpu: impl Cpu) {
    kinds.set(off, k);
    op_char.clear(off, cpu);
}
