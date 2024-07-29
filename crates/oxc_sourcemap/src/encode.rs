use std::borrow::Cow;

#[cfg(feature = "concurrent")]
use rayon::prelude::*;

use crate::JSONSourceMap;
/// Port from https://github.com/getsentry/rust-sourcemap/blob/master/src/encoder.rs
/// It is a helper for encode `SourceMap` to vlq sourcemap string, but here some different.
/// - Quote `source_content` at parallel.
/// - If you using `ConcatSourceMapBuilder`, serialize `tokens` to vlq `mappings` at parallel.
use crate::{token::TokenChunk, SourceMap, Token};

pub fn encode(sourcemap: &SourceMap) -> JSONSourceMap {
    JSONSourceMap {
        file: sourcemap.get_file().map(ToString::to_string),
        mappings: Some(serialize_sourcemap_mappings(sourcemap)),
        source_root: sourcemap.get_source_root().map(ToString::to_string),
        sources: Some(sourcemap.sources.iter().map(ToString::to_string).map(Some).collect()),
        sources_content: sourcemap
            .source_contents
            .as_ref()
            .map(|x| x.iter().map(ToString::to_string).map(Some).collect()),
        names: Some(sourcemap.names.iter().map(ToString::to_string).collect()),
    }
}

// Here using `serde_json` to serialize `names` / `source_contents` / `sources`.
// It will escape the string to avoid invalid JSON string.
pub fn encode_to_string(sourcemap: &SourceMap) -> String {
    let max_segments = 12
        + sourcemap.names.len() * 2
        + sourcemap.sources.len() * 2
        + sourcemap.source_contents.as_ref().map_or(0, |sources| sources.len() * 2 + 1)
        + sourcemap.x_google_ignore_list.as_ref().map_or(0, |x| x.len() * 2 + 1);
    let mut contents = PreAllocatedString::new(max_segments);

    contents.push("{\"version\":3,".into());
    if let Some(file) = sourcemap.get_file() {
        contents.push("\"file\":\"".into());
        contents.push(file.into());
        contents.push("\",".into());
    }

    if let Some(source_root) = sourcemap.get_source_root() {
        contents.push("\"sourceRoot\":\"".into());
        contents.push(source_root.into());
        contents.push("\",".into());
    }

    contents.push("\"names\":[".into());
    contents.push_list(sourcemap.names.iter(), escape_json_string);

    contents.push("],\"sources\":[".into());
    contents.push_list(sourcemap.sources.iter(), escape_json_string);

    // Quote `source_content` in parallel
    if let Some(source_contents) = &sourcemap.source_contents {
        contents.push("],\"sourcesContent\":[".into());
        cfg_if::cfg_if! {
            if #[cfg(feature = "concurrent")] {
                let quoted_source_contents: Vec<_> = source_contents
                    .par_iter()
                    .map(|source| {
                        let mut thread_contents = PreAllocatedString::new(source.len());
                        escape_json_string(source, &mut thread_contents);
                        thread_contents
                    })
                    .collect();
                contents.push_list(quoted_source_contents.into_iter(), |thread_contents, contents| {
                    contents.extend_from(thread_contents);
                });
            } else {
                contents.push_list(source_contents.iter(), escape_json_string);
            }
        };
    }

    if let Some(x_google_ignore_list) = &sourcemap.x_google_ignore_list {
        contents.push("],\"x_google_ignoreList\":[".into());
        contents.push_list(x_google_ignore_list.iter(), |n, contents| {
            contents.push(Cow::Owned(n.to_string()));
        });
    }

    contents.push("],\"mappings\":\"".into());
    contents.push(serialize_sourcemap_mappings(sourcemap).into());
    contents.push("\"}".into());

    contents.consume()
}

#[allow(clippy::cast_possible_truncation)]
fn serialize_sourcemap_mappings(sm: &SourceMap) -> String {
    sm.token_chunks.as_ref().map_or_else(
        || {
            serialize_mappings(
                &sm.tokens,
                &TokenChunk::new(0, sm.tokens.len() as u32, 0, 0, 0, 0, 0, 0),
            )
        },
        |token_chunks| {
            // Serialize `tokens` to vlq `mappings` at parallel.
            cfg_if::cfg_if! {
                if #[cfg(feature = "concurrent")] {
                    token_chunks
                        .par_iter()
                        .map(|token_chunk| serialize_mappings(&sm.tokens, token_chunk))
                        .collect::<String>()
                } else {
                    token_chunks
                        .iter()
                        .map(|token_chunk| serialize_mappings(&sm.tokens, token_chunk))
                        .collect::<String>()
                }
            }
        },
    )
}

