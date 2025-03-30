use crate::is_global_reference::IsGlobalReference;

pub trait MayHaveSideEffectsContext: IsGlobalReference {}
