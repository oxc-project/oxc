//! Type-related utilities.
//!
//! Currently just the [`implements!`] macro, answering "does this type implement this trait?" as a `bool`.

/// Does a type implement a trait? Answers as a compile-time `bool`.
///
/// ```
/// use oxc_data_structures::types::implements;
///
/// assert!(implements!(u32: Copy));
/// assert!(!implements!(String: Copy));
/// assert!(implements!(String: From<&'static str>));
/// assert!(!implements!(std::rc::Rc<u32>: Send));
/// ```
///
/// This macro can be used in const context:
///
/// ```
/// # use oxc_data_structures::types::implements;
/// const IS_COPY: bool = implements!(u32: Copy);
/// assert!(IS_COPY);
///
/// const _: () = assert!(implements!(u32: Copy));
/// ```
///
/// # IMPORTANT
///
/// Only works with concrete types. Inside a generic function, the answer for a type parameter
/// `T` reflects `T`'s declared bounds, not the concrete type at any call site, which is very
/// likely not what was intended:
///
/// ```
/// # use oxc_data_structures::types::implements;
/// fn is_copy<T>() -> bool {
///     // WRONG - always `false`, because `T` has no `Copy` bound
///     implements!(T: Copy)
/// }
///
/// fn is_copy_bounded<T: Copy>() -> bool {
///     // Pointless - always `true` - the bound answers, not the call site
///     implements!(T: Copy)
/// }
///
/// // `u32` is `Copy`, but `is_copy` still answers `false`!
/// assert!(!is_copy::<u32>());
/// assert!(is_copy_bounded::<u32>());
/// ```
///
/// Passing a type through a *macro* is fine - expansion substitutes the concrete type
/// before `implements!` examines it:
///
/// ```
/// # use oxc_data_structures::types::implements;
/// macro_rules! assert_copy {
///     ($ty:ty) => {
///         const _: () = assert!(implements!($ty: Copy));
///     };
/// }
///
/// assert_copy!(u32);
/// assert_copy!((u32, char));
/// ```
///
/// # Notes
///
/// The result is a `const` expression, and both answers are first-class values - so tests can
/// assert that a type does *not* implement a trait as easily as that it does.
///
/// Prefer this over a `compile_fail` doctest for trait-presence tests. A `compile_fail` block passes
/// if the code fails to compile for *any* reason, so it can silently stop testing what it intends to.
/// A test using `implements!` does not have this problem.
///
/// # Lifetimes
///
/// For a type with lifetime parameters, omit them:
///
/// ```
/// # use oxc_data_structures::types::implements;
/// #[derive(Clone, Copy)]
/// struct Borrowed<'a>(&'a str);
///
/// assert!(implements!(Borrowed: Copy));
/// ```
///
/// The elided lifetimes are inferred. Writing `'static` for all of them is equivalent -
/// both forms answer whether the `'static` instantiation implements the trait.
///
/// That is also the answer for every other lifetime, unless an impl is deliberately
/// lifetime-dependent (e.g. `impl Copy for Foo<'static>` and no blanket impl) - the only
/// lifetime-dependence an impl can express, and vanishingly rare.
///
/// Lifetimes in the *trait's* arguments must be named (e.g. `From<&'static str>`) -
/// the query becomes an impl bound, where elided lifetimes are not permitted.
///
/// # How it works
///
/// The macro's block defines a `Wrapper<T>` with an *inherent* `DOES_IMPL: bool = true` gated on `T: Trait`,
/// and the blanket-implemented [`DoesNotImpl`] trait supplies `DOES_IMPL = false` for every type.
///
/// Associated-item resolution prefers inherent items to trait items, so the inherent `true` wins
/// exactly when the bound holds - otherwise resolution falls back to the trait's `false`.
/// This is the same mechanism as `impls` crate uses.
///
/// Resolution ignores lifetimes. If a trait is implemented, but only for specific lifetimes
/// (e.g. only for `Foo<'static>`), the inherent `true` still wins, and the lifetime requirement
/// is then enforced on the queried type - satisfied by `'static` or an elided lifetime,
/// a hard error for a generic lifetime parameter (there is no fallback to `false`).
///
/// `Wrapper` must be defined inside each expansion. If it were shared, two expansions querying
/// different traits would produce overlapping inherent impls (e.g. `impl<T: Copy> Wrapper<T>`
/// and `impl<T: Send> Wrapper<T>`, both defining `DOES_IMPL`) - a compile error (E0592).
/// `DoesNotImpl` has no such constraint (one blanket impl serves every query), so it lives in
/// [`__private`] and is imported anonymously (`as _`) into each expansion, keeping expansions
/// small and polluting no other scope.
///
/// [`DoesNotImpl`]: __private::DoesNotImpl
#[macro_export]
macro_rules! implements {
    ($ty:ty: $trait_:path) => {{
        #[allow(unused_imports, clippy::allow_attributes)]
        use $crate::types::__private::DoesNotImpl as _;

        struct Wrapper<T: ?Sized>(::std::marker::PhantomData<T>);

        #[allow(dead_code, clippy::allow_attributes)]
        impl<T: ?Sized + $trait_> Wrapper<T> {
            const DOES_IMPL: bool = true;
        }

        <Wrapper<$ty>>::DOES_IMPL
    }};
}

pub use implements;

/// Not public API. Referenced by the expansion of the [`implements!`] macro.
#[doc(hidden)]
pub mod __private {
    /// Fallback for [`implements!`]: supplies `DOES_IMPL = false` for every type.
    ///
    /// The inherent `DOES_IMPL = true` defined in the macro's expansion
    /// (gated on the queried trait) shadows it when the trait is implemented.
    ///
    /// [`implements!`]: super::implements
    pub trait DoesNotImpl {
        const DOES_IMPL: bool = false;
    }

    impl<T: ?Sized> DoesNotImpl for T {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn implemented() {
        assert!(implements!(u32: Copy));
        assert!(implements!(String: Clone));
        assert!(implements!(Vec<u32>: IntoIterator));
        assert!(implements!(u32: PartialEq<u32>));
        assert!(implements!(u16: From<u8>));
        assert!(implements!(&'static str: Copy));
        // Unsized types can be queried
        assert!(implements!(str: Send));
    }

    #[test]
    fn not_implemented() {
        assert!(!implements!(String: Copy));
        assert!(!implements!(u16: From<u32>));
        assert!(!implements!(std::rc::Rc<u32>: Send));
        assert!(!implements!(std::cell::Cell<u32>: Sync));
        assert!(!implements!(str: Sized));
    }

    #[test]
    fn const_usable() {
        const { assert!(implements!(u32: Copy)) };
        const { assert!(!implements!(String: Copy)) };
    }
}
