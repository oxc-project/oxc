//! Port of typescript-go's `internal/core` package (the parts the checker needs).

mod compileroptions;
mod parsedoptions;

pub(crate) use compileroptions::for_each_compiler_option;
pub use compileroptions::{
    CompilerOptions, CompilerOptionsPathsMap, JsxEmit, ModuleDetectionKind, ModuleKind,
    ModuleResolutionKind, NewLineKind, ScriptTarget,
};
pub use parsedoptions::ParsedOptions;
