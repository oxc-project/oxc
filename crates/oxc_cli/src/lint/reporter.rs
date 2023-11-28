use std::{
    io::{BufWriter, Write},
    path::PathBuf,
};

use miette::Report;
use oxc_diagnostics::{
    DiagnosticService, Error, GraphicalReportHandler, JSONReportHandler, MessageWithPath,
    MinifiedFileError,
};

use crate::FormatForFormatter;

pub enum ReportHandler {
    DefaultHandler,
    JsonHandler,
}

impl ReportHandler {
    pub fn handle(&self, diagnostic_service: &DiagnosticService) {
        match self {
            Self::DefaultHandler => {
                let mut buf_writer = BufWriter::new(std::io::stdout());
                let handler = GraphicalReportHandler::new();

                let process_diagnostic =
                    |diagnostics_option: Option<&Vec<Report>>, file_path: String| {
                        match diagnostics_option {
                            Some(diagnostics) => {
                                let mut output = String::new();
                                for diagnostic in diagnostics {
                                    let mut err = String::new();
                                    handler.render_report(&mut err, diagnostic.as_ref()).unwrap();
                                    // Skip large output and print only once
                                    if err.lines().any(|line| line.len() >= 400) {
                                        let minified_diagnostic =
                                            Error::new(MinifiedFileError(PathBuf::from(file_path)));
                                        err = format!("{minified_diagnostic:?}");
                                        output = err;
                                        break;
                                    }
                                    output.push_str(&err);
                                }

                                buf_writer.write_all(output.as_bytes()).unwrap();
                            }
                            None => {
                                // print out graph
                                buf_writer.flush().unwrap();
                            }
                        }
                    };

                diagnostic_service.report_while_recv(process_diagnostic);
            }
            Self::JsonHandler => {
                let mut buf_writer = BufWriter::new(std::io::stdout());
                let mut messages_with_path = vec![];

                let process_diagnostic =
                    |diagnostics_option: Option<&Vec<Report>>, file_path: String| {
                        if let Some(diagnostics) = diagnostics_option {
                            let mut messages = vec![];
                            for diagnostic in diagnostics {
                                let message = JSONReportHandler::render_report(diagnostic.as_ref());
                                messages.push(message);
                            }
                            messages_with_path.push(MessageWithPath { file_path, messages });
                        } else {
                            // print out json
                            let json_str = serde_json::to_string(&messages_with_path).unwrap();
                            diagnostic_service.set_output(json_str.clone());
                            buf_writer.write_all(json_str.as_bytes()).unwrap();
                            buf_writer.flush().unwrap();
                        }
                    };

                diagnostic_service.report_while_recv(process_diagnostic);
            }
        }
    }
}

pub fn get_handler(format: &FormatForFormatter) -> ReportHandler {
    match format {
        FormatForFormatter::Default => ReportHandler::DefaultHandler,
        FormatForFormatter::Json => ReportHandler::JsonHandler,
    }
}
