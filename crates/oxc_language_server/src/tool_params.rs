use std::sync::Arc;

use tower_lsp_server::ls_types::{CodeActionContext, Range, Uri};

/// Code-action request data passed from the language-server backend to a tool.
pub struct CodeActionParams {
    pub uri: Uri,
    pub range: Range,
    pub context: CodeActionContext,
    /// Content the server holds in memory for `uri`, `None` when no document is open for it.
    pub document: Option<OpenDocument>,
}

/// In-memory content of an open document, with the version the client reported for it.
pub struct OpenDocument {
    pub content: Arc<str>,
    pub version: i32,
}
