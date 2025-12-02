use std::collections::VecDeque;

use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
use tower_lsp_server::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::{ErrorCode, Id, Request, Response},
    lsp_types::*,
};

use crate::{Tool, ToolBuilder, ToolRestartChanges};

pub struct FakeToolBuilder;

impl ToolBuilder for FakeToolBuilder {
    fn build_boxed(&self, _root_uri: &Uri, _options: serde_json::Value) -> Box<dyn Tool> {
        Box::new(FakeTool)
    }
}

pub struct FakeTool;

pub const FAKE_COMMAND: &str = "fake.command";

const WORKSPACE: &str = "file:///path/to/workspace";

impl Tool for FakeTool {
    fn name(&self) -> &'static str {
        "FakeTool"
    }

    fn is_responsible_for_command(&self, command: &str) -> bool {
        command == FAKE_COMMAND
    }

    fn execute_command(
        &self,
        command: &str,
        arguments: Vec<serde_json::Value>,
    ) -> Result<Option<WorkspaceEdit>, ErrorCode> {
        if command != FAKE_COMMAND {
            return Err(ErrorCode::MethodNotFound);
        }

        if !arguments.is_empty() {
            return Ok(Some(WorkspaceEdit::default()));
        }

        Ok(None)
    }

    fn handle_configuration_change(
        &self,
        root_uri: &Uri,
        _old_options_json: &serde_json::Value,
        new_options_json: serde_json::Value,
    ) -> ToolRestartChanges {
        if new_options_json.as_u64() == Some(1) {
            return ToolRestartChanges {
                tool: Some(FakeToolBuilder.build_boxed(root_uri, new_options_json)),
                diagnostic_reports: None,
                watch_patterns: None,
            };
        }
        if new_options_json.as_u64() == Some(2) {
            return ToolRestartChanges {
                tool: None,
                diagnostic_reports: None,
                watch_patterns: Some(vec!["**/new_watcher.config".to_string()]),
            };
        }
        if new_options_json.as_u64() == Some(3) {
            return ToolRestartChanges {
                tool: None,
                diagnostic_reports: Some(vec![(
                    root_uri.to_string(),
                    vec![Diagnostic {
                        message: "Fake diagnostic".to_string(),
                        ..Default::default()
                    }],
                )]),
                watch_patterns: None,
            };
        }
        ToolRestartChanges { tool: None, diagnostic_reports: None, watch_patterns: None }
    }

    fn get_watcher_patterns(
        &self,
        options: serde_json::Value,
    ) -> Vec<tower_lsp_server::lsp_types::Pattern> {
        if !matches!(options, serde_json::Value::Null) {
            return vec![];
        }
        vec!["**/fake.config".to_string()]
    }

    fn handle_watched_file_change(
        &self,
        changed_uri: &Uri,
        root_uri: &Uri,
        options: serde_json::Value,
    ) -> ToolRestartChanges {
        if changed_uri.as_str().ends_with("tool.config") {
            return ToolRestartChanges {
                tool: Some(FakeToolBuilder.build_boxed(root_uri, options)),
                diagnostic_reports: None,
                watch_patterns: None,
            };
        }
        if changed_uri.as_str().ends_with("watcher.config") {
            return ToolRestartChanges {
                tool: None,
                diagnostic_reports: None,
                watch_patterns: Some(vec!["**/new_watcher.config".to_string()]),
            };
        }
        if changed_uri.as_str().ends_with("diagnostics.config") {
            return ToolRestartChanges {
                tool: None,
                diagnostic_reports: Some(vec![(
                    changed_uri.to_string(),
                    vec![Diagnostic {
                        message: "Fake diagnostic".to_string(),
                        ..Default::default()
                    }],
                )]),
                watch_patterns: None,
            };
        }
        ToolRestartChanges { tool: None, diagnostic_reports: None, watch_patterns: None }
    }

    fn get_code_actions_or_commands(
        &self,
        uri: &Uri,
        _range: &Range,
        _only_code_action_kinds: Option<Vec<CodeActionKind>>,
    ) -> Vec<CodeActionOrCommand> {
        if uri.as_str().ends_with("code_action.config") {
            return vec![CodeActionOrCommand::CodeAction(CodeAction {
                title: "Code Action title".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                edit: Some(WorkspaceEdit::default()),
                ..Default::default()
            })];
        }

        vec![]
    }
}

