use serde::Deserialize;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
/// ES2016 transform options.
pub struct ES2016Options {
    /// Enable exponentiation operator transform.
    #[serde(skip)]
    pub exponentiation_operator: bool,
}
