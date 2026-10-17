use std::{
    collections::VecDeque,
    str::FromStr,
    sync::{Arc, Mutex},
};

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
use tower_lsp_server::{
    Client, Server,
    gen_lsp_types::*,
    jsonrpc::{ErrorCode, Id, Request, Response},
};

use crate::{
    DiagnosticMode, TextDocument, Tool, ToolBuildResult, ToolBuilder, ToolRestartChanges,
    WorkerManager,
    backend::Backend,
    build_lsp_service,
    tool::{ClientMessage, DiagnosticResult},
    working_directories::sub_worker_options,
};

#[derive(Default)]
pub struct FakeToolBuilder {
    diagnostic_mode: DiagnosticMode,
    cache_uris: Option<Arc<Mutex<Vec<Uri>>>>,
    pub build_client_message: Vec<ClientMessage>,
    delays: FakeToolDelays,
    /// How long building a tool takes (ms), to interleave a rebuild with a configuration change.
    build_delay: u64,
    working_directories_disabled: bool,
    build_roots: Option<Arc<Mutex<Vec<Uri>>>>,
    discovered_extends: Option<(Uri, String)>,
    diagnostic_contexts: Option<Arc<Mutex<Vec<usize>>>>,
    build_contexts: Option<Arc<Mutex<Vec<usize>>>>,
}

#[derive(Default, Clone, Copy)]
pub struct FakeToolDelays {
    run_diagnostic: u64,
}

impl FakeToolBuilder {
    pub fn new(diagnostic_mode: DiagnosticMode) -> Self {
        Self {
            diagnostic_mode,
            cache_uris: None,
            build_client_message: Vec::new(),
            delays: FakeToolDelays::default(),
            build_delay: 0,
            working_directories_disabled: false,
            build_roots: None,
            discovered_extends: None,
            diagnostic_contexts: None,
            build_contexts: None,
        }
    }

    /// Record how many roots every build of the tool excluded, in build order.
    pub fn with_build_context_tracking(self, build_contexts: Arc<Mutex<Vec<usize>>>) -> Self {
        Self { build_contexts: Some(build_contexts), ..self }
    }

    /// Record how many roots the tool was built with as excluded, every time it runs diagnostics.
    /// It tells a test which build of the tool linted a document.
    pub fn with_diagnostic_context_tracking(
        self,
        diagnostic_contexts: Arc<Mutex<Vec<usize>>>,
    ) -> Self {
        Self { diagnostic_contexts: Some(diagnostic_contexts), ..self }
    }

    /// Report `extended_path` as an absolute watcher pattern, as a config discovered below
    /// `marker_dir` and extended from outside the root, for as long as `marker_dir` is not one of
    /// the excluded roots of the build.
    pub fn with_discovered_extends(self, marker_dir: Uri, extended_path: String) -> Self {
        Self { discovered_extends: Some((marker_dir, extended_path)), ..self }
    }

    /// Record the root of every tool which is built, to assert that a worker was rebuilt.
    pub fn with_build_tracking(self, build_roots: Arc<Mutex<Vec<Uri>>>) -> Self {
        Self { build_roots: Some(build_roots), ..self }
    }

    /// Make building a tool take `build_delay` milliseconds.
    pub fn with_build_delay(self, build_delay: u64) -> Self {
        Self { build_delay, ..self }
    }

    /// Ignore `workingDirectories`, like a tool in Vite+ mode.
    pub fn with_working_directories_disabled(self) -> Self {
        Self { working_directories_disabled: true, ..self }
    }

    pub fn with_cache_tracking(self, cache_uris: Arc<Mutex<Vec<Uri>>>) -> Self {
        Self { cache_uris: Some(cache_uris), ..self }
    }

    pub fn with_delays(self, delays: FakeToolDelays) -> Self {
        Self { delays, ..self }
    }
}

impl ToolBuilder for FakeToolBuilder {
    fn build(&self, root_uri: &Uri, options: serde_json::Value) -> ToolBuildResult {
        if self.build_delay > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.build_delay));
        }
        if let Some(build_roots) = &self.build_roots {
            build_roots.lock().unwrap().push(root_uri.clone());
        }
        ToolBuildResult {
            tool: Box::new(FakeTool {
                cache_uris: self.cache_uris.clone(),
                delays: self.delays,
                extended_path: None,
                excluded_roots: 0,
                diagnostic_contexts: self.diagnostic_contexts.clone(),
                clean_uris: clean_uris(&options),
            }),
            client_messages: self.build_client_message.clone(),
        }
    }

    fn build_with_context(
        &self,
        root_uri: &Uri,
        options: serde_json::Value,
        context: crate::BuildContext<'_>,
    ) -> ToolBuildResult {
        let mut result = self.build(root_uri, options.clone());

        let extended_path =
            self.discovered_extends.as_ref().and_then(|(marker_dir, discovered)| {
                let excluded = context
                    .excluded_roots
                    .iter()
                    .any(|excluded| crate::utils::roots_are_equal(excluded, marker_dir));
                (!excluded).then(|| discovered.clone())
            });

        if let Some(build_contexts) = &self.build_contexts {
            build_contexts.lock().unwrap().push(context.excluded_roots.len());
        }

        result.tool = Box::new(FakeTool {
            cache_uris: self.cache_uris.clone(),
            delays: self.delays,
            extended_path,
            excluded_roots: context.excluded_roots.len(),
            diagnostic_contexts: self.diagnostic_contexts.clone(),
            clean_uris: clean_uris(&options),
        });

        result
    }

    fn use_working_directories(&self) -> bool {
        !self.working_directories_disabled
    }

    fn server_capabilities(
        &self,
        capabilities: &mut ServerCapabilities,
        backend_capabilities: &mut crate::Capabilities,
    ) {
        backend_capabilities.diagnostic_mode = self.diagnostic_mode.clone();

        // tell the client we support pull diagnostics
        capabilities.diagnostic_provider =
            if backend_capabilities.diagnostic_mode == DiagnosticMode::Pull {
                Some(DiagnosticProvider::DiagnosticOptions(DiagnosticOptions::default()))
            } else {
                None
            };
    }
}

pub struct FakeTool {
    cache_uris: Option<Arc<Mutex<Vec<Uri>>>>,
    delays: FakeToolDelays,
    extended_path: Option<String>,
    excluded_roots: usize,
    diagnostic_contexts: Option<Arc<Mutex<Vec<usize>>>>,
    /// Documents this build of the tool reports as clean, regardless of their name. A re-lint
    /// which finds nothing still has to clear what the previous owner published.
    clean_uris: Vec<String>,
}

/// The `cleanUris` option of the fake tool: the documents it reports as clean.
fn clean_uris(options: &serde_json::Value) -> Vec<String> {
    options
        .get("cleanUris")
        .and_then(|value| serde_json::from_value::<Vec<String>>(value.clone()).ok())
        .unwrap_or_default()
}

pub const FAKE_COMMAND: &str = "fake.command";

const WORKSPACE: &str = "file:///path/to/workspace";

const NESTED_WORKSPACE: &str = "file:///path/to/workspace/nested";

const WORKSPACE_2: &str = "file:///path/to/another_workspace";

impl Tool for FakeTool {
    fn execute_command(
        &self,
        command: &str,
        arguments: Vec<serde_json::Value>,
    ) -> Result<Option<WorkspaceEdit>, ErrorCode> {
        if command != FAKE_COMMAND {
            return Err(ErrorCode::InvalidParams);
        }

        if !arguments.is_empty() {
            return Ok(Some(WorkspaceEdit::default()));
        }

        Ok(None)
    }

    fn handle_configuration_change(
        &self,
        builder: &dyn ToolBuilder,
        root_uri: &Uri,
        old_options_json: &serde_json::Value,
        new_options_json: serde_json::Value,
    ) -> ToolRestartChanges {
        // a real tool restarts when its options change, and reports no new watcher patterns: they
        // are derived from the options and from the build context it was just rebuilt with.
        // `workingDirectories` is owned by the language server, it never restarts a tool.
        if new_options_json.is_object()
            && sub_worker_options(&new_options_json) != sub_worker_options(old_options_json)
        {
            let result = builder.build(root_uri, new_options_json);
            return ToolRestartChanges {
                tool: Some(result.tool),
                watch_patterns: None,
                client_messages: result.client_messages,
            };
        }
        if new_options_json.as_u64() == Some(1) || new_options_json.as_u64() == Some(3) {
            let result = builder.build(root_uri, new_options_json);
            return ToolRestartChanges {
                tool: Some(result.tool),
                watch_patterns: None,
                client_messages: result.client_messages,
            };
        }
        if new_options_json.as_u64() == Some(2) {
            return ToolRestartChanges {
                tool: None,
                watch_patterns: Some(vec!["**/new_watcher.config".to_string()]),
                client_messages: Vec::new(),
            };
        }
        if new_options_json.as_u64() == Some(4) {
            return ToolRestartChanges {
                tool: None,
                watch_patterns: None,
                client_messages: vec![ClientMessage {
                    message: "Fake misconfiguration message".to_string(),
                    r#type: MessageType::Warning,
                }],
            };
        }
        ToolRestartChanges { tool: None, watch_patterns: None, client_messages: Vec::new() }
    }

    fn get_watcher_patterns(
        &self,
        options: serde_json::Value,
    ) -> Vec<tower_lsp_server::gen_lsp_types::Pattern> {
        let mut patterns = if matches!(options, serde_json::Value::Null) {
            vec!["**/fake.config".to_string()]
        } else {
            vec![]
        };
        patterns.extend(self.extended_path.clone());
        patterns
    }

    fn handle_watched_file_change(
        &self,
        builder: &dyn ToolBuilder,
        changed_uri: &Uri,
        root_uri: &Uri,
        options: serde_json::Value,
    ) -> ToolRestartChanges {
        if changed_uri.as_ref().ends_with("tool.config") {
            let result = builder.build(root_uri, options);
            return ToolRestartChanges {
                tool: Some(result.tool),
                watch_patterns: None,
                client_messages: result.client_messages,
            };
        }
        if changed_uri.as_ref().ends_with("watcher.config") {
            return ToolRestartChanges {
                tool: None,
                watch_patterns: Some(vec!["**/new_watcher.config".to_string()]),
                client_messages: Vec::new(),
            };
        }
        if changed_uri.as_ref().ends_with("misconfiguration.config") {
            return ToolRestartChanges {
                tool: None,
                watch_patterns: None,
                client_messages: vec![ClientMessage {
                    message: "Fake misconfiguration message".to_string(),
                    r#type: MessageType::Warning,
                }],
            };
        }

        ToolRestartChanges { tool: None, watch_patterns: None, client_messages: Vec::new() }
    }

    fn get_code_actions_or_commands(
        &self,
        params: crate::CodeActionParams,
    ) -> Vec<CodeActionResponse> {
        if params.uri.as_ref().ends_with("code_action.config") {
            return vec![CodeActionResponse::CodeAction(CodeAction {
                title: "Code Action title".to_string(),
                kind: Some(CodeActionKind::QuickFix),
                edit: Some(WorkspaceEdit::default()),
                ..Default::default()
            })];
        }

        vec![]
    }

    fn run_diagnostic(&self, document: TextDocument) -> DiagnosticResult {
        if self.delays.run_diagnostic > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.delays.run_diagnostic));
        }
        if let Some(cache_uris) = &self.cache_uris {
            cache_uris.lock().unwrap().push(document.uri.clone());
        }
        if let Some(diagnostic_contexts) = &self.diagnostic_contexts {
            diagnostic_contexts.lock().unwrap().push(self.excluded_roots);
        }
        if self.clean_uris.iter().any(|clean| document.uri.as_ref().ends_with(clean)) {
            return Ok(Vec::new());
        }

        if document.uri.as_ref().ends_with("diagnostics.config") {
            return Ok(vec![(
                document.uri.clone(),
                vec![Diagnostic {
                    message: Message::String(format!(
                        "Fake diagnostic for content: {}",
                        document.text.as_deref().unwrap_or("<no content>")
                    )),
                    ..Default::default()
                }],
            )]);
        }

        if document.uri.as_ref().ends_with("error.config") {
            return Err("Fake diagnostic error".to_string());
        }

        Ok(Vec::new())
    }

    fn run_diagnostic_on_change(&self, document: TextDocument) -> DiagnosticResult {
        // For this fake tool, we use the same logic as run_diagnostic
        self.run_diagnostic(document)
    }

    fn run_diagnostic_on_save(&self, document: TextDocument) -> DiagnosticResult {
        // For this fake tool, we use the same logic as run_diagnostic
        self.run_diagnostic(document)
    }

    fn remove_uri_cache(&self, uri: &Uri) {
        if let Some(cache_uris) = &self.cache_uris {
            cache_uris.lock().unwrap().retain(|cached_uri| cached_uri != uri);
        }
    }
}

// A test server that can send requests and receive responses.
// Copied from <https://github.com/veryl-lang/veryl/blob/888d83abaa58ca5a7ffef501a1c557e48c750b92/crates/languageserver/src/tests.rs>
struct TestServer {
    req_stream: DuplexStream,
    res_stream: DuplexStream,
    responses: VecDeque<String>,
    response_buffer: Vec<u8>,
}

impl TestServer {
    fn new<F>(init: F) -> Self
    where
        F: FnOnce(Client) -> Backend,
    {
        async fn test_configuration_handler(
            service: &Backend,
            _params: Value,
        ) -> Result<Value, tower_lsp_server::jsonrpc::Error> {
            let mut configs = vec![];
            for worker in &*service.worker_manager.read_workspace_workers().await {
                configs.push(worker.options.lock().await.clone());
            }
            Ok(json!(configs))
        }

        async fn test_workers_handler(
            service: &Backend,
            _params: Value,
        ) -> Result<Value, tower_lsp_server::jsonrpc::Error> {
            let mut workers = vec![];
            for worker in &*service.worker_manager.read_workspace_workers().await {
                workers.push(json!({
                    "root": worker.get_root_uri().as_str(),
                    "parent": worker.get_parent_uri().map(Uri::as_str),
                    "excluded": worker
                        .get_working_directories()
                        .await
                        .iter()
                        .map(|uri| uri.as_str().to_string())
                        .collect::<Vec<_>>(),
                }));
            }
            Ok(json!(workers))
        }

        let (req_client, req_server) = tokio::io::duplex(1024);
        let (res_server, res_client) = tokio::io::duplex(1024);

        let (service, socket) = build_lsp_service(init)
            .custom_method("test/configuration", test_configuration_handler)
            .custom_method("test/workers", test_workers_handler)
            .finish();

        tokio::spawn(Server::new(req_server, res_server, socket).serve(service));

        Self {
            req_stream: req_client,
            res_stream: res_client,
            responses: VecDeque::new(),
            response_buffer: Vec::new(),
        }
    }

    fn encode(payload: &str) -> String {
        format!("Content-Length: {}\r\n\r\n{}", payload.len(), payload)
    }

    fn decode(buffer: &mut Vec<u8>) -> Vec<String> {
        let mut ret = Vec::new();

        while !buffer.is_empty() {
            let Some(p) = buffer.windows(4).position(|window| window == b"\r\n\r\n") else {
                break;
            };

            let header = std::str::from_utf8(&buffer[..p + 4]).unwrap();
            let len =
                header.strip_prefix("Content-Length: ").unwrap().strip_suffix("\r\n\r\n").unwrap();
            let len: usize = len.parse().unwrap();
            let body_start = p + 4;
            let body_end = body_start + len;
            if buffer.len() < body_end {
                break;
            }

            let body = String::from_utf8(buffer[body_start..body_end].to_vec()).unwrap();
            ret.push(body);
            buffer.drain(..body_end);
        }

        ret
    }

    async fn read_more_messages(&mut self) {
        let mut buf = vec![0; 1024];
        let n = self.res_stream.read(&mut buf).await.unwrap();
        assert_ne!(n, 0);
        self.response_buffer.extend_from_slice(&buf[..n]);
        for x in Self::decode(&mut self.response_buffer) {
            self.responses.push_front(x);
        }
    }

    async fn read_until_message(&mut self) {
        while self.responses.is_empty() {
            self.read_more_messages().await;
        }
    }

    async fn send_request(&mut self, req: Request) {
        let req = serde_json::to_string(&req).unwrap();
        let req = Self::encode(&req);
        self.req_stream.write_all(req.as_bytes()).await.unwrap();
    }

