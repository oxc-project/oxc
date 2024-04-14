use crate::SourceMap;

/// The `Token` is used to generate vlq `mappings`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub(crate) dst_line: u32,
    pub(crate) dst_col: u32,
    pub(crate) src_line: u32,
    pub(crate) src_col: u32,
    pub(crate) source_id: Option<u32>,
    pub(crate) name_id: Option<u32>,
}

impl Token {
    pub fn new(
        dst_line: u32,
        dst_col: u32,
        src_line: u32,
        src_col: u32,
        source_id: Option<u32>,
        name_id: Option<u32>,
    ) -> Self {
        Self { dst_line, dst_col, src_line, src_col, source_id, name_id }
    }

    pub fn get_dst_line(&self) -> u32 {
        self.dst_line
    }

    pub fn get_dst_col(&self) -> u32 {
        self.dst_col
    }

    pub fn get_src_line(&self) -> u32 {
        self.src_line
    }

    pub fn get_src_col(&self) -> u32 {
        self.src_col
    }

    pub fn get_name_id(&self) -> Option<u32> {
        self.name_id
    }

    pub fn get_source_id(&self) -> Option<u32> {
        self.source_id
    }
}

/// The `TokenChunk` used by encode tokens to vlq mappings at parallel.
/// It is a slice of `SourceMap::tokens`, it is a unit of parallel.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TokenChunk {
    pub start: u32,
    pub end: u32,
    pub prev_dst_line: u32,
    pub prev_dst_col: u32,
    pub prev_src_line: u32,
    pub prev_src_col: u32,
    pub prev_name_id: u32,
    pub prev_source_id: u32,
}

impl TokenChunk {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        start: u32,
        end: u32,
        prev_dst_line: u32,
        prev_dst_col: u32,
        prev_src_line: u32,
        prev_src_col: u32,
        prev_name_id: u32,
        prev_source_id: u32,
    ) -> Self {
        Self {
            start,
            end,
            prev_dst_line,
            prev_dst_col,
            prev_src_line,
            prev_src_col,
            prev_name_id,
            prev_source_id,
        }
    }
}

/// The `SourceViewToken` provider extra `source` and `source_content` value.
#[derive(Debug, Clone, Copy)]
pub struct SourceViewToken<'a> {
    pub(crate) token: &'a Token,
    pub(crate) sourcemap: &'a SourceMap,
}

impl<'a> SourceViewToken<'a> {
    pub fn new(token: &'a Token, sourcemap: &'a SourceMap) -> Self {
        Self { token, sourcemap }
    }

    pub fn get_dst_line(&self) -> u32 {
        self.token.dst_line
    }

    pub fn get_dst_col(&self) -> u32 {
        self.token.dst_col
    }

    pub fn get_src_line(&self) -> u32 {
        self.token.src_line
    }

    pub fn get_src_col(&self) -> u32 {
        self.token.src_col
    }

    pub fn get_name_id(&self) -> Option<u32> {
        self.token.name_id
    }

    pub fn get_source_id(&self) -> Option<u32> {
        self.token.source_id
    }

    pub fn get_name(&self) -> Option<&str> {
        self.token.name_id.and_then(|id| self.sourcemap.get_name(id))
    }

    pub fn get_source(&self) -> Option<&str> {
        self.token.source_id.and_then(|id| self.sourcemap.get_source(id))
    }

    pub fn get_source_content(&self) -> Option<&str> {
        self.token.source_id.and_then(|id| self.sourcemap.get_source_content(id))
    }

    pub fn get_source_and_content(&self) -> Option<(&str, &str)> {
        self.token.source_id.and_then(|id| self.sourcemap.get_source_and_content(id))
    }

    pub fn to_tuple(&self) -> (Option<&str>, u32, u32, Option<&str>) {
        (self.get_source(), self.get_src_line(), self.get_src_col(), self.get_name())
    }
}
