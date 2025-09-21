use std::{
    borrow::Cow,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
};

use cow_utils::CowUtils;
use percent_encoding::AsciiSet;
#[cfg(not(windows))]
use std::fs::canonicalize as strict_canonicalize;

use crate::{
    Error, NamedSource, OxcDiagnostic, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult},
};

pub type DiagnosticTuple = (PathBuf, Vec<Error>);
pub type DiagnosticSender = mpsc::Sender<DiagnosticTuple>;
pub type DiagnosticReceiver = mpsc::Receiver<DiagnosticTuple>;

/// Listens for diagnostics sent over a [channel](DiagnosticSender) by some job, and
/// formats/reports them to the user.
///
/// [`DiagnosticService`] is designed to support multi-threaded jobs that may produce
/// reports. These jobs can send [messages](DiagnosticTuple) to the service over its
/// multi-producer, single-consumer channel.
///
/// # Example
/// ```rust
/// use std::{path::PathBuf, thread};
/// use oxc_diagnostics::{Error, OxcDiagnostic, DiagnosticService, GraphicalReportHandler};
///
/// // Create a service with a graphical reporter
/// let (mut service, sender) = DiagnosticService::new(Box::new(GraphicalReportHandler::new()));
///
/// // Spawn a thread that does work and reports diagnostics
/// thread::spawn(move || {
///     sender.send((
///         PathBuf::from("file.txt"),
///         vec![Error::new(OxcDiagnostic::error("Something went wrong"))],
///     ));
///
///     // The service will stop listening when all senders are dropped.
///     // No explicit termination signal is needed.
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

    receiver: DiagnosticReceiver,
}

