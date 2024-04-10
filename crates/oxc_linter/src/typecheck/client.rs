use std::process::{Child, ChildStdin, ChildStdout};

use super::{requests::*, utils::read_message, ProtocolError};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};

pub struct TSServerClient<W: std::io::Write, R: std::io::Read> {
    server: Child,
    seq: usize,
    command_stream: W,
    result_stream: R,
    running: bool,
}

impl<W: std::io::Write, R: std::io::Read> TSServerClient<W, R> {
    pub fn status(&mut self) -> Result<String, ProtocolError> {
        self.send_command("status", None)?;

        let response = read_message(&mut self.result_stream)?;
        Ok(response)
    }

    pub fn exit(&mut self) -> Result<(), ProtocolError> {
        if !self.running {
            return Ok(());
        }

        let _ = self.send_command("exit", None);

        self.running = false;
        self.server.wait()?;

        Ok(())
    }

    pub fn open(&mut self, opts: OpenRequest<'_>) -> Result<(), ProtocolError> {
        let args = serde_json::to_string(&opts)?;
        self.send_command("open", Some(args.as_str()))?;
        Ok(())
    }

    pub fn close(&mut self, opts: FileRequest<'_>) -> Result<(), ProtocolError> {
        let args = serde_json::to_string(&opts)?;
        self.send_command("close", Some(args.as_str()))?;
        Ok(())
    }

    pub fn get_node(&mut self, opts: NodeRequest<'_>) -> Result<String, ProtocolError> {
        let args = serde_json::to_string(&opts)?;
        self.send_command("getNode", Some(args.as_str()))?;

        let response = read_message(&mut self.result_stream)?;
        Ok(response)
    }

    pub fn is_promise_array(&mut self, opts: LocationRequest<'_>) -> Result<String, ProtocolError> {
        let args = serde_json::to_string(&opts)?;
        self.send_command("noFloatingPromises::isPromiseArray", Some(args.as_str()))?;

        let response = read_message(&mut self.result_stream)?;
        Ok(response)
    }

    pub fn is_promise_like(&mut self, opts: LocationRequest<'_>) -> Result<String, ProtocolError> {
        let args = serde_json::to_string(&opts)?;
        self.send_command("noFloatingPromises::isPromiseLike", Some(args.as_str()))?;

        let response = read_message(&mut self.result_stream)?;
        Ok(response)
    }

    fn send_command(&mut self, command: &str, args: Option<&str>) -> Result<(), std::io::Error> {
        self.seq += 1;
        let seq = self.seq;
        let args_str = args.map(|x| format!(r#","arguments":{x}"#)).unwrap_or("".to_string());
        let msg =
            format!("{{\"seq\":{seq},\"type\":\"request\",\"command\":\"{command}\"{args_str}}}\n");

        self.command_stream.write_all(msg.as_bytes())
    }
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic()]
pub enum FromChildError {
    #[error("child stdout must be piped")]
    MissingStdoutStream,
    #[error("child stdin must be piped")]
    MissingStdinStream,
}

impl TryFrom<Child> for TSServerClient<ChildStdin, ChildStdout> {
    type Error = FromChildError;

    fn try_from(mut value: Child) -> Result<Self, Self::Error> {
        let command_stream = value.stdin.take().ok_or(FromChildError::MissingStdinStream)?;
        let result_stream = value.stdout.take().ok_or(FromChildError::MissingStdoutStream)?;

        Ok(Self { server: value, seq: 0, command_stream, result_stream, running: true })
    }
}

impl<W: std::io::Write, R: std::io::Read> Drop for TSServerClient<W, R> {
    fn drop(&mut self) {
        if self.running {
            let _ = self.exit();
        }
    }
}
