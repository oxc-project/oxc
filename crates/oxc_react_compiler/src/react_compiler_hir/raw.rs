//! Raw, unmodeled AST subtrees carried through the HIR.
//!
//! [`RawNode`] holds the metadata the compiler reads off type annotations, class
//! bodies and other parser extras the IR does not model with typed nodes; the
//! original text is re-parsed from source during codegen.

use crate::react_compiler_diagnostics::SourceLocation;

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
