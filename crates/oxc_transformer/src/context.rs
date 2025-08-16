use crate::state::TransformState;

pub type TraverseCtx<'a> = oxc_traverse::TraverseCtx<'a, TransformState<'a>>;
