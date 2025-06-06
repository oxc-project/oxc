use criterion::{Criterion, criterion_group, criterion_main};
use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

fn send_lsp_message(stdin: &mut impl Write, message: &str) {
    let full = format!("Content-Length: {}\r\n\r\n{}", message.len(), message);
    stdin.write_all(full.as_bytes()).unwrap();
    stdin.flush().unwrap();
}

fn wait_lsp_response(reader: &mut BufReader<impl std::io::Read>) {
    let mut buffer = String::new();
    let mut content_length = 0;

    // Read headers
    loop {
        buffer.clear();
        reader.read_line(&mut buffer).unwrap();
        if buffer.starts_with("Content-Length:") {
            content_length = buffer[16..].trim().parse().unwrap();
        }
        if buffer == "\r\n" || buffer == "\n" {
            break;
        }
    }

    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body).unwrap();
}

fn code_action_request(path: &Path, id: u64) -> String {
    format!(
        r#"{{
        "jsonrpc":"2.0",
        "id": {id},
        "method":"textDocument/codeAction",
        "params":{{
            "textDocument": {{"uri": "file://{}"}},
            "range": {{
                "start": {{ "line": 0, "character": 0 }},
                "end": {{ "line": 0, "character": 10 }}
            }},
            "context": {{
                "diagnostics": []
            }}
        }}
    }}"#,
        path.to_string_lossy()
    )
}

#[expect(clippy::zombie_processes)] // fix me
fn benchmark_language_server(criterion: &mut Criterion) {
    let vscode_dir =
        env::current_dir().unwrap().join("../../editors/vscode").canonicalize().unwrap();
    let mut group = criterion.benchmark_group("language_server");

    let mut child = Command::new(vscode_dir.join("target/debug/oxc_language_server"))
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start LSP server");

    let mut stdin = child.stdin.take().expect("failed to open stdin");
    let stdout = child.stdout.take().expect("failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // 1. Send initialize request
    let initialize = format!(
        r#"{{
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {{
            "capabilities": {{}},
            "workspaceFolders": [{{
                "name": "bench",
                "uri": "file://{}"
            }}]
        }}
    }}"#,
        vscode_dir.join("fixtures/debugger_error").to_string_lossy()
    );
    send_lsp_message(&mut stdin, &initialize);

    // 2. Wait for initialize response
    wait_lsp_response(&mut reader);

    // 3. Send initialized notification (so we start the linter)
    let initialized = r#"{
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    }"#;
    send_lsp_message(&mut stdin, initialized);

    let mut count = 0;
    let file_path = vscode_dir.join("fixtures/debugger_error/debugger.js");

    // Benchmark the `textDocument/codeAction` request without sending a `textDocument/didOpen` request.
    // so we are making sure the server is not caching the diagnostics, instead it lints the file every time.
    // can be improved by creating a custom request endpoint.
    group.bench_function("code_action", |b| {
        b.iter_custom(|iters| {
            use std::time::Instant;
            let start = Instant::now();
            for _ in 0..iters {
                send_lsp_message(&mut stdin, &code_action_request(&file_path, count + 2));
                wait_lsp_response(&mut reader);
                count += 1;
            }

            start.elapsed()
        });
    });

    // Kill the server
    let _ = child.kill();
}

criterion_group!(benches, benchmark_language_server);
criterion_main!(benches);
