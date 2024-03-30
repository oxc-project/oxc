use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactJsxSourceOptions;

/// [plugin-transform-react-jsx](https://babeljs.io/docs/babel-plugin-transform-react-jsx)
///
/// This plugin generates production-ready JS code.
///
/// If you are developing a React app in a development environment,
/// please use @babel/plugin-transform-react-jsx-development for a better debugging experience.
///
/// This plugin is included in `preset-react`.
///
/// ## Example
///
/// In: `<sometag />`
/// Out: `<sometag __source={ { fileName: 'this/file.js', lineNumber: 10, columnNumber: 1 } } />`
#[derive(Debug, Default)]
pub struct ReactJsxSource {
    #[allow(unused)]
    options: ReactJsxSourceOptions,
}

impl ReactJsxSource {
    pub fn new(options: ReactJsxSourceOptions) -> Self {
        Self { options }
    }
}
