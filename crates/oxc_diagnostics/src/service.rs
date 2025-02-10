use std::{
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::{mpsc, Arc},
};

use cow_utils::CowUtils;

use crate::{
    reporter::{DiagnosticReporter, DiagnosticResult},
    Error, NamedSource, OxcDiagnostic, Severity,
};

pub type DiagnosticTuple = (PathBuf, Vec<Error>);
pub type DiagnosticSender = mpsc::Sender<Option<DiagnosticTuple>>;
pub type DiagnosticReceiver = mpsc::Receiver<Option<DiagnosticTuple>>;

/// Listens for diagnostics sent over a [channel](DiagnosticSender) by some job, and
/// formats/reports them to the user.
///
/// [`DiagnosticService`] is designed to support multi-threaded jobs that may produce
/// reports. These jobs can send [messages](DiagnosticTuple) to the service over its
/// multi-producer, single-consumer [channel](DiagnosticService::sender).
///
/// # Example
/// ```rust
/// use std::thread;
/// use oxc_diagnostics::{Error, OxcDiagnostic, DiagnosticService};
///
/// // By default, services will pretty-print diagnostics to the console
/// let mut service = DiagnosticService::default();
/// // Get a clone of the sender to send diagnostics to the service
/// let mut sender = service.sender().clone();
///
/// // Spawn a thread that does work and reports diagnostics
/// thread::spawn(move || {
///     sender.send(Some((
///         PathBuf::from("file.txt"),
///         vec![Error::new(OxcDiagnostic::error("Something went wrong"))],
///     )));
///
///     // Send `None` to have the service stop listening for messages.
///     // If you don't ever send `None`, the service will poll forever.
///     sender.send(None);
/// });
///
/// // Listen for and process messages
/// service.run()
/// ```
pub struct DiagnosticService {
    reporter: Box<dyn DiagnosticReporter>,

    /// Disable reporting on warnings, only errors are reported
    quiet: bool,

    /// Do not display any diagnostics
    silent: bool,

    /// Specify a warning threshold,
    /// which can be used to force exit with an error status if there are too many warning-level rule violations in your project
    max_warnings: Option<usize>,

    sender: DiagnosticSender,
    receiver: DiagnosticReceiver,
}

