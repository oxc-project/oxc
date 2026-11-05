use std::{
    boxed::Box as StdBox, cell::Cell, marker::PhantomData, mem, ptr::NonNull, sync::Mutex,
    vec::Vec as StdVec,
};

use crate::{Allocator, Box, Dummy, Vec};

/// A trait to replace an existing AST node in place with a new node built from the old one.
pub trait ReplaceWith<'a>: Dummy<'a> {
    /// Replace the node in place with the value returned by `replacer`.
    ///
    /// `replacer` is called with an owned copy of the node, and whatever it returns is written back in its place.
    /// This is for the common case where a node is taken purely to build a new node out of it
    /// (typically wrapping the old node, or extracting just a part of it), which is then stored back in the same slot.
    ///
    /// Prefer this over [`take_in`] + writing the result back. That writes a dummy node into arena,
    /// which takes up space in arena forever, even though it's pointless - it's overwritten immediately.
    /// This method avoids the need for the dummy node, so is faster, and takes less memory.
    ///
    /// ```
    /// # use oxc_allocator::{Allocator, Box, Dummy};
    /// # enum Expression<'a> {
    /// #     ParenthesizedExpression(Box<'a, ParenthesizedExpression<'a>>),
    /// #     Empty,
    /// # }
    /// # struct ParenthesizedExpression<'a> {
    /// #     expression: Expression<'a>,
    /// # }
    /// # impl<'a> Dummy<'a> for Expression<'a> {
    /// #     fn dummy(allocator: &'a Allocator) -> Self {
    /// #         Self::Empty
    /// #     }
    /// # }
    /// # impl<'a> ReplaceWith<'a> for Expression<'a> {}
    /// use oxc_allocator::ReplaceWith;
    ///
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
    /// # Panic handling
    ///
    /// If `replacer` panics, the original place is left holding a dummy (see [`Dummy`]), so it always
    /// remains valid and usable. Building that dummy is the only situation in which this method allocates,
    /// so the common (non-panicking) path allocates nothing.
    ///
    /// When compiled with `panic = "abort"`, the code for writing the dummy is optimized out entirely.
    ///
    /// Handling panics in this way is only required to avoid UB in obscure cases where `catch_unwind`
    /// is used and the original value is accessed after the panic.
    ///
    /// Without the panic handling, these would both be Undefined Behavior:
    ///
    /// #### Double drop
    ///
    /// ```
    /// use std::{
    ///     boxed::Box as StdBox,
    ///     mem::ManuallyDrop,
    ///     panic::{AssertUnwindSafe, catch_unwind},
    /// };
    /// use oxc_allocator::{Allocator, Dummy, ReplaceWith};
    ///
    /// // `ManuallyDrop` wrapper is not required here, but it demonstrates that
    /// // a const assertion in `replace_with` that `Self: !Drop` would not be
    /// // sufficient to prevent UB - `Wrapper` is `!Drop`.
    /// struct Wrapper(ManuallyDrop<StdBox<u32>>);
    ///
    /// impl ReplaceWith<'_> for Wrapper {}
    ///
    /// impl<'a> Dummy<'a> for Wrapper {
    ///     fn dummy(_allocator: &'a Allocator) -> Self {
    ///         Self(ManuallyDrop::new(StdBox::new(0)))
    ///     }
    /// }
    ///
    /// let mut wrapper = Wrapper(ManuallyDrop::new(StdBox::new(123)));
    ///
    /// let _ = catch_unwind(AssertUnwindSafe(|| {
    ///     wrapper.replace_with(|old| {
    ///         let boxed: StdBox<u32> = ManuallyDrop::into_inner(old.0);
    ///         // `Box` is freed
    ///         drop(boxed);
    ///         panic!("Unwind before write-back");
    ///     });
    /// }));
    ///
    /// let boxed: StdBox<u32> = ManuallyDrop::into_inner(wrapper.0);
    /// // Without the panic guard, `boxed` would be the same `Box`
    /// // that was freed in closure above. Double drop!
    /// drop(boxed);
    /// ```
    ///
    /// #### Aliasing violation
    ///
    /// ```
    /// use std::panic::{AssertUnwindSafe, catch_unwind};
    /// use oxc_allocator::{Allocator, ArenaBox, Dummy, ReplaceWith};
    ///
    /// struct Wrapper<'a>(ArenaBox<'a, u32>);
    ///
    /// impl<'a> ReplaceWith<'a> for Wrapper<'a> {}
    ///
    /// impl<'a> Dummy<'a> for Wrapper<'a> {
    ///     fn dummy(allocator: &'a Allocator) -> Self {
    ///         Self(ArenaBox::new_in(0, &allocator))
    ///     }
    /// }
    ///
    /// let allocator = Allocator::new();
    /// let allocator = &allocator;
    ///
    /// let mut wrapper = Wrapper(ArenaBox::new_in(1, &allocator));
    /// let mut copy: Option<Wrapper> = None;
    ///
    /// let _ = catch_unwind(AssertUnwindSafe(|| {
    ///     wrapper.replace_with(|old| {
    ///         // Move the copy out of the closure
    ///         copy = Some(old);
    ///         panic!("Unwind before write-back");
    ///     });
    /// }));
    ///
    /// let mut boxed: ArenaBox<u32> = wrapper.0;
    /// let mut copy: ArenaBox<u32> = copy.unwrap().0;
    /// let boxed_mut: &mut u32 = boxed.as_mut();
    /// let copy_mut: &mut u32 = copy.as_mut();
    /// // Without the panic guard, `boxed` and `copy` would both point to same `u32`.
    /// // We'd now have 2 `&mut u32`s pointing to same place. Aliasing violation!
    /// *boxed_mut = 2;
    /// *copy_mut = 3;
    /// ```
    ///
    /// [`take_in`]: crate::TakeIn::take_in
    //
    // `#[inline]` so that compiler can elide reading fields of struct which are not used,
    // rather than pulling whole struct onto the stack
    #[inline]
    fn replace_with(&mut self, replacer: impl FnOnce(Self) -> Self) {
        let ptr = NonNull::from(self);

        // SAFETY: `ptr` is derived from a `&mut Self`, so is valid for reads and writes, aligned,
        // and points to an initialized `Self`. `ptr.read()` makes an owned bitwise copy of the node.
        //
        // This leaves a bitwise duplicate in `*ptr`, but it is never dropped or exposed as a value:
        //
        // * On success, `ptr.write(new)` below overwrites it (`write` does not drop the old contents -
        //   correct, as ownership moved into `old`, and thence into `new`).
        // * On panic, `PanicGuard::drop` overwrites it with a fresh dummy (again without dropping it),
        //   before it can be accessed by any other code.
        //
        // So `*ptr` never retains a duplicate that could be observed alongside `old` (which would allow
        // aliasing `&mut`s, double frees, etc), and no value is ever dropped twice.
        // This holds for any `Self: Dummy` - `Drop` types included.
        let old = unsafe { ptr.read() };

        // Create a guard before calling `replacer`.
        // If `replacer` unwinds, `guard`'s `Drop` writes a fresh dummy into `*ptr`,
        // so the slot never keeps a bitwise duplicate of the value moved into `replacer`.
        let guard = PanicGuard::<'a, Self> { ptr, marker: PhantomData };

        let new = replacer(old);

        // `replacer` returned normally. Disarm the guard - we write `new` ourselves below.
        mem::forget(guard);

        // Overwrite original value without dropping it.
        // SAFETY: See above.
        unsafe { ptr.write(new) };
    }
}

