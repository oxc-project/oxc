use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ES2019Options {
    #[serde(skip)]
    pub optional_catch_binding: bool,
}

impl ES2019Options {
    #[must_use]
    pub fn with_optional_catch_binding(mut self, enable: bool) -> Self {
        self.optional_catch_binding = enable;
        self
    }
}
