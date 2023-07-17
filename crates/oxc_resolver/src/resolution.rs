use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Resolution {
    pub(crate) path: PathBuf,

    /// path query `?query`, contains `?`.
    pub(crate) query: Option<String>,

    /// path fragment `#query`, contains `#`.
    pub(crate) fragment: Option<String>,
}

impl Resolution {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn into_path_buf(self) -> PathBuf {
        self.path
    }

    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }
}
