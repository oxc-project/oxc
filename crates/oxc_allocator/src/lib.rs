use std::{
    fmt,
    hash::{Hash, Hasher},
    ops::{self, Deref},
    ptr,
};

use blink_alloc::{Blink, BlinkAlloc};
use serde::ser::{Serialize, Serializer};

// #[derive(Debug)]
pub struct Allocator {
    blink: Blink,
}

// SAFETY: Make Bump Sync and Send, it's our responsibility to never
// simultaneously mutate across threads.
unsafe impl Send for Allocator {}
unsafe impl Sync for Allocator {}

// pub type Box<'a, T> = allocator_api2::boxed::Box<T, &'a Allocator>;
pub type Vec<'a, T> = allocator_api2::vec::Vec<T, &'a BlinkAlloc>;

impl Default for Allocator {
    fn default() -> Self {
        Self { blink: Blink::new() }
    }
}

impl Deref for Allocator {
    type Target = Blink;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.blink
    }
}

pub struct String<'a> {
    vec: Vec<'a, u8>,
}

impl<'a> String<'a> {
    #[inline(always)]
    pub fn new_in(alloc: &'a Allocator) -> Self {
        String { vec: Vec::new_in(alloc.allocator()) }
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.vec) }
    }

    #[inline(always)]
    pub fn push(&mut self, ch: char) {
        match ch.len_utf8() {
            1 => self.vec.push(ch as u8),
            _ => self.vec.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes()),
        }
    }

    #[inline(always)]
    pub fn leak(self) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(self.vec.leak()) }
    }

    #[inline(always)]
    pub fn from_str_in(s: &str, alloc: &'a Allocator) -> Self {
        let mut vec = Vec::with_capacity_in(s.len(), alloc.allocator());
        vec.extend_from_slice(s.as_bytes());
        String { vec }
    }

    #[inline(always)]
    pub fn push_str(&mut self, s: &str) {
        self.vec.extend_from_slice(s.as_bytes());
    }
}

impl<'a> fmt::Debug for String<'a> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

/// Bumpalo Box
pub struct Box<'alloc, T: ?Sized>(pub &'alloc mut T);

impl<'alloc, T> Box<'alloc, T> {
    #[must_use]
    pub fn into_inner(b: Self) -> T {
        // This pointer read is safe because the reference `self.0` is
        // guaranteed to be unique--not just now, but we're guaranteed it's not
        // borrowed from some other reference. This in turn is because we never
        // construct an alloc::Box with a borrowed reference, only with a fresh
        // one just allocated from a Bump.
        unsafe { ptr::read(b.0 as *mut T) }
    }
}

impl<'alloc, T: ?Sized> ops::Deref for Box<'alloc, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0
    }
}

impl<'alloc, T: ?Sized> ops::DerefMut for Box<'alloc, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

impl<'alloc, T: ?Sized + fmt::Debug> fmt::Debug for Box<'alloc, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'alloc, T> PartialEq for Box<'alloc, T>
where
    T: PartialEq<T> + ?Sized,
{
    fn eq(&self, other: &Box<'alloc, T>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

impl<'alloc, T> Serialize for Box<'alloc, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(s)
    }
}

impl<'alloc, T: Hash> Hash for Box<'alloc, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
