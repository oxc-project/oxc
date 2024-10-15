use std::borrow::Cow;

use oxc_ast::{Comment, CommentKind};
use oxc_syntax::identifier::is_irregular_whitespace;

use crate::{JsxRuntime, TransformCtx, TransformOptions};

/// Scan through all comments and find the following pragmas:
///
/// * @jsxRuntime classic / automatic
/// * @jsxImportSource custom-jsx-library
/// * @jsxFrag Preact.Fragment
/// * @jsx Preact.h
///
/// The comment does not need to be a JSDoc comment,
/// otherwise `JSDoc` could be used instead.
///
/// This behavior is aligned with Babel.
pub(crate) fn update_options_with_comments(
    comments: &[Comment],
    options: &mut TransformOptions,
    ctx: &TransformCtx,
) {
    for comment in comments {
        update_options_with_comment(options, comment, ctx.source_text);
    }
}

fn update_options_with_comment(
    options: &mut TransformOptions,
    comment: &Comment,
    source_text: &str,
) {
    let Some((keyword, remainder)) = find_jsx_pragma(comment, source_text) else { return };

    match keyword {
        // @jsx
        "" => {
            // Don't set React option unless React transform is enabled
            // otherwise can cause error in `ReactJsx::new`
            if options.react.jsx_plugin || options.react.development {
                options.react.pragma = Some(remainder.to_string());
            }
            options.typescript.jsx_pragma = Cow::from(remainder.to_string());
        }
        // @jsxRuntime
        "Runtime" => {
            options.react.runtime = match remainder {
                "classic" => JsxRuntime::Classic,
                "automatic" => JsxRuntime::Automatic,
                _ => return,
            };
        }
        // @jsxImportSource
        "ImportSource" => {
            options.react.import_source = Some(remainder.to_string());
        }
        // @jsxFrag
        "Frag" => {
            // Don't set React option unless React transform is enabled
            // otherwise can cause error in `ReactJsx::new`
            if options.react.jsx_plugin || options.react.development {
                options.react.pragma_frag = Some(remainder.to_string());
            }
            options.typescript.jsx_pragma_frag = Cow::from(remainder.to_string());
        }
        _ => {}
    }
}

/// Search comment for a JSX pragma.
///
/// Searches for `@jsx` in the comment.
///
/// If found, returns:
/// * The pragma keyword (not including `jsx` prefix).
/// * The remainder of the comment (with whitespace trimmed off).
///
/// If none found, returns `None`.
fn find_jsx_pragma<'a>(
    comment: &Comment,
    source_text: &'a str,
) -> Option<(/* keyword */ &'a str, /* remainder */ &'a str)> {
    // Strip whitespace and `*`s from start of comment, and find leading `@`.
    // Slice from start of comment to end of file, not end of comment.
    // This allows `find_at_sign` functions to search in chunks of 8 bytes without hitting end of string.
    let comment_str = &source_text[comment.span.start as usize..];
    let comment_str = match comment.kind {
        CommentKind::Line => find_at_sign_in_line_comment(comment_str)?,
        CommentKind::Block => find_at_sign_in_block_comment(comment_str)?,
    };

    // Check next 3 chars after `@` is `jsx`
    let first_3_bytes = comment_str.as_bytes().get(..3)?;
    if first_3_bytes != b"jsx" {
        return None;
    }
    let comment_str = &comment_str[3..];
    // `@jsx` found. `comment_str` contains all source text after `@jsx`

    // Find end of `@` keyword. `keyword` does not include 'jsx' prefix.
    let (keyword, remainder) = split_at_whitespace(comment_str)?;

    // Slice off after end of comment
    let remainder_start = source_text.len() - remainder.len();
    if remainder_start >= comment.span.end as usize {
        // Space was after end of comment
        return None;
    }
    let len = comment.span.end as usize - remainder_start;
    let remainder = &remainder[..len];
    // Trim excess whitespace/line breaks from end
    let remainder = trim_end(remainder);

    Some((keyword, remainder))
}

/// Find `@` character in a single-line comment.
///
/// Returns the remainder of the string after the `@`.
/// Returns `None` if any other character except space, or tab, or irregular whitespace is found first.
/// That includes line breaks, since this is a single-line comment.
fn find_at_sign_in_line_comment(str: &str) -> Option<&str> {
    // Note: Neither `accept` nor `skip` matches line breaks, so will not search beyond end of the comment
    let accept = |byte| byte == b'@';
    let skip = |byte| matches!(byte, b' ' | b'\t');
    let find_unicode = |str: &str| {
        let len = str.len();
        let str = str.trim_start().strip_prefix('@')?;
        Some(len - str.len() - 1)
    };
    let index = find(str, accept, skip, find_unicode)?;
    Some(&str[index + 1..])
}

