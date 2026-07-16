use std::cell::Cell;

use oxc_allocator::{Allocator, GetAllocator};
use oxc_ast::builder::{AstBuild, AstBuilder, AstCounts, GetAstBuilder};
use oxc_syntax::{node::NodeId, reference::ReferenceId, scope::ScopeId, symbol::SymbolId};

/// [`AstBuild`]er which wraps [`AstBuilder`] and counts AST nodes, scopes, symbols,
/// and references as they are created.
///
/// Counts are returned to the caller in `ParserReturn::ast_counts`, so `SemanticBuilder`
/// can pre-allocate sufficient capacity without traversing the AST to count.
///
/// Counts must be saved and restored when the parser rewinds to a checkpoint,
/// so that speculatively-parsed nodes which are discarded are not counted.
pub struct CountingAstBuilder<'a> {
    ast: AstBuilder<'a>,
    nodes: Cell<u32>,
    scopes: Cell<u32>,
    symbols: Cell<u32>,
    references: Cell<u32>,
}

impl<'a> CountingAstBuilder<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            ast: AstBuilder::new(allocator),
            nodes: Cell::new(0),
            scopes: Cell::new(0),
            symbols: Cell::new(0),
            references: Cell::new(0),
        }
    }

    pub fn counts(&self) -> AstCounts {
        AstCounts {
            nodes: self.nodes.get(),
            scopes: self.scopes.get(),
            symbols: self.symbols.get(),
            references: self.references.get(),
        }
    }

    pub fn set_counts(&self, counts: AstCounts) {
        self.nodes.set(counts.nodes);
        self.scopes.set(counts.scopes);
        self.symbols.set(counts.symbols);
        self.references.set(counts.references);
    }

    /// Count a symbol which is not created via a `BindingIdentifier` (enum member names).
    pub fn count_extra_symbol(&self) {
        self.symbols.set(self.symbols.get() + 1);
    }
}

impl<'a> GetAllocator<'a> for CountingAstBuilder<'a> {
    #[inline]
    fn allocator(&self) -> &'a Allocator {
        self.ast.allocator()
    }
}

impl<'a> AstBuild<'a> for CountingAstBuilder<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.nodes.set(self.nodes.get() + 1);
        NodeId::DUMMY
    }

    #[inline]
    fn scope_id(&self) -> Option<ScopeId> {
        self.scopes.set(self.scopes.get() + 1);
        None
    }

    #[inline]
    fn symbol_id(&self) -> Option<SymbolId> {
        self.symbols.set(self.symbols.get() + 1);
        None
    }

    #[inline]
    fn reference_id(&self) -> Option<ReferenceId> {
        self.references.set(self.references.get() + 1);
        None
    }
}

impl<'a> GetAstBuilder<'a> for CountingAstBuilder<'a> {
    type Builder = Self;

    #[inline]
    fn builder(&self) -> &Self {
        self
    }
}
