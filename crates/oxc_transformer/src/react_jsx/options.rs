use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactJsxOptions {
    /// Decides which runtime to use.
    pub runtime: ReactJsxRuntime,
    /// Toggles whether or not to throw an error if an XML namespaced tag name is used. e.g. `<f:image />`
    /// Though the JSX spec allows this, it is disabled by default since React's JSX does not currently have support for it.
    pub throw_if_namespace: Option<bool>,
    /// Replaces the import source when importing functions. default to `react`
    pub import_source: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReactJsxRuntime {
    /// Does not automatically import anything (default).
    #[default]
    Classic,
    /// Auto imports the functions that JSX transpiles to.
    Automatic,
}

impl ReactJsxRuntime {
    pub fn is_classic(&self) -> bool {
        matches!(self, Self::Classic)
    }

    pub fn is_automatic(&self) -> bool {
        matches!(self, Self::Automatic)
    }
}
