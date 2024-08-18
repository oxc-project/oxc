use oxc_allocator::{Allocator, Box};
use oxc_ast::{
    ast::{Expression, IdentifierReference, Statement},
    AstBuilder,
};
use oxc_semantic::{ScopeTree, SymbolTable};
use oxc_span::{Atom, CompactStr, Span};
use oxc_syntax::{
    reference::{ReferenceFlag, ReferenceId},
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};

use crate::ancestor::{Ancestor, AncestorType};
mod ancestry;
mod ast_operations;
pub use ancestry::TraverseAncestry;
mod scoping;
pub use scoping::TraverseScoping;

/// Traverse context.
///
/// Passed to all AST visitor functions.
///
/// Provides ability to:
/// * Query parent/ancestor of current node via [`parent`], [`ancestor`], [`ancestors`].
/// * Get scopes tree and symbols table via [`scopes`], [`symbols`], [`scopes_mut`], [`symbols_mut`],
///   [`ancestor_scopes`].
/// * Create AST nodes via AST builder [`ast`].
/// * Allocate into arena via [`alloc`].
///
/// # Namespaced APIs
///
/// All APIs are provided via 2 routes:
///
/// 1. Directly on `TraverseCtx`.
/// 2. Via "namespaces".
///
/// | Direct                   | Namespaced                       |
/// |--------------------------|----------------------------------|
/// | `ctx.parent()`           | `ctx.ancestry.parent()`          |
/// | `ctx.current_scope_id()` | `ctx.scoping.current_scope_id()` |
/// | `ctx.alloc(thing)`       | `ctx.ast.alloc(thing)`           |
///
/// Purpose of the "namespaces" is to support if you want to mutate scope tree or symbol table
/// while holding an `&Ancestor`, or AST nodes obtained from an `&Ancestor`.
///
/// For example, this will not compile because it attempts to borrow `ctx`
/// immutably and mutably at same time:
///
/// ```nocompile
/// use oxc_ast::ast::*;
/// use oxc_traverse::{Ancestor, Traverse, TraverseCtx};
///
/// struct MyTransform;
/// impl<'a> Traverse<'a> for MyTransform {
///     fn enter_unary_expression(&mut self, unary_expr: &mut UnaryExpression<'a>, ctx: &mut TraverseCtx<'a>) {
///         // `right` is ultimately borrowed from `ctx`
///         let right = match ctx.parent() {
///             Ancestor::BinaryExpressionLeft(bin_expr) => bin_expr.right(),
///             _ => return,
///         };
///
///         // Won't compile! `ctx.scopes_mut()` attempts to mut borrow `ctx`
///         // while it's already borrowed by `right`.
///         let scope_tree_mut = ctx.scopes_mut();
///
///         // Use `right` later on
///         dbg!(right);
///     }
/// }
/// ```
///
/// You can fix this by using the "namespaced" methods instead.
/// This works because you can borrow `ctx.ancestry` and `ctx.scoping` simultaneously:
///
/// ```
/// use oxc_ast::ast::*;
/// use oxc_traverse::{Ancestor, Traverse, TraverseCtx};
///
/// struct MyTransform;
/// impl<'a> Traverse<'a> for MyTransform {
///     fn enter_unary_expression(&mut self, unary_expr: &mut UnaryExpression<'a>, ctx: &mut TraverseCtx<'a>) {
///         let right = match ctx.ancestry.parent() {
///             Ancestor::BinaryExpressionLeft(bin_expr) => bin_expr.right(),
///             _ => return,
///         };
///
///         let scope_tree_mut = ctx.scoping.scopes_mut();
///
///         dbg!(right);
///     }
/// }
/// ```
///
/// [`parent`]: `TraverseCtx::parent`
/// [`ancestor`]: `TraverseCtx::ancestor`
/// [`ancestors`]: `TraverseCtx::ancestors`
/// [`scopes`]: `TraverseCtx::scopes`
/// [`symbols`]: `TraverseCtx::symbols`
/// [`scopes_mut`]: `TraverseCtx::scopes_mut`
/// [`symbols_mut`]: `TraverseCtx::symbols_mut`
/// [`ancestor_scopes`]: `TraverseCtx::ancestor_scopes`
/// [`ast`]: `TraverseCtx::ast`
/// [`alloc`]: `TraverseCtx::alloc`
pub struct TraverseCtx<'a> {
    pub ancestry: TraverseAncestry<'a>,
    pub scoping: TraverseScoping,
    pub ast: AstBuilder<'a>,
}

