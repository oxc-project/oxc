//! This module is just for documentation purposes, and is hidden behind the
//! `example_generated` feature, which is off by default.
//!
//! Note that a `cargo expand`ed version of this module (with some slight
//! cleanup -- e.g. removing all the code that comes from builtin derives) is
//! checked in to the [repository](https://github.com/thomcc/index_vec), and may
//! be easier/better to look at.

/// I'm a doc comment on the type.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CoolIndex {
    _raw: u32,
}

impl CoolIndex {
    /// If `Self::CHECKS_MAX_INDEX` is true, we'll assert if trying to
    /// produce a value larger than this in any of the ctors that don't
    /// have `unchecked` in their name.
    pub const MAX_INDEX: usize = i32::max_value() as usize;
    /// Does this index type assert if asked to construct an index
    /// larger than MAX_INDEX?
    pub const CHECKS_MAX_INDEX: bool = !false;
    /// Construct this index type from a usize. Alias for `from_usize`.
    #[inline(always)]
    pub fn new(value: usize) -> Self {
        Self::from_usize(value)
    }
    /// Construct this index type from the wrapped integer type.
    #[inline(always)]
    pub fn from_raw(value: u32) -> Self {
        Self::from_usize(value as usize)
    }
    /// Construct this index type from one in a different domain
    #[inline(always)]
    pub fn from_foreign<F: crate::Idx>(value: F) -> Self {
        Self::from_usize(value.index())
    }
    /// Construct from a usize without any checks.
    #[inline(always)]
    pub const fn from_usize_unchecked(value: usize) -> Self {
        Self { _raw: value as u32 }
    }
    /// Construct from the underlying type without any checks.
    #[inline(always)]
    pub const fn from_raw_unchecked(raw: u32) -> Self {
        Self { _raw: raw }
    }
    /// Construct this index type from a usize.
    #[inline]
    pub fn from_usize(value: usize) -> Self {
        Self::check_index(value as usize);
        Self { _raw: value as u32 }
    }
    /// Get the wrapped index as a usize.
    #[inline(always)]
    pub const fn index(self) -> usize {
        self._raw as usize
    }
    /// Get the wrapped index.
    #[inline(always)]
    pub const fn raw(self) -> u32 {
        self._raw
    }
    /// Asserts `v <= Self::MAX_INDEX` unless Self::CHECKS_MAX_INDEX is false.
    #[inline]
    pub fn check_index(v: usize) {
        if Self::CHECKS_MAX_INDEX && (v > Self::MAX_INDEX) {
            crate::__max_check_fail(v, Self::MAX_INDEX);
        }
    }
    const _ENSURE_RAW_IS_UNSIGNED: [(); 0] = [(); <u32>::min_value() as usize];
}
impl core::fmt::Debug for CoolIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(::core::fmt::Arguments::new_v1(
            &["CI(", ")"],
            &match (&self.index(),) {
                (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
            },
        ))
    }
}
impl core::cmp::PartialOrd<usize> for CoolIndex {
    #[inline]
    fn partial_cmp(&self, other: &usize) -> Option<core::cmp::Ordering> {
        self.index().partial_cmp(other)
    }
}
impl core::cmp::PartialOrd<CoolIndex> for usize {
    #[inline]
    fn partial_cmp(&self, other: &CoolIndex) -> Option<core::cmp::Ordering> {
        self.partial_cmp(&other.index())
    }
}
impl PartialEq<usize> for CoolIndex {
    #[inline]
    fn eq(&self, other: &usize) -> bool {
        self.index() == *other
    }
}
impl PartialEq<CoolIndex> for usize {
    #[inline]
    fn eq(&self, other: &CoolIndex) -> bool {
        *self == other.index()
    }
}
impl core::ops::Add<usize> for CoolIndex {
    type Output = Self;
    #[inline]
    fn add(self, other: usize) -> Self {
        Self::new(self.index().wrapping_add(other))
    }
}
impl core::ops::Sub<usize> for CoolIndex {
    type Output = Self;
    #[inline]
    fn sub(self, other: usize) -> Self {
        Self::new(self.index().wrapping_sub(other))
    }
}
impl core::ops::AddAssign<usize> for CoolIndex {
    #[inline]
    fn add_assign(&mut self, other: usize) {
        *self = *self + other
    }
}
impl core::ops::SubAssign<usize> for CoolIndex {
    #[inline]
    fn sub_assign(&mut self, other: usize) {
        *self = *self - other;
    }
}
impl core::ops::Rem<usize> for CoolIndex {
    type Output = Self;
    #[inline]
    fn rem(self, other: usize) -> Self {
        Self::new(self.index() % other)
    }
}
impl core::ops::Add<CoolIndex> for usize {
    type Output = CoolIndex;
    #[inline]
    fn add(self, other: CoolIndex) -> CoolIndex {
        other + self
    }
}
impl core::ops::Sub<CoolIndex> for usize {
    type Output = CoolIndex;
    #[inline]
    fn sub(self, other: CoolIndex) -> CoolIndex {
        CoolIndex::new(self.wrapping_sub(other.index()))
    }
}
impl core::ops::Add for CoolIndex {
    type Output = CoolIndex;
    #[inline]
    fn add(self, other: CoolIndex) -> CoolIndex {
        CoolIndex::new(other.index() + self.index())
    }
}
impl core::ops::Sub for CoolIndex {
    type Output = CoolIndex;
    #[inline]
    fn sub(self, other: CoolIndex) -> CoolIndex {
        CoolIndex::new(other.index().wrapping_sub(self.index()))
    }
}
impl core::ops::AddAssign for CoolIndex {
    #[inline]
    fn add_assign(&mut self, other: CoolIndex) {
        *self = *self + other
    }
}
impl core::ops::SubAssign for CoolIndex {
    #[inline]
    fn sub_assign(&mut self, other: CoolIndex) {
        *self = *self - other;
    }
}
impl crate::Idx for CoolIndex {
    #[inline]
    fn from_usize(value: usize) -> Self {
        Self::from(value)
    }
    #[inline]
    fn index(self) -> usize {
        usize::from(self)
    }
}
impl From<CoolIndex> for usize {
    #[inline]
    fn from(v: CoolIndex) -> usize {
        v.index()
    }
}
impl From<usize> for CoolIndex {
    #[inline]
    fn from(value: usize) -> Self {
        CoolIndex::from_usize(value)
    }
}
const _: [(); 1] = [(); true as usize];
impl From<CoolIndex> for u32 {
    #[inline]
    fn from(v: CoolIndex) -> u32 {
        v.raw()
    }
}
impl From<u32> for CoolIndex {
    #[inline]
    fn from(value: u32) -> Self {
        Self::from_raw(value)
    }
}
impl core::fmt::Display for CoolIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(::core::fmt::Arguments::new_v1(
            &["", " is a ~Cool Index~"],
            &match (&self.index(),) {
                (arg0,) => [::core::fmt::ArgumentV1::new(
                    arg0,
                    ::core::fmt::Display::fmt,
                )],
            },
        ))
    }
}
impl Default for CoolIndex {
    #[inline]
    fn default() -> Self {
        CoolIndex::new(0)
    }
}