//! `assert_unchecked!` macro.

/// Macro that makes a soundness promise to the compiler that this condition always holds.
///
/// This is wrapper around [`std::hint::assert_unchecked`] which performs a standard safe `assert!`
/// in debug builds, and accepts a format string for the error in that case.
///
/// See documentation for [`std::hint::assert_unchecked`] for further details.
///
/// This macro can be used in const context, but only without format arguments,
/// because `assert!` with format arguments is not const.
///
/// # SAFETY
/// It is undefined behaviour if the condition evaluates to `false`.
///
/// # Examples
/// ```
/// // Requires `assert_unchecked` feature
/// use oxc_data_structures::assert_unchecked;
///
/// /// Read first byte from a `&[u8]`, without bounds checks.
/// ///
/// /// # SAFETY
/// /// Caller must ensure `slice` is not empty.
/// unsafe fn first_unchecked(slice: &[u8]) -> u8 {
///     // SAFETY: Caller guarantees `slice` is not empty
///     unsafe {
///         assert_unchecked!(!slice.is_empty(), "Oh no. Slice is empty.");
///     }
///     // Compiler elides the bounds check here. https://godbolt.org/z/xaGK8GzbG
///     slice[0]
/// }
/// ```
#[macro_export]
macro_rules! assert_unchecked {
    ($cond:expr) => ($crate::assert_unchecked!($cond,));
    ($cond:expr, $($arg:tt)*) => ({
        #[cfg(debug_assertions)]
        {
            // Own block so `needs_unsafe` can't shadow a name used in `$cond` or format args
            {
                #[inline(always)]
                const unsafe fn needs_unsafe() {}
                needs_unsafe();
            }
            ::std::assert!($cond, $($arg)*);
        }
        #[cfg(not(debug_assertions))]
        {
            ::std::hint::assert_unchecked($cond);
        }
    })
}

/**
Doctest to ensure macro cannot be used without `unsafe {}`.
```compile_fail
use oxc_data_structures::assert_unchecked;
assert_unchecked!(true);
```
*/
const _MACRO_CANNOT_BE_USED_WITHOUT_UNSAFE: () = ();

#[cfg(test)]
#[expect(clippy::undocumented_unsafe_blocks)]
mod test {
    mod pass {
        #[test]
        fn plain() {
            unsafe { assert_unchecked!(0 == 0) };
        }

        #[test]
        fn string_literal() {
            unsafe { assert_unchecked!(0 == 0, "String literal") };
        }

        #[test]
        fn fmt_string() {
            unsafe { assert_unchecked!(0 == 0, "Format str: {} {:?}", 123, 456) };
        }

        #[test]
        fn const_context() {
            const fn assert_is_zero(n: u32) {
                unsafe { assert_unchecked!(n == 0, "String literal") }
            }

            const _: () = assert_is_zero(0);
        }
    }

    // Cannot test failing assertions in release mode as it'd be undefined behavior!
    #[cfg(debug_assertions)]
    mod fail {
        #[test]
        #[should_panic(expected = "assertion failed: 0 == 1")]
        fn plain() {
            unsafe { assert_unchecked!(0 == 1) };
        }

        #[test]
        #[should_panic(expected = "String literal")]
        fn string_literal() {
            unsafe { assert_unchecked!(0 == 1, "String literal") };
        }

        #[test]
        #[should_panic(expected = "Format str: 123 456")]
        fn fmt_string() {
            unsafe { assert_unchecked!(0 == 1, "Format str: {} {:?}", 123, 456) };
        }

        #[test]
        #[should_panic(expected = "Caller value: 123")]
        fn caller_name_not_shadowed() {
            let needs_unsafe = 123;
            unsafe { assert_unchecked!(needs_unsafe == 0, "Caller value: {}", needs_unsafe) };
        }
    }
}
