//! Builds an index mapping identifier node-IDs to source locations.
//!
//! Walks the function's AST to collect `(node_id, start, SourceLocation, is_jsx)`
//! for every Identifier and JSXIdentifier node. Keyed by node_id for identity
//! lookups; each entry also stores `start` (byte offset) for range-containment
//! checks in `gather_captured_context`.

use rustc_hash::FxHashMap;

use crate::react_compiler_ast::expressions::*;
use crate::react_compiler_ast::jsx::JSXIdentifier;
use crate::react_compiler_ast::jsx::JSXOpeningElement;
use crate::react_compiler_ast::scope::ScopeId;
use crate::react_compiler_ast::scope::ScopeInfo;
use crate::react_compiler_ast::statements::FunctionDeclaration;
use crate::react_compiler_ast::visitor::AstWalker;
use crate::react_compiler_ast::visitor::Visitor;
use crate::react_compiler_hir::SourceLocation;

use crate::react_compiler_lowering::FunctionNode;

/// Source location and whether the identifier is a JSXIdentifier.
pub struct IdentifierLocEntry {
    /// The byte offset of the identifier (base.start). Stored here so that
    /// callers iterating by node_id can still do position-range containment
    /// checks without a separate bridge map.
    pub start: u32,
    pub loc: SourceLocation,
    pub is_jsx: bool,
    /// For JSX identifiers that are the root name of a JSXOpeningElement,
    /// stores the JSXOpeningElement's loc (which spans the full tag).
    pub opening_element_loc: Option<SourceLocation>,
    /// True if this identifier is the name of a function/class declaration
    /// (not an expression reference). Used by `gather_captured_context` to
    /// skip non-expression positions, matching the TS behavior where the
    /// Expression visitor doesn't visit declaration names.
    pub is_declaration_name: bool,
    /// True if this identifier sits inside a type annotation subtree
    /// (TypeAnnotation/TSTypeAnnotation/TypeAlias/TSTypeAliasDeclaration).
    /// `gather_captured_context` skips these to match the TS
    /// gatherCapturedContext traverse, which skips those subtrees; the
    /// hoisting analysis and FindContextIdentifiers do NOT skip them in TS.
    pub in_type_annotation: bool,
}

/// Index mapping node_id → IdentifierLocEntry for all Identifier
/// and JSXIdentifier nodes in a function's AST.
pub type IdentifierLocIndex = FxHashMap<u32, IdentifierLocEntry>;

struct IdentifierLocVisitor {
    index: IdentifierLocIndex,
    /// Tracks the current JSXOpeningElement's loc while walking its name.
    current_opening_element_loc: Option<SourceLocation>,
}

fn convert_loc(loc: &crate::react_compiler_ast::common::SourceLocation) -> SourceLocation {
    SourceLocation {
        start: crate::react_compiler_hir::Position {
            line: loc.start.line,
            column: loc.start.column,
            index: loc.start.index,
        },
        end: crate::react_compiler_hir::Position {
            line: loc.end.line,
            column: loc.end.column,
            index: loc.end.index,
        },
    }
}

impl IdentifierLocVisitor {
    fn insert_identifier(&mut self, node: &Identifier, is_declaration_name: bool) {
        if let (Some(nid), Some(start), Some(loc)) =
            (node.base.node_id, node.base.start, &node.base.loc)
        {
            self.index.insert(
                nid,
                IdentifierLocEntry {
                    start,
                    loc: convert_loc(loc),
                    is_jsx: false,
                    opening_element_loc: None,
                    is_declaration_name,
                    in_type_annotation: false,
                },
            );
        }
    }
}

impl<'ast> Visitor<'ast> for IdentifierLocVisitor {
    fn enter_identifier(&mut self, node: &'ast Identifier, _scope_stack: &[ScopeId]) {
        self.insert_identifier(node, false);
    }

    fn enter_jsx_identifier(&mut self, node: &'ast JSXIdentifier, _scope_stack: &[ScopeId]) {
        if let (Some(nid), Some(start), Some(loc)) =
            (node.base.node_id, node.base.start, &node.base.loc)
        {
            self.index.insert(
                nid,
                IdentifierLocEntry {
                    start,
                    loc: convert_loc(loc),
                    is_jsx: true,
                    opening_element_loc: self.current_opening_element_loc.clone(),
                    is_declaration_name: false,
                    in_type_annotation: false,
                },
            );
        }
    }

