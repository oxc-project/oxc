use serde::Deserialize;

use crate::options::default_as_true;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Es2022Options {
    /// [class-properties](https://babeljs.io/docs/babel-plugin-transform-class-properties)
    #[serde(default = "default_as_true")]
    pub class_properties: bool,

    /// [class-static-block](https://babeljs.io/docs/babel-plugin-transform-class-static-block)
    #[serde(default = "default_as_true")]
    pub class_static_block: bool,

    /// [private-methods](https://babeljs.io/docs/babel-plugin-transform-private-methods)
    #[serde(default = "default_as_true")]
    pub class_private_methods: bool,

    /// [private-property-in-object](https://babeljs.io/docs/babel-plugin-transform-private-property-in-object)
    #[serde(default = "default_as_true")]
    pub class_private_properties: bool,
}

impl Default for Es2022Options {
    fn default() -> Self {
        Self {
            class_properties: default_as_true(),
            class_static_block: default_as_true(),
            class_private_methods: default_as_true(),
            class_private_properties: default_as_true(),
        }
    }
}
