use oxc_allocator::{Allocator, Box};
use oxc_ast::{
    ast::{Expression, IdentifierReference, Statement},
    AstBuilder,
};
use oxc_semantic::{ScopeTree, SymbolTable};
use oxc_span::{Atom, CompactStr, Span};
use oxc_syntax::{
    reference::{ReferenceFlags, ReferenceId},
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};

use crate::{
    ancestor::{Ancestor, AncestorType},
    ast_operations::{get_var_name_from_node, GatherNodeParts},
};

mod ancestry;
mod bound_identifier;
mod maybe_bound_identifier;
mod reusable;
mod scoping;
use ancestry::PopToken;
pub use ancestry::TraverseAncestry;
pub use bound_identifier::BoundIdentifier;
pub use maybe_bound_identifier::MaybeBoundIdentifier;
pub use reusable::ReusableTraverseCtx;
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
    pub fn parent<'t>(&'t self) -> Ancestor<'a, 't> {
        self.ancestry.parent()
    }

    /// Get ancestor of current node.
    ///
    /// `level` is number of levels above parent.
    /// `ancestor(0)` is equivalent to `parent()` (but better to use `parent()` as it's more efficient).
    ///
    /// If `level` is out of bounds (above `Program`), returns `Ancestor::None`.
    ///
    /// Shortcut for `ctx.ancestry.ancestor`.
    #[inline]
    pub fn ancestor<'t>(&'t self, level: usize) -> Ancestor<'a, 't> {
        self.ancestry.ancestor(level)
    }

    /// Get iterator over ancestors, starting with parent and working up.
    ///
    /// Last `Ancestor` returned will be `Program`. `Ancestor::None` is not included in iteration.
    ///
    /// Shortcut for `ctx.ancestry.ancestors`.
    #[inline]
    pub fn ancestors<'t>(&'t self) -> impl Iterator<Item = Ancestor<'a, 't>> {
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

    /// Get current var hoisting scope ID.
    ///
    /// Shortcut for `ctx.scoping.current_hoist_scope_id`.
    #[inline]
    pub fn current_hoist_scope_id(&self) -> ScopeId {
        self.scoping.current_hoist_scope_id()
    }

    /// Get current block scope ID.
    ///
    /// Shortcut for `ctx.scoping.current_block_scope_id`.
    #[inline]
    pub fn current_block_scope_id(&self) -> ScopeId {
        self.scoping.current_block_scope_id()
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
    #[inline]
    pub fn ancestor_scopes(&self) -> impl Iterator<Item = ScopeId> + '_ {
        self.scoping.ancestor_scopes()
    }

    /// Create new scope as child of provided scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    ///
    /// This is a shortcut for `ctx.scoping.create_child_scope`.
    #[inline]
    pub fn create_child_scope(&mut self, parent_id: ScopeId, flags: ScopeFlags) -> ScopeId {
        self.scoping.create_child_scope(parent_id, flags)
    }

    /// Create new scope as child of current scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    ///
    /// This is a shortcut for `ctx.scoping.create_child_scope_of_current`.
    #[inline]
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
    #[inline]
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
    #[inline]
    pub fn insert_scope_below_expression(
        &mut self,
        expr: &Expression,
        flags: ScopeFlags,
    ) -> ScopeId {
        self.scoping.insert_scope_below_expression(expr, flags)
    }

    /// Remove scope for an expression from the scope chain.
    ///
    /// Delete the scope and set parent of its child scopes to its parent scope.
    /// e.g.:
    /// * Starting scopes parentage `A -> B`, `B -> C`, `B -> D`.
    /// * Remove scope `B` from chain.
    /// * End result: scopes `A -> C`, `A -> D`.
    ///
    /// Use this when removing an expression which owns a scope, without removing its children.
    /// For example when unwrapping `(() => foo)()` to just `foo`.
    /// `foo` here could be an expression which itself contains scopes.
    ///
    /// This is a shortcut for `ctx.scoping.remove_scope_for_expression`.
    pub fn remove_scope_for_expression(&mut self, scope_id: ScopeId, expr: &Expression) {
        self.scoping.remove_scope_for_expression(scope_id, expr);
    }

    /// Generate binding.
    ///
    /// Creates a symbol with the provided name and flags and adds it to the specified scope.
    ///
    /// This is a shortcut for `ctx.scoping.generate_binding`.
    pub fn generate_binding(
        &mut self,
        name: Atom<'a>,
        scope_id: ScopeId,
        flags: SymbolFlags,
    ) -> BoundIdentifier<'a> {
        self.scoping.generate_binding(name, scope_id, flags)
    }

    /// Generate binding in current scope.
    ///
    /// Creates a symbol with the provided name and flags and adds it to the current scope.
    ///
    /// This is a shortcut for `ctx.scoping.generate_binding_in_current_scope`.
    pub fn generate_binding_in_current_scope(
        &mut self,
        name: Atom<'a>,
        flags: SymbolFlags,
    ) -> BoundIdentifier<'a> {
        self.scoping.generate_binding_in_current_scope(name, flags)
    }

    /// Generate UID var name.
    ///
    /// Finds a unique variable name which does clash with any other variables used in the program.
    ///
    /// See [`TraverseScoping::generate_uid_name`] for important information on how UIDs are generated.
    /// There are some potential "gotchas".
    ///
    /// This is a shortcut for `ctx.scoping.generate_uid_name`.
    pub fn generate_uid_name(&mut self, name: &str) -> CompactStr {
        self.scoping.generate_uid_name(name)
    }

    /// Generate UID.
    ///
    /// See also comments on [`TraverseScoping::generate_uid_name`] for important information
    /// on how UIDs are generated. There are some potential "gotchas".
    #[inline]
    pub fn generate_uid(
        &mut self,
        name: &str,
        scope_id: ScopeId,
        flags: SymbolFlags,
    ) -> BoundIdentifier<'a> {
        // Get name for UID
        let name = self.generate_uid_name(name);
        let name_atom = self.ast.atom(&name);
        let symbol_id = self.scoping.add_binding(&name, scope_id, flags);
        BoundIdentifier::new(name_atom, symbol_id)
    }

    /// Generate UID in current scope.
    ///
    /// See also comments on [`TraverseScoping::generate_uid_name`] for important information
    /// on how UIDs are generated. There are some potential "gotchas".
    #[inline]
    pub fn generate_uid_in_current_scope(
        &mut self,
        name: &str,
        flags: SymbolFlags,
    ) -> BoundIdentifier<'a> {
        self.generate_uid(name, self.current_scope_id(), flags)
    }

    /// Generate UID in root scope.
    ///
    /// See also comments on [`TraverseScoping::generate_uid_name`] for important information
    /// on how UIDs are generated. There are some potential "gotchas".
    #[inline]
    pub fn generate_uid_in_root_scope(
        &mut self,
        name: &str,
        flags: SymbolFlags,
    ) -> BoundIdentifier<'a> {
        self.generate_uid(name, self.scopes().root_scope_id(), flags)
    }

    /// Generate UID based on node.
    ///
    /// Recursively gathers the identifying names of a node, and joins them with `$`.
    ///
    /// Based on Babel's `scope.generateUidBasedOnNode` logic.
    /// <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L543>
    #[inline]
    pub fn generate_uid_based_on_node<N: GatherNodeParts<'a>>(
        &mut self,
        node: &N,
        scope_id: ScopeId,
        flags: SymbolFlags,
    ) -> BoundIdentifier<'a> {
        let name = get_var_name_from_node(node);
        self.generate_uid(&name, scope_id, flags)
    }

    /// Generate UID in current scope based on node.
    ///
    /// See also comments on [`TraverseScoping::generate_uid_name`] for important information
    /// on how UIDs are generated. There are some potential "gotchas".
    #[inline]
    pub fn generate_uid_in_current_scope_based_on_node<N: GatherNodeParts<'a>>(
        &mut self,
        node: &N,
        flags: SymbolFlags,
    ) -> BoundIdentifier<'a> {
        self.generate_uid_based_on_node(node, self.current_scope_id(), flags)
    }

    /// Generate UID in current hoist scope.
    ///
    /// See also comments on [`TraverseScoping::generate_uid_name`] for important information
    /// on how UIDs are generated. There are some potential "gotchas".
    #[inline]
    pub fn generate_uid_in_current_hoist_scope(&mut self, name: &str) -> BoundIdentifier<'a> {
        self.generate_uid(name, self.current_hoist_scope_id(), SymbolFlags::FunctionScopedVariable)
    }

    /// Generate UID in current hoist scope based on node.
    ///
    /// See also comments on [`TraverseScoping::generate_uid_name`] for important information
    /// on how UIDs are generated. There are some potential "gotchas".
    #[inline]
    pub fn generate_uid_in_current_hoist_scope_based_on_node<N: GatherNodeParts<'a>>(
        &mut self,
        node: &N,
    ) -> BoundIdentifier<'a> {
        let name = get_var_name_from_node(node);
        self.generate_uid_in_current_hoist_scope(&name)
    }

    /// Create a reference bound to a `SymbolId`.
    ///
    /// This is a shortcut for `ctx.scoping.create_bound_reference`.
    #[inline]
    pub fn create_bound_reference(
        &mut self,
        symbol_id: SymbolId,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        self.scoping.create_bound_reference(symbol_id, flags)
    }

    /// Create an `IdentifierReference` bound to a `SymbolId`.
    pub fn create_bound_ident_reference(
        &mut self,
        span: Span,
        name: Atom<'a>,
        symbol_id: SymbolId,
        flags: ReferenceFlags,
    ) -> IdentifierReference<'a> {
        let reference_id = self.create_bound_reference(symbol_id, flags);
        self.ast.identifier_reference_with_reference_id(span, name, reference_id)
    }

    /// Create an `Expression::Identifier` bound to a `SymbolId`.
    pub fn create_bound_ident_expr(
        &mut self,
        span: Span,
        name: Atom<'a>,
        symbol_id: SymbolId,
        flags: ReferenceFlags,
    ) -> Expression<'a> {
        let ident = self.create_bound_ident_reference(span, name, symbol_id, flags);
        Expression::Identifier(self.ast.alloc(ident))
    }

    /// Create an unbound reference.
    ///
    /// This is a shortcut for `ctx.scoping.create_unbound_reference`.
    #[inline]
    pub fn create_unbound_reference(&mut self, name: &str, flags: ReferenceFlags) -> ReferenceId {
        self.scoping.create_unbound_reference(name, flags)
    }

    /// Create an unbound `IdentifierReference`.
    pub fn create_unbound_ident_reference(
        &mut self,
        span: Span,
        name: Atom<'a>,
        flags: ReferenceFlags,
    ) -> IdentifierReference<'a> {
        let reference_id = self.create_unbound_reference(name.as_str(), flags);
        self.ast.identifier_reference_with_reference_id(span, name, reference_id)
    }

    /// Create an unbound `Expression::Identifier`.
    pub fn create_unbound_ident_expr(
        &mut self,
        span: Span,
        name: Atom<'a>,
        flags: ReferenceFlags,
    ) -> Expression<'a> {
        let ident = self.create_unbound_ident_reference(span, name, flags);
        Expression::Identifier(self.ast.alloc(ident))
    }

    /// Create a reference optionally bound to a `SymbolId`.
    ///
    /// If you know if there's a `SymbolId` or not, prefer `TraverseCtx::create_bound_reference`
    /// or `TraverseCtx::create_unbound_reference`.
    ///
    /// This is a shortcut for `ctx.scoping.create_reference`.
    #[inline]
    pub fn create_reference(
        &mut self,
        name: &str,
        symbol_id: Option<SymbolId>,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        self.scoping.create_reference(name, symbol_id, flags)
    }

    /// Create an `IdentifierReference` optionally bound to a `SymbolId`.
    ///
    /// If you know if there's a `SymbolId` or not, prefer `TraverseCtx::create_bound_ident_reference`
    /// or `TraverseCtx::create_unbound_ident_reference`.
    pub fn create_ident_reference(
        &mut self,
        span: Span,
        name: Atom<'a>,
        symbol_id: Option<SymbolId>,
        flags: ReferenceFlags,
    ) -> IdentifierReference<'a> {
        if let Some(symbol_id) = symbol_id {
            self.create_bound_ident_reference(span, name, symbol_id, flags)
        } else {
            self.create_unbound_ident_reference(span, name, flags)
        }
    }

    /// Create an `Expression::Identifier` optionally bound to a `SymbolId`.
    ///
    /// If you know if there's a `SymbolId` or not, prefer `TraverseCtx::create_bound_ident_expr`
    /// or `TraverseCtx::create_unbound_ident_expr`.
    pub fn create_ident_expr(
        &mut self,
        span: Span,
        name: Atom<'a>,
        symbol_id: Option<SymbolId>,
        flags: ReferenceFlags,
    ) -> Expression<'a> {
        if let Some(symbol_id) = symbol_id {
            self.create_bound_ident_expr(span, name, symbol_id, flags)
        } else {
            self.create_unbound_ident_expr(span, name, flags)
        }
    }

    /// Create reference in current scope, looking up binding for `name`,
    ///
    /// This is a shortcut for `ctx.scoping.create_reference_in_current_scope`.
    #[inline]
    pub fn create_reference_in_current_scope(
        &mut self,
        name: &str,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        self.scoping.create_reference_in_current_scope(name, flags)
    }

    /// Delete a reference.
    ///
    /// Provided `name` must match `reference_id`.
    ///
    /// This is a shortcut for `ctx.scoping.delete_reference`.
    pub fn delete_reference(&mut self, reference_id: ReferenceId, name: &str) {
        self.scoping.delete_reference(reference_id, name);
    }

    /// Delete reference for an `IdentifierReference`.
    ///
    /// This is a shortcut for `ctx.scoping.delete_reference_for_identifier`.
    pub fn delete_reference_for_identifier(&mut self, ident: &IdentifierReference) {
        self.scoping.delete_reference_for_identifier(ident);
    }

    /// Rename symbol.
    ///
    /// Preserves original order of bindings for scope.
    ///
    /// The following must be true for successful operation:
    /// * Binding exists in specified scope for `symbol_id`.
    /// * No binding already exists in scope for `new_name`.
    ///
    /// Panics in debug mode if either of the above are not satisfied.
    ///
    /// This is a shortcut for `ctx.scoping.rename_symbol`.
    pub fn rename_symbol(&mut self, symbol_id: SymbolId, scope_id: ScopeId, new_name: CompactStr) {
        self.scoping.rename_symbol(symbol_id, scope_id, new_name);
    }
}

