use std::{
    cell::Cell,
    path::{Path, PathBuf},
    sync::{mpsc, Arc},
};

use crate::{
    miette::NamedSource, reporter::DiagnosticReporter, Error, MinifiedFileError, Severity,
};

pub type DiagnosticTuple = (PathBuf, Vec<Error>);
pub type DiagnosticSender = mpsc::Sender<Option<DiagnosticTuple>>;
pub type DiagnosticReceiver = mpsc::Receiver<Option<DiagnosticTuple>>;

pub struct DiagnosticService {
    reporter: DiagnosticReporter,

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
}

impl Default for DiagnosticService {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            reporter: DiagnosticReporter::new_graphical(),
            quiet: false,
            max_warnings: None,
            warnings_count: Cell::new(0),
            errors_count: Cell::new(0),
            sender,
            receiver,
        }
    }
}

impl DiagnosticService {
    pub fn set_json_reporter(&mut self) {
        self.reporter = DiagnosticReporter::new_json();
    }

    pub fn is_graphical_output(&self) -> bool {
        matches!(self.reporter, DiagnosticReporter::Graphical { .. })
    }

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
    pub fn run(&mut self) {
        while let Ok(Some((path, diagnostics))) = self.receiver.recv() {
            let mut output = String::new();
            for diagnostic in diagnostics {
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
                        continue;
                    }
                }

                if let Some(mut err_str) = self.reporter.render_error(diagnostic) {
                    // Skip large output and print only once
                    if err_str.lines().any(|line| line.len() >= 400) {
                        let minified_diagnostic = Error::new(MinifiedFileError(path.clone()));
                        err_str = format!("{minified_diagnostic:?}");
                        output = err_str;
                        break;
                    }
                    output.push_str(&err_str);
                }
            }
            self.reporter.render_diagnostics(output.as_bytes());
        }

        self.reporter.finish();
    }
}
