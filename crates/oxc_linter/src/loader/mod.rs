use std::{error::Error, fmt, path::Path};

mod partial_loader;
mod source;
use oxc_span::VALID_EXTENSIONS;
pub use partial_loader::{LINT_PARTIAL_LOADER_EXTENSIONS, PartialLoader};
pub use source::JavaScriptSource;

// TODO: use oxc_resolver::FileSystem. We can't do so until that crate exposes FileSystemOs
// externally.
#[derive(Default, Clone)]
pub struct Loader;

/// File extensions that has similar syntax or based on JS/TS, (e.g. Vue SFCs)
/// and can be transformed into JS/TS(X) using a specific loader.
pub const LINT_TRANSFORM_LOADER_EXTENSIONS: &[&str] = &["vue"];

/// All valid JavaScript/TypeScript extensions, plus additional framework files that
/// contain JavaScript/TypeScript code in them (e.g., Astro, Svelte, etc.).
pub const LINTABLE_EXTENSIONS: &[&str] = constcat::concat_slices!([&str]: VALID_EXTENSIONS, LINT_TRANSFORM_LOADER_EXTENSIONS, LINT_PARTIAL_LOADER_EXTENSIONS);

impl Loader {
    pub fn can_load<P: AsRef<Path>>(path: P) -> bool {
        let path = path.as_ref();
        path.extension()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|ext| LINTABLE_EXTENSIONS.contains(&ext))
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
