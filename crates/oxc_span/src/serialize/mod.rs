use oxc_ast_macros::ast_meta;
use oxc_estree::{ESTree, Serializer};

use crate::Span;

/// Deserialized as `i32` in raw transfer.
///
/// Source text is limited to 1 GiB in raw transfer, so `i32` can represent full range of possible values,
/// and V8 handles `i32`s better as they can be stored inline in structs as SMIs.
#[ast_meta]
#[estree(ts_type = "number", raw_deser = "DESER[i32](POS_OFFSET.start)", raw_deser_inline)]
pub struct SpanStart<'b>(pub &'b Span);

impl ESTree for SpanStart<'_> {
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) {
        self.0.start.serialize(serializer);
    }
}

/// Deserialized as `i32` in raw transfer.
///
/// Source text is limited to 1 GiB in raw transfer, so `i32` can represent full range of possible values,
/// and V8 handles `i32`s better as they can be stored inline in structs as SMIs.
#[ast_meta]
#[estree(ts_type = "number", raw_deser = "DESER[i32](POS_OFFSET.end)", raw_deser_inline)]
pub struct SpanEnd<'b>(pub &'b Span);

impl ESTree for SpanEnd<'_> {
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) {
        self.0.end.serialize(serializer);
    }
}
