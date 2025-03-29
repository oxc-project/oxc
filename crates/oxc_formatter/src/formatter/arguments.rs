use std::{ffi::c_void, marker::PhantomData};

use super::{Buffer, Format, FormatResult, Formatter};

/// Mono-morphed type to format an object. Used by the [crate::format!], [crate::format_args!], and
/// [crate::write!] macros.
///
/// This struct is similar to a dynamic dispatch (using `dyn Format`) because it stores a pointer to the value.
/// However, it doesn't store the pointer to `dyn Format`'s vtable, instead it statically resolves the function
/// pointer of `Format::format` and stores it in `formatter`.
#[derive(Clone, Copy)]
pub struct Argument<'fmt, 'ast> {
    /// The value to format stored as a raw pointer where `lifetime` stores the value's lifetime.
    value: *const c_void,

    /// Stores the lifetime of the value. To get the most out of our dear borrow checker.
    lifetime: PhantomData<&'fmt ()>,

    /// The function pointer to `value`'s `Format::format` method
    formatter: fn(*const c_void, &mut Formatter<'_, 'ast>) -> FormatResult<()>,
}

impl<'fmt, 'ast> Argument<'fmt, 'ast> {
    /// Called by the [biome_formatter::format_args] macro. Creates a mono-morphed value for formatting
    /// an object.
    #[doc(hidden)]
    #[inline]
    pub fn new<F: Format<'ast>>(value: &'fmt F) -> Self {
        #[inline(always)]
        fn formatter<'ast, F: Format<'ast>>(
            ptr: *const c_void,
            fmt: &mut Formatter<'_, 'ast>,
        ) -> FormatResult<()> {
            // SAFETY: Safe because the 'fmt lifetime is captured by the 'lifetime' field.
            F::fmt(unsafe { &*ptr.cast::<F>() }, fmt)
        }

        Self {
            value: std::ptr::from_ref::<F>(value).cast::<std::ffi::c_void>(),
            lifetime: PhantomData,
            formatter: formatter::<F>,
        }
    }

    /// Formats the value stored by this argument using the given formatter.
    #[inline(always)]
    pub(super) fn format(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        (self.formatter)(self.value, f)
    }
}

impl<'ast> Format<'ast> for Argument<'_, 'ast> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        self.format(f)
    }
}

/// Sequence of objects that should be formatted in the specified order.
///
/// The [`format_args!`] macro will safely create an instance of this structure.
///
/// You can use the `Arguments<a>` that [`format_args!]` return in `Format` context as seen below.
/// It will call the `format` function for every of it's objects.
///
/// ```rust
/// use biome_formatter::prelude::*;
/// use biome_formatter::{format, format_args};
///
/// # fn main() -> FormatResult<()> {
/// let formatted = format!(SimpleFormatContext::default(), [
///     format_args!(text("a"), space(), text("b"))
/// ])?;
///
/// assert_eq!("a b", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct Arguments<'fmt, 'ast>(pub &'fmt [Argument<'fmt, 'ast>]);

impl<'fmt, 'ast> Arguments<'fmt, 'ast> {
    #[doc(hidden)]
    #[inline(always)]
    pub fn new(arguments: &'fmt [Argument<'fmt, 'ast>]) -> Self {
        Self(arguments)
    }

    /// Returns the arguments
    #[inline]
    pub fn items(&self) -> &'fmt [Argument<'fmt, 'ast>] {
        self.0
    }
}

impl<'ast> Format<'ast> for Arguments<'_, 'ast> {
    #[inline(always)]
    fn fmt(&self, formatter: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        formatter.write_fmt(*self)
    }
}

impl std::fmt::Debug for Arguments<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Arguments[...]")
    }
}

impl<'fmt, 'ast> From<&'fmt Argument<'fmt, 'ast>> for Arguments<'fmt, 'ast> {
    fn from(argument: &'fmt Argument<'fmt, 'ast>) -> Self {
        Arguments::new(std::slice::from_ref(argument))
    }
}
