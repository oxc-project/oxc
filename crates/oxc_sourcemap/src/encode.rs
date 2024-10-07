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
        mappings: serialize_sourcemap_mappings(sourcemap),
        source_root: sourcemap.get_source_root().map(ToString::to_string),
        sources: sourcemap.sources.iter().map(ToString::to_string).collect(),
        sources_content: sourcemap
            .source_contents
            .as_ref()
            .map(|x| x.iter().map(ToString::to_string).map(Some).collect()),
        names: sourcemap.names.iter().map(ToString::to_string).collect(),
        debug_id: sourcemap.get_debug_id().map(ToString::to_string),
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
    contents.push_list(sourcemap.names.iter().map(escape_json_string));

    contents.push("],\"sources\":[".into());
    contents.push_list(sourcemap.sources.iter().map(escape_json_string));

    // Quote `source_content` in parallel
    if let Some(source_contents) = &sourcemap.source_contents {
        contents.push("],\"sourcesContent\":[".into());
        cfg_if::cfg_if! {
            if #[cfg(feature = "concurrent")] {
                let quoted_source_contents: Vec<_> = source_contents
                    .par_iter()
                    .map(escape_json_string)
                    .collect();
                contents.push_list(quoted_source_contents.into_iter());
            } else {
                contents.push_list(source_contents.iter().map(escape_json_string));
            }
        };
    }

    if let Some(x_google_ignore_list) = &sourcemap.x_google_ignore_list {
        contents.push("],\"x_google_ignoreList\":[".into());
        contents.push_list(x_google_ignore_list.iter().map(ToString::to_string));
    }

    contents.push("],\"mappings\":\"".into());
    contents.push(serialize_sourcemap_mappings(sourcemap).into());

    if let Some(debug_id) = sourcemap.get_debug_id() {
        contents.push("\",\"debugId\":\"".into());
        contents.push(debug_id.into());
    }

    contents.push("\"}".into());

    // Check we calculated number of segments required correctly
    debug_assert!(contents.num_segments() <= max_segments);

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

// Max length of a single VLQ encoding
const MAX_VLQ_BYTES: usize = 7;

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

    let mut prev_token = if start == 0 { None } else { Some(&tokens[start as usize - 1]) };

    for token in &tokens[start as usize..end as usize] {
        // Max length of a single VLQ encoding is 7 bytes. Max number of calls to `encode_vlq_diff` is 5.
        // Also need 1 byte for each line number difference, or 1 byte if no line num difference.
        // Reserve this amount of capacity in `rv` early, so can skip bounds checks in code below.
        // As well as skipping the bounds checks, this also removes a function call to
        // `alloc::raw_vec::RawVec::grow_one` for every byte that's pushed.
        // https://godbolt.org/z/44G8jjss3
        const MAX_TOTAL_VLQ_BYTES: usize = 5 * MAX_VLQ_BYTES;

        let num_line_breaks = token.get_dst_line() - prev_dst_line;
        if num_line_breaks != 0 {
            rv.reserve(MAX_TOTAL_VLQ_BYTES + num_line_breaks as usize);
            // SAFETY: We have reserved sufficient capacity for `num_line_breaks` bytes
            unsafe { push_bytes_unchecked(&mut rv, b';', num_line_breaks) };
            prev_dst_col = 0;
            prev_dst_line += num_line_breaks;
        } else if let Some(prev_token) = prev_token {
            if prev_token == token {
                continue;
            }
            rv.reserve(MAX_TOTAL_VLQ_BYTES + 1);
            // SAFETY: We have reserved sufficient capacity for 1 byte
            unsafe { push_byte_unchecked(&mut rv, b',') };
        }

        // SAFETY: We have reserved enough capacity above to satisfy safety contract
        // of `encode_vlq_diff` for all calls below
        unsafe {
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

        prev_token = Some(token);
    }

    rv
}

/// Encode diff as VLQ and push encoding into `out`.
/// Will push between 1 byte (num = 0) and 7 bytes (num = -u32::MAX).
///
/// # SAFETY
/// Caller must ensure at least 7 bytes spare capacity in `out`,
/// as this function does not perform any bounds checks.
#[inline]
unsafe fn encode_vlq_diff(out: &mut String, a: u32, b: u32) {
    encode_vlq(out, i64::from(a) - i64::from(b));
}

// Align chars lookup table on 64 so occupies a single cache line
#[repr(align(64))]
struct Aligned64([u8; 64]);

static B64_CHARS: Aligned64 = Aligned64([
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'+', b'/',
]);

/// Encode number as VLQ and push encoding into `out`.
/// Will push between 1 byte (num = 0) and 7 bytes (num = -u32::MAX).
///
/// # SAFETY
/// Caller must ensure at least 7 bytes spare capacity in `out`,
/// as this function does not perform any bounds checks.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unnecessary_safety_comment
)]
unsafe fn encode_vlq(out: &mut String, num: i64) {
    let mut num = if num < 0 { ((-num) << 1) + 1 } else { num << 1 };

    // Breaking out of loop early when have reached last char (rather than conditionally adding
    // 32 for last char within the loop) removes 3 instructions from the loop.
    // https://godbolt.org/z/Es4Pavh9j
    // This translates to a 16% speed-up for VLQ encoding.
    let mut digit;
    loop {
        digit = num & 0b11111;
        num >>= 5;
        if num == 0 {
            break;
        }

        let b = B64_CHARS.0[digit as usize + 32];
        // SAFETY:
        // * This loop can execute a maximum of 7 times, and on last turn will exit before getting here.
        //   Caller promises there are at least 7 bytes spare capacity in `out` at start. We only
        //   push 1 byte on each turn, so guaranteed there is at least 1 byte capacity in `out` here.
        // * All values in `B64_CHARS` lookup table are ASCII bytes.
        push_byte_unchecked(out, b);
    }

    let b = B64_CHARS.0[digit as usize];
    // SAFETY:
    // * The loop above pushes max 6 bytes. Caller promises there are at least 7 bytes spare capacity
    //   in `out` at start. So guaranteed there is at least 1 byte capacity in `out` here.
    // * All values in `B64_CHARS` lookup table are ASCII bytes.
    push_byte_unchecked(out, b);
}