fn serialize_mappings(tokens: &[Token], token_chunk: &TokenChunk) -> String {
    let TokenChunk {
        start,
        end,
        mut prev_dst_line,
        mut prev_dst_col,
        mut prev_src_line,
        mut prev_src_col,
        mut prev_name_id,
        mut prev_source_id,
    } = *token_chunk;

    let capacity = ((end - start) * 10) as usize;

    let mut rv = String::with_capacity(capacity);

    for (idx, token) in tokens[start as usize..end as usize].iter().enumerate() {
        let index = start as usize + idx;
        if token.get_dst_line() != prev_dst_line {
            prev_dst_col = 0;
            while token.get_dst_line() != prev_dst_line {
                rv.push(';');
                prev_dst_line += 1;
            }
        } else if index > 0 {
            if Some(token) == tokens.get(index - 1) {
                continue;
            }
            rv.push(',');
        }

        encode_vlq_diff(&mut rv, token.get_dst_col(), prev_dst_col);
        prev_dst_col = token.get_dst_col();

        if let Some(source_id) = token.get_source_id() {
            encode_vlq_diff(&mut rv, source_id, prev_source_id);
            prev_source_id = source_id;
            encode_vlq_diff(&mut rv, token.get_src_line(), prev_src_line);
            prev_src_line = token.get_src_line();
            encode_vlq_diff(&mut rv, token.get_src_col(), prev_src_col);
            prev_src_col = token.get_src_col();
            if let Some(name_id) = token.get_name_id() {
                encode_vlq_diff(&mut rv, name_id, prev_name_id);
                prev_name_id = name_id;
            }
        }
    }

    rv
}

#[inline]
fn encode_vlq_diff(out: &mut String, a: u32, b: u32) {
    encode_vlq(out, i64::from(a) - i64::from(b));
}

const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn encode_vlq(out: &mut String, num: i64) {
    let mut num = if num < 0 { ((-num) << 1) + 1 } else { num << 1 };

    loop {
        let mut digit = num & 0b11111;
        num >>= 5;
        if num > 0 {
            digit |= 1 << 5;
        }
        out.push(B64_CHARS[digit as usize] as char);
        if num == 0 {
            break;
        }
    }
}

/// A helper for pre-allocate string buffer.
///
/// Pre-allocate a Cow<'a, str> buffer, and push the segment into it.
/// Finally, convert it to a pre-allocated length String.
struct PreAllocatedString<'a> {
    buf: Vec<Cow<'a, str>>,
    len: usize,
}

impl<'a> PreAllocatedString<'a> {
    fn new(max_segments: usize) -> Self {
        Self { buf: Vec::with_capacity(max_segments), len: 0 }
    }

    #[inline]
    fn push(&mut self, s: Cow<'a, str>) {
        self.len += s.len();
        self.buf.push(s);
    }

    #[inline]
    fn push_list<S, I, P>(&mut self, mut iter: I, pusher: P)
    where
        I: Iterator<Item = S>,
        P: Fn(S, &mut Self) -> (),
    {
        let Some(first) = iter.next() else {
            return;
        };
        pusher(first, self);

        for other in iter {
            self.push(Cow::Borrowed(","));
            pusher(other, self);
        }
    }

    #[allow(dead_code)] // Only used when `concurrent` mode enabled
    fn extend_from(&mut self, mut other: Self) {
        self.buf.extend(other.buf.drain(..));
    }

