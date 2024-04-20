use std::io::{BufWriter, Stdout, Write};

use crate::{
    miette::{Error, JSONReportHandler},
    GraphicalReportHandler, Severity,
};

/// stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
/// See `https://github.com/rust-lang/rust/issues/60673`.
fn writer() -> BufWriter<Stdout> {
    BufWriter::new(std::io::stdout())
}

#[allow(clippy::large_enum_variant)] // Lerge size is fine because this is a singleton
#[derive(Debug)]
#[non_exhaustive]
pub enum DiagnosticReporter {
    Graphical { handler: GraphicalReportHandler, writer: BufWriter<Stdout> },
    Json { diagnostics: Vec<Error> },
    Unix { total: usize, writer: BufWriter<Stdout> },
}

impl DiagnosticReporter {
    pub fn new_graphical() -> Self {
        Self::Graphical { handler: GraphicalReportHandler::new(), writer: writer() }
    }

    pub fn new_json() -> Self {
        Self::Json { diagnostics: vec![] }
    }

    pub fn new_unix() -> Self {
        Self::Unix { total: 0, writer: writer() }
    }

    pub fn finish(&mut self) {
        match self {
            Self::Graphical { writer, .. } => {
                writer.flush().unwrap();
            }
            // NOTE: this output does not conform to eslint json format yet
            // https://eslint.org/docs/latest/use/formatters/#json
            Self::Json { diagnostics } => {
                format_json(diagnostics);
            }
            Self::Unix { total, writer } => {
                if *total > 0 {
                    let line = format!("\n{total} problem{}\n", if *total > 1 { "s" } else { "" });
                    writer.write_all(line.as_bytes()).unwrap();
                }
                writer.flush().unwrap();
            }
        }
    }

    pub fn render_diagnostics(&mut self, s: &[u8]) {
        match self {
            Self::Graphical { writer, .. } | Self::Unix { writer, .. } => {
                writer.write_all(s).unwrap();
            }
            Self::Json { .. } => {}
        }
    }

    pub fn render_error(&mut self, error: Error) -> Option<String> {
        match self {
            Self::Graphical { handler, .. } => {
                let mut output = String::new();
                handler.render_report(&mut output, error.as_ref()).unwrap();
                Some(output)
            }
            Self::Json { diagnostics } => {
                diagnostics.push(error);
                None
            }
            Self::Unix { total: count, .. } => {
                *count += 1;
                Some(format_unix(&error))
            }
        }
    }
}

/// <https://github.com/fregante/eslint-formatters/tree/main/packages/eslint-formatter-json>
fn format_json(diagnostics: &mut Vec<Error>) {
    let handler = JSONReportHandler::new();
    let messages = diagnostics
        .drain(..)
        .map(|error| {
            let mut output = String::from("\t");
            handler.render_report(&mut output, error.as_ref()).unwrap();
            output
        })
        .collect::<Vec<_>>()
        .join(",\n");
    println!("[\n{messages}\n]");
}

/// <https://github.com/fregante/eslint-formatters/tree/main/packages/eslint-formatter-unix>
fn format_unix(diagnostic: &Error) -> String {
    let mut line = 0;
    let mut column = 0;
    let mut filename = String::new();
    let mut message = String::new();
    let mut severity = "Warning";
    let mut rule_id = String::new();
    if let Some(mut labels) = diagnostic.labels() {
        if let Some(source) = diagnostic.source_code() {
            if let Some(label) = labels.next() {
                if let Ok(span_content) = source.read_span(label.inner(), 0, 0) {
                    line = span_content.line() + 1;
                    column = span_content.column() + 1;
                    if let Some(name) = span_content.name() {
                        filename = name.to_string();
                    };
                    if matches!(diagnostic.severity(), Some(Severity::Error)) {
                        severity = "Warning";
                    }
                    let msg = diagnostic.to_string();
                    // Our messages usually comes with `eslint(rule): message`
                    (rule_id, message) = msg.split_once(':').map_or_else(
                        || (String::new(), msg.to_string()),
                        |(id, msg)| (id.to_string(), msg.trim().to_string()),
                    );
                }
            }
        }
    }
    format!("{filename}:{line}:{column}: {message} [{severity}/{rule_id}]\n")
}
