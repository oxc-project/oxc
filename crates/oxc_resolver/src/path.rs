//! Path Utilities
//!
//! Code adapted from the following libraries
//! * [path-absolutize](https://docs.rs/path-absolutize)
//! * [normalize_path](https://docs.rs/normalize-path)
use std::path::{Component, Path, PathBuf};

/// Extension trait to add path normalization to std's [`Path`].
pub trait PathUtil {
    /// Normalize this path without performing I/O.
    ///
    /// All redundant separator and up-level references are collapsed.
    ///
    /// However, this does not resolve links.
    fn normalize(&self) -> PathBuf;

    /// Normalize with subpath assuming this path is normalized without performing I/O.
    ///
    /// All redundant separator and up-level references are collapsed.
    ///
    /// However, this does not resolve links.
    fn normalize_with<P: AsRef<Path>>(&self, subpath: P) -> PathBuf;
}

impl PathUtil for Path {
    fn normalize(&self) -> PathBuf {
        let mut components = self.components().peekable();
        let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek() {
            let buf = PathBuf::from(c.as_os_str());
            components.next();
            buf
        } else {
            PathBuf::new()
        };

        for component in components {
            match component {
                Component::Prefix(..) => unreachable!(),
                Component::RootDir => {
                    ret.push(component.as_os_str());
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    ret.pop();
                }
                Component::Normal(c) => {
                    ret.push(c);
                }
            }
        }

        ret
    }

    fn normalize_with<B: AsRef<Self>>(&self, subpath: B) -> PathBuf {
        let subpath = subpath.as_ref();
        let mut components = subpath.components().peekable();
        if subpath.is_absolute() || matches!(components.peek(), Some(Component::Prefix(..))) {
            return subpath.to_path_buf();
        }

        let mut ret = self.to_path_buf();
        for component in subpath.components() {
            match component {
                Component::Prefix(..) | Component::RootDir => unreachable!(),
                Component::CurDir => {}
                Component::ParentDir => {
                    ret.pop();
                }
                Component::Normal(c) => {
                    ret.push(c);
                }
            }
        }

        ret
    }
}
