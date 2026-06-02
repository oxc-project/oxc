pub use oxc_formatter_core::util::{NumberFormatOptions, format_trimmed_number, is_simple_number};

use crate::formatter::{Format, JsFormatter, prelude::*};

pub fn format_number_token(
    text: &str,
    options: NumberFormatOptions,
) -> CleanedNumberLiteralText<'_>
where
{
    CleanedNumberLiteralText { text, options }
}

pub struct CleanedNumberLiteralText<'a> {
    text: &'a str,
    options: NumberFormatOptions,
}

impl<'a> Format<'a, JsFormatContext<'a>> for CleanedNumberLiteralText<'a> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let text = format_trimmed_number(self.text, self.options);
        text_without_whitespace(f.allocator().alloc_str(&text)).fmt(f);
    }
}
