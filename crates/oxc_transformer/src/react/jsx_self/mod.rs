use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactJsxSelfOptions;

/// [plugin-transform-react-jsx-self](https://babeljs.io/docs/babel-plugin-transform-react-jsx-self)
///
/// This plugin is included in `preset-react`.
///
/// ## Example
///
/// In: `<sometag />`
/// Out: `<sometag __self={this} />`
#[derive(Debug, Default)]
pub struct ReactJsxSelf {
    #[allow(unused)]
    options: ReactJsxSelfOptions,
}

impl ReactJsxSelf {
    pub fn new(options: ReactJsxSelfOptions) -> Self {
        Self { options }
    }
}
