use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::{token::Token, SourceMap};

/// The `SourceMapBuilder` is a helper to generate sourcemap.
#[derive(Debug, Default)]
pub struct SourceMapBuilder {
    pub(crate) names_map: FxHashMap<Arc<str>, u32>,
    pub(crate) names: Vec<Arc<str>>,
    pub(crate) sources: Vec<Arc<str>>,
    pub(crate) sources_map: FxHashMap<Arc<str>, u32>,
    pub(crate) source_contents: Vec<Arc<str>>,
    pub(crate) tokens: Vec<Token>,
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

    pub fn into_sourcemap(self) -> SourceMap {
        SourceMap::new(
            None,
            self.names,
            self.sources,
            Some(self.source_contents),
            self.tokens,
            None,
        )
    }
}

#[test]
fn test_sourcemap_builder() {
    let mut builder = SourceMapBuilder::default();
    builder.set_source_and_content("baz.js", "");
    builder.add_name("x");

    let sm = builder.into_sourcemap();
    assert_eq!(sm.get_source(0), Some("baz.js"));
    assert_eq!(sm.get_name(0), Some("x"));

    let expected =
        r#"{"version":3,"names":["x"],"sources":["baz.js"],"sourcesContent":[""],"mappings":""}"#;
    assert_eq!(expected, sm.to_json_string());
}
