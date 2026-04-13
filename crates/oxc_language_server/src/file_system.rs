#[cfg(all(test, target_os = "windows"))]
use cow_utils::CowUtils;
#[cfg(test)]
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tower_lsp_server::ls_types::Uri;

use crate::{ConcurrentHashMap, LanguageId, TextDocument};

#[derive(Debug, Default)]
pub struct LSPFileSystem {
    files: ConcurrentHashMap<Uri, (LanguageId, Arc<str>)>,
}

/// Represents a resolved file path that can be used for file system operations.
///
/// On Windows and macOS, the file system is case-insensitive, so we need to resolve the path to its canonical form.
/// Or else we might not find the right file or workspace worker.
/// On other platforms, we can just use the path as is, since the file system is case-sensitive.
///
/// VS Code as an example likes to send mixed URI styles within the same connection:
/// - workspace: `file:///c:/Path/To/file.js`
/// - lint/format: `file:///C:/path/to/file.js`
#[cfg(test)]
struct ResolvedPath(PathBuf);

#[cfg(test)]
impl From<PathBuf> for ResolvedPath {
    fn from(path: PathBuf) -> Self {
        Self::canonical(path)
    }
}

#[cfg(test)]
impl TryFrom<Uri> for ResolvedPath {
    type Error = String;

    fn try_from(uri: Uri) -> Result<Self, Self::Error> {
        let path = uri.to_file_path().ok_or_else(|| "Invalid URI".to_string())?;
        Ok(Self::canonical(path.to_path_buf()))
    }
}

#[cfg(test)]
impl ResolvedPath {
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Resolves the given path to its canonical form if necessary, based on the operating system.
    fn canonical(path: PathBuf) -> Self {
        // on non windows and non macos, we can just return the path as is,
        // since we don't have to worry about case sensitivity.
        // Note: it really depends on the disk / folder settings, but checking the OS is a good enough heuristic for now,
        // and we can always add more heuristics later if needed.
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        {
            Self(path)
        }

        // on windows and macos, we need to resolve the path to its canonical form,
        // since the URI does not respect case sensitivity, but the file system does
        #[cfg(target_os = "macos")]
        {
            // if the path does not exist, we can just return it as is
            std::fs::canonicalize(&path).map_or_else(|_| Self(path), Self)
        }

        // on windows we need to remove `\\?\` prefix if it exists, since it can cause issues with some file system operations
        #[cfg(target_os = "windows")]
        {
            let path = std::fs::canonicalize(&path).unwrap_or_else(|_| path);
            let path_str = path.to_string_lossy();
            Self(PathBuf::from(path_str.cow_replace(r"\\?\", "").as_ref()))
        }
    }
}

impl LSPFileSystem {
    pub fn clear(&self) {
        self.files.pin().clear();
    }

    pub fn set(&self, uri: Uri, content: String) {
        let language_id = self.get_language_id(&uri).unwrap_or_default();
        self.files.pin().insert(uri, (language_id, Arc::from(content)));
    }

    pub fn set_with_language(&self, uri: Uri, language_id: LanguageId, content: String) {
        self.files.pin().insert(uri, (language_id, Arc::from(content)));
    }

    pub fn get_language_id(&self, uri: &Uri) -> Option<LanguageId> {
        self.files.pin().get(uri).map(|(lang, _)| lang.clone())
    }

    pub fn get_document<'a>(&self, uri: &'a Uri) -> TextDocument<'a> {
        self.files.pin().get(uri).map_or_else(
            || TextDocument { uri, language_id: LanguageId::default(), text: None },
            |(language_id, content)| TextDocument {
                uri,
                language_id: language_id.clone(),
                text: Some(Arc::clone(content)),
            },
        )
    }

    pub fn remove(&self, uri: &Uri) {
        self.files.pin().remove(uri);
    }

    pub fn keys(&self) -> Vec<Uri> {
        self.files.pin().keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use cow_utils::CowUtils;
    use std::{borrow::Cow, path::Path};

    use super::*;

    fn path_from_fixture(fixture: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures").join(fixture)
    }

    fn path_to_lossy_string(path: &Path) -> Cow<'_, str> {
        path.as_os_str().to_string_lossy()
    }

    #[test]
    fn test_uri_to_resolved_path_file() {
        let dir = path_from_fixture("same_path_different_uri");
        let file = dir.join("test.txt");

        let uri = Uri::from_file_path(&file).unwrap();
        let unresolved_uri = Uri::from_file_path(dir.join("Test.txt")).unwrap();

        let resolved_path = ResolvedPath::try_from(uri).unwrap();
        let unresolved_path = ResolvedPath::try_from(unresolved_uri).unwrap();

        assert_eq!(*resolved_path.as_path(), file);

        #[cfg(any(target_os = "windows", target_os = "macos"))]
        assert_eq!(path_to_lossy_string(unresolved_path.as_path()), path_to_lossy_string(&file));

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        assert_eq!(
            path_to_lossy_string(unresolved_path.as_path()),
            path_to_lossy_string(&file).cow_replace("test.txt", "Test.txt")
        );
    }

    #[test]
    fn test_uri_to_resolved_path_dir() {
        let dir = path_from_fixture("same_path_different_uri");
        let unresolved_dir = path_to_lossy_string(&dir);
        let unresolved_dir =
            unresolved_dir.cow_replace("same_path_different_uri", "Same_Path_Different_URI");

        let uri = Uri::from_file_path(&dir).unwrap();
        let unresolved_uri = Uri::from_file_path(unresolved_dir.as_ref()).unwrap();

        let resolved_path = ResolvedPath::try_from(uri).unwrap();
        let unresolved_path = ResolvedPath::try_from(unresolved_uri).unwrap();

        assert_eq!(*resolved_path.as_path(), dir);

        #[cfg(any(target_os = "windows", target_os = "macos"))]
        assert_eq!(path_to_lossy_string(unresolved_path.as_path()), path_to_lossy_string(&dir));

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        assert_eq!(
            path_to_lossy_string(unresolved_path.as_path()),
            path_to_lossy_string(&dir)
                .cow_replace("same_path_different_uri", "Same_Path_Different_URI")
        );
    }
}
