//! Words and whitespace.
//!
//! Text is printed RAW (escapes and entities as written);
//! only the decisions below touch it:
//! - `*` / `_` runs that could open or close emphasis get escaped inside emphasis / strong
//! - a lone `===` / `---` word starting a line gets escaped (it would read as a setext underline)
//! - a line break before a word that would start a block (`-`, `1.`, `#`, `>`) never breaks
//!
//! TODO: CJK word kinds and the `""` whitespace between CJ characters.

use std::borrow::Cow;

use oxc_formatter_core::arena_cow_str;
use oxc_markdown_parser::unicode;

use crate::options::ProseWrap;

use super::{
    MarkdownFormatter,
    inline::{Parts, Sep},
};

/// The word after a whitespace, when the next sibling is a text.
#[derive(Clone, Copy)]
pub struct NextWord<'a> {
    pub word: &'a str,
    /// The word is its text's only one.
    pub alone_on_line: bool,
}

#[derive(Clone, Copy, Default)]
pub struct TextContext<'a> {
    /// The text is the first child of an `Emphasis` / `Strong`.
    pub first_of_delimiter: bool,
    /// The previous sibling is a soft break (the text starts a source line).
    pub after_soft_break: bool,
    /// The first word of the next sibling text, for the whitespace that ends this text.
    pub next_word: Option<NextWord<'a>>,
    /// Inside a reference link's raw content, where a line break never prevents a break.
    pub is_link: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Ws {
    Space,
    Newline,
}

/// Word separators: space, tab, newline.
pub fn is_split_whitespace(c: char) -> bool {
    matches!(c, '\t' | '\n' | ' ')
}

/// Splits `raw` into words and whitespace and pushes them, in one pass.
pub fn push_text<'a>(
    raw: &'a str,
    cx: TextContext<'a>,
    parts: &mut Parts<'a>,
    f: &MarkdownFormatter<'_, 'a>,
) {
    let in_delimiter = f.context().delimiter_depth().get() > 0;
    let prose_wrap = f.options().prose_wrap;

    let mut rest = raw;
    let mut is_first = true;
    let leading_ws = raw.starts_with(is_split_whitespace);
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
                Some(NextWord { word, alone_on_line: is_first && is_last })
            };
            push_whitespace(ws_kind(ws), &TextContext { next_word: next, ..cx }, parts, f);
        }
        if word.is_empty() {
            break;
        }
        let printed: Cow<'a, str> = if in_delimiter {
            let prev = if is_first { None } else { Some(' ') };
            let next = if is_last { None } else { Some(' ') };
            print_delimited_word(word, cx.first_of_delimiter && is_first && !leading_ws, prev, next)
        } else if prose_wrap == ProseWrap::Preserve
            && is_first
            && is_last
            && !leading_ws
            && cx.after_soft_break
            && is_fake_setext_underline(word)
        {
            Cow::Owned(format!("\\{word}"))
        } else {
            Cow::Borrowed(word)
        };
        parts.push_str(arena_cow_str(&printed, f));
        rest = tail;
        is_first = false;
    }
}

fn ws_kind(ws: &str) -> Ws {
    if ws.contains('\n') { Ws::Newline } else { Ws::Space }
}

/// A whitespace as a separator (may break) or a space.
pub fn push_whitespace<'a>(
    ws: Ws,
    cx: &TextContext<'a>,
    parts: &mut Parts<'a>,
    f: &MarkdownFormatter<'_, 'a>,
) {
    let options = f.options();
    let prose_wrap = if !cx.is_link && prevents_break(ws, cx.next_word, options.prose_wrap) {
        ProseWrap::Never
    } else {
        options.prose_wrap
    };

    if prose_wrap == ProseWrap::Preserve && ws == Ws::Newline {
        parts.push_sep(Sep::HardLine);
        return;
    }
    // TODO: between CJ characters the whitespace is `""` and a `\n` is not a space
    if prose_wrap == ProseWrap::Always && f.context().no_wrap_depth().get() == 0 {
        parts.push_sep(Sep::Line);
    } else {
        parts.push_str(" ");
    }
}

/// Never break before a word that would start a block.
fn prevents_break(ws: Ws, next: Option<NextWord<'_>>, prose_wrap: ProseWrap) -> bool {
    let Some(next) = next else { return false };
    if !looks_like_block_start(next.word) {
        return false;
    }
    // A lone `-` after a newline is going to be escaped as a fake setext underline instead
    !(prose_wrap == ProseWrap::Preserve
        && ws == Ws::Newline
        && next.word == "-"
        && next.alone_on_line)
}