    fn enter_jsx_opening_element(
        &mut self,
        node: &'ast JSXOpeningElement,
        _scope_stack: &[ScopeId],
    ) {
        self.current_opening_element_loc = node.base.loc.as_ref().map(|loc| convert_loc(loc));
    }

    fn leave_jsx_opening_element(
        &mut self,
        _node: &'ast JSXOpeningElement,
        _scope_stack: &[ScopeId],
    ) {
        self.current_opening_element_loc = None;
    }

    // Visit function/class declaration and expression name identifiers,
    // which are not walked by the generic walker (to avoid affecting
    // other Visitor consumers like find_context_identifiers).
    fn enter_function_declaration(
        &mut self,
        node: &'ast FunctionDeclaration,
        _scope_stack: &[ScopeId],
    ) {
        if let Some(id) = &node.id {
            self.insert_identifier(id, true);
        }
    }

    fn enter_function_expression(
        &mut self,
        node: &'ast FunctionExpression,
        _scope_stack: &[ScopeId],
    ) {
        if let Some(id) = &node.id {
            self.insert_identifier(id, true);
        }
    }

    fn enter_class_declaration(
        &mut self,
        node: &'ast crate::react_compiler_ast::statements::ClassDeclaration,
        _scope_stack: &[ScopeId],
    ) {
        if let Some(id) = &node.id {
            self.insert_identifier(id, true);
        }
        // Class body identifiers are indexed via `visit_raw_node` (the walker
        // visits each `body.body` member's pre-extracted metadata).
    }

    fn enter_class_expression(
        &mut self,
        node: &'ast crate::react_compiler_ast::expressions::ClassExpression,
        _scope_stack: &[ScopeId],
    ) {
        if let Some(id) = &node.id {
            self.insert_identifier(id, true);
        }
    }

    /// Index identifiers inside unmodeled (`RawNode`) subtrees — type annotations,
    /// class bodies, decorators — from their pre-extracted metadata. The typed
    /// walker skips these, so this is where type-annotation identifiers (and the
    /// `in_type_annotation` flag) enter the index. `or_insert` keeps any richer
    /// entry already recorded by the typed walker.
    fn visit_raw_node(&mut self, raw: &'ast crate::react_compiler_ast::common::RawNode) {
        for id in &raw.idents {
            let Some(loc) = &id.loc else { continue };
            self.index.entry(id.node_id).or_insert(IdentifierLocEntry {
                start: id.start,
                loc: convert_loc(loc),
                is_jsx: id.is_jsx,
                opening_element_loc: None,
                is_declaration_name: false,
                in_type_annotation: id.in_type_annotation,
            });
        }
    }
}

/// Build an index of all Identifier and JSXIdentifier positions in a function's AST.
pub fn build_identifier_loc_index(
    func: &FunctionNode<'_>,
    scope_info: &ScopeInfo,
) -> IdentifierLocIndex {
    let func_scope =
        scope_info.resolve_scope_for_node(func.node_id()).unwrap_or(scope_info.program_scope);

    let mut visitor =
        IdentifierLocVisitor { index: FxHashMap::default(), current_opening_element_loc: None };
    let mut walker = AstWalker::with_initial_scope(scope_info, func_scope);

    // Visit the top-level function's own name identifier (if any),
    // since the walker only walks params + body, not the function node itself.
    match func {
        FunctionNode::FunctionDeclaration(d) => {
            if let Some(id) = &d.id {
                visitor.enter_identifier(id, &[]);
            }
            for param in &d.params {
                walker.walk_pattern(&mut visitor, param);
            }
            walker.walk_block_statement(&mut visitor, &d.body);
        }
        FunctionNode::FunctionExpression(e) => {
            if let Some(id) = &e.id {
                visitor.enter_identifier(id, &[]);
            }
            for param in &e.params {
                walker.walk_pattern(&mut visitor, param);
            }
            walker.walk_block_statement(&mut visitor, &e.body);
        }
        FunctionNode::ArrowFunctionExpression(a) => {
            for param in &a.params {
                walker.walk_pattern(&mut visitor, param);
            }
            match a.body.as_ref() {
                ArrowFunctionBody::BlockStatement(block) => {
                    walker.walk_block_statement(&mut visitor, block);
                }
                ArrowFunctionBody::Expression(expr) => {
                    walker.walk_expression(&mut visitor, expr);
                }
            }
        }
    }

    // Type-annotation and class-body identifiers (which the typed walker skips)
    // are indexed via the walker's `visit_raw_node` hook from each RawNode's
    // pre-extracted `idents`, so no separate JSON walk is needed.
    visitor.index
}
