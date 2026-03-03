mod binding;
mod cycles;
mod dynamic_exports;
mod exec_order;
mod exports_kind;
mod safely_merge_cjs_ns;
mod side_effects;
mod tla;
mod wrapping;

pub use binding::match_imports_collect;
pub use binding::{
    BindingError, ResolvedExportsMap, bind_imports_and_exports, build_resolved_exports,
    match_imports,
};
pub use cycles::find_cycles;
pub use dynamic_exports::compute_has_dynamic_exports;
pub use exec_order::{ExecOrderConfig, ExecOrderResult, compute_exec_order};
pub use exports_kind::{ExportsKindConfig, ExportsKindResult, determine_module_exports_kind};
pub use safely_merge_cjs_ns::{SafelyMergeCjsNsInfo, determine_safely_merge_cjs_ns};
pub use side_effects::determine_side_effects;
pub use tla::compute_tla;
pub use wrapping::{WrapModulesConfig, WrapModulesResult, wrap_modules};
