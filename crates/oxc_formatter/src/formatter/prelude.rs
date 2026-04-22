pub use super::{Buffer as _, BufferExtensions, Format, Format as _};
pub use super::{
    JsFormatContext, JsFormatter,
    builders::*,
    format_element::{
        tag::{LabelId, Tag, TagKind},
        *,
    },
    format_extensions::{MemoizeFormat, Memoized},
    formatter::Formatter,
    trivia::{format_dangling_comments, format_leading_comments},
};

use super::builders::FormatWith;

/// JS-specific version of [`format_with`] that fixes the context to [`JsFormatContext`].
///
/// This avoids closure type inference issues when the context type parameter `C` is generic.
/// Use this in JS/TS formatting code instead of `format_with`.
pub fn js_format_with<'ast>(
    formatter: impl Fn(&mut Formatter<'_, 'ast, JsFormatContext<'ast>>),
) -> FormatWith<impl Fn(&mut Formatter<'_, 'ast, JsFormatContext<'ast>>)> {
    FormatWith::new(formatter)
}

/// JS-specific version of [`format_once`] that fixes the context to [`JsFormatContext`].
///
/// This avoids closure type inference issues when the context type parameter `C` is generic.
/// Use this in JS/TS formatting code instead of `format_once`.
pub fn js_format_once<'ast>(
    formatter: impl FnOnce(&mut Formatter<'_, 'ast, JsFormatContext<'ast>>),
) -> super::builders::FormatOnce<impl FnOnce(&mut Formatter<'_, 'ast, JsFormatContext<'ast>>)> {
    super::builders::FormatOnce::new(formatter)
}
