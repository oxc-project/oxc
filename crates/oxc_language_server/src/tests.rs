use std::collections::VecDeque;

use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
use tower_lsp_server::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::{Id, Request, Response},
    lsp_types::*,
};

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

    async fn _send_ack(&mut self, id: &Id) {
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
}

fn initialize_request(workspace_configuration: bool) -> Request {
    let params = InitializeParams {
        workspace_folders: Some(vec![WorkspaceFolder {
            uri: "file:///path/to/workspace".parse().unwrap(),
            name: "workspace".to_string(),
        }]),
        capabilities: ClientCapabilities {
            workspace: Some(WorkspaceClientCapabilities {
                configuration: Some(workspace_configuration),
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

#[cfg(test)]
mod test_suite {
    use serde_json::json;
    use tower_lsp_server::{
        jsonrpc::{Id, Response},
        lsp_types::InitializeResult,
    };

    use crate::{
        backend::Backend,
        tests::{TestServer, initialize_request, initialized_notification, shutdown_request},
    };

    #[tokio::test]
    async fn test_basic_start_and_shutdown_flow() {
        let mut server = TestServer::new(|client| Backend::new(client, vec![]));

        // initialize request
        server.send_request(initialize_request(false)).await;
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
        let mut server = TestServer::new(|client| Backend::new(client, vec![]));

        // initialize request
        server.send_request(initialize_request(true)).await;
        let initialize_result = server.recv_response().await;
        assert!(initialize_result.is_ok());

        // initialized notification
        server.send_request(initialized_notification()).await;

        // workspace/configuration request
        let workspace_config_request = server.recv_notification().await;
        assert_eq!(workspace_config_request.method(), "workspace/configuration");
        assert_eq!(workspace_config_request.id(), Some(&Id::Number(0)));
        assert_eq!(
            workspace_config_request.params(),
            Some(&json!({
                "items": [
                    {
                        "scopeUri": "file:///path/to/workspace",
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
}
