use std::{mem, ptr::NonNull};

use crate::{Box, Vec};

/// A trait to replace an existing AST node in place with a new node built from the old one.
pub trait ReplaceWith: Sized {
    /// Replace the node in place with the value returned by `replacer`.
    ///
    /// `replacer` is called with an owned copy of the node, and whatever it returns is written back in its place.
    /// This is for the common case where a node is taken purely to build a new node out of it
    /// (typically wrapping the old node, or extracting just a part of it), which is then stored back in the same slot.
    ///
    /// Prefer this over [`take_in`] + writing the result back. That writes a dummy node into arena,
    /// which takes up space in arena forever, even though it's pointless - it's overwritten immediately.
    /// This method avoids the need for the dummy node so is faster, and takes less memory.
    ///
    /// ```ignore
    /// // If `expr` is a parenthesized expression `(inner)`, unwrap it to `inner` in place
    /// fn unwrap_parens<'a>(expr: &mut Expression<'a>) {
    ///     if matches!(expr, Expression::ParenthesizedExpression(_)) {
    ///         expr.replace_with(|expr| {
    ///             let Expression::ParenthesizedExpression(paren) = expr else { unreachable!() };
    ///             paren.unbox().expression
    ///         });
    ///     }
    /// }
    /// ```
    ///
    /// [`take_in`]: crate::TakeIn::take_in
    //
    // `#[inline]` so that compiler can elide reading fields of struct which are not used,
    // rather than pulling whole struct onto the stack
    #[inline]
    fn replace_with(&mut self, replacer: impl FnOnce(Self) -> Self) {
        // `replace_with` reads the old value out of `self` without leaving a valid value behind
        // (until `replacer` returns). If `replacer` panics, `*self` is left holding a bitwise copy
        // of the old value, which is also owned by `replacer`'s frame as `old`.
        //
        // For a `Drop` type that would be a double-drop on unwind - both new and old copies would be dropped.
        // So we restrict to non-`Drop` types, for which the duplicated value needs no destructor
        // and the bitwise copy left in `*self` is harmless.
        //
        // All AST types are non-`Drop`.
        const {
            assert!(!mem::needs_drop::<Self>(), "Cannot use `replace_with` on a `Drop` type");
        }

        let ptr = NonNull::from(self);

        // SAFETY:
        // * `ptr` is derived from a `&mut Self`, so is valid for reads and writes, aligned,
        //   and points to an initialized `Self`.
        // * `ptr.read()` makes an owned copy of the node.
        // * The bitwise copy left in `*ptr` is never read again before being overwritten by `ptr.write`
        //   (which does not drop the old contents - correct, as ownership moved into `old`).
        // * The duplicate is sound for the same reason `mem::replace` is - only one of the two copies
        //   is ever used as a value. `replacer` receives `old` by value and has no access to `*ptr`
        //   (this method takes `&mut self`, guaranteeing exclusive access) so it cannot observe
        //   the moved-from slot.
        // * `Self: !Drop` (asserted above) rules out a double-drop if `replacer` panics.
        unsafe {
            let old = ptr.read();
            ptr.write(replacer(old));
        }
    }
}

impl<T> ReplaceWith for Box<'_, T> {}

impl<T> ReplaceWith for Vec<'_, T> {}

#[cfg(test)]
mod test {
    use crate::{Allocator, Vec};

    use super::ReplaceWith;

    /// A non-`Drop` type holding a `&mut`, used to check that `replace_with` is sound when the
    /// value contains a live mutable reference (the bitwise-duplicated reference must never be used twice).
    struct RefHolder<'a>(&'a mut u64);

    impl ReplaceWith for RefHolder<'_> {}

    #[test]
    fn replace_with_wraps_old_value() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let mut vec = Vec::from_iter_in([1u32, 2, 3], &allocator);
        vec.replace_with(|old| {
            // The closure receives the old value, not a dummy.
            assert_eq!(&*old, &[1, 2, 3]);
            // Build a new value out of the old one (prepend `0`).
            Vec::from_iter_in(std::iter::once(0).chain(old), &allocator)
        });
        assert_eq!(&*vec, &[0, 1, 2, 3]);
    }

    #[test]
    fn replace_with_extracts_part_of_old_value() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let mut vec = Vec::from_iter_in([10u32, 20, 30], &allocator);
        // Keep just the first element of the old value.
        vec.replace_with(|old| {
            let first = old.into_iter().next().unwrap();
            Vec::from_iter_in([first], &allocator)
        });
        assert_eq!(&*vec, &[10]);
    }

    #[test]
    fn replace_with_identity_leaves_value_unchanged() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let mut vec = Vec::from_iter_in([7u32], &allocator);
        vec.replace_with(|old| old);
        assert_eq!(&*vec, &[7]);
    }

    #[test]
    fn replace_with_mutating_through_moved_out_ref() {
        let mut value = 10u64;
        let mut holder = RefHolder(&mut value);

        // Mutate through the moved-out reference, then put the same reference back.
        holder.replace_with(|old| {
            *old.0 += 1;
            old
        });
        // The reference left in the slot is still valid and points at `value`.
        *holder.0 += 100;

        assert_eq!(value, 111);
    }

    #[test]
    fn replace_with_swapping_in_a_different_ref() {
        let mut a = 1u64;
        let mut b = 2u64;

        let mut holder = RefHolder(&mut a);
        // Drop the old reference (to `a`) and store a reference to `b` instead.
        holder.replace_with(|_old| RefHolder(&mut b));
        *holder.0 += 40;

        assert_eq!(a, 1);
        assert_eq!(b, 42);
    }
}
