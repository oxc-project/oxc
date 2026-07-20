use oxc_ast::ast::Expression;
use oxc_span::{SPAN, Span};
use oxc_str::Str;

use crate::{PropertyKeyOrigin, state::TransformState};

pub type TraverseCtx<'a> = oxc_traverse::TraverseCtx<'a, TransformState<'a>>;

/// Type alias for backward compatibility.
pub type TransformCtx<'a> = TransformState<'a>;

/// Record the original syntactic class of a property-key-derived node.
#[track_caller]
pub fn record_property_key_origin(
    span: Span,
    origin: PropertyKeyOrigin,
    ctx: &mut TraverseCtx<'_>,
) {
    debug_assert_ne!(span, SPAN, "property-key provenance requires an original source span");
    let previous = ctx.state.property_key_provenance.insert(span, origin);
    debug_assert!(
        previous.is_none_or(|previous| previous == origin),
        "one source span cannot have conflicting property-key origins"
    );
}

/// Create a string derived from a property key and preserve its original syntactic class.
#[track_caller]
pub fn create_property_key_string<'a, S>(
    span: Span,
    value: S,
    origin: PropertyKeyOrigin,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a>
where
    S: Into<Str<'a>>,
{
    record_property_key_origin(span, origin, ctx);
    Expression::new_string_literal(span, value, None, ctx)
}
