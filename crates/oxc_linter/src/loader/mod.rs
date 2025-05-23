use std::{error::Error, fmt, path::Path};

use oxc_span::SourceType;

mod partial_loader;
mod source;
pub use partial_loader::{LINT_PARTIAL_LOADER_EXTENSIONS, LINTABLE_EXTENSIONS, PartialLoader};
pub use source::JavaScriptSource;

// TODO: use oxc_resolver::FileSystem. We can't do so until that crate exposes FileSystemOs
// externally.
#[derive(Default, Clone)]
pub struct Loader;

impl Loader {
    pub fn can_load<P: AsRef<Path>>(path: P) -> bool {
        let path = path.as_ref();
        SourceType::from_path(path).is_ok()
            || path
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .is_some_and(|ext| LINT_PARTIAL_LOADER_EXTENSIONS.contains(&ext))
    }
}

#[derive(Debug, Clone)]
pub enum LoadError {
    TooLarge,
    NoExtension,
    UnsupportedFileType(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => f.write_str("file is too large. Only files up to 4GB are supported."),
            Self::NoExtension => f.write_str("no extension"),
            Self::UnsupportedFileType(ext) => write!(f, "unsupported file type: {ext}"),
        }
    }
}

impl Error for LoadError {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_loader_can_handle() {
        let paths = [
            "foo.js",
            "foo.jsx",
            "foo.mjs",
            "foo.cjs",
            "foo.ts",
            "foo.tsx",
            "foo.mts",
            "foo.cts",
            "foo.d.ts",
            "foo.d.tsx",
            "foo.d.mts",
            "foo.d.cts",
            "foo.astro",
            "foo.svelte",
            "foo.vue",
        ];

        for path in paths {
            assert!(Loader::can_load(path));
        }
    }
}
