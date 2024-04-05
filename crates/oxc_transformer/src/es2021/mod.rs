use serde::Deserialize;

use crate::options::default_as_true;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Es2021Options {
    /// [logical-assignment-operators](https://babeljs.io/docs/babel-plugin-transform-logical-assignment-operators)
    #[serde(default = "default_as_true")]
    pub logical_assignment_operators: bool,

    /// [numeric-separator](https://babeljs.io/docs/babel-plugin-transform-numeric-separator)
    #[serde(default = "default_as_true")]
    pub numeric_separators: bool,
}

impl Default for Es2021Options {
    fn default() -> Self {
        Self {
            logical_assignment_operators: default_as_true(),
            numeric_separators: default_as_true(),
        }
    }
}
