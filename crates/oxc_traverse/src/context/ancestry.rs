use crate::ancestor::{Ancestor, AncestorType};

const INITIAL_STACK_CAPACITY: usize = 64; // 64 entries = 1 KiB

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

// Public methods
impl<'a> TraverseAncestry<'a> {
    /// Get parent of current node.
    #[inline]

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

    /// Get iterator over ancestors, starting with closest ancestor
    pub fn ancestors<'b>(&'b self) -> impl Iterator<Item = &'b Ancestor<'a>> {
        self.stack.iter().rev()
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
    pub(super) fn new() -> Self {
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
