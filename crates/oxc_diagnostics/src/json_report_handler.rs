use std::{collections::HashMap, fmt, io, path::Path};

use miette::Severity;
use serde::{Deserialize, Serialize};

use crate::Report;
pub struct JsonReportHandler {
    pub reports: HashMap<String, JsonReport>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct JsonReport {
    file_path: String,
    error_count: usize,
    warning_count: usize,
    messages: Vec<JsonMessage>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct JsonMessage {
    severity: Option<Severity>,
    message: Option<String>,
    line: usize,
    column: usize,
    offset: usize,
    end_line: usize,
    end_column: usize,
}

impl JsonReportHandler {
    pub(crate) fn new() -> Self {
        Self { reports: HashMap::default() }
    }
}

impl JsonReportHandler {
    pub(crate) fn handle_diagnostics(&mut self, path: &Path, diagnostics: &Vec<Report>) {
        let path_string = path.to_string_lossy().to_string();
        let report = self.reports.entry(path_string).or_default();
        report.file_path = path.to_string_lossy().to_string();
        let mut message = vec![]; // let
        let (warning_count, error_count) = get_severity_count(diagnostics);
        for diagnostic in diagnostics {
            if let Some(source) = diagnostic.source_code() {
                if let Some(labels) = diagnostic.labels() {
                    let mut labels = labels.collect::<Vec<_>>();
                    labels.sort_unstable_by_key(|l| l.inner().offset());
                    if !labels.is_empty() {
                        for label in &labels {
                            let span_contents = source.read_span(label.inner(), 0, 0);
                            if let Ok(span_contents) = span_contents {
                                let line = span_contents.line();
                                let column = span_contents.column();
                                let mut offset = span_contents.span().offset();
                                let mut end_line = line;
                                let mut end_column = column + span_contents.span().len();
                                let context = std::str::from_utf8(span_contents.data())
                                    .expect("Bad utf8 detected");
                                let mut iter = context.chars().peekable();
                                let label_name = label.label();
                                while let Some(char) = iter.next() {
                                    offset += char.len_utf8();
                                    match char {
                                        '\r' => {
                                            if iter.next_if_eq(&'\n').is_some() {
                                                offset += 1;
                                                end_line += 1;
                                                end_column = 0;
                                            } else {
                                                end_column += 1;
                                            }
                                        }
                                        '\n' => {
                                            end_line += 1;
                                            end_column = 0;
                                        }
                                        _ => {
                                            end_column += 1;
                                        }
                                    }
                                }

                                message.push(JsonMessage {
                                    severity: diagnostic.severity(),
                                    message: label_name.map(ToString::to_string),
                                    line,
                                    column,
                                    offset,
                                    end_line,
                                    end_column,
                                });
                            }
                        }
                    }
                }
            }
        }
        report.messages = message;
        report.error_count = error_count;
        report.warning_count = warning_count;
    }
    pub(crate) fn write(&self, f: &mut impl io::Write) -> fmt::Result {
        let values: Vec<&JsonReport> = self.reports.values().collect();
        serde_json::to_writer(f, &values).map_err(|_| fmt::Error)?;
        Ok(())
    }
}

fn get_severity_count(diagnostics: &Vec<Report>) -> (usize, usize) {
    let mut warnings_count = 0;
    let mut errors_count = 0;
    for diagnostic in diagnostics {
        let severity = diagnostic.severity();
        let is_warning = severity == Some(Severity::Warning);
        let is_error = severity.is_none() || severity == Some(Severity::Error);

        if is_warning || is_error {
            if is_warning {
                warnings_count += 1;
            }
            if is_error {
                errors_count += 1;
            }
        }
    }
    (warnings_count, errors_count)
}