/// Find `@` character in a block comment.
///
/// Returns the remainder of the string after the `@`.
/// Returns `None` if any other character except whitespace, line breaks, or `*` is found first.
///
/// Line breaks and `*` are allowed in order to handle e.g.:
/// ```js
/// /*
///  * @jsx Preact.h
///  */
/// ```
fn find_at_sign_in_block_comment(str: &str) -> Option<&str> {
    // Note: Neither `accept` nor `skip` matches `/`, so will not search beyond end of the comment
    let accept = |byte| byte == b'@';
    let skip = |byte| byte == b'*' || is_ascii_whitespace(byte);
    let find_unicode = |str: &str| {
        let len = str.len();
        let mut str = str.trim_start();
        // Strip leading jsdoc comment `*` and then whitespaces
        while let Some(cur_str) = str.strip_prefix('*') {
            str = cur_str.trim_start();
        }
        let str = str.strip_prefix('@')?;
        Some(len - str.len() - 1)
    };
    let index = find(str, accept, skip, find_unicode)?;
    Some(&str[index + 1..])
}

/// Split string into 2 parts on spaces, tabs, or irregular whitespaces.
/// Removes any amount of whitespace between the 2 parts.
/// Returns `None` if no whitespace found, or if no further characters after the whitespace.
fn split_at_whitespace(str: &str) -> Option<(&str, &str)> {
    // Find first space, tab, or irregular whitespace
    let mut space_bytes = 1;
    let accept = |byte| matches!(byte, b' ' | b'\t');
    let skip = |_| true;
    let find_unicode = |str: &str| {
        str.find(|c| {
            if c == ' ' || c == '\t' {
                true
            } else if is_irregular_whitespace(c) {
                space_bytes = c.len_utf8();
                true
            } else {
                false
            }
        })
    };
    let space_index = find(str, accept, skip, find_unicode)?;

    let before = &str[..space_index];
    let after_space_index = space_index + space_bytes;

    // Consume any further spaces.
    // Don't use `find` to search in chunks here, as usually there's only a single space and this loop
    // will exit on first turn.
    let more_spaces_after;
    let mut iter = str.as_bytes()[after_space_index..].iter().enumerate();
    loop {
        if let Some((index, &byte)) = iter.next() {
            more_spaces_after = match byte {
                b' ' | b'\t' => continue,
                _ if byte.is_ascii() => index,
                _ => cold_branch(|| {
                    let is_space = |c| c == ' ' || c == '\t' || is_irregular_whitespace(c);
                    str[after_space_index..].find(|c| !is_space(c)).unwrap_or(0)
                }),
            };
            break;
        }
        return None;
    }
    let after = &str[after_space_index + more_spaces_after..];

    Some((before, after))
}

/// Trim whitespace and line breaks from end of string.
///
/// Equivalent to `str::trim_end`, but optimized for ASCII strings.
/// Comparison: <https://godbolt.org/z/4nfW6183z>
fn trim_end(str: &str) -> &str {
    let mut iter = str.as_bytes().iter().enumerate().rev();
    let index = loop {
        if let Some((index, &byte)) = iter.next() {
            match byte {
                _ if is_ascii_whitespace(byte) => continue,
                _ if !byte.is_ascii() => return cold_branch(|| str.trim_end()),
                _ => break index,
            }
        }
        return "";
    };

    // SAFETY: `index` came from a safe iterator, so must be before end of `str`.
    // Therefore `index + 1` must be in bounds (or at end of string).
    // We have only seen ASCII bytes, so `index + 1` must be on a UTF-8 char boundary.
    #[expect(clippy::range_plus_one)]
    unsafe {
        str.get_unchecked(..index + 1)
    }
}

/// Test if a byte is ASCII whitespace, using the same group of ASCII chars that `std::str::trim_start` uses.
/// These the are ASCII chars which `char::is_whitespace` returns `true` for.
/// Note: Slightly different from `u8::is_ascii_whitespace`, which does not include VT.
/// <https://doc.rust-lang.org/std/primitive.u8.html#method.is_ascii_whitespace>
#[inline]
fn is_ascii_whitespace(byte: u8) -> bool {
    const VT: u8 = 0x0B;
    const FF: u8 = 0x0C;
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | VT | FF)
}

