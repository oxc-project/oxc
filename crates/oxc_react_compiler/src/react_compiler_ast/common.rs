pub use crate::react_compiler_hir::raw::{RawIdent, RawNode, RawTypeCategory};

#[derive(Debug, Clone)]
pub struct Position {
    pub line: u32,
    pub column: u32,
    pub index: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub start: Position,
    pub end: Position,
    pub filename: Option<String>,
    pub identifier_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Comment {
    CommentBlock(CommentData),
    CommentLine(CommentData),
}

#[derive(Debug, Clone)]
pub struct CommentData {
    pub value: String,
    pub start: Option<u32>,
    pub end: Option<u32>,
    pub loc: Option<SourceLocation>,
}

#[derive(Debug, Clone, Default)]
pub struct BaseNode {
    // NOTE: When creating AST nodes for code generation output, use
    // `BaseNode::typed("NodeTypeName")` instead of `BaseNode::default()`
    // to ensure the "type" field is emitted during serialization.
    /// The node type string (e.g. "BlockStatement").
    /// When deserialized through a `#[serde(tag = "type")]` enum, the enum
    /// consumes the "type" field so this defaults to None. When deserialized
    /// directly, this captures the "type" field for round-trip fidelity.
    pub node_type: Option<String>,
    pub start: Option<u32>,
    pub end: Option<u32>,
    pub loc: Option<SourceLocation>,
    pub range: Option<(u32, u32)>,
    pub extra: Option<RawNode>,
    pub leading_comments: Option<Vec<Comment>>,
    pub inner_comments: Option<Vec<Comment>>,
    pub trailing_comments: Option<Vec<Comment>>,
    pub node_id: Option<u32>,
}

impl BaseNode {
    /// Create a BaseNode with the given type name.
    /// Use this when creating AST nodes for code generation to ensure the
    /// `"type"` field is present in serialized output.
    pub fn typed(type_name: &str) -> Self {
        Self { node_type: Some(type_name.to_string()), ..Default::default() }
    }
}
