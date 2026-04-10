use std::sync::Arc;

use tower_lsp_server::ls_types::Uri;

use crate::{ConcurrentHashMap, LanguageId, TextDocument};

#[derive(Debug, Default)]
pub struct LSPFileSystem {
    files: ConcurrentHashMap<Uri, (LanguageId, Arc<str>)>,
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
