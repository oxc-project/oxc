/// An AST subtree the compiler does not model with typed nodes (type
/// annotations, class bodies, parser extras). The original text is re-parsed
/// from source during codegen; only the metadata the compiler reads is kept.
#[derive(Debug, Clone, Default)]
pub struct RawNode {
    /// Identifiers (incl. JSX identifiers) inside this subtree, pre-extracted
    /// with node-id (== source start offset), location and flags, so the core
    /// never walks JSON for the loc index, reference scans or renaming.
    pub idents: Vec<RawIdent>,
    /// Whether the subtree contains a hook call or JSX (for class-body members).
    pub contains_hook_or_jsx: bool,
    /// The type node's `type` tag, unwrapped past any
    /// `TypeAnnotation`/`TSTypeAnnotation` wrapper (e.g. `"TSTypeReference"`).
    pub node_type: Option<String>,
    /// Source span of the unwrapped type, for re-parsing it from source.
    pub type_start: Option<u32>,
    pub type_end: Option<u32>,
    /// Coarse classification of the unwrapped type, for HIR `Type` lowering.
    pub type_category: RawTypeCategory,
}

/// Coarse classification of a type annotation, mirroring the cases HIR type
/// lowering distinguishes (array / primitive / everything else).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RawTypeCategory {
    Array,
    Primitive,
    #[default]
    Other,
}

/// A reference to an identifier discovered inside a [`RawNode`] subtree.
#[derive(Debug, Clone)]
pub struct RawIdent {
    pub name: String,
    /// Babel `_nodeId`, equal to the identifier's source start offset.
    pub node_id: u32,
    pub start: u32,
    pub loc: Option<SourceLocation>,
    pub is_jsx: bool,
    /// True if the identifier sits inside a type-annotation subtree.
    pub in_type_annotation: bool,
    /// Set by the rename pass to the new name when this identifier is renamed.
    pub renamed_to: Option<String>,
}

impl RawNode {
    /// An empty placeholder carrying no metadata (e.g. for decorators / type
    /// parameters / class bodies re-emitted verbatim from source).
    pub fn empty() -> Self {
        RawNode::default()
    }

    /// Alias for [`RawNode::empty`].
    pub fn null() -> Self {
        RawNode::default()
    }

    /// A RawNode for a TS type, carrying its tag, source span, classification and
    /// referenced identifiers.
    pub fn type_node(
        node_type: Option<String>,
        type_start: Option<u32>,
        type_end: Option<u32>,
        type_category: RawTypeCategory,
        idents: Vec<RawIdent>,
    ) -> Self {
        RawNode {
            idents,
            contains_hook_or_jsx: false,
            node_type,
            type_start,
            type_end,
            type_category,
        }
    }

    /// A RawNode for an unmodeled subtree, carrying its identifiers and whether it
    /// contains a hook call or JSX.
    pub fn unknown(idents: Vec<RawIdent>, contains_hook_or_jsx: bool) -> Self {
        RawNode { idents, contains_hook_or_jsx, ..RawNode::default() }
    }
}

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
