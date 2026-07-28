use crate::{Arguments, Buffer, Formatter};

/// Formatting trait for types that can create a formatted representation. The `biome_formatter` equivalent
/// to [std::fmt::Display].
///
/// ## Example
/// Implementing `Format` for a custom struct
///
/// ```text
/// use biome_formatter::{format, write, IndentStyle, LineWidth};
/// use biome_formatter::prelude::*;
/// use biome_rowan::TextSize;
///
/// struct Paragraph(String);
///
/// impl Format<SimpleFormatContext> for Paragraph {
///     fn fmt(&self, f: &mut Formatter<SimpleFormatContext>)  {
///         write!(f, [
///             hard_line_break(),
///             text(&self.0, TextSize::from(0)),
///             hard_line_break(),
///         ])
///     }
/// }
///
/// # fn main()  {
/// let paragraph = Paragraph(String::from("test"));
/// let formatted = format!(SimpleFormatContext::default(), [paragraph])?;
///
/// assert_eq!("test\n", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
pub trait Format<'ast, C> {
    /// Formats the object using the given formatter.
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>);
}

impl<'ast, C, T> Format<'ast, C> for &T
where
    T: ?Sized + Format<'ast, C>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        Format::fmt(&**self, f);
    }
}

impl<'ast, C, T> Format<'ast, C> for &mut T
where
    T: ?Sized + Format<'ast, C>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        Format::fmt(&**self, f);
    }
}

impl<'ast, C, T> Format<'ast, C> for Option<T>
where
    T: Format<'ast, C>,
{
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        if let Some(value) = self {
            value.fmt(f);
        }
    }
}

impl<C> Format<'_, C> for () {
    #[inline]
    fn fmt(&self, _: &mut Formatter<'_, '_, C>) {
        // Intentionally left empty
    }
}

/// The `write` function takes a target buffer and an `Arguments` struct that can be precompiled with the `format_args!` macro.
///
/// The arguments will be formatted in-order into the output buffer provided.
#[inline(always)]
pub fn write<'ast, C>(output: &mut dyn Buffer<'ast, C>, args: Arguments<'_, 'ast, C>) {
    Formatter::new(output).write_fmt(args);
}
