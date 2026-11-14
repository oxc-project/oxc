use oxc_simd::{BASELINE, Cpu, cpu_multiversion, cpu_version};

use crate::{error::DiagCode, options::LexOptions, token::TokenKind};

use crate::pipeline::{
    bitmap::{Bitmap, Bitmaps},
    bytes::{is_digit, is_id_start, is_word},
    disambiguate::DisambiguateState,
    find::find,
    kinds::Kinds,
    lanes::Lanes,
    lex::{Lex, LexState},
    scan::scan_ident_esc,
    source::Source,
};

use super::{
    comments::{
        html_close_comment_at, html_open_comment_at, lex_comment_at, lex_html_close_comment,
        lex_html_open_comment,
    },
    common::{
        OPENER_BYTES, OPENER8_BYTES, is_unicode_escape_brace, lex_hashbang,
        lex_jsx_attribute_string, lex_slash, lex_string, lex_template_segment, mark_token,
        skip_unicode_brace_escape,
    },
};

mod angle_brackets;
mod hyphens;
mod names;

use angle_brackets::{jsx_over_type_params, lex_type_arguments};
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
enum JsxMode {
    Js,
    Tag,
    Text,
}

#[derive(Clone, Copy)]
pub(in crate::pipeline) struct JsxFrame {
    kind: JsxFrameKind,
    /// Mode to restore when this frame pops.
    parent: JsxMode,
    /// Nested-brace counter (TemplateSub / Container only).
    depth: u32,
    start: u32,
    name_pos: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum JsxFrameKind {
    /// `${...}` of a split template literal (mode JS; `depth` counts nested `{}`).
    TemplateSub,
    /// `{...}` JSX expression container (mode JS; `depth` counts nested `{}`).
    Container,
    /// A `<...>` opening tag being lexed (mode TAG).
    Tag,
    /// An opened element whose children are being lexed (mode TEXT).
    Element,
}

/// JSX-aware carve: JS, tag and text modes over one stack of open frames.
#[cpu_multiversion]
pub(super) fn carve_jsx(lex: Lex, state: &mut LexState, cpu: impl Cpu) {
    let LexState { lanes, disambiguate, jsx_stack, diag_suppress, .. } = state;
    jsx_stack.clear();
    let mut carve = JsxCarve {
        lex,
        bitmaps: *lex.bitmaps,
        options: *lex.options,
        lanes,
        disambiguate,
        diag_suppress,
        stack: jsx_stack,
        mode: JsxMode::Js,
        text_start: 0,
        html_close: false,
    };
    let mut i = lex_hashbang(lex.src, lex.bitmaps.token_start, lex.kinds, cpu);
    while let Some(next) = carve.step(i, cpu) {
        i = next;
    }
    carve.report_unclosed(cpu);
}

/// The JSX carve between stops: what it writes, the frames open around it and its mode.
struct JsxCarve<'a> {
    lex: Lex<'a>,
    /// Copies of the bitmaps and options in lex, so each stop doesn't load them again.
    bitmaps: Bitmaps<'a>,
    options: LexOptions,
    lanes: &'a mut Lanes,
    disambiguate: &'a mut DisambiguateState,
    diag_suppress: &'a mut Vec<(u32, u32)>,
    stack: &'a mut Vec<JsxFrame>,
    mode: JsxMode,
    /// Where the text run being lexed in text mode starts.
    text_start: usize,
    /// Set by a script's first <!--; JS mode then stops at >, so an earlier --> stays operators.
    html_close: bool,
}

