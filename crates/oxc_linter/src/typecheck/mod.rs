pub(self) mod client;
pub(self) mod protocol_error;
pub mod requests;
pub mod response;
pub(self) mod utils;

#[cfg(windows)]
pub(self) const EOL_LENGTH: usize = 2;
#[cfg(not(windows))]
pub(self) const EOL_LENGTH: usize = 1;

use std::{
    error::Error,
    process::{Command, Stdio},
};

pub use protocol_error::ProtocolError;

pub type TSServerClient =
    client::TSServerClient<std::process::ChildStdin, std::process::ChildStdout>;

pub fn start_typecheck_server(server_path: &str) -> Result<TSServerClient, Box<dyn Error>> {
    // Start the TSServer
    let tsserver = Command::new("node")
        .args([
            "--max-old-space-size=4096",
            server_path,
            "--disableAutomaticTypingAcquisition",
            "--suppressDiagnosticEvents",
        ])
        .env_clear()
        .env("NODE_ENV", "production")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        // .env("DEBUG", "true")
        // .stderr(Stdio::inherit())
        .spawn()?;

    let mut client = TSServerClient::try_from(tsserver)?;
    client.status()?;

    Ok(client)
}