/// `>`-led, or exactly `*` / `+` / `-`, 1-6 `#`, or digits then `.` / `)`.
fn looks_like_block_start(word: &str) -> bool {
    if word.starts_with('>') || matches!(word, "*" | "+" | "-") {
        return true;
    }
    if !word.is_empty() && word.len() <= 6 && word.bytes().all(|b| b == b'#') {
        return true;
    }
    let digits = word.trim_end_matches(['.', ')']);
    digits.len() + 1 == word.len()
        && !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit())
}

/// `^(?:=+|-+)$`
fn is_fake_setext_underline(word: &str) -> bool {
    !word.is_empty() && (word.bytes().all(|b| b == b'=') || word.bytes().all(|b| b == b'-'))
}

/// A word inside emphasis / strong, with the runs that could end it escaped.
///
/// `prev` / `next` are the characters around the word within its text; `None` at the text edges,
/// where neighbours across nodes are not consulted (as in Prettier) and nothing is escaped.
fn print_delimited_word<'a>(
    word: &'a str,
    escape_leading: bool,
    prev: Option<char>,
    next: Option<char>,
) -> Cow<'a, str> {
    if !word.contains(['*', '_']) {
        return Cow::Borrowed(word);
    }
    let text: Cow<'a, str> = if escape_leading && (word.starts_with('*') || word.starts_with('_')) {
        Cow::Owned(format!("\\{word}"))
    } else {
        Cow::Borrowed(word)
    };
    escape_delimiter_runs(text, prev, next)
}

/// Escapes every `*` / `_` run that could open or close emphasis where it stands.
///
/// One rule for every run
/// (Prettier's regex scan skips a run right after another run's following character;
/// that gap is not reproduced).
/// A run already behind an odd number of backslashes is escaped as written.
fn escape_delimiter_runs(
    text: Cow<'_, str>,
    prev: Option<char>,
    next: Option<char>,
) -> Cow<'_, str> {
    let bytes = text.as_bytes();
    let mut out: Option<String> = None;
    let mut emitted = 0;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b != b'*' && b != b'_' {
            i += 1;
            continue;
        }
        // `*`, `_`, `\` are ASCII, so byte positions here are char boundaries
        let run_start = i;
        while i < bytes.len() && bytes[i] == b {
            i += 1;
        }
        let backslashes = bytes[..run_start].iter().rev().take_while(|&&c| c == b'\\').count();
        let already_escaped = backslashes % 2 == 1;
        let preceding = text[..run_start].chars().next_back().or(prev);
        let following = text[i..].chars().next().or(next);
        if !already_escaped && can_open_or_close(preceding, b as char, following) == Some(true) {
            let out = out.get_or_insert_with(|| String::with_capacity(text.len() + 2));
            out.push_str(&text[emitted..run_start]);
            out.push('\\');
            emitted = run_start;
        }
    }
    match out {
        None => text,
        Some(mut out) => {
            out.push_str(&text[emitted..]);
            Cow::Owned(out)
        }
    }
}

/// Whether a run of `indicator` could open or close emphasis here (CommonMark flanking);
/// `None` when a side is unknown.
fn can_open_or_close(
    preceding: Option<char>,
    indicator: char,
    following: Option<char>,
) -> Option<bool> {
    let (preceding, following) = (preceding?, following?);
    let followed_by_whitespace = unicode::is_whitespace(following);
    let preceded_by_whitespace = unicode::is_whitespace(preceding);
    let followed_by_punctuation = unicode::is_punctuation(following);
    let preceded_by_punctuation = unicode::is_punctuation(preceding);

    let left_flanking = !followed_by_whitespace
        && (!followed_by_punctuation || preceded_by_whitespace || preceded_by_punctuation);
    let right_flanking = !preceded_by_whitespace
        && (!preceded_by_punctuation || followed_by_whitespace || followed_by_punctuation);

    if indicator == '*' {
        return Some(left_flanking || right_flanking);
    }
    if left_flanking {
        return Some(!right_flanking || preceded_by_punctuation);
    }
    if right_flanking {
        return Some(!left_flanking || followed_by_punctuation);
    }
    Some(false)
}
