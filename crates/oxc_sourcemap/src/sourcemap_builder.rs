use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::{
    token::{Token, TokenChunk},
    SourceMap,
};

pub struct SourceMapBuilder {
    pub(crate) names_map: FxHashMap<Arc<str>, u32>,
    pub(crate) names: Vec<Arc<str>>,
    pub(crate) sources: Vec<Arc<str>>,
    pub(crate) sources_map: FxHashMap<Arc<str>, u32>,
    pub(crate) source_contents: Vec<Arc<str>>,
    pub(crate) tokens: Vec<Token>,
    pub(crate) token_chunks: Option<Vec<TokenChunk>>,
}

impl Default for SourceMapBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::cast_possible_truncation)]
impl SourceMapBuilder {
    pub fn new() -> Self {
        Self {
            names_map: FxHashMap::default(),
            names: Vec::new(),
            sources: Vec::new(),
            sources_map: FxHashMap::default(),
            source_contents: Vec::new(),
            tokens: Vec::new(),
            token_chunks: None,
        }
    }

    pub fn add_name(&mut self, name: &str) -> u32 {
        let count = self.names.len() as u32;
        let id = *self.names_map.entry(name.into()).or_insert(count);
        if id == count {
            self.names.push(name.into());
        }
        id
    }

    /// If `source` maybe duplicate, please use it.
    pub fn add_source_and_content(&mut self, source: &str, source_content: &str) -> u32 {
        let count = self.sources.len() as u32;
        let id = *self.sources_map.entry(source.into()).or_insert(count);
        if id == count {
            self.sources.push(source.into());
            self.source_contents.push(source_content.into());
        }
        id
    }

    /// If `source` hasn't duplicate，it will avoid extra hash calculation.
    pub fn set_source_and_content(&mut self, source: &str, source_content: &str) -> u32 {
        let count = self.sources.len() as u32;
        self.sources.push(source.into());
        self.source_contents.push(source_content.into());
        count
    }

    pub fn add_token(
        &mut self,
        dst_line: u32,
        dst_col: u32,
        src_line: u32,
        src_col: u32,
        src_id: Option<u32>,
        name_id: Option<u32>,
    ) {
        self.tokens.push(Token::new(dst_line, dst_col, src_line, src_col, src_id, name_id));
    }

    /// It used for concat sourcemap.
    pub fn add_sourcemap(&mut self, sourcemap: &SourceMap, line_offset: u32) {
        let source_offset = self.sources.len() as u32;
        let name_offset = self.names.len() as u32;

        if let Some(token_chunks) = self.token_chunks.as_mut() {
            if let Some(last_token) = self.tokens.last() {
                token_chunks.push(TokenChunk::new(
                    self.tokens.len() as u32,
                    self.tokens.len() as u32 + sourcemap.tokens.len() as u32,
                    last_token.get_dst_line(),
                    last_token.get_dst_col(),
                    last_token.get_src_line(),
                    last_token.get_src_col(),
                    name_offset - 1,
                    source_offset - 1,
                ));
            }
        } else {
            self.token_chunks =
                Some(vec![TokenChunk::new(0, sourcemap.tokens.len() as u32, 0, 0, 0, 0, 0, 0)]);
        }

        self.sources.reserve(sourcemap.sources.len());
        for (index, source) in sourcemap.get_sources().enumerate() {
            let source_content = sourcemap.get_source_content(index as u32).unwrap_or_default();
            self.set_source_and_content(source, source_content);
        }

        self.tokens.reserve(sourcemap.names.len());
        self.names.extend(sourcemap.get_names().map(Into::into));

        self.tokens.reserve(sourcemap.tokens.len());
        self.tokens.extend(sourcemap.get_tokens().map(|token| {
            Token::new(
                token.get_dst_line() + line_offset,
                token.get_dst_col(),
                token.get_src_line(),
                token.get_src_col(),
                token.get_source_id().map(|x| x + source_offset),
                token.get_name_id().map(|x| x + name_offset),
            )
        }));
    }

    pub fn into_sourcemap(self) -> SourceMap {
        SourceMap::new(
            None,
            self.names,
            self.sources,
            Some(self.source_contents),
            self.tokens,
            self.token_chunks,
        )
    }
}

#[test]
fn test_builder_into_sourcemap() {
    let mut builder = SourceMapBuilder::new();
    builder.set_source_and_content("baz.js", "");
    builder.add_name("x");

    let sm = builder.into_sourcemap();
    assert_eq!(sm.get_source(0), Some("baz.js"));
    assert_eq!(sm.get_name(0), Some("x"));

    let expected =
        r#"{"version":3,"names":["x"],"sources":["baz.js"],"sourcesContent":[""],"mappings":""}"#;
    assert_eq!(expected, sm.to_json_string());
}