    async fn send_response(&mut self, res: Response) {
        let res = serde_json::to_string(&res).unwrap();
        let res = Self::encode(&res);
        self.req_stream.write_all(res.as_bytes()).await.unwrap();
    }

    async fn send_ack(&mut self, id: &Id) {
        let req = Response::from_ok(id.clone(), None::<serde_json::Value>.into());
        let req = serde_json::to_string(&req).unwrap();
        let req = Self::encode(&req);
        self.req_stream.write_all(req.as_bytes()).await.unwrap();
    }

    async fn recv_response(&mut self) -> Response {
        if self.responses.is_empty() {
            self.read_until_message().await;
        }
        let res = self.responses.pop_back().unwrap();
        serde_json::from_str(&res).unwrap()
    }

    async fn recv_notification(&mut self) -> Request {
        let mut skipped_responses = Vec::new();
        loop {
            self.read_until_message().await;

            let res = self.responses.pop_back().unwrap();
            let val: serde_json::Value = serde_json::from_str(&res).unwrap();
            if val.get("method").is_some() {
                for response in skipped_responses.into_iter().rev() {
                    self.responses.push_back(response);
                }
                return serde_json::from_value(val).unwrap();
            }

            skipped_responses.push(res);
        }
    }

    /// Creates a new TestServer and performs the initialize and initialized sequence.
    /// The `init` closure is used to create the LanguageServer instance.
    async fn new_initialized<F>(init: F, initialize: Request) -> Self
    where
        F: FnOnce(Client) -> Backend,
    {
        let mut server = Self::new(init);
        let initialize_id = initialize.id().cloned();
        // Send initialize request
        server.send_request(initialize).await;
        let initialize_response = server.recv_response().await;
        assert!(initialize_response.is_ok());
        assert_eq!(Some(initialize_response.id()), initialize_id.as_ref());

        // Send initialized notification
        server.send_request(initialized_notification()).await;

        server
    }

    async fn shutdown(&mut self, id: i64) {
        self.send_request(shutdown_request(id)).await;
        let shutdown_result = self.recv_response().await;
        assert!(shutdown_result.is_ok());
        assert_eq!(shutdown_result.id(), &Id::Number(id));
    }

    async fn shutdown_with_diagnostic_clear(&mut self, id: i64, uris_to_clear: Vec<Uri>) {
        self.send_request(shutdown_request(id)).await;

        for uri in uris_to_clear {
            let publish_diagnostics = self.recv_notification().await;
            assert_eq!(publish_diagnostics.method(), "textDocument/publishDiagnostics");
            let params: PublishDiagnosticsParams =
                serde_json::from_value(publish_diagnostics.params().unwrap().clone()).unwrap();
            assert_eq!(params.uri, uri);
            assert_eq!(params.diagnostics, []);
        }

        let shutdown_result = self.recv_response().await;
        assert!(shutdown_result.is_ok());
        assert_eq!(shutdown_result.id(), &Id::Number(id));
    }
}

#[derive(Default)]
struct InitializeRequestOptions {
    workspace_configuration: bool,
    dynamic_watchers: bool,
    workspace_edit: bool,
    pull_mode: bool,
    initialization_options: Option<Value>,
    workspace_folders: Option<Vec<WorkspaceFolder>>,
    root_uri: Option<Uri>,
}

fn initialize_request_workspace_folders(options: InitializeRequestOptions) -> Request {
    let params = InitializeParams {
        workspace_folders_initialize_params: WorkspaceFoldersInitializeParams {
            workspace_folders: options.workspace_folders.map(WorkspaceFolders::WorkspaceFolderList),
        },
        capabilities: ClientCapabilities {
            text_document: Some(TextDocumentClientCapabilities {
                diagnostic: if options.pull_mode {
                    Some(DiagnosticClientCapabilities::default())
                } else {
                    None
                },
                ..Default::default()
            }),
            workspace: Some(WorkspaceClientCapabilities {
                apply_edit: Some(options.workspace_edit),
                configuration: Some(options.workspace_configuration),
                diagnostics: Some(DiagnosticWorkspaceClientCapabilities {
                    refresh_support: Some(options.pull_mode),
                }),
                did_change_watched_files: Some(DidChangeWatchedFilesClientCapabilities {
                    dynamic_registration: Some(options.dynamic_watchers),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        },
        initialization_options: options.initialization_options,
        #[expect(deprecated)]
        root_uri: options.root_uri,
        ..Default::default()
    };

    Request::build("initialize").params(json!(params)).id(1).finish()
}

fn initialize_request(mut options: InitializeRequestOptions) -> Request {
    options.workspace_folders = Some(vec![WorkspaceFolder {
        uri: Uri::from_str(WORKSPACE).unwrap(),
        name: "workspace".to_string(),
    }]);

    initialize_request_workspace_folders(options)
}

fn initialized_notification() -> Request {
    let params = InitializedParams {};

    Request::build("initialized").params(json!(params)).finish()
}

fn shutdown_request(id: i64) -> Request {
    Request::build("shutdown").id(id).finish()
}

fn execute_command_request(command: &str, arguments: &[serde_json::Value], id: i64) -> Request {
    Request::build("workspace/executeCommand")
        .id(id)
        .params(json!({
            "command": command,
            "arguments": arguments,
        }))
        .finish()
}

fn workspace_folders_changed(
    added: Vec<WorkspaceFolder>,
    removed: Vec<WorkspaceFolder>,
) -> Request {
    let params =
        DidChangeWorkspaceFoldersParams { event: WorkspaceFoldersChangeEvent { added, removed } };

    Request::build("workspace/didChangeWorkspaceFolders").params(json!(params)).finish()
}

async fn acknowledge_registrations(server: &mut TestServer) {
    // client/registerCapability request
    let register_request = server.recv_notification().await;
    assert_eq!(register_request.method(), "client/registerCapability");

    // Acknowledge the registration
    server.send_ack(register_request.id().unwrap()).await;
}

async fn acknowledge_unregistrations(server: &mut TestServer) {
    // client/unregisterCapability request
    let unregister_request = server.recv_notification().await;
    assert_eq!(unregister_request.method(), "client/unregisterCapability");

    // Acknowledge the unregistration
    server.send_ack(unregister_request.id().unwrap()).await;
}

async fn acknowledge_diagnostic_refresh(server: &mut TestServer) {
    let diagnostic_refresh_request = server.recv_notification().await;
    assert_eq!(diagnostic_refresh_request.method(), "workspace/diagnostic/refresh");
    server.send_ack(diagnostic_refresh_request.id().unwrap()).await;
}

async fn response_to_configuration(
    server: &mut TestServer,
    configurations: Vec<serde_json::Value>,
) {
    let workspace_config_request = server.recv_notification().await;
    assert_eq!(workspace_config_request.method(), "workspace/configuration");
    server
        .send_response(Response::from_ok(
            workspace_config_request.id().unwrap().clone(),
            json!(configurations),
        ))
        .await;
}

fn did_change_watched_files(uri: &str) -> Request {
    Request::build("workspace/didChangeWatchedFiles")
        .params(json!({
            "changes": [
                {
                    "uri": uri,
                    "type": 2 // Changed
                }
            ]
        }))
        .finish()
}

fn did_change_configuration(new_config: Option<serde_json::Value>) -> Request {
    Request::build("workspace/didChangeConfiguration")
        .params(json!(DidChangeConfigurationParams { settings: new_config.unwrap_or_default() }))
        .finish()
}

fn did_open(uri: &str, text: &str) -> Request {
    let params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: Uri::from_str(uri).unwrap(),
            language_id: LanguageKind::Plaintext,
            version: 1,
            text: text.to_string(),
        },
    };

    Request::build("textDocument/didOpen").params(json!(params)).finish()
}

fn did_change(uri: &str, text: &str) -> Request {
    let params = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            text_document_identifier: TextDocumentIdentifier { uri: Uri::from_str(uri).unwrap() },
            version: 2,
        },
        content_changes: vec![
            TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(
                TextDocumentContentChangeWholeDocument { text: text.to_string() },
            ),
        ],
    };

    Request::build("textDocument/didChange").params(json!(params)).finish()
}

fn did_save(uri: &str, text: &str) -> Request {
    let params = DidSaveTextDocumentParams {
        text_document: TextDocumentIdentifier { uri: Uri::from_str(uri).unwrap() },
        text: Some(text.to_string()),
    };

    Request::build("textDocument/didSave").params(json!(params)).finish()
}

fn did_close(uri: &str) -> Request {
    let params = DidCloseTextDocumentParams {
        text_document: TextDocumentIdentifier { uri: Uri::from_str(uri).unwrap() },
    };

    Request::build("textDocument/didClose").params(json!(params)).finish()
}

