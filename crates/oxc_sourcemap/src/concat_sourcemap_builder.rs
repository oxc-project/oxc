use crate::{token::TokenChunk, SourceMap, Token};
use std::sync::Arc;

/// The `ConcatSourceMapBuilder` is a helper to concat sourcemaps.
#[derive(Debug, Default)]
pub struct ConcatSourceMapBuilder {
    pub(crate) names: Vec<Arc<str>>,
    pub(crate) sources: Vec<Arc<str>>,
    pub(crate) source_contents: Vec<Arc<str>>,
    pub(crate) tokens: Vec<Token>,
    /// The `token_chunks` is used for encode tokens to vlq mappings at parallel.
    pub(crate) token_chunks: Vec<TokenChunk>,
}

#[allow(clippy::cast_possible_truncation)]
impl ConcatSourceMapBuilder {
    pub fn set_source_and_content(&mut self, source: &str, source_content: &str) -> u32 {
        let count = self.sources.len() as u32;
        self.sources.push(source.into());
        self.source_contents.push(source_content.into());
        count
    }

    pub fn add_sourcemap(&mut self, sourcemap: &SourceMap, line_offset: u32) {
        let source_offset = self.sources.len() as u32;
        let name_offset = self.names.len() as u32;

        // Add `token_chunks`, See `TokenChunk`.
        if let Some(last_token) = self.tokens.last() {
            self.token_chunks.push(TokenChunk::new(
                self.tokens.len() as u32,
                self.tokens.len() as u32 + sourcemap.tokens.len() as u32,
                last_token.get_dst_line(),
                last_token.get_dst_col(),
                last_token.get_src_line(),
                last_token.get_src_col(),
                name_offset - 1,
                source_offset - 1,
            ));
        } else {
            self.token_chunks.push(TokenChunk::new(
                0,
                sourcemap.tokens.len() as u32,
                0,
                0,
                0,
                0,
                0,
                0,
            ));
        }

        // Extend `sources` and `source_contents`.
        self.sources.reserve(sourcemap.sources.len());
        for (index, source) in sourcemap.get_sources().enumerate() {
            let source_content = sourcemap.get_source_content(index as u32).unwrap_or_default();
            self.set_source_and_content(source, source_content);
        }

        // Extend `names`.
        self.names.reserve(sourcemap.names.len());
        self.names.extend(sourcemap.get_names().map(Into::into));

        // Extend `tokens`.
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
            Some(self.token_chunks),
        )
    }
}
