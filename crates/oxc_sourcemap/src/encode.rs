/// Port from https://github.com/getsentry/rust-sourcemap/blob/master/src/encoder.rs
/// It is a helper for encode `SourceMap` to vlq sourcemap string, but here some different.
/// - Quote `source_content` at parallel.
/// - If you using `ConcatSourceMapBuilder`, serialize `tokens` to vlq `mappings` at parallel.
use crate::{token::TokenChunk, SourceMap, Token};
use rayon::prelude::*;

pub fn encode(sourcemap: &SourceMap) -> String {
    let mut buf = String::new();
    buf.push_str("{\"version\":3,");
    if let Some(file) = sourcemap.get_file() {
        buf.push_str("\"file\":\"");
        buf.push_str(file);
        buf.push_str("\",");
    }
    buf.push_str("\"names\":[");
    buf.push_str(&sourcemap.names.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>().join(","));
    buf.push_str("],\"sources\":[");
    buf.push_str(&sourcemap.sources.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>().join(","));
    // Quote `source_content` at parallel.
    if let Some(source_contents) = &sourcemap.source_contents {
        buf.push_str("],\"sourcesContent\":[");
        buf.push_str(
            &source_contents.par_iter().map(|x| format!("{x:?}")).collect::<Vec<_>>().join(","),
        );
    }
    buf.push_str("],\"mappings\":\"");
    buf.push_str(&serialize_sourcemap_mappings(sourcemap));
    buf.push_str("\"}");
    buf
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
            token_chunks
                .par_iter()
                .map(|token_chunk| serialize_mappings(&sm.tokens, token_chunk))
                .collect::<String>()
        },
    )
}

fn serialize_mappings(tokens: &[Token], token_chunk: &TokenChunk) -> String {
    let mut rv = String::new();

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

    for (idx, token) in tokens[start as usize..end as usize].iter().enumerate() {
        if token.get_dst_line() != prev_dst_line {
            prev_dst_col = 0;
            while token.get_dst_line() != prev_dst_line {
                rv.push(';');
                prev_dst_line += 1;
            }
        } else if idx > 0 {
            if Some(token) == tokens.get(idx - 1) {
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

#[test]
fn test_encode() {
    let input = r#"{
        "version": 3,
        "sources": ["coolstuff.js"],
        "names": ["x","alert"],
        "mappings": "AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM"
    }"#;
    let sm = SourceMap::from_json_string(input).unwrap();
    let sm2 = SourceMap::from_json_string(&sm.to_json_string()).unwrap();

    for (tok1, tok2) in sm.get_tokens().zip(sm2.get_tokens()) {
        assert_eq!(tok1, tok2);
    }
}
