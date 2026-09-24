//! Words and whitespace.
//!
//! Text is printed RAW (escapes and entities as written);
//! only the decisions below touch it:
//! - `*` / `_` runs that could open or close emphasis get escaped inside emphasis / strong
//! - a lone `===` / `---` word starting a line gets escaped (it would read as a setext underline)
//! - a line break before a word that would start a block (`-`, `1.`, `#`, `>`) never breaks
//!
//! Around Chinese / Japanese text a whitespace follows Prettier's `printWhitespace`:
//! a line break between CJ characters is not a space, and a space next to one never breaks
//! (`cjk` classifies the word edges).

use std::borrow::Cow;

use oxc_formatter_core::arena_cow_str;
use oxc_markdown_parser::{Constructs, lexical};

use crate::options::ProseWrap;

use super::{
    MarkdownFormatter,
    cjk::{self, Edge, Kind},
    escape::{
        ends_with_unescaped_backslash, is_fake_setext_underline, leading_run_escaped,
        print_delimited_word,
    },
    is_split_whitespace,
    line_shape::{is_line_shape_start, line_opens_block, may_open_block},
    parts::{Parts, Sep},
};

/// The word after a whitespace: the one a break there would put at a line start.
#[derive(Clone, Copy)]
pub struct NextWord<'a> {
    pub word: &'a str,
    /// The source line from the word on, as it prints: the line a kept break puts the word at the start of.
    /// Only `preserve` reads it (a text never spans a line, so only `next_word_of` fills it).
    pub line: &'a str,
    /// The word's leading `*` / `_` run gets escaped (inside emphasis):
    /// printed, it starts with `\` and cannot open a block.
    pub escaped: bool,
    /// The word starts a text of the same sentence: the CJK rules see it.
    /// Past another node's edge they do not (Prettier's sentence ends there), so the whitespace may break and stays a space.
    pub in_sentence: bool,
}

#[derive(Clone, Copy, Default)]
pub struct TextContext<'a> {
    /// Bytes at the text's start that extend the autolink literal before it
    /// (its trailing punctuation stretch): an escape there changes the link.
    pub autolink_stretch: usize,
    /// The text is the first / last child of an `Emphasis` / `Strong`: its marker as printed.
    pub first_of_delimiter: Option<u8>,
    pub last_of_delimiter: Option<u8>,
    /// The character right before / after the text as printed:
    /// a space for a line break, the neighbor node's edge character, the enclosing marker;
    /// `None` at a paragraph edge.
    pub edge_prev: Option<char>,
    pub edge_next: Option<char>,
    /// The previous sibling is a soft break (the text starts a source line).
    pub after_soft_break: bool,
    /// The whitespace follows a liquid tag: under `always` it never breaks.
    /// With no break before a tag either (`prevents_break`), a tag never stands alone on a line,
    /// where it would be a flow tag.
    pub after_liquid: bool,
    /// The next sibling is a soft break (the text ends a source line).
    pub before_soft_break: bool,
    /// The first word the next sibling starts, for the whitespace that ends this text.
    pub next_word: Option<NextWord<'a>>,
    /// The text's last word with the nodes glued after it on the source line (`:-:` + `[[w]]`),
    /// the word a break before it would put at a line start.
    pub glued_last_word: Option<&'a str>,
    /// The last word of the previous sibling text (a soft break's other side), for the CJK rules only;
    /// a trailing `\` there was escaped by the text (see `push_text`).
    pub prev_word: Option<&'a str>,
    /// The sentence puts spaces between CJ and non-CJK words (Prettier's `usesCJSpaces`);
    /// `None` when it has no CJK text at all.
    pub cj_spaces: Option<bool>,
    /// Inside a reference link's raw content, where a line break never prevents a break.
    pub is_link: bool,
}

