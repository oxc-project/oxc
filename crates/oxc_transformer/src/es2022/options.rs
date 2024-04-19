use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Es2022Options {
    #[serde(skip)]
    pub class_static_block_plugin: bool,
}

impl Default for Es2022Options {
    fn default() -> Self {
        Self { class_static_block_plugin: true }
    }
}
