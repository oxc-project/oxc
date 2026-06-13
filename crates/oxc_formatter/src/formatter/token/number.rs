pub use oxc_formatter_core::spec::{format_trimmed_number, is_simple_number};

use crate::formatter::{Format, JsFormatter, prelude::*};

pub fn format_number_token(
    text: &str,
    keep_one_trailing_decimal_zero: bool,
) -> CleanedNumberLiteralText<'_> {
    CleanedNumberLiteralText { text, keep_one_trailing_decimal_zero }
}

pub struct CleanedNumberLiteralText<'a> {
    text: &'a str,
    keep_one_trailing_decimal_zero: bool,
}

impl<'a> Format<'a, JsFormatContext<'a>> for CleanedNumberLiteralText<'a> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let text = format_trimmed_number(self.text, self.keep_one_trailing_decimal_zero);
        text_without_whitespace(f.allocator().alloc_str(&text)).fmt(f);
    }
}
