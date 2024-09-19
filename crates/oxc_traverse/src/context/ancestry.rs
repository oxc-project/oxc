use std::mem::transmute;

use crate::ancestor::{Ancestor, AncestorType};

const INITIAL_STACK_CAPACITY: usize = 64; // 64 entries = 1 KiB

/// Traverse ancestry context.
///
/// Contains a stack of `Ancestor`s, and provides methods to get parent/ancestor of current node.
///
/// `walk_*` methods push/pop `Ancestor`s to `stack` when entering/exiting nodes.
///
/// `Ancestor<'a, 't>` is an owned type.
/// * `'a` is lifetime of AST nodes.
/// * `'t` is lifetime of the `Ancestor` (derived from `&'t TraverseAncestry`).
///
/// `'t` is constrained in `parent`, `ancestor` and `ancestors` methods to only live as long as
/// the `&'t TraverseAncestry` passed to the method.
/// i.e. `Ancestor`s can only live as long as `enter_*` or `exit_*` method in which they're obtained,
/// and cannot "escape" those methods.
/// This is required for soundness. If an `Ancestor` could be retained longer, the references that
/// can be got from it could alias a `&mut` reference to the same AST node.
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
    stack: Vec<Ancestor<'a, 'static>>,
}

// Public methods
impl<'a> TraverseAncestry<'a> {
    /// Get parent of current node.
    #[inline]
    pub fn parent<'t>(&'t self) -> Ancestor<'a, 't> {
        debug_assert!(!self.stack.is_empty());
        // SAFETY: Stack contains 1 entry initially. Entries are pushed as traverse down the AST,
        // and popped as go back up. So even when visiting `Program`, the initial entry is in the stack.
        let ancestor = unsafe { *self.stack.last().unwrap_unchecked() };
        // Shrink `Ancestor`'s `'t` lifetime to lifetime of `&'t self`.
        // SAFETY: The `Ancestor` is guaranteed valid for `'t`. It is not possible to obtain
        // a `&mut` ref to any AST node which this `Ancestor` gives access to during `'t`.
        unsafe { transmute::<Ancestor<'a, '_>, Ancestor<'a, 't>>(ancestor) }
    }

    /// Get ancestor of current node.
    ///
    /// `level` is number of levels above parent.
    /// `ancestor(0)` is equivalent to `parent()` (but better to use `parent()` as it's more efficient).
    ///
    /// If `level` is out of bounds (above `Program`), returns `Ancestor::None`.
    #[inline]
    pub fn ancestor<'t>(&'t self, level: usize) -> Ancestor<'a, 't> {
        // Behavior with different values:
        // `len = 1, level = 0` -> return `Ancestor::None` from else branch
        // `len = 1, level = 1` -> return `Ancestor::None` from else branch (out of bounds)
        // `len = 3, level = 0` -> return parent (index 2)
        // `len = 3, level = 1` -> return grandparent (index 1)
        // `len = 3, level = 2` -> return `Ancestor::None` from else branch
        // `len = 3, level = 3` -> return `Ancestor::None` from else branch (out of bounds)

        // `self.stack.len()` is always at least 1, so `self.stack.len() - 1` cannot wrap around.
        // `level <= last_index` would also work here, but `level < last_index` avoids a read from memory
        // when that read would just get `Ancestor::None` anyway.
        debug_assert!(!self.stack.is_empty());
        let last_index = self.stack.len() - 1;
        if level < last_index {
            // SAFETY: We just checked that `level < last_index` so `last_index - level` cannot wrap around,
            // and `last_index - level` must be a valid index
            let ancestor = unsafe { *self.stack.get_unchecked(last_index - level) };

            // Shrink `Ancestor`'s `'t` lifetime to lifetime of `&'t self`.
            // SAFETY: The `Ancestor` is guaranteed valid for `'t`. It is not possible to obtain
            // a `&mut` ref to any AST node which this `Ancestor` gives access to during `'t`.
            unsafe { transmute::<Ancestor<'a, '_>, Ancestor<'a, 't>>(ancestor) }
        } else {
            Ancestor::None
        }
    }

    /// Get iterator over ancestors, starting with parent and working up.
    ///
    /// Last `Ancestor` returned will be `Program`. `Ancestor::None` is not included in iteration.
    pub fn ancestors<'t>(&'t self) -> impl Iterator<Item = Ancestor<'a, 't>> {
        debug_assert!(!self.stack.is_empty());
        // SAFETY: Stack always has at least 1 entry
        let stack_without_first = unsafe { self.stack.get_unchecked(1..) };
        stack_without_first.iter().rev().map(|&ancestor| {
            // Shrink `Ancestor`'s `'t` lifetime to lifetime of `&'t self`.
            // SAFETY: The `Ancestor` is guaranteed valid for `'t`. It is not possible to obtain
            // a `&mut` ref to any AST node which this `Ancestor` gives access to during `'t`.
            unsafe { transmute::<Ancestor<'a, '_>, Ancestor<'a, 't>>(ancestor) }
        })
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
    pub(crate) fn push_stack(&mut self, ancestor: Ancestor<'a, 'static>) -> PopToken {
        self.stack.push(ancestor);

        // Return `PopToken` which can be used to pop this entry off again
        PopToken(())
    }

    /// Pop last item off ancestry stack.
    ///
    /// # SAFETY
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    #[inline]
    #[allow(unused_variables, clippy::needless_pass_by_value)]
    pub(crate) fn pop_stack(&mut self, token: PopToken) {
        debug_assert!(self.stack.len() >= 2);
        // SAFETY: `PopToken`s are only created in `push_stack`, so the fact that caller provides one
        // guarantees that a push has happened. This method consumes the token which guarantees another
        // pop hasn't occurred already corresponding to that push.
        // Therefore the stack cannot by empty.
        // The stack starts with 1 entry, so also it cannot be left empty after this pop.
        unsafe { self.stack.pop().unwrap_unchecked() };
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
    /// * Stack must have length of at least 2 (so we are not retagging dummy root `Ancestor`).
    /// * Last item on stack must contain pointer to type corresponding to provided `AncestorType`.
    ///
    /// This method must not be public outside this crate, or consumer could break safety invariants.
    #[inline]
    #[allow(unsafe_code, clippy::ptr_as_ptr, clippy::ref_as_ptr)]
    pub(crate) unsafe fn retag_stack(&mut self, ty: AncestorType) {
        debug_assert!(self.stack.len() >= 2);
        *(self.stack.last_mut().unwrap_unchecked() as *mut _ as *mut AncestorType) = ty;
    }
}

/// Zero sized token which allows popping from stack. Used to ensure push and pop always correspond.
/// Inner field is private to this module so can only be created by methods in this file.
/// It is not `Clone` or `Copy`, so no way to obtain one except in this file.
/// Only method which generates a `PopToken` is `push_stack`, and `pop_stack` consumes one,
/// which guarantees you can't have more pops than pushes.
pub(crate) struct PopToken(());