/// Push a byte to `out` without bounds checking.
///
/// # SAFETY
/// * `out` must have at least 1 byte spare capacity.
/// * `b` must be an ASCII byte (i.e. not `>= 128`).
//
// `#[inline(always)]` to ensure that `len` is stored in a register during `encode_vlq`'s loop.
#[allow(clippy::inline_always)]
#[inline(always)]
unsafe fn push_byte_unchecked(out: &mut String, b: u8) {
    debug_assert!(out.len() < out.capacity());
    debug_assert!(b.is_ascii());

    let out = out.as_mut_vec();
    let len = out.len();
    let ptr = out.as_mut_ptr().add(len);
    ptr.write(b);
    out.set_len(len + 1);
}

/// Push a byte to `out` a number of times without bounds checking.
///
/// # SAFETY
/// * `out` must have at least `repeats` bytes spare capacity.
/// * `b` must be an ASCII byte (i.e. not `>= 128`).
#[inline]
unsafe fn push_bytes_unchecked(out: &mut String, b: u8, repeats: u32) {
    debug_assert!(out.capacity() - out.len() >= repeats as usize);
    debug_assert!(b.is_ascii());

    let out = out.as_mut_vec();
    let len = out.len();
    let mut ptr = out.as_mut_ptr().add(len);
    for _ in 0..repeats {
        ptr.write(b);
        ptr = ptr.add(1);
    }
    out.set_len(len + repeats as usize);
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
    fn push_list<I>(&mut self, mut iter: I)
    where
        I: Iterator<Item = String>,
    {
        let Some(first) = iter.next() else {
            return;
        };
        self.push(Cow::Owned(first));

        for other in iter {
            self.push(Cow::Borrowed(","));
            self.push(Cow::Owned(other));
        }
    }

    #[inline]
    fn consume(self) -> String {
        let mut buf = String::with_capacity(self.len);
        buf.extend(self.buf);
        buf
    }

    fn num_segments(&self) -> usize {
        self.buf.len()
    }
}

fn escape_json_string<S: AsRef<str>>(s: S) -> String {
    let s = s.as_ref();
    let mut escaped_buf = Vec::with_capacity(s.len() * 2 + 2);
    // This call is infallible as only error it can return is if the writer errors.
    // Writing to a `Vec<u8>` is infallible, so that's not possible here.
    serde::Serialize::serialize(s, &mut serde_json::Serializer::new(&mut escaped_buf)).unwrap();
    // Safety: `escaped_buf` is valid utf8.
    unsafe { String::from_utf8_unchecked(escaped_buf) }
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
        assert_eq!(escape_json_string(input), *expected);
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
    sm.set_debug_id("56431d54-c0a6-451d-8ea2-ba5de5d8ca2e");
    assert_eq!(
        sm.to_json_string(),
        r#"{"version":3,"names":["name_length_greater_than_16_\u0000"],"sources":["\u0000"],"sourcesContent":["emoji-👀-\u0000"],"x_google_ignoreList":[0],"mappings":"","debugId":"56431d54-c0a6-451d-8ea2-ba5de5d8ca2e"}"#
    );
}

#[test]
fn test_vlq_encode_diff() {
    // Most import tests here are that with maximum values, `encode_vlq_diff` pushes maximum of 7 bytes.
    // This invariant is essential to safety of `encode_vlq_diff`.
    #[rustfmt::skip]
    const FIXTURES: &[(u32, u32, &str)] = &[
        (0,           0, "A"),
        (1,           0, "C"),
        (2,           0, "E"),
        (15,          0, "e"),
        (16,          0, "gB"),
        (511,         0, "+f"),
        (512,         0, "ggB"),
        (16_383,      0, "+/f"),
        (16_384,      0, "gggB"),
        (524_287,     0, "+//f"),
        (524_288,     0, "ggggB"),
        (16_777_215,  0, "+///f"),
        (16_777_216,  0, "gggggB"),
        (536_870_911, 0, "+////f"),
        (536_870_912, 0, "ggggggB"),
        (u32::MAX,    0, "+/////H"), // 7 bytes

        (0, 1,           "D"),
        (0, 2,           "F"),
        (0, 15,          "f"),
        (0, 16,          "hB"),
        (0, 511,         "/f"),
        (0, 512,         "hgB"),
        (0, 16_383,      "//f"),
        (0, 16_384,      "hggB"),
        (0, 524_287,     "///f"),
        (0, 524_288,     "hgggB"),
        (0, 16_777_215,  "////f"),
        (0, 16_777_216,  "hggggB"),
        (0, 536_870_911, "/////f"),
        (0, 536_870_912, "hgggggB"),
        (0, u32::MAX,    "//////H"), // 7 bytes
    ];

    for (a, b, res) in FIXTURES.iter().copied() {
        let mut out = String::with_capacity(MAX_VLQ_BYTES);
        // SAFETY: `out` has 7 bytes spare capacity
        unsafe { encode_vlq_diff(&mut out, a, b) };
        assert_eq!(&out, res);
    }
}
