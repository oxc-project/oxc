use std::sync::{OnceLock, RwLock};
use tower_lsp_server::{Client, lsp_types::MessageType};

// Global LSP client used for window/logMessage forwarding. We allow replacement
// on language server restarts. OnceLock guards initialization of the RwLock itself.
static GLOBAL_CLIENT: OnceLock<RwLock<Option<Client>>> = OnceLock::new();

fn client_cell() -> &'static RwLock<Option<Client>> {
    GLOBAL_CLIENT.get_or_init(|| RwLock::new(None))
}

/// Set or replace the global client (called from Backend::new on each start/restart).
pub fn set_global_client(client: Client) {
    *client_cell().write().unwrap() = Some(client);
}

/// Get a clone of the current client for sending notifications.
pub fn get_global_client() -> Option<Client> {
    client_cell().read().unwrap().as_ref().cloned()
}

// Global logger wrapper forwarding ALL log crate emissions to LSP window/logMessage.
pub struct LspForwardingLogger {
    fallback: env_logger::Logger,
}

impl log::Log for LspForwardingLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.fallback.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        // Try forwarding to LSP client if initialized.
        if let Some(client) = get_global_client() {
            let level = match record.level() {
                log::Level::Error => MessageType::ERROR,
                log::Level::Warn => MessageType::WARNING,
                log::Level::Info => MessageType::INFO,
                _ => MessageType::LOG,
            };
            // The tower_lsp_server Client API is async; calling log_message returns a lazy Future.
            // Previously we ignored it, so the notification was never actually sent.
            // Spawn the future on the current Tokio runtime if available. If no runtime is
            // present (e.g. in certain test contexts), we silently skip forwarding to avoid
            // panicking. The fallback logger below still emits the message to stdout/stderr.
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                let msg = record.args().to_string();
                handle.spawn(async move {
                    client.log_message(level, msg).await;
                });
            }
        }
        // Always emit via fallback (stdout/stderr formatting, filtering, etc.).
        self.fallback.log(record);
    }

    fn flush(&self) {
        self.fallback.flush();
    }
}

/// Initialize the global logger with LSP forwarding. Call exactly once at program start.
pub fn init_global_logger() {
    // Build env_logger using environment configuration (RUST_LOG, etc.)
    let mut builder = env_logger::Builder::from_env(env_logger::Env::default());
    let fallback = builder.build();
    // Install composite logger. Ignore error if someone already set a logger (tests).
    if log::set_boxed_logger(Box::new(LspForwardingLogger { fallback })).is_ok() {
        // Allow logger's internal filter to decide; pass everything through log crate.
        log::set_max_level(log::LevelFilter::Trace);
    }
}
