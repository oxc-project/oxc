use tower_lsp_server::ls_types::Uri;

use crate::{ConcurrentHashMap, LanguageId, TextDocument};

#[derive(Debug, Default)]
pub struct LSPFileSystem {
    files: ConcurrentHashMap<Uri, (LanguageId, String)>,
    /// Tracks the LSP document version for each open file.
    /// Updated on `textDocument/didOpen` and `textDocument/didChange`.
    versions: ConcurrentHashMap<Uri, i32>,
}

impl LSPFileSystem {
    pub fn clear(&self) {
        self.files.pin().clear();
        self.versions.pin().clear();
    }

    pub fn set(&self, uri: Uri, content: String) {
        let language_id = self.get_language_id(&uri).unwrap_or_default();
        self.files.pin().insert(uri, (language_id, content));
    }

    pub fn set_with_language(&self, uri: Uri, language_id: LanguageId, content: String) {
        self.files.pin().insert(uri, (language_id, content));
    }

    /// Store the LSP document version for the given URI.
    pub fn set_version(&self, uri: &Uri, version: i32) {
        self.versions.pin().insert(uri.clone(), version);
    }

    /// Retrieve the last stored LSP document version for the given URI.
    pub fn get_version(&self, uri: &Uri) -> Option<i32> {
        self.versions.pin().get(uri).copied()
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
                text: Some(content.clone()),
            },
        )
    }

    pub fn remove(&self, uri: &Uri) {
        self.files.pin().remove(uri);
        self.versions.pin().remove(uri);
    }

    pub fn keys(&self) -> Vec<Uri> {
        self.files.pin().keys().cloned().collect()
    }
}
