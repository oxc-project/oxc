//! Escapes, the only places text is changed (AGENTS.md "Escaping"): pure functions over one word.

use std::borrow::Cow;

use oxc_markdown_parser::attention;

/// An odd run of `\` ends the word.
pub fn ends_with_unescaped_backslash(word: &str) -> bool {
    word.bytes().rev().take_while(|&b| b == b'\\').count() % 2 == 1
}

/// `^(?:=+|-+)$`
pub fn is_fake_setext_underline(word: &str) -> bool {
    !word.is_empty() && (word.bytes().all(|b| b == b'=') || word.bytes().all(|b| b == b'-'))
}

/// A word inside emphasis / strong, with the runs that could end it escaped.
///
/// `prev` / `next` are the characters around the word within its text;
/// `None` at the text edges, where neighbours across nodes are not consulted (as in Prettier) and nothing is escaped.
pub fn print_delimited_word<'a>(
    word: &'a str,
    escape_leading: bool,
    autolink_stretch: usize,
    prev: Option<char>,
    next: Option<char>,
) -> Cow<'a, str> {
    if !word.contains(['*', '_']) {
        return Cow::Borrowed(word);
    }
    let text: Cow<'a, str> =
        if escape_leading { Cow::Owned(format!("\\{word}")) } else { Cow::Borrowed(word) };
    escape_delimiter_runs(text, autolink_stretch, prev, next)
}

/// Escapes every `*` / `_` run that could open or close emphasis where it stands, character by character
/// (a single backslash only shortens the run, and the rest can still pair: Prettier's `\***`).
///
/// One rule for every run,
/// Prettier's regex scan skips a run right after another run's following character;
/// that gap is not reproduced.
fn escape_delimiter_runs(
    text: Cow<'_, str>,
    autolink_stretch: usize,
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
        // A backslash escapes one character: the rest of the run is a run of its own
        let run_start = run_start + usize::from(ends_with_unescaped_backslash(&text[..run_start]));
        if run_start == i || run_start < autolink_stretch {
            continue;
        }
        let preceding = text[..run_start].chars().next_back().or(prev);
        let following = text[i..].chars().next().or(next);
        if can_open_or_close(preceding, b as char, following) == Some(true) {
            // Every character: escaping only the first leaves a shorter run that can still pair
            let out = out.get_or_insert_with(|| String::with_capacity(text.len() + 2));
            out.push_str(&text[emitted..run_start]);
            for _ in run_start..i {
                out.push('\\');
                out.push(b as char);
            }
            emitted = i;
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

/// Whether the `*` / `_` run a word starts with is escaped where it stands (`escape_delimiter_runs`),
/// so that the printed word starts with a backslash. `prev` / `next` as for the run's neighbors.
pub fn leading_run_escaped(word: &str, prev: Option<char>, next: Option<char>) -> bool {
    let Some(&marker) = word.as_bytes().first().filter(|b| matches!(b, b'*' | b'_')) else {
        return false;
    };
    let run = word.bytes().take_while(|&b| b == marker).count();
    let following = word[run..].chars().next().or(next);
    can_open_or_close(prev, marker as char, following) == Some(true)
}

/// Whether a run of `indicator` could open or close emphasis here,
/// by the parser's own rule (micromark's flanking, marker-adjacency relaxation included);
/// `None` when a side is untouchable (an autolink, whose extent an escape would change).
fn can_open_or_close(
    preceding: Option<char>,
    indicator: char,
    following: Option<char>,
) -> Option<bool> {
    let (preceding, following) = (preceding?, following?);
    let (open, close) =
        attention::classify(indicator as u8, Some(preceding), Some(following), true);
    Some(open || close)
}
