use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct DecoratorOptions {
    #[serde(skip)]
    // TODO: true for debug
    #[serde(default = "default_as_true")]
    pub legacy: bool,
}

impl Default for DecoratorOptions {
    fn default() -> Self {
        Self { legacy: true }
    }
}

fn default_as_true() -> bool {
    true
}