/// Splits `raw` into words and whitespace and pushes them, in one pass.
pub fn push_text<'a>(
    raw: &'a str,
    cx: TextContext<'a>,
    parts: &mut Parts<'a>,
    f: &MarkdownFormatter<'_, 'a>,
) {
    // Markers printed as written need no escapes: the text pairs as the source did
    let in_delimiter =
        f.context().delimiter_depth().get() > 0 && !f.context().literal_markers().get();
    let prose_wrap = f.options().prose_wrap;

    let mut rest = raw;
    let mut is_first = true;
    let leading_ws = raw.starts_with(is_split_whitespace);
    let mut prev_word: Option<&'a str> = None;
    // `prev_word` is the text's first word
    let mut prev_is_first = false;
    loop {
        // Whitespace before the next word (leading, or the run after the previous word).
        let after_ws = rest.trim_start_matches(is_split_whitespace);
        let ws = &rest[..rest.len() - after_ws.len()];
        rest = after_ws;
        let word_end = rest.find(is_split_whitespace).unwrap_or(rest.len());
        let (word, tail) = rest.split_at(word_end);
        let is_last = tail.trim_start_matches(is_split_whitespace).is_empty();
        if !ws.is_empty() {
            let next = if word.is_empty() {
                cx.next_word
            } else {
                let escaped = in_delimiter && {
                    let prev = if is_first { cx.edge_prev } else { Some(' ') };
                    let next = if is_last { cx.edge_next } else { Some(' ') };
                    leading_run_escaped(word, prev, next)
                };
                let word = match cx.glued_last_word {
                    Some(glued) if is_last => glued,
                    _ => word,
                };
                Some(NextWord { word, line: "", escaped, in_sentence: true })
            };
            let cx = TextContext {
                next_word: next,
                // The first word inside an autolink literal's stretch is part of the link:
                // the whitespace after it stays a space (a CJ neighbor must not glue to the link)
                prev_word: prev_word.filter(|_| !(cx.autolink_stretch > 0 && prev_is_first)),
                after_liquid: is_first && cx.after_liquid,
                ..cx
            };
            // A line break right after an unescaped `\` would be a hard break
            // (the text's last word gets its `\` escaped instead, and may break)
            if prev_word.is_some_and(ends_with_unescaped_backslash) {
                parts.push_str(" ");
            } else {
                push_whitespace(ws.contains('\n'), &cx, parts, f);
            }
        }
        if word.is_empty() {
            break;
        }
        prev_word = Some(word);
        prev_is_first = is_first;
        let printed: Cow<'a, str> = if in_delimiter {
            let prev = if is_first { cx.edge_prev } else { Some(' ') };
            let next = if is_last { cx.edge_next } else { Some(' ') };
            // A leading marker character would merge with the opening marker (`**foo*`)
            let escape_leading =
                is_first && !leading_ws && cx.first_of_delimiter == Some(word.as_bytes()[0]);
            let autolink_stretch = if is_first { cx.autolink_stretch } else { 0 };
            print_delimited_word(word, escape_leading, autolink_stretch, prev, next)
        } else if prose_wrap == ProseWrap::Preserve
            && is_first
            && is_last
            && !leading_ws
            && cx.after_soft_break
            // Alone on its line: a node after it (`== `x``) makes the line text
            && cx.next_word.is_none()
            && is_fake_setext_underline(word)
        {
            Cow::Owned(format!("\\{word}"))
        } else {
            Cow::Borrowed(word)
        };
        // A line ending with `\` is a hard break (the source had whitespace after it, which goes);
        // a `\` right before the closing marker would escape the marker's first character
        let printed: Cow<'a, str> = if is_last
            && (cx.before_soft_break || cx.last_of_delimiter.is_some())
            && ends_with_unescaped_backslash(&printed)
        {
            Cow::Owned(format!("{printed}\\"))
        } else {
            printed
        };
        parts.push_str(arena_cow_str(&printed, f));
        rest = tail;
        is_first = false;
    }
}

