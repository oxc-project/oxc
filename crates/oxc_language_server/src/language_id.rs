/// Represents language IDs passed from the client in `textDocument/didOpen` notifications.
///
/// These are used to select the appropriate parser strategy for a given non-file document.
/// To be aligned with the CLI integration, for files the extension should be used.
/// If the document is not a file (protocol not `file://`), or the extension is not known,
/// the language ID is used to select the parser strategy.
/// For non file protocols (e.g. `untitled://`, `vscode-notebook-cell://`, etc.),
/// the language ID is always used to select the parser strategy.
///
/// For a starting list of known language identifiers, see:
/// <https://code.visualstudio.com/docs/languages/identifiers#_known-language-identifiers>
/// Extensions of an editor can also contribute new language identifiers, so this list is not exhaustive.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LanguageId(String);

impl LanguageId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
