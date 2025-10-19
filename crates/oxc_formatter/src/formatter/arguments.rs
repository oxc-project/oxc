use super::{Buffer, Format, FormatResult, Formatter};

/// Type-erased wrapper to format an object. Used by the [crate::format!], [crate::format_args!], and
/// [crate::write!] macros.
///
/// This struct uses dynamic dispatch via trait objects to allow storing heterogeneous
/// `Format<'ast>` implementors in a homogeneous collection without heap allocation.
#[derive(Clone, Copy)]
pub struct Argument<'fmt, 'ast> {
    /// The value to format stored as a trait object reference.
    value: &'fmt dyn Format<'ast>,
}

impl<'fmt, 'ast> Argument<'fmt, 'ast> {
    /// Called by the [crate::format_args] macro. Creates a type-erased value for formatting
    /// an object.
    #[doc(hidden)]
    #[inline]
    pub fn new<F: Format<'ast>>(value: &'fmt F) -> Self {
        Self { value }
    }

    /// Formats the value stored by this argument using the given formatter.
    #[inline(always)]
    pub(super) fn format(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        self.value.fmt(f)
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