// A test server that can send requests and receive responses.
// Copied from <https://github.com/veryl-lang/veryl/blob/888d83abaa58ca5a7ffef501a1c557e48c750b92/crates/languageserver/src/tests.rs>
struct TestServer {
    req_stream: DuplexStream,
    res_stream: DuplexStream,
    responses: VecDeque<String>,
}

impl TestServer {
    fn new<F, S>(init: F) -> Self
    where
        F: FnOnce(Client) -> S,
        S: LanguageServer,
    {
        let (req_client, req_server) = tokio::io::duplex(1024);
        let (res_server, res_client) = tokio::io::duplex(1024);

        let (service, socket) = LspService::new(init);

        tokio::spawn(Server::new(req_server, res_server, socket).serve(service));

        Self { req_stream: req_client, res_stream: res_client, responses: VecDeque::new() }
    }

    fn encode(payload: &str) -> String {
        format!("Content-Length: {}\r\n\r\n{}", payload.len(), payload)
    }

    fn decode(text: &str) -> Vec<String> {
        let mut ret = Vec::new();
        let mut temp = text;

        while !temp.is_empty() {
            let p = temp.find("\r\n\r\n").unwrap();
            let (header, body) = temp.split_at(p + 4);
            let len =
                header.strip_prefix("Content-Length: ").unwrap().strip_suffix("\r\n\r\n").unwrap();
            let len: usize = len.parse().unwrap();
            let (body, rest) = body.split_at(len);
            ret.push(body.to_string());
            temp = rest;
        }

        ret
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
            let mut buf = vec![0; 1024];
            let n = self.res_stream.read(&mut buf).await.unwrap();
            let ret = String::from_utf8(buf[..n].to_vec()).unwrap();
            for x in Self::decode(&ret) {
                self.responses.push_front(x);
            }
        }
        let res = self.responses.pop_back().unwrap();
        serde_json::from_str(&res).unwrap()
    }

    async fn recv_notification(&mut self) -> Request {
        if self.responses.is_empty() {
            let mut buf = vec![0; 1024];
            let n = self.res_stream.read(&mut buf).await.unwrap();
            let ret = String::from_utf8(buf[..n].to_vec()).unwrap();
            for x in Self::decode(&ret) {
                self.responses.push_front(x);
            }
        }
        let res = self.responses.pop_back().unwrap();
        serde_json::from_str(&res).unwrap()
    }

    /// Creates a new TestServer and performs the initialize and initialized sequence.
    /// The `init` closure is used to create the LanguageServer instance.
    async fn new_initialized<F, S>(init: F, initialize: Request) -> Self
    where
        F: FnOnce(Client) -> S,
        S: LanguageServer,
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

    async fn shutdown_with_watchers(&mut self, id: i64) {
        // shutdown request
        self.send_request(shutdown_request(id)).await;

        // watcher unregistration expected
        acknowledge_unregistrations(self).await;

        // shutdown response
        let shutdown_result = self.recv_response().await;

        assert!(shutdown_result.is_ok());
        assert_eq!(shutdown_result.id(), &Id::Number(id));
    }
}

