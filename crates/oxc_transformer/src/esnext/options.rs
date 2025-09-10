use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ESNextOptions {
    #[serde(skip)]
    pub explicit_resource_management: bool,
}

impl Default for ESNextOptions {
    fn default() -> Self {
        Self {
            explicit_resource_management: true, // Default to transform
        }
    }
}