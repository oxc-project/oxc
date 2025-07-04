mod inject_global_variables;
mod module_runner_transform;
mod replace_global_defines;

pub use inject_global_variables::*;
pub use module_runner_transform::*;
pub use replace_global_defines::*;

type TraverseCtx<'a> = oxc_traverse::TraverseCtx<'a, ()>;
