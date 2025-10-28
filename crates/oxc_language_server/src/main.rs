use rustc_hash::FxBuildHasher;
#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use tokio::net::UnixListener;
use tower_lsp_server::{LspService, Server};

mod backend;
mod capabilities;
mod code_actions;
mod commands;
mod file_system;
mod formatter;
mod linter;
mod log_bridge;
mod options;
#[cfg(test)]
mod tester;
mod utils;
mod worker;

use crate::backend::Backend;

type ConcurrentHashMap<K, V> = papaya::HashMap<K, V, FxBuildHasher>;

const LINT_CONFIG_FILE: &str = ".oxlintrc.json";
const FORMAT_CONFIG_FILES: &[&str; 2] = &[".oxfmtrc.json", ".oxfmtrc.jsonc"];

#[tokio::main]
async fn main() {
    // Initialize composite logger that forwards log crate events to LSP (client set later).
    crate::log_bridge::init_global_logger();
    // Optional external listen mode for debugger / external host integration.
    // Activate by setting env var: OXC_LS_LISTEN=unix:/tmp/oxc_ls.sock
    #[cfg(unix)]
    if let Ok(listen_spec) = std::env::var("OXC_LS_LISTEN") {
        if let Some(path) = listen_spec.strip_prefix("unix:") {
            // Remove stale socket file (ignore errors) to avoid bind failures.
            let _ = std::fs::remove_file(path);

            match UnixListener::bind(path) {
                Ok(listener) => {
                    // Guard to remove the socket file when the process exits (listener dropped).
                    struct SocketCleanup(PathBuf);
                    impl Drop for SocketCleanup {
                        fn drop(&mut self) {
                            // Best-effort cleanup; ignore errors.
                            let _ = std::fs::remove_file(&self.0);
                        }
                    }
                    let _cleanup = SocketCleanup(PathBuf::from(path));

                    eprintln!("[oxc-language-server] Listening on unix socket: {path}");

                    // Accept loop: allow sequential LSP sessions (e.g., VSCode client.restart())
                    // without requiring an external supervisor to recreate the server.
                    // Each iteration runs a full LSP lifecycle (initialize -> ... -> shutdown/exit).
                    // The VSCode extension sends explicit shutdown/exit before a restart when
                    // OXC_LS_CONNECT is set. After serve() returns we immediately accept the next
                    // connection. This prevents connection refusal races previously observed when
                    // the server exited entirely and the client retried before a new process was
                    // spawned.
                    loop {
                        let (stream, _addr) = match listener.accept().await {
                            Ok(v) => v,
                            Err(err) => {
                                eprintln!(
                                    "[oxc-language-server] Accept error: {err}. Shutting down accept loop."
                                );
                                break; // exit to stdio fallback? we already bound unix, so just end.
                            }
                        };

                        eprintln!("[oxc-language-server] Client connected. Starting LSP session.");
                        let (service, socket) = LspService::build(Backend::new).finish();
                        let (read_half, write_half) = tokio::io::split(stream);
                        Server::new(read_half, write_half, socket).serve(service).await;
                        eprintln!(
                            "[oxc-language-server] LSP session ended. Waiting for next client..."
                        );
                        // Loop continues to accept next connection (e.g., after a client restart).
                    }
                    // After breaking from loop (e.g., accept error), exit main.
                    return;
                }
                Err(err) => {
                    eprintln!(
                        "[oxc-language-server] Failed to bind unix socket {path}: {err}. Falling back to stdio."
                    );
                }
            }
        }
    }

    // Fallback stdio mode (default when no listen spec provided)
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new).finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
