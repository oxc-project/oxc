use tower_lsp_server::gen_lsp_types::{CodeActionContext, Range, Uri};

/// Code-action request data passed from the language-server backend to a tool.
pub struct CodeActionParams {
    pub uri: Uri,
    pub range: Range,
    pub context: CodeActionContext,
    pub is_open_document: bool,
}
