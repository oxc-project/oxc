use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ES2020Options {
    #[serde(skip)]
    pub nullish_coalescing_operator: bool,
}

impl ES2020Options {
    #[must_use]
    pub fn with_nullish_coalescing_operator(mut self, enable: bool) -> Self {
        self.nullish_coalescing_operator = enable;
        self
    }
}