impl DiagnosticService {
    /// Create a new [`DiagnosticService`] that will render and report diagnostics using the
    /// provided [`DiagnosticReporter`].
    pub fn new(reporter: Box<dyn DiagnosticReporter>) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { reporter, quiet: false, silent: false, max_warnings: None, sender, receiver }
    }

    /// Set to `true` to only report errors and ignore warnings.
    ///
    /// Use [`with_silent`](DiagnosticService::with_silent) to disable reporting entirely.
    ///
    /// Default: `false`
    #[must_use]
    pub fn with_quiet(mut self, yes: bool) -> Self {
        self.quiet = yes;
        self
    }

    /// Set to `true` to disable reporting entirely.
    ///
    /// Use [`with_quiet`](DiagnosticService::with_quiet) to only disable reporting on warnings.
    ///
    /// Default is `false`.
    #[must_use]
    pub fn with_silent(mut self, yes: bool) -> Self {
        self.silent = yes;
        self
    }

    /// Specify a warning threshold, which can be used to force exit with an error status if there
    /// are too many warning-level rule violations in your project. Errors do not count towards the
    /// warning limit.
    ///
    /// Use [`DiagnosticResult`](DiagnosticResult::max_warnings_exceeded) to check if too
    /// many warnings have been received.
    ///
    /// Default: [`None`]
    #[must_use]
    pub fn with_max_warnings(mut self, max_warnings: Option<usize>) -> Self {
        self.max_warnings = max_warnings;
        self
    }

    /// Channel for sending [diagnostic messages] to the service.
    ///
    /// The service will only start processing diagnostics after [`run`](DiagnosticService::run)
    /// has been called.
    ///
    /// [diagnostics]: DiagnosticTuple
    pub fn sender(&self) -> &DiagnosticSender {
        &self.sender
    }

    /// Check if the max warning threshold, as set by
    /// [`with_max_warnings`](DiagnosticService::with_max_warnings), has been exceeded.
    fn max_warnings_exceeded(&self, warnings_count: usize) -> bool {
        self.max_warnings.is_some_and(|max_warnings| warnings_count > max_warnings)
    }

    /// Wrap [diagnostics] with the source code and path, converting them into [Error]s.
    ///
    /// [diagnostics]: OxcDiagnostic
    pub fn wrap_diagnostics<P: AsRef<Path>>(
        path: P,
        source_text: &str,
        diagnostics: Vec<OxcDiagnostic>,
    ) -> (PathBuf, Vec<Error>) {
        let path = path.as_ref();
        let path_display = path.to_string_lossy();
        // replace windows \ path separator with posix style one
        // reflects what eslint is outputting
        let path_display = path_display.cow_replace('\\', "/");

        let source = Arc::new(NamedSource::new(path_display, source_text.to_owned()));
        let diagnostics = diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.with_source_code(Arc::clone(&source)))
            .collect();
        (path.to_path_buf(), diagnostics)
    }

    /// # Panics
    ///
    /// * When the writer fails to write
    ///
    /// ToDo:
    /// We are passing [`DiagnosticResult`] to the [`DiagnosticReporter`] already
    /// currently for the GraphicalReporter there is another extra output,
    /// which does some more things. This is the reason why we are returning it.
    /// Let's check at first it we can easily change for the default output before removing this return.
    pub fn run(&mut self, writer: &mut dyn Write) -> DiagnosticResult {
        let mut warnings_count: usize = 0;
        let mut errors_count: usize = 0;

        while let Ok(Some((path, diagnostics))) = self.receiver.recv() {
            for diagnostic in diagnostics {
                let severity = diagnostic.severity();
                let is_warning = severity == Some(Severity::Warning);
                let is_error = severity == Some(Severity::Error) || severity.is_none();
                if is_warning || is_error {
                    if is_warning {
                        warnings_count += 1;
                    }
                    if is_error {
                        errors_count += 1;
                    }
                    // The --quiet flag follows ESLint's --quiet behavior as documented here: https://eslint.org/docs/latest/use/command-line-interface#--quiet
                    // Note that it does not disable ALL diagnostics, only Warning diagnostics
                    else if self.quiet {
                        continue;
                    }
                }

                if self.silent {
                    continue;
                }

                if let Some(err_str) = self.reporter.render_error(diagnostic) {
                    // Skip large output and print only once.
                    // Setting to 1200 because graphical output may contain ansi escape codes and other decorations.
                    if err_str.lines().any(|line| line.len() >= 1200) {
                        let minified_diagnostic = Error::new(
                            OxcDiagnostic::warn("File is too long to fit on the screen")
                                .with_help(format!("{path:?} seems like a minified file")),
                        );

                        if let Some(err_str) = self.reporter.render_error(minified_diagnostic) {
                            writer
                                .write_all(err_str.as_bytes())
                                .or_else(Self::check_for_writer_error)
                                .unwrap();
                        }
                        break;
                    }

                    writer
                        .write_all(err_str.as_bytes())
                        .or_else(Self::check_for_writer_error)
                        .unwrap();
                }
            }
        }

        let result = DiagnosticResult::new(
            warnings_count,
            errors_count,
            self.max_warnings_exceeded(warnings_count),
        );

        if let Some(finish_output) = self.reporter.finish(&result) {
            writer
                .write_all(finish_output.as_bytes())
                .or_else(Self::check_for_writer_error)
                .unwrap();
        }

        writer.flush().or_else(Self::check_for_writer_error).unwrap();

        result
    }

    fn check_for_writer_error(error: std::io::Error) -> Result<(), std::io::Error> {
        // Do not panic when the process is killed (e.g. piping into `less`).
        if matches!(error.kind(), ErrorKind::Interrupted | ErrorKind::BrokenPipe) {
            Ok(())
        } else {
            Err(error)
        }
    }
}
