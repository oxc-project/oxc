use std::{
    cell::Cell,
    io::{BufWriter, Write},
    ops::ControlFlow,
    path::{Path, PathBuf},
    sync::mpsc,
    sync::Arc,
};

use miette::Error;

use crate::json_report_handler::JsonReportHandler;
use crate::{miette::NamedSource, GraphicalReportHandler, MinifiedFileError, Severity};

pub type DiagnosticTuple = (PathBuf, Vec<Error>);
pub type DiagnosticSender = mpsc::Sender<Option<DiagnosticTuple>>;
pub type DiagnosticReceiver = mpsc::Receiver<Option<DiagnosticTuple>>;

pub struct DiagnosticService {
    /// Disable reporting on warnings, only errors are reported
    quiet: bool,

    /// Specify a warning threshold,
    /// which can be used to force exit with an error status if there are too many warning-level rule violations in your project
    max_warnings: Option<usize>,

    /// Total number of warnings received
    warnings_count: Cell<usize>,

    /// Total number of errors received
    errors_count: Cell<usize>,

    sender: DiagnosticSender,
    receiver: DiagnosticReceiver,
    output_type: OutputType,
}

#[derive(Debug, Default)]
pub enum OutputType {
    Json,
    #[default]
    Stylish,
}

impl Default for DiagnosticService {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            quiet: false,
            max_warnings: None,
            warnings_count: Cell::new(0),
            errors_count: Cell::new(0),
            sender,
            receiver,
            output_type: OutputType::default(),
        }
    }
}

impl DiagnosticService {
    #[must_use]
    pub fn with_quiet(mut self, yes: bool) -> Self {
        self.quiet = yes;
        self
    }

    #[must_use]
    pub fn with_max_warnings(mut self, max_warnings: Option<usize>) -> Self {
        self.max_warnings = max_warnings;
        self
    }

    #[must_use]
    pub fn with_output_type(mut self, format: Option<String>) -> Self {
        match format {
            Some(f) => {
                let f = f.to_lowercase();
                match &*f {
                    "json" => {
                        self.output_type = OutputType::Json;
                    }
                    "stylish" => {
                        self.output_type = OutputType::Stylish;
                    }
                    _ => {}
                }
            }
            None => {
                self.output_type = OutputType::Stylish;
            }
        }
        self
    }

    pub fn sender(&self) -> &DiagnosticSender {
        &self.sender
    }

    pub fn warnings_count(&self) -> usize {
        self.warnings_count.get()
    }

    pub fn errors_count(&self) -> usize {
        self.errors_count.get()
    }

    pub fn max_warnings_exceeded(&self) -> bool {
        self.max_warnings.map_or(false, |max_warnings| self.warnings_count.get() > max_warnings)
    }

    pub fn wrap_diagnostics(
        path: &Path,
        source_text: &str,
        diagnostics: Vec<Error>,
    ) -> (PathBuf, Vec<Error>) {
        let source = Arc::new(NamedSource::new(path.to_string_lossy(), source_text.to_owned()));
        let diagnostics = diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.with_source_code(Arc::clone(&source)))
            .collect();
        (path.to_path_buf(), diagnostics)
    }

    /// # Panics
    ///
    /// * When the writer fails to write
    pub fn run(&self) {
        match self.output_type {
            OutputType::Json => self.run_json(),
            OutputType::Stylish => self.run_stylish(),
        }
    }
    fn run_json(&self) {
        let mut buf_writer = BufWriter::new(std::io::stdout());
        let mut json_handler = JsonReportHandler::new();

        while let Ok(Some((path, diagnostics))) = self.receiver.recv() {
            json_handler.handle_diagnostics(&path, &diagnostics);
            for diagnostic in diagnostics {
                if self.handle_severity(&diagnostic) == ControlFlow::Break(()) {
                    continue;
                }
            }
        }
        json_handler.write(buf_writer.by_ref()).unwrap();
        buf_writer.flush().unwrap();
    }

    fn handle_severity(&self, diagnostic: &Error) -> ControlFlow<()> {
        let severity = diagnostic.severity();
        let is_warning = severity == Some(Severity::Warning);
        let is_error = severity.is_none() || severity == Some(Severity::Error);
        if is_warning || is_error {
            if is_warning {
                let warnings_count = self.warnings_count() + 1;
                self.warnings_count.set(warnings_count);
            }
            if is_error {
                let errors_count = self.errors_count() + 1;
                self.errors_count.set(errors_count);
            }
            // The --quiet flag follows ESLint's --quiet behavior as documented here: https://eslint.org/docs/latest/use/command-line-interface#--quiet
            // Note that it does not disable ALL diagnostics, only Warning diagnostics
            else if self.quiet {
                return ControlFlow::Break(());
            }

            if let Some(max_warnings) = self.max_warnings {
                if self.warnings_count() > max_warnings {
                    return ControlFlow::Break(());
                }
            }
        }
        ControlFlow::Continue(())
    }

    fn run_stylish(&self) {
        let mut buf_writer = BufWriter::new(std::io::stdout());
        let handler = GraphicalReportHandler::new();
        while let Ok(Some((path, diagnostics))) = self.receiver.recv() {
            let mut output = String::new();
            for diagnostic in diagnostics {
                if self.handle_severity(&diagnostic) == ControlFlow::Break(()) {
                    continue;
                }

                let mut err = String::new();
                handler.render_report(&mut err, diagnostic.as_ref()).unwrap();
                // Skip large output and print only once
                if err.lines().any(|line| line.len() >= 400) {
                    let minified_diagnostic = Error::new(MinifiedFileError(path.clone()));
                    err = format!("{minified_diagnostic:?}");
                    output = err;
                    break;
                }
                output.push_str(&err);
            }
            buf_writer.write_all(output.as_bytes()).unwrap();
        }
        buf_writer.flush().unwrap();
    }
}