/// The whitespace (a space run, or one holding a `newline`) between `cx.prev_word`
/// and `cx.next_word` (`None`: the whitespace touches a node edge):
/// a separator that may break, a space, or nothing.
pub fn push_whitespace<'a>(
    newline: bool,
    cx: &TextContext<'a>,
    parts: &mut Parts<'a>,
    f: &MarkdownFormatter<'_, 'a>,
) {
    let options = f.options();
    // Never a break right after a liquid tag under `always` (nor before one):
    // alone on a line it would be a flow tag.
    if cx.after_liquid && options.prose_wrap == ProseWrap::Always {
        parts.push_str(" ");
        return;
    }
    // A space only matters where it may become a break (`always`)
    let prose_wrap = if !cx.is_link
        && (newline || options.prose_wrap == ProseWrap::Always)
        && prevents_break(newline, cx.next_word, options.prose_wrap)
    {
        ProseWrap::Never
    } else {
        options.prose_wrap
    };

    if prose_wrap == ProseWrap::Preserve && newline {
        parts.push_sep(Sep::HardLine);
        return;
    }
    // The CJK rules only apply to a sentence with CJK text
    let (prev, next) = if cx.cj_spaces.is_some() {
        (
            cx.prev_word.and_then(|w| cjk::edges(w).map(|(_, last)| last)),
            cx.next_word
                .filter(|w| w.in_sentence)
                .and_then(|w| cjk::edges(w.word).map(|(first, _)| first)),
        )
    } else {
        (None, None)
    };
    let can_be_space = !newline || line_break_can_be_space(prev, next, cx);
    let breakable = prose_wrap == ProseWrap::Always
        && f.context().no_wrap_depth().get() == 0
        && is_breakable(prev, next, cx);
    match (breakable, can_be_space) {
        (true, true) => parts.push_sep(Sep::Line),
        (true, false) => parts.push_sep(Sep::SoftLine),
        (false, true) => parts.push_str(" "),
        (false, false) => {}
    }
}

/// Prettier's `lineBreakCanBeConvertedToSpace`.
fn line_break_can_be_space(prev: Option<Edge>, next: Option<Edge>, cx: &TextContext<'_>) -> bool {
    if cx.is_link {
        return true;
    }
    let (Some(prev), Some(next)) = (prev, next) else { return true };
    let latin_like = |kind: Kind| matches!(kind, Kind::NonCjk | Kind::KLetter);
    // Between non-CJK / Korean words, or Korean and CJ, a line break is a space
    if (latin_like(prev.kind) && latin_like(next.kind))
        || matches!(
            (prev.kind, next.kind),
            (Kind::KLetter, Kind::CjLetter) | (Kind::CjLetter, Kind::KLetter)
        )
    {
        return true;
    }
    // A delimiter run glued to a CJ character gains flanking it did not have (`」\n**` → `」**` closes);
    // a space keeps it inert, as the line break did
    if is_delimiter_char(prev.ch) || is_delimiter_char(next.ch) {
        return true;
    }
    // Around CJK punctuation or between CJ letters it is nothing
    if prev.kind == Kind::CjkPunctuation
        || next.kind == Kind::CjkPunctuation
        || (prev.kind == Kind::CjLetter && next.kind == Kind::CjLetter)
    {
        return false;
    }
    // Between CJ and non-CJK: a space next to ASCII punctuation (`:::` fences),
    // nothing next to other punctuation (`〜`, `…`), else the sentence's own style.
    if prev.ch.is_ascii_punctuation() || next.ch.is_ascii_punctuation() {
        return true;
    }
    if prev.punctuation || next.punctuation {
        return false;
    }
    cx.cj_spaces.unwrap_or(false)
}

/// `*` `_` `~` `` ` ``: runs of these pair up by flanking or by length, so their neighbors matter.
fn is_delimiter_char(c: char) -> bool {
    matches!(c, '*' | '_' | '~' | '`')
}

/// Prettier's `isBreakable` for a `" "` / `"\n"` whitespace under `always`.
fn is_breakable(prev: Option<Edge>, next: Option<Edge>, cx: &TextContext<'_>) -> bool {
    if cx.is_link {
        return true;
    }
    let (Some(prev), Some(next)) = (prev, next) else { return true };
    // Korean next to CJ breaks;
    // anything else next to CJ never does (browsers turn the break into a space)
    matches!(
        (prev.kind, next.kind),
        (Kind::KLetter, Kind::CjLetter) | (Kind::CjLetter, Kind::KLetter)
    ) || !(prev.kind.is_cj() || next.kind.is_cj())
}

