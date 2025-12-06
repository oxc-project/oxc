use tower_lsp_server::lsp_types::Uri;

use crate::ConcurrentHashMap;

#[derive(Debug, Default)]
pub struct LSPFileSystem {
    files: ConcurrentHashMap<Uri, String>,
}

impl LSPFileSystem {
    pub fn clear(&self) {
        self.files.pin().clear();
    }

    pub fn set(&self, uri: Uri, content: String) {
        self.files.pin().insert(uri, content);
    }

    pub fn get(&self, uri: &Uri) -> Option<String> {
        self.files.pin().get(uri).cloned()
    }

    pub fn remove(&self, uri: &Uri) {
        self.files.pin().remove(uri);
    }

    pub fn keys(&self) -> Vec<Uri> {
        self.files.pin().keys().cloned().collect()
    }
}
