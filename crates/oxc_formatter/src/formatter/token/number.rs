use std::borrow::Cow;

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
        // In the common case the number needs no reformatting, so `format_trimmed_number`
        // returns a `Cow::Borrowed` slice of the source (lifetime `'a`); pass it straight
        // through and only copy into the arena for the owned (reformatted) case. Mirrors the
        // borrowed-passthrough for string literals in `utils/string.rs`.
        let text_str: &'a str = match &text {
            Cow::Borrowed(borrowed) => borrowed,
            Cow::Owned(owned) => f.allocator().alloc_str(owned),
        };
        text_without_whitespace(text_str).fmt(f);
    }
}
