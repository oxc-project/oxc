use std::path::Path;

use ignore::{DirEntry, WalkBuilder};
use oxc_ast::VALID_EXTENSIONS;

use crate::CliOptions;

pub struct Walk {
    inner: ignore::Walk,
}

impl Walk {
    pub fn new<P: AsRef<Path>>(path: P, options: &CliOptions) -> Self {
        let inner = WalkBuilder::new(path)
            .add_custom_ignore_filename(&options.ignore_path)
            .ignore(false)
            .git_global(false)
            .build();
        Self { inner }
    }

    pub fn iter(self) -> impl Iterator<Item = Box<Path>> {
        self.inner
            .filter_map(Result::ok)
            .filter(Self::is_wanted_entry)
            .map(|entry| entry.path().to_path_buf().into_boxed_path())
    }

    fn is_wanted_entry(dir_entry: &DirEntry) -> bool {
        let Some(file_type) = dir_entry.file_type() else { return false };
        if !file_type.is_file() {
            return false;
        }
        let Some(extension) = dir_entry.path().extension() else { return false };
        VALID_EXTENSIONS.contains(&extension.to_string_lossy().as_ref())
    }
}
