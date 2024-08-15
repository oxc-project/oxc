//! Bumpalo memory arena utilities
//! Copied from [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/crates/ast/src/arena.rs)

use std::{
    self,
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{self, Deref},
    ptr::{self, NonNull},
};

use allocator_api2::vec;
pub use bumpalo::collections::String;
use bumpalo::Bump;
#[cfg(any(feature = "serialize", test))]
use serde::{ser::SerializeSeq, Serialize, Serializer};

use crate::Allocator;

/// A Box without Drop.
/// This is used for over coming self-referential structs.
/// It is a memory leak if the boxed value has a `Drop` implementation.
pub struct Box<'alloc, T: ?Sized>(NonNull<T>, PhantomData<(&'alloc (), T)>);

impl<'alloc, T> Box<'alloc, T> {
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
    pub fn new_in(value: T, allocator: &Allocator) -> Self {
        Self(NonNull::from(allocator.alloc(value)), PhantomData)
    }

    /// Create a fake `Box` with a dangling pointer.
    /// # SAFETY
    /// Safe to create, but must never be dereferenced, as does not point to a valid `T`.
    /// Only purpose is for mocking types without allocating for const assertions.
    #[allow(unsafe_code, clippy::missing_safety_doc)]
    pub const unsafe fn dangling() -> Self {
        Self(NonNull::dangling(), PhantomData)
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

/// Bumpalo Vec
#[derive(Debug, PartialEq, Eq)]
pub struct Vec<'alloc, T>(vec::Vec<T, &'alloc Bump>);

impl<'alloc, T> Vec<'alloc, T> {
    #[inline]
    pub fn new_in(allocator: &'alloc Allocator) -> Self {
        Self(vec::Vec::new_in(allocator))
    }

    #[inline]
    pub fn with_capacity_in(capacity: usize, allocator: &'alloc Allocator) -> Self {
        Self(vec::Vec::with_capacity_in(capacity, allocator))
    }

    #[inline]
    pub fn from_iter_in<I: IntoIterator<Item = T>>(iter: I, allocator: &'alloc Allocator) -> Self {
        let iter = iter.into_iter();
        let capacity = iter.size_hint().1.unwrap_or(0);
        let mut vec = vec::Vec::with_capacity_in(capacity, &**allocator);
        vec.extend(iter);
        Self(vec)
    }
}

impl<'alloc, T> ops::Deref for Vec<'alloc, T> {
    type Target = vec::Vec<T, &'alloc Bump>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'alloc, T> ops::DerefMut for Vec<'alloc, T> {
    fn deref_mut(&mut self) -> &mut vec::Vec<T, &'alloc Bump> {
        &mut self.0
    }
}

impl<'alloc, T> IntoIterator for Vec<'alloc, T> {
    type IntoIter = <vec::Vec<T, &'alloc Bump> as IntoIterator>::IntoIter;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'alloc, T> IntoIterator for &'alloc Vec<'alloc, T> {
    type IntoIter = std::slice::Iter<'alloc, T>;
    type Item = &'alloc T;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'alloc, T> ops::Index<usize> for Vec<'alloc, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<'alloc, T> ops::Index<usize> for &'alloc Vec<'alloc, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

// Unused right now.
// impl<'alloc, T> ops::IndexMut<usize> for Vec<'alloc, T> {
// fn index_mut(&mut self, index: usize) -> &mut Self::Output {
// self.0.index_mut(index)
// }
// }

#[cfg(any(feature = "serialize", test))]
impl<'alloc, T> Serialize for Vec<'alloc, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = s.serialize_seq(Some(self.0.len()))?;
        for e in &self.0 {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

impl<'alloc, T: Hash> Hash for Vec<'alloc, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for e in &self.0 {
            e.hash(state);
        }
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
    #[inline]
    pub fn address(&self) -> Address {
        Address(ptr::addr_of!(**self) as usize)
    }
}

#[cfg(test)]
mod test {
    use std::hash::{DefaultHasher, Hash, Hasher};

    use crate::{Allocator, Box, Vec};

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
    fn vec_debug() {
        let allocator = Allocator::default();
        let mut v = Vec::new_in(&allocator);
        v.push("x");
        let v = format!("{v:?}");
        assert_eq!(v, "Vec([\"x\"])");
    }

    #[test]
    fn box_serialize() {
        let allocator = Allocator::default();
        let b = Box::new_in("x", &allocator);
        let b = serde_json::to_string(&b).unwrap();
        assert_eq!(b, "\"x\"");
    }

    #[test]
    fn vec_serialize() {
        let allocator = Allocator::default();
        let mut v = Vec::new_in(&allocator);
        v.push("x");
        let v = serde_json::to_string(&v).unwrap();
        assert_eq!(v, "[\"x\"]");
    }

    #[test]
    fn lifetime_variance() {
        fn _assert_box_variant_lifetime<'a: 'b, 'b, T>(program: Box<'a, T>) -> Box<'b, T> {
            program
        }
        fn _assert_vec_variant_lifetime<'a: 'b, 'b, T>(program: Vec<'a, T>) -> Vec<'b, T> {
            program
        }
    }
}
