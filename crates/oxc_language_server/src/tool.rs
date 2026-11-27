use std::path::{Path, PathBuf};

use tower_lsp_server::{
    gen_lsp_types::{
        CodeActionResponse, Diagnostic, MessageType, Pattern, ServerCapabilities, TextEdit, Uri,
        WorkspaceEdit,
    },
    jsonrpc::ErrorCode,
};

use crate::{
    CodeActionParams, TextDocument, capabilities::Capabilities, file_system::ResolvedPath,
    uri_utils::uri_to_file_path,
};

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
    ///
    /// The builder a [`Tool`] receives builds with the context of its worker, whichever of
    /// [`Self::build`] and this method the tool calls, so a tool which rebuilds itself through it
    /// keeps the context.
    ///
    /// A method added to this trait must also be forwarded by the builder a worker hands to its
    /// tool.
    fn build_with_context(
        &self,
        root_uri: &Uri,
        options: serde_json::Value,
        _context: BuildContext<'_>,
    ) -> ToolBuildResult {
        self.build(root_uri, options)
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
    ///
    /// The given URI may not match the watch patterns or may be irrelevant for the workspace.
    /// A file change can affect multiple workspaces, so the Tool should check if it is relevant.
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

    /// Get code actions or commands provided by this tool.
    /// The tool should filter the code actions based on the requested range.
    /// The context can be used to further filter the code actions,
    /// for example by the `only` field which indicates that only code actions of certain kinds are requested.
    fn get_code_actions_or_commands(&self, _params: CodeActionParams) -> Vec<CodeActionResponse> {
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
#[non_exhaustive]
pub struct BuildContext<'a> {
    /// Workspace folders nested below the worker root, which are served by their own worker.
    ///
    /// A tool which eagerly walks its root directory (config discovery, ignore file collection,
    /// ...) must skip those directories, otherwise the same file would be handled twice, with two
    /// different configurations.
    pub excluded_roots: &'a [Uri],
    /// The outermost workspace folder which contains the worker root, if any.
    ///
    /// A nested worker still has to honour the ignore files of that folder, as the tool does when
    /// it is run from inside the nested folder.
    pub parent_root: Option<&'a Uri>,
}

impl BuildContext<'_> {
    /// The file paths of the [`BuildContext::excluded_roots`] which are below `root_uri`. The
    /// other roots, and the ones which cannot be resolved to a file path, are dropped.
    ///
    /// The paths are spelled like the path of `root_uri`, the root of the tool. The client may spell
    /// the roots differently (the case of a drive letter or of a directory, on Windows and macOS),
    /// and a tool compares them with paths it derives from its own root.
    pub fn excluded_root_paths(&self, root_uri: &Uri) -> Vec<PathBuf> {
        self.excluded_roots
            .iter()
            .filter_map(|uri| {
                let relative = relative_to(uri, root_uri)?;
                Some(uri_to_file_path(root_uri)?.join(relative))
            })
            .collect()
    }

    /// The file path of [`BuildContext::parent_root`], spelled like the path of `root_uri`.
    pub fn parent_root_path(&self, root_uri: &Uri) -> Option<PathBuf> {
        let depth = relative_to(root_uri, self.parent_root?)?.components().count();
        let root = uri_to_file_path(root_uri)?;
        root.ancestors().nth(depth).map(Path::to_path_buf)
    }
}

/// The path of `uri` below `base`, both resolved the way the worker manager routes a file.
fn relative_to(uri: &Uri, base: &Uri) -> Option<PathBuf> {
    let uri = ResolvedPath::try_from(uri).ok()?;
    let base = ResolvedPath::try_from(base).ok()?;
    uri.strip_prefix(&base).map(Path::to_path_buf)
}

/// The [`ToolBuilder`] a worker hands to its [`Tool`]: it builds with the context of the worker.
pub struct ContextBuilder<'a> {
    pub inner: &'a dyn ToolBuilder,
    pub context: BuildContext<'a>,
}

#[deny(clippy::missing_trait_methods)]
impl ToolBuilder for ContextBuilder<'_> {
    fn server_capabilities(
        &self,
        capabilities: &mut ServerCapabilities,
        backend_capabilities: &mut Capabilities,
    ) {
        self.inner.server_capabilities(capabilities, backend_capabilities);
    }

    fn build(&self, root_uri: &Uri, options: serde_json::Value) -> ToolBuildResult {
        self.inner.build_with_context(root_uri, options, self.context)
    }

    /// A tool never chooses its context: the one of its worker always applies.
    fn build_with_context(
        &self,
        root_uri: &Uri,
        options: serde_json::Value,
        _context: BuildContext<'_>,
    ) -> ToolBuildResult {
        self.build(root_uri, options)
    }

    fn shutdown(&self, root_uri: &Uri) {
        self.inner.shutdown(root_uri);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        sync::{Arc, Mutex},
    };

    use tower_lsp_server::gen_lsp_types::Uri;

    use crate::{
        tests::FakeToolBuilder,
        tool::{BuildContext, ContextBuilder, ToolBuilder},
    };

    #[test]
    fn test_build_context_paths_are_spelled_like_the_root() {
        let root: Uri = "file:///repo/packages/a".parse().unwrap();
        let excluded: Vec<Uri> = vec![
            "file:///repo/packages/a/src/".parse().unwrap(),
            "file:///repo/packages/b".parse().unwrap(),
        ];
        let parent: Uri = "file:///repo".parse().unwrap();
        let context = BuildContext { excluded_roots: &excluded, parent_root: Some(&parent) };

        // a root which is not below `root` is not an excluded root
        assert_eq!(context.excluded_root_paths(&root), [PathBuf::from("/repo/packages/a/src")]);
        assert_eq!(context.parent_root_path(&root), Some(PathBuf::from("/repo")));
        assert_eq!(BuildContext::default().parent_root_path(&root), None);
    }

    /// A tool which builds through the builder of its worker keeps the context of the worker,
    /// whichever method it calls.
    #[test]
    fn test_the_builder_of_a_worker_builds_with_the_context_of_the_worker() {
        let built = Arc::new(Mutex::new(vec![]));
        let inner = FakeToolBuilder::default().with_context_tracking(Arc::clone(&built));
        let root: Uri = "file:///repo".parse().unwrap();
        let excluded: Vec<Uri> = vec!["file:///repo/a".parse().unwrap()];
        let builder = ContextBuilder {
            inner: &inner,
            context: BuildContext { excluded_roots: &excluded, parent_root: None },
        };

        builder.build_with_context(&root, serde_json::Value::Null, BuildContext::default());

        assert_eq!(*built.lock().unwrap(), vec![(root, excluded, None)]);
    }
}
