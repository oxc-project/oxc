//! Arena Box.
//!
//! Originally based on [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/24004745a8ed4939fc0dc7332bfd1268ac52285f/crates/ast/src/arena.rs)

use std::{
    self,
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{self, Deref},
    ptr::{self, NonNull},
};

#[cfg(any(feature = "serialize", test))]
use serde::{Serialize, Serializer};

use crate::Allocator;

/// A `Box` without [`Drop`], which stores its data in the arena allocator.
///
/// # No `Drop`s
///
/// Objects allocated into Oxc memory arenas are never [`Dropped`](Drop). Memory is released in bulk
/// when the allocator is dropped, without dropping the individual objects in the arena.
///
/// Therefore, it would produce a memory leak if you allocated [`Drop`] types into the arena
/// which own memory allocations outside the arena.
///
/// Static checks make this impossible to do. [`Box::new_in`] will refuse to compile if called
/// with a [`Drop`] type.
pub struct Box<'alloc, T: ?Sized>(NonNull<T>, PhantomData<(&'alloc (), T)>);

impl<T: ?Sized> Box<'_, T> {
    /// Const assertion that `T` is not `Drop`.
    /// Must be referenced in all methods which create a `Box`.
    const ASSERT_T_IS_NOT_DROP: () =
        assert!(!std::mem::needs_drop::<T>(), "Cannot create a Box<T> where T is a Drop type");
}

impl<T> Box<'_, T> {
    /// Put a `value` into a memory arena and get back a [`Box`] with ownership
    /// to the allocation.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::{Allocator, Box};
    ///
    /// let arena = Allocator::default();
    /// let in_arena: Box<i32> = Box::new_in(5, &arena);
    /// ```
    //
    // `#[inline(always)]` because this is a hot path and `Allocator::alloc` is a very small function.
    // We always want it to be inlined.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn new_in(value: T, allocator: &Allocator) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        Self(NonNull::from(allocator.alloc(value)), PhantomData)
    }

    /// Create a fake [`Box`] with a dangling pointer.
    ///
    /// # SAFETY
    /// Safe to create, but must never be dereferenced, as does not point to a valid `T`.
    /// Only purpose is for mocking types without allocating for const assertions.
    #[allow(unsafe_code, clippy::missing_safety_doc)]
    pub const unsafe fn dangling() -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        Self(NonNull::dangling(), PhantomData)
    }

    /// Take ownership of the value stored in this [`Box`], consuming the box in
    /// the process.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::{Allocator, Box};
    ///
    /// let arena = Allocator::default();
    ///
    /// // Put `5` into the arena and on the heap.
    /// let boxed: Box<i32> = Box::new_in(5, &arena);
    /// // Move it back to the stack. `boxed` has been consumed.
    /// let i = boxed.unbox();
    ///
    /// assert_eq!(i, 5);
    /// ```
    #[inline]
    pub fn unbox(self) -> T {
        // SAFETY:
        // This pointer read is safe because the reference `self.0` is
        // guaranteed to be unique - not just now, but we're guaranteed it's not
        // borrowed from some other reference. This in turn is because we never
        // construct a `Box` with a borrowed reference, only with a fresh
        // one just allocated from a `Bump`.
        unsafe { ptr::read(self.0.as_ptr()) }
    }
}

impl<T: ?Sized> Box<'_, T> {
    /// Create a [`Box`] from a raw pointer to a value.
    ///
    /// The [`Box`] takes ownership of the data pointed to by `ptr`.
    ///
    /// # SAFETY
    /// Data pointed to by `ptr` must live as long as `'alloc`.
    /// This requirement is met if the pointer was obtained from other data in the arena.
    ///
    /// Data pointed to by `ptr` must *only* be used for this `Box`. i.e. it must be unique,
    /// with no other aliases. You must not, for example, create 2 `Box`es from the same pointer.
    ///
    /// `ptr` must have been created from a `*mut T` or `&mut T` (not a `*const T` / `&T`).
    //
    // `#[inline(always)]` because this is a no-op
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(crate) const unsafe fn from_non_null(ptr: NonNull<T>) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        Self(ptr, PhantomData)
    }

    /// Consume a [`Box`] and return a [`NonNull`] pointer to its contents.
    //
    // `#[inline(always)]` because this is a no-op
    #[expect(clippy::inline_always, clippy::needless_pass_by_value)]
    #[inline(always)]
    pub fn into_non_null(boxed: Self) -> NonNull<T> {
        boxed.0
    }
}

impl<T: ?Sized> ops::Deref for Box<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: self.0 is always a unique reference allocated from a Bump in Box::new_in
        unsafe { self.0.as_ref() }
    }
}

impl<T: ?Sized> ops::DerefMut for Box<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: self.0 is always a unique reference allocated from a Bump in Box::new_in
        unsafe { self.0.as_mut() }
    }
}

impl<T: ?Sized> AsRef<T> for Box<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T: ?Sized> AsMut<T> for Box<'_, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<T: ?Sized + Debug> Debug for Box<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

// Unused right now.
// impl<'alloc, T> PartialEq for Box<'alloc, T>
// where
// T: PartialEq<T> + ?Sized,
// {
// fn eq(&self, other: &Box<'alloc, T>) -> bool {
// PartialEq::eq(&**self, &**other)
// }
// }

#[cfg(any(feature = "serialize", test))]
impl<T> Serialize for Box<'_, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.deref().serialize(s)
    }
}

impl<T: Hash> Hash for Box<'_, T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

#[cfg(test)]
mod test {
    use std::hash::{DefaultHasher, Hash, Hasher};

    use super::Box;
    use crate::Allocator;

    #[test]
    fn box_deref_mut() {
        let allocator = Allocator::default();
        let mut b = Box::new_in("x", &allocator);
        let b = &mut *b;
        *b = allocator.alloc("v");
        assert_eq!(*b, "v");
    }

    #[test]
    fn box_debug() {
        let allocator = Allocator::default();
        let b = Box::new_in("x", &allocator);
        let b = format!("{b:?}");
        assert_eq!(b, "\"x\"");
    }

    #[test]
    fn box_hash() {
        fn hash(val: &impl Hash) -> u64 {
            let mut hasher = DefaultHasher::default();
            val.hash(&mut hasher);
            hasher.finish()
        }

        let allocator = Allocator::default();
        let a = Box::new_in("x", &allocator);
        let b = Box::new_in("x", &allocator);

        assert_eq!(hash(&a), hash(&b));
    }

    #[test]
    fn box_serialize() {
        let allocator = Allocator::default();
        let b = Box::new_in("x", &allocator);
        let b = serde_json::to_string(&b).unwrap();
        assert_eq!(b, "\"x\"");
    }

    #[test]
    fn lifetime_variance() {
        fn _assert_box_variant_lifetime<'a: 'b, 'b, T>(program: Box<'a, T>) -> Box<'b, T> {
            program
        }
    }
}
