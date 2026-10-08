// Port of internal/utils/overlay_vfs.go / cmd/tsgolint/overlayfs.go: an FS whose listed files have
// in-memory contents (payload `source_overrides`, and the rule tester's virtual files).

use std::sync::Arc;
use std::time::SystemTime;

use rustc_hash::FxHashMap;
use tsrs_core::tspath;
use tsrs_vfs::{Entries, FS, FileInfo};

pub struct OverlayFS {
    base: Arc<dyn FS>,
    files: FxHashMap<String, String>,
}

impl OverlayFS {
    pub fn new(base: Arc<dyn FS>, files: FxHashMap<String, String>) -> OverlayFS {
        let files = files.into_iter().map(|(k, v)| (tspath::normalize_path(&k), v)).collect();
        OverlayFS { base, files }
    }
}

impl FS for OverlayFS {
    fn use_case_sensitive_file_names(&self) -> bool {
        self.base.use_case_sensitive_file_names()
    }
    fn file_exists(&self, path: &str) -> bool {
        self.files.contains_key(path) || self.base.file_exists(path)
    }
    fn read_file(&self, path: &str) -> Option<String> {
        if let Some(s) = self.files.get(path) {
            return Some(s.clone());
        }
        self.base.read_file(path)
    }
    fn write_file(&self, path: &str, data: &str) -> Result<(), String> {
        self.base.write_file(path, data)
    }
    fn append_file(&self, path: &str, data: &str) -> Result<(), String> {
        self.base.append_file(path, data)
    }
    fn remove(&self, path: &str) -> Result<(), String> {
        self.base.remove(path)
    }
    fn chtimes(&self, path: &str, a: SystemTime, m: SystemTime) -> Result<(), String> {
        self.base.chtimes(path, a, m)
    }
    fn directory_exists(&self, path: &str) -> bool {
        if self.base.directory_exists(path) {
            return true;
        }
        let prefix = format!("{}/", path.trim_end_matches('/'));
        self.files.keys().any(|k| k.starts_with(&prefix))
    }
    fn get_accessible_entries(&self, path: &str) -> Entries {
        let mut entries = self.base.get_accessible_entries(path);
        let dir = path.trim_end_matches('/');
        // Sorted, not in hash order: a tsconfig's include globs list files in entry order.
        let mut names: Vec<String> = self
            .files
            .keys()
            .filter(|k| tspath::get_directory_path(k) == dir)
            .map(|k| tspath::get_base_file_name(k))
            .collect();
        names.sort_unstable();
        for name in names {
            if !entries.files.contains(&name) {
                entries.files.push(name);
            }
        }
        entries
    }
    fn stat(&self, path: &str) -> Option<FileInfo> {
        self.base.stat(path)
    }
    fn realpath(&self, path: &str) -> String {
        if self.files.contains_key(path) {
            return path.to_string();
        }
        self.base.realpath(path)
    }
}
