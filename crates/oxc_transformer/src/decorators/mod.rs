use serde::Deserialize;

/// Only `"2023-11"` will be implemented because Babel 8 will only support "2023-11" and "legacy".
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecoratorsOptions;

/// [proposal-decorators](https://babeljs.io/docs/babel-plugin-proposal-decorators)
#[derive(Debug, Default)]
pub struct Decorators {
    #[allow(unused)]
    options: DecoratorsOptions,
}

impl Decorators {
    pub fn new(options: DecoratorsOptions) -> Self {
        Self { options }
    }
}