// Blanket impls on wrapper types.
//
// `ReplaceWith` is not implemented on all types which implement `Dummy`, because there is no point
// in using `replace_with` on `Copy` types - just copying these types is cheaper and simpler.
//
// Similarly, bound of `T: ReplaceWith` on `Option<T>` is to avoid implementing `ReplaceWith`
// on `Option<T>` where `T: Copy`, and therefore `Option<T>` is `Copy` too.
//
// `Cell<T>` is not `Copy` even if `T` is, but `Cell::get` is available where `T: Copy`,
// and is a better choice than `replace_with` in this case. So it's also bound on `T: ReplaceWith`.

impl<'a, T: ReplaceWith<'a>> ReplaceWith<'a> for Option<T> {}

impl<'a, T: Dummy<'a>> ReplaceWith<'a> for Box<'a, T> {}
impl<'a, T> ReplaceWith<'a> for Box<'a, [T]> {}

impl<'a, T> ReplaceWith<'a> for Vec<'a, T> {}

impl<'a, T: ReplaceWith<'a>> ReplaceWith<'a> for Cell<T> {}

/// Guard which restores a valid value to a slot if [`replace_with`]'s `replacer` panics.
///
/// `replace_with` reads the old node out of the slot with `NonNull::read`, leaving a bitwise duplicate behind,
/// then calls `replacer`. If `replacer` panics, this guard's `Drop` overwrites that duplicate with a fresh dummy,
/// so the slot cannot be observed holding a copy that aliases the value which was moved into `replacer`.
/// On the success path the guard is `mem::forget`-ed, so it costs nothing.
///
/// [`replace_with`]: ReplaceWith::replace_with
struct PanicGuard<'a, T: Dummy<'a>> {
    ptr: NonNull<T>,
    marker: PhantomData<&'a ()>,
}

