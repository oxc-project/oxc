//! `assert_unchecked!` and `unreachable_unchecked!` macros.

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
const _ASSERT_UNCHECKED_CANNOT_BE_USED_WITHOUT_UNSAFE: () = ();

/// Macro that makes a soundness promise to the compiler that this code is never reached.
///
/// This is wrapper around [`std::hint::unreachable_unchecked`] which performs a standard safe
/// `unreachable!` in debug builds, and accepts a format string for the error in that case.
///
/// See documentation for [`std::hint::unreachable_unchecked`] for further details.
///
/// This macro can be used in const context, but only without a format string,
/// because `unreachable!` with a format string is not const.
///
/// # SAFETY
/// It is undefined behaviour if this code is reached.
///
/// # Examples
/// ```
/// // Requires `assert_unchecked` feature
/// use oxc_data_structures::unreachable_unchecked;
///
/// /// Get value from an `Option`, without checking that it's `Some`.
/// ///
/// /// # SAFETY
/// /// Caller must ensure `option` is `Some`.
/// unsafe fn unwrap_unchecked(option: Option<u32>) -> u32 {
///     match option {
///         Some(value) => value,
///         // SAFETY: Caller guarantees `option` is `Some`
///         None => unsafe { unreachable_unchecked!("Oh no. Option is `None`.") },
///     }
/// }
/// ```
#[macro_export]
macro_rules! unreachable_unchecked {
    ($($arg:tt)*) => ({
        #[cfg(debug_assertions)]
        {
            // Own block so `needs_unsafe` can't shadow a name used in format args
            {
                #[inline(always)]
                const unsafe fn needs_unsafe() {}
                needs_unsafe();
            }
            ::std::unreachable!($($arg)*);
        }
        #[cfg(not(debug_assertions))]
        {
            ::std::hint::unreachable_unchecked();
        }
    })
}

/**
Doctest to ensure macro cannot be used without `unsafe {}`.
```compile_fail
use oxc_data_structures::unreachable_unchecked;
unreachable_unchecked!();
```
*/
const _UNREACHABLE_UNCHECKED_CANNOT_BE_USED_WITHOUT_UNSAFE: () = ();

#[cfg(test)]
#[expect(clippy::undocumented_unsafe_blocks)]
mod test {
    mod assert_unchecked {
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

    mod unreachable_unchecked {
        mod pass {
            #[test]
            fn plain() {
                let Some(value) = Some(1) else { unsafe { unreachable_unchecked!() } };
                assert_eq!(value, 1);
            }

            #[test]
            fn string_literal() {
                let Some(value) = Some(1) else {
                    unsafe { unreachable_unchecked!("String literal") }
                };
                assert_eq!(value, 1);
            }

            #[test]
            fn fmt_string() {
                let Some(value) = Some(1) else {
                    unsafe { unreachable_unchecked!("Format str: {} {:?}", 123, 456) }
                };
                assert_eq!(value, 1);
            }

            #[test]
            fn const_context() {
                const fn unwrap(option: Option<u32>) -> u32 {
                    match option {
                        Some(value) => value,
                        None => unsafe { unreachable_unchecked!() },
                    }
                }

                const VALUE: u32 = unwrap(Some(1));
                assert_eq!(VALUE, 1);
            }
        }

        // Cannot test reaching the macro in release mode as it'd be undefined behavior!
        #[cfg(debug_assertions)]
        mod fail {
            #[test]
            #[should_panic(expected = "internal error: entered unreachable code")]
            fn plain() {
                unsafe { unreachable_unchecked!() };
            }

            #[test]
            #[should_panic(expected = "internal error: entered unreachable code: String literal")]
            fn string_literal() {
                unsafe { unreachable_unchecked!("String literal") };
            }

            #[test]
            #[should_panic(
                expected = "internal error: entered unreachable code: Format str: 123 456"
            )]
            fn fmt_string() {
                unsafe { unreachable_unchecked!("Format str: {} {:?}", 123, 456) };
            }

            #[test]
            #[should_panic(
                expected = "internal error: entered unreachable code: Caller value: 123"
            )]
            fn caller_name_not_shadowed() {
                let needs_unsafe = 123;
                unsafe { unreachable_unchecked!("Caller value: {}", needs_unsafe) };
            }
        }
    }
}
