use std::num::NonZero;

/// Types which contain no brand, so their lifetime can be changed
/// by [`BrandScope::extend_lifetime`].
///
/// Implement it on the type itself (with any lifetime).
/// `WithLifetime<'a>` is the same type, with lifetime `'a`.
///
/// ```
/// use oxc_data_structures::brands::Unbranded;
///
/// struct Word<'a>(&'a str);
///
/// // SAFETY: `Word`'s only field is a `&'a str`.
/// // Nothing relies on its lifetime to identify an owner.
/// unsafe impl Unbranded for Word<'_> {
///     type WithLifetime<'a> = Word<'a>;
/// }
/// ```
///
/// You can't implement it for a type from another crate (Rust's orphan rule).
/// Wrap that type in a new type, and implement it on the wrapper.
///
/// It's already implemented for:
///
/// - `&T`, `&mut T`, `&[T]` and `&mut [T]`, where `T` is `Unbranded`.
/// - `&str` and `&mut str`.
/// - `Option<T>`, `Result<T, E>`, `Box<T>`, `Vec<T>`, and arrays `[T; N]`,
///   where `T` and `E` are `Unbranded`.
/// - Tuples of up to 12 `Unbranded` types.
/// - `()`, `bool`, `char`, `String`, and the integer, float and `NonZero` integer types.
///   These have no lifetime, so `WithLifetime<'a>` is the type itself.
///   They're here so types built from them are `Unbranded` (e.g. `&u32`, `(&str, usize)`),
///   and for generic code which requires an `Unbranded` type.
///   On its own, a value with no lifetime can be returned from the closure as it is.
///
/// # SAFETY
///
/// Nothing may rely on the lifetime `'a` of a `WithLifetime<'a>` to identify an owner.
/// [`BrandScope::extend_lifetime`] changes that lifetime. Anything which relied on it could then
/// be used with the wrong owner, or leave the closure it's confined to (see [module docs]).
///
/// So `WithLifetime<'a>` must not:
///
/// - Name [`BrandOrigin`] or [`Brand`] anywhere in the type. Not as a field, a type parameter,
///   a parameter or return type of a `fn` pointer or `dyn Fn`, or a parameter of a `dyn Trait`.
///   The same goes for any type which names one (e.g. `NameIndex<'a>` in the [module docs]).
/// - Contain a `dyn Trait + 'a`. The value behind it could be an owner, or hold a brand,
///   and the type doesn't show it.
/// - Use `'a` as a brand of its own (e.g. an invariant marker which unsafe code relies on).
///
/// [module docs]: super
/// [`BrandOrigin`]: super::BrandOrigin
/// [`Brand`]: super::Brand
/// [`BrandScope::extend_lifetime`]: super::BrandScope::extend_lifetime
pub unsafe trait Unbranded {
    /// The same type, with lifetime `'a`.
    ///
    /// It must outlive `'a`, for every `'a`. So `'a` and `'static` are the only lifetimes
    /// it can contain. That does 2 things:
    ///
    /// - It allows a reference `&'a T::WithLifetime<'a>` to exist.
    /// - A brand can only get into the type through `'a` itself, never through a 2nd lifetime.
    type WithLifetime<'a>: 'a;
}

// SAFETY: `T::WithLifetime<'a>` meets `Unbranded`'s contract, so a reference to one does too
unsafe impl<T: Unbranded> Unbranded for &T {
    type WithLifetime<'a> = &'a T::WithLifetime<'a>;
}

// SAFETY: `T::WithLifetime<'a>` meets `Unbranded`'s contract, so a reference to one does too
unsafe impl<T: Unbranded> Unbranded for &mut T {
    type WithLifetime<'a> = &'a mut T::WithLifetime<'a>;
}

// The impls for `&T` and `&mut T` above only cover `Sized` types.
// The impls below are for references to unsized types - `str`, and slices.

// SAFETY: `str` has no lifetime, so a reference to one meets `Unbranded`'s contract
unsafe impl Unbranded for &str {
    type WithLifetime<'a> = &'a str;
}