/// Find a match in a string.
///
/// Optimized for searching through strings which only contain ASCII.
/// Non-ASCII chars are considered unlikely and are handled in a cold fallback path.
///
/// Search occurs in batches of 8 bytes, with a slower fallback for the last 7 bytes.
/// Provide the longest string possible to be able to avoid the slower fallback.
///
/// Iterates through string byte-by-byte, calling `accept` and `skip` for each byte.
/// * If a non-ASCII byte is found, hands control to `find_unicode` and returns whatever it returns.
/// * If `accept` returns `true`, this function returns the index of that byte.
/// * If `skip` returns `true`, continues search.
/// * If both `accept` and `skip` return `false`, this function returns `None`.
/// * If reaches the end of the string without exiting, returns `None`.
fn find<'s, Accept, Skip, FindUnicode>(
    str: &'s str,
    accept: Accept,
    skip: Skip,
    find_unicode: FindUnicode,
) -> Option<usize>
where
    Accept: Fn(u8) -> bool,
    Skip: Fn(u8) -> bool,
    FindUnicode: FnOnce(&'s str) -> Option<usize>,
{
    // Process string in chunks of 8 bytes.
    // Check chunks for any non-ASCII bytes in one go, and deopt to unicode handler if so.
    let mut chunks = str.as_bytes().chunks_exact(8);
    for (chunk_index, chunk) in chunks.by_ref().enumerate() {
        let chunk: [u8; 8] = chunk.try_into().unwrap();
        if !chunk_is_ascii(chunk) {
            return cold_branch(|| find_unicode(str));
        }

        // Compiler will unroll this loop if `accept` and `skip` are small enough
        for (byte_index, byte) in chunk.into_iter().enumerate() {
            match byte {
                _ if accept(byte) => return Some(chunk_index * 8 + byte_index),
                _ if skip(byte) => continue,
                _ => return None,
            }
        }
    }

    // We only get here if we're close to end of the string
    let chunk_start = str.len() & !7;
    for (byte_index, &byte) in chunks.remainder().iter().enumerate() {
        match byte {
            _ if !byte.is_ascii() => return cold_branch(|| find_unicode(str)),
            _ if accept(byte) => return Some(chunk_start + byte_index),
            _ if skip(byte) => continue,
            _ => return None,
        }
    }

    None
}

#[inline]
fn chunk_is_ascii(chunk: [u8; 8]) -> bool {
    const HIGH_BITS: u64 = 0x8080_8080_8080_8080;
    let chunk_u64 = u64::from_ne_bytes(chunk);
    chunk_u64 & HIGH_BITS == 0
}

/// Call a closure while hinting to compiler that this branch is rarely taken.
/// "Cold trampoline function", suggested in:
/// <https://users.rust-lang.org/t/is-cold-the-only-reliable-way-to-hint-to-branch-predictor/106509/2>
#[cold]
#[inline(never)]
pub fn cold_branch<F: FnOnce() -> T, T>(f: F) -> T {
    f()
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_ast::CommentPosition;
    use oxc_span::Span;

    static PRE_AND_POSTFIX: &[(&str, &str)] = &[
        ("", ""),
        ("\n\n\n", "\n"),
        ("", "\n@jsx AfterCommentWeShouldNotFind\n\n"),
        ("\n\n\n@jsx BeforeCommentWeShouldNotFind\n\n", ""),
        ("\n\n\n@jsx BeforeCommentWeShouldNotFind\n\n", "\n@jsx AfterCommentWeShouldNotFind\n\n"),
    ];

    fn run_tests<'c>(cases: impl Iterator<Item = (&'c str, Option<(&'c str, &'c str)>)>) {
        for (comment_str, expected) in cases {
            for (before, after) in PRE_AND_POSTFIX {
                let (comment, source_text) = create_comment(comment_str, before, after);
                assert_eq!(find_jsx_pragma(&comment, &source_text), expected);
            }
        }
    }

    fn create_comment(comment_str: &str, before: &str, after: &str) -> (Comment, String) {
        let (kind, end_bytes) = if comment_str.starts_with("//") {
            (CommentKind::Line, 0)
        } else {
            assert!(comment_str.starts_with("/*") && comment_str.ends_with("*/"));
            (CommentKind::Block, 2)
        };

        let source_text = format!("{before}{comment_str}{after}");
        #[expect(clippy::cast_possible_truncation)]
        let span = Span::new(
            (before.len() + 2) as u32,
            (before.len() + comment_str.len() - end_bytes) as u32,
        );
        let comment = Comment {
            span,
            kind,
            position: CommentPosition::Leading,
            attached_to: 0,
            preceded_by_newline: true,
            followed_by_newline: true,
        };
        (comment, source_text)
    }

    #[test]
    fn find_jsx_pragma_line_comment_not_found() {
        let cases = [
            // No `@`
            "//",
            "// ",
            "// blah blah blah",
            "//              blah blah blah",
            "//          ",
            // `@` but not valid
            "//@",
            "// @",
            "// @ ",
            "// @j",
            "// @j ",
            "// @j sx",
            "// @j sx ",
            "// @js",
            "// @js ",
            "// @js x",
            "// @js blah",
            "// @ jsx blah",
            "// @    jsx blah",
            "// @xjsx blah",
            "//              @xjsx blah",
            "//              @xjsx                 blah",
            "// @jsx",
            "// @jsx ",
            "// @jsx        ",
            "// @jsxX",
            "// @jsxRuntime",
            "// @jsxRuntime       ",
            "// @jsxImportSource",
            "// @jsxImportSource ",
            "// @jsxFrag",
            "// @jsxFrag ",
            // Unicode space
            "//\u{85}",
            "//    \u{85}   ",
        ];

        run_tests(cases.into_iter().map(|comment_str| (comment_str, None)));
    }

    #[test]
    fn find_jsx_pragma_line_comment_found() {
        let cases = [
            // comment, keyword, remainder
            // `@jsx` pragma
            ("//@jsx foo", "", "foo"),
            ("// @jsx foo", "", "foo"),
            ("//     @jsx       foo", "", "foo"),
            ("//\t@jsx foo", "", "foo"),
            ("//  \t\t      \t\t    @jsx foo", "", "foo"),
            ("// @jsx\tfoo", "", "foo"),
            ("// @jsx\t  \t  \t\t foo", "", "foo"),
            ("// @jsx foo ", "", "foo"),
            ("// @jsx foo\t", "", "foo"),
            ("// @jsx foo             \t\t      \t\t     ", "", "foo"),
            // Other pragmas
            ("// @jsxRuntime foo", "Runtime", "foo"),
            ("// @jsxRuntime         \t\t\t     foo", "Runtime", "foo"),
            ("// @jsxRuntime         \t\t\t     foo      \t\t\t     ", "Runtime", "foo"),
            ("// @jsxImportSource foo", "ImportSource", "foo"),
            ("// @jsxFrag foo", "Frag", "foo"),
            // Unicode space
            ("//\u{85}@jsx foo", "", "foo"),
            ("//\u{85}\t\u{85}@jsx foo", "", "foo"),
            ("// @jsx\u{85}foo", "", "foo"),
            ("// @jsx\u{85}   foo", "", "foo"),
            ("// @jsx   \u{85}foo", "", "foo"),
            ("// @jsx\u{85}   \u{85}foo", "", "foo"),
            ("// @jsx\u{85}\u{85}\u{85}foo", "", "foo"),
            ("// @jsx foo\u{85}", "", "foo"),
            ("// @jsx foo\u{85}   ", "", "foo"),
            ("// @jsx foo   \u{85}", "", "foo"),
            ("// @jsx foo\u{85}   \u{85}", "", "foo"),
            ("// @jsx foo\u{85}\u{85}\u{85}", "", "foo"),
        ];

        run_tests(
            cases
                .into_iter()
                .map(|(comment_str, keyword, remainder)| (comment_str, Some((keyword, remainder)))),
        );
    }

    #[test]
    fn find_jsx_pragma_block_comment_not_found() {
        let cases = [
            // No `@`
            "/**/",
            "/* */",
            "/* blah blah blah*/",
            "/*              blah blah blah*/",
            "/*          */",
            // `@` but not valid
            "/*@*/",
            "/* @*/",
            "/*@ */",
            "/* @ */",
            "/* @j*/",
            "/* @j */",
            "/* @j sx */",
            "/* @js*/",
            "/* @js */",
            "/* @js x*/",
            "/* @js x */",
            "/* @js blah */",
            "/* @ jsx blah */",
            "/* @    jsx blah */",
            "/* @xjsx blah */",
            "/*              @xjsx blah */",
            "/*              @xjsx                 blah */",
            "/*@jsx*/",
            "/* @jsx*/",
            "/* @jsx */",
            "/* @jsx        */",
            "/* @jsxX */",
            "/* @jsxRuntime*/",
            "/* @jsxRuntime       */",
            "/* @jsxImportSource*/",
            "/* @jsxImportSource */",
            "/* @jsxFrag*/",
            "/* @jsxFrag */",
            // Multi-line
            "/*\n*/",
            "/*
              */",
            "/*
              *
              */",
            "/*
              * @jsx
              */",
            "/*
              * @jsxX
              */",
            "/*
              * @js
              */",
            // Unicode space
            "/*\u{85}*/",
            "/*    \u{85}   */",
        ];

        run_tests(cases.into_iter().map(|comment_str| (comment_str, None)));
    }

    #[test]
    fn find_jsx_pragma_block_comment_found() {
        let cases = [
            // comment, keyword, remainder
            // `@jsx` pragma single line
            ("/*@jsx foo*/", "", "foo"),
            ("/* @jsx foo*/", "", "foo"),
            ("/*     @jsx       foo*/", "", "foo"),
            ("/*\t@jsx foo*/", "", "foo"),
            ("/*  \t\t      \t\t    @jsx foo*/", "", "foo"),
            ("/* @jsx\tfoo*/", "", "foo"),
            ("/* @jsx\t  \t  \t\t foo*/", "", "foo"),
            ("/* @jsx foo */", "", "foo"),
            ("/* @jsx foo\t*/", "", "foo"),
            ("/* @jsx foo             \t\t      \t\t     */", "", "foo"),
            // `@jsx` pragma multi line
            (
                "/*
                   * @jsx foo
                   */",
                "",
                "foo",
            ),
            (
                "/*
                   * @jsx foo*/",
                "",
                "foo",
            ),
            (
                "/* @jsx foo
                   */",
                "",
                "foo",
            ),
            (
                "/*
                   *
                   *
                   * @jsx foo
                   */",
                "",
                "foo",
            ),
            // Other pragmas
            ("/* @jsxRuntime foo*/", "Runtime", "foo"),
            ("/* @jsxRuntime foo */", "Runtime", "foo"),
            ("/* @jsxRuntime         \t\t\t     foo*/", "Runtime", "foo"),
            ("/* @jsxRuntime         \t\t\t     foo      \t\t\t     */", "Runtime", "foo"),
            ("/* @jsxImportSource foo */", "ImportSource", "foo"),
            ("/* @jsxFrag foo */", "Frag", "foo"),
            // Unicode space
            ("/*\u{85}@jsx foo*/", "", "foo"),
            ("/*\u{85}\t\u{85}@jsx foo*/", "", "foo"),
            ("/* @jsx\u{85}foo*/", "", "foo"),
            ("/* @jsx\u{85}   foo*/", "", "foo"),
            ("/* @jsx   \u{85}foo*/", "", "foo"),
            ("/* @jsx\u{85}   \u{85}foo*/", "", "foo"),
            ("/* @jsx\u{85}\u{85}\u{85}foo*/", "", "foo"),
            ("/* @jsx foo\u{85}*/", "", "foo"),
            ("/* @jsx foo\u{85}   */", "", "foo"),
            ("/* @jsx foo   \u{85}*/", "", "foo"),
            ("/* @jsx foo\u{85}   \u{85}*/", "", "foo"),
            ("/* @jsx foo\u{85}\u{85}\u{85}*/", "", "foo"),
        ];

        run_tests(
            cases
                .into_iter()
                .map(|(comment_str, keyword, remainder)| (comment_str, Some((keyword, remainder)))),
        );
    }

    #[test]
    fn test_trim_end() {
        let cases = [
            // Empty
            ("", ""),
            (" ", ""),
            ("\t", ""),
            ("\r", ""),
            ("\n", ""),
            ("\u{0B}", ""),
            ("\u{0C}", ""),
            ("   \t \n  \r\n \u{0B} \u{0C}   ", ""),
            // Single char
            ("a", "a"),
            ("a ", "a"),
            ("a\t", "a"),
            ("a\r", "a"),
            ("a\n", "a"),
            ("a\u{0B}", "a"),
            ("a\u{0C}", "a"),
            ("a   \t \n  \r\n \u{0B} \u{0C}   ", "a"),
            // Multiple chars
            ("abc", "abc"),
            ("abc ", "abc"),
            ("abc\t", "abc"),
            ("abc\r", "abc"),
            ("abc\n", "abc"),
            ("abc\u{0B}", "abc"),
            ("abc\u{0C}", "abc"),
            ("abc   \t \n  \r\n \u{0B} \u{0C}   ", "abc"),
            // Unicode whitespace
            ("\u{85}", ""),
            ("\u{85}\u{85}\u{85}", ""),
            ("a\u{85}", "a"),
            ("a\u{85}\u{85}\u{85}", "a"),
            ("abc\u{85}", "abc"),
            ("abc\u{85}\u{85}\u{85}", "abc"),
            // Spaces on start
            ("   abc", "   abc"),
            ("   abc   ", "   abc"),
        ];

        for (str, expected) in cases {
            assert_eq!(trim_end(str), expected);
        }
    }
}
