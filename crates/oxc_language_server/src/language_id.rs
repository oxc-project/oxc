/// Represents language IDs passed from the client in `textDocument/didOpen` notifications.
///
/// These are used to select the appropriate parser strategy for a given file.
/// The language ID should be preferred over the file extension for determining the language of the content.
/// The tool can fall back to using the file extension or other heuristics, if the language ID is unrecognized or unsupported.
/// Files like `.oxlintrc.json` can support comments like jsonc.
/// The editor/IDE should know the preferred language for such files, and send the appropriate language ID to the server.
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