// Methods used internally within crate
impl<'a> TraverseCtx<'a> {
    /// Create new traversal context.
    ///
    /// # SAFETY
    /// This function must not be public to maintain soundness of [`TraverseAncestry`].
    pub(crate) fn new(scopes: ScopeTree, symbols: SymbolTable, allocator: &'a Allocator) -> Self {
        let ancestry = TraverseAncestry::new();
        let scoping = TraverseScoping::new(scopes, symbols);
        let ast = AstBuilder::new(allocator);
        Self { ancestry, scoping, ast }
    }

    /// Shortcut for `self.ancestry.push_stack`, to make `walk_*` methods less verbose.
    ///
    /// # SAFETY
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    #[inline]
    pub(crate) fn push_stack(&mut self, ancestor: Ancestor<'a, 'static>) -> PopToken {
        self.ancestry.push_stack(ancestor)
    }

    /// Shortcut for `self.ancestry.pop_stack`, to make `walk_*` methods less verbose.
    ///
    /// # SAFETY
    /// See safety constraints of `TraverseAncestry.pop_stack`.
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    #[inline]
    pub(crate) unsafe fn pop_stack(&mut self, token: PopToken) {
        self.ancestry.pop_stack(token);
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

    /// Shortcut for `ctx.scoping.set_current_hoist_scope_id`, to make `walk_*` methods less verbose.
    #[inline]
    pub(crate) fn set_current_hoist_scope_id(&mut self, scope_id: ScopeId) {
        self.scoping.set_current_hoist_scope_id(scope_id);
    }

    /// Shortcut for `ctx.scoping.set_current_block_scope_id`, to make `walk_*` methods less verbose.
    #[inline]
    pub(crate) fn set_current_block_scope_id(&mut self, scope_id: ScopeId) {
        self.scoping.set_current_block_scope_id(scope_id);
    }
}
