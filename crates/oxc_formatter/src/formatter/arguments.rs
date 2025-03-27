use super::FormatResult;
use super::{Buffer, Format, Formatter};
use std::ffi::c_void;
use std::marker::PhantomData;

/// Mono-morphed type to format an object. Used by the [crate::format!], [crate::format_args!], and
/// [crate::write!] macros.
///
/// This struct is similar to a dynamic dispatch (using `dyn Format`) because it stores a pointer to the value.
/// However, it doesn't store the pointer to `dyn Format`'s vtable, instead it statically resolves the function
/// pointer of `Format::format` and stores it in `formatter`.
pub struct Argument<'fmt> {
    /// The value to format stored as a raw pointer where `lifetime` stores the value's lifetime.
    value: *const c_void,

    /// Stores the lifetime of the value. To get the most out of our dear borrow checker.
    lifetime: PhantomData<&'fmt ()>,

    /// The function pointer to `value`'s `Format::format` method
    formatter: fn(*const c_void, &mut Formatter<'_>) -> FormatResult<()>,
}

impl Clone for Argument<'_> {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for Argument<'_> {}

impl<'fmt> Argument<'fmt> {
    /// Called by the [biome_formatter::format_args] macro. Creates a mono-morphed value for formatting
    /// an object.
    #[doc(hidden)]
    #[inline]
    pub fn new<F: Format>(value: &'fmt F) -> Self {
        #[inline(always)]
        fn formatter<F: Format>(ptr: *const c_void, fmt: &mut Formatter) -> FormatResult<()> {
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
    pub(super) fn format(&self, f: &mut Formatter) -> FormatResult<()> {
        (self.formatter)(self.value, f)
    }
}

impl Format for Argument<'_> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter) -> FormatResult<()> {
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
pub struct Arguments<'fmt>(pub &'fmt [Argument<'fmt>]);

impl<'fmt> Arguments<'fmt> {
    #[doc(hidden)]
    #[inline(always)]
    pub fn new(arguments: &'fmt [Argument<'fmt>]) -> Self {
        Self(arguments)
    }

    /// Returns the arguments
    #[inline]
    pub fn items(&self) -> &'fmt [Argument<'fmt>] {
        self.0
    }
}

impl Copy for Arguments<'_> {}

impl Clone for Arguments<'_> {
    fn clone(&self) -> Self {
        *self
    }
}

impl Format for Arguments<'_> {
    #[inline(always)]
    fn fmt(&self, formatter: &mut Formatter) -> FormatResult<()> {
        formatter.write_fmt(*self)
    }
}

impl std::fmt::Debug for Arguments<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Arguments[...]")
    }
}

impl<'fmt> From<&'fmt Argument<'fmt>> for Arguments<'fmt> {
    fn from(argument: &'fmt Argument<'fmt>) -> Self {
        Arguments::new(std::slice::from_ref(argument))
    }
}

#[cfg(test)]
mod tests {
    use crate::format_element::tag::Tag;
    use crate::prelude::*;
    use crate::{FormatState, VecBuffer, format_args, write};

    #[test]
    fn test_nesting() {
        let mut context = FormatState::new(());
        let mut buffer = VecBuffer::new(&mut context);

        write!(
            &mut buffer,
            [
                text("function"),
                space(),
                text("a"),
                space(),
                group(&format_args!(text("("), text(")")))
            ]
        )
        .unwrap();

        assert_eq!(
            buffer.into_vec(),
            vec![
                FormatElement::StaticText { text: "function" },
                FormatElement::Space,
                FormatElement::StaticText { text: "a" },
                FormatElement::Space,
                // Group
                FormatElement::Tag(Tag::StartGroup(tag::Group::new())),
                FormatElement::StaticText { text: "(" },
                FormatElement::StaticText { text: ")" },
                FormatElement::Tag(Tag::EndGroup)
            ]
        );
    }
}