impl DiagnosticService {
    /// Create a new [`DiagnosticService`] that will render and report diagnostics using the
    /// provided [`DiagnosticReporter`].
    pub fn new(reporter: Box<dyn DiagnosticReporter>) -> (Self, DiagnosticSender) {
        let (sender, receiver) = mpsc::channel();
        (Self { reporter, quiet: false, silent: false, max_warnings: None, receiver }, sender)
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

    /// Check if the max warning threshold, as set by
    /// [`with_max_warnings`](DiagnosticService::with_max_warnings), has been exceeded.
    fn max_warnings_exceeded(&self, warnings_count: usize) -> bool {
        self.max_warnings.is_some_and(|max_warnings| warnings_count > max_warnings)
    }

    /// Wrap [diagnostics] with the source code and path, converting them into [Error]s.
    ///
    /// [diagnostics]: OxcDiagnostic
    pub fn wrap_diagnostics<C: AsRef<Path>, P: AsRef<Path>>(
        cwd: C,
        path: P,
        source_text: &str,
        diagnostics: Vec<OxcDiagnostic>,
    ) -> Vec<Error> {
        // TODO: This causes snapshots to fail when running tests through a JetBrains terminal.
        let is_jetbrains =
            std::env::var("TERMINAL_EMULATOR").is_ok_and(|x| x.eq("JetBrains-JediTerm"));

        let path_ref = path.as_ref();
        let path_display = if is_jetbrains { from_file_path(path_ref) } else { None }
            .unwrap_or_else(|| {
                let relative_path =
                    path_ref.strip_prefix(cwd).unwrap_or(path_ref).to_string_lossy();
                let normalized_path = relative_path.cow_replace('\\', "/");
                normalized_path.to_string()
            });

        let source = Arc::new(NamedSource::new(path_display, source_text.to_owned()));
        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.with_source_code(Arc::clone(&source)))
            .collect()
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

        while let Ok((path, diagnostics)) = self.receiver.recv() {
            let mut is_minified = false;
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

                if self.silent || is_minified {
                    continue;
                }

                if let Some(err_str) = self.reporter.render_error(diagnostic) {
                    // Skip large output and print only once.
                    // Setting to 1200 because graphical output may contain ansi escape codes and other decorations.
                    if err_str.lines().any(|line| line.len() >= 1200) {
                        let minified_diagnostic = Error::new(
                            OxcDiagnostic::warn("File is too long to fit on the screen").with_help(
                                format!("{} seems like a minified file", path.display()),
                            ),
                        );

                        if let Some(err_str) = self.reporter.render_error(minified_diagnostic) {
                            writer
                                .write_all(err_str.as_bytes())
                                .or_else(Self::check_for_writer_error)
                                .unwrap();
                        }
                        is_minified = true;
                        continue;
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

// The following from_file_path and strict_canonicalize implementations are from tower-lsp-community/tower-lsp-server
// available under the MIT License or Apache 2.0 License.
//
// Copyright (c) 2023 Eyal Kalderon
// https://github.com/tower-lsp-community/tower-lsp-server/blob/85506ddcbd108c514438e0b62e0eb858c812adcf/src/uri_ext.rs

const ASCII_SET: AsciiSet =
    // RFC3986 allows only alphanumeric characters, `-`, `.`, `_`, and `~` in the path.
    percent_encoding::NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'.')
        .remove(b'_')
        .remove(b'~')
        // we do not want path separators to be percent-encoded
        .remove(b'/');

fn from_file_path<A: AsRef<Path>>(path: A) -> Option<String> {
    let path = path.as_ref();

    let fragment = if path.is_absolute() {
        Cow::Borrowed(path)
    } else {
        match strict_canonicalize(path) {
            Ok(path) => Cow::Owned(path),
            Err(_) => return None,
        }
    };

    if cfg!(windows) {
        // we want to write a triple-slash path for Windows paths
        // it's a shorthand for `file://localhost/C:/Windows` with the `localhost` omitted.
        let mut components = fragment.components();
        let drive = components.next();

        if let Some(drive) = drive {
            Some(format!(
                "file:///{}{}",
                drive.as_os_str().to_string_lossy().cow_replace('\\', "/"),
                // Skip encoding ":" in the drive "C:/".
                percent_encoding::utf8_percent_encode(
                    &components.collect::<PathBuf>().to_string_lossy().cow_replace('\\', "/"),
                    &ASCII_SET
                )
            ))
        } else {
            Some(format!(
                "file:///{}",
                percent_encoding::utf8_percent_encode(
                    &components.collect::<PathBuf>().to_string_lossy().cow_replace('\\', "/"),
                    &ASCII_SET
                )
            ))
        }
    } else {
        Some(format!(
            "file://{}",
            percent_encoding::utf8_percent_encode(&fragment.to_string_lossy(), &ASCII_SET)
        ))
    }
}

/// On Windows, rewrites the wide path prefix `\\?\C:` to `C:`
/// Source: <https://stackoverflow.com/a/70970317>
#[inline]
#[cfg(windows)]
fn strict_canonicalize<P: AsRef<Path>>(path: P) -> std::io::Result<PathBuf> {
    use std::io;

    fn impl_(path: &Path) -> std::io::Result<PathBuf> {
        let head = path.components().next().ok_or(io::Error::other("empty path"))?;
        let disk_;
        let head = if let std::path::Component::Prefix(prefix) = head {
            if let std::path::Prefix::VerbatimDisk(disk) = prefix.kind() {
                disk_ = format!("{}:", disk as char);
                Path::new(&disk_)
                    .components()
                    .next()
                    .ok_or(io::Error::other("failed to parse disk component"))?
            } else {
                head
            }
        } else {
            head
        };
        Ok(std::iter::once(head).chain(path.components().skip(1)).collect())
    }

    let canon = std::fs::canonicalize(path)?;
    impl_(&canon)
}

#[cfg(test)]
mod tests {
    use crate::service::from_file_path;
    use std::path::PathBuf;

    fn with_schema(path: &str) -> String {
        const EXPECTED_SCHEMA: &str = if cfg!(windows) { "file:///" } else { "file://" };
        format!("{EXPECTED_SCHEMA}{path}")
    }

    #[test]
    #[cfg(windows)]
    fn test_idempotent_canonicalization() {
        use crate::service::strict_canonicalize;
        use std::path::Path;

        let lhs = strict_canonicalize(Path::new(".")).unwrap();
        let rhs = strict_canonicalize(&lhs).unwrap();
        assert_eq!(lhs, rhs);
    }

    #[test]
    #[cfg(unix)]
    fn test_path_to_uri() {
        let paths = [
            PathBuf::from("/some/path/to/file.txt"),
            PathBuf::from("/some/path/to/file with spaces.txt"),
            PathBuf::from("/some/path/[[...rest]]/file.txt"),
            PathBuf::from("/some/path/to/файл.txt"),
            PathBuf::from("/some/path/to/文件.txt"),
        ];

        let expected = [
            with_schema("/some/path/to/file.txt"),
            with_schema("/some/path/to/file%20with%20spaces.txt"),
            with_schema("/some/path/%5B%5B...rest%5D%5D/file.txt"),
            with_schema("/some/path/to/%D1%84%D0%B0%D0%B9%D0%BB.txt"),
            with_schema("/some/path/to/%E6%96%87%E4%BB%B6.txt"),
        ];

        for (path, expected) in paths.iter().zip(expected) {
            let uri = from_file_path(path).unwrap();
            assert_eq!(uri.to_string(), expected);
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_path_to_uri_windows() {
        let paths = [
            PathBuf::from("C:\\some\\path\\to\\file.txt"),
            PathBuf::from("C:\\some\\path\\to\\file with spaces.txt"),
            PathBuf::from("C:\\some\\path\\[[...rest]]\\file.txt"),
            PathBuf::from("C:\\some\\path\\to\\файл.txt"),
            PathBuf::from("C:\\some\\path\\to\\文件.txt"),
        ];

        let expected = [
            with_schema("C:/some/path/to/file.txt"),
            with_schema("C:/some/path/to/file%20with%20spaces.txt"),
            with_schema("C:/some/path/%5B%5B...rest%5D%5D/file.txt"),
            with_schema("C:/some/path/to/%D1%84%D0%B0%D0%B9%D0%BB.txt"),
            with_schema("C:/some/path/to/%E6%96%87%E4%BB%B6.txt"),
        ];

        for (path, expected) in paths.iter().zip(expected) {
            let uri = from_file_path(path).unwrap();
            assert_eq!(uri, expected);
        }
    }
}
