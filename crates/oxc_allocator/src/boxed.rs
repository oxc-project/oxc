//! Arena Box.
//!
//! Originally based on [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/crates/ast/src/arena.rs)

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
/// Should only be used for storing AST types.
///
/// Must NOT be used to store types which have a [`Drop`] implementation.
/// `T::drop` will NOT be called on the `Box`'s contents when the `Box` is dropped.
/// If `T` owns memory outside of the arena, this will be a memory leak.
///
/// Note: This is not a soundness issue, as Rust does not support relying on `drop`
/// being called to guarantee soundness.
pub struct Box<'alloc, T: ?Sized>(NonNull<T>, PhantomData<(&'alloc (), T)>);

impl<'alloc, T> Box<'alloc, T> {
    /// Take ownership of the value stored in this [`Box`], consuming the box in
    /// the process.
    ///
    /// ## Example
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
    pub fn unbox(self) -> T {
        // SAFETY:
        // This pointer read is safe because the reference `self.0` is
        // guaranteed to be unique--not just now, but we're guaranteed it's not
        // borrowed from some other reference. This in turn is because we never
        // construct a `Box` with a borrowed reference, only with a fresh
        // one just allocated from a Bump.
        unsafe { ptr::read(self.0.as_ptr()) }
    }
}

impl<'alloc, T> Box<'alloc, T> {
    /// Put a `value` into a memory arena and get back a [`Box`] with ownership
    /// to the allocation.
    ///
    /// ## Example
    /// ```
    /// use oxc_allocator::{Allocator, Box};
    ///
    /// let arena = Allocator::default();
    /// let in_arena: Box<i32> = Box::new_in(5, &arena);
    /// ```
    pub fn new_in(value: T, allocator: &Allocator) -> Self {
        Self(NonNull::from(allocator.alloc(value)), PhantomData)
    }

    /// Create a fake [`Box`] with a dangling pointer.
    ///
    /// # SAFETY
    /// Safe to create, but must never be dereferenced, as does not point to a valid `T`.
    /// Only purpose is for mocking types without allocating for const assertions.
    #[allow(unsafe_code, clippy::missing_safety_doc)]
    pub const unsafe fn dangling() -> Self {
        Self(NonNull::dangling(), PhantomData)
    }
}

impl<'alloc, T: ?Sized> Box<'alloc, T> {
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
    #[inline]
    pub(crate) const unsafe fn from_non_null(ptr: NonNull<T>) -> Self {
        Self(ptr, PhantomData)
    }
}

impl<'alloc, T: ?Sized> ops::Deref for Box<'alloc, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: self.0 is always a unique reference allocated from a Bump in Box::new_in
        unsafe { self.0.as_ref() }
    }
}

impl<'alloc, T: ?Sized> ops::DerefMut for Box<'alloc, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: self.0 is always a unique reference allocated from a Bump in Box::new_in
        unsafe { self.0.as_mut() }
    }
}

impl<'alloc, T: ?Sized> AsRef<T> for Box<'alloc, T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<'alloc, T: ?Sized> AsMut<T> for Box<'alloc, T> {
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<'alloc, T: ?Sized + Debug> Debug for Box<'alloc, T> {
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
impl<'alloc, T> Serialize for Box<'alloc, T>
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

impl<'alloc, T: Hash> Hash for Box<'alloc, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

/// Memory address of an AST node in arena.
///
/// `Address` is generated from a `Box<T>`.
/// AST nodes in a `Box` in an arena are guaranteed to never move in memory,
/// so this address acts as a unique identifier for the duration of the arena's existence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address(usize);

impl<'a, T> Box<'a, T> {
    /// Get the memory address of a value allocated in the arena.
    #[inline]
    pub fn address(&self) -> Address {
        Address(ptr::addr_of!(**self) as usize)
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
