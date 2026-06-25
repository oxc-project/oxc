use serde::Deserialize;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
/// ES2017 transform options.
pub struct ES2017Options {
    /// Enable async-to-generator transform.
    #[serde(skip)]
    pub async_to_generator: bool,
}