impl<'a, T: Dummy<'a>> Drop for PanicGuard<'a, T> {
    // A trivial forwarder to `write_dummy`, which is `#[inline(never)]` and does the real work.
    //
    // Keeping `drop` itself `#[inline]` lets it inline into the caller's unwind landing pad,
    // where it reads `self.ptr` straight from a register and passes it *by value* to `write_dummy`.
    // So the guard never has to be spilled to the stack - which it would be if `drop` were
    // the out-of-line function, because then the landing pad would have to form `&guard`
    // in memory to call it.
    //
    // `write_dummy` being `#[inline(never)]` still keeps the expensive work out of every `#[inline]`d
    // `replace_with` call site's landing pad.
    #[inline]
    fn drop(&mut self) {
        write_dummy(self.ptr);
    }
}

/// Overwrite the bitwise duplicate left in `*ptr` (by `replace_with`) with a fresh dummy.
///
/// This is the cold panic-path body of `PanicGuard::drop`, factored out as a free function that takes
/// `ptr` *by value* so it can be `#[inline(never)]` while `PanicGuard::drop` stays an inlinable
/// forwarder (see the comment there for why that matters).
#[cold]
#[inline(never)]
fn write_dummy<'a, T: Dummy<'a>>(ptr: NonNull<T>) {
    // `replacer` panicked. Build a fresh dummy in its own dedicated `'static` allocator.
    let allocator = fresh_dummy_allocator();
    let dummy = T::dummy(allocator);

    // Write the dummy into `*ptr`.
    // SAFETY: `ptr` was derived from a `&mut Self` in `replace_with` and remains valid.
    // `write` overwrites the leftover duplicate without dropping it - correct, as that value's
    // ownership was moved into `replacer`.
    unsafe { ptr.write(dummy) };
}

/// Storage for the [`Allocator`]s that dummies are created in on the panic path of [`replace_with`].
///
/// A dummy written into a node's slot on panic must be a valid value for `'a`,
/// which means allocating it in an [`Allocator`] that lives for at least `'a`.
///
/// `replace_with` can't reuse the AST's own allocator, because that isn't available there,
/// and it can't share a single global allocator, because the dummy may retain a handle to whatever allocator
/// it was built in (e.g. `Vec` contains a reference to the `Allocator`, and reallocates through it on `push`).
/// A shared allocator would then be mutated without synchronization from whichever thread owns the AST.
///
/// We also can't have 1 `Allocator` per thread (in a `thread_local!`) because they don't live for `'static` -
/// they're dropped when the thread exits. Some code moves ASTs to other threads (using `self_cell`)
/// so there's no guarantee the thread would live long enough, and the synchronization problem discussed above
/// could also come into play.
///
/// So each dummy gets its *own* dedicated [`Allocator`]. These allocators must:
///
/// 1. live for `'static` (the dummy escapes into the AST, whose lifetime we can't bound here), and
/// 2. have stable addresses (a `Vec` in the dummy holds an `&Allocator` pointing to one).
///
/// We get both by boxing each `Allocator` and keeping the boxes alive forever in this `static`.
/// The `Box` gives a stable heap address, and `Vec` growth only moves the `Box`es themselves,
/// not the *contents* of the `Box`es (the `Allocator`s).
///
/// This is only ever touched on the panic path - i.e. when there is a bug in a `replacer` closure.
/// So the `Mutex` is effectively never contended.
///
/// If the panic is caught (with `catch_unwind`) and the process kept alive, this leaks memory,
/// because `Allocator`s are added to the `Vec`, but never removed, never freed.
/// But panics should not be happening at all (and if they do, that's a bug and we'll fix it),
/// so this is not a problem in practice.
///
/// No memory is leaked unless there's a panic in a `replacer` closure.
///
/// [`replace_with`]: ReplaceWith::replace_with
#[expect(clippy::vec_box)]
static DUMMY_ALLOCATORS: Mutex<StdVec<StdBox<Allocator>>> = Mutex::new(StdVec::new());