impl JsxCarve<'_> {
    /// Carve from i to the next stop of the current mode; None once the source is done.
    #[cpu_multiversion]
    #[inline(always)]
    fn step(&mut self, i: usize, cpu: impl Cpu) -> Option<usize> {
        match self.mode {
            JsxMode::Js => self.js(i, cpu),
            JsxMode::Tag => self.tag(i, cpu),
            JsxMode::Text => self.text(i, cpu),
        }
    }

    #[cpu_multiversion]
    #[inline(always)]
    fn js(&mut self, i: usize, cpu: impl Cpu) -> Option<usize> {
        let Lex { src, kinds, .. } = self.lex;
        let Bitmaps { token_start, op_char, .. } = self.bitmaps;
        let module = self.options.source_type_module;
        let in_brace = self
            .stack
            .last()
            .is_some_and(|f| matches!(f.kind, JsxFrameKind::TemplateSub | JsxFrameKind::Container));
        let s = match (in_brace, self.html_close) {
            (true, false) => find(src, i, OPENER_JSX7_BYTES, cpu),
            (true, true) => find(src, i, OPENER8_BYTES, cpu),
            (false, false) => find(src, i, OPENER_JSX5_BYTES, cpu),
            (false, true) => find(src, i, OPENER_BYTES, cpu),
        };
        if s >= src.len() {
            return None;
        }
        let c = src[s];
        Some(match c {
            b'"' | b'\'' => lex_string(src, token_start, kinds, s, c, self.lanes, cpu),
            b'`' => self.template(s, TokenKind::TemplateHead, TokenKind::TemplateNoSub, cpu),
            b'{' if is_unicode_escape_brace(src, s, cpu) => skip_unicode_brace_escape(src, s, cpu),
            b'{' => {
                if let Some(f) = self.stack.last_mut() {
                    f.depth += 1;
                }
                s + 1
            }
            b'}' => self.close_brace(s, cpu),
            b'/' => lex_slash(self.lex, s, self.lanes, self.disambiguate, cpu),
            b'<' => self.js_less_than(s, cpu),
            // Only a script's finder stops here: Annex B B.1.3 HTML close comments.
            b'>' if html_close_comment_at(src, s, module, cpu) => {
                lex_html_close_comment(src, token_start, kinds, op_char, s, self.lanes)
            }
            _ => s + 1,
        })
    }

    /// A < in JS mode: an HTML open comment, a shift, a comparison, or the start of an element.
    #[cpu_multiversion]
    #[inline(always)]
    fn js_less_than(&mut self, s: usize, cpu: impl Cpu) -> usize {
        let Lex { src, kinds, .. } = self.lex;
        let Bitmaps { token_start, op_char, .. } = self.bitmaps;
        let module = self.options.source_type_module;
        let c1 = if s + 1 < src.len() { src[s + 1] } else { 0 };
        if c1 == b'!' && html_open_comment_at(src, s, module, cpu) {
            // Annex B B.1.1: the goal, not the JSX setting, decides.
            self.html_close = !module;
            return lex_html_open_comment(src, token_start, kinds, op_char, s, module, self.lanes);
        }
        if c1 == b'<' {
            // A shift: skip both, or the second < would read the first as an operand preceder.
            return s + 2;
        }
        if c1 == b'=' || is_digit(c1, cpu) {
            // Comparisons like a<=b and a<5 are left for coalesce.
            return s + 1;
        }
        if !self.disambiguate.not_operator_position(self.lex, s, cpu) {
            // In operator position: a less-than.
            return s + 1;
        }
        // Whitespace, Unicode too, and comments may separate the < from the name.
        let name_pos = jsx_skip_trivia_fast(src, s + 1, cpu);
        let name_byte = if name_pos < src.len() { src[name_pos] } else { 0 };
        // A fragment or element, unless tsx reads a type-parameter list, which stays a less-than.
        if name_byte == b'>'
            || (is_id_start(name_byte, cpu)
                && jsx_over_type_params(self.lex, s, name_pos, self.lanes, self.disambiguate, cpu))
        {
            self.open_tag(s, name_pos, cpu);
        }
        s + 1
    }

    /// A } in JS mode closes a nested brace, a template substitution or an expression container.
    #[cpu_multiversion]
    #[inline(always)]
    fn close_brace(&mut self, s: usize, cpu: impl Cpu) -> usize {
        match self.stack.last_mut() {
            Some(f) if f.depth > 0 => {
                f.depth -= 1;
                s + 1
            }
            Some(f) if f.kind == JsxFrameKind::TemplateSub => {
                self.stack.pop();
                self.template(s, TokenKind::TemplateMiddle, TokenKind::TemplateTail, cpu)
            }
            Some(f) if f.kind == JsxFrameKind::Container => self.close(s + 1),
            _ => s + 1,
        }
    }

    /// Lex the template segment at s, opening a substitution frame if it ends in a dollar brace.
    #[cpu_multiversion]
    #[inline(always)]
    fn template(&mut self, s: usize, head: TokenKind, flat: TokenKind, cpu: impl Cpu) -> usize {
        let Lex { src, kinds, .. } = self.lex;
        let token_start = self.bitmaps.token_start;
        let (end, opened) =
            lex_template_segment(src, token_start, kinds, s, head, flat, self.lanes, cpu);
        if opened {
            self.open(JsxFrameKind::TemplateSub, s, 0);
        }
        end
    }

    #[cpu_multiversion]
    #[inline(always)]
    fn tag(&mut self, i: usize, cpu: impl Cpu) -> Option<usize> {
        let Lex { src, bitmaps, kinds, .. } = self.lex;
        let Bitmaps { token_start, keyword_initial, op_char, .. } = self.bitmaps;
        let s = find(src, i, JSX_TAG_BYTES, cpu);
        if s >= src.len() {
            return None;
        }
        // Tag and attribute names that spell keywords stay identifiers.
        if s > i {
            keyword_initial.clear_range(i, s - 1, cpu);
            jsx_glue_hyphens(src, bitmaps, i, s, cpu);
        }
        let c = src[s];
        // An element as an attribute value.
        if c == b'<' && self.disambiguate.prev_sig(self.lex, s, cpu).is_some_and(|q| src[q] == b'=')
        {
            let name_pos = jsx_skip_trivia(src, s + 1, cpu);
            self.open_tag(s, name_pos, cpu);
            return Some(s + 1);
        }
        // A tsx type-argument list on the element name ends in a > that keeps the tag open.
        if self.options.ts && c == b'<' {
            let p = lex_type_arguments(self.lex, s, self.lanes, cpu);
            // Later passes can still report from inside the run, so its diagnostics are dropped.
            self.diag_suppress.push((s as u32, p as u32));
            return Some(p);
        }
        Some(match c {
            b'"' | b'\'' => {
                lex_jsx_attribute_string(src, token_start, kinds, s, c, self.lanes, cpu)
            }
            b'{' => {
                self.open(JsxFrameKind::Container, s, 0);
                s + 1
            }
            b'/' => self.tag_slash(s, cpu),
            b'>' => {
                // The opening tag ends and its children begin.
                if let Some(f @ JsxFrame { kind: JsxFrameKind::Tag, .. }) = self.stack.last_mut() {
                    f.kind = JsxFrameKind::Element;
                }
                jsx_punct(kinds, op_char, s, TokenKind::Gt, cpu);
                self.mode = JsxMode::Text;
                self.text_start = s + 1;
                s + 1
            }
            _ => s + 1,
        })
    }

    /// A / in a tag: a comment, or the /> that closes the tag, with any trivia between the two.
    #[cpu_multiversion]
    #[inline(always)]
    fn tag_slash(&mut self, s: usize, cpu: impl Cpu) -> usize {
        let Lex { src, kinds, .. } = self.lex;
        let Bitmaps { token_start, op_char, .. } = self.bitmaps;
        let d = if s + 1 < src.len() { src[s + 1] } else { 0 };
        if d == b'*' || d == b'/' {
            return lex_comment_at(src, token_start, kinds, s, self.lanes, cpu);
        }
        let gt = if d == b'>' {
            s + 1
        } else {
            let w = jsx_skip_trivia(src, s + 1, cpu);
            if w >= src.len() || src[w] != b'>' {
                // A lone slash, which is malformed, stays a slash.
                return s + 1;
            }
            w
        };
        // Carve the comments between the / and the >, since the carve resumes past the >.
        self.lex_comments(s + 1, gt, cpu);
        jsx_punct(kinds, op_char, gt, TokenKind::JsxTagEnd, cpu);
        self.close(gt + 1)
    }

    #[cpu_multiversion]
    #[inline(always)]
    fn text(&mut self, i: usize, cpu: impl Cpu) -> Option<usize> {
        let Lex { src, kinds, .. } = self.lex;
        let Bitmaps { token_start, word, keyword_initial, op_char, digit, dot, .. } = self.bitmaps;
        let s = find(src, i, JSX_TEXT_BYTES, cpu);
        let run_end = if s < src.len() { s } else { src.len() };
        let start = self.text_start;
        if run_end > start {
            // One text token for the run, its start byte cleared of everything coalesce reads.
            // SAFETY: start < run_end <= src.len(), and carve cut the handles to the source.
            unsafe {
                token_start.set_unchecked(start, cpu);
                op_char.clear_unchecked(start, cpu);
                digit.clear_unchecked(start, cpu);
                dot.clear_unchecked(start, cpu);
                keyword_initial.clear_unchecked(start, cpu);
                mark_token(token_start, kinds, start, run_end, TokenKind::JsxText, cpu);
            }
        }
        if s >= src.len() {
            return None;
        }
        Some(match src[s] {
            b'{' => {
                if is_unicode_escape_brace(src, s, cpu) {
                    restore_escaped_container(src, token_start, word, kinds, s);
                }
                self.open(JsxFrameKind::Container, s, 0);
                s + 1
            }
            b'<' => self.text_less_than(s, cpu),
            _ => {
                // A stray > or } ends the run; clearing op_char stops coalesce fusing strays.
                op_char.clear(s, cpu);
                self.lanes.push_diag(s as u32, 1, DiagCode::JsxTextInvalidCharacter);
                self.text_start = s + 1;
                s + 1
            }
        })
    }

    /// A < in text: a closing tag, a child element or fragment, or a stray < left in the text.
    #[cpu_multiversion]
    #[inline(always)]
    fn text_less_than(&mut self, s: usize, cpu: impl Cpu) -> usize {
        let src = self.lex.src;
        // Trivia may follow: a comment, or Unicode whitespace whose lead byte passes for a name.
        let c1 = if s + 1 < src.len() { src[s + 1] } else { 0 };
        let c2 = if s + 2 < src.len() { src[s + 2] } else { 0 };
        let direct = c1 == b'>'
            || (c1 == b'/' && c2 != b'*' && c2 != b'/')
            || (c1 < 0x80 && is_id_start(c1, cpu));
        let name_pos = if direct { s + 1 } else { jsx_skip_trivia(src, s + 1, cpu) };
        let name_byte = if name_pos < src.len() { src[name_pos] } else { 0 };
        if name_byte == b'/' {
            self.close_tag(s, name_pos, cpu)
        } else if name_byte == b'>' || is_id_start(name_byte, cpu) {
            self.open_tag(s, name_pos, cpu);
            s + 1
        } else {
            // A malformed lone < in text: clearing op_char stops coalesce fusing a shift.
            self.bitmaps.op_char.clear(s, cpu);
            self.text_start = s + 1;
            s + 1
        }
    }

    /// The closing tag whose < is at s and / at slash: carve its trivia, check its name, close it.
    #[cpu_multiversion]
    #[inline(always)]
    fn close_tag(&mut self, s: usize, slash: usize, cpu: impl Cpu) -> usize {
        let Lex { src, bitmaps, kinds, .. } = self.lex;
        let Bitmaps { token_start, word, keyword_initial, op_char, .. } = self.bitmaps;
        // Carve the comments between the < and the /, since the carve resumes past the >.
        self.lex_comments(s + 1, slash, cpu);
        let mut gt = slash + 1;
        loop {
            gt = find(src, gt, [b'>', b'/'], cpu);
            if gt >= src.len() || src[gt] == b'>' {
                break;
            }
            gt = lex_comment_at(src, token_start, kinds, gt, self.lanes, cpu);
        }
        if gt > slash + 1 {
            keyword_initial.clear_range(slash + 1, gt - 1, cpu);
            // A closing tag name holds no expression container, so its hyphens all belong to it.
            jsx_glue_hyphens(src, bitmaps, slash + 1, gt, cpu);
        }
        jsx_punct(kinds, op_char, s, TokenKind::JsxLt, cpu);
        if gt < src.len() {
            jsx_punct(kinds, op_char, gt, TokenKind::JsxTagEnd, cpu);
        }
        let after = if gt < src.len() { gt + 1 } else { src.len() };
        if gt >= src.len() {
            self.lanes.push_diag(s as u32, (src.len() - s) as u32, DiagCode::UnterminatedJsxTag);
        } else if let Some(f) = self.stack.last() {
            let c = src[slash + 1];
            let name_start = if (c < 0x80 && is_word(c, cpu)) || c == b'>' {
                slash + 1
            } else {
                jsx_skip_trivia(src.until(gt), slash + 1, cpu)
            };
            if !jsx_names_equal_fast(src, word, f.name_pos as usize, name_start, gt, cpu) {
                let code = DiagCode::JsxClosingTagMismatch;
                self.lanes.push_diag(s as u32, (after - s) as u32, code);
            }
        }
        self.close(after)
    }

    /// Lex the comments between from and to, where there is only trivia.
    #[cpu_multiversion]
    #[inline(always)]
    fn lex_comments(&mut self, from: usize, to: usize, cpu: impl Cpu) {
        let Lex { src, kinds, .. } = self.lex;
        let mut q = from;
        while q < to {
            q = lex_comment_at(src, self.bitmaps.token_start, kinds, q, self.lanes, cpu);
        }
    }

    /// Open the tag whose < is at s and whose name starts at name_pos.
    #[cpu_multiversion]
    #[inline(always)]
    fn open_tag(&mut self, s: usize, name_pos: usize, cpu: impl Cpu) {
        jsx_punct(self.lex.kinds, self.bitmaps.op_char, s, TokenKind::JsxLt, cpu);
        self.open(JsxFrameKind::Tag, s, name_pos);
    }

    /// Open a frame at start and enter the mode its contents are lexed in.
    #[inline(always)]
    fn open(&mut self, kind: JsxFrameKind, start: usize, name_pos: usize) {
        let (start, name_pos) = (start as u32, name_pos as u32);
        self.stack.push(JsxFrame { kind, parent: self.mode, depth: 0, start, name_pos });
        self.mode = if kind == JsxFrameKind::Tag { JsxMode::Tag } else { JsxMode::Js };
    }

    /// Close the innermost frame and go back to its parent's mode at resume.
    #[inline(always)]
    fn close(&mut self, resume: usize) -> usize {
        let parent = self.stack.last().map_or(JsxMode::Js, |f| f.parent);
        self.stack.pop();
        self.mode = parent;
        if parent == JsxMode::Text {
            self.text_start = resume;
        }
        resume
    }

    /// Report the innermost frame still open at the end of the source.
    #[cpu_multiversion]
    #[inline(always)]
    fn report_unclosed(&mut self, cpu: impl Cpu) {
        let src = self.lex.src;
        let Some(f) = self.stack.last() else { return };
        let to_end = src.len() as u32 - f.start;
        match f.kind {
            JsxFrameKind::Tag => {
                self.lanes.push_diag(f.start, to_end, DiagCode::UnterminatedJsxTag);
            }
            JsxFrameKind::Element => {
                let name_end = jsx_name_end(src, f.name_pos as usize, cpu) as u32;
                let code = DiagCode::UnterminatedJsxElement;
                self.lanes.push_diag(f.start, name_end - f.start, code);
            }
            JsxFrameKind::Container => {
                self.lanes.push_diag(f.start, to_end, DiagCode::UnterminatedJsxContainer);
            }
            JsxFrameKind::TemplateSub => {}
        }
    }
}

/// Cold: misc_pre read a backslash-u in JSX text as an escape running into the container at s.
#[cpu_version(Generic)]
#[cold]
#[inline(never)]
fn restore_escaped_container(
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
        kinds.set(i, TokenKind::IdentEscaped);
        i = scan_ident_esc(src, i, cpu);
    }
    if src[i] == b'}' {
        token_start.set(i, cpu);
    }
}

/// Set a JSX punctuator's kind and clear its op_char bit so coalesce cannot fuse it.
#[cpu_multiversion]
#[inline(always)]
fn jsx_punct(kinds: Kinds, op_char: Bitmap, off: usize, k: TokenKind, cpu: impl Cpu) {
    kinds.set(off, k);
    op_char.clear(off, cpu);
}
