use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::{
    token::{Token, TokenChunk},
    SourceMap,
};

/// The `SourceMapBuilder` is a helper to generate sourcemap.
#[derive(Debug, Default)]
pub struct SourceMapBuilder {
    pub(crate) file: Option<Arc<str>>,
    pub(crate) names_map: FxHashMap<Arc<str>, u32>,
    pub(crate) names: Vec<Arc<str>>,
    pub(crate) sources: Vec<Arc<str>>,
    pub(crate) sources_map: FxHashMap<Arc<str>, u32>,
    pub(crate) source_contents: Vec<Arc<str>>,
    pub(crate) tokens: Vec<Token>,
    pub(crate) token_chunks: Option<Vec<TokenChunk>>,
}

#[allow(clippy::cast_possible_truncation)]
impl SourceMapBuilder {
    /// Add item to `SourceMap::name`.
    pub fn add_name(&mut self, name: &str) -> u32 {
        let count = self.names.len() as u32;
        let id = *self.names_map.entry(name.into()).or_insert(count);
        if id == count {
            self.names.push(name.into());
        }
        id
    }

    /// Add item to `SourceMap::sources` and `SourceMap::source_contents`.
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

    /// Add item to `SourceMap::sources` and `SourceMap::source_contents`.
    /// If `source` hasn't duplicate，it will avoid extra hash calculation.
    pub fn set_source_and_content(&mut self, source: &str, source_content: &str) -> u32 {
        let count = self.sources.len() as u32;
        self.sources.push(source.into());
        self.source_contents.push(source_content.into());
        count
    }

    /// Add item to `SourceMap::tokens`.
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

    pub fn set_file(&mut self, file: &str) {
        self.file = Some(file.into());
    }

    /// Set the `SourceMap::token_chunks` to make the sourcemap to vlq mapping at parallel.
    pub fn set_token_chunks(&mut self, token_chunks: Vec<TokenChunk>) {
        self.token_chunks = Some(token_chunks);
    }

    pub fn into_sourcemap(self) -> SourceMap {
        SourceMap::new(
            self.file,
            self.names,
            None,
            self.sources,
            Some(self.source_contents),
            self.tokens,
            self.token_chunks,
        )
    }
}

#[test]
fn test_sourcemap_builder() {
    let mut builder = SourceMapBuilder::default();
    builder.set_source_and_content("baz.js", "");
    builder.add_name("x");
    builder.set_file("file");

    let sm = builder.into_sourcemap();
    assert_eq!(sm.get_source(0), Some("baz.js"));
    assert_eq!(sm.get_name(0), Some("x"));
    assert_eq!(sm.get_file(), Some("file"));

    let expected = r#"{"version":3,"file":"file","names":["x"],"sources":["baz.js"],"sourcesContent":[""],"mappings":""}"#;
    assert_eq!(expected, sm.to_json_string().unwrap());
}