fn code_action(id: i64, uri: &str) -> Request {
    let params = CodeActionParams {
        text_document: TextDocumentIdentifier { uri: Uri::from_str(uri).unwrap() },
        range: Range::default(),
        context: CodeActionContext { diagnostics: vec![], only: None, trigger_kind: None },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    Request::build("textDocument/codeAction").id(id).params(json!(params)).finish()
}

fn test_workers_request(id: i64) -> Request {
    Request::build("test/workers").id(id).params(json!(null)).finish()
}

fn test_configuration_request(id: i64) -> Request {
    Request::build("test/configuration").id(id).params(json!(null)).finish()
}

fn diagnostic(id: i64, uri: &str) -> Request {
    let params = DocumentDiagnosticParams {
        text_document: TextDocumentIdentifier { uri: Uri::from_str(uri).unwrap() },
        identifier: None,
        previous_result_id: None,
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    Request::build("textDocument/diagnostic").id(id).params(json!(params)).finish()
}

fn create_workspace_manager() -> WorkerManager {
    WorkerManager::new(Arc::new(FakeToolBuilder::default()))
}

fn create_workspace_manager_with_builder(builder: FakeToolBuilder) -> WorkerManager {
    WorkerManager::new(Arc::new(builder))
}

fn create_dynamic_workspace_manager(builder: FakeToolBuilder) -> WorkerManager {
    WorkerManager::new_dynamic(Arc::new(builder))
}

/// `workingDirectories` turns directories below a workspace folder into their own workers.
/// These tests need a real directory tree, because the option is resolved against the file system.
#[cfg(test)]
mod working_directories_suite {
    use std::{
        fs,
        path::Path,
        sync::{Arc, Mutex},
    };

    use serde_json::json;
    use tower_lsp_server::{
        gen_lsp_types::{ServerInfo, Uri, WorkspaceFolder},
        jsonrpc::Id,
    };

    use crate::{
        DiagnosticMode,
        backend::Backend,
        tests::{
            FakeToolBuilder, InitializeRequestOptions, PublishDiagnosticsParams, Request,
            TestServer, acknowledge_diagnostic_refresh, acknowledge_registrations,
            acknowledge_unregistrations, create_workspace_manager,
            create_workspace_manager_with_builder, did_change_configuration,
            did_change_watched_files, did_open, initialize_request_workspace_folders,
            test_workers_request, workspace_folders_changed,
        },
    };

    fn server_info() -> ServerInfo {
        ServerInfo { name: "oxc".to_owned(), version: Some("1.0.0".to_owned()) }
    }

    /// A workspace folder with `packages/a` and `packages/b`.
    fn create_workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a")).unwrap();
        fs::create_dir_all(dir.path().join("packages/b")).unwrap();
        dir
    }

    fn uri(path: &Path) -> Uri {
        crate::uri_utils::file_path_to_uri(path).unwrap()
    }

    fn workspace_folder(path: &Path) -> WorkspaceFolder {
        WorkspaceFolder { uri: uri(path), name: "workspace".to_string() }
    }

    fn initialize(path: &Path, working_directories: &serde_json::Value) -> Request {
        initialize_request_workspace_folders(InitializeRequestOptions {
            initialization_options: Some(json!([{
                "workspaceUri": uri(path).as_str(),
                "options": { "workingDirectories": working_directories },
            }])),
            workspace_folders: Some(vec![workspace_folder(path)]),
            ..Default::default()
        })
    }

    async fn workers(server: &mut TestServer, id: i64) -> Vec<(String, Option<String>)> {
        server.send_request(test_workers_request(id)).await;
        let response = server.recv_response().await;
        assert_eq!(response.id(), &Id::Number(id));
        assert!(response.is_ok(), "{response:?}");

        response
            .result()
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|worker| {
                (
                    worker["root"].as_str().unwrap().to_string(),
                    worker["parent"].as_str().map(ToString::to_string),
                )
            })
            .collect()
    }

    /// The ids of the registrations (or unregistrations) of a client request.
    fn ids(registrations: &serde_json::Value) -> Vec<String> {
        registrations
            .as_array()
            .unwrap()
            .iter()
            .map(|registration| registration["id"].as_str().unwrap().to_string())
            .collect()
    }

    /// The excluded roots of the worker rooted at `root`.
    async fn excluded_roots(server: &mut TestServer, id: i64, root: &Uri) -> Vec<String> {
        server.send_request(test_workers_request(id)).await;
        let response = server.recv_response().await;
        assert_eq!(response.id(), &Id::Number(id));

        response
            .result()
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .find(|worker| worker["root"].as_str() == Some(root.as_str()))
            .map(|worker| {
                worker["excluded"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|root| root.as_str().unwrap().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    #[tokio::test]
    async fn test_sub_workers_are_created_on_initialize() {
        let dir = create_workspace();
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize(dir.path(), &json!(["packages/a", "packages/b"])),
        )
        .await;

        let workers = workers(&mut server, 2).await;
        assert_eq!(workers.len(), 3);
        // the workspace folder worker comes first and has no parent
        assert_eq!(workers[0], (uri(dir.path()).as_str().to_string(), None));
        assert_eq!(
            workers[1],
            (
                uri(&dir.path().join("packages/a")).as_str().to_string(),
                Some(uri(dir.path()).as_str().to_string())
            )
        );
        assert_eq!(
            workers[2],
            (
                uri(&dir.path().join("packages/b")).as_str().to_string(),
                Some(uri(dir.path()).as_str().to_string())
            )
        );

        server.shutdown(3).await;
    }

    /// Semantics rule 9: an absent or empty option changes nothing.
    #[tokio::test]
    async fn test_no_sub_workers_without_the_option() {
        let dir = create_workspace();
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize(dir.path(), &json!([])),
        )
        .await;

        assert_eq!(workers(&mut server, 2).await.len(), 1);

        server.shutdown(3).await;
    }

    /// Semantics rule 10: a tool which does not use working directories gets no sub worker.
    #[tokio::test]
    async fn test_no_sub_workers_when_the_tool_does_not_use_the_option() {
        let dir = create_workspace();
        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(
                        FakeToolBuilder::default().with_working_directories_disabled(),
                    ),
                )
            },
            initialize(dir.path(), &json!(["packages/a", "packages/b"])),
        )
        .await;

        assert_eq!(workers(&mut server, 2).await.len(), 1);

        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_invalid_entries_are_ignored() {
        let dir = create_workspace();
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize(dir.path(), &json!(["../escaping", "/absolute", "packages/a"])),
        )
        .await;

        // the rejected entries are reported to the client as warnings
        for _ in 0..2 {
            let message = server.recv_notification().await;
            assert_eq!(message.method(), "window/showMessage");
        }

        let workers = workers(&mut server, 2).await;
        assert_eq!(workers.len(), 2);
        assert_eq!(workers[1].0, uri(&dir.path().join("packages/a")).as_str());

        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_sub_workers_are_synced_on_configuration_change() {
        let dir = create_workspace();
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize(dir.path(), &json!(["packages/a", "packages/b"])),
        )
        .await;
        assert_eq!(workers(&mut server, 2).await.len(), 3);

        // `packages/b` is not a working directory anymore, its worker is shut down
        server
            .send_request(did_change_configuration(Some(json!([{
                "workspaceUri": uri(dir.path()).as_str(),
                "options": { "workingDirectories": ["packages/a"] },
            }]))))
            .await;
        // the watchers of the removed worker are unregistered
        acknowledge_unregistrations(&mut server).await;

        let workers_after_removal = workers(&mut server, 3).await;
        assert_eq!(workers_after_removal.len(), 2);
        assert_eq!(workers_after_removal[1].0, uri(&dir.path().join("packages/a")).as_str());

        // `packages/b` is a working directory again, its worker is created
        server
            .send_request(did_change_configuration(Some(json!([{
                "workspaceUri": uri(dir.path()).as_str(),
                "options": { "workingDirectories": ["packages/a", "packages/b"] },
            }]))))
            .await;

        assert_eq!(workers(&mut server, 4).await.len(), 3);

        server.shutdown(5).await;
    }

    #[tokio::test]
    async fn test_sub_workers_are_removed_with_their_workspace_folder() {
        let dir = create_workspace();
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize(dir.path(), &json!(["packages/a", "packages/b"])),
        )
        .await;
        assert_eq!(workers(&mut server, 2).await.len(), 3);

        server
            .send_request(workspace_folders_changed(vec![], vec![workspace_folder(dir.path())]))
            .await;

        assert_eq!(workers(&mut server, 3).await, []);

        server.shutdown(4).await;
    }

    /// Semantics rule 1: a sub worker created for a configuration which spells the workspace
    /// folder differently still goes away with that folder.
    #[tokio::test]
    async fn test_sub_workers_created_for_another_spelling_are_removed_with_their_folder() {
        let dir = create_workspace();
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize(dir.path(), &json!(["packages/a"])),
        )
        .await;

        server
            .send_request(did_change_configuration(Some(json!([{
                // a trailing slash is the same directory
                "workspaceUri": format!("{}/", uri(dir.path()).as_str()),
                "options": { "workingDirectories": ["packages/a", "packages/b"] },
            }]))))
            .await;
        assert_eq!(workers(&mut server, 2).await.len(), 3);

        server
            .send_request(workspace_folders_changed(vec![], vec![workspace_folder(dir.path())]))
            .await;
        assert_eq!(workers(&mut server, 3).await, []);

        server.shutdown(4).await;
    }

    #[tokio::test]
    async fn test_a_workspace_folder_is_never_shadowed_by_a_sub_worker() {
        let dir = create_workspace();
        let package_a = dir.path().join("packages/a");

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request_workspace_folders(InitializeRequestOptions {
                initialization_options: Some(json!([{
                    "workspaceUri": uri(dir.path()).as_str(),
                    "options": { "workingDirectories": ["packages/a", "packages/b"] },
                }])),
                workspace_folders: Some(vec![
                    workspace_folder(dir.path()),
                    WorkspaceFolder { uri: uri(&package_a), name: "a".to_string() },
                ]),
                ..Default::default()
            }),
        )
        .await;

        let workers = workers(&mut server, 2).await;
        // the workspace folder, the workspace folder the client opened for `packages/a` and the
        // sub worker for `packages/b`
        assert_eq!(workers.len(), 3, "{workers:?}");

        let for_package_a =
            workers.iter().filter(|(root, _)| root == uri(&package_a).as_str()).collect::<Vec<_>>();
        assert_eq!(for_package_a.len(), 1, "{workers:?}");
        // it stayed the client workspace folder worker, it was not replaced by a sub worker
        assert_eq!(for_package_a[0].1, None);

        server.shutdown(3).await;
    }

    /// Semantics rule 4: the workspace folder excludes its working directories and each working
    /// directory excludes the ones nested below it.
    #[tokio::test]
    async fn test_nested_working_directories_exclude_each_other() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a/nested")).unwrap();
        let package_a = uri(&dir.path().join("packages/a"));
        let nested = uri(&dir.path().join("packages/a/nested"));

        let manager = create_workspace_manager();
        let folder = manager.create_worker(uri(dir.path()), DiagnosticMode::None);
        let (sub_workers, _messages) = manager
            .start_folder_worker(
                &folder,
                json!({ "workingDirectories": ["packages/a", "packages/a/nested"] }),
                &DiagnosticMode::None,
                &[uri(dir.path())],
                &[],
            )
            .await;

        // the workspace folder owns neither of them
        assert_eq!(folder.get_working_directories().await, vec![package_a.clone(), nested.clone()]);

        assert_eq!(sub_workers.len(), 2);
        assert_eq!(sub_workers[0].get_root_uri(), &package_a);
        // the outer working directory excludes the one nested inside it
        assert_eq!(sub_workers[0].get_working_directories().await, vec![nested.clone()]);
        assert_eq!(sub_workers[1].get_root_uri(), &nested);
        assert_eq!(sub_workers[1].get_working_directories().await, Vec::<Uri>::new());
    }

    /// Same for a working directory which contains another one: dropping the inner one changes the
    /// build context of the outer one.
    #[tokio::test]
    async fn test_a_surviving_sub_worker_is_rebuilt_when_its_nested_roots_change() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a/nested")).unwrap();

        let builds = Arc::new(Mutex::new(vec![]));
        let tracked = Arc::clone(&builds);

        let mut server = TestServer::new_initialized(
            move |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(
                        FakeToolBuilder::default().with_build_tracking(tracked),
                    ),
                )
            },
            initialize(dir.path(), &json!(["packages/a", "packages/a/nested"])),
        )
        .await;

        assert_eq!(workers(&mut server, 2).await.len(), 3);
        builds.lock().unwrap().clear();

        // `packages/a/nested` belongs to `packages/a` again
        server
            .send_request(did_change_configuration(Some(json!([{
                "workspaceUri": uri(dir.path()).as_str(),
                "options": { "workingDirectories": ["packages/a"] },
            }]))))
            .await;
        acknowledge_unregistrations(&mut server).await;

        assert_eq!(workers(&mut server, 3).await.len(), 2);
        let builds = builds.lock().unwrap().clone();
        assert!(
            builds.iter().any(|root| root == &uri(&dir.path().join("packages/a"))),
            "the surviving sub worker was not rebuilt: {builds:?}"
        );

        server.shutdown(4).await;
    }

    #[tokio::test]
    async fn test_sub_worker_changes_refresh_the_diagnostics_in_pull_mode() {
        let dir = create_workspace();
        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(FakeToolBuilder::new(
                        DiagnosticMode::Pull,
                    )),
                )
            },
            initialize_request_workspace_folders(InitializeRequestOptions {
                pull_mode: true,
                initialization_options: Some(json!([{
                    "workspaceUri": uri(dir.path()).as_str(),
                    "options": { "workingDirectories": [] },
                }])),
                workspace_folders: Some(vec![workspace_folder(dir.path())]),
                ..Default::default()
            }),
        )
        .await;

        let document = format!("{}/packages/a/diagnostics.config", uri(dir.path()).as_str());
        server.send_request(did_open(&document, "some text")).await;
        // round trip to make sure the document is known before the configuration changes
        assert_eq!(workers(&mut server, 2).await.len(), 1);

        // the document is handed over to a new sub worker, the client has to ask for its
        // diagnostics again
        server
            .send_request(did_change_configuration(Some(json!([{
                "workspaceUri": uri(dir.path()).as_str(),
                "options": { "workingDirectories": ["packages/a"] },
            }]))))
            .await;
        acknowledge_diagnostic_refresh(&mut server).await;

        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_rebuild_does_not_relint_the_files_of_a_sub_worker() {
        let dir = create_workspace();
        let linted = Arc::new(Mutex::new(vec![]));
        let tracked = Arc::clone(&linted);

        let mut server = TestServer::new_initialized(
            move |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(
                        FakeToolBuilder::new(DiagnosticMode::Push).with_cache_tracking(tracked),
                    ),
                )
            },
            initialize(dir.path(), &json!(["packages/a", "packages/b"])),
        )
        .await;

        let document = format!("{}/packages/a/diagnostics.config", uri(dir.path()).as_str());
        server.send_request(did_open(&document, "some text")).await;
        let notification = server.recv_notification().await;
        assert_eq!(notification.method(), "textDocument/publishDiagnostics");

        linted.lock().unwrap().clear();

        // every worker rebuilds its tool, but only the worker responsible for the document
        // revalidates it
        server
            .send_request(did_change_watched_files(
                format!("{}/tool.config", uri(dir.path()).as_str()).as_str(),
            ))
            .await;
        let notification = server.recv_notification().await;
        assert_eq!(notification.method(), "textDocument/publishDiagnostics");

        let linted = linted.lock().unwrap().clone();
        assert_eq!(
            linted.iter().filter(|uri| uri.as_str() == document).count(),
            1,
            "the document was linted more than once: {linted:?}"
        );

        server.shutdown_with_diagnostic_clear(3, vec![document.parse().unwrap()]).await;
    }

    /// Semantics rule 5: a document whose re-lint finds nothing is not a document which was
    /// published for. The diagnostics its previous owner left behind still have to be cleared.
    #[tokio::test]
    async fn test_an_orphaned_document_which_is_clean_now_is_cleared() {
        let dir = create_workspace();
        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(FakeToolBuilder::new(
                        DiagnosticMode::Push,
                    )),
                )
            },
            initialize(dir.path(), &json!(["packages/a"])),
        )
        .await;

        // one document of the sub worker, one of the workspace folder worker
        let orphaned = format!("{}/packages/a/diagnostics.config", uri(dir.path()).as_str());
        let kept = format!("{}/packages/b/diagnostics.config", uri(dir.path()).as_str());
        for document in [&orphaned, &kept] {
            server.send_request(did_open(document, "some text")).await;
            let notification = server.recv_notification().await;
            assert_eq!(notification.method(), "textDocument/publishDiagnostics");
        }

        // the sub worker disappears and the workspace folder worker, which takes its document
        // over, reports it as clean now
        server
            .send_request(did_change_configuration(Some(json!([{
                "workspaceUri": uri(dir.path()).as_str(),
                "options": { "workingDirectories": [], "cleanUris": [&orphaned] },
            }]))))
            .await;

        // the document of the removed worker is cleared, it has no diagnostics anymore
        let notification = server.recv_notification().await;
        assert_eq!(notification.method(), "textDocument/publishDiagnostics");
        let params: PublishDiagnosticsParams =
            serde_json::from_value(notification.params().unwrap().clone()).unwrap();
        assert_eq!(params.uri.as_str(), orphaned);
        assert!(params.diagnostics.is_empty(), "{:?}", params.diagnostics);

        // the other document is published by the restarted worker, as usual
        let notification = server.recv_notification().await;
        assert_eq!(notification.method(), "textDocument/publishDiagnostics");
        let params: PublishDiagnosticsParams =
            serde_json::from_value(notification.params().unwrap().clone()).unwrap();
        assert_eq!(params.uri.as_str(), kept);
        assert_eq!(params.diagnostics.len(), 1);

        // the watchers of the removed sub worker
        acknowledge_unregistrations(&mut server).await;

        server.shutdown_with_diagnostic_clear(2, vec![kept.parse().unwrap()]).await;
    }

    /// Semantics rule 5: the open documents are revalidated exactly once by their new owner.
    /// The rebuild of the affected workers is covered by
    /// `test_the_folder_worker_is_rebuilt_when_its_working_directories_change` and the pull mode
    /// refresh by `test_sub_worker_changes_refresh_the_diagnostics_in_pull_mode`.
    #[tokio::test]
    async fn test_creating_and_removing_a_sub_worker_relints_its_documents() {
        let dir = create_workspace();
        let linted = Arc::new(Mutex::new(vec![]));
        let tracked = Arc::clone(&linted);
        // how many roots the tool which linted a document was built with as excluded
        let contexts = Arc::new(Mutex::new(vec![]));
        let tracked_contexts = Arc::clone(&contexts);

        let mut server = TestServer::new_initialized(
            move |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(
                        FakeToolBuilder::new(DiagnosticMode::Push)
                            .with_cache_tracking(tracked)
                            .with_diagnostic_context_tracking(tracked_contexts),
                    ),
                )
            },
            initialize(dir.path(), &json!([])),
        )
        .await;

        let document = format!("{}/packages/a/diagnostics.config", uri(dir.path()).as_str());
        server.send_request(did_open(&document, "some text")).await;
        let notification = server.recv_notification().await;
        assert_eq!(notification.method(), "textDocument/publishDiagnostics");

        linted.lock().unwrap().clear();

        // the new sub worker takes the document over and has to lint it itself
        server
            .send_request(did_change_configuration(Some(json!([{
                "workspaceUri": uri(dir.path()).as_str(),
                "options": { "workingDirectories": ["packages/a"] },
            }]))))
            .await;
        let notification = server.recv_notification().await;
        assert_eq!(notification.method(), "textDocument/publishDiagnostics");
        assert!(
            linted.lock().unwrap().iter().any(|uri| uri.as_str() == document),
            "the new sub worker did not lint the document it took over"
        );
        assert_eq!(workers(&mut server, 2).await.len(), 2);

        linted.lock().unwrap().clear();
        contexts.lock().unwrap().clear();

        // removing it hands the document back to the workspace folder worker
        server
            .send_request(did_change_configuration(Some(json!([{
                "workspaceUri": uri(dir.path()).as_str(),
                "options": { "workingDirectories": [] },
            }]))))
            .await;

        // the new owner publishes once, the diagnostics of the removed worker are not cleared
        // first: that would publish the same document twice and make the editor flicker
        let notification = server.recv_notification().await;
        assert_eq!(notification.method(), "textDocument/publishDiagnostics");
        let params: PublishDiagnosticsParams =
            serde_json::from_value(notification.params().unwrap().clone()).unwrap();
        assert_eq!(params.uri.as_str(), document);
        assert_eq!(params.diagnostics.len(), 1);
        // the next message is the watcher unregistration of the removed worker, not a second
        // publish for the same document
        acknowledge_unregistrations(&mut server).await;

        assert!(
            linted.lock().unwrap().iter().any(|uri| uri.as_str() == document),
            "the workspace folder worker did not lint the orphaned document"
        );
        // and it did so with the rebuilt tool: the working directory is not excluded anymore
        let contexts = contexts.lock().unwrap().clone();
        assert_eq!(contexts, vec![0], "the orphaned document was linted by a stale tool");
        assert_eq!(workers(&mut server, 3).await.len(), 1);

        server.shutdown_with_diagnostic_clear(4, vec![document.parse().unwrap()]).await;
    }

    /// Semantics rule 8: the validation warnings are reported when the option value changes, not
    /// again on every later reconciliation.
    #[tokio::test]
    async fn test_validation_warnings_are_only_reported_when_the_option_changes() {
        let dir = create_workspace();
        let options = json!({ "workingDirectories": ["packages/a", "../escaping"] });

        let manager = create_workspace_manager();
        let folder = manager.create_worker(uri(dir.path()), DiagnosticMode::None);
        let (sub_workers, messages) = manager
            .start_folder_worker(
                &folder,
                options.clone(),
                &DiagnosticMode::None,
                &[uri(dir.path())],
                &[],
            )
            .await;

        // the rejected entry is reported once, when the option is first seen
        assert_eq!(messages.len(), 1, "{messages:?}");
        assert!(messages[0].message.contains("escapes the workspace folder"));

        let mut workers = vec![folder];
        workers.extend(sub_workers);
        manager.add_workers(workers).await;

        // a reconciliation with the very same option value
        let sync = manager
            .sync_sub_workers(&uri(dir.path()), &options, &DiagnosticMode::None, false, false, true)
            .await;
        assert!(sync.client_messages.is_empty(), "{:?}", sync.client_messages);

        // an actual option change reports it again
        let sync = manager
            .sync_sub_workers(&uri(dir.path()), &options, &DiagnosticMode::None, false, true, true)
            .await;
        assert_eq!(sync.client_messages.len(), 1, "{:?}", sync.client_messages);
    }

    /// Semantics rule 8: an option for a folder which is not served reports nothing.
    #[tokio::test]
    async fn test_options_of_an_unknown_folder_report_nothing() {
        let dir = create_workspace();
        let manager = create_workspace_manager();

        let sync = manager
            .sync_sub_workers(
                &uri(dir.path()),
                &json!({ "workingDirectories": ["../escaping"] }),
                &DiagnosticMode::None,
                false,
                true,
                true,
            )
            .await;

        assert!(sync.client_messages.is_empty(), "{:?}", sync.client_messages);
    }

    /// Semantics rule 5: a configuration change which lands before `initialized` started the
    /// workspace folder leaves it to `initialized`, which resolves the option itself.
    #[tokio::test]
    async fn test_a_folder_which_is_not_started_is_not_reconciled() {
        let dir = create_workspace();
        let manager = create_workspace_manager();
        let folder = manager.create_worker(uri(dir.path()), DiagnosticMode::None);
        manager.add_workers(vec![folder]).await;

        let sync = manager
            .sync_sub_workers(
                &uri(dir.path()),
                &json!({ "workingDirectories": ["packages/a", "../escaping"] }),
                &DiagnosticMode::None,
                true,
                true,
                true,
            )
            .await;

        assert_eq!(sync.added_roots.len(), 0);
        assert_eq!(sync.registrations, []);
        assert!(sync.client_messages.is_empty(), "{:?}", sync.client_messages);
        assert_eq!(manager.read_workspace_workers().await.len(), 1);
    }

    /// Semantics rule 1: single-file workers have no working directories.
    #[tokio::test]
    async fn test_single_file_workers_have_no_working_directories() {
        let dir = create_workspace();
        let manager = create_workspace_manager();
        manager.set_single_file_mode(true);
        let worker = manager.create_worker(uri(dir.path()), DiagnosticMode::None);
        worker.start_worker(json!({})).await;
        manager.add_workers(vec![worker]).await;

        let sync = manager
            .sync_sub_workers(
                &uri(dir.path()),
                &json!({ "workingDirectories": ["packages/a"] }),
                &DiagnosticMode::None,
                false,
                true,
                true,
            )
            .await;

        assert_eq!(sync.added_roots.len(), 0);
        assert_eq!(manager.read_workspace_workers().await.len(), 1);
    }

    /// Semantics rule 1: two reconciliations running at once create one sub worker per root.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_concurrent_reconciliations_create_one_sub_worker_per_root() {
        let dir = create_workspace();
        let options = json!({ "workingDirectories": ["packages/a"] });

        let manager = Arc::new(create_workspace_manager());
        let folder = manager.create_worker(uri(dir.path()), DiagnosticMode::None);
        folder.start_worker(options.clone()).await;
        manager.add_workers(vec![folder]).await;

        let root = uri(dir.path());
        let reconcile = || {
            let (manager, root, options) = (Arc::clone(&manager), root.clone(), options.clone());
            tokio::spawn(async move {
                manager
                    .sync_sub_workers(&root, &options, &DiagnosticMode::None, false, false, true)
                    .await;
            })
        };
        let (first, second) = (reconcile(), reconcile());
        first.await.unwrap();
        second.await.unwrap();

        // the workspace folder worker and one sub worker
        assert_eq!(manager.read_workspace_workers().await.len(), 2);
    }

    /// There is exactly one worker, and one watcher registration, per root: a workspace folder
    /// opened on a working directory replaces its sub worker.
    #[tokio::test]
    async fn test_a_workspace_folder_added_on_a_working_directory_replaces_its_sub_worker() {
        let dir = create_workspace();
        let package_a = uri(&dir.path().join("packages/a"));

        let builds = Arc::new(Mutex::new(vec![]));
        let tracked = Arc::clone(&builds);

        let mut server = TestServer::new_initialized(
            move |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(
                        FakeToolBuilder::new(DiagnosticMode::Pull).with_build_tracking(tracked),
                    ),
                )
            },
            initialize_request_workspace_folders(InitializeRequestOptions {
                pull_mode: true,
                initialization_options: Some(json!([{
                    "workspaceUri": uri(dir.path()).as_str(),
                    "options": { "workingDirectories": ["packages/a", "packages/b"] },
                }])),
                workspace_folders: Some(vec![workspace_folder(dir.path())]),
                ..Default::default()
            }),
        )
        .await;

        let before = workers(&mut server, 2).await;
        assert_eq!(before.len(), 3);
        assert!(
            before.iter().any(|(root, parent)| root == package_a.as_str() && parent.is_some()),
            "{before:?}"
        );

        // an open document below the working directory changes owner with it
        let document = format!("{}/diagnostics.config", package_a.as_str());
        server.send_request(did_open(&document, "some text")).await;
        assert_eq!(workers(&mut server, 3).await.len(), 3);
        builds.lock().unwrap().clear();

        server
            .send_request(workspace_folders_changed(
                vec![WorkspaceFolder { uri: package_a.clone(), name: "a".to_string() }],
                vec![],
            ))
            .await;

        // the document is served by another worker now, the client has to ask again
        acknowledge_diagnostic_refresh(&mut server).await;

        // the resolved working directories did not change, so the workspace folder worker keeps
        // its build context and must not be rebuilt
        let builds = builds.lock().unwrap().clone();
        assert!(!builds.contains(&uri(dir.path())), "{builds:?}");

        let after = workers(&mut server, 4).await;
        assert_eq!(after.len(), 3, "{after:?}");
        let for_package_a =
            after.iter().filter(|(root, _)| root == package_a.as_str()).collect::<Vec<_>>();
        assert_eq!(for_package_a.len(), 1, "{after:?}");
        // it is the client workspace folder worker now, the sub worker was shut down
        assert_eq!(for_package_a[0].1, None);

        server.shutdown(5).await;
    }

    /// The other direction: removing the workspace folder hands the root back to a sub worker.
    #[tokio::test]
    async fn test_removing_a_workspace_folder_gives_its_root_back_to_a_sub_worker() {
        let dir = create_workspace();
        let package_a = uri(&dir.path().join("packages/a"));

        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(FakeToolBuilder::new(
                        DiagnosticMode::Pull,
                    )),
                )
            },
            initialize_request_workspace_folders(InitializeRequestOptions {
                pull_mode: true,
                initialization_options: Some(json!([{
                    "workspaceUri": uri(dir.path()).as_str(),
                    "options": { "workingDirectories": ["packages/a", "packages/b"] },
                }])),
                workspace_folders: Some(vec![
                    workspace_folder(dir.path()),
                    WorkspaceFolder { uri: package_a.clone(), name: "a".to_string() },
                ]),
                ..Default::default()
            }),
        )
        .await;

        let before = workers(&mut server, 2).await;
        assert_eq!(before.len(), 3, "{before:?}");
        assert!(
            before.iter().any(|(root, parent)| root == package_a.as_str() && parent.is_none()),
            "{before:?}"
        );

        // an open document below the working directory changes owner with it
        let document = format!("{}/diagnostics.config", package_a.as_str());
        server.send_request(did_open(&document, "some text")).await;
        assert_eq!(workers(&mut server, 3).await.len(), 3);

        server
            .send_request(workspace_folders_changed(
                vec![],
                vec![WorkspaceFolder { uri: package_a.clone(), name: "a".to_string() }],
            ))
            .await;

        // the document is served by another worker now, the client has to ask again
        acknowledge_diagnostic_refresh(&mut server).await;

        let after = workers(&mut server, 4).await;
        assert_eq!(after.len(), 3, "{after:?}");
        let for_package_a =
            after.iter().filter(|(root, _)| root == package_a.as_str()).collect::<Vec<_>>();
        assert_eq!(for_package_a.len(), 1, "{after:?}");
        // a sub worker serves it again
        assert_eq!(for_package_a[0].1, Some(uri(dir.path()).as_str().to_string()));

        server.shutdown(5).await;
    }

    /// Semantics rule 5 across a workspace folder removal: the sub worker which takes the root
    /// back publishes for the open document, and the worker which was shut down does not clear it
    /// first.
    #[tokio::test]
    async fn test_a_removed_workspace_folder_does_not_clear_what_the_sub_worker_republishes() {
        let dir = create_workspace();
        let package_a = uri(&dir.path().join("packages/a"));
        let linted = Arc::new(Mutex::new(vec![]));
        let tracked = Arc::clone(&linted);

        let mut server = TestServer::new_initialized(
            move |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(
                        FakeToolBuilder::new(DiagnosticMode::Push).with_cache_tracking(tracked),
                    ),
                )
            },
            initialize_request_workspace_folders(InitializeRequestOptions {
                initialization_options: Some(json!([{
                    "workspaceUri": uri(dir.path()).as_str(),
                    "options": { "workingDirectories": ["packages/a"] },
                }])),
                workspace_folders: Some(vec![
                    workspace_folder(dir.path()),
                    WorkspaceFolder { uri: package_a.clone(), name: "a".to_string() },
                ]),
                ..Default::default()
            }),
        )
        .await;

        // the workspace folder worker of `packages/a` owns the document, no sub worker shadows it
        let before = workers(&mut server, 2).await;
        assert_eq!(before.len(), 2, "{before:?}");
        assert!(
            before.iter().any(|(root, parent)| root == package_a.as_str() && parent.is_none()),
            "{before:?}"
        );

        let document = format!("{}/diagnostics.config", package_a.as_str());
        server.send_request(did_open(&document, "some text")).await;
        let notification = server.recv_notification().await;
        assert_eq!(notification.method(), "textDocument/publishDiagnostics");

        linted.lock().unwrap().clear();

        server
            .send_request(workspace_folders_changed(
                vec![],
                vec![WorkspaceFolder { uri: package_a.clone(), name: "a".to_string() }],
            ))
            .await;

        // the sub worker which takes the root back publishes once: the diagnostics of the removed
        // folder worker are not cleared first, which would publish the same document twice
        let notification = server.recv_notification().await;
        assert_eq!(notification.method(), "textDocument/publishDiagnostics");
        let params: PublishDiagnosticsParams =
            serde_json::from_value(notification.params().unwrap().clone()).unwrap();
        assert_eq!(params.uri.as_str(), document);
        assert_eq!(params.diagnostics.len(), 1);
        assert!(
            linted.lock().unwrap().iter().any(|uri| uri.as_str() == document),
            "the sub worker did not lint the document it took over"
        );

        let after = workers(&mut server, 3).await;
        assert_eq!(after.len(), 2, "{after:?}");
        let for_package_a =
            after.iter().filter(|(root, _)| root == package_a.as_str()).collect::<Vec<_>>();
        assert_eq!(for_package_a.len(), 1, "{after:?}");
        assert_eq!(for_package_a[0].1, Some(uri(dir.path()).as_str().to_string()));

        // the only remaining publish is the one the shutdown sends: there was no second one
        server.shutdown_with_diagnostic_clear(4, vec![document.parse().unwrap()]).await;
    }

    /// A root which a workspace folder claims is still excluded by the working directory above it:
    /// the exclusions are computed from the resolved set, not from the sub workers which exist.
    #[tokio::test]
    async fn test_a_claimed_nested_root_stays_excluded_across_reconciliations() {
        let dir = create_workspace();
        fs::create_dir_all(dir.path().join("packages/a/nested")).unwrap();
        let package_a = uri(&dir.path().join("packages/a"));
        let nested = uri(&dir.path().join("packages/a/nested"));

        let options = json!([{
            "workspaceUri": uri(dir.path()).as_str(),
            "options": { "workingDirectories": ["packages/a", "packages/a/nested"] },
        }]);

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request_workspace_folders(InitializeRequestOptions {
                initialization_options: Some(options.clone()),
                workspace_folders: Some(vec![
                    workspace_folder(dir.path()),
                    WorkspaceFolder { uri: nested.clone(), name: "nested".to_string() },
                ]),
                ..Default::default()
            }),
        )
        .await;

        assert_eq!(
            excluded_roots(&mut server, 2, &package_a).await,
            vec![nested.as_str().to_string()]
        );

        // a reconciliation with the very same option keeps it excluded
        server.send_request(did_change_configuration(Some(options))).await;
        assert_eq!(
            excluded_roots(&mut server, 3, &package_a).await,
            vec![nested.as_str().to_string()]
        );

        server.shutdown(4).await;
    }

    /// Semantics rule 5: a rebuild which changes the watcher patterns of a worker re-registers
    /// them, here because the config it extended from outside the root is not discovered anymore.
    #[tokio::test]
    async fn test_a_rebuild_re_registers_the_watchers_when_the_patterns_change() {
        let dir = create_workspace();
        let package_a = uri(&dir.path().join("packages/a"));
        let marker = package_a.clone();

        let mut server = TestServer::new_initialized(
            move |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(
                        FakeToolBuilder::default()
                            .with_discovered_extends(marker, "/outside/shared.json".to_string()),
                    ),
                )
            },
            initialize_request_workspace_folders(InitializeRequestOptions {
                dynamic_watchers: true,
                initialization_options: Some(json!([{
                    "workspaceUri": uri(dir.path()).as_str(),
                    "options": { "workingDirectories": [] },
                }])),
                workspace_folders: Some(vec![workspace_folder(dir.path())]),
                ..Default::default()
            }),
        )
        .await;

        // the workspace folder worker watches the extended config
        acknowledge_registrations(&mut server).await;

        // `packages/a` becomes a working directory, so its config is not discovered by the
        // workspace folder worker anymore and the absolute watcher pattern disappears with it
        server
            .send_request(did_change_configuration(Some(json!([{
                "workspaceUri": uri(dir.path()).as_str(),
                "options": { "workingDirectories": ["packages/a"] },
            }]))))
            .await;

        acknowledge_unregistrations(&mut server).await;

        let register_request = server.recv_notification().await;
        assert_eq!(register_request.method(), "client/registerCapability");
        let params: serde_json::Value = register_request.params().unwrap().clone();
        let registrations = params["registrations"].as_array().unwrap();
        // only the new sub worker watches it now
        assert_eq!(registrations.len(), 1, "{registrations:?}");
        assert_eq!(registrations[0]["id"], format!("watcher-{}", package_a.as_str()));
        server.send_ack(register_request.id().unwrap()).await;

        server.shutdown(2).await;
    }

    #[tokio::test]
    async fn test_sub_workers_are_created_with_a_new_workspace_folder() {
        let first = create_workspace();
        let second = create_workspace();

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize(first.path(), &json!([])),
        )
        .await;
        assert_eq!(workers(&mut server, 2).await.len(), 1);

        server
            .send_request(workspace_folders_changed(vec![workspace_folder(second.path())], vec![]))
            .await;

        // the new folder is started with the default options, which declare no working
        // directories, so it only adds its own worker
        assert_eq!(workers(&mut server, 3).await.len(), 2);

        server.shutdown(4).await;
    }

    /// Semantics rule 7: `mode` and `pattern` are not supported in this version. The entries are
    /// reported to the client and create no sub worker.
    #[tokio::test]
    async fn test_unsupported_entries_are_reported_and_create_nothing() {
        let dir = create_workspace();
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize(dir.path(), &json!([{ "mode": "auto" }, { "pattern": "packages/*" }])),
        )
        .await;

        for _ in 0..2 {
            let message = server.recv_notification().await;
            assert_eq!(message.method(), "window/showMessage");
            let params = message.params().unwrap().clone();
            assert!(
                params["message"].as_str().unwrap().contains("not supported in this version"),
                "{params:?}"
            );
        }

        assert_eq!(workers(&mut server, 2).await.len(), 1);

        server.shutdown(3).await;
    }

    /// The build context of the workspace folder worker changes when a working directory appears
    /// or disappears, even though its options stay the same: what used to be owned by the sub
    /// worker (its config and its ignore files) belongs to the workspace folder again.
    #[tokio::test]
    async fn test_the_folder_worker_is_rebuilt_when_its_working_directories_change() {
        let dir = create_workspace();

        let builds = Arc::new(Mutex::new(vec![]));
        let tracked = Arc::clone(&builds);

        let mut server = TestServer::new_initialized(
            move |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(
                        FakeToolBuilder::default().with_build_tracking(tracked),
                    ),
                )
            },
            initialize(dir.path(), &json!(["packages/a"])),
        )
        .await;

        assert_eq!(workers(&mut server, 2).await.len(), 2);
        builds.lock().unwrap().clear();

        // `packages/a` belongs to the workspace folder worker again
        server
            .send_request(did_change_configuration(Some(json!([{
                "workspaceUri": uri(dir.path()).as_str(),
                "options": { "workingDirectories": [] },
            }]))))
            .await;
        // the watchers of the removed sub worker
        acknowledge_unregistrations(&mut server).await;

        assert_eq!(workers(&mut server, 3).await.len(), 1);
        // the rebuild of the workspace folder worker happens before the notification is done
        let builds = builds.lock().unwrap().clone();
        assert_eq!(
            builds,
            vec![uri(dir.path())],
            "the workspace folder worker was not rebuilt once"
        );

        server.shutdown(4).await;
    }

    /// Semantics rule 6: a watched file event is handed to every worker, whatever its root: a
    /// config of a working directory is still read by the workspace folder worker, and a config of
    /// the workspace folder by the sub workers, the same way nested client folders see it.
    #[tokio::test]
    async fn test_every_worker_receives_a_watched_file_event() {
        let dir = create_workspace();
        let root = uri(dir.path());
        let package_a = uri(&dir.path().join("packages/a"));
        let package_b = uri(&dir.path().join("packages/b"));

        let builds = Arc::new(Mutex::new(vec![]));
        let tracked = Arc::clone(&builds);

        let mut server = TestServer::new_initialized(
            move |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(
                        FakeToolBuilder::new(DiagnosticMode::Pull).with_build_tracking(tracked),
                    ),
                )
            },
            initialize_request_workspace_folders(InitializeRequestOptions {
                pull_mode: true,
                initialization_options: Some(json!([{
                    "workspaceUri": root.as_str(),
                    "options": { "workingDirectories": ["packages/a", "packages/b"] },
                }])),
                workspace_folders: Some(vec![workspace_folder(dir.path())]),
                ..Default::default()
            }),
        )
        .await;
        assert_eq!(workers(&mut server, 2).await.len(), 3);

        // a config inside `packages/a` and one at the workspace folder root
        for (index, path) in ["packages/a/tool.config", "tool.config"].into_iter().enumerate() {
            builds.lock().unwrap().clear();
            server
                .send_request(did_change_watched_files(uri(&dir.path().join(path)).as_str()))
                .await;
            // every worker rebuilt its tool, the client is asked to pull again
            acknowledge_diagnostic_refresh(&mut server).await;

            let mut built = builds.lock().unwrap().clone();
            built.sort_by(|left, right| left.as_str().cmp(right.as_str()));
            assert_eq!(
                built,
                vec![root.clone(), package_a.clone(), package_b.clone()],
                "event {index} did not reach every worker"
            );
        }

        server.shutdown(3).await;
    }

    /// A tool which rebuilds itself through the builder it is handed keeps the build context of
    /// its worker: it still skips the roots owned by the other workers.
    #[tokio::test]
    async fn test_a_tool_rebuilding_through_the_builder_keeps_its_context() {
        let dir = create_workspace();
        let contexts = Arc::new(Mutex::new(vec![]));
        let tracked = Arc::clone(&contexts);

        let mut server = TestServer::new_initialized(
            move |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(
                        FakeToolBuilder::new(DiagnosticMode::Pull)
                            .with_build_context_tracking(tracked),
                    ),
                )
            },
            initialize_request_workspace_folders(InitializeRequestOptions {
                pull_mode: true,
                initialization_options: Some(json!([{
                    "workspaceUri": uri(dir.path()).as_str(),
                    "options": { "workingDirectories": ["packages/a"] },
                }])),
                workspace_folders: Some(vec![workspace_folder(dir.path())]),
                ..Default::default()
            }),
        )
        .await;
        assert_eq!(workers(&mut server, 2).await.len(), 2);
        contexts.lock().unwrap().clear();

        // the working directories do not change, the tools restart for the unrelated option
        server
            .send_request(did_change_configuration(Some(json!([{
                "workspaceUri": uri(dir.path()).as_str(),
                "options": { "workingDirectories": ["packages/a"], "unrelated": true },
            }]))))
            .await;
        acknowledge_diagnostic_refresh(&mut server).await;

        // the workspace folder worker excludes `packages/a`, the sub worker excludes nothing
        assert_eq!(contexts.lock().unwrap().clone(), vec![1, 0]);

        server.shutdown(3).await;
    }

    /// Semantics rule 5: rewriting the option without changing the roots it resolves to rebuilds
    /// nothing, whatever the spelling of the entries.
    #[tokio::test]
    async fn test_rewriting_the_entries_rebuilds_nothing() {
        let dir = create_workspace();
        let builds = Arc::new(Mutex::new(vec![]));
        let tracked = Arc::clone(&builds);

        let mut server = TestServer::new_initialized(
            move |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(
                        FakeToolBuilder::default().with_build_tracking(tracked),
                    ),
                )
            },
            initialize(dir.path(), &json!(["packages/a"])),
        )
        .await;
        assert_eq!(workers(&mut server, 2).await.len(), 2);
        builds.lock().unwrap().clear();

        server
            .send_request(did_change_configuration(Some(json!([{
                "workspaceUri": uri(dir.path()).as_str(),
                "options": { "workingDirectories": ["./packages//a/."] },
            }]))))
            .await;

        assert_eq!(workers(&mut server, 3).await.len(), 2);
        assert!(builds.lock().unwrap().is_empty(), "{:?}", builds.lock().unwrap());

        server.shutdown(4).await;
    }

    /// Semantics rule 5: a configuration change which alters the options of the tool and the
    /// working directories at once registers every watcher id at most once.
    #[tokio::test]
    async fn test_one_configuration_change_registers_each_watcher_id_once() {
        let dir = create_workspace();
        let package_a = uri(&dir.path().join("packages/a"));
        let marker = package_a.clone();

        let mut server = TestServer::new_initialized(
            move |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(
                        FakeToolBuilder::default()
                            .with_discovered_extends(marker, "/outside/shared.json".to_string()),
                    ),
                )
            },
            initialize_request_workspace_folders(InitializeRequestOptions {
                dynamic_watchers: true,
                initialization_options: Some(json!([{
                    "workspaceUri": uri(dir.path()).as_str(),
                    "options": { "workingDirectories": [] },
                }])),
                workspace_folders: Some(vec![workspace_folder(dir.path())]),
                ..Default::default()
            }),
        )
        .await;
        acknowledge_registrations(&mut server).await;

        // the tool options change (it restarts) and `packages/a` becomes a working directory
        server
            .send_request(did_change_configuration(Some(json!([{
                "workspaceUri": uri(dir.path()).as_str(),
                "options": { "workingDirectories": ["packages/a"], "unrelated": true },
            }]))))
            .await;

        let unregister_request = server.recv_notification().await;
        assert_eq!(unregister_request.method(), "client/unregisterCapability");
        let params: serde_json::Value = unregister_request.params().unwrap().clone();
        let unregistered = ids(&params["unregisterations"]);
        server.send_ack(unregister_request.id().unwrap()).await;

        let register_request = server.recv_notification().await;
        assert_eq!(register_request.method(), "client/registerCapability");
        let params: serde_json::Value = register_request.params().unwrap().clone();
        let registered = ids(&params["registrations"]);
        server.send_ack(register_request.id().unwrap()).await;

        // the workspace folder worker lost the extended config, the new sub worker watches it
        assert_eq!(unregistered, vec![format!("watcher-{}", uri(dir.path()).as_str())]);
        assert_eq!(registered, vec![format!("watcher-{}", package_a.as_str())]);

        server.shutdown(2).await;
    }

    /// Semantics rule 7: a working directory inside a workspace folder of its own belongs to that
    /// folder, it is reported and gets no sub worker, while the others still resolve.
    #[tokio::test]
    async fn test_a_working_directory_inside_a_nested_workspace_folder_is_skipped() {
        let dir = create_workspace();
        fs::create_dir_all(dir.path().join("packages/a/sub")).unwrap();
        fs::create_dir_all(dir.path().join("packages/b/sub")).unwrap();
        let package_a = uri(&dir.path().join("packages/a"));

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request_workspace_folders(InitializeRequestOptions {
                initialization_options: Some(json!([{
                    "workspaceUri": uri(dir.path()).as_str(),
                    "options": { "workingDirectories": ["packages/a/sub", "packages/b/sub"] },
                }])),
                workspace_folders: Some(vec![
                    workspace_folder(dir.path()),
                    WorkspaceFolder { uri: package_a.clone(), name: "a".to_string() },
                ]),
                ..Default::default()
            }),
        )
        .await;

        let message = server.recv_notification().await;
        assert_eq!(message.method(), "window/showMessage");

        let workers = workers(&mut server, 2).await;
        // the workspace folder, the client folder for `packages/a`, and `packages/b/sub`
        assert_eq!(workers.len(), 3, "{workers:?}");
        assert!(
            workers
                .iter()
                .any(|(root, parent)| root == uri(&dir.path().join("packages/b/sub")).as_str()
                    && parent.is_some()),
            "{workers:?}"
        );
        assert!(
            !workers
                .iter()
                .any(|(root, _)| root == uri(&dir.path().join("packages/a/sub")).as_str()),
            "{workers:?}"
        );

        server.shutdown(3).await;
    }

    /// The root of a worker is resolved for every lookup, never cached: a worker created before
    /// its directory exists still routes the files of that directory once it does.
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    #[tokio::test]
    async fn test_a_worker_created_before_its_directory_exists_still_routes_after_creation() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("packages/a");

        let manager = create_workspace_manager();
        let worker = manager.create_worker(uri(&root), DiagnosticMode::None);
        manager.add_workers(vec![worker]).await;

        // the temporary directory is a symbolic link on macOS, and the casing differs on Windows:
        // only a directory which exists resolves to its canonical form
        fs::create_dir_all(&root).unwrap();
        let file = root.join("index.ts");
        fs::write(&file, "").unwrap();

        assert!(manager.get_worker_for_uri(&uri(&file)).await.is_some());
    }
}

