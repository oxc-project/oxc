use tower_lsp_server::{
    jsonrpc::ErrorCode,
    ls_types::{
        CodeActionContext, CodeActionOrCommand, Diagnostic, Pattern, Range, ServerCapabilities,
        TextEdit, Uri, WorkspaceEdit,
    },
};

use crate::{TextDocument, capabilities::Capabilities};

pub trait ToolBuilder: Send + Sync {
    /// Modify the server capabilities to include capabilities provided by this tool.
    fn server_capabilities(
        &self,
        _capabilities: &mut ServerCapabilities,
        _backend_capabilities: &mut Capabilities,
    ) {
    }

    /// Build a boxed instance of the tool for the given root URI and options.
    fn build_boxed(&self, root_uri: &Uri, options: serde_json::Value) -> Box<dyn Tool>;

    /// Shutdown hook for the tool. Implementors may perform any necessary cleanup here.
    fn shutdown(&self, _root_uri: &Uri) {
        // Default implementation does nothing.
    }
}

pub type DiagnosticResult = Result<Vec<(Uri, Vec<Diagnostic>)>, String>;

pub trait Tool: Send + Sync {
    /// The Server has new configuration changes.
    /// Returns a [ToolRestartChanges] indicating what changes were made for the Tool.
    fn handle_configuration_change(
        &self,
        builder: &dyn ToolBuilder,
        root_uri: &Uri,
        old_options_json: &serde_json::Value,
        new_options_json: serde_json::Value,
    ) -> ToolRestartChanges;

    /// Get the file watcher patterns for this tool based on the provided options.
    /// These patterns will be used to watch for file changes relevant to the tool.
    fn get_watcher_patterns(&self, options: serde_json::Value) -> Vec<Pattern>;

    /// Handle a watched file change event for the given URI.
    /// Returns a [ToolRestartChanges] indicating what changes were made for the Tool.
    /// The Tool should decide whether it needs to restart or take any action based on the URI.
    fn handle_watched_file_change(
        &self,
        builder: &dyn ToolBuilder,
        changed_uri: &Uri,
        root_uri: &Uri,
        options: serde_json::Value,
    ) -> ToolRestartChanges;

    /// Tries to execute the given command with the provided arguments.
    /// If the command is not recognized, returns `Err(ErrorCode)`.
    /// If the command is recognized and executed it can return:
    /// - `Ok(Some(WorkspaceEdit))` if the command was executed successfully and produced a workspace edit.
    /// - `Ok(None)` if the command was executed successfully but did not produce any workspace edit.
    ///
    /// # Errors
    /// If there was an error executing the command, returns an `Err(ErrorCode)`.
    fn execute_command(
        &self,
        _command: &str,
        _arguments: Vec<serde_json::Value>,
    ) -> Result<Option<WorkspaceEdit>, ErrorCode> {
        Err(ErrorCode::InvalidParams)
    }

    /// Get code actions or commands provided by this tool for the given URI and range.
    /// The tool should filter the code actions based on the provided range.
    /// The context can be used to further filter the code actions,
    /// for example by the `only` field which indicates that only code actions of certain kinds are requested.
    fn get_code_actions_or_commands(
        &self,
        _uri: &Uri,
        _range: &Range,
        _context: &CodeActionContext,
    ) -> Vec<CodeActionOrCommand> {
        Vec::new()
    }

    /// Format the given text document.
    ///
    /// Implementors should use `document.text` as the source to format, and may use
    /// `document.uri` and `document.language_id` to determine how to format the content.
    /// Returns a vector of `TextEdit` representing the formatting changes.
    ///
    /// Not all tools will implement formatting, so the default implementation returns an empty vector.
    ///
    /// # Errors
    /// Return [`Err`] when an error occurs; ignoring formatting should return [`Ok`] with an empty vector.
    fn run_format(&self, _document: &TextDocument) -> Result<Vec<TextEdit>, String> {
        Ok(Vec::new())
    }

    /// Run diagnostics on the given text document.
    ///
    /// Implementors should inspect `document.text` to produce diagnostics, and may use
    /// `document.uri` and `document.language_id` to provide accurate locations and rules.
    /// Not all tools will implement diagnostics, so the default implementation returns [`Ok`] with an empty vector.
    ///
    /// # Errors
    /// Return [`Err`] when an error occurs; ignoring diagnostics should return [`Ok`] with an empty vector.
    fn run_diagnostic(&self, _document: &TextDocument) -> DiagnosticResult {
        Ok(Vec::new())
    }

    /// Run diagnostics on save for the given text document.
    ///
    /// Implementors should inspect `document.text` to produce diagnostics, and may use
    /// `document.uri` and `document.language_id` to determine how and where diagnostics apply.
    /// Returns a vector of `(Uri, Vec<Diagnostic>)` tuples representing the diagnostic results.
    /// Not all tools will implement diagnostics on save, so the default implementation returns [`Ok`] with an empty vector.
    ///
    /// # Errors
    /// Return [`Err`] when an error occurs; ignoring diagnostics should return [`Ok`] with an empty vector.
    fn run_diagnostic_on_save(&self, _document: &TextDocument) -> DiagnosticResult {
        Ok(Vec::new())
    }

    /// Run diagnostics on change for the given text document.
    ///
    /// Implementors should inspect `document.text` to produce diagnostics, and may use
    /// `document.uri` and `document.language_id` to determine how and where diagnostics apply.
    /// Returns a vector of `(Uri, Vec<Diagnostic>)` tuples representing the diagnostic results.
    /// Not all tools will implement diagnostics on change, so the default implementation returns [`Ok`] with an empty vector.
    ///
    /// # Errors
    /// Return [`Err`] when an error occurs; ignoring diagnostics should return [`Ok`] with an empty vector.
    fn run_diagnostic_on_change(&self, _document: &TextDocument) -> DiagnosticResult {
        Ok(Vec::new())
    }

    /// Remove internal cache for the given URI, if any.
    fn remove_uri_cache(&self, _uri: &Uri) {
        // Default implementation does nothing.
    }
}

pub struct ToolRestartChanges {
    /// The tool that was restarted (linter, formatter).
    /// If None, no tool was restarted.
    pub tool: Option<Box<dyn Tool>>,
    /// The patterns that were added during the tool restart
    /// Old patterns will be automatically unregistered
    pub watch_patterns: Option<Vec<Pattern>>,
}
