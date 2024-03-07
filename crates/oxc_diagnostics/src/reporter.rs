use std::io::{BufWriter, Stdout, Write};

use crate::{
    miette::{Error, JSONReportHandler},
    GraphicalReportHandler,
};

#[allow(clippy::large_enum_variant)] // Lerge size is fine because this is a singleton
#[derive(Debug)]
#[non_exhaustive]
pub enum DiagnosticReporter {
    // stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
    // See `https://github.com/rust-lang/rust/issues/60673`.
    Graphical { handler: GraphicalReportHandler, writer: BufWriter<Stdout> },
    Json { diagnostics: Vec<Error> },
}

impl DiagnosticReporter {
    pub fn new_graphical() -> Self {
        Self::Graphical {
            handler: GraphicalReportHandler::new(),
            writer: BufWriter::new(std::io::stdout()),
        }
    }

    pub fn new_json() -> Self {
        Self::Json { diagnostics: vec![] }
    }

    pub fn finish(&mut self) {
        match self {
            Self::Graphical { writer, .. } => {
                writer.flush().unwrap();
            }
            // NOTE: this output does not conform to eslint json format yet
            // https://eslint.org/docs/latest/use/formatters/#json
            Self::Json { diagnostics } => {
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
        }
    }

    pub fn render_diagnostics(&mut self, s: &[u8]) {
        match self {
            Self::Graphical { writer, .. } => {
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
        }
    }
}