/// Prettier's `usesCJSpaces`, over the words of one sentence (the texts of a soft-break run):
/// whether spaces outnumber nothing between CJ and non-CJK words.
/// `words` yields each text's words with the whitespace before them (`None` for the first).
pub fn uses_cj_spaces<'a>(texts: impl Iterator<Item = &'a str>) -> bool {
    let (mut spaces, mut nothing) = (0u32, 0u32);
    let counts = |a: Kind, b: Kind| {
        matches!((a, b), (Kind::CjLetter, Kind::NonCjk) | (Kind::NonCjk, Kind::CjLetter))
    };
    for text in texts {
        let mut prev_last: Option<Kind> = None;
        let mut rest = text;
        loop {
            let after_ws = rest.trim_start_matches(is_split_whitespace);
            let ws = &rest[..rest.len() - after_ws.len()];
            rest = after_ws;
            let word_end = rest.find(is_split_whitespace).unwrap_or(rest.len());
            let (word, tail) = rest.split_at(word_end);
            if word.is_empty() {
                break;
            }
            let mut kinds = cjk::kinds(word);
            let first = kinds.next().unwrap_or(Kind::NonCjk);
            if let Some(prev) = prev_last
                && !ws.is_empty()
                && !ws.contains('\n')
                && counts(prev, first)
            {
                spaces += 1;
            }
            let mut last = first;
            for kind in kinds {
                if counts(last, kind) {
                    nothing += 1;
                }
                last = kind;
            }
            prev_last = Some(last);
            rest = tail;
        }
    }
    spaces > nothing
}

/// Never break before a word that would start a block.
pub fn prevents_break(newline: bool, next: Option<NextWord<'_>>, prose_wrap: ProseWrap) -> bool {
    let Some(next) = next else { return false };
    if next.escaped {
        return false;
    }
    // Under `preserve` lines stay where they are, so whether the word stands alone is known
    if !looks_like_block_start(next, prose_wrap == ProseWrap::Preserve) {
        return false;
    }
    // A lone `---` / `===` after a newline is going to be escaped as a fake setext underline instead
    !(prose_wrap == ProseWrap::Preserve
        && newline
        && is_fake_setext_underline(next.line.trim_end_matches(is_split_whitespace)))
}

/// The parser's line-start classification (`lexical::line_start`), asked about the word alone:
/// list markers, `#`s, `>`, thematic breaks and setext underlines, fences, HTML and directive openers,
/// table rows, footnote definitions.
/// (Prettier tests `/^>|^(?:[*+-]|#{1,6}|\d+[).])$/` only, which is how a wrapped `<div>` or `***` opens a block.)
fn looks_like_block_start(next: NextWord<'_>, exact: bool) -> bool {
    // `exact` (a kept break under `preserve`): the line is known, ask about it.
    // Otherwise the line depends on the wrapping itself, so both candidates count:
    // the word alone (`***` is a break, `-|-` a delimiter row)
    // and the word with content after it (`- x` interrupts a paragraph where an empty `-` does not).
    let target = if exact { next.line } else { next.word };
    if !target.as_bytes().first().is_some_and(|&b| may_open_block(b)) {
        return false;
    }
    if exact {
        // A `|` row is inert on its own: the delimiter row after it is what makes a table, and that break is kept apart
        return line_opens_block(target, true) || is_line_shape_start(target);
    }
    let constructs = Constructs::markdown();
    // A dialect line shape at a line start is printed raw from then on: never create one
    // (asked about the word alone, a `:::` word counts even where `::: note` would be an opener)
    if lexical::line_start(&constructs, target, true).is_some() || is_line_shape_start(target) {
        return true;
    }
    let mut buf = [0u8; 64];
    let len = next.word.len();
    if len + 2 > buf.len() {
        return false;
    }
    buf[..len].copy_from_slice(next.word.as_bytes());
    buf[len..len + 2].copy_from_slice(b" x");
    let line = std::str::from_utf8(&buf[..len + 2]).unwrap_or(next.word);
    lexical::line_start(&constructs, line, true).is_some()
}
