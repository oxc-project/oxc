use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ES2022Options {
    #[serde(skip)]
    pub class_static_block: bool,
}
