/// Port from https://github.com/getsentry/rust-sourcemap/blob/master/src/decoder.rs
/// It is a helper for decode vlq soucemap string to `SourceMap`.
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::{SourceMap, Token};

/// See <https://github.com/tc39/source-map/blob/main/source-map-rev3.md>.
#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JSONSourceMap {
    /// An optional name of the generated code that this source map is associated with.
    pub file: Option<String>,
    /// A string with the encoded mapping data.
    pub mappings: String,
    /// An optional source root, useful for relocating source files on a server or removing repeated values in the “sources” entry.
    /// This value is prepended to the individual entries in the “source” field.
    pub source_root: Option<String>,
    /// A list of original sources used by the “mappings” entry.
    pub sources: Vec<String>,
    /// An optional list of source content, useful when the “source” can’t be hosted.
    /// The contents are listed in the same order as the sources in line 5. “null” may be used if some original sources should be retrieved by name.
    pub sources_content: Option<Vec<Option<String>>>,
    /// A list of symbol names used by the “mappings” entry.
    pub names: Vec<String>,
    /// An optional field containing the debugId for this sourcemap.
    pub debug_id: Option<String>,
}

pub fn decode(json: JSONSourceMap) -> Result<SourceMap> {
    let tokens = decode_mapping(&json.mappings, json.names.len(), json.sources.len())?;
    Ok(SourceMap {
        file: json.file.map(Arc::from),
        names: json.names.into_iter().map(Arc::from).collect(),
        source_root: json.source_root,
        sources: json.sources.into_iter().map(Arc::from).collect(),
        source_contents: json.sources_content.map(|content| {
            content.into_iter().map(|c| c.map(Arc::from).unwrap_or_default()).collect()
        }),
        tokens,
        token_chunks: None,
        x_google_ignore_list: None,
        debug_id: json.debug_id,
    })
}

pub fn decode_from_string(value: &str) -> Result<SourceMap> {
    decode(serde_json::from_str(value)?)
}

#[allow(clippy::cast_possible_truncation)]
fn decode_mapping(mapping: &str, names_len: usize, sources_len: usize) -> Result<Vec<Token>> {
    let mut tokens = vec![];

    let mut dst_col;
    let mut src_id = 0;
    let mut src_line = 0;
    let mut src_col = 0;
    let mut name_id = 0;
    let mut nums = Vec::with_capacity(6);

    for (dst_line, line) in mapping.split(';').enumerate() {
        if line.is_empty() {
            continue;
        }

        dst_col = 0;

        for segment in line.split(',') {
            if segment.is_empty() {
                continue;
            }

            nums.clear();
            parse_vlq_segment_into(segment, &mut nums)?;
            dst_col = (i64::from(dst_col) + nums[0]) as u32;

            let mut src = !0;
            let mut name = !0;

            if nums.len() > 1 {
                if nums.len() != 4 && nums.len() != 5 {
                    return Err(Error::BadSegmentSize(nums.len() as u32));
                }
                src_id = (i64::from(src_id) + nums[1]) as u32;
                if src_id >= sources_len as u32 {
                    return Err(Error::BadSourceReference(src_id));
                }

                src = src_id;
                src_line = (i64::from(src_line) + nums[2]) as u32;
                src_col = (i64::from(src_col) + nums[3]) as u32;

                if nums.len() > 4 {
                    name_id = (i64::from(name_id) + nums[4]) as u32;
                    if name_id >= names_len as u32 {
                        return Err(Error::BadNameReference(name_id));
                    }
                    name = name_id;
                }
            }

            tokens.push(Token::new(
                dst_line as u32,
                dst_col,
                src_line,
                src_col,
                if src == !0 { None } else { Some(src) },
                if name == !0 { None } else { Some(name) },
            ));
        }
    }

    Ok(tokens)
}

#[rustfmt::skip]
const B64: [i8; 256] = [ -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 62, -1, -1, -1, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1, -1, -1, -1, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, -1, -1, -1, -1, -1 - 1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, ];

fn parse_vlq_segment_into(segment: &str, rv: &mut Vec<i64>) -> Result<()> {
    let mut cur = 0;
    let mut shift = 0;

    for c in segment.bytes() {
        let enc = i64::from(B64[c as usize]);
        let val = enc & 0b11111;
        let cont = enc >> 5;
        cur += val.checked_shl(shift).ok_or(Error::VlqOverflow)?;
        shift += 5;

        if cont == 0 {
            let sign = cur & 1;
            cur >>= 1;
            if sign != 0 {
                cur = -cur;
            }
            rv.push(cur);
            cur = 0;
            shift = 0;
        }
    }

    if cur != 0 || shift != 0 {
        Err(Error::VlqLeftover)
    } else if rv.is_empty() {
        Err(Error::VlqNoValues)
    } else {
        Ok(())
    }
}

#[test]
fn test_decode_sourcemap() {
    let input = r#"{
        "version": 3,
        "sources": ["coolstuff.js"],
        "sourceRoot": "x",
        "names": ["x","alert"],
        "mappings": "AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM"
    }"#;
    let sm = SourceMap::from_json_string(input).unwrap();
    assert_eq!(sm.get_source_root(), Some("x"));
    let mut iter = sm.get_source_view_tokens().filter(|token| token.get_name_id().is_some());
    assert_eq!(iter.next().unwrap().to_tuple(), (Some("coolstuff.js"), 0, 4, Some("x")));
    assert_eq!(iter.next().unwrap().to_tuple(), (Some("coolstuff.js"), 1, 4, Some("x")));
    assert_eq!(iter.next().unwrap().to_tuple(), (Some("coolstuff.js"), 2, 2, Some("alert")));
    assert!(iter.next().is_none());
}

#[test]
fn test_decode_sourcemap_optional_filed() {
    let input = r#"{
        "names": [],
        "sources": [],
        "sourcesContent": [null],
        "mappings": ""
    }"#;
    SourceMap::from_json_string(input).expect("should success");
}
