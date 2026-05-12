//! Multi-sink diagnostic dispatcher: fans each diagnostic out to N
//! `(formatter, reporter, writer)` sinks, enabling oxlint's dual-output
//! (`--format` to stdout + `--output-file-format` to a file) in one run.

use std::{
    fs::{self, File},
    io::{BufWriter, ErrorKind, Write},
    path::Path,
    sync::Arc,
};

#[cfg(test)]
use oxc_diagnostics::DiagnosticSender;
use oxc_diagnostics::{
    DiagnosticReceiver, Error, OxcDiagnostic, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult},
};

use crate::output_formatter::{LintCommandInfo, OutputFormatter};

pub struct OutputSink<'a> {
    formatter: OutputFormatter,
    reporter: Box<dyn DiagnosticReporter>,
    writer: DispatcherWriter<'a>,
    /// When true, this sink skips per-diagnostic rendering (its `finish` summary is still
    /// emitted). Set on the stdout sink when `--silent` is used so the file sink can keep
    /// recording the full report.
    silent: bool,
}

impl<'a> OutputSink<'a> {
    pub fn new(formatter: OutputFormatter, writer: DispatcherWriter<'a>) -> Self {
        let reporter = formatter.get_diagnostic_reporter();
        Self { formatter, reporter, writer, silent: false }
    }

    #[must_use]
    pub fn with_silent(mut self, yes: bool) -> Self {
        self.silent = yes;
        self
    }
}

pub enum DispatcherWriter<'a> {
    Borrowed(&'a mut dyn Write),
    #[expect(dead_code, reason = "kept for callers that need owned writers")]
    Owned(Box<dyn Write + 'a>),
}

impl Write for DispatcherWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Borrowed(w) => w.write(buf),
            Self::Owned(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Borrowed(w) => w.flush(),
            Self::Owned(w) => w.flush(),
        }
    }
}

pub fn open_output_file(path: &Path) -> std::io::Result<BufWriter<File>> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    Ok(BufWriter::new(file))
}

pub struct MultiSinkDispatcher<'a> {
    sinks: Vec<OutputSink<'a>>,
    receiver: DiagnosticReceiver,
    quiet: bool,
    max_warnings: Option<usize>,
}

impl<'a> MultiSinkDispatcher<'a> {
    #[cfg(test)]
    pub fn new(sinks: Vec<OutputSink<'a>>) -> (Self, DiagnosticSender) {
        let (sender, receiver) = std::sync::mpsc::channel();
        (Self::from_receiver(sinks, receiver), sender)
    }

    pub fn from_receiver(sinks: Vec<OutputSink<'a>>, receiver: DiagnosticReceiver) -> Self {
        debug_assert!(!sinks.is_empty(), "MultiSinkDispatcher requires at least one sink");
        Self { sinks, receiver, quiet: false, max_warnings: None }
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

    fn max_warnings_exceeded(&self, warnings_count: usize) -> bool {
        self.max_warnings.is_some_and(|max_warnings| warnings_count > max_warnings)
    }

    pub fn run(&mut self) -> DiagnosticResult {
        let mut warnings_count: usize = 0;
        let mut errors_count: usize = 0;
        // Tracked per-sink so a JSON file sink keeps emitting full output while a human-readable
        // stdout sink stops on the same minified file.
        let mut sink_minified: Vec<bool> = vec![false; self.sinks.len()];
        let sink_supports_fallback: Vec<bool> =
            self.sinks.iter_mut().map(|s| s.reporter.supports_minified_file_fallback()).collect();

        while let Ok(diagnostics) = self.receiver.recv() {
            sink_minified.fill(false);

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
                    // `--quiet` still counts warnings (so `--max-warnings` triggers) but skips
                    // rendering them.
                    else if self.quiet {
                        continue;
                    }
                }

                if self.sinks.iter().all(|s| s.silent) {
                    continue;
                }

                let path = diagnostic
                    .source_code()
                    .and_then(|source| source.name())
                    .map(ToString::to_string);

                let shared = Arc::new(diagnostic);

                for (idx, sink) in self.sinks.iter_mut().enumerate() {
                    if sink.silent || sink_minified[idx] {
                        continue;
                    }

                    let Some(rendered) = sink.reporter.render_error(Arc::clone(&shared)) else {
                        continue;
                    };

                    if sink_supports_fallback[idx]
                        && rendered.lines().any(|line| line.len() >= 1200)
                    {
                        let mut warning =
                            OxcDiagnostic::warn("File is too long to fit on the screen");
                        if let Some(path) = path.as_ref() {
                            warning =
                                warning.with_help(format!("{path} seems like a minified file"));
                        }
                        let warning = Arc::new(Error::new(warning));
                        if let Some(fallback) = sink.reporter.render_error(warning) {
                            write_all(&mut sink.writer, fallback.as_bytes());
                        }
                        sink_minified[idx] = true;
                        continue;
                    }

                    write_all(&mut sink.writer, rendered.as_bytes());
                }
            }
        }