    #[inline]
    fn consume(self) -> String {
        let mut buf = String::with_capacity(self.len);
        buf.extend(self.buf);
        buf
    }
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum EscapeCode {
    None = 0,
    Z0 = 1,
    Z1 = 2,
    Z2 = 3,
    Z3 = 4,
    Z4 = 5,
    Z5 = 6,
    Z6 = 7,
    Z7 = 8,
    BB = 9,
    TT = 10,
    NN = 11,
    ZB = 12,
    FF = 13,
    RR = 14,
    ZE = 15,
    ZF = 16,
    S0 = 17,
    S1 = 18,
    S2 = 19,
    S3 = 20,
    S4 = 21,
    S5 = 22,
    S6 = 23,
    S7 = 24,
    S8 = 25,
    S9 = 26,
    SA = 27,
    SB = 28,
    SC = 29,
    SD = 30,
    SE = 31,
    SF = 32,
    QU = 33,
    BS = 34,
}

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
static ESCAPE_TABLE: [EscapeCode; 256] = {
    use EscapeCode::*;
    let __ = EscapeCode::None;
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        Z0, Z1, Z2, Z3, Z4, Z5, Z6, Z7, BB, TT, NN, ZB, FF, RR, ZE, ZF, // 0
        S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, SA, SB, SC, SD, SE, SF, // 1
        __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
};

static ESCAPES: [&'static str; 35] = [
    "",        // Not used
    "\\u0000", // \x00
    "\\u0001", // \x01
    "\\u0002", // \x02
    "\\u0003", // \x03
    "\\u0004", // \x04
    "\\u0005", // \x05
    "\\u0006", // \x06
    "\\u0007", // \x07
    "\\b",     // \x08
    "\\t",     // \x09
    "\\n",     // \x0A
    "\\u000b", // \x0B
    "\\f",     // \x0C
    "\\r",     // \x0D
    "\\u000e", // \x0E
    "\\u000f", // \x0F
    "\\u0010", // \x10
    "\\u0011", // \x11
    "\\u0012", // \x12
    "\\u0013", // \x13
    "\\u0014", // \x14
    "\\u0015", // \x15
    "\\u0016", // \x16
    "\\u0017", // \x17
    "\\u0018", // \x18
    "\\u0019", // \x19
    "\\u001a", // \x1A
    "\\u001b", // \x1B
    "\\u001c", // \x1C
    "\\u001d", // \x1D
    "\\u001e", // \x1E
    "\\u001f", // \x1F
    "\\\"",    // \x22
    "\\\\",    // \x5C
];

fn escape_json_string<'a, S: AsRef<str>>(s: S, contents: &mut PreAllocatedString<'a>) {
    contents.push(Cow::Borrowed("\""));

    let s = s.as_ref();
    // Extend lifetime of `s`.
    // SAFETY: This is safe, because all strings are owned by `SourceMap` and live until after
    // `PreAllocatedString` is dropped.
    // TODO: Sort out lifetimes to make this work properly without `unsafe`.
    let s: &'a str = unsafe { std::mem::transmute(s) };

    let bytes = s.as_bytes();
    let mut start = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        let escape = ESCAPE_TABLE[byte as usize];
        if escape == EscapeCode::None {
            continue;
        }

        if start < i {
            contents.push(Cow::Borrowed(&s[start..i]));
        }

        contents.push(Cow::Borrowed(ESCAPES[escape as usize]));

        start = i + 1;
    }

    if start < bytes.len() {
        contents.push(Cow::Borrowed(&s[start..]));
    }

    contents.push(Cow::Borrowed("\""));
}

#[test]
fn test_escape_json_string() {
    const FIXTURES: &[(char, &str)] = &[
        ('n', "\"n\""),
        ('"', "\"\\\"\""),
        ('\\', "\"\\\\\""),
        ('/', "\"/\""),
        ('\x08', "\"\\b\""),
        ('\x0C', "\"\\f\""),
        ('\n', "\"\\n\""),
        ('\r', "\"\\r\""),
        ('\t', "\"\\t\""),
        ('\x0B', "\"\\u000b\""),
        ('虎', "\"虎\""),
        ('\u{3A3}', "\"\u{3A3}\""),
    ];

    for (c, expected) in FIXTURES {
        let mut input = String::new();
        input.push(*c);
        let mut contents = PreAllocatedString::new(0);
        escape_json_string(&input, &mut contents);
        let encoded = contents.consume();
        let _ = input; // Keep `input` alive until after `contents` consumed
        assert_eq!(encoded, *expected);
    }
}

#[test]
fn test_encode() {
    let input = r#"{
        "version": 3,
        "sources": ["coolstuff.js"],
        "sourceRoot": "x",
        "names": ["x","alert"],
        "mappings": "AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM"
    }"#;
    let sm = SourceMap::from_json_string(input).unwrap();
    let sm2 = SourceMap::from_json_string(&sm.to_json_string()).unwrap();

    for (tok1, tok2) in sm.get_tokens().zip(sm2.get_tokens()) {
        assert_eq!(tok1, tok2);
    }
}

#[test]
fn test_encode_escape_string() {
    // '\0' should be escaped.
    let mut sm = SourceMap::new(
        None,
        vec!["name_length_greater_than_16_\0".into()],
        None,
        vec!["\0".into()],
        Some(vec!["emoji-👀-\0".into()]),
        vec![],
        None,
    );
    sm.set_x_google_ignore_list(vec![0]);
    assert_eq!(
        sm.to_json_string(),
        r#"{"version":3,"names":["name_length_greater_than_16_\u0000"],"sources":["\u0000"],"sourcesContent":["emoji-👀-\u0000"],"x_google_ignoreList":[0],"mappings":""}"#
    );
}
