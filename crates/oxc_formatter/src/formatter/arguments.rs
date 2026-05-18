use std::{ffi::c_void, marker::PhantomData};

use super::{Buffer, Format, Formatter};

/// Mono-morphed type to format an object. Used by the [crate::format!], [crate::format_args!], and
/// [crate::write!] macros.
///
/// This struct is similar to a dynamic dispatch (using `dyn Format`) because it stores a pointer to the value.
/// However, it doesn't store the pointer to `dyn Format`'s vtable, instead it statically resolves the function
/// pointer of `Format::format` and stores it in `formatter`.
pub struct Argument<'fmt, 'ast, C> {
    /// The value to format stored as a raw pointer where `lifetime` stores the value's lifetime.
    value: *const c_void,

    /// Stores the lifetime of the value. To get the most out of our dear borrow checker.
    lifetime: PhantomData<&'fmt ()>,

    /// The function pointer to `value`'s `Format::format` method
    formatter: fn(*const c_void, &mut Formatter<'_, 'ast, C>),
}

// Manual `Copy` / `Clone` to avoid imposing `C: Copy` / `C: Clone` bounds.
impl<C> Copy for Argument<'_, '_, C> {}
impl<C> Clone for Argument<'_, '_, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'fmt, 'ast, C> Argument<'fmt, 'ast, C> {
    /// Called by the [biome_formatter::format_args] macro. Creates a mono-morphed value for formatting
    /// an object.
    #[doc(hidden)]
    #[inline]
    pub fn new<F: Format<'ast, C>>(value: &'fmt F) -> Self {
        #[inline(always)]
        fn formatter<'ast, C, F: Format<'ast, C>>(
            ptr: *const c_void,
            fmt: &mut Formatter<'_, 'ast, C>,
        ) {
            // SAFETY: Safe because the 'fmt lifetime is captured by the 'lifetime' field.
            F::fmt(unsafe { &*ptr.cast::<F>() }, fmt);
        }

        Self {
            value: std::ptr::from_ref::<F>(value).cast::<std::ffi::c_void>(),
            lifetime: PhantomData,
            formatter: formatter::<C, F>,
        }
    }

    /// Formats the value stored by this argument using the given formatter.
    #[inline(always)]
    pub(super) fn format(&self, f: &mut Formatter<'_, 'ast, C>) {
        (self.formatter)(self.value, f);
    }
}

impl<'ast, C> Format<'ast, C> for Argument<'_, 'ast, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        self.format(f);
    }
}

/// Sequence of objects that should be formatted in the specified order.
///
/// The [`format_args!`] macro will safely create an instance of this structure.
pub struct Arguments<'fmt, 'ast, C>(pub &'fmt [Argument<'fmt, 'ast, C>]);

// Manual `Copy` / `Clone` to avoid imposing `C: Copy` / `C: Clone` bounds.
impl<C> Copy for Arguments<'_, '_, C> {}
impl<C> Clone for Arguments<'_, '_, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'fmt, 'ast, C> Arguments<'fmt, 'ast, C> {
    #[doc(hidden)]
    #[inline(always)]
    pub fn new(arguments: &'fmt [Argument<'fmt, 'ast, C>]) -> Self {
        Self(arguments)
    }

    /// Returns the arguments
    #[inline]
    pub fn items(&self) -> &'fmt [Argument<'fmt, 'ast, C>] {
        self.0
    }
}

impl<'ast, C> Format<'ast, C> for Arguments<'_, 'ast, C> {
    #[inline(always)]
    fn fmt(&self, formatter: &mut Formatter<'_, 'ast, C>) {
        formatter.write_fmt(*self);
    }
}

impl<C> std::fmt::Debug for Arguments<'_, '_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Arguments[...]")
    }
}

impl<'fmt, 'ast, C> From<&'fmt Argument<'fmt, 'ast, C>> for Arguments<'fmt, 'ast, C> {
    fn from(argument: &'fmt Argument<'fmt, 'ast, C>) -> Self {
        Arguments::new(std::slice::from_ref(argument))
    }
}
