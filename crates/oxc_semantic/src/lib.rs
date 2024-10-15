//! Semantic analysis of a JavaScript/TypeScript program.
//!
//! # Example
//! ```rust
#![doc = include_str!("../examples/simple.rs")]
//! ```

use std::ops::RangeBounds;
use std::sync::Arc;

use oxc_ast::{
    ast::IdentifierReference, comments_range, has_comments_between, AstKind, Comment, CommentsRange,
};
use oxc_cfg::ControlFlowGraph;
use oxc_span::{GetSpan, SourceType, Span};
pub use oxc_syntax::{
    module_record::ModuleRecord,
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};

pub mod dot;

mod binder;
mod builder;
mod checker;
mod class;
mod diagnostics;
mod jsdoc;
mod label;
mod module_record;
mod node;
mod reference;
mod scope;
mod stats;
mod symbol;
mod unresolved_stack;

pub use crate::{
    builder::{SemanticBuilder, SemanticBuilderReturn},
    jsdoc::{JSDoc, JSDocFinder, JSDocTag},
    node::{AstNode, AstNodes, NodeId},
    reference::{Reference, ReferenceFlags, ReferenceId},
    scope::ScopeTree,
    stats::Stats,
    symbol::{IsGlobalReference, SymbolTable},
};
use class::ClassTable;

/// Semantic analysis of a JavaScript/TypeScript program.
///
/// [`Semantic`] contains the results of analyzing a program, including the
/// [`Abstract Syntax Tree (AST)`], [`scope tree`], [`symbol table`], and
/// [`control flow graph (CFG)`].
///
/// Do not construct this struct directly; instead, use [`SemanticBuilder`].
///
/// [`Abstract Syntax Tree (AST)`]: crate::AstNodes
/// [`scope tree`]: crate::ScopeTree
/// [`symbol table`]: crate::SymbolTable
/// [`control flow graph (CFG)`]: crate::ControlFlowGraph
pub struct Semantic<'a> {
    /// Source code of the JavaScript/TypeScript program being analyzed.
    source_text: &'a str,

    /// What kind of source code is being analyzed. Comes from the parser.
    source_type: SourceType,

    /// The Abstract Syntax Tree (AST) nodes.
    nodes: AstNodes<'a>,

    /// The scope tree containing scopes and what identifier names are bound in
    /// each one.
    scopes: ScopeTree,

    /// Symbol table containing all symbols in the program and their references.
    symbols: SymbolTable,

    classes: ClassTable,

    /// Parsed comments.
    comments: &'a oxc_allocator::Vec<'a, Comment>,
    irregular_whitespaces: Box<[Span]>,

    module_record: Arc<ModuleRecord>,

    /// Parsed JSDoc comments.
    jsdoc: JSDocFinder<'a>,

    unused_labels: Vec<NodeId>,

    /// Control flow graph. Only present if [`Semantic`] is built with cfg
    /// creation enabled using [`SemanticBuilder::with_cfg`].
    cfg: Option<ControlFlowGraph>,
}

impl<'a> Semantic<'a> {
    /// Extract the [`SymbolTable`] and [`ScopeTree`] from the [`Semantic`]
    /// instance, consuming `self`.
    pub fn into_symbol_table_and_scope_tree(self) -> (SymbolTable, ScopeTree) {
        (self.symbols, self.scopes)
    }

