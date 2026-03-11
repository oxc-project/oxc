mod format_config;
mod to_external_options;
mod to_oxfmt_options;

pub(crate) use format_config::json_deep_merge;
pub use format_config::{
    EndOfLineConfig, FormatConfig, OxfmtOverrideConfig, Oxfmtrc, SortImportsConfig,
    SortPackageJsonUserConfig, SortTailwindcssConfig,
};
pub use to_external_options::{finalize_external_options, sync_external_options};
pub use to_oxfmt_options::{OxfmtOptions, to_oxfmt_options};
