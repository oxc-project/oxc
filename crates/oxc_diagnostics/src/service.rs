use std::{
    borrow::Cow,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use cow_utils::CowUtils;
use percent_encoding::AsciiSet;
#[cfg(not(windows))]
use std::fs::canonicalize as strict_canonicalize;

use crate::{
    Error, NamedSource, OxcDiagnostic, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult},
};

pub type DiagnosticSender = mpsc::Sender<Vec<Error>>;
pub type DiagnosticReceiver = mpsc::Receiver<Vec<Error>>;

/// Listens for diagnostics sent over a [channel](DiagnosticSender) by some job, and
/// formats/reports them to the user.
///
/// [`DiagnosticService`] is designed to support multi-threaded jobs that may produce
/// reports. These jobs can send messages to the service over its multi-producer,
/// single-consumer channel.
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

        while let Ok(diagnostics) = self.receiver.recv() {
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

                let path = diagnostic
                    .source_code()
                    .and_then(|source| source.name())
                    .map(ToString::to_string);

                if let Some(err_str) = self.reporter.render_error(diagnostic) {
                    // Skip large output and print only once.
                    // Setting to 1200 because graphical output may contain ansi escape codes and other decorations.
                    if err_str.lines().any(|line| line.len() >= 1200) {
                        let mut diagnostic =
                            OxcDiagnostic::warn("File is too long to fit on the screen");
                        if let Some(path) = path {
                            diagnostic =
                                diagnostic.with_help(format!("{path} seems like a minified file"));
                        }

                        let minified_diagnostic = Error::new(diagnostic);

                        if let Some(err_str) = self.reporter.render_error(minified_diagnostic) {
                            Self::write_all_retry(writer, err_str.as_bytes()).unwrap();
                        }
                        is_minified = true;
                        continue;
                    }

                    Self::write_all_retry(writer, err_str.as_bytes()).unwrap();
                }
            }
        }

        let result = DiagnosticResult::new(
            warnings_count,
            errors_count,
            self.max_warnings_exceeded(warnings_count),
        );

        if let Some(finish_output) = self.reporter.finish(&result) {
            Self::write_all_retry(writer, finish_output.as_bytes()).unwrap();
        }

        Self::flush_retry(writer).unwrap();

        result
    }

    /// Write all bytes to the writer, retrying on `WouldBlock` and `Interrupted`.
    /// Silently returns `Ok(())` if the pipe is broken (consumer gone).
    fn write_all_retry(writer: &mut dyn Write, mut buf: &[u8]) -> std::io::Result<()> {
        while !buf.is_empty() {
            match writer.write(buf) {
                Ok(0) => {
                    // Writer cannot accept more bytes - treat as broken pipe
                    return Ok(());
                }
                Ok(n) => {
                    buf = &buf[n..];
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    // Pipe buffer is full - wait and retry
                    thread::sleep(Duration::from_millis(1));
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {
                    // Interrupted by signal - retry immediately
                }
                Err(e) if e.kind() == ErrorKind::BrokenPipe => {
                    // Consumer closed the pipe - silently ignore
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Flush the writer, retrying on `WouldBlock` and `Interrupted`.
    /// Silently returns `Ok(())` if the pipe is broken (consumer gone).
    fn flush_retry(writer: &mut dyn Write) -> std::io::Result<()> {
        loop {
            match writer.flush() {
                Ok(()) => return Ok(()),
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(1));
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) if e.kind() == ErrorKind::BrokenPipe => return Ok(()),
                Err(e) => return Err(e),
            }
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
    use std::io::{self, ErrorKind, Write};
    use std::path::PathBuf;

    use crate::service::{DiagnosticService, from_file_path};

    fn with_schema(path: &str) -> String {
        const EXPECTED_SCHEMA: &str = if cfg!(windows) { "file:///" } else { "file://" };
        format!("{EXPECTED_SCHEMA}{path}")
    }

    /// A mock writer that simulates WouldBlock errors for testing retry behavior.
    struct WouldBlockWriter {
        /// Number of WouldBlock errors to return before succeeding
        would_block_count: usize,
        /// Bytes written so far
        written: Vec<u8>,
        /// Number of write calls made
        write_calls: usize,
        /// Number of flush calls made
        flush_calls: usize,
        /// Number of WouldBlock errors to return on flush before succeeding
        flush_would_block_count: usize,
    }

    impl WouldBlockWriter {
        fn new(would_block_count: usize) -> Self {
            Self {
                would_block_count,
                written: Vec::new(),
                write_calls: 0,
                flush_calls: 0,
                flush_would_block_count: 0,
            }
        }

        fn with_flush_would_block(mut self, count: usize) -> Self {
            self.flush_would_block_count = count;
            self
        }
    }

    impl Write for WouldBlockWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.write_calls += 1;
            if self.would_block_count > 0 {
                self.would_block_count -= 1;
                return Err(io::Error::new(ErrorKind::WouldBlock, "would block"));
            }
            self.written.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flush_calls += 1;
            if self.flush_would_block_count > 0 {
                self.flush_would_block_count -= 1;
                return Err(io::Error::new(ErrorKind::WouldBlock, "would block"));
            }
            Ok(())
        }
    }

    /// A mock writer that returns BrokenPipe.
    struct BrokenPipeWriter;

    impl Write for BrokenPipeWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(ErrorKind::BrokenPipe, "broken pipe"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::new(ErrorKind::BrokenPipe, "broken pipe"))
        }
    }

    /// A mock writer that returns a custom error.
    struct ErrorWriter {
        kind: ErrorKind,
    }

    impl Write for ErrorWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(self.kind, "error"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::new(self.kind, "error"))
        }
    }

    /// A mock writer that simulates partial writes.
    struct PartialWriter {
        /// Max bytes to write per call
        max_per_write: usize,
        /// Bytes written so far
        written: Vec<u8>,
    }

    impl PartialWriter {
        fn new(max_per_write: usize) -> Self {
            Self { max_per_write, written: Vec::new() }
        }
    }

    impl Write for PartialWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let len = buf.len().min(self.max_per_write);
            self.written.extend_from_slice(&buf[..len]);
            Ok(len)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_write_all_retry_success() {
        let mut writer = WouldBlockWriter::new(0);
        let data = b"hello world";

        let result = DiagnosticService::write_all_retry(&mut writer, data);

        assert!(result.is_ok());
        assert_eq!(writer.written, data);
        assert_eq!(writer.write_calls, 1);
    }

    #[test]
    fn test_write_all_retry_with_would_block() {
        let mut writer = WouldBlockWriter::new(3);
        let data = b"hello world";

        let result = DiagnosticService::write_all_retry(&mut writer, data);

        assert!(result.is_ok());
        assert_eq!(writer.written, data);
        assert_eq!(writer.write_calls, 4); // 3 WouldBlock + 1 success
    }

    #[test]
    fn test_write_all_retry_broken_pipe() {
        let mut writer = BrokenPipeWriter;
        let data = b"hello world";

        let result = DiagnosticService::write_all_retry(&mut writer, data);

        // Broken pipe is silently ignored, returns Ok(())
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_all_retry_other_error() {
        let mut writer = ErrorWriter { kind: ErrorKind::PermissionDenied };
        let data = b"hello world";

        let result = DiagnosticService::write_all_retry(&mut writer, data);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::PermissionDenied);
    }

    #[test]
    fn test_write_all_retry_partial_writes() {
        let mut writer = PartialWriter::new(3);
        let data = b"hello world";

        let result = DiagnosticService::write_all_retry(&mut writer, data);

        assert!(result.is_ok());
        assert_eq!(writer.written, data);
    }

    #[test]
    fn test_write_all_retry_empty_buffer() {
        let mut writer = WouldBlockWriter::new(0);
        let data = b"";

        let result = DiagnosticService::write_all_retry(&mut writer, data);

        assert!(result.is_ok());
        assert!(writer.written.is_empty());
        assert_eq!(writer.write_calls, 0); // No write calls for empty buffer
    }

    #[test]
    fn test_flush_retry_success() {
        let mut writer = WouldBlockWriter::new(0);

        let result = DiagnosticService::flush_retry(&mut writer);

        assert!(result.is_ok());
        assert_eq!(writer.flush_calls, 1);
    }

    #[test]
    fn test_flush_retry_with_would_block() {
        let mut writer = WouldBlockWriter::new(0).with_flush_would_block(2);

        let result = DiagnosticService::flush_retry(&mut writer);

        assert!(result.is_ok());
        assert_eq!(writer.flush_calls, 3); // 2 WouldBlock + 1 success
    }

    #[test]
    fn test_flush_retry_broken_pipe() {
        let mut writer = BrokenPipeWriter;

        let result = DiagnosticService::flush_retry(&mut writer);

        // Broken pipe is silently ignored, returns Ok(())
        assert!(result.is_ok());
    }

    #[test]
    fn test_flush_retry_other_error() {
        let mut writer = ErrorWriter { kind: ErrorKind::PermissionDenied };

        let result = DiagnosticService::flush_retry(&mut writer);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::PermissionDenied);
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
            assert_eq!(uri.clone(), expected);
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
