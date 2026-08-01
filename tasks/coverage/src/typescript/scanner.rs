//! Ports of TypeScript's scanner / line-position helpers used by the symbol/type/error
//! baseline harnesses.
//!
//! Mirrors typescript-go `internal/scanner` (`ComputeECMALineStarts`,
//! `GetECMALineOfPosition`, `GetECMALineAndUTF16CharacterOfPosition`) and the regexes in
//! `internal/testutil/tsbaseline/{type_symbol_baseline,error_baseline}.go`.
//!
//! oxc `Span`s are UTF-8 byte offsets, so every offset here is a byte offset. ECMA line
//! terminators are `\n`, `\r`, `\r\n` (a single break), `U+2028` (LS) and `U+2029` (PS).
//! Columns are counted in UTF-16 code units to match
//! `GetECMALineAndUTF16CharacterOfPosition`.

use lazy_regex::{Lazy, Regex, lazy_regex};

/// `bracketLineRegex` in `type_symbol_baseline.go`: a line consisting solely of `{` or `}`.
pub static BRACKET_LINE: Lazy<Regex> = lazy_regex!(r"^\s*[{}]\s*$");

/// `lineDelimiter` in typescript-go: `\r?\n`, used to strip line breaks from a node's
/// source text before printing it on a `>` annotation line.
pub static LINE_DELIMITER: Lazy<Regex> = lazy_regex!(r"\r?\n");

/// Is there a `U+2028` (LS) or `U+2029` (PS) encoded at byte `i`? (`E2 80 A8` / `E2 80 A9`.)
fn is_ls_or_ps(bytes: &[u8], i: usize) -> bool {
    i + 2 < bytes.len()
        && bytes[i] == 0xE2
        && bytes[i + 1] == 0x80
        && matches!(bytes[i + 2], 0xA8 | 0xA9)
}

/// Port of `ComputeECMALineStarts`: byte offset where each line begins. `starts[0]` is always
/// `0`; a trailing terminator yields a final (empty) line start at `text.len()`.
#[expect(clippy::cast_possible_truncation)]
pub fn compute_line_starts(text: &str) -> Vec<u32> {
    let bytes = text.as_bytes();
    let mut starts = vec![0u32];
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                i += 1;
                starts.push(i as u32);
            }
            b'\r' => {
                i += 1;
                if i < bytes.len() && bytes[i] == b'\n' {
                    i += 1;
                }
                starts.push(i as u32);
            }
            0xE2 if is_ls_or_ps(bytes, i) => {
                i += 3;
                starts.push(i as u32);
            }
            _ => i += 1,
        }
    }
    starts
}

/// Port of `GetECMALineOfPosition`: 0-based line index of a byte offset.
#[expect(clippy::cast_possible_truncation)]
pub fn line_of_position(line_starts: &[u32], pos: u32) -> u32 {
    // The line is the largest `i` with `line_starts[i] <= pos`.
    (line_starts.partition_point(|&s| s <= pos) - 1) as u32
}

/// Port of `GetECMALineAndUTF16CharacterOfPosition`: (0-based line, 0-based UTF-16 column).
#[expect(clippy::cast_possible_truncation)]
pub fn line_and_character(text: &str, line_starts: &[u32], pos: u32) -> (u32, u32) {
    let line = line_of_position(line_starts, pos);
    let line_start = line_starts[line as usize] as usize;
    let col = text[line_start..pos as usize].encode_utf16().count() as u32;
    (line, col)
}

/// Split `text` into line contents on ECMA terminators (terminators stripped). Port of
/// `codeLinesRegexp.Split`; `\r\n` is treated as one break.
pub fn split_lines(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut lines = Vec::new();
    let mut line_start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                lines.push(&text[line_start..i]);
                i += 1;
                line_start = i;
            }
            b'\r' => {
                lines.push(&text[line_start..i]);
                i += 1;
                if i < bytes.len() && bytes[i] == b'\n' {
                    i += 1;
                }
                line_start = i;
            }
            0xE2 if is_ls_or_ps(bytes, i) => {
                lines.push(&text[line_start..i]);
                i += 3;
                line_start = i;
            }
            _ => i += 1,
        }
    }
    lines.push(&text[line_start..]);
    lines
}

/// Port of `node.Pos()` (trivia-inclusive start = end of the previous token).
///
/// `token_ends` are the ascending end offsets of every lexed token. For a node beginning at
/// byte `start`, its leading trivia begins where the previous token ended, which is the
/// largest token end `<= start`. Comments are trivia, not tokens, so this matches TypeScript.
pub fn full_start(token_ends: &[u32], start: u32) -> u32 {
    let k = token_ends.partition_point(|&e| e <= start);
    if k == 0 { 0 } else { token_ends[k - 1] }
}
