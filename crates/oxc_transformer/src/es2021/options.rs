use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ES2021Options {
    #[serde(skip)]
    pub logical_assignment_operators: bool,
}

impl ES2021Options {
    #[must_use]
    pub fn with_logical_assignment_operators(mut self, enable: bool) -> Self {
        self.logical_assignment_operators = enable;
        self
    }
}