// SAFETY: `str` has no lifetime, so a reference to one meets `Unbranded`'s contract
unsafe impl Unbranded for &mut str {
    type WithLifetime<'a> = &'a mut str;
}

// SAFETY: `T::WithLifetime<'a>` meets `Unbranded`'s contract,
// so a reference to a slice of them does too
unsafe impl<T: Unbranded> Unbranded for &[T] {
    type WithLifetime<'a> = &'a [T::WithLifetime<'a>];
}

// SAFETY: `T::WithLifetime<'a>` meets `Unbranded`'s contract,
// so a reference to a slice of them does too
unsafe impl<T: Unbranded> Unbranded for &mut [T] {
    type WithLifetime<'a> = &'a mut [T::WithLifetime<'a>];
}

// SAFETY: `T::WithLifetime<'a>` meets `Unbranded`'s contract, so an array of them does too
unsafe impl<T: Unbranded, const N: usize> Unbranded for [T; N] {
    type WithLifetime<'a> = [T::WithLifetime<'a>; N];
}

// SAFETY: `T::WithLifetime<'a>` meets `Unbranded`'s contract, so an `Option` of one does too
unsafe impl<T: Unbranded> Unbranded for Option<T> {
    type WithLifetime<'a> = Option<T::WithLifetime<'a>>;
}

// SAFETY: `T::WithLifetime<'a>` and `E::WithLifetime<'a>` meet `Unbranded`'s contract,
// so a `Result` of them does too
unsafe impl<T: Unbranded, E: Unbranded> Unbranded for Result<T, E> {
    type WithLifetime<'a> = Result<T::WithLifetime<'a>, E::WithLifetime<'a>>;
}

// SAFETY: `T::WithLifetime<'a>` meets `Unbranded`'s contract, so a `Box` of one does too
unsafe impl<T: Unbranded> Unbranded for Box<T> {
    type WithLifetime<'a> = Box<T::WithLifetime<'a>>;
}

// SAFETY: `T::WithLifetime<'a>` meets `Unbranded`'s contract, so a `Vec` of them does too
unsafe impl<T: Unbranded> Unbranded for Vec<T> {
    type WithLifetime<'a> = Vec<T::WithLifetime<'a>>;
}

/// Implement [`Unbranded`] for types which have no lifetime.
///
/// Only for use on types which contain no lifetime at all.
macro_rules! impl_unbranded_without_lifetime {
    ($($ty:ty),+ $(,)?) => {
        $(
            // SAFETY: This type has no lifetime, so it meets `Unbranded`'s contract
            unsafe impl Unbranded for $ty {
                type WithLifetime<'a> = $ty;
            }
        )+
    };
}

#[rustfmt::skip]
impl_unbranded_without_lifetime!(
    (), bool, char, String,
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
    NonZero<u8>, NonZero<u16>, NonZero<u32>, NonZero<u64>, NonZero<u128>, NonZero<usize>,
    NonZero<i8>, NonZero<i16>, NonZero<i32>, NonZero<i64>, NonZero<i128>, NonZero<isize>,
    f32, f64,
);

/// Implement [`Unbranded`] for tuples of `Unbranded` types.
///
/// Implements it for a tuple of all the types given, then for each shorter tuple,
/// down to a tuple of 1 type.
macro_rules! impl_unbranded_for_tuples {
    ($first:ident $(, $rest:ident)*) => {
        // SAFETY: All the `WithLifetime<'a>` types meet `Unbranded`'s contract,
        // so a tuple of them does too
        unsafe impl<$first: Unbranded $(, $rest: Unbranded)*> Unbranded for ($first, $($rest,)*) {
            type WithLifetime<'a> = ($first::WithLifetime<'a>, $($rest::WithLifetime<'a>,)*);
        }

        impl_unbranded_for_tuples!($($rest),*);
    };
    () => {};
}

impl_unbranded_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
