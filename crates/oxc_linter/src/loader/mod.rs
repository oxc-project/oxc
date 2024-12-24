use std::{error::Error, fmt, path::Path};

use oxc_span::SourceType;

mod partial_loader;
mod source;
pub use partial_loader::{PartialLoader, LINT_PARTIAL_LOADER_EXT};
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
                .is_some_and(|ext| LINT_PARTIAL_LOADER_EXT.contains(&ext))
    }

    /// # Errors
    /// - If the file is too large (> 4GB, or u32::MAX)
    /// - If the file has no extension
    /// - If the file extension is not supported
    pub fn load_str<'a, P: AsRef<Path>>(
        &self,
        path: P,
        source_text: &'a str,
    ) -> Result<Vec<JavaScriptSource<'a>>, LoadError> {
        if source_text.len() > u32::MAX as usize {
            return Err(LoadError::TooLarge);
        }

        let path = path.as_ref();
        let ext = path.extension().ok_or(LoadError::NoExtension)?;
        // file extension is not unicode, we definitely don't support it.
        let ext = ext.to_str().ok_or_else(|| LoadError::unsupported(ext))?;

        // let source_type = SourceType::from_path(path);
        if let Ok(source_type) = SourceType::from_path(path) {
            Ok(vec![JavaScriptSource::new(source_text, source_type)])
        } else {
            let partial = PartialLoader::parse(ext, source_text);
            partial.ok_or_else(|| LoadError::UnsupportedFileType(ext.to_string()))
        }
    }
}

#[derive(Debug, Clone)]
pub enum LoadError {
    TooLarge,
    NoExtension,
    UnsupportedFileType(String),
}

impl LoadError {
    pub(super) fn unsupported(ext: &std::ffi::OsStr) -> Self {
        Self::UnsupportedFileType(ext.to_string_lossy().to_string())
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => write!(f, "file is too large. Only files up to 4GB are supported."),
            Self::NoExtension => write!(f, "no extension"),
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
