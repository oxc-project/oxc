//! Builds an index mapping identifier references and declarations to source
//! locations, keyed by semantic `ReferenceId` / `SymbolId`.
//!
//! Walks the function's oxc AST to collect resolved identifier references and
//! binding declarations (including identifiers inside TS type annotations).
//!
//! This is a translation of the original immutable `IdentifierLocVisitor`, which
//! was driven by the in-tree `AstWalker`/`Visitor`
//! (`crate::react_compiler_ast::visitor`). That walker deliberately visited only
//! a NARROW set of identifier positions, and TS type identifiers came from
//! `collect_type_idents`, which collected only `IdentifierReference` /
//! `IdentifierName` (never `BindingIdentifier`). The oxc walk driving this pass
//! lives in [`super::pre_pass`], where it shares a single traversal with
//! `FindContextIdentifiers`; its overrides restrict the walk to match those
//! positions instead of relying on oxc's default full-AST traversal. The
//! traversal records:
//!
//! * every identifier reference and (declaration-map) binding identifier
//! * function / class declaration & expression names, into the declaration map
//! * JSX element-name identifier references, carrying the enclosing
//!   `JSXOpeningElement`'s span as `opening_element_span`
//! * identifiers inside TS type subtrees → `in_type_annotation = true`
//!
//! Positions deliberately NOT recorded, matching the original walker:
//!
//! * non-computed member property names (`a.b` → `b`)
//! * non-computed object / class member keys (`{ a: 1 }` → `a`)
//! * JSX attribute names and JSX closing-element names
//! * label identifiers (`LabeledStatement` / `break` / `continue` targets)
//! * class `super_class` (`extends Foo`) and class member bodies
//! * TS declaration statements (type alias / interface / enum / module)
//! * `BindingIdentifier`s inside TS type subtrees (e.g. type-parameter names)

use rustc_hash::FxHashMap;

use oxc_ast::ast::*;

use crate::scope::{ReferenceId, SymbolId};

/// Source location data for a resolved identifier reference.
pub struct IdentifierLocEntry {
    pub span: Span,
    /// For JSX element-name identifiers, the enclosing `JSXOpeningElement`'s
    /// span (which spans the full tag).
    pub opening_element_span: Option<Span>,
    /// True if this identifier sits inside a type annotation subtree
    /// (TypeAnnotation/TSTypeAnnotation/TypeAlias/TSTypeAliasDeclaration).
    /// `gather_captured_context` skips these to match the TS
    /// gatherCapturedContext traverse, which skips those subtrees; the
    /// hoisting analysis and FindContextIdentifiers do NOT skip them in TS.
    pub in_type_annotation: bool,
}

impl IdentifierLocEntry {
    /// True for JSX element-name identifiers. The TS hoisting analysis does
    /// not traverse JSX elements, so hoisting skips these references.
    pub fn is_jsx(&self) -> bool {
        self.opening_element_span.is_some()
    }
}

/// Identifier locations for a function's AST, keyed by semantic identity.
///
/// Only identifiers the original Babel walker visited are recorded, so
/// membership in these maps is itself meaningful: a symbol has a declaration
/// span here iff its declaration identifier sits in a position the walker
/// recorded (inside the compiled function, outside type subtrees).
#[derive(Default)]
pub struct IdentifierLocIndex {
    /// Resolved identifier references, keyed by their `reference_id` cell.
    refs: FxHashMap<ReferenceId, IdentifierLocEntry>,
    /// Declaration (binding) identifier spans, keyed by their `symbol_id` cell.
    /// First declaration wins for redeclared symbols.
    decl_spans: FxHashMap<SymbolId, Span>,
}

impl IdentifierLocIndex {
    pub fn reference(&self, reference_id: ReferenceId) -> Option<&IdentifierLocEntry> {
        self.refs.get(&reference_id)
    }

    pub fn declaration_span(&self, symbol_id: SymbolId) -> Option<Span> {
        self.decl_spans.get(&symbol_id).copied()
    }
}

/// State for the identifier-loc pre-pass. The AST walk driving it lives in
/// [`super::pre_pass`], where it shares a single traversal (and a single
/// generated-walk instantiation) with the `FindContextIdentifiers` pre-pass.
#[derive(Default)]
pub(super) struct IdentifierLocVisitor {
    pub(super) index: IdentifierLocIndex,
    /// Tracks the current JSXOpeningElement's span while walking its name.
    pub(super) current_opening_element_span: Option<Span>,
    /// Depth of TS type subtrees currently being walked. Identifiers recorded
    /// while this is non-zero get `in_type_annotation = true`.
    pub(super) type_depth: u32,
}

impl IdentifierLocVisitor {
    /// `current_opening_element_span` is set only while walking a JSX
    /// element name, so it doubles as the is-JSX signal.
    pub(super) fn record_reference(&mut self, ident: &IdentifierReference<'_>) {
        let Some(reference_id) = ident.reference_id.get() else { return };
        self.index.refs.entry(reference_id).or_insert(IdentifierLocEntry {
            span: ident.span,
            opening_element_span: self.current_opening_element_span,
            in_type_annotation: self.type_depth > 0,
        });
    }

    pub(super) fn record_declaration(&mut self, ident: &BindingIdentifier<'_>) {
        let Some(symbol_id) = ident.symbol_id.get() else { return };
        self.index.decl_spans.entry(symbol_id).or_insert(ident.span);
    }
}