    /// Source code of the JavaScript/TypeScript program being analyzed.
    pub fn source_text(&self) -> &'a str {
        self.source_text
    }

    /// What kind of source code is being analyzed. Comes from the parser.
    pub fn source_type(&self) -> &SourceType {
        &self.source_type
    }

    /// Nodes in the Abstract Syntax Tree (AST)
    pub fn nodes(&self) -> &AstNodes<'a> {
        &self.nodes
    }

    /// The [`ScopeTree`] containing scopes and what identifier names are bound in
    /// each one.
    pub fn scopes(&self) -> &ScopeTree {
        &self.scopes
    }

    pub fn classes(&self) -> &ClassTable {
        &self.classes
    }

    /// Get a mutable reference to the [`ScopeTree`].
    pub fn scopes_mut(&mut self) -> &mut ScopeTree {
        &mut self.scopes
    }

    pub fn set_irregular_whitespaces(&mut self, irregular_whitespaces: Box<[Span]>) {
        self.irregular_whitespaces = irregular_whitespaces;
    }

    /// Trivias (comments) found while parsing
    pub fn comments(&self) -> &[Comment] {
        self.comments
    }

    pub fn comments_range<R>(&self, range: R) -> CommentsRange<'_>
    where
        R: RangeBounds<u32>,
    {
        comments_range(self.comments, range)
    }

    pub fn has_comments_between(&self, span: Span) -> bool {
        has_comments_between(self.comments, span)
    }

    pub fn irregular_whitespaces(&self) -> &[Span] {
        &self.irregular_whitespaces
    }

    /// Parsed [`JSDoc`] comments.
    ///
    /// Will be empty if JSDoc parsing is disabled.
    pub fn jsdoc(&self) -> &JSDocFinder<'a> {
        &self.jsdoc
    }

    /// ESM module record containing imports and exports.
    pub fn module_record(&self) -> &ModuleRecord {
        self.module_record.as_ref()
    }

    /// [`SymbolTable`] containing all symbols in the program and their
    /// [`Reference`]s.
    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    pub fn unused_labels(&self) -> &Vec<NodeId> {
        &self.unused_labels
    }

    /// Control flow graph.
    ///
    /// Only present if [`Semantic`] is built with cfg creation enabled using
    /// [`SemanticBuilder::with_cfg`].
    pub fn cfg(&self) -> Option<&ControlFlowGraph> {
        self.cfg.as_ref()
    }

    /// Get statistics about data held in `Semantic`.
    pub fn stats(&self) -> Stats {
        #[allow(clippy::cast_possible_truncation)]
        Stats::new(
            self.nodes.len() as u32,
            self.scopes.len() as u32,
            self.symbols.len() as u32,
            self.symbols.references.len() as u32,
        )
    }

    pub fn is_unresolved_reference(&self, node_id: NodeId) -> bool {
        let reference_node = self.nodes.get_node(node_id);
        let AstKind::IdentifierReference(id) = reference_node.kind() else {
            return false;
        };
        self.scopes().root_unresolved_references().contains_key(id.name.as_str())
    }

    /// Find which scope a symbol is declared in
    pub fn symbol_scope(&self, symbol_id: SymbolId) -> ScopeId {
        self.symbols.get_scope_id(symbol_id)
    }

    /// Get all resolved references for a symbol
    pub fn symbol_references(&self, symbol_id: SymbolId) -> impl Iterator<Item = &Reference> + '_ {
        self.symbols.get_resolved_references(symbol_id)
    }

    pub fn symbol_declaration(&self, symbol_id: SymbolId) -> &AstNode<'a> {
        self.nodes.get_node(self.symbols.get_declaration(symbol_id))
    }

    pub fn is_reference_to_global_variable(&self, ident: &IdentifierReference) -> bool {
        self.scopes().root_unresolved_references().contains_key(ident.name.as_str())
    }

    pub fn reference_name(&self, reference: &Reference) -> &str {
        let node = self.nodes.get_node(reference.node_id());
        match node.kind() {
            AstKind::IdentifierReference(id) => id.name.as_str(),
            _ => unreachable!(),
        }
    }

    pub fn reference_span(&self, reference: &Reference) -> Span {
        let node = self.nodes.get_node(reference.node_id());
        node.kind().span()
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_ast::{ast::VariableDeclarationKind, AstKind};
    use oxc_span::{Atom, SourceType};

    use super::*;

    /// Create a [`Semantic`] from source code, assuming there are no syntax/semantic errors.
    fn get_semantic<'s, 'a: 's>(
        allocator: &'a Allocator,
        source: &'s str,
        source_type: SourceType,
    ) -> Semantic<'s> {
        let parse = oxc_parser::Parser::new(allocator, source, source_type).parse();
        assert!(parse.errors.is_empty());
        let semantic = SemanticBuilder::new().build(&parse.program);
        assert!(semantic.errors.is_empty(), "Parse error: {}", semantic.errors[0]);
        semantic.semantic
    }

    #[test]
    fn test_symbols() {
        let source = "
            let a;
            function foo(a) {
                return a + 1;
            }
            let b = a + foo(1);";
        let allocator = Allocator::default();
        let semantic = get_semantic(&allocator, source, SourceType::default());

        let top_level_a =
            semantic.scopes().get_binding(semantic.scopes().root_scope_id(), "a").unwrap();

        let decl = semantic.symbol_declaration(top_level_a);
        match decl.kind() {
            AstKind::VariableDeclarator(decl) => {
                assert_eq!(decl.kind, VariableDeclarationKind::Let);
            }
            kind => panic!("Expected VariableDeclarator for 'let', got {kind:?}"),
        }

        let references = semantic.symbol_references(top_level_a);
        assert_eq!(references.count(), 1);
    }

    #[test]
    fn test_top_level_symbols() {
        let source = "function Fn() {}";
        let allocator = Allocator::default();
        let semantic = get_semantic(&allocator, source, SourceType::default());

        let top_level_a = semantic
            .scopes()
            .iter_bindings()
            .find(|(_scope_id, _symbol_id, name)| name.as_str() == "Fn")
            .unwrap();
        assert_eq!(semantic.symbols.get_scope_id(top_level_a.1), top_level_a.0);
    }

    #[test]
    fn test_is_global() {
        let source = "
            var a = 0;
            function foo() {
            a += 1;
            }

            var b = a + 2;
        ";
        let allocator = Allocator::default();
        let semantic = get_semantic(&allocator, source, SourceType::default());
        for node in semantic.nodes() {
            if let AstKind::IdentifierReference(id) = node.kind() {
                assert!(!semantic.is_reference_to_global_variable(id));
            }
        }
    }

    #[test]
    fn type_alias_gets_reference() {
        let source = "type A = 1; type B = A";
        let allocator = Allocator::default();
        let source_type: SourceType = SourceType::default().with_typescript(true);
        let semantic = get_semantic(&allocator, source, source_type);
        assert!(semantic.symbols().references.len() == 1);
    }

    #[test]
    fn test_reference_resolutions_simple_read_write() {
        let alloc = Allocator::default();
        let target_symbol_name = Atom::from("a");
        let typescript = SourceType::ts();
        let sources = [
            // simple cases
            (SourceType::default(), "let a = 1; a = 2", ReferenceFlags::write()),
            (SourceType::default(), "let a = 1, b; b = a", ReferenceFlags::read()),
            (SourceType::default(), "let a = 1, b; b[a]", ReferenceFlags::read()),
            (SourceType::default(), "let a = 1, b = 1, c; c = a + b", ReferenceFlags::read()),
            (SourceType::default(), "function a() { return }; a()", ReferenceFlags::read()),
            (SourceType::default(), "class a {}; new a()", ReferenceFlags::read()),
            (SourceType::default(), "let a; function foo() { return a }", ReferenceFlags::read()),
            // pattern assignment
            (SourceType::default(), "let a = 1, b; b = { a }", ReferenceFlags::read()),
            (SourceType::default(), "let a, b; ({ b } = { a })", ReferenceFlags::read()),
            (SourceType::default(), "let a, b; ({ a } = { b })", ReferenceFlags::write()),
            (SourceType::default(), "let a, b; ([ b ] = [ a ])", ReferenceFlags::read()),
            (SourceType::default(), "let a, b; ([ a ] = [ b ])", ReferenceFlags::write()),
            // property access/mutation
            (SourceType::default(), "let a = { b: 1 }; a.b = 2", ReferenceFlags::read()),
            (SourceType::default(), "let a = { b: 1 }; a.b += 2", ReferenceFlags::read()),
            // parens are pass-through
            (SourceType::default(), "let a = 1, b; b = (a)", ReferenceFlags::read()),
            (SourceType::default(), "let a = 1, b; b = ++(a)", ReferenceFlags::read_write()),
            (SourceType::default(), "let a = 1, b; b = ++((((a))))", ReferenceFlags::read_write()),
            (SourceType::default(), "let a = 1, b; b = ((++((a))))", ReferenceFlags::read_write()),
            // simple binops/calls for sanity check
            (SourceType::default(), "let a, b; a + b", ReferenceFlags::read()),
            (SourceType::default(), "let a, b; b(a)", ReferenceFlags::read()),
            (SourceType::default(), "let a, b; a = 5", ReferenceFlags::write()),
            // unary op counts as write, but checking continues up tree
            (SourceType::default(), "let a = 1, b; b = ++a", ReferenceFlags::read_write()),
            (SourceType::default(), "let a = 1, b; b = --a", ReferenceFlags::read_write()),
            (SourceType::default(), "let a = 1, b; b = a++", ReferenceFlags::read_write()),
            (SourceType::default(), "let a = 1, b; b = a--", ReferenceFlags::read_write()),
            // assignment expressions count as read-write
            (SourceType::default(), "let a = 1, b; b = a += 5", ReferenceFlags::read_write()),
            (SourceType::default(), "let a = 1; a += 5", ReferenceFlags::read_write()),
            // note: we consider a to be written, and the read of `1` propagates upwards
            (SourceType::default(), "let a, b; b = a = 1", ReferenceFlags::read_write()),
            (SourceType::default(), "let a, b; b = (a = 1)", ReferenceFlags::read_write()),
            (SourceType::default(), "let a, b, c; b = c = a", ReferenceFlags::read()),
            // sequences return last read_write in sequence
            (SourceType::default(), "let a, b; b = (0, a++)", ReferenceFlags::read_write()),
            // loops
            (
                SourceType::default(),
                "var a, arr = [1, 2, 3]; for(a in arr) { break }",
                ReferenceFlags::write(),
            ),
            (
                SourceType::default(),
                "var a, obj = { }; for(a of obj) { break }",
                ReferenceFlags::write(),
            ),
            (SourceType::default(), "var a; for(; false; a++) { }", ReferenceFlags::read_write()),
            (SourceType::default(), "var a = 1; while(a < 5) { break }", ReferenceFlags::read()),
            // if statements
            (
                SourceType::default(),
                "let a; if (a) { true } else { false }",
                ReferenceFlags::read(),
            ),
            (
                SourceType::default(),
                "let a, b; if (a == b) { true } else { false }",
                ReferenceFlags::read(),
            ),
            (
                SourceType::default(),
                "let a, b; if (b == a) { true } else { false }",
                ReferenceFlags::read(),
            ),
            // identifiers not in last read_write are also considered a read (at
            // least, or now)
            (SourceType::default(), "let a, b; b = (a, 0)", ReferenceFlags::read()),
            (SourceType::default(), "let a, b; b = (--a, 0)", ReferenceFlags::read_write()),
            // other reads after a is written
            // a = 1 writes, but the CallExpression reads the rhs (1) so a isn't read
            (
                SourceType::default(),
                "let a; function foo(a) { return a }; foo(a = 1)",
                ReferenceFlags::read_write(),
            ),
            // member expression
            (SourceType::default(), "let a; a.b = 1", ReferenceFlags::read()),
            (SourceType::default(), "let a; let b; b[a += 1] = 1", ReferenceFlags::read_write()),
            (
                SourceType::default(),
                "let a; let b; let c; b[c[a = c['a']] = 'c'] = 'b'",
                ReferenceFlags::read_write(),
            ),
            (
                SourceType::default(),
                "let a; let b; let c; a[c[b = c['a']] = 'c'] = 'b'",
                ReferenceFlags::read(),
            ),
            (SourceType::default(), "console.log;let a=0;a++", ReferenceFlags::write()),
            // typescript
            (typescript, "let a: number = 1; (a as any) = true", ReferenceFlags::write()),
            (typescript, "let a: number = 1; a = true as any", ReferenceFlags::write()),
            (typescript, "let a: number = 1; a = 2 as const", ReferenceFlags::write()),
            (typescript, "let a: number = 1; a = 2 satisfies number", ReferenceFlags::write()),
            (typescript, "let a: number; (a as any) = 1;", ReferenceFlags::write()),
        ];

        for (source_type, source, flags) in sources {
            let semantic = get_semantic(&alloc, source, source_type);
            let a_id =
                semantic.scopes().get_root_binding(&target_symbol_name).unwrap_or_else(|| {
                    panic!("no references for '{target_symbol_name}' found");
                });
            let a_refs: Vec<_> = semantic.symbol_references(a_id).collect();
            let num_refs = a_refs.len();

            assert!(
                num_refs == 1,
                "expected to find 1 reference to '{target_symbol_name}' but {num_refs} were found\n\nsource:\n{source}"
            );
            let ref_type = a_refs[0];
            if flags.is_write() {
                assert!(
                    ref_type.is_write(),
                    "expected reference to '{target_symbol_name}' to be write\n\nsource:\n{source}"
                );
            } else {
                assert!(
                    !ref_type.is_write(),
                    "expected reference to '{target_symbol_name}' not to have been written to, but it is\n\nsource:\n{source}"
                );
            }
            if flags.is_read() {
                assert!(
                    ref_type.is_read(),
                    "expected reference to '{target_symbol_name}' to be read\n\nsource:\n{source}"
                );
            } else {
                assert!(
                    !ref_type.is_read(),
                    "expected reference to '{target_symbol_name}' not to be read, but it is\n\nsource:\n{source}"
                );
            }
        }
    }
}
