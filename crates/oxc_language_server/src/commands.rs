use log::error;
use serde::Deserialize;
use tower_lsp::{
    jsonrpc::{self, Error},
    lsp_types::{
        ApplyWorkspaceEditParams, TextEdit, Url, WorkspaceEdit, request::ApplyWorkspaceEdit,
    },
};

use crate::{Backend, capabilities::Capabilities};

pub const LSP_COMMANDS: [WorkspaceCommands; 1] = [WorkspaceCommands::FixAll(FixAllCommand)];

pub trait WorkspaceCommand {
    fn command_id(&self) -> String;
    fn available(&self, cap: Capabilities) -> bool;
    type CommandArgs<'a>: serde::Deserialize<'a>;
    async fn execute(
        &self,
        backend: &Backend,
        args: Self::CommandArgs<'_>,
    ) -> jsonrpc::Result<Option<serde_json::Value>>;
}

pub enum WorkspaceCommands {
    FixAll(FixAllCommand),
}

impl WorkspaceCommands {
    pub fn command_id(&self) -> String {
        match self {
            WorkspaceCommands::FixAll(c) => c.command_id(),
        }
    }
    pub fn available(&self, cap: Capabilities) -> bool {
        match self {
            WorkspaceCommands::FixAll(c) => c.available(cap),
        }
    }
    pub async fn execute(
        &self,
        backend: &Backend,
        args: Vec<serde_json::Value>,
    ) -> jsonrpc::Result<Option<serde_json::Value>> {
        match self {
            WorkspaceCommands::FixAll(c) => {
                let arg: Result<
                    <FixAllCommand as WorkspaceCommand>::CommandArgs<'_>,
                    serde_json::Error,
                > = serde_json::from_value(serde_json::Value::Array(args));
                if let Err(e) = arg {
                    error!("Invalid args passed to {:?}: {e}", c.command_id());
                    return Err(Error::invalid_request());
                }
                let arg = arg.unwrap();

                c.execute(backend, arg).await
            }
        }
    }
}

pub struct FixAllCommand;

#[derive(Deserialize)]
pub struct FixAllCommandArg {
    uri: String,
}

impl WorkspaceCommand for FixAllCommand {
    fn command_id(&self) -> String {
        "oxc.fixAll".into()
    }
    fn available(&self, cap: Capabilities) -> bool {
        cap.workspace_apply_edit
    }
    type CommandArgs<'a> = (FixAllCommandArg,);

    async fn execute(
        &self,
        backend: &Backend,
        args: Self::CommandArgs<'_>,
    ) -> jsonrpc::Result<Option<serde_json::Value>> {
        let url = Url::parse(&args.0.uri);
        if let Err(e) = url {
            error!("Invalid uri passed to {:?}: {e}", self.command_id());
            return Err(Error::invalid_request());
        }
        let url = url.unwrap();

        let mut edits = vec![];
        if let Some(value) = backend.diagnostics_report_map.pin_owned().get(&url.to_string()) {
            for report in value {
                if let Some(fixed) = &report.fixed_content {
                    edits.push(TextEdit { range: fixed.range, new_text: fixed.code.clone() });
                }
            }
            let _ = backend
                .client
                .send_request::<ApplyWorkspaceEdit>(ApplyWorkspaceEditParams {
                    label: Some(match edits.len() {
                        1 => "Oxlint: 1 fix applied".into(),
                        n => format!("Oxlint: {n} fixes applied"),
                    }),
                    edit: WorkspaceEdit {
                        #[expect(clippy::disallowed_types)]
                        changes: Some(std::collections::HashMap::from([(url, edits)])),
                        ..WorkspaceEdit::default()
                    },
                })
                .await;
        }

        Ok(None)
    }
}
