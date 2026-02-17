/// Represents language IDs passed from the client in `textDocument/didOpen` notifications.
///
/// These are used to select the appropriate parser strategy for a given file.
/// It is the tool's responsibility to use the correct parser strategy for a file
/// based on its file extension, but newly created files may not have an extension,
/// so we rely on the language ID to determine which parser strategy to use.
///
/// For a more complete list of known language identifiers, see:
/// <https://code.visualstudio.com/docs/languages/identifiers#_known-language-identifiers>
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
