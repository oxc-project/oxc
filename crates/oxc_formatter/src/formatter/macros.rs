/// Creates the Format IR for a value.
///
/// The first argument `format!` receives is the [JsFormatContext] that specify how elements must be formatted.
/// Additional parameters passed get formatted by using their [crate::Format] implementation.
///
///
/// ## Examples
///
/// ```text
/// use biome_formatter::prelude::*;
/// use biome_formatter::format;
///
/// let formatted = format!(SimpleFormatContext::default(), [token("("), token("a"), token(")")]).unwrap();
///
/// assert_eq!(
///     formatted.into_document(),
///     Document::from(vec![
///         FormatElement::Token { text: "(" },
///         FormatElement::Token { text: "a" },
///         FormatElement::Token { text: ")" },
///     ])
/// );
/// ```
#[macro_export]
macro_rules! format {
    ($context:expr, [$($arg:expr),+ $(,)?]) => {{
        ($crate::formatter::format($context, $crate::format_args!($($arg),+)))
    }}
}
