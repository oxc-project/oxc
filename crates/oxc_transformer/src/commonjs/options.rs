use crate::commonjs::types::import_interop::ImportInterop;
use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct CommonjsOptions {
    #[serde(skip)]
    pub import_interop: ImportInterop,
    #[serde(skip)]
    pub loose: bool,
    #[serde(skip)]
    pub strict: bool,
    #[serde(skip)]
    pub transform_import_and_export: bool,
}