/// Get a fresh [`Allocator`], dedicated to a single dummy, which lives for `'static`.
///
/// See [`DUMMY_ALLOCATORS`] for why each dummy needs its own `'static` allocator.
fn fresh_dummy_allocator() -> &'static Allocator {
    // `unwrap` can never fire here. `fresh_dummy_allocator` is only ever called from `PanicGuard::drop`,
    // i.e. while unwinding from a `replacer` panic.
    // The one operation below that could panic - `push` on capacity overflow - would therefore panic
    // during an existing unwind, which is a double panic that aborts the process immediately.
    // So the mutex can never be observed poisoned - anything that could poison it aborts first.
    // Capacity overflow is unreachable anyway - the `Vec`'s reallocation OOMs and aborts long before
    // ~`isize::MAX` entries.
    let mut allocators = DUMMY_ALLOCATORS.lock().unwrap();

    // Create an `Allocator` and store it in a `Box` in `DUMMY_ALLOCATORS`.
    // Get reference to the `Allocator` on the heap.
    let allocator = &**allocators.push_mut(StdBox::new(Allocator::new()));

    // Extend `&Allocator`'s lifetime to `'static`.
    // SAFETY: The `Allocator` is owned by `DUMMY_ALLOCATORS`, a `static` from which allocators
    // are never removed, so it lives for `'static`. Growing the `Vec` later would move the `Box`
    // but never moves its *contents* (the `Allocator` itself), so this reference remains valid.
    unsafe { NonNull::from(allocator).as_ref() }
}

#[cfg(test)]
mod test {
    use std::{
        iter,
        panic::{AssertUnwindSafe, catch_unwind},
    };

    use crate::{Allocator, Dummy, Vec};

    use super::ReplaceWith;

    #[test]
    fn replace_with_wraps_old_value() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let mut vec = Vec::from_iter_in([1u32, 2, 3], &allocator);
        vec.replace_with(|old| {
            // The closure receives the old value, not a dummy.
            assert_eq!(&*old, &[1, 2, 3]);
            // Build a new value out of the old one (prepend `0`).
            Vec::from_iter_in(iter::once(0).chain(old), &allocator)
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
    fn replace_with_writes_dummy_on_panic() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let mut vec = Vec::from_iter_in([1u32, 2, 3], &allocator);
        let result = catch_unwind(AssertUnwindSafe(|| {
            vec.replace_with(|_old| panic!("boom"));
        }));
        assert!(result.is_err());

        // The slot holds a fresh empty dummy - *not* a bitwise duplicate of the old value.
        assert_eq!(&*vec, &[] as &[u32]);
        // The dummy is fully usable: it has its own dedicated `'static` allocator, so it can grow.
        vec.push(42);
        assert_eq!(&*vec, &[42]);
    }

    /// A type holding a `&mut`. Duplicating a `&mut` is the classic way to make `replace_with` unsound.
    /// This checks the panic guard prevents that - the guard replaces the slot with a dummy holding a *fresh* `&mut`,
    /// so it never aliases the moved-out value.
    struct RefHolder<'a>(&'a mut u64);

    impl<'a> Dummy<'a> for RefHolder<'a> {
        fn dummy(allocator: &'a Allocator) -> Self {
            RefHolder(allocator.alloc(0u64))
        }
    }

    impl<'a> ReplaceWith<'a> for RefHolder<'a> {}

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