        let result = DiagnosticResult::new(
            warnings_count,
            errors_count,
            self.max_warnings_exceeded(warnings_count),
        );

        for sink in &mut self.sinks {
            if let Some(finish_output) = sink.reporter.finish(&result) {
                write_all(&mut sink.writer, finish_output.as_bytes());
            }
            flush_writer(&mut sink.writer);
        }

        result
    }

    pub fn write_lint_command_info(&mut self, info: &LintCommandInfo) {
        for sink in &mut self.sinks {
            if let Some(end) = sink.formatter.lint_command_info(info) {
                write_all(&mut sink.writer, end.as_bytes());
                flush_writer(&mut sink.writer);
            }
        }
    }
}

fn write_all(writer: &mut DispatcherWriter<'_>, bytes: &[u8]) {
    writer.write_all(bytes).or_else(check_for_writer_error).unwrap();
}

fn flush_writer(writer: &mut DispatcherWriter<'_>) {
    writer.flush().or_else(check_for_writer_error).unwrap();
}

fn check_for_writer_error(error: std::io::Error) -> Result<(), std::io::Error> {
    if matches!(
        error.kind(),
        ErrorKind::Interrupted | ErrorKind::BrokenPipe | ErrorKind::WouldBlock
    ) {
        Ok(())
    } else {
        Err(error)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use oxc_diagnostics::{Error, NamedSource, OxcDiagnostic};
    use oxc_span::Span;

    use oxc_linter::OxlintSuppressionFileAction;

    use crate::output_formatter::{LintCommandInfo, OutputFormat, OutputFormatter};

    use super::{DispatcherWriter, MultiSinkDispatcher, OutputSink};

    fn make_diagnostic(message: &str) -> Error {
        OxcDiagnostic::warn(message.to_string())
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file://test.ts", "debugger;"))
    }

    fn run_dispatch(
        formats: &[OutputFormat],
        diagnostics: Vec<Vec<Error>>,
        configure: impl for<'a> FnOnce(MultiSinkDispatcher<'a>) -> MultiSinkDispatcher<'a>,
        emit_command_info: bool,
    ) -> (oxc_diagnostics::reporter::DiagnosticResult, Vec<String>) {
        run_dispatch_with_sinks(formats, diagnostics, configure, emit_command_info, &[])
    }

    fn run_dispatch_with_sinks(
        formats: &[OutputFormat],
        diagnostics: Vec<Vec<Error>>,
        configure: impl for<'a> FnOnce(MultiSinkDispatcher<'a>) -> MultiSinkDispatcher<'a>,
        emit_command_info: bool,
        silent_flags: &[bool],
    ) -> (oxc_diagnostics::reporter::DiagnosticResult, Vec<String>) {
        let mut buffers: Vec<Vec<u8>> =
            std::iter::repeat_with(Vec::new).take(formats.len()).collect();
        let result = {
            let sinks: Vec<OutputSink<'_>> = buffers
                .iter_mut()
                .zip(formats.iter())
                .enumerate()
                .map(|(idx, (buf, fmt))| {
                    let silent = silent_flags.get(idx).copied().unwrap_or(false);
                    OutputSink::new(OutputFormatter::new(*fmt), DispatcherWriter::Borrowed(buf))
                        .with_silent(silent)
                })
                .collect();
            let (dispatcher, sender) = MultiSinkDispatcher::new(sinks);
            let mut dispatcher = configure(dispatcher);
            for batch in diagnostics {
                sender.send(batch).unwrap();
            }
            drop(sender);
            let result = dispatcher.run();
            if emit_command_info {
                dispatcher.write_lint_command_info(&LintCommandInfo {
                    number_of_files: 1,
                    number_of_rules: Some(1),
                    threads_count: 1,
                    start_time: Duration::from_millis(1),
                    oxlint_suppression_file_action: OxlintSuppressionFileAction::None,
                });
            }
            result
        };
        let outputs = buffers.into_iter().map(|b| String::from_utf8(b).unwrap()).collect();
        (result, outputs)
    }

    #[test]
    fn single_sink_runs_and_finishes() {
        let (result, outputs) =
            run_dispatch(&[OutputFormat::Json], vec![vec![make_diagnostic("err")]], |d| d, true);
        assert_eq!(result.warnings_count(), 1);
        assert_eq!(result.errors_count(), 0);
        assert!(outputs[0].contains("\"diagnostics\""));
        assert!(outputs[0].contains("\"number_of_files\""));
    }

    #[test]
    fn dual_sink_fans_out_per_format() {
        let (result, outputs) = run_dispatch(
            &[OutputFormat::Default, OutputFormat::Json],
            vec![vec![make_diagnostic("dual sink message")]],
            |d| d,
            true,
        );
        assert_eq!(result.warnings_count(), 1);

        let stdout_str = &outputs[0];
        let file_str = &outputs[1];
        assert!(stdout_str.contains("Found") && stdout_str.contains("warning"));
        assert!(file_str.contains("\"diagnostics\"") && file_str.contains("dual sink message"));
        assert!(!file_str.contains("Found 1 warning"));
    }

    #[test]
    fn quiet_skips_warnings_but_counts_them() {
        let (result, _outputs) = run_dispatch(
            &[OutputFormat::Default],
            vec![vec![make_diagnostic("warn one")]],
            |d| d.with_quiet(true).with_max_warnings(Some(0)),
            false,
        );
        assert_eq!(result.warnings_count(), 1);
        assert!(result.max_warnings_exceeded());
    }

    #[test]
    fn silent_sink_skips_render_but_still_counts_and_finishes() {
        let (result, outputs) = run_dispatch_with_sinks(
            &[OutputFormat::Default],
            vec![vec![make_diagnostic("hidden")]],
            |d| d,
            false,
            &[true],
        );
        assert_eq!(result.warnings_count(), 1);
        assert!(outputs[0].contains("Found 1 warning"));
        assert!(!outputs[0].contains("hidden"));
    }

    #[test]
    fn silent_is_per_sink_so_file_sink_still_renders() {
        let (result, outputs) = run_dispatch_with_sinks(
            &[OutputFormat::Default, OutputFormat::Json],
            vec![vec![make_diagnostic("dual silent message")]],
            |d| d,
            true,
            &[true, false],
        );
        assert_eq!(result.warnings_count(), 1);
        let stdout_str = &outputs[0];
        let file_str = &outputs[1];
        assert!(!stdout_str.contains("dual silent message"));
        assert!(stdout_str.contains("Found 1 warning"));
        assert!(file_str.contains("dual silent message"));
        assert!(file_str.contains("\"diagnostics\""));
    }

    #[test]
    fn lint_command_info_only_writes_when_formatter_returns_some() {
        // `Unix` uses the default `lint_command_info` returning None.
        let (_result, outputs_without) = run_dispatch(&[OutputFormat::Unix], vec![], |d| d, false);
        let (_result, outputs_with) = run_dispatch(&[OutputFormat::Unix], vec![], |d| d, true);
        assert_eq!(outputs_without[0], outputs_with[0]);
    }
}
