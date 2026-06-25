use serde::Deserialize;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
/// ES2021 transform options.
pub struct ES2021Options {
    /// Enable logical assignment operator transform.
    #[serde(skip)]
    pub logical_assignment_operators: bool,
}