// Public methods
impl<'a> TraverseCtx<'a> {
    /// Create new traversal context.
    pub fn new(scopes: ScopeTree, symbols: SymbolTable, allocator: &'a Allocator) -> Self {
        let ancestry = TraverseAncestry::new();
        let scoping = TraverseScoping::new(scopes, symbols);
        let ast = AstBuilder::new(allocator);
        Self { ancestry, scoping, ast }
    }

    /// Allocate a node in the arena.
    ///
    /// Returns a [`Box<T>`].
    ///
    /// Shortcut for `ctx.ast.alloc`.
    #[inline]
    pub fn alloc<T>(&self, node: T) -> Box<'a, T> {
        self.ast.alloc(node)
    }

    /// Get parent of current node.
    ///
    /// Shortcut for `ctx.ancestry.parent`.
    #[inline]

    pub fn parent(&self) -> &Ancestor<'a> {
        self.ancestry.parent()
    }

    /// Get ancestor of current node.
    ///
    /// `level` is number of levels above.
    /// `ancestor(1).unwrap()` is equivalent to `parent()`.
    ///
    /// Shortcut for `ctx.ancestry.ancestor`.
    #[inline]
    pub fn ancestor(&self, level: usize) -> Option<&Ancestor<'a>> {
        self.ancestry.ancestor(level)
    }

    /// Get iterator over ancestors, starting with closest ancestor.
    ///
    /// Shortcut for `ctx.ancestry.ancestors`.
    #[inline]
    pub fn ancestors<'b>(&'b self) -> impl Iterator<Item = &'b Ancestor<'a>> {
        self.ancestry.ancestors()
    }

    /// Get depth in the AST.
    ///
    /// Count includes current node. i.e. in `Program`, depth is 1.
    ///
    /// Shortcut for `self.ancestry.ancestors_depth`.
    #[inline]
    pub fn ancestors_depth(&self) -> usize {
        self.ancestry.ancestors_depth()
    }

    /// Get current scope ID.
    ///
    /// Shortcut for `ctx.scoping.current_scope_id`.
    #[inline]
    pub fn current_scope_id(&self) -> ScopeId {
        self.scoping.current_scope_id()
    }

    /// Get current scope flags.
    ///
    /// Shortcut for `ctx.scoping.current_scope_flags`.
    #[inline]
    pub fn current_scope_flags(&self) -> ScopeFlags {
        self.scoping.current_scope_flags()
    }

    /// Get scopes tree.
    ///
    /// Shortcut for `ctx.scoping.scopes`.
    #[inline]
    pub fn scopes(&self) -> &ScopeTree {
        self.scoping.scopes()
    }

    /// Get mutable scopes tree.
    ///
    /// Shortcut for `ctx.scoping.scopes_mut`.
    #[inline]
    pub fn scopes_mut(&mut self) -> &mut ScopeTree {
        self.scoping.scopes_mut()
    }

    /// Get symbols table.
    ///
    /// Shortcut for `ctx.scoping.symbols`.
    #[inline]
    pub fn symbols(&self) -> &SymbolTable {
        self.scoping.symbols()
    }

    /// Get mutable symbols table.
    ///
    /// Shortcut for `ctx.scoping.symbols_mut`.
    #[inline]
    pub fn symbols_mut(&mut self) -> &mut SymbolTable {
        self.scoping.symbols_mut()
    }

    /// Get iterator over scopes, starting with current scope and working up.
    ///
    /// This is a shortcut for `ctx.scoping.parent_scopes`.
    pub fn ancestor_scopes(&self) -> impl Iterator<Item = ScopeId> + '_ {
        self.scoping.ancestor_scopes()
    }

    /// Create new scope as child of provided scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    ///
    /// This is a shortcut for `ctx.scoping.create_child_scope`.
    pub fn create_child_scope(&mut self, parent_id: ScopeId, flags: ScopeFlags) -> ScopeId {
        self.scoping.create_child_scope(parent_id, flags)
    }

    /// Create new scope as child of current scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    ///
    /// This is a shortcut for `ctx.scoping.create_child_scope_of_current`.
    pub fn create_child_scope_of_current(&mut self, flags: ScopeFlags) -> ScopeId {
        self.scoping.create_child_scope_of_current(flags)
    }

    /// Insert a scope into scope tree below a statement.
    ///
    /// Statement must be in current scope.
    /// New scope is created as child of current scope.
    /// All child scopes of the statement are reassigned to be children of the new scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    ///
    /// This is a shortcut for `ctx.scoping.insert_scope_below_statement`.
    pub fn insert_scope_below_statement(&mut self, stmt: &Statement, flags: ScopeFlags) -> ScopeId {
        self.scoping.insert_scope_below_statement(stmt, flags)
    }

    /// Insert a scope into scope tree below an expression.
    ///
    /// Expression must be in current scope.
    /// New scope is created as child of current scope.
    /// All child scopes of the expression are reassigned to be children of the new scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    ///
    /// This is a shortcut for `ctx.scoping.insert_scope_below_expression`.
    pub fn insert_scope_below_expression(
        &mut self,
        expr: &Expression,
        flags: ScopeFlags,
    ) -> ScopeId {
        self.scoping.insert_scope_below_expression(expr, flags)
    }

    /// Generate UID.
    ///
    /// This is a shortcut for `ctx.scoping.generate_uid`.
    pub fn generate_uid(&mut self, name: &str, scope_id: ScopeId, flags: SymbolFlags) -> SymbolId {
        self.scoping.generate_uid(name, scope_id, flags)
    }

    /// Generate UID in current scope.
    ///
    /// This is a shortcut for `ctx.scoping.generate_uid_in_current_scope`.
    pub fn generate_uid_in_current_scope(&mut self, name: &str, flags: SymbolFlags) -> SymbolId {
        self.scoping.generate_uid_in_current_scope(name, flags)
    }

    /// Generate UID in root scope.
    ///
    /// This is a shortcut for `ctx.scoping.generate_uid_in_root_scope`.
    pub fn generate_uid_in_root_scope(&mut self, name: &str, flags: SymbolFlags) -> SymbolId {
        self.scoping.generate_uid_in_root_scope(name, flags)
    }

    /// Create a reference bound to a `SymbolId`.
    ///
    /// This is a shortcut for `ctx.scoping.create_bound_reference`.
    pub fn create_bound_reference(
        &mut self,
        symbol_id: SymbolId,
        flag: ReferenceFlag,
    ) -> ReferenceId {
        self.scoping.create_bound_reference(symbol_id, flag)
    }

    /// Create an `IdentifierReference` bound to a `SymbolId`.
    ///
    /// This is a shortcut for `ctx.scoping.create_bound_reference_id`.
    pub fn create_bound_reference_id(
        &mut self,
        span: Span,
        name: Atom<'a>,
        symbol_id: SymbolId,
        flag: ReferenceFlag,
    ) -> IdentifierReference<'a> {
        self.scoping.create_bound_reference_id(span, name, symbol_id, flag)
    }

    /// Create an unbound reference.
    ///
    /// This is a shortcut for `ctx.scoping.create_unbound_reference`.
    pub fn create_unbound_reference(
        &mut self,
        name: CompactStr,
        flag: ReferenceFlag,
    ) -> ReferenceId {
        self.scoping.create_unbound_reference(name, flag)
    }

    /// Create an unbound `IdentifierReference`.
    ///
    /// This is a shortcut for `ctx.scoping.create_unbound_reference_id`.
    pub fn create_unbound_reference_id(
        &mut self,
        span: Span,
        name: Atom<'a>,
        flag: ReferenceFlag,
    ) -> IdentifierReference<'a> {
        self.scoping.create_unbound_reference_id(span, name, flag)
    }

    /// Create a reference optionally bound to a `SymbolId`.
    ///
    /// If you know if there's a `SymbolId` or not, prefer `TraverseCtx::create_bound_reference`
    /// or `TraverseCtx::create_unbound_reference`.
    ///
    /// This is a shortcut for `ctx.scoping.create_reference`.
    pub fn create_reference(
        &mut self,
        name: CompactStr,
        symbol_id: Option<SymbolId>,
        flag: ReferenceFlag,
    ) -> ReferenceId {
        self.scoping.create_reference(name, symbol_id, flag)
    }

    /// Create an `IdentifierReference` optionally bound to a `SymbolId`.
    ///
    /// If you know if there's a `SymbolId` or not, prefer `TraverseCtx::create_bound_reference_id`
    /// or `TraverseCtx::create_unbound_reference_id`.
    ///
    /// This is a shortcut for `ctx.scoping.create_reference_id`.
    pub fn create_reference_id(
        &mut self,
        span: Span,
        name: Atom<'a>,
        symbol_id: Option<SymbolId>,
        flag: ReferenceFlag,
    ) -> IdentifierReference<'a> {
        self.scoping.create_reference_id(span, name, symbol_id, flag)
    }

    /// Create reference in current scope, looking up binding for `name`,
    ///
    /// This is a shortcut for `ctx.scoping.create_reference_in_current_scope`.
    pub fn create_reference_in_current_scope(
        &mut self,
        name: CompactStr,
        flag: ReferenceFlag,
    ) -> ReferenceId {
        self.scoping.create_reference_in_current_scope(name, flag)
    }

    /// Clone `IdentifierReference` based on the original reference's `SymbolId` and name.
    ///
    /// This method makes a lookup of the `SymbolId` for the reference. If you need to create multiple
    /// `IdentifierReference`s for the same binding, it is better to look up the `SymbolId` only once,
    /// and generate `IdentifierReference`s with `TraverseCtx::create_reference_id`.
    ///
    /// This is a shortcut for `ctx.scoping.clone_identifier_reference`.
    pub fn clone_identifier_reference(
        &mut self,
        ident: &IdentifierReference<'a>,
        flag: ReferenceFlag,
    ) -> IdentifierReference<'a> {
        self.scoping.clone_identifier_reference(ident, flag)
    }
}

