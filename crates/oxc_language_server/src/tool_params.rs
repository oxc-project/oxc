use std::sync::Arc;

use tower_lsp_server::gen_lsp_types::{CodeActionContext, Range, Uri};

/// Code-action request data passed from the language-server backend to a tool.
pub struct CodeActionParams {
    pub uri: Uri,
    pub range: Range,
    pub context: CodeActionContext,
    /// Version of the open document, `None` when the document is not open.
    pub document_version: Option<i32>,
    /// Content of the open document at `document_version`, `Some` exactly when that is.
    pub document_text: Option<Arc<str>>,
}
