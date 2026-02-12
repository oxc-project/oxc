use serde::Deserialize;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
/// ES2019 transform options.
pub struct ES2019Options {
    /// Enable optional catch binding transform.
    #[serde(skip)]
    pub optional_catch_binding: bool,
}
