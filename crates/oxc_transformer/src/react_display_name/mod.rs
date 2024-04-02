use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactDisplayNameOptions;

/// [plugin-transform-react-display-name](https://babeljs.io/docs/babel-plugin-transform-react-display-name)
///
/// This plugin is included in `preset-react`.
///
/// ## Example
///
/// In: `var bar = createReactClass({});`
/// Out: `var bar = createReactClass({ displayName: "bar" });`
#[derive(Debug, Default)]
pub struct ReactDisplayName {
    #[allow(unused)]
    options: ReactDisplayNameOptions,
}

impl ReactDisplayName {
    pub fn new(options: ReactDisplayNameOptions) -> Self {
        Self { options }
    }
}
