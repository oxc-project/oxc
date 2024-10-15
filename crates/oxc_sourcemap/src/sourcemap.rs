use std::sync::Arc;

use crate::{
    decode::{decode, decode_from_string, JSONSourceMap},
    encode::{encode, encode_to_string},
    error::Result,
    token::{Token, TokenChunk},
    SourceViewToken,
};

#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    pub(crate) file: Option<Arc<str>>,
    pub(crate) names: Vec<Arc<str>>,
    pub(crate) source_root: Option<String>,
    pub(crate) sources: Vec<Arc<str>>,
    pub(crate) source_contents: Option<Vec<Arc<str>>>,
    pub(crate) tokens: Vec<Token>,
    pub(crate) token_chunks: Option<Vec<TokenChunk>>,
    /// Identifies third-party sources (such as framework code or bundler-generated code), allowing developers to avoid code that they don't want to see or step through, without having to configure this beforehand.
    /// The `x_google_ignoreList` field refers to the `sources` array, and lists the indices of all the known third-party sources in that source map.
    /// When parsing the source map, developer tools can use this to determine sections of the code that the browser loads and runs that could be automatically ignore-listed.
    pub(crate) x_google_ignore_list: Option<Vec<u32>>,
    pub(crate) debug_id: Option<String>,
}

#[allow(clippy::cast_possible_truncation)]
impl SourceMap {
    pub fn new(
        file: Option<Arc<str>>,
        names: Vec<Arc<str>>,
        source_root: Option<String>,
        sources: Vec<Arc<str>>,
        source_contents: Option<Vec<Arc<str>>>,
        tokens: Vec<Token>,
        token_chunks: Option<Vec<TokenChunk>>,
    ) -> Self {
        Self {
            file,
            names,
            source_root,
            sources,
            source_contents,
            tokens,
            token_chunks,
            x_google_ignore_list: None,
            debug_id: None,
        }
    }

    /// Convert the vlq sourcemap to to `SourceMap`.
    /// # Errors
    ///
    /// The `serde_json` deserialize Error.
    pub fn from_json(value: JSONSourceMap) -> Result<Self> {
        decode(value)
    }

    /// Convert the vlq sourcemap string to `SourceMap`.
    /// # Errors
    ///
    /// The `serde_json` deserialize Error.
    pub fn from_json_string(value: &str) -> Result<Self> {
        decode_from_string(value)
    }

    /// Convert `SourceMap` to vlq sourcemap.
    pub fn to_json(&self) -> JSONSourceMap {
        encode(self)
    }

    /// Convert `SourceMap` to vlq sourcemap string.
    pub fn to_json_string(&self) -> String {
        encode_to_string(self)
    }

    /// Convert `SourceMap` to vlq sourcemap data url.
    pub fn to_data_url(&self) -> String {
        let base_64_str = base64_simd::STANDARD.encode_to_string(self.to_json_string().as_bytes());
        format!("data:application/json;charset=utf-8;base64,{base_64_str}")
    }

    pub fn get_file(&self) -> Option<&str> {
        self.file.as_deref()
    }

    pub fn set_file(&mut self, file: &str) {
        self.file = Some(file.into());
    }

    pub fn get_source_root(&self) -> Option<&str> {
        self.source_root.as_deref()
    }

    /// Set `x_google_ignoreList`.
    pub fn set_x_google_ignore_list(&mut self, x_google_ignore_list: Vec<u32>) {
        self.x_google_ignore_list = Some(x_google_ignore_list);
    }

    pub fn set_debug_id(&mut self, debug_id: &str) {
        self.debug_id = Some(debug_id.into());
    }

    pub fn get_debug_id(&self) -> Option<&str> {
        self.debug_id.as_deref()
    }

    pub fn get_names(&self) -> impl Iterator<Item = &str> {
        self.names.iter().map(AsRef::as_ref)
    }

    /// Adjust `sources`.
    pub fn set_sources(&mut self, sources: Vec<&str>) {
        self.sources = sources.into_iter().map(Into::into).collect();
    }

    pub fn get_sources(&self) -> impl Iterator<Item = &str> {
        self.sources.iter().map(AsRef::as_ref)
    }

    /// Adjust `source_content`.
    pub fn set_source_contents(&mut self, source_contents: Vec<&str>) {
        self.source_contents = Some(source_contents.into_iter().map(Into::into).collect());
    }

    pub fn get_source_contents(&self) -> Option<impl Iterator<Item = &str>> {
        self.source_contents.as_ref().map(|v| v.iter().map(AsRef::as_ref))
    }

    pub fn get_token(&self, index: u32) -> Option<&Token> {
        self.tokens.get(index as usize)
    }