#[cfg(test)]
mod test_suite {
    use std::{
        str::FromStr,
        sync::{Arc, Mutex},
    };

    use serde_json::{Value, json};
    use tower_lsp_server::{
        gen_lsp_types::{
            ApplyWorkspaceEditResult, InitializeResult, Message, MessageType,
            PublishDiagnosticsParams, ServerInfo, Uri, WorkspaceEdit, WorkspaceFolder,
        },
        jsonrpc::{Error, ErrorCode, Id, Response},
    };

    use crate::{
        ClientMessage, DiagnosticMode,
        backend::Backend,
        tests::{
            FAKE_COMMAND, FakeToolBuilder, InitializeRequestOptions, NESTED_WORKSPACE, TestServer,
            WORKSPACE, WORKSPACE_2, acknowledge_diagnostic_refresh, acknowledge_registrations,
            acknowledge_unregistrations, code_action, create_workspace_manager,
            create_workspace_manager_with_builder, diagnostic, did_change,
            did_change_configuration, did_change_watched_files, did_close, did_open, did_save,
            execute_command_request, initialize_request, initialize_request_workspace_folders,
            initialized_notification, response_to_configuration, shutdown_request,
            test_configuration_request, workspace_folders_changed,
        },
    };

    fn server_info() -> ServerInfo {
        ServerInfo { name: "oxc".to_owned(), version: Some("1.0.0".to_owned()) }
    }

