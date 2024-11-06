use serde::Deserialize;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub enum ModuleTarget {
    CJS,
    AMD,
    UMD,

    #[default]
    Preserve,
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ModuleOptions {
    pub target: ModuleTarget,
}