    pub fn get_source_view_token(&self, index: u32) -> Option<SourceViewToken<'_>> {
        self.tokens.get(index as usize).map(|token| SourceViewToken::new(token, self))
    }

    /// Get raw tokens.
    pub fn get_tokens(&self) -> impl Iterator<Item = &Token> {
        self.tokens.iter()
    }

    /// Get source view tokens. See [`SourceViewToken`] for more information.
    pub fn get_source_view_tokens(&self) -> impl Iterator<Item = SourceViewToken<'_>> {
        self.tokens.iter().map(|token| SourceViewToken::new(token, self))
    }

    pub fn get_name(&self, id: u32) -> Option<&str> {
        self.names.get(id as usize).map(AsRef::as_ref)
    }

    pub fn get_source(&self, id: u32) -> Option<&str> {
        self.sources.get(id as usize).map(AsRef::as_ref)
    }

    pub fn get_source_content(&self, id: u32) -> Option<&str> {
        self.source_contents.as_ref().and_then(|x| x.get(id as usize).map(AsRef::as_ref))
    }

    pub fn get_source_and_content(&self, id: u32) -> Option<(&str, &str)> {
        let source = self.get_source(id)?;
        let content = self.get_source_content(id)?;
        Some((source, content))
    }

    /// Generate a lookup table, it will be used at `lookup_token` or `lookup_source_view_token`.
    pub fn generate_lookup_table(&self) -> Vec<(u32, u32, u32)> {
        let mut table = self
            .tokens
            .iter()
            .enumerate()
            .map(|(idx, token)| (token.dst_line, token.dst_col, idx as u32))
            .collect::<Vec<_>>();
        table.sort_unstable();
        table
    }

    /// Lookup a token by line and column, it will used at remapping.
    pub fn lookup_token(
        &self,
        lookup_table: &[(u32, u32, u32)],
        line: u32,
        col: u32,
    ) -> Option<&Token> {
        let table = greatest_lower_bound(lookup_table, &(line, col), |table| (table.0, table.1))?;
        self.get_token(table.2)
    }

    /// Lookup a token by line and column, it will used at remapping. See `SourceViewToken`.
    pub fn lookup_source_view_token(
        &self,
        lookup_table: &[(u32, u32, u32)],
        line: u32,
        col: u32,
    ) -> Option<SourceViewToken<'_>> {
        self.lookup_token(lookup_table, line, col).map(|token| SourceViewToken::new(token, self))
    }
}

fn greatest_lower_bound<'a, T, K: Ord, F: Fn(&'a T) -> K>(
    slice: &'a [T],
    key: &K,
    map: F,
) -> Option<&'a T> {
    let mut idx = match slice.binary_search_by_key(key, &map) {
        Ok(index) => index,
        Err(index) => {
            // If there is no match, then we know for certain that the index is where we should
            // insert a new token, and that the token directly before is the greatest lower bound.
            return slice.get(index.checked_sub(1)?);
        }
    };

    // If we get an exact match, then we need to continue looking at previous tokens to see if
    // they also match. We use a linear search because the number of exact matches is generally
    // very small, and almost certainly smaller than the number of tokens before the index.
    for i in (0..idx).rev() {
        if map(&slice[i]) == *key {
            idx = i;
        } else {
            break;
        }
    }
    slice.get(idx)
}

#[test]
fn test_sourcemap_lookup_token() {
    let input = r#"{
        "version": 3,
        "sources": ["coolstuff.js"],
        "sourceRoot": "x",
        "names": ["x","alert"],
        "mappings": "AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM"
    }"#;
    let sm = SourceMap::from_json_string(input).unwrap();
    let lookup_table = sm.generate_lookup_table();
    assert_eq!(
        sm.lookup_source_view_token(&lookup_table, 0, 0).unwrap().to_tuple(),
        (Some("coolstuff.js"), 0, 0, None)
    );
    assert_eq!(
        sm.lookup_source_view_token(&lookup_table, 0, 3).unwrap().to_tuple(),
        (Some("coolstuff.js"), 0, 4, Some("x"))
    );
    assert_eq!(
        sm.lookup_source_view_token(&lookup_table, 0, 24).unwrap().to_tuple(),
        (Some("coolstuff.js"), 2, 8, None)
    );

    // Lines continue out to infinity
    assert_eq!(
        sm.lookup_source_view_token(&lookup_table, 0, 1000).unwrap().to_tuple(),
        (Some("coolstuff.js"), 2, 8, None)
    );

    // Token can return prior lines.
    assert_eq!(
        sm.lookup_source_view_token(&lookup_table, 1000, 0).unwrap().to_tuple(),
        (Some("coolstuff.js"), 2, 8, None)
    );
}

#[test]
fn test_sourcemap_source_view_token() {
    let sm = SourceMap::new(
        None,
        vec!["foo".into()],
        None,
        vec!["foo.js".into()],
        None,
        vec![Token::new(1, 1, 1, 1, Some(0), Some(0))],
        None,
    );
    let mut source_view_tokens = sm.get_source_view_tokens();
    assert_eq!(source_view_tokens.next().unwrap().to_tuple(), (Some("foo.js"), 1, 1, Some("foo")));
}

#[test]
fn test_mut_sourcemap() {
    let mut sm = SourceMap::default();
    sm.set_file("index.js");
    sm.set_sources(vec!["foo.js"]);
    sm.set_source_contents(vec!["foo"]);

    assert_eq!(sm.get_file(), Some("index.js"));
    assert_eq!(sm.get_source(0), Some("foo.js"));
    assert_eq!(sm.get_source_content(0), Some("foo"));
}
