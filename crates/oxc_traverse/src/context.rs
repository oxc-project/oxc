use oxc_allocator::{Allocator, Box};
use oxc_ast::AstBuilder;
use oxc_semantic::{ScopeTree, SymbolTable};
use oxc_syntax::scope::{ScopeFlags, ScopeId};

use crate::ancestor::{Ancestor, AncestorType};

const INITIAL_STACK_CAPACITY: usize = 64; // 64 entries = 1 KiB

/// Traverse context.
///
/// Passed to all AST visitor functions.
///
/// Provides ability to:
/// * Query parent/ancestor of current node via [`parent`], [`ancestor`], [`find_ancestor`].
/// * Get scopes tree and symbols table via [`scopes`], [`symbols`], [`scopes_mut`], [`symbols_mut`],
///   [`find_scope`], [`find_scope_by_flags`].
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
/// [`find_ancestor`]: `TraverseCtx::find_ancestor`
/// [`scopes`]: `TraverseCtx::scopes`
/// [`symbols`]: `TraverseCtx::symbols`
/// [`scopes_mut`]: `TraverseCtx::scopes_mut`
/// [`symbols_mut`]: `TraverseCtx::symbols_mut`
/// [`find_scope`]: `TraverseCtx::find_scope`
/// [`find_scope_by_flags`]: `TraverseCtx::find_scope_by_flags`
/// [`ast`]: `TraverseCtx::ast`
/// [`alloc`]: `TraverseCtx::alloc`
pub struct TraverseCtx<'a> {
    pub ancestry: TraverseAncestry<'a>,
    pub scoping: TraverseScoping,
    pub ast: AstBuilder<'a>,
}

/// Traverse ancestry context.
///
/// Contains a stack of `Ancestor`s, and provides methods to get parent/ancestor of current node.
///
/// `walk_*` methods push/pop `Ancestor`s to `stack` when entering/exiting nodes.
///
/// # SAFETY
/// This type MUST NOT be mutable by consumer.
///
/// The safety scheme is entirely reliant on `stack` being in sync with the traversal,
/// to prevent consumer from accessing fields of nodes which traversal has passed through,
/// so as to not violate Rust's aliasing rules.
/// If consumer could alter `stack` in any way, they could break the safety invariants and cause UB.
///
/// We prevent this in 3 ways:
/// 1. `TraverseAncestry`'s `stack` field is private.
/// 2. Public methods of `TraverseAncestry` provide no means for mutating `stack`.
/// 3. Visitors receive a `&mut TraverseCtx`, but cannot overwrite its `ancestry` field because they:
///    a. cannot create a new `TraverseAncestry` - `TraverseAncestry::new` is private.
///    b. cannot obtain an owned `TraverseAncestry` from a `&TraverseAncestry`
///       - `TraverseAncestry` is not `Clone`.
pub struct TraverseAncestry<'a> {
    stack: Vec<Ancestor<'a>>,
}

/// Traverse scope context.
///
/// Contains the scope tree and symbols table, and provides methods to access them.
///
/// `current_scope_id` is the ID of current scope during traversal.
/// `walk_*` functions update this field when entering/exiting a scope.
pub struct TraverseScoping {
    scopes: ScopeTree,
    symbols: SymbolTable,
    current_scope_id: ScopeId,
}

/// Return value when using [`TraverseCtx::find_ancestor`].
pub enum FinderRet<T> {
    Found(T),
    Stop,
    Continue,
}