// Methods used internally within crate
impl<'a> TraverseCtx<'a> {
    /// Shortcut for `self.ancestry.push_stack`, to make `walk_*` methods less verbose.
    ///
    /// # SAFETY
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    #[inline]
    pub(crate) fn push_stack(&mut self, ancestor: Ancestor<'a>) {
        self.ancestry.push_stack(ancestor);
    }

    /// Shortcut for `self.ancestry.pop_stack`, to make `walk_*` methods less verbose.
    ///
    /// # SAFETY
    /// See safety constraints of `TraverseAncestry.pop_stack`.
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    #[inline]

    pub(crate) unsafe fn pop_stack(&mut self) {
        self.ancestry.pop_stack();
    }

    /// Shortcut for `self.ancestry.retag_stack`, to make `walk_*` methods less verbose.
    ///
    /// # SAFETY
    /// See safety constraints of `TraverseAncestry.retag_stack`.
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    #[inline]

    pub(crate) unsafe fn retag_stack(&mut self, ty: AncestorType) {
        self.ancestry.retag_stack(ty);
    }

    /// Shortcut for `ctx.scoping.set_current_scope_id`, to make `walk_*` methods less verbose.
    #[inline]
    pub(crate) fn set_current_scope_id(&mut self, scope_id: ScopeId) {
        self.scoping.set_current_scope_id(scope_id);
    }
}
