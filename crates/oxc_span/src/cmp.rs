//! Specialized comparison traits

/// This trait works similarly to [PartialEq] but it gives the liberty of checking the equality of the
/// content loosely. This would mean the implementor can skip some parts of the content while doing
/// equality checks.
/// As an example, In AST types we ignore fields such as [crate::Span].
///
/// One should always prefer using the [PartialEq] over this since implementations of this trait
/// inherently are slower or in the best-case scenario as fast as the [PartialEq] comparison.
pub trait ContentEq {
    /// This method tests for contents of `self` and `other` to be equal.
    #[must_use]
    fn content_eq(&self, other: &Self) -> bool;

    /// This method tests for contents of `self` and `other` not to be equal.
    /// The default implementation is almost always
    /// sufficient, and should not be overridden without very good reason.
    #[inline]
    #[must_use]
    fn content_ne(&self, other: &Self) -> bool {
        !self.content_eq(other)
    }
}

impl ContentEq for () {
    #[inline]
    fn content_eq(&self, _other: &()) -> bool {
        true
    }

    #[inline]
    fn content_ne(&self, _other: &()) -> bool {
        false
    }
}

/// Blanket implementation for [Option] types
impl<T: ContentEq> ContentEq for Option<T> {
    #[inline]
    fn content_eq(&self, other: &Self) -> bool {
        // NOTE: based on the standard library
        // Spelling out the cases explicitly optimizes better than
        // `_ => false`
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Some(lhs), Some(rhs)) => lhs.content_eq(rhs),
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (None, None) => true,
        }
    }
}

/// Blanket implementation for [oxc_allocator::Box] types
impl<'a, T: ContentEq> ContentEq for oxc_allocator::Box<'a, T> {
    #[inline]
    fn content_eq(&self, other: &Self) -> bool {
        self.as_ref().content_eq(other.as_ref())
    }
}

/// Blanket implementation for [oxc_allocator::Vec] types
///
/// # Warning
/// This implementation is slow compared to [PartialEq] for native types which are [Copy] (e.g. `u32`).
/// Prefer comparing the 2 vectors using `==` if they contain such native types (e.g. `Vec<u32>`).
/// <https://godbolt.org/z/54on5sMWc>
impl<'a, T: ContentEq> ContentEq for oxc_allocator::Vec<'a, T> {
    #[inline]
    fn content_eq(&self, other: &Self) -> bool {
        if self.len() == other.len() {
            !self.iter().zip(other).any(|(lhs, rhs)| lhs.content_ne(rhs))
        } else {
            false
        }
    }
}

mod content_eq_auto_impls {
    #![allow(clippy::float_cmp)]
    use super::ContentEq;
    macro_rules! content_eq_impl {
        ($($t:ty)*) => ($(
            impl ContentEq for $t {
                #[inline]
                fn content_eq(&self, other: &$t) -> bool { (*self) == (*other) }
                #[inline]
                fn content_ne(&self, other: &$t) -> bool { (*self) != (*other) }
            }
        )*)
    }

    content_eq_impl! {
        char &str
        bool isize usize
        u8 u16 u32 u64 u128
        i8 i16 i32 i64 i128
        f32 f64
    }
}
