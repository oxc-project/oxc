use tower_lsp_server::{
    jsonrpc::ErrorCode,
    lsp_types::{
        CodeActionKind, CodeActionOrCommand, Diagnostic, Pattern, Range, ServerCapabilities,
        TextEdit, Uri, WorkspaceEdit,
    },
};

pub trait ToolBuilder: Send + Sync {
    /// Modify the server capabilities to include capabilities provided by this tool.
    fn server_capabilities(&self, _capabilities: &mut ServerCapabilities) {}

    /// Build a boxed instance of the tool for the given root URI and options.
    fn build_boxed(&self, root_uri: &Uri, options: serde_json::Value) -> Box<dyn Tool>;
}

pub trait Tool: Send + Sync {
    /// Get the name of the tool.
    fn name(&self) -> &'static str;

    /// The Server has new configuration changes.
    /// Returns a [ToolRestartChanges] indicating what changes were made for the Tool.
    fn handle_configuration_change(
        &self,
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
        changed_uri: &Uri,
        root_uri: &Uri,
        options: serde_json::Value,
    ) -> ToolRestartChanges;

    /// Check if this tool is responsible for handling the given command.
    fn is_responsible_for_command(&self, _command: &str) -> bool {
        false
    }

    /// Tries to execute the given command with the provided arguments.
    /// If the command is not recognized, returns `Ok(None)`.
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
        Ok(None)
    }

    /// Get code actions or commands provided by this tool for the given URI and range.
    /// The `only_code_action_kinds` parameter can be used to filter the results based on specific code action kinds.
    fn get_code_actions_or_commands(
        &self,
        _uri: &Uri,
        _range: &Range,
        _only_code_action_kinds: Option<&Vec<CodeActionKind>>,
    ) -> Vec<CodeActionOrCommand> {
        Vec::new()
    }

    /// Format the content of the given URI.
    /// If `content` is `None`, the tool should read the content from the file system.
    /// Returns a vector of `TextEdit` representing the formatting changes.
    ///
    /// Not all tools will implement formatting, so the default implementation returns `None`.
    fn run_format(&self, _uri: &Uri, _content: Option<&str>) -> Option<Vec<TextEdit>> {
        None
    }

    /// Run diagnostics on the content of the given URI.
    /// If `content` is `None`, the tool should read the content from the file system.
    /// Returns a vector of `Diagnostic` representing the diagnostic results.
    /// Not all tools will implement diagnostics, so the default implementation returns `None`.
    fn run_diagnostic(&self, _uri: &Uri, _content: Option<&str>) -> Option<Vec<Diagnostic>> {
        None
    }

    /// Run diagnostics on save for the content of the given URI.
    /// If `content` is `None`, the tool should read the content from the file system.
    /// Returns a vector of `Diagnostic` representing the diagnostic results.
    /// Not all tools will implement diagnostics on save, so the default implementation returns `None`.
    fn run_diagnostic_on_save(
        &self,
        _uri: &Uri,
        _content: Option<&str>,
    ) -> Option<Vec<Diagnostic>> {
        None
    }

    /// Run diagnostics on change for the content of the given URI.
    /// If `content` is `None`, the tool should read the content from the file system.
    /// Returns a vector of `Diagnostic` representing the diagnostic results.
    /// Not all tools will implement diagnostics on change, so the default implementation returns `None`.
    fn run_diagnostic_on_change(
        &self,
        _uri: &Uri,
        _content: Option<&str>,
    ) -> Option<Vec<Diagnostic>> {
        None
    }

    /// Remove diagnostics associated with the given URI.
    fn remove_diagnostics(&self, _uri: &Uri) {
        // Default implementation does nothing.
    }

    /// Shutdown the tool and return any necessary changes to be made after shutdown.
    fn shutdown(&self) -> ToolShutdownChanges {
        ToolShutdownChanges { uris_to_clear_diagnostics: None }
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

pub struct ToolShutdownChanges {
    /// The URIs that need to have their diagnostics removed after the tool shutdown
    pub uris_to_clear_diagnostics: Option<Vec<Uri>>,
}
