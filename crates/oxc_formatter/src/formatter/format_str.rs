//! `Format` impl for `&'static str` specialized to `JsFormatContext`.

use crate::formatter::{Buffer, Format, Formatter, JsFormatContext, builders::token};

// Hardcoded to `JsFormatContext` rather than generic over `C` so the blanket
// `&T where T: Format` doesn't overlap (str doesn't impl Format for any C).
// Uses `Token` (not `Text`) so downstream IR transforms (e.g. `sort_imports`) can match
// on token text shape.
impl<'ast> Format<'ast, JsFormatContext<'ast>> for &'static str {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'ast, JsFormatContext<'ast>>) {
        crate::write!(f, token(self));
    }
}
