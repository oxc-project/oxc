use tower_lsp_server::{
    jsonrpc::ErrorCode,
    lsp_types::{
        CodeActionKind, CodeActionOrCommand, Diagnostic, Pattern, Range, Uri, WorkspaceEdit,
    },
};

pub trait ToolBuilder<T: Tool> {
    fn new(root_uri: Uri, options: serde_json::Value) -> Self;
    fn build(&self) -> T;
}

pub trait Tool: Sized {
    /// The Server has new configuration changes.
    /// Returns a [ToolRestartChanges] indicating what changes were made for the Tool.
    async fn handle_configuration_change(
        &self,
        root_uri: &Uri,
        old_options_json: &serde_json::Value,
        new_options_json: serde_json::Value,
    ) -> ToolRestartChanges<Self>;

    /// Get the file watcher patterns for this tool based on the provided options.
    /// These patterns will be used to watch for file changes relevant to the tool.
    fn get_watcher_patterns(&self, options: serde_json::Value) -> Vec<Pattern>;

    /// Handle a watched file change event for the given URI.
    /// Returns a [ToolRestartChanges] indicating what changes were made for the Tool.
    /// The Tool should decide whether it needs to restart or take any action based on the URI.
    async fn handle_watched_file_change(
        &self,
        changed_uri: &Uri,
        root_uri: &Uri,
        options: serde_json::Value,
    ) -> ToolRestartChanges<Self>;

    /// Check if this tool is responsible for handling the given command.
    fn is_responsible_for_command(&self, _command: &str) -> bool {
        false
    }

    /// Tries to execute the given command with the provided arguments.
    /// If the command is not recognized, returns `Ok(None)`.
    /// If the command is recognized and executed it can return:
    /// - `Ok(Some(WorkspaceEdit))` if the command was executed successfully and produced a workspace edit.
    /// - `Ok(None)` if the command was executed successfully but did not produce any workspace edit.
    async fn execute_command(
        &self,
        _command: &str,
        _arguments: Vec<serde_json::Value>,
    ) -> Result<Option<WorkspaceEdit>, ErrorCode> {
        Ok(None)
    }

    /// Get code actions or commands provided by this tool for the given URI and range.
    /// The `only_code_action_kinds` parameter can be used to filter the results based on specific code action kinds.
    async fn get_code_actions_or_commands(
        &self,
        _uri: &Uri,
        _range: &Range,
        _only_code_action_kinds: Option<Vec<CodeActionKind>>,
    ) -> Vec<CodeActionOrCommand> {
        Vec::new()
    }
}

pub struct ToolRestartChanges<T> {
    /// The tool that was restarted (linter, formatter).
    /// If None, no tool was restarted.
    pub tool: Option<T>,
    /// The diagnostic reports that need to be revalidated after the tool restart
    pub diagnostic_reports: Option<Vec<(String, Vec<Diagnostic>)>>,
    /// The patterns that were added during the tool restart
    /// Old patterns will be automatically unregistered
    pub watch_patterns: Option<Vec<Pattern>>,
}