    #[tokio::test]
    async fn test_client_message_deferred_until_initialized() {
        let builder = FakeToolBuilder {
            build_client_message: vec![ClientMessage {
                message: "Fake misconfiguration message".to_string(),
                r#type: MessageType::Warning,
            }],
            ..Default::default()
        };
        let mut server = TestServer::new(|client| {
            Backend::new(client, server_info(), create_workspace_manager_with_builder(builder))
        });

        // initialize: worker starts here (no workspace_configuration), message must NOT be sent yet
        server.send_request(initialize_request(InitializeRequestOptions::default())).await;
        let initialize_result = server.recv_response().await;
        assert!(initialize_result.is_ok());

        // initialized: message must be sent now
        server.send_request(initialized_notification()).await;
        let show_message = server.recv_notification().await;
        assert_eq!(show_message.method(), "window/showMessage");
        let params = show_message.params().unwrap();
        assert_eq!(params["message"], "Fake misconfiguration message");

        server.shutdown(2).await;
    }

    #[tokio::test]
    async fn test_client_message_cap_max_messages() {
        let builder = FakeToolBuilder {
            build_client_message: (1..=6)
                .map(|index| ClientMessage {
                    message: format!("Fake misconfiguration message {index}"),
                    r#type: MessageType::Warning,
                })
                .collect(),
            ..Default::default()
        };
        let mut server = TestServer::new(|client| {
            Backend::new(client, server_info(), create_workspace_manager_with_builder(builder))
        });

        // initialize: worker starts here (no workspace_configuration), messages must NOT be sent yet
        server.send_request(initialize_request(InitializeRequestOptions::default())).await;
        let initialize_result = server.recv_response().await;
        assert!(initialize_result.is_ok());

        // initialized: messages are capped to 5 with the last one being an overflow warning
        server.send_request(initialized_notification()).await;

        for index in 1..=4 {
            let show_message = server.recv_notification().await;
            assert_eq!(show_message.method(), "window/showMessage");
            let params = show_message.params().unwrap();
            assert_eq!(params["message"], format!("Fake misconfiguration message {index}"));
            assert_eq!(params["type"], json!(MessageType::Warning));
        }

        let show_message = server.recv_notification().await;
        assert_eq!(show_message.method(), "window/showMessage");
        let params = show_message.params().unwrap();
        assert_eq!(params["message"], "2 more messages not shown. See LSP logs for details.");
        assert_eq!(params["type"], json!(MessageType::Warning));

        server.shutdown(2).await;
    }

    #[tokio::test]
    async fn test_basic_start_and_shutdown_flow() {
        let mut server = TestServer::new(|client| {
            Backend::new(client, server_info(), create_workspace_manager())
        });
        // initialize request
        server.send_request(initialize_request(InitializeRequestOptions::default())).await;
        let initialize_result = server.recv_response().await;

        assert!(initialize_result.is_ok());
        let initialize_result: InitializeResult =
            serde_json::from_value(initialize_result.result().unwrap().clone()).unwrap();

        assert_eq!((initialize_result.server_info.unwrap().name), "oxc");
        assert!(initialize_result.capabilities.text_document_sync.is_some());

        // initialized notification
        server.send_request(initialized_notification()).await;

        // shutdown request
        server.send_request(shutdown_request(2)).await;
        let shutdown_result = server.recv_response().await;

        assert!(shutdown_result.is_ok());
        assert_eq!(shutdown_result.id(), &Id::Number(2));

        // exit notification
        // is handled by the lsp service itself
    }

    #[tokio::test]
    async fn test_initialize_with_options() {
        let init_options = InitializeRequestOptions {
            initialization_options: Some(json!([
                {
                    "workspaceUri": WORKSPACE,
                    "options": {
                        "run": true,
                        "configPath": "./custom.json",
                        "fmt.experimental": true
                    }
                }
            ])),
            ..Default::default()
        };

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(init_options),
        )
        .await;

        server.send_request(test_configuration_request(2)).await;
        let config_response = server.recv_response().await;

        assert!(config_response.is_ok());
        assert_eq!(config_response.id(), &Id::Number(2));
        assert_eq!(
            *config_response.result().unwrap(),
            json!([{
                "run": true,
                "configPath": "./custom.json",
                "fmt.experimental": true
            }])
        );

        // shutdown request
        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_initialize_with_deprecated_options() {
        let init_options = InitializeRequestOptions {
            initialization_options: Some(json!({
                "settings": {
                    "run": true,
                    "configPath": "./custom.json",
                    "fmt.experimental": true
                }
            })),
            root_uri: Some(Uri::from_str(WORKSPACE).unwrap()),
            ..Default::default()
        };

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request_workspace_folders(init_options),
        )
        .await;

        server.send_request(test_configuration_request(2)).await;
        let config_response = server.recv_response().await;

        assert!(config_response.is_ok());
        assert_eq!(config_response.id(), &Id::Number(2));
        assert_eq!(
            *config_response.result().unwrap(),
            json!([{
                "run": true,
                "configPath": "./custom.json",
                "fmt.experimental": true
            }])
        );
        // shutdown request
        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_initialize_non_file_workspace_uri() {
        let init_options = InitializeRequestOptions {
            workspace_folders: Some(vec![WorkspaceFolder {
                uri: Uri::from_str("file://").unwrap(),
                name: "workspace".to_string(),
            }]),
            ..Default::default()
        };

        let mut server = TestServer::new(|client| {
            Backend::new(client, server_info(), create_workspace_manager())
        });
        let initialize = initialize_request_workspace_folders(init_options);

        let initialize_id = initialize.id().cloned();
        // Send initialize request
        server.send_request(initialize).await;
        let initialize_response = server.recv_response().await;

        assert_eq!(Some(initialize_response.id()), initialize_id.as_ref());
        assert!(initialize_response.is_error());
        assert_eq!(
            *initialize_response.error().unwrap(),
            Error {
                code: ErrorCode::InvalidParams,
                message: "workspace URI is not a valid file path: file://".into(),
                data: None,
            }
        );
    }

    #[tokio::test]
    async fn test_workspace_configuration_on_initialized() {
        let init_options =
            InitializeRequestOptions { workspace_configuration: true, ..Default::default() };

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(init_options),
        )
        .await;

        // workspace/configuration request
        let workspace_config_request = server.recv_notification().await;
        assert_eq!(workspace_config_request.method(), "workspace/configuration");
        assert_eq!(workspace_config_request.id(), Some(&Id::Number(0)));
        assert_eq!(
            workspace_config_request.params(),
            Some(&json!({
                "items": [
                    {
                        "scopeUri":  WORKSPACE,
                        "section": "oxc_language_server"
                    }
                ]
            }))
        );
        server
            .send_response(Response::from_ok(
                workspace_config_request.id().unwrap().clone(),
                json!([null]),
            ))
            .await;

