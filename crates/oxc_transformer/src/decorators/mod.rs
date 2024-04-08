use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecoratorsOptions {
    pub version: DecoratorVersion,
}

/// Only "2023-11" will be implemented because Babel 8 will only support "2023-11" and "legacy".
#[derive(Debug, Default, Clone, Deserialize)]
pub enum DecoratorVersion {
    November2023, // 2023-11
    #[default]
    Legacy,
}

/// [proposal-decorators](https://babeljs.io/docs/babel-plugin-proposal-decorators)
#[derive(Debug, Default)]
pub struct Decorators {
    #[allow(unused)]
    options: DecoratorsOptions,
}

impl Decorators {
    #[allow(unused)]
    pub fn new(options: DecoratorsOptions) -> Self {
        Self { options }
    }
}
