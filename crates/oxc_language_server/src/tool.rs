use std::path::Path;

use tower_lsp_server::{
    jsonrpc::ErrorCode,
    ls_types::{
        CodeActionOrCommand, Diagnostic, MessageType, Pattern, ServerCapabilities, TextEdit, Uri,
        WorkspaceEdit,
    },
};

use crate::{CodeActionParams, TextDocument, capabilities::Capabilities};

pub trait ToolBuilder: Send + Sync {
    /// Modify the server capabilities to include capabilities provided by this tool.
    fn server_capabilities(
        &self,
        _capabilities: &mut ServerCapabilities,
        _backend_capabilities: &mut Capabilities,
    ) {
    }

    /// Build a boxed instance of the tool for the given root URI and options.
    fn build(&self, root_uri: &Uri, options: serde_json::Value) -> ToolBuildResult;

    /// Build a boxed instance of the tool with the extra [`BuildContext`] of its worker.
    ///
    /// The default implementation ignores the context, which is correct for tools which only
    /// resolve their configuration upwards from their root and never walk it eagerly.
    fn build_with_context(
        &self,
        root_uri: &Uri,
        options: serde_json::Value,
        _context: BuildContext<'_>,
    ) -> ToolBuildResult {
        self.build(root_uri, options)
    }

    /// The configuration file names which make a directory a project root for this tool.
    ///
    /// The language server uses them, next to `package.json`, to decide whether a watched file
    /// change can add or remove a `workingDirectories` entry. They are the names
    /// [`Self::is_project_root`] looks for, not the watcher patterns: a setup with an explicit
    /// `configPath` watches one file but still gains a working directory when a config appears
    /// somewhere else.
    fn config_file_names(&self) -> Vec<&'static str> {
        Vec::new()
    }

    /// Whether `dir` is a project root for this tool, used by
    /// `"workingDirectories": [{ "mode": "auto" }]`.
    ///
    /// Implementors should return `true` when the directory contains both a `package.json` and one
    /// of the tool configuration files.
    fn is_project_root(&self, _dir: &Path) -> bool {
        false
    }

    /// Shutdown hook for the tool. Implementors may perform any necessary cleanup here.
    fn shutdown(&self, _root_uri: &Uri) {
        // Default implementation does nothing.
    }
}

pub type DiagnosticResult = Result<Vec<(Uri, Vec<Diagnostic>)>, String>;

pub trait Tool: Send + Sync {
    /// The Server has new configuration changes.
    /// Returns a [ToolRestartChanges] indicating what changes were made for the Tool.
    ///
    /// `context` is the [`BuildContext`] of the worker: an implementor which rebuilds itself has
    /// to hand it back to [`ToolBuilder::build_with_context`], or it loses the roots it must skip.
    fn handle_configuration_change(
        &self,
        builder: &dyn ToolBuilder,
        root_uri: &Uri,
        context: BuildContext<'_>,
        old_options_json: &serde_json::Value,
        new_options_json: serde_json::Value,
    ) -> ToolRestartChanges;

    /// Get the file watcher patterns for this tool based on the provided options.
    /// These patterns will be used to watch for file changes relevant to the tool.
    fn get_watcher_patterns(&self, options: serde_json::Value) -> Vec<Pattern>;

    /// Handle a watched file change event for the given URI.
    /// Returns a [ToolRestartChanges] indicating what changes were made for the Tool.
    /// The Tool should decide whether it needs to restart or take any action based on the URI.
    ///
    /// The given URI may not match the watch patterns or may be irrelevant for the workspace.
    /// A file change can affect multiple workspaces, so the Tool should check if it is relevant.
    ///
    /// `context` is the [`BuildContext`] of the worker: an implementor which rebuilds itself has
    /// to hand it back to [`ToolBuilder::build_with_context`], or it loses the roots it must skip.
    fn handle_watched_file_change(
        &self,
        builder: &dyn ToolBuilder,
        changed_uri: &Uri,
        root_uri: &Uri,
        context: BuildContext<'_>,
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

    /// Get code actions or commands provided by this tool.
    /// The tool should filter the code actions based on the requested range.
    /// The context can be used to further filter the code actions,
    /// for example by the `only` field which indicates that only code actions of certain kinds are requested.
    fn get_code_actions_or_commands(&self, _params: CodeActionParams) -> Vec<CodeActionOrCommand> {
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
    fn run_format(&self, _document: TextDocument) -> Result<Vec<TextEdit>, String> {
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
    fn run_diagnostic(&self, _document: TextDocument) -> DiagnosticResult {
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
    fn run_diagnostic_on_save(&self, _document: TextDocument) -> DiagnosticResult {
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
    fn run_diagnostic_on_change(&self, _document: TextDocument) -> DiagnosticResult {
        Ok(Vec::new())
    }

    /// Remove internal cache for the given URI, if any.
    fn remove_uri_cache(&self, _uri: &Uri) {
        // Default implementation does nothing.
    }
}

/// Extra context a worker passes to [`ToolBuilder::build_with_context`].
#[derive(Debug, Default, Clone, Copy)]
pub struct BuildContext<'a> {
    /// Roots below the worker root which are owned by another worker, i.e. the resolved
    /// `workingDirectories` of this worker.
    ///
    /// A tool which eagerly walks its root directory (config discovery, ignore file collection,
    /// ...) must skip those directories, otherwise the same file would be handled twice, with two
    /// different configurations.
    pub excluded_roots: &'a [Uri],
    /// The workspace folder this worker belongs to, when it is a `workingDirectories` sub worker.
    ///
    /// A sub worker still has to honour the ignore files between that folder and its own root,
    /// the same way the CLI does when it is run from inside the directory.
    pub parent_root: Option<&'a Uri>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientMessage {
    /// The message to be sent to the client.
    pub message: String,
    /// The type of message to be sent to the client (e.g., error, warning).
    pub r#type: MessageType,
}

pub struct ToolBuildResult {
    /// The tool that was started (linter, formatter).
    /// It should always be started and on internal errors, fallback to the default configuration of the tool.
    pub tool: Box<dyn Tool>,
    /// Even if the tool started successfully, it may have encountered issues during initialization.
    /// The `client_messages` field can be used to communicate any warnings or errors to the client.
    pub client_messages: Vec<ClientMessage>,
}

pub struct ToolRestartChanges {
    /// The tool that was restarted (linter, formatter).
    /// If None, no tool was restarted.
    pub tool: Option<Box<dyn Tool>>,
    /// The patterns that were added during the tool restart
    /// Old patterns will be automatically unregistered
    pub watch_patterns: Option<Vec<Pattern>>,
    /// Even if the tool restarted successfully, it may have encountered issues during initialization.
    /// The `client_messages` field can be used to communicate any warnings or errors to the client.
    pub client_messages: Vec<ClientMessage>,
}