// Public methods
impl<'a> TraverseCtx<'a> {
    /// Create new traversal context.
    pub(crate) fn new(scopes: ScopeTree, symbols: SymbolTable, allocator: &'a Allocator) -> Self {
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
    #[allow(unsafe_code)]
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

    /// Walk up trail of ancestors to find a node.
    ///
    /// `finder` should return:
    /// * `FinderRet::Found(value)` to stop walking and return `Some(value)`.
    /// * `FinderRet::Stop` to stop walking and return `None`.
    /// * `FinderRet::Continue` to continue walking up.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_ast::ast::ThisExpression;
    /// use oxc_traverse::{Ancestor, FinderRet, Traverse, TraverseCtx};
    ///
    /// struct MyTraverse;
    /// impl<'a> Traverse<'a> for MyTraverse {
    ///     fn enter_this_expression(&mut self, this_expr: &mut ThisExpression, ctx: &mut TraverseCtx<'a>) {
    ///         // Get name of function where `this` is bound.
    ///         // NB: This example doesn't handle `this` in class fields or static blocks.
    ///         let fn_id = ctx.find_ancestor(|ancestor| {
    ///             match ancestor {
    ///                 Ancestor::FunctionBody(func) => FinderRet::Found(func.id()),
    ///                 Ancestor::FunctionParams(func) => FinderRet::Found(func.id()),
    ///                 _ => FinderRet::Continue
    ///             }
    ///         });
    ///     }
    /// }
    /// ```
    ///
    /// Shortcut for `self.ancestry.find_ancestor`.
    pub fn find_ancestor<'c, F, O>(&'c self, finder: F) -> Option<O>
    where
        F: Fn(&'c Ancestor<'a>) -> FinderRet<O>,
    {
        self.ancestry.find_ancestor(finder)
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

    /// Walk up trail of scopes to find a scope.
    ///
    /// `finder` is called with `ScopeId`.
    ///
    /// `finder` should return:
    /// * `FinderRet::Found(value)` to stop walking and return `Some(value)`.
    /// * `FinderRet::Stop` to stop walking and return `None`.
    /// * `FinderRet::Continue` to continue walking up.
    ///
    /// This is a shortcut for `ctx.scoping.find_scope`.
    pub fn find_scope<F, O>(&self, finder: F) -> Option<O>
    where
        F: Fn(ScopeId) -> FinderRet<O>,
    {
        self.scoping.find_scope(finder)
    }

    /// Walk up trail of scopes to find a scope by checking `ScopeFlags`.
    ///
    /// `finder` is called with `ScopeFlags`.
    ///
    /// `finder` should return:
    /// * `FinderRet::Found(value)` to stop walking and return `Some(value)`.
    /// * `FinderRet::Stop` to stop walking and return `None`.
    /// * `FinderRet::Continue` to continue walking up.
    ///
    /// This is a shortcut for `ctx.scoping.find_scope_by_flags`.
    pub fn find_scope_by_flags<F, O>(&self, finder: F) -> Option<O>
    where
        F: Fn(ScopeFlags) -> FinderRet<O>,
    {
        self.scoping.find_scope_by_flags(finder)
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
    #[allow(unsafe_code)]
    pub(crate) unsafe fn pop_stack(&mut self) {
        self.ancestry.pop_stack();
    }

    /// Shortcut for `self.ancestry.retag_stack`, to make `walk_*` methods less verbose.
    ///
    /// # SAFETY
    /// See safety constraints of `TraverseAncestry.retag_stack`.
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    #[inline]
    #[allow(unsafe_code, clippy::ptr_as_ptr, clippy::ref_as_ptr)]
    pub(crate) unsafe fn retag_stack(&mut self, ty: AncestorType) {
        self.ancestry.retag_stack(ty);
    }

    /// Shortcut for `ctx.scoping.set_current_scope_id`, to make `walk_*` methods less verbose.
    #[inline]
    pub(crate) fn set_current_scope_id(&mut self, scope_id: ScopeId) {
        self.scoping.set_current_scope_id(scope_id);
    }
}

// Public methods
impl<'a> TraverseAncestry<'a> {
    /// Get parent of current node.
    #[inline]
    #[allow(unsafe_code)]
    pub fn parent(&self) -> &Ancestor<'a> {
        // SAFETY: Stack contains 1 entry initially. Entries are pushed as traverse down the AST,
        // and popped as go back up. So even when visiting `Program`, the initial entry is in the stack.
        unsafe { self.stack.last().unwrap_unchecked() }
    }

    /// Get ancestor of current node.
    ///
    /// `level` is number of levels above.
    /// `ancestor(1).unwrap()` is equivalent to `parent()`.
    #[inline]
    pub fn ancestor(&self, level: usize) -> Option<&Ancestor<'a>> {
        self.stack.get(self.stack.len() - level)
    }

    /// Walk up trail of ancestors to find a node.
    ///
    /// `finder` should return:
    /// * `FinderRet::Found(value)` to stop walking and return `Some(value)`.
    /// * `FinderRet::Stop` to stop walking and return `None`.
    /// * `FinderRet::Continue` to continue walking up.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_ast::ast::ThisExpression;
    /// use oxc_traverse::{Ancestor, FinderRet, Traverse, TraverseCtx};
    ///
    /// struct MyTraverse;
    /// impl<'a> Traverse<'a> for MyTraverse {
    ///     fn enter_this_expression(&mut self, this_expr: &mut ThisExpression, ctx: &mut TraverseCtx<'a>) {
    ///         // Get name of function where `this` is bound.
    ///         // NB: This example doesn't handle `this` in class fields or static blocks.
    ///         let fn_id = ctx.ancestry.find_ancestor(|ancestor| {
    ///             match ancestor {
    ///                 Ancestor::FunctionBody(func) => FinderRet::Found(func.id()),
    ///                 Ancestor::FunctionParams(func) => FinderRet::Found(func.id()),
    ///                 _ => FinderRet::Continue
    ///             }
    ///         });
    ///     }
    /// }
    /// ```
    //
    // `'c` lifetime on `&'c self` and `&'c Ancestor` passed into the closure
    // allows an `Ancestor` or AST node to be returned from the closure.
    pub fn find_ancestor<'c, F, O>(&'c self, finder: F) -> Option<O>
    where
        F: Fn(&'c Ancestor<'a>) -> FinderRet<O>,
    {
        for ancestor in self.stack.iter().rev() {
            match finder(ancestor) {
                FinderRet::Found(res) => return Some(res),
                FinderRet::Stop => return None,
                FinderRet::Continue => {}
            }
        }
        None
    }

    /// Get depth in the AST.
    ///
    /// Count includes current node. i.e. in `Program`, depth is 1.
    #[inline]
    pub fn ancestors_depth(&self) -> usize {
        self.stack.len()
    }
}

