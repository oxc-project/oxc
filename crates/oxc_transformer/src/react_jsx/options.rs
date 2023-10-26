use serde::Deserialize;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub struct ReactJsxOptions {
    /// Decides which runtime to use.
    pub runtime: ReactJsxRuntime,
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
