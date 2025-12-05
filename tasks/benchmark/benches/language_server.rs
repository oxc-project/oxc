use std::collections::VecDeque;

use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_language_server::{ServerLinterBuilder, build_backend};
use oxc_tasks_common::TestFiles;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
use tower_lsp_server::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::{Request, Response},
    ls_types::{
        DidOpenTextDocumentParams, InitializeParams, InitializedParams, TextDocumentItem,
        WorkspaceFolder,
    },
};

/// Creates an initialize request with the given parameters.
/// Uses a single workspace folder at WORKSPACE.
///
/// # Panics
/// - If the workspace URI is not a valid URI.
fn initialize_request() -> Request {
    let params = InitializeParams {
        workspace_folders: Some(vec![WorkspaceFolder {
            uri: WORKSPACE.parse().unwrap(),
            name: "workspace".to_string(),
        }]),
        ..Default::default()
    };

    Request::build("initialize").params(json!(params)).id(1).finish()
}

fn initialized_notification() -> Request {
    let params = InitializedParams {};

    Request::build("initialized").params(json!(params)).finish()
}

pub fn shutdown_request(id: i64) -> Request {
    Request::build("shutdown").id(id).finish()
}

/// Creates a didOpen notification for the given URI and text.
///
/// # Panics
/// - If the URI is not a valid URI.
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

const WORKSPACE: &str = "file:///path/to/workspace";

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

        let (service, socket) = LspService::build(init).finish();

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

    /// Sends a request to the server.
    ///
    /// # Panics
    /// - If the stream cannot be written to.
    pub async fn send_request(&mut self, req: Request) {
        let req = serde_json::to_string(&req).unwrap();
        let req = Self::encode(&req);
        self.req_stream.write_all(req.as_bytes()).await.unwrap();
    }

    /// Receives a response from the server.
    ///
    /// # Panics
    /// - If the stream cannot be read.
    /// - If the response cannot be deserialized.
    pub async fn recv_response(&mut self) -> Response {
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

fn bench_linter(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("language_server");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(format!("{}-linter", file.file_name));
        let source_text = &file.source_text;
        let uri = format!("file:///{WORKSPACE}{}", file.file_name);

        group.bench_function(id, |b| {
            b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
                let mut server = TestServer::new(|client| {
                    build_backend(
                        client,
                        "benchmark".to_string(),
                        "0.0.0".to_string(),
                        vec![Box::new(ServerLinterBuilder)],
                    )
                });
                // Send initialize request
                server.send_request(initialize_request()).await;
                let _ = server.recv_response().await;

                // Send initialized notification
                server.send_request(initialized_notification()).await;

                // Send didOpen notification, expecting the linter to run
                server.send_request(did_open(&uri, source_text)).await;

                // Shutdown the server
                server.send_request(shutdown_request(2)).await;
                let _ = server.recv_response().await;
            });
        });
    }
    group.finish();
}

criterion_group!(language_server, bench_linter);
criterion_main!(language_server);
