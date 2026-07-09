//! Port of typescript-go's `internal/core/projectreference.go`.

use std::path::PathBuf;

use crate::tspath::file_extension_is;

/// tsgo `core.ProjectReference`: one entry of a tsconfig's `references` array.
#[derive(Debug)]
pub struct ProjectReference {
    /// The referenced project, resolved against the root config's directory (tsgo `Path`).
    pub path: PathBuf,
    /// The reference's `path` as written in the config (tsgo `OriginalPath`).
    pub original_path: String,
    /// The reference's `circular` flag (tsgo `Circular`).
    pub circular: bool,
}

impl ProjectReference {
    /// tsgo `ResolveProjectReferencePath` / `ResolveConfigFileNameOfProjectReference`: the
    /// config file a reference names — [`Self::path`] itself when it points at a `.json`
    /// file, otherwise the `tsconfig.json` inside the referenced directory.
    pub fn resolve_project_reference_path(&self) -> PathBuf {
        if file_extension_is(&self.path.to_string_lossy(), ".json") {
            self.path.clone()
        } else {
            self.path.join("tsconfig.json")
        }
    }
}