fn initialize_request(
    workspace_configuration: bool,
    dynamic_watchers: bool,
    workspace_edit: bool,
) -> Request {
    let params = InitializeParams {
        workspace_folders: Some(vec![WorkspaceFolder {
            uri: WORKSPACE.parse().unwrap(),
            name: "workspace".to_string(),
        }]),
        capabilities: ClientCapabilities {
            workspace: Some(WorkspaceClientCapabilities {
                apply_edit: Some(workspace_edit),
                configuration: Some(workspace_configuration),
                did_change_watched_files: Some(DidChangeWatchedFilesClientCapabilities {
                    dynamic_registration: Some(dynamic_watchers),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    };

    Request::build("initialize").params(json!(params)).id(1).finish()
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
            uri: uri.parse().unwrap(),
            language_id: "plaintext".to_string(),
            version: 1,
            text: text.to_string(),
        },
    };

    Request::build("textDocument/didOpen").params(json!(params)).finish()
}

fn did_change(uri: &str, text: &str) -> Request {
    let params = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier { uri: uri.parse().unwrap(), version: 2 },
        content_changes: vec![TextDocumentContentChangeEvent {
            text: text.to_string(),
            range: None,
            range_length: None,
        }],
    };

    Request::build("textDocument/didChange").params(json!(params)).finish()
}

fn did_close(uri: &str) -> Request {
    let params = DidCloseTextDocumentParams {
        text_document: TextDocumentIdentifier { uri: uri.parse().unwrap() },
    };

    Request::build("textDocument/didClose").params(json!(params)).finish()
}

fn code_action(id: i64, uri: &str) -> Request {
    let params = CodeActionParams {
        text_document: TextDocumentIdentifier { uri: uri.parse().unwrap() },
        range: Range::default(),
        context: CodeActionContext { diagnostics: vec![], only: None, trigger_kind: None },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    Request::build("textDocument/codeAction").id(id).params(json!(params)).finish()
}

#[cfg(test)]
mod test_suite {
    use serde_json::{Value, json};
    use tower_lsp_server::{
        jsonrpc::{Id, Response},
        lsp_types::{
            ApplyWorkspaceEditResponse, InitializeResult, ServerInfo, WorkspaceEdit,
            WorkspaceFolder,
        },
    };

    use crate::{
        backend::Backend,
        tests::{
            FAKE_COMMAND, FakeToolBuilder, TestServer, WORKSPACE, acknowledge_registrations,
            acknowledge_unregistrations, code_action, did_change, did_change_configuration,
            did_change_watched_files, did_close, did_open, execute_command_request,
            initialize_request, initialized_notification, response_to_configuration,
            shutdown_request, workspace_folders_changed,
        },
    };

    fn server_info() -> ServerInfo {
        ServerInfo { name: "oxc".to_owned(), version: Some("1.0.0".to_owned()) }
    }

    #[tokio::test]
    async fn test_basic_start_and_shutdown_flow() {
        let mut server = TestServer::new(|client| Backend::new(client, server_info(), vec![]));
        // initialize request
        server.send_request(initialize_request(false, false, false)).await;
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
    async fn test_workspace_configuration_on_initialized() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![]),
            initialize_request(true, false, false),
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
    async fn test_dynamic_watched_files_registration() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(false, true, false),
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
                        "id": format!("watcher-FakeTool-{WORKSPACE}"),
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

        // client/unregisterCapability request
        let unregister_request = server.recv_notification().await;
        assert_eq!(unregister_request.method(), "client/unregisterCapability");
        assert_eq!(unregister_request.id(), Some(&Id::Number(1)));
        assert_eq!(
            unregister_request.params(),
            Some(&json!({
                "unregisterations": [
                    {
                        "id": format!("watcher-FakeTool-{WORKSPACE}"),
                        "method": "workspace/didChangeWatchedFiles",
                    }
                ]
            }))
        );
        // Acknowledge the unregistration
        server.send_ack(&Id::Number(1)).await;

        // shutdown response
        let shutdown_result = server.recv_response().await;

        assert!(shutdown_result.is_ok());
        assert_eq!(shutdown_result.id(), &Id::Number(2));
    }

    #[tokio::test]
    async fn test_execute_workspace_command_with_apply_edit() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(false, false, true),
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
                json!(ApplyWorkspaceEditResponse {
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
        assert!(execute_command_response.id() == &Id::Number(3));
        assert_eq!(execute_command_response.result().unwrap(), &json!(null));

        // shutdown request
        server.shutdown(4).await;
    }

    #[tokio::test]
    async fn test_execute_workspace_command_with_no_edit() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(false, false, false),
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
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(false, false, false),
        )
        .await;

        // execute command request with an invalid command
        let execute_command_request = execute_command_request("invalid.command", &[], 3);
        server.send_request(execute_command_request).await;

        // Should not return an error, but a null result
        let execute_command_response = server.recv_response().await;
        assert!(execute_command_response.is_ok());
        assert_eq!(execute_command_response.id(), &Id::Number(3));
        assert_eq!(execute_command_response.result().unwrap(), &json!(null));

        server.shutdown(4).await;
    }

    #[tokio::test]
    async fn test_workspace_added() {
        // workspace/didChangeWorkspaceFolders notification
        let folders_changed_notification = workspace_folders_changed(
            vec![WorkspaceFolder {
                uri: "file:///path/to/new_folder".parse().unwrap(),
                name: "new_folder".to_string(),
            }],
            vec![],
        );

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(false, false, false),
        )
        .await;
        server.send_request(folders_changed_notification).await;

        // No direct response expected for notifications, client does not support workspace configuration or watchers
        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_workspace_added_watchers() {
        // workspace/didChangeWorkspaceFolders notification
        let folders_changed_notification = workspace_folders_changed(
            vec![WorkspaceFolder {
                uri: "file:///path/to/new_folder".parse().unwrap(),
                name: "new_folder".to_string(),
            }],
            vec![],
        );

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(false, true, false),
        )
        .await;
        acknowledge_registrations(&mut server).await;
        server.send_request(folders_changed_notification).await;

        // new watcher registration expected
        acknowledge_registrations(&mut server).await;
        server.shutdown_with_watchers(4).await;
    }

    #[tokio::test]
    async fn test_workspace_added_configuration() {
        // workspace/didChangeWorkspaceFolders notification
        let folders_changed_notification = workspace_folders_changed(
            vec![WorkspaceFolder {
                uri: "file:///path/to/new_folder".parse().unwrap(),
                name: "new_folder".to_string(),
            }],
            vec![],
        );

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(true, false, false),
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
                uri: WORKSPACE.parse().unwrap(),
                name: "workspace".to_string(),
            }],
        );

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(false, false, false),
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
                uri: WORKSPACE.parse().unwrap(),
                name: "workspace".to_string(),
            }],
        );

        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(false, true, false),
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
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(false, true, false),
        )
        .await;
        acknowledge_registrations(&mut server).await;

        // Simulate a watched file change notification
        let file_change_notification =
            did_change_watched_files(format!("{WORKSPACE}/unknown.file").as_str());
        server.send_request(file_change_notification).await;

        // Since FakeToolBuilder does not know about "unknown.file", no diagnostics or registrations are expected
        // Thus, no further requests or responses should occur

        server.shutdown_with_watchers(3).await;
    }

    #[tokio::test]
    async fn test_watched_file_changed_new_watchers() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(false, true, false),
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

        server.shutdown_with_watchers(3).await;
    }

    #[tokio::test]
    async fn test_did_change_configuration_no_changes() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(false, true, false),
        )
        .await;
        acknowledge_registrations(&mut server).await;

        // Simulate a configuration change that does not affect the tool
        let config_change_notification = did_change_configuration(None);
        server.send_request(config_change_notification).await;

        // When `null` is sent and the client does not support workspace configuration requests,
        // no configuration changes occur, so no diagnostics or registrations are expected.
        server.shutdown_with_watchers(3).await;
    }

    #[tokio::test]
    async fn test_did_change_configuration_config_passed_new_watchers() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(false, true, false),
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

        server.shutdown_with_watchers(3).await;
    }

    #[tokio::test]
    async fn test_did_change_configuration_config_requested_new_watchers() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(true, true, false),
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

        server.shutdown_with_watchers(3).await;
    }

    #[tokio::test]
    async fn test_file_notifications() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(false, false, false),
        )
        .await;

        let file = format!("{WORKSPACE}/file.txt");

        server.send_request(did_open(&file, "some text")).await;
        server.send_request(did_change(&file, "changed text")).await;
        server.send_request(did_close(&file)).await;
        server.shutdown(3).await;
    }

    #[tokio::test]
    async fn test_code_action_no_actions() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(false, false, false),
        )
        .await;

        let file = format!("{WORKSPACE}/file.txt");

        server.send_request(did_open(&file, "some text")).await;

        // No code actions expected
        server.send_request(code_action(3, &file)).await;
        let response = server.recv_response().await;
        assert!(response.is_ok());
        assert!(response.id() == &Id::Number(3));
        assert!(response.result().is_some_and(|result| *result == Value::Null));

        server.shutdown(4).await;
    }

    #[tokio::test]
    async fn test_code_actions_with_actions() {
        let mut server = TestServer::new_initialized(
            |client| Backend::new(client, server_info(), vec![Box::new(FakeToolBuilder)]),
            initialize_request(false, false, false),
        )
        .await;

        let file = format!("{WORKSPACE}/code_action.config");

        server.send_request(did_open(&file, "some text")).await;

        // Code actions expected
        server.send_request(code_action(3, &file)).await;
        let response = server.recv_response().await;
        assert!(response.is_ok());
        assert!(response.id() == &Id::Number(3));
        let actions: Vec<serde_json::Value> =
            serde_json::from_value(response.result().unwrap().clone()).unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0]["title"], "Code Action title");

        server.shutdown(4).await;
    }
}
