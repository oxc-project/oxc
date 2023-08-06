use std::path::{Path, PathBuf};

/// The final path resolution with optional `?query` and `#fragment`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Resolution {
    pub(crate) path: PathBuf,

    /// path query `?query`, contains `?`.
    pub(crate) query: Option<String>,

    /// path fragment `#query`, contains `#`.
    pub(crate) fragment: Option<String>,
}

impl Resolution {
    /// Returns the path without query and fragment
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the path without query and fragment
    pub fn into_path_buf(self) -> PathBuf {
        self.path
    }

    /// Returns the path query `?query`, contains the leading `?`
    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    /// Returns the path fragment `#fragment`, contains the leading `#`
    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }

    /// Returns the full path with query and fragment
    pub fn full_path(self) -> PathBuf {
        let mut path = self.path.into_os_string();
        if let Some(query) = &self.query {
            path.push(query);
        }
        if let Some(fragment) = &self.fragment {
            path.push(fragment);
        }
        PathBuf::from(path)
    }
}
