use serde::Deserialize;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
/// ES2026 transform options.
pub struct ES2026Options {
    /// Enable explicit resource management transform (`using` declarations).
    #[serde(skip)]
    pub explicit_resource_management: bool,
}
