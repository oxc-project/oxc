use crate::state::TransformState;

pub type TraverseCtx<'a> = oxc_traverse::TraverseCtx<'a, TransformState<'a>>;

/// Type alias for backward compatibility.
pub type TransformCtx<'a> = TransformState<'a>;