// Methods used internally within crate.
impl<'a> TraverseAncestry<'a> {
    /// Create new `TraverseAncestry`.
    ///
    /// # SAFETY
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    fn new() -> Self {
        let mut stack = Vec::with_capacity(INITIAL_STACK_CAPACITY);
        stack.push(Ancestor::None);
        Self { stack }
    }

    /// Push item onto ancestry stack.
    ///
    /// # SAFETY
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    #[inline]
    pub(crate) fn push_stack(&mut self, ancestor: Ancestor<'a>) {
        self.stack.push(ancestor);
    }

    /// Pop last item off ancestry stack.
    ///
    /// # SAFETY
    /// * Stack must not be empty.
    /// * Each `pop_stack` call must correspond to a `push_stack` call for same type.
    ///
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    #[inline]
    #[allow(unsafe_code)]
    pub(crate) unsafe fn pop_stack(&mut self) {
        self.stack.pop().unwrap_unchecked();
    }

    /// Retag last item on ancestry stack.
    ///
    /// i.e. Alter discriminant of `Ancestor` enum, without changing the "payload" it contains
    /// of pointer to the ancestor node.
    ///
    /// This is purely a performance optimization. If the last item on stack already contains the
    /// correct pointer, then `ctx.retag_stack(AncestorType::ProgramBody)` is equivalent to:
    ///
    /// ```nocompile
    /// ctx.pop_stack();
    /// ctx.push_stack(Ancestor::ProgramBody(ProgramWithoutBody(node_ptr)));
    /// ```
    ///
    /// `retag_stack` is only a single 2-byte write operation.
    ///
    /// # SAFETY
    /// * Stack must not be empty.
    /// * Last item on stack must contain pointer to type corresponding to provided `AncestorType`.
    ///
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    #[inline]
    #[allow(unsafe_code, clippy::ptr_as_ptr, clippy::ref_as_ptr)]
    pub(crate) unsafe fn retag_stack(&mut self, ty: AncestorType) {
        *(self.stack.last_mut().unwrap_unchecked() as *mut _ as *mut AncestorType) = ty;
    }
}

// Public methods
impl TraverseScoping {
    /// Get current scope ID
    #[inline]
    pub fn current_scope_id(&self) -> ScopeId {
        self.current_scope_id
    }

    /// Get scopes tree
    #[inline]
    pub fn scopes(&self) -> &ScopeTree {
        &self.scopes
    }

    /// Get mutable scopes tree
    #[inline]
    pub fn scopes_mut(&mut self) -> &mut ScopeTree {
        &mut self.scopes
    }

    /// Get symbols table
    #[inline]
    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    /// Get mutable symbols table
    #[inline]
    pub fn symbols_mut(&mut self) -> &mut SymbolTable {
        &mut self.symbols
    }

    /// Walk up trail of scopes to find a scope.
    ///
    /// `finder` is called with `ScopeId`.
    ///
    /// `finder` should return:
    /// * `FinderRet::Found(value)` to stop walking and return `Some(value)`.
    /// * `FinderRet::Stop` to stop walking and return `None`.
    /// * `FinderRet::Continue` to continue walking up.
    pub fn find_scope<F, O>(&self, finder: F) -> Option<O>
    where
        F: Fn(ScopeId) -> FinderRet<O>,
    {
        let mut scope_id = self.current_scope_id;
        loop {
            match finder(scope_id) {
                FinderRet::Found(res) => return Some(res),
                FinderRet::Stop => return None,
                FinderRet::Continue => {}
            }

            if let Some(parent_scope_id) = self.scopes.get_parent_id(scope_id) {
                scope_id = parent_scope_id;
            } else {
                return None;
            }
        }
    }

    /// Walk up trail of scopes to find a scope by checking `ScopeFlags`.
    ///
    /// `finder` is called with `ScopeFlags`.
    ///
    /// `finder` should return:
    /// * `FinderRet::Found(value)` to stop walking and return `Some(value)`.
    /// * `FinderRet::Stop` to stop walking and return `None`.
    /// * `FinderRet::Continue` to continue walking up.
    pub fn find_scope_by_flags<F, O>(&self, finder: F) -> Option<O>
    where
        F: Fn(ScopeFlags) -> FinderRet<O>,
    {
        self.find_scope(|scope_id| {
            let flags = self.scopes.get_flags(scope_id);
            finder(flags)
        })
    }
}

// Methods used internally within crate
impl TraverseScoping {
    /// Create new `TraverseScoping`
    fn new(scopes: ScopeTree, symbols: SymbolTable) -> Self {
        Self {
            scopes,
            symbols,
            // Dummy value. Immediately overwritten in `walk_program`.
            current_scope_id: ScopeId::new(0),
        }
    }

    /// Set current scope ID
    #[inline]
    pub(crate) fn set_current_scope_id(&mut self, scope_id: ScopeId) {
        self.current_scope_id = scope_id;
    }
}
