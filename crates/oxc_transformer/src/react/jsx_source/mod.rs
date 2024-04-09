use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactJsxSourceOptions;

/// [plugin-transform-react-jsx-source](https://babeljs.io/docs/babel-plugin-transform-react-jsx-source)
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
