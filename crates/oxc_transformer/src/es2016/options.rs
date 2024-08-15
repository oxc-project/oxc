use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ES2016Options {
    #[serde(skip)]
    pub exponentiation_operator: bool,
}

impl ES2016Options {
    #[must_use]
    pub fn with_exponentiation_operator(mut self, enable: bool) -> Self {
        self.exponentiation_operator = enable;
        self
    }
}
