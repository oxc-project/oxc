use oxc_allocator::{Allocator, Box};
use oxc_ast::AstBuilder;
use oxc_syntax::scope::ScopeFlags;

use crate::ancestor::{Ancestor, AncestorType};

const INITIAL_STACK_CAPACITY: usize = 64; // 64 entries = 1 KiB
const INITIAL_SCOPE_STACK_CAPACITY: usize = 32; // 32 entries = 64 bytes

/// Traverse context.
///
/// Passed to all AST visitor functions.
///
/// Provides ability to:
/// * Query parent/ancestor of current node via [`parent`], [`ancestor`], [`find_ancestor`].
/// * Get type of current scope via [`scope`], [`ancestor_scope`], [`find_scope`].
/// * Create AST nodes via AST builder [`ast`].
/// * Allocate into arena via [`alloc`].
///
/// [`parent`]: `TraverseCtx::parent`
/// [`ancestor`]: `TraverseCtx::ancestor`
/// [`find_ancestor`]: `TraverseCtx::find_ancestor`
/// [`scope`]: `TraverseCtx::scope`
/// [`ancestor_scope`]: `TraverseCtx::ancestor_scope`
/// [`find_scope`]: `TraverseCtx::find_scope`
/// [`ast`]: `TraverseCtx::ast`
/// [`alloc`]: `TraverseCtx::alloc`
pub struct TraverseCtx<'a> {
    stack: Vec<Ancestor<'a>>,
    scope_stack: Vec<ScopeFlags>,
    pub ast: AstBuilder<'a>,
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
    pub(crate) fn new(allocator: &'a Allocator) -> Self {
        let mut stack = Vec::with_capacity(INITIAL_STACK_CAPACITY);
        stack.push(Ancestor::None);

        let mut scope_stack = Vec::with_capacity(INITIAL_SCOPE_STACK_CAPACITY);
        scope_stack.push(ScopeFlags::empty());

        Self { stack, scope_stack, ast: AstBuilder::new(allocator) }
    }

    /// Allocate a node in the arena.
    /// Returns a [`Box<T>`].
    #[inline]
    pub fn alloc<T>(&self, node: T) -> Box<'a, T> {
        self.ast.alloc(node)
    }

    /// Get parent of current node.
    #[inline]
    #[allow(unsafe_code)]
    pub fn parent(&self) -> &Ancestor<'a> {
        // SAFETY: Stack contains 1 entry initially. Entries are pushed as traverse down the AST,
        // and popped as go back up. So even when visiting `Program`, the initial entry is in the stack.
        unsafe { self.stack.last().unwrap_unchecked() }
    }

    /// Get ancestor of current node.
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
    ///     fn enter_this_expression(&mut self, this_expr: &mut ThisExpression, ctx: &TraverseCtx<'a>) {
    ///         // Get name of function where `this` is bound
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

    /// Get current scope info.
    #[inline]
    #[allow(unsafe_code)]
    pub fn scope(&self) -> ScopeFlags {
        // SAFETY: Scope stack contains 1 entry initially. Entries are pushed as traverse down the AST,
        // and popped as go back up. So even when visiting `Program`, the initial entry is in the stack.
        unsafe { *self.scope_stack.last().unwrap_unchecked() }
    }

    /// Get scope ancestor.
    /// `level` is number of scopes above.
    /// `ancestor_scope(1).unwrap()` is equivalent to `scope()`.
    #[inline]
    pub fn ancestor_scope(&self, level: usize) -> Option<ScopeFlags> {
        self.scope_stack.get(self.stack.len() - level).copied()
    }

    /// Walk up trail of scopes to find a scope.
    ///
    /// `finder` should return:
    /// * `FinderRet::Found(value)` to stop walking and return `Some(value)`.
    /// * `FinderRet::Stop` to stop walking and return `None`.
    /// * `FinderRet::Continue` to continue walking up.
    pub fn find_scope<F, O>(&self, finder: F) -> Option<O>
    where
        F: Fn(ScopeFlags) -> FinderRet<O>,
    {
        for flags in self.scope_stack.iter().rev().copied() {
            match finder(flags) {
                FinderRet::Found(res) => return Some(res),
                FinderRet::Stop => return None,
                FinderRet::Continue => {}
            }
        }
        None
    }

    /// Get depth of scopes.
    ///
    /// Count includes global scope.
    /// i.e. in `Program`, depth is 2 (global scope + program top level scope).
    #[inline]
    pub fn scopes_depth(&self) -> usize {
        self.scope_stack.len()
    }
}

// Methods used internally within crate
impl<'a> TraverseCtx<'a> {
    /// Push item onto stack.
    #[inline]
    pub(crate) fn push_stack(&mut self, ancestor: Ancestor<'a>) {
        self.stack.push(ancestor);
    }

    /// Pop last item off stack.
    /// # SAFETY
    /// * Stack must not be empty.
    /// * Each `pop_stack` call must correspond to a `push_stack` call for same type.
    #[inline]
    #[allow(unsafe_code)]
    pub(crate) unsafe fn pop_stack(&mut self) {
        self.stack.pop().unwrap_unchecked();
    }

    /// Retag last item on stack.
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
    #[inline]
    #[allow(unsafe_code, clippy::ptr_as_ptr, clippy::ref_as_ptr)]
    pub(crate) unsafe fn retag_stack(&mut self, ty: AncestorType) {
        *(self.stack.last_mut().unwrap_unchecked() as *mut _ as *mut AncestorType) = ty;
    }

    /// Push scope flags onto scope stack.
    ///
    /// `StrictMode` flag is inherited from parent.
    #[inline]
    pub(crate) fn push_scope_stack(&mut self, flags: ScopeFlags) {
        self.scope_stack.push(flags | (self.scope() & ScopeFlags::StrictMode));
    }

    /// Pop last item off scope stack.
    /// # SAFETY
    /// * Stack must not be empty.
    /// * Each `pop_scope_stack` call must correspond to an earlier `push_scope_stack` call.
    #[inline]
    #[allow(unsafe_code)]
    pub(crate) unsafe fn pop_scope_stack(&mut self) {
        self.scope_stack.pop().unwrap_unchecked();
    }
}