        // shutdown request
        server.send_request(shutdown_request(2)).await;
        let shutdown_result = server.recv_response().await;

        assert!(shutdown_result.is_ok());
        assert_eq!(shutdown_result.id(), &Id::Number(2));
    }

    #[tokio::test]
    async fn test_workspace_configuration_runs_push_diagnostics_for_open_files() {
        let init_options = InitializeRequestOptions {
            workspace_configuration: true,
            workspace_folders: Some(vec![
                WorkspaceFolder { uri: WORKSPACE.parse().unwrap(), name: "workspace".to_string() },
                WorkspaceFolder {
                    uri: NESTED_WORKSPACE.parse().unwrap(),
                    name: "nested".to_string(),
                },
            ]),
            ..Default::default()
        };
        let mut server = TestServer::new(|client| {
            Backend::new(
                client,
                server_info(),
                create_workspace_manager_with_builder(FakeToolBuilder::new(DiagnosticMode::Push)),
            )
        });

        server.send_request(initialize_request_workspace_folders(init_options)).await;
        assert!(server.recv_response().await.is_ok());

        let uri = format!("{NESTED_WORKSPACE}/diagnostics.config");
        let content = "initialized content";
        server.send_request(did_open(&uri, content)).await;

        // Send initialized notification
        server.send_request(initialized_notification()).await;

        // workspace configuration request expected
        response_to_configuration(&mut server, vec![json!(null), json!(null)]).await;

        let diagnostic_response = server.recv_notification().await;
        assert_eq!(diagnostic_response.method(), "textDocument/publishDiagnostics");
        let params: PublishDiagnosticsParams =
            serde_json::from_value(diagnostic_response.params().unwrap().clone()).unwrap();
        assert_eq!(params.uri, Uri::from_str(&uri).unwrap());
        assert_eq!(params.diagnostics.len(), 1);
        assert_eq!(
            params.diagnostics[0].message,
            format!("Fake diagnostic for content: {content}").into()
        );

        server.shutdown_with_diagnostic_clear(2, vec![uri.parse().unwrap()]).await;
    }

    #[tokio::test]
    async fn test_dynamic_watched_files_registration() {
        let init_options =
            InitializeRequestOptions { dynamic_watchers: true, ..Default::default() };

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(init_options),
        )
        .await;

        // client/registerCapability request
        let workspace_config_request = server.recv_notification().await;
        assert_eq!(workspace_config_request.method(), "client/registerCapability");
        assert_eq!(workspace_config_request.id(), Some(&Id::Number(0)));
        assert_eq!(
            workspace_config_request.params(),
            Some(&json!({
                "registrations": [
                    {
                        "id": format!("watcher-{WORKSPACE}"),
                        "method": "workspace/didChangeWatchedFiles",
                        "registerOptions": {
                            "watchers": [
                                {
                                    "globPattern": {
                                        "baseUri": WORKSPACE,
                                        "pattern": "**/fake.config",
                                    },
                                    "kind": 7
                                }
                            ]
                        },
                    }
                ]
            }))
        );

        // Acknowledge the registration
        server.send_ack(&Id::Number(0)).await;

        // shutdown request
        server.send_request(shutdown_request(2)).await;

        // shutdown response
        let shutdown_result = server.recv_response().await;

        assert!(shutdown_result.is_ok());
        assert_eq!(shutdown_result.id(), &Id::Number(2));
    }

    #[tokio::test]
    async fn test_execute_workspace_command_with_apply_edit() {
        let init_options = InitializeRequestOptions { workspace_edit: true, ..Default::default() };

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(init_options),
        )
        .await;

        // execute command request
        let execute_command_request =
            execute_command_request(FAKE_COMMAND, &[json!({"some_option": true})], 3);
        server.send_request(execute_command_request).await;

        // workspace apply edit request
        let apply_edit_request = server.recv_notification().await;
        assert_eq!(apply_edit_request.method(), "workspace/applyEdit");
        assert_eq!(
            apply_edit_request.params(),
            Some(&json!({
                "edit": WorkspaceEdit::default(),
            }))
        );

        // Acknowledge the apply edit
        server
            .send_response(Response::from_ok(
                apply_edit_request.id().unwrap().clone(),
                json!(ApplyWorkspaceEditResult {
                    applied: true,
                    failure_reason: None,
                    failed_change: None
                }),
            ))
            .await;

        // execute command response
        let execute_command_response = server.recv_response().await;
        assert!(execute_command_response.is_ok());
        assert!(execute_command_response.result().is_some());
        assert_eq!(execute_command_response.id(), &Id::Number(3));
        assert_eq!(execute_command_response.result().unwrap(), &json!(null));

        // shutdown request
        server.shutdown(4).await;
    }

    #[tokio::test]
    async fn test_initialize_with_options_and_multiple_workspace_folders() {
        let init_options = InitializeRequestOptions {
            initialization_options: Some(json!([
            // correctly matches options to workspace folders regardless of order
            {
                "workspaceUri": WORKSPACE_2,
                "options": {
                    "run": false,
                    "configPath": "./another_custom.json",
                    "fmt.experimental": false
                }
            },
            {
                "workspaceUri": WORKSPACE,
                "options": {
                    "run": true,
                    "configPath": "./custom.json",
                    "fmt.experimental": true
                }
            },
            ])),
            workspace_folders: Some(vec![
                WorkspaceFolder {
                    uri: Uri::from_str(WORKSPACE).unwrap(),
                    name: "workspace".to_string(),
                },
                WorkspaceFolder {
                    uri: Uri::from_str(WORKSPACE_2).unwrap(),
                    name: "workspace_2".to_string(),
                },
            ]),
            ..Default::default()
        };

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request_workspace_folders(init_options),
        )
        .await;

        server.send_request(test_configuration_request(2)).await;
        let config_response = server.recv_response().await;

        assert!(config_response.is_ok());
        assert_eq!(config_response.id(), &Id::Number(2));
        assert_eq!(
            *config_response.result().unwrap(),
            json!([
                {
                    "run": true,
                    "configPath": "./custom.json",
                    "fmt.experimental": true
                },
                {
                    "run": false,
                    "configPath": "./another_custom.json",
                    "fmt.experimental": false
                }
            ])
        );

        // shutdown request
        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_execute_workspace_command_with_no_edit() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(InitializeRequestOptions::default()),
        )
        .await;

        // execute command request with no arguments
        let execute_command_request = execute_command_request(FAKE_COMMAND, &[], 3);
        server.send_request(execute_command_request).await;

        // Should get a direct response with null result (no workspace edit)
        let execute_command_response = server.recv_response().await;
        assert!(execute_command_response.is_ok());
        assert_eq!(execute_command_response.id(), &Id::Number(3));
        assert_eq!(execute_command_response.result().unwrap(), &json!(null));

        // shutdown request
        server.shutdown(4).await;
    }

    #[tokio::test]
    async fn test_execute_workspace_command_with_invalid_command() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(InitializeRequestOptions::default()),
        )
        .await;

        // execute command request with an invalid command
        let execute_command_request = execute_command_request("invalid.command", &[], 3);
        server.send_request(execute_command_request).await;

        // Should return an error
        let execute_command_response = server.recv_response().await;
        assert!(execute_command_response.is_error());
        assert_eq!(execute_command_response.id(), &Id::Number(3));
        assert_eq!(execute_command_response.error().unwrap().code, ErrorCode::InvalidParams);

        server.shutdown(4).await;
    }

    #[tokio::test]
    async fn test_workspace_added() {
        // workspace/didChangeWorkspaceFolders notification
        let folders_changed_notification = workspace_folders_changed(
            vec![WorkspaceFolder {
                uri: Uri::from_str("file:///path/to/new_folder").unwrap(),
                name: "new_folder".to_string(),
            }],
            vec![],
        );

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(InitializeRequestOptions::default()),
        )
        .await;
        server.send_request(folders_changed_notification).await;

        // No direct response expected for notifications, client does not support workspace configuration or watchers
        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_workspace_added_shows_client_message() {
        // workspace/didChangeWorkspaceFolders notification
        let folders_changed_notification = workspace_folders_changed(
            vec![WorkspaceFolder {
                uri: Uri::from_str("file:///path/to/new_folder").unwrap(),
                name: "new_folder".to_string(),
            }],
            vec![],
        );

        let builder = FakeToolBuilder {
            build_client_message: vec![ClientMessage {
                message: "Fake misconfiguration message".to_string(),
                r#type: MessageType::Warning,
            }],
            ..Default::default()
        };

        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(client, server_info(), create_workspace_manager_with_builder(builder))
            },
            initialize_request(InitializeRequestOptions::default()),
        )
        .await;

        // Initial worker startup message is sent on initialized; consume it first.
        let show_message = server.recv_notification().await;
        assert_eq!(show_message.method(), "window/showMessage");

        server.send_request(folders_changed_notification).await;

        // Adding a workspace starts a new worker and should surface its client message.
        let show_message = server.recv_notification().await;
        assert_eq!(show_message.method(), "window/showMessage");
        let params = show_message.params().unwrap();
        assert_eq!(params["message"], "Fake misconfiguration message");
        assert_eq!(params["type"], json!(MessageType::Warning));

        server.shutdown(4).await;
    }

    #[tokio::test]
    async fn test_workspace_added_watchers() {
        // workspace/didChangeWorkspaceFolders notification
        let folders_changed_notification = workspace_folders_changed(
            vec![WorkspaceFolder {
                uri: Uri::from_str("file:///path/to/new_folder").unwrap(),
                name: "new_folder".to_string(),
            }],
            vec![],
        );

        let init_options =
            InitializeRequestOptions { dynamic_watchers: true, ..Default::default() };

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(init_options),
        )
        .await;
        acknowledge_registrations(&mut server).await;
        server.send_request(folders_changed_notification).await;

        // new watcher registration expected
        acknowledge_registrations(&mut server).await;
        server.shutdown(4).await;
    }

    #[tokio::test]
    async fn test_workspace_added_configuration() {
        // workspace/didChangeWorkspaceFolders notification
        let folders_changed_notification = workspace_folders_changed(
            vec![WorkspaceFolder {
                uri: Uri::from_str("file:///path/to/new_folder").unwrap(),
                name: "new_folder".to_string(),
            }],
            vec![],
        );

        let init_options =
            InitializeRequestOptions { workspace_configuration: true, ..Default::default() };

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(init_options),
        )
        .await;
        // workspace configuration request expected, one for initial workspace
        response_to_configuration(&mut server, vec![json!(null)]).await;

        server.send_request(folders_changed_notification).await;

        // workspace configuration request expected, one for new folder
        response_to_configuration(&mut server, vec![json!(null)]).await;

        server.shutdown(4).await;
    }

    #[tokio::test]
    async fn test_workspace_removed() {
        // workspace/didChangeWorkspaceFolders notification
        let folders_changed_notification = workspace_folders_changed(
            vec![],
            vec![WorkspaceFolder {
                uri: Uri::from_str(WORKSPACE).unwrap(),
                name: "workspace".to_string(),
            }],
        );

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(InitializeRequestOptions::default()),
        )
        .await;
        server.send_request(folders_changed_notification).await;

        // No direct response expected for notifications, client does not support watchers
        server.shutdown(2).await;
    }

    #[tokio::test]
    async fn test_workspace_removed_watchers() {
        // workspace/didChangeWorkspaceFolders notification
        let folders_changed_notification = workspace_folders_changed(
            vec![],
            vec![WorkspaceFolder {
                uri: Uri::from_str(WORKSPACE).unwrap(),
                name: "workspace".to_string(),
            }],
        );

        let init_options =
            InitializeRequestOptions { dynamic_watchers: true, ..Default::default() };

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(init_options),
        )
        .await;
        acknowledge_registrations(&mut server).await;
        server.send_request(folders_changed_notification).await;

        // watcher unregistration expected
        acknowledge_unregistrations(&mut server).await;

        server.shutdown(2).await;
    }

    #[tokio::test]
    async fn test_watched_file_changed_unknown() {
        let init_options =
            InitializeRequestOptions { dynamic_watchers: true, ..Default::default() };

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(init_options),
        )
        .await;
        acknowledge_registrations(&mut server).await;

        // Simulate a watched file change notification
        let file_change_notification =
            did_change_watched_files(format!("{WORKSPACE}/unknown.file").as_str());
        server.send_request(file_change_notification).await;

        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_watched_file_changed_new_watchers() {
        let init_options =
            InitializeRequestOptions { dynamic_watchers: true, ..Default::default() };
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(init_options),
        )
        .await;
        acknowledge_registrations(&mut server).await;

        // Simulate a watched file change notification for "watcher.config"
        let file_change_notification =
            did_change_watched_files(format!("{WORKSPACE}/watcher.config").as_str());
        server.send_request(file_change_notification).await;

        // Old watcher unregistration expected
        acknowledge_unregistrations(&mut server).await;
        // New watcher registration expected
        acknowledge_registrations(&mut server).await;

        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_watched_file_changed_triggers_both_workspaces() {
        let init_options = InitializeRequestOptions {
            dynamic_watchers: true,
            workspace_folders: Some(vec![
                WorkspaceFolder {
                    uri: Uri::from_str(WORKSPACE).unwrap(),
                    name: "workspace".to_string(),
                },
                WorkspaceFolder {
                    uri: Uri::from_str(WORKSPACE_2).unwrap(),
                    name: "workspace_2".to_string(),
                },
            ]),
            ..Default::default()
        };

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request_workspace_folders(init_options),
        )
        .await;

        acknowledge_registrations(&mut server).await;

        let file_change_notification =
            did_change_watched_files(format!("{WORKSPACE}/watcher.config").as_str());
        server.send_request(file_change_notification).await;

        // Old watcher unregistration expected
        acknowledge_unregistrations(&mut server).await;

        let register_request = server.recv_notification().await;
        assert_eq!(register_request.method(), "client/registerCapability");
        let register_params: Value = register_request.params().unwrap().clone();
        let registrations = register_params["registrations"].as_array().unwrap();
        assert_eq!(registrations.len(), 2);
        assert!(
            registrations
                .iter()
                .any(|registration| { registration["id"] == format!("watcher-{WORKSPACE}") })
        );
        assert!(
            registrations
                .iter()
                .any(|registration| { registration["id"] == format!("watcher-{WORKSPACE_2}") })
        );
        server.send_ack(register_request.id().unwrap()).await;

        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_watched_file_changed_revalidate_diagnostics() {
        let init_options =
            InitializeRequestOptions { dynamic_watchers: true, ..Default::default() };
        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(FakeToolBuilder::new(
                        DiagnosticMode::Push,
                    )),
                )
            },
            initialize_request(init_options),
        )
        .await;
        acknowledge_registrations(&mut server).await;

        let uri = format!("{WORKSPACE}/diagnostics.config");
        let content = "some text";
        server.send_request(did_open(&uri, content)).await;
        let diagnostic_response = server.recv_notification().await;
        assert_eq!(diagnostic_response.method(), "textDocument/publishDiagnostics");

        // Simulate a watched file change notification for "tool.config"
        let file_change_notification =
            did_change_watched_files(format!("{WORKSPACE}/tool.config").as_str());
        server.send_request(file_change_notification).await;

        // expecting diagnostics to be re-validated
        let diagnostic_response = server.recv_notification().await;
        assert_eq!(diagnostic_response.method(), "textDocument/publishDiagnostics");
        let params: PublishDiagnosticsParams =
            serde_json::from_value(diagnostic_response.params().unwrap().clone()).unwrap();
        assert_eq!(params.uri.to_string(), uri);
        assert_eq!(params.diagnostics.len(), 1);
        assert_eq!(
            params.diagnostics[0].message,
            Message::String(format!("Fake diagnostic for content: {content}"))
        );

        server.shutdown_with_diagnostic_clear(3, vec![Uri::from_str(&uri).unwrap()]).await;
    }

    #[tokio::test]
    async fn test_watched_file_changed_revalidate_diagnostics_pull_mode() {
        let init_options = InitializeRequestOptions {
            dynamic_watchers: true,
            pull_mode: true,
            ..Default::default()
        };

        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(FakeToolBuilder::new(
                        DiagnosticMode::Pull,
                    )),
                )
            },
            initialize_request(init_options),
        )
        .await;
        acknowledge_registrations(&mut server).await;

        let uri = format!("{WORKSPACE}/diagnostics.config");
        let content = "some text";
        server.send_request(did_open(&uri, content)).await;

        // Simulate a watched file change notification for "tool.config"
        let file_change_notification =
            did_change_watched_files(format!("{WORKSPACE}/tool.config").as_str());
        server.send_request(file_change_notification).await;

        // Acknowledge the refresh request
        acknowledge_diagnostic_refresh(&mut server).await;

        server.shutdown(3).await;
    }

    /// A client is entitled to do work before replying to
    /// `workspace/diagnostic/refresh` — typically re-pulling diagnostics from
    /// this same server. The server must therefore keep servicing requests
    /// while refresh replies are outstanding. Before the fix, each in-flight
    /// refresh pinned one of the transport's 4 concurrency slots inside the
    /// notification handler that sent it, so 4 unanswered refreshes wedged the
    /// server permanently (issue #24955): this test then timed out waiting for
    /// the shutdown response.
    #[tokio::test]
    async fn test_outstanding_diagnostic_refreshes_do_not_wedge_the_server() {
        let init_options = InitializeRequestOptions {
            dynamic_watchers: true,
            pull_mode: true,
            ..Default::default()
        };

        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(FakeToolBuilder::new(
                        DiagnosticMode::Pull,
                    )),
                )
            },
            initialize_request(init_options),
        )
        .await;
        acknowledge_registrations(&mut server).await;

        // Fire enough watched-file events to trigger more refresh requests
        // than the transport has concurrency slots (tower-lsp-server's
        // default is 4), and read each refresh WITHOUT replying to it.
        for _ in 0..4 {
            let file_change_notification =
                did_change_watched_files(format!("{WORKSPACE}/tool.config").as_str());
            server.send_request(file_change_notification).await;

            let refresh_request = server.recv_notification().await;
            assert_eq!(refresh_request.method(), "workspace/diagnostic/refresh");
            // Deliberately no ack: the client is still busy re-pulling.
        }

        // With 4 refresh replies outstanding, the server must still answer
        // new requests — a wedged server never responds and the timeout
        // fails the test instead of hanging it.
        server.send_request(shutdown_request(3)).await;
        let shutdown_result =
            tokio::time::timeout(std::time::Duration::from_secs(10), server.recv_response())
                .await
                .expect(
                    "server did not answer while diagnostic-refresh replies were outstanding \
                     (deadlocked, issue #24955)",
                );
        assert!(shutdown_result.is_ok());
        assert_eq!(shutdown_result.id(), &Id::Number(3));
    }

    #[tokio::test]
    async fn test_did_change_configuration_no_changes() {
        let init_options =
            InitializeRequestOptions { dynamic_watchers: true, ..Default::default() };

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(init_options),
        )
        .await;
        acknowledge_registrations(&mut server).await;

        // Simulate a configuration change that does not affect the tool
        let config_change_notification = did_change_configuration(None);
        server.send_request(config_change_notification).await;

        // When `null` is sent and the client does not support workspace configuration requests,
        // no configuration changes occur, so no diagnostics or registrations are expected.
        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_did_change_configuration_config_passed_new_watchers() {
        let init_options =
            InitializeRequestOptions { dynamic_watchers: true, ..Default::default() };

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(init_options),
        )
        .await;
        acknowledge_registrations(&mut server).await;

        // Simulate a configuration change that affects watcher patterns
        let config_change_notification = did_change_configuration(Some(json!([
            {
                "workspaceUri": WORKSPACE,
                "options": 2
            }
        ])));
        server.send_request(config_change_notification).await;

        // Old watcher unregistration expected
        acknowledge_unregistrations(&mut server).await;
        // New watcher registration expected
        acknowledge_registrations(&mut server).await;

        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_did_change_configuration_config_requested_new_watchers() {
        let init_options = InitializeRequestOptions {
            dynamic_watchers: true,
            workspace_configuration: true,
            ..Default::default()
        };

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(init_options),
        )
        .await;
        response_to_configuration(&mut server, vec![json!(null)]).await;

        acknowledge_registrations(&mut server).await;

        // Simulate a configuration change that affects watcher patterns
        let config_change_notification = did_change_configuration(None);
        server.send_request(config_change_notification).await;

        // requesting workspace configuration
        response_to_configuration(&mut server, vec![json!(2)]).await;
        // Old watcher unregistration expected
        acknowledge_unregistrations(&mut server).await;
        // New watcher registration expected
        acknowledge_registrations(&mut server).await;

        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_did_change_configuration_config_skips_revalidate_diagnostics() {
        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(FakeToolBuilder::new(
                        DiagnosticMode::None,
                    )),
                )
            },
            initialize_request(InitializeRequestOptions::default()),
        )
        .await;

        // Simulate a configuration change that affects diagnostics
        let config_change_notification = did_change_configuration(Some(json!([
            {
                "workspaceUri": WORKSPACE,
                "options": 3
            }
        ])));
        server.send_request(config_change_notification).await;

        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_did_change_configuration_config_revalidate_diagnostics() {
        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(FakeToolBuilder::new(
                        DiagnosticMode::Push,
                    )),
                )
            },
            initialize_request(InitializeRequestOptions::default()),
        )
        .await;
        let uri = format!("{WORKSPACE}/diagnostics.config");
        let content = "some text";
        server.send_request(did_open(&uri, content)).await;
        let diagnostic_response = server.recv_notification().await;
        assert_eq!(diagnostic_response.method(), "textDocument/publishDiagnostics");

        // Simulate a configuration change that affects diagnostics
        let config_change_notification = did_change_configuration(Some(json!([
            {
                "workspaceUri": WORKSPACE,
                "options": 3
            }
        ])));
        server.send_request(config_change_notification).await;

        // expecting diagnostics to be re-validated
        let diagnostic_response = server.recv_notification().await;
        assert_eq!(diagnostic_response.method(), "textDocument/publishDiagnostics");
        let params: PublishDiagnosticsParams =
            serde_json::from_value(diagnostic_response.params().unwrap().clone()).unwrap();
        assert_eq!(params.uri.to_string(), uri);
        assert_eq!(params.diagnostics.len(), 1);
        assert_eq!(
            params.diagnostics[0].message,
            Message::String(format!("Fake diagnostic for content: {content}"))
        );

        server.shutdown_with_diagnostic_clear(3, vec![Uri::from_str(&uri).unwrap()]).await;
    }

    #[tokio::test]
    async fn test_did_change_configuration_config_revalidate_diagnostics_pull_mode() {
        let init_options = InitializeRequestOptions { pull_mode: true, ..Default::default() };

        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(FakeToolBuilder::new(
                        DiagnosticMode::Pull,
                    )),
                )
            },
            initialize_request(init_options),
        )
        .await;
        let uri = format!("{WORKSPACE}/diagnostics.config");
        let content = "some text";
        server.send_request(did_open(&uri, content)).await;

        // Simulate a configuration change that affects diagnostics
        let config_change_notification = did_change_configuration(Some(json!([
            {
                "workspaceUri": WORKSPACE,
                "options": 3
            }
        ])));
        server.send_request(config_change_notification).await;

        // Acknowledge the refresh request
        acknowledge_diagnostic_refresh(&mut server).await;

        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_file_notifications() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(InitializeRequestOptions::default()),
        )
        .await;

        let file = format!("{WORKSPACE}/file.txt");

        server.send_request(did_open(&file, "some text")).await;
        server.send_request(did_change(&file, "changed text")).await;
        server.send_request(did_save(&file, "changed text")).await; // should be the same as last content
        server.send_request(did_close(&file)).await;
        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_did_change_clears_uri_cache() {
        let init_options = InitializeRequestOptions { pull_mode: true, ..Default::default() };
        let cache_uris = Arc::new(Mutex::new(Vec::new()));

        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(
                        FakeToolBuilder::new(DiagnosticMode::Pull)
                            .with_cache_tracking(Arc::clone(&cache_uris)),
                    ),
                )
            },
            initialize_request(init_options),
        )
        .await;

        let file = format!("{WORKSPACE}/file.txt");
        server.send_request(did_open(&file, "some text")).await;

        server.send_request(diagnostic(3, &file)).await;
        let diagnostic_response = server.recv_response().await;
        assert!(diagnostic_response.is_ok());
        assert_eq!(diagnostic_response.id(), &Id::Number(3));

        {
            let removed_cache_uris = cache_uris.lock().unwrap();
            assert_eq!(removed_cache_uris.len(), 1);
            assert_eq!(removed_cache_uris[0].to_string(), file);
        }

        server.send_request(did_change(&file, "changed text")).await;

        // didChange is a notification; use a follow-up request to ensure it was processed.
        server.send_request(test_configuration_request(4)).await;
        let response = server.recv_response().await;
        assert!(response.is_ok());
        assert_eq!(response.id(), &Id::Number(4));

        {
            let removed_cache_uris = cache_uris.lock().unwrap();
            assert_eq!(removed_cache_uris.len(), 0);
        }

        server.shutdown(5).await;
    }

    #[tokio::test]
    async fn test_code_action_no_actions() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(InitializeRequestOptions::default()),
        )
        .await;

        let file = format!("{WORKSPACE}/file.txt");

        server.send_request(did_open(&file, "some text")).await;

        // No code actions expected
        server.send_request(code_action(3, &file)).await;
        let response = server.recv_response().await;
        assert!(response.is_ok());
        assert_eq!(response.id(), &Id::Number(3));
        assert!(response.result().is_some_and(|result| *result == Value::Null));

        server.shutdown(4).await;
    }

    #[tokio::test]
    async fn test_code_actions_with_actions() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), create_workspace_manager()),
            initialize_request(InitializeRequestOptions::default()),
        )
        .await;

        let file = format!("{WORKSPACE}/code_action.config");

        server.send_request(did_open(&file, "some text")).await;

        // Code actions expected
        server.send_request(code_action(3, &file)).await;
        let response = server.recv_response().await;
        assert!(response.is_ok());
        assert_eq!(response.id(), &Id::Number(3));
        let actions: Vec<serde_json::Value> =
            serde_json::from_value(response.result().unwrap().clone()).unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0]["title"], "Code Action title");

        server.shutdown(4).await;
    }

    #[tokio::test]
    async fn test_diagnostic_on_open() {
        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(FakeToolBuilder::new(
                        DiagnosticMode::Push,
                    )),
                )
            },
            initialize_request(InitializeRequestOptions::default()),
        )
        .await;

        let file = format!("{WORKSPACE}/diagnostics.config");
        let content = "some text";
        server.send_request(did_open(&file, content)).await;

        let diagnostic_response = server.recv_notification().await;
        assert_eq!(diagnostic_response.method(), "textDocument/publishDiagnostics");
        let params: PublishDiagnosticsParams =
            serde_json::from_value(diagnostic_response.params().unwrap().clone()).unwrap();
        assert_eq!(params.uri.to_string(), file);
        assert_eq!(params.diagnostics.len(), 1);
        assert_eq!(
            params.diagnostics[0].message,
            Message::String(format!("Fake diagnostic for content: {content}"))
        );

        server.shutdown_with_diagnostic_clear(4, vec![Uri::from_str(&file).unwrap()]).await;
    }

    /// This test verifies that the tool is not requested to provide diagnostics,
    /// when ALL tools doe not change diagnostics mode.
    #[tokio::test]
    async fn test_diagnostic_on_open_no_backend_capability() {
        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(FakeToolBuilder::new(
                        DiagnosticMode::None,
                    )),
                )
            },
            initialize_request(InitializeRequestOptions::default()),
        )
        .await;

        let file = format!("{WORKSPACE}/diagnostics.config");
        let content = "some text";
        server.send_request(did_open(&file, content)).await;

        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_diagnostic_on_change() {
        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(FakeToolBuilder::new(
                        DiagnosticMode::Push,
                    )),
                )
            },
            initialize_request(InitializeRequestOptions::default()),
        )
        .await;

        let file = format!("{WORKSPACE}/diagnostics.config");
        let content = "new text";
        server.send_request(did_open(&file, "old text")).await;
        let diagnostic_response = server.recv_notification().await;
        assert_eq!(diagnostic_response.method(), "textDocument/publishDiagnostics");

        server.send_request(did_change(&file, content)).await;

        let diagnostic_response = server.recv_notification().await;
        assert_eq!(diagnostic_response.method(), "textDocument/publishDiagnostics");
        let params: PublishDiagnosticsParams =
            serde_json::from_value(diagnostic_response.params().unwrap().clone()).unwrap();
        assert_eq!(params.uri.to_string(), file);
        assert_eq!(params.diagnostics.len(), 1);
        assert_eq!(
            params.diagnostics[0].message,
            Message::String(format!("Fake diagnostic for content: {content}"))
        );

        server.shutdown_with_diagnostic_clear(4, vec![Uri::from_str(&file).unwrap()]).await;
    }

    #[tokio::test]
    async fn test_diagnostic_on_save() {
        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(FakeToolBuilder::new(
                        DiagnosticMode::Push,
                    )),
                )
            },
            initialize_request(InitializeRequestOptions::default()),
        )
        .await;

        let file = format!("{WORKSPACE}/diagnostics.config");
        let content = "new text";
        server.send_request(did_open(&file, "old text")).await;
        let diagnostic_response = server.recv_notification().await;
        assert_eq!(diagnostic_response.method(), "textDocument/publishDiagnostics");

        server.send_request(did_change(&file, content)).await;

        let diagnostic_response = server.recv_notification().await;
        assert_eq!(diagnostic_response.method(), "textDocument/publishDiagnostics");

        server.send_request(did_save(&file, content)).await;

        let diagnostic_response = server.recv_notification().await;
        assert_eq!(diagnostic_response.method(), "textDocument/publishDiagnostics");
        let params: PublishDiagnosticsParams =
            serde_json::from_value(diagnostic_response.params().unwrap().clone()).unwrap();
        assert_eq!(params.uri.to_string(), file);
        assert_eq!(params.diagnostics.len(), 1);
        assert_eq!(
            params.diagnostics[0].message,
            Message::String(format!("Fake diagnostic for content: {content}"))
        );

        server.shutdown_with_diagnostic_clear(4, vec![Uri::from_str(&file).unwrap()]).await;
    }

    #[tokio::test]
    async fn test_no_diagnostics_on_pull_mode_on_save() {
        let init_options = InitializeRequestOptions { pull_mode: true, ..Default::default() };

        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(FakeToolBuilder::new(
                        DiagnosticMode::Pull,
                    )),
                )
            },
            initialize_request(init_options),
        )
        .await;

        let file = format!("{WORKSPACE}/diagnostics.config");
        let content = "new text";
        server.send_request(did_open(&file, "old text")).await;
        server.send_request(did_change(&file, content)).await;
        server.send_request(did_save(&file, content)).await;
        server.shutdown(4).await;
    }

    #[tokio::test]
    async fn test_diagnostics_pull_mode() {
        let init_options = InitializeRequestOptions { pull_mode: true, ..Default::default() };

        let mut server = TestServer::new_initialized(
            |client| {
                Backend::new(
                    client,
                    server_info(),
                    create_workspace_manager_with_builder(FakeToolBuilder::new(
                        DiagnosticMode::Pull,
                    )),
                )
            },
            initialize_request(init_options),
        )
        .await;

        let file = format!("{WORKSPACE}/diagnostics.config");
        let content = "pull mode text";
        server.send_request(did_open(&file, content)).await;

        server.send_request(diagnostic(3, &file)).await;

        let diagnostic_response = server.recv_response().await;
        assert!(diagnostic_response.is_ok());
        assert_eq!(diagnostic_response.id(), &Id::Number(3));

        let report: serde_json::Value =
            serde_json::from_value(diagnostic_response.result().unwrap().clone()).unwrap();

        assert_eq!(report["kind"], "full");
        assert_eq!(report["items"].as_array().unwrap().len(), 1);
        assert_eq!(
            report["items"][0]["message"],
            format!("Fake diagnostic for content: {content}")
        );

        server.shutdown(4).await;
    }

    // ── Single-file mode (no workspace folders / root URI on initialize) ──────
    #[cfg(not(target_os = "windows"))] // TODO: fix Windows paths in single-file mode tests, first guess it the uri->path->uri conversation with non-windows paths
    mod single_file_mode {
        use tower_lsp_server::gen_lsp_types::Uri;

        use super::*;
        /// Helper: build an initialize request that puts the server into single-file mode.
        fn single_file_mode_initialize() -> crate::tests::InitializeRequestOptions {
            // workspace_folders = None, root_uri = None → single file mode
            InitializeRequestOptions::default()
        }

        /// When all workspace folders are removed and the server had explicit workspace folders,
        /// it should enter single-file mode so subsequent file opens create workers dynamically.
        #[tokio::test]
        async fn test_entering_single_file_mode_when_all_workspaces_removed() {
            let mut server = TestServer::new_initialized(
                |client| Backend::new(client, server_info(), create_workspace_manager()),
                initialize_request(InitializeRequestOptions::default()),
            )
            .await;

            // Confirm there is initially one workspace worker.
            server.send_request(test_configuration_request(2)).await;
            let response = server.recv_response().await;
            assert_eq!(response.result().unwrap().as_array().unwrap().len(), 1);

            // Remove the only workspace folder.
            server
                .send_request(workspace_folders_changed(
                    vec![],
                    vec![WorkspaceFolder {
                        uri: Uri::from_str(WORKSPACE).unwrap(),
                        name: "workspace".to_string(),
                    }],
                ))
                .await;

            // No workers remain; the server should be in single-file mode.
            server.send_request(test_configuration_request(3)).await;
            let response = server.recv_response().await;
            assert_eq!(*response.result().unwrap(), json!([]));

            // Opening a file should now dynamically create a worker (single-file mode).
            let file = "file:///path/to/some/file.js";
            server.send_request(did_open(file, "content")).await;

            server.send_request(test_configuration_request(4)).await;
            let response = server.recv_response().await;
            assert_eq!(response.result().unwrap().as_array().unwrap().len(), 1);

            server.shutdown(5).await;
        }

        /// When workspace folders are added while the server is in single-file mode,
        /// it should exit single-file mode and shut down existing single-file workers.
        #[tokio::test]
        async fn test_exiting_single_file_mode_when_workspace_added() {
            let mut server = TestServer::new_initialized(
                |client| Backend::new(client, server_info(), create_workspace_manager()),
                initialize_request_workspace_folders(single_file_mode_initialize()),
            )
            .await;

            // Open a file to create a single-file worker.
            let file = "file:///path/to/some/file.js";
            server.send_request(did_open(file, "content")).await;

            // Confirm the dynamically-created worker exists.
            server.send_request(test_configuration_request(2)).await;
            let response = server.recv_response().await;
            assert_eq!(response.result().unwrap().as_array().unwrap().len(), 1);

            // Add a workspace folder — this should exit single-file mode and remove the
            // dynamically-created worker, replacing it with the new explicit workspace worker.
            server
                .send_request(workspace_folders_changed(
                    vec![WorkspaceFolder {
                        uri: Uri::from_str(WORKSPACE).unwrap(),
                        name: "workspace".to_string(),
                    }],
                    vec![],
                ))
                .await;

            // Now there should be exactly one worker: the explicitly added workspace.
            server.send_request(test_configuration_request(3)).await;
            let response = server.recv_response().await;
            assert!(response.is_ok());
            let workers = response.result().unwrap().as_array().unwrap();
            assert_eq!(workers.len(), 1);

            server.shutdown(4).await;
        }

        #[tokio::test]
        async fn test_dynamic_workers_show_client_message_in_single_file_mode() {
            let builder = FakeToolBuilder {
                build_client_message: vec![ClientMessage {
                    message: "Fake misconfiguration message".to_string(),
                    r#type: MessageType::Warning,
                }],
                ..Default::default()
            };

            let mut server = TestServer::new_initialized(
                |client| {
                    Backend::new(
                        client,
                        server_info(),
                        create_workspace_manager_with_builder(builder),
                    )
                },
                initialize_request_workspace_folders(single_file_mode_initialize()),
            )
            .await;

            // Opening files from different parent folders creates distinct dynamic workers.
            let file_a = "file:///path/to/dir_a/file.js";
            let file_b = "file:///path/to/dir_b/file.js";

            server.send_request(did_open(file_a, "a")).await;
            let show_message = server.recv_notification().await;
            assert_eq!(show_message.method(), "window/showMessage");
            let params = show_message.params().unwrap();
            assert_eq!(params["message"], "Fake misconfiguration message");
            assert_eq!(params["type"], json!(MessageType::Warning));

            server.send_request(did_open(file_b, "b")).await;
            let show_message = server.recv_notification().await;
            assert_eq!(show_message.method(), "window/showMessage");
            let params = show_message.params().unwrap();
            assert_eq!(params["message"], "Fake misconfiguration message");
            assert_eq!(params["type"], json!(MessageType::Warning));

            server.shutdown(4).await;
        }

        #[tokio::test]
        async fn test_single_file_mode_creates_worker_on_open() {
            let mut server = TestServer::new_initialized(
                |client| Backend::new(client, server_info(), create_workspace_manager()),
                initialize_request_workspace_folders(single_file_mode_initialize()),
            )
            .await;

            // Before any file is opened there should be no workers.
            server.send_request(test_configuration_request(2)).await;
            let response = server.recv_response().await;
            assert!(response.is_ok());
            assert_eq!(*response.result().unwrap(), json!([]));

            // Open a file – this should cause a workspace worker to be created for its parent directory.
            let file = "file:///path/to/some/file.js";
            server.send_request(did_open(file, "content")).await;

            // After opening the file there should be exactly one worker.
            server.send_request(test_configuration_request(3)).await;
            let response = server.recv_response().await;
            assert!(response.is_ok());
            assert_eq!(response.id(), &Id::Number(3));
            let workers = response.result().unwrap().as_array().unwrap().len();
            assert_eq!(workers, 1);

            server.shutdown(4).await;
        }

        #[tokio::test]
        async fn test_single_file_mode_removes_worker_on_last_close() {
            let mut server = TestServer::new_initialized(
                |client| Backend::new(client, server_info(), create_workspace_manager()),
                initialize_request_workspace_folders(single_file_mode_initialize()),
            )
            .await;

            let file = "file:///path/to/some/file.js";
            server.send_request(did_open(file, "content")).await;

            // Confirm the worker was created.
            server.send_request(test_configuration_request(2)).await;
            let response = server.recv_response().await;
            assert_eq!(response.result().unwrap().as_array().unwrap().len(), 1);

            // Close the only open file – the workspace worker should be removed.
            server.send_request(did_close(file)).await;

            server.send_request(test_configuration_request(3)).await;
            let response = server.recv_response().await;
            assert!(response.is_ok());
            assert_eq!(response.id(), &Id::Number(3));
            assert_eq!(*response.result().unwrap(), json!([]));

            server.shutdown(4).await;
        }

        #[tokio::test]
        async fn test_single_file_mode_keeps_worker_when_sibling_still_open() {
            let mut server = TestServer::new_initialized(
                |client| Backend::new(client, server_info(), create_workspace_manager()),
                initialize_request_workspace_folders(single_file_mode_initialize()),
            )
            .await;

            let file_a = "file:///path/to/some/a.js";
            let file_b = "file:///path/to/some/b.js";

            // Open two files in the same directory – both should share one worker.
            server.send_request(did_open(file_a, "a")).await;
            server.send_request(did_open(file_b, "b")).await;

            server.send_request(test_configuration_request(2)).await;
            let response = server.recv_response().await;
            assert_eq!(response.result().unwrap().as_array().unwrap().len(), 1);

            // Close one of them – the worker must remain because the sibling is still open.
            server.send_request(did_close(file_a)).await;

            server.send_request(test_configuration_request(3)).await;
            let response = server.recv_response().await;
            assert!(response.is_ok());
            assert_eq!(response.id(), &Id::Number(3));
            assert_eq!(response.result().unwrap().as_array().unwrap().len(), 1);

            server.shutdown(4).await;
        }

        #[tokio::test]
        async fn test_single_file_mode_separate_workers_for_different_dirs() {
            let mut server = TestServer::new_initialized(
                |client| Backend::new(client, server_info(), create_workspace_manager()),
                initialize_request_workspace_folders(single_file_mode_initialize()),
            )
            .await;

            let file_a = "file:///path/to/dir_a/file.js";
            let file_b = "file:///path/to/dir_b/file.js";

            // Open files from two different directories – each should get its own worker.
            server.send_request(did_open(file_a, "a")).await;
            server.send_request(did_open(file_b, "b")).await;

            server.send_request(test_configuration_request(2)).await;
            let response = server.recv_response().await;
            assert!(response.is_ok());
            assert_eq!(response.result().unwrap().as_array().unwrap().len(), 2);

            server.shutdown(4).await;
        }

        #[tokio::test]
        async fn test_single_file_mode_non_file_uri_skipped() {
            let mut server = TestServer::new_initialized(
                |client| Backend::new(client, server_info(), create_workspace_manager()),
                initialize_request_workspace_folders(single_file_mode_initialize()),
            )
            .await;

            // Non-file:// URIs (e.g. untitled) must not cause worker creation in single-file mode.
            server.send_request(did_open("untitled:///Untitled-1", "content")).await;

            server.send_request(test_configuration_request(2)).await;
            let response = server.recv_response().await;
            assert!(response.is_ok());
            assert_eq!(*response.result().unwrap(), json!([]));

            server.shutdown(3).await;
        }

        #[tokio::test]
        async fn test_single_file_mode_push_diagnostics() {
            let mut server = TestServer::new_initialized(
                |client| {
                    Backend::new(
                        client,
                        server_info(),
                        create_workspace_manager_with_builder(FakeToolBuilder::new(
                            DiagnosticMode::Push,
                        )),
                    )
                },
                initialize_request_workspace_folders(single_file_mode_initialize()),
            )
            .await;

            // Opening a diagnostic file should trigger push diagnostics even in single-file mode.
            let file = "file:///path/to/some/diagnostics.config";
            server.send_request(did_open(file, "content")).await;

            let notification = server.recv_notification().await;
            assert_eq!(notification.method(), "textDocument/publishDiagnostics");
            let params: PublishDiagnosticsParams =
                serde_json::from_value(notification.params().unwrap().clone()).unwrap();
            assert_eq!(params.uri.to_string(), file);
            assert_eq!(params.diagnostics.len(), 1);

            // Closing the file shuts down the dynamic workspace and clears the pushed diagnostics.
            server.send_request(did_close(file)).await;
            let clear_notification = server.recv_notification().await;
            assert_eq!(clear_notification.method(), "textDocument/publishDiagnostics");
            let clear_params: PublishDiagnosticsParams =
                serde_json::from_value(clear_notification.params().unwrap().clone()).unwrap();
            assert_eq!(clear_params.uri.to_string(), file);
            assert!(clear_params.diagnostics.is_empty(), "diagnostics should be cleared on close");

            server.shutdown(2).await;
        }

        #[tokio::test]
        async fn test_single_file_mode_close_all_removes_all_workers() {
            let mut server = TestServer::new_initialized(
                |client| Backend::new(client, server_info(), create_workspace_manager()),
                initialize_request_workspace_folders(single_file_mode_initialize()),
            )
            .await;

            let file_a = "file:///path/to/dir_a/file.js";
            let file_b = "file:///path/to/dir_b/file.js";

            server.send_request(did_open(file_a, "a")).await;
            server.send_request(did_open(file_b, "b")).await;

            // Close both files – both dynamically created workspace workers should be removed.
            server.send_request(did_close(file_a)).await;
            server.send_request(did_close(file_b)).await;

            server.send_request(test_configuration_request(2)).await;
            let response = server.recv_response().await;
            assert!(response.is_ok());
            assert_eq!(*response.result().unwrap(), json!([]));

            server.shutdown(3).await;
        }
    }

    mod dynamic_mode {
        use crate::tests::create_dynamic_workspace_manager;

        use super::*;

        #[tokio::test]
        async fn test_dynamic_mode_pull_diagnostics() {
            let init_options = InitializeRequestOptions { pull_mode: true, ..Default::default() };

            let mut server = TestServer::new_initialized(
                |client| {
                    Backend::new(
                        client,
                        server_info(),
                        create_dynamic_workspace_manager(FakeToolBuilder::default()),
                    )
                },
                initialize_request(init_options),
            )
            .await;

            let file = format!("{WORKSPACE}/diagnostics.config");
            let content = "pull mode text";
            server.send_request(did_open(&file, content)).await;
            server.send_request(diagnostic(3, &file)).await;

            let diagnostic_response = server.recv_response().await;
            assert!(diagnostic_response.is_ok());
            assert_eq!(diagnostic_response.id(), &Id::Number(3));

            let report: serde_json::Value =
                serde_json::from_value(diagnostic_response.result().unwrap().clone()).unwrap();

            assert_eq!(report["kind"], "full");
            assert_eq!(report["items"].as_array().unwrap().len(), 1);

            let file = "file:///outside-workspace/diagnostics.config";

            server.send_request(did_open(file, content)).await;
            server.send_request(diagnostic(4, file)).await;

            let diagnostic_response = server.recv_response().await;
            assert!(diagnostic_response.is_ok());
            assert_eq!(diagnostic_response.id(), &Id::Number(4));

            let report: serde_json::Value =
                serde_json::from_value(diagnostic_response.result().unwrap().clone()).unwrap();

            assert_eq!(report["kind"], "full");
            assert_eq!(report["items"].as_array().unwrap().len(), 1);

            server.shutdown(5).await;
        }

        #[tokio::test]
        async fn test_dynamic_mode_init_watchers() {
            let mut server = TestServer::new_initialized(
                |client| {
                    Backend::new(
                        client,
                        server_info(),
                        create_dynamic_workspace_manager(FakeToolBuilder::default()),
                    )
                },
                initialize_request(InitializeRequestOptions {
                    dynamic_watchers: true,
                    ..Default::default()
                }),
            )
            .await;

            let workspace_config_request = server.recv_notification().await;
            assert_eq!(workspace_config_request.method(), "client/registerCapability");
            assert_eq!(workspace_config_request.id(), Some(&Id::Number(0)));
            // we do not expect any more watchers for the dynamic workspace
            // the current implementation is too expensive, it watches for the complete file system
            assert_eq!(
                workspace_config_request.params(),
                Some(&json!({
                    "registrations": [
                        {
                            "id": format!("watcher-{WORKSPACE}"),
                            "method": "workspace/didChangeWatchedFiles",
                            "registerOptions": {
                                "watchers": [
                                    {
                                        "globPattern": {
                                            "baseUri": WORKSPACE,
                                            "pattern": "**/fake.config",
                                        },
                                        "kind": 7
                                    }
                                ]
                            },
                        }
                    ]
                }))
            );

            server.send_ack(&Id::Number(0)).await;

            server.shutdown(2).await;
        }
    }

    mod request_locks {
        use std::time::Instant;

        use crate::tests::{FakeToolDelays, create_dynamic_workspace_manager};

        use super::*;
        #[tokio::test]
        #[ignore = "This needs to be fixed"]
        async fn test_request_locks() {
            let delay = 100;
            let mut server = TestServer::new_initialized(
                |client| {
                    Backend::new(
                        client,
                        server_info(),
                        create_dynamic_workspace_manager(
                            FakeToolBuilder::new(DiagnosticMode::Pull)
                                .with_delays(FakeToolDelays { run_diagnostic: delay }),
                        ),
                    )
                },
                initialize_request(InitializeRequestOptions::default()),
            )
            .await;

            let file = format!("{WORKSPACE}/diagnostics.config");
            server.send_request(did_open(&file, "content")).await;

            let now = Instant::now();

            // Send multiple diagnostic requests
            for i in 0..5 {
                server.send_request(diagnostic(1 + i, &file)).await;
            }
            server.send_request(code_action(6, &file)).await;

            let mut diagnostic_responses = 0;
            loop {
                let response = server.recv_response().await;
                if response.id() == &Id::Number(6) {
                    break;
                }
                diagnostic_responses += 1;
            }

            let elapsed = now.elapsed().as_millis();

            while diagnostic_responses < 5 {
                let response = server.recv_response().await;
                assert_ne!(response.id(), &Id::Number(6));
                diagnostic_responses += 1;
            }

            server.shutdown(7).await;

            // The diagnostic request should not block the code action request,
            // so the total elapsed time should be less than delay * number of diagnostic requests.
            assert!(
                elapsed < u128::from(delay * 5),
                "Diagnostic requests are blocking the code action request, elapsed time: {elapsed}ms, expected under {}ms",
                delay * 5
            );
        }
    }
}
