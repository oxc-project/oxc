use miette::{Diagnostic, Severity};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MessageWithPath {
    pub file_path: String,
    pub messages: Vec<MessageDetail>,
}

#[derive(Debug, Serialize)]
pub struct MessageDetail {
    pub severity: i32,
    pub message: String,
    // TODO
    // rule_id: String,
    labels: Vec<MessageLabel>,
}
#[derive(Debug, Serialize)]
struct MessageLabel {
    label: String,
    span: MessageSpan,
}

#[derive(Debug, Serialize)]
struct MessageSpan {
    offset: usize,
    len: usize,
}

pub struct JSONReportHandler;

impl JSONReportHandler {
    /// # Panics
    pub fn render_report(diagnostic: &(dyn Diagnostic)) -> MessageDetail {
        let severity = diagnostic.severity();
        let is_warning = severity == Some(Severity::Warning);
        let labels = diagnostic.labels().unwrap();

        let mut labels_output = vec![];
        for label in labels {
            let message_label = MessageLabel {
                label: label.label().map_or_else(String::new, ToOwned::to_owned),
                span: MessageSpan { offset: label.offset(), len: label.len() },
            };
            labels_output.push(message_label);
        }

        MessageDetail {
            // rule_id: "todo".to_owned(),
            severity: if is_warning { 1 } else { 2 },
            labels: labels_output,
            message: diagnostic.to_string(),
        }
    }
}
