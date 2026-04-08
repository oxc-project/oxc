use std::{ffi::OsStr, path::Path};

pub(super) fn is_css_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == OsStr::new("css"))
}
