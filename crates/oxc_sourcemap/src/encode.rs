use std::borrow::Cow;

#[cfg(feature = "concurrent")]
use rayon::prelude::*;

use crate::error::{Error, Result};
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

// Here using `serde_json::to_string` to serialization `names/source_contents/sources`.
// It will escape the string to avoid invalid JSON string.
pub fn encode_to_string(sourcemap: &SourceMap) -> Result<String> {
    let mut contents = PreAllocatedString::new(
        10 + sourcemap.names.len() * 2
            + sourcemap.sources.len() * 2
            + if let Some(x) = &sourcemap.x_google_ignore_list { x.len() * 2 } else { 0 },
    );
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
    for n in &sourcemap.names {
        contents.push(serde_json::to_string(n.as_ref())?.into());
        contents.push(",".into());
    }
    if !sourcemap.names.is_empty() {
        // Remove the last `,`.
        contents.pop();
    }
    contents.push(Cow::Borrowed("],\"sources\":["));
    for s in &sourcemap.sources {
        contents.push(serde_json::to_string(s.as_ref())?.into());
        contents.push(",".into());
    }
    if !sourcemap.sources.is_empty() {
        // Remove the last `,`.
        contents.pop();
    }
    // Quote `source_content` at parallel.
    if let Some(source_contents) = &sourcemap.source_contents {
        contents.push("],\"sourcesContent\":[".into());
        cfg_if::cfg_if! {
            if #[cfg(feature = "concurrent")] {
                let quote_source_contents = source_contents
                    .par_iter()
                    .map(|x| serde_json::to_string(x.as_ref()))
                    .collect::<std::result::Result<Vec<_>, serde_json::Error>>()
                    .map_err(Error::from)?;
            } else {
                let quote_source_contents = source_contents
                    .iter()
                    .map(|x| serde_json::to_string(x.as_ref()))
                    .collect::<std::result::Result<Vec<_>, serde_json::Error>>()
                    .map_err(Error::from)?;
            }
        };

        contents.push(quote_source_contents.join(",").into());
    }
    if let Some(x_google_ignore_list) = &sourcemap.x_google_ignore_list {
        contents.push("],\"x_google_ignoreList\":[".into());
        for ignore in x_google_ignore_list {
            contents.push(ignore.to_string().into());
            contents.push(",".into());
        }
        if !x_google_ignore_list.is_empty() {
            // Remove the last `,`.
            contents.pop();
        }
    }
    contents.push("],\"mappings\":\"".into());
    contents.push(serialize_sourcemap_mappings(sourcemap).into());
    contents.push("\"}".into());
    Ok(contents.consume())
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
    fn pop(&mut self) {
        if let Some(s) = self.buf.pop() {
            self.len -= s.len();
        }
    }

    #[inline]
    fn consume(self) -> String {
        let mut buf = String::with_capacity(self.len);
        buf.extend(self.buf);
        buf
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
    let sm2 = SourceMap::from_json_string(&sm.to_json_string().unwrap()).unwrap();

    for (tok1, tok2) in sm.get_tokens().zip(sm2.get_tokens()) {
        assert_eq!(tok1, tok2);
    }
}

#[test]
fn test_encode_escape_string() {
    // '\0' should be escaped.
    let mut sm = SourceMap::new(
        None,
        vec!["\0".into()],
        None,
        vec!["\0".into()],
        Some(vec!["\0".into()]),
        vec![],
        None,
    );
    sm.set_x_google_ignore_list(vec![0]);
    assert_eq!(
        sm.to_json_string().unwrap(),
        r#"{"version":3,"names":["\u0000"],"sources":["\u0000"],"sourcesContent":["\u0000"],"x_google_ignoreList":[0],"mappings":""}"#
    );
}
