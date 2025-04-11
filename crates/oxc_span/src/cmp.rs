//! Specialized comparison traits

/// This trait works similarly to [PartialEq] but it gives the liberty of checking the equality of the
/// content loosely.
///
/// This would mean the implementor can skip some parts of the content while doing equality checks.
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

/// Compare `f64` as bits instead of using `==`.
///
/// Result is the same as `partial_eq` (`==`), with the following exceptions:
///
/// * `+0` and `-0` are not `content_eq` (they are `partial_eq`).
/// * `f64::NAN` and `f64::NAN` are `content_eq` (they are not `partial_eq`).
///
/// <https://play.rust-lang.org/?version=stable&mode=release&edition=2021&gist=5f9ec4b26128363a660e27582d1de7cd>
///
/// ### NaN
///
/// Comparison of `NaN` is complicated. From Rust's docs for `f64`:
///
/// > Note that IEEE 754 doesn’t define just a single NaN value;
/// > a plethora of bit patterns are considered to be NaN.
///
/// <https://doc.rust-lang.org/std/primitive.f64.html#associatedconstant.NAN>
///
/// If either value is `NaN`, `f64::content_eq` only returns `true` if both are the *same* `NaN`,
/// with the same bit pattern. This means, for example:
///
/// ```text
/// f64::NAN.content_eq(f64::NAN) == true
/// f64::NAN.content_eq(-f64::NAN) == false
/// f64::NAN.content_eq(--f64::NAN) == true
/// ```
///
/// Any other `NaN`s which are created through an arithmetic operation, rather than explicitly
/// with `f64::NAN`, are not guaranteed to equal `f64::NAN`.
///
/// ```text
/// // This results in `false` on at least some flavors of `x84_64`,
/// // but that's not specified - could also result in `true`!
/// (-1f64).sqrt().content_eq(f64::NAN) == false
/// ```
impl ContentEq for f64 {
    #[inline]
    fn content_eq(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}

/// Blanket implementation for [Option] types
impl<T: ContentEq> ContentEq for Option<T> {
    #[inline]
    fn content_eq(&self, other: &Self) -> bool {
        // NOTE: based on the standard library
        // Spelling out the cases explicitly optimizes better than
        // `_ => false`
        #[expect(clippy::match_same_arms)]
        match (self, other) {
            (Some(lhs), Some(rhs)) => lhs.content_eq(rhs),
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (None, None) => true,
        }
    }
}

/// Blanket implementation for [oxc_allocator::Box] types
impl<T: ContentEq> ContentEq for oxc_allocator::Box<'_, T> {
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
impl<T: ContentEq> ContentEq for oxc_allocator::Vec<'_, T> {
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
    }
}
