use tower_lsp_server::{
    jsonrpc::ErrorCode,
    lsp_types::{Diagnostic, Pattern, Uri, WorkspaceEdit},
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
