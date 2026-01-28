//! [Reporters](DiagnosticReporter) for rendering and writing diagnostics.

use std::fmt::Write;

use miette::{Diagnostic, GraphicalReportHandler, SourceCode, SourceSpan};
use similar::{ChangeTag, TextDiff};

use crate::{Error, OxcDiagnostic, Patch, Severity, show_fix_diff};

/// Reporters are responsible for rendering diagnostics to some format and writing them to some
/// form of output stream.
///
/// Reporters get used by [`DiagnosticService`](crate::service::DiagnosticService) when they
/// receive diagnostics.
///
/// ## Example
/// ```
/// use oxc_diagnostics::{DiagnosticReporter, Error, Severity};
///
/// #[derive(Default)]
/// pub struct BufferedReporter;
///
/// impl DiagnosticReporter for BufferedReporter {
///     // render the finished output, some reporters will store the errors in memory
///     // to output all diagnostics at the end
///     fn finish(&mut self) -> Option<String> {
///         None
///     }
///
///     // render diagnostics to a simple Apache-like log format
///     fn render_error(&mut self, error: Error) -> Option<String> {
///         let level = match error.severity().unwrap_or_default() {
///             Severity::Error => "ERROR",
///             Severity::Warning => "WARN",
///             Severity::Advice => "INFO",
///         };
///         let rendered = format!("[{level}]: {error}");
///
///         Some(rendered)
///     }
/// }
/// ```
pub trait DiagnosticReporter {
    /// Lifecycle hook that gets called when no more diagnostics will be reported.
    ///
    /// Some reporters (e.g. `JSONReporter`) store all diagnostics in memory, then write them
    /// all at once.
    ///
    /// While this method _should_ only ever be called a single time, this is not a guarantee
    /// upheld in Oxc's API. Do not rely on this behavior.
    fn finish(&mut self, result: &DiagnosticResult) -> Option<String>;

    /// Render a diagnostic into this reporter's desired format. For example, a JSONLinesReporter
    /// might return a stringified JSON object on a single line. Returns [`None`] to skip reporting
    /// of this diagnostic.
    ///
    /// Reporters should use this method to write diagnostics to their output stream.
    fn render_error(&mut self, error: Error) -> Option<String>;
}

/// DiagnosticResult will be submitted to the Reporter when the [`DiagnosticService`](crate::service::DiagnosticService)
/// is finished receiving all files
#[derive(Default, Debug)]
pub struct DiagnosticResult {
    /// Total number of warnings received
    warnings_count: usize,

    /// Total number of errors received
    errors_count: usize,

    /// Did the threshold for warnings exceeded the max_warnings?
    /// ToDo: We giving the input from outside, let the owner calculate the result
    max_warnings_exceeded: bool,
}

impl DiagnosticResult {
    pub fn new(warnings_count: usize, errors_count: usize, max_warnings_exceeded: bool) -> Self {
        Self { warnings_count, errors_count, max_warnings_exceeded }
    }

    /// Get the number of warning-level diagnostics received.
    pub fn warnings_count(&self) -> usize {
        self.warnings_count
    }

    /// Get the number of error-level diagnostics received.
    pub fn errors_count(&self) -> usize {
        self.errors_count
    }

    /// Did the threshold for warnings exceeded the max_warnings?
    pub fn max_warnings_exceeded(&self) -> bool {
        self.max_warnings_exceeded
    }
}

#[derive(Debug)]
pub struct Info {
    pub start: InfoPosition,
    pub end: InfoPosition,
    pub filename: String,
    pub message: String,
    pub severity: Severity,
    pub rule_id: Option<String>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct InfoPosition {
    pub line: usize,
    pub column: usize,
}

impl Info {
    pub fn new(diagnostic: &Error) -> Self {
        let mut start = InfoPosition { line: 0, column: 0 };
        let mut end = InfoPosition { line: 0, column: 0 };
        let mut filename = String::new();
        let mut message = String::new();
        let mut severity = Severity::Warning;
        let rule_id = diagnostic.code().map(|code| code.to_string());

        if let Some(mut labels) = diagnostic.labels()
            && let Some(source) = diagnostic.source_code()
            && let Some(label) = labels.next()
            && let Ok(span_content) = source.read_span(label.inner(), 0, 0)
        {
            start.line = span_content.line() + 1;
            start.column = span_content.column() + 1;

            let end_offset = label.inner().offset() + label.inner().len();

            if let Ok(span_content) = source.read_span(&SourceSpan::from((end_offset, 0)), 0, 0) {
                end.line = span_content.line() + 1;
                end.column = span_content.column() + 1;
            }

            if let Some(name) = span_content.name() {
                filename = name.to_string();
            }
            if matches!(diagnostic.severity(), Some(Severity::Error)) {
                severity = Severity::Error;
            }

            message = diagnostic.to_string();
            // Our messages usually are in format `eslint(rule): message`.
            // Trim off before the colon.
            if let Some((_, msg)) = message.split_once(':') {
                // Equivalent to `message = msg.trim().to_string()`, but operates in place
                let msg = msg.trim();
                let start = msg.as_ptr() as usize - message.as_str().as_ptr() as usize;
                message.truncate(start + msg.len());
                message.replace_range(..start, "");
            }
        }

        Self { start, end, filename, message, severity, rule_id }
    }
}

/// A graphical report handler that can also render inline fix diffs.
///
/// Wraps miette's [`GraphicalReportHandler`] and adds the ability to render
/// patches/diffs when fixes are available for a diagnostic.
///
/// To enable diff rendering, set the `OXC_DIAGNOSTIC_SHOW_FIX_DIFF` environment
/// variable to `1` or `true`.
///
/// # Example
/// ```text
/// help: you might have meant to use let instead of var
///    ╭╴
///  1 - var answer = 42;
///  1 + let answer = 42;
///    ╰╴
/// ```
pub struct GraphicalReportHandlerWithDiff {
    handler: GraphicalReportHandler,
    /// Whether to force showing fix diffs regardless of the environment variable.
    force_show_diff: bool,
    /// Whether to use colors in the diff output.
    use_colors: bool,
}

impl GraphicalReportHandlerWithDiff {
    /// Create a new handler with default settings (colors enabled).
    #[must_use]
    pub fn new() -> Self {
        Self { handler: GraphicalReportHandler::new(), force_show_diff: false, use_colors: true }
    }

    /// Create a new handler wrapping the provided [`GraphicalReportHandler`].
    #[must_use]
    pub fn with_handler(handler: GraphicalReportHandler) -> Self {
        Self { handler, force_show_diff: false, use_colors: true }
    }

    /// Force showing fix diffs regardless of the environment variable.
    ///
    /// Useful for tests that want to always show diffs in snapshots.
    #[must_use]
    pub fn always_show_diff(mut self) -> Self {
        self.force_show_diff = true;
        self
    }

    /// Disable colors in the diff output.
    ///
    /// Useful for tests where ANSI codes would make snapshots harder to read.
    #[must_use]
    pub fn without_colors(mut self) -> Self {
        self.use_colors = false;
        self
    }

    fn should_show_diff(&self) -> bool {
        self.force_show_diff || show_fix_diff()
    }

    /// Render a diagnostic report to a string.
    ///
    /// This renders the standard miette diagnostic first, then appends inline
    /// diffs for any patches if `OXC_DIAGNOSTIC_SHOW_FIX_DIFF` is enabled.
    #[expect(clippy::missing_errors_doc)]
    pub fn render_report(
        &self,
        output: &mut String,
        diagnostic: &(dyn Diagnostic + Send + Sync),
    ) -> std::fmt::Result {
        // Render the standard diagnostic
        self.handler.render_report(output, diagnostic)?;
        Ok(())
    }

    /// Render an [`OxcDiagnostic`] with optional fix diffs.
    ///
    /// This renders the standard miette diagnostic first, then appends inline
    /// diffs for any patches if `OXC_DIAGNOSTIC_SHOW_FIX_DIFF` is enabled.
    ///
    /// The `source` parameter provides the source code context for rendering.
    #[expect(clippy::missing_errors_doc)]
    pub fn render_oxc_diagnostic<S: SourceCode + 'static>(
        &self,
        output: &mut String,
        diagnostic: &OxcDiagnostic,
        source: &S,
    ) -> std::fmt::Result {
        // Create a temporary struct that implements Diagnostic with the source
        struct DiagnosticWithSource<'a, S: SourceCode> {
            diagnostic: &'a OxcDiagnostic,
            source: &'a S,
        }

        impl<S: SourceCode> std::fmt::Display for DiagnosticWithSource<'_, S> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.diagnostic.fmt(f)
            }
        }

        impl<S: SourceCode> std::fmt::Debug for DiagnosticWithSource<'_, S> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.diagnostic.fmt(f)
            }
        }

        impl<S: SourceCode> std::error::Error for DiagnosticWithSource<'_, S> {}

        impl<S: SourceCode> Diagnostic for DiagnosticWithSource<'_, S> {
            fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
                self.diagnostic.code()
            }
            fn severity(&self) -> Option<miette::Severity> {
                self.diagnostic.severity()
            }
            fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
                self.diagnostic.help()
            }
            fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
                self.diagnostic.url()
            }
            fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
                self.diagnostic.labels()
            }
            fn source_code(&self) -> Option<&dyn SourceCode> {
                Some(self.source)
            }
            fn note<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
                self.diagnostic.note()
            }
        }

        let wrapper = DiagnosticWithSource { diagnostic, source };

        // Render the standard diagnostic with source
        self.handler.render_report(output, &wrapper)?;

        // If fix diff is enabled and we have patches, render them
        if self.should_show_diff()
            && let Some(patches) = &diagnostic.patches
        {
            self.render_patches(output, patches, source)?;
        }

        Ok(())
    }

    /// Render patches as inline diffs.
    fn render_patches(
        &self,
        output: &mut String,
        patches: &[Vec<Patch>],
        source: &dyn SourceCode,
    ) -> std::fmt::Result {
        if patches.is_empty() {
            return Ok(());
        }

        // Filter out empty fixes
        let non_empty_patches: Vec<_> = patches.iter().filter(|p| !p.is_empty()).collect();
        if non_empty_patches.is_empty() {
            return Ok(());
        }

        let num_fixes = non_empty_patches.len();

        for (fix_idx, fix_patches) in non_empty_patches.iter().enumerate() {
            // For each alternative fix, compute the combined diff
            if let Some((original, replacement, start_line)) =
                Self::compute_combined_diff(fix_patches, source)
            {
                // Render the diff with box-drawing characters
                self.render_diff_block(
                    output,
                    &original,
                    &replacement,
                    start_line,
                    fix_idx == 0,
                    fix_idx == num_fixes - 1,
                )?;
            }
        }

        Ok(())
    }

    /// Compute the combined before/after text for a set of patches.
    ///
    /// Returns (original_text, replacement_text, start_line_number).
    /// Expands the span to cover full lines so diffs show complete lines.
    fn compute_combined_diff(
        patches: &[Patch],
        source: &dyn SourceCode,
    ) -> Option<(String, String, usize)> {
        if patches.is_empty() {
            return None;
        }

        // Sort patches by span start
        let mut sorted_patches: Vec<_> = patches.iter().collect();
        sorted_patches.sort_by_key(|p| p.span.offset());

        // Find the full range covered by all patches
        let first_patch = sorted_patches.first()?;
        let last_patch = sorted_patches.last()?;

        let start_offset = first_patch.span.offset();
        let end_offset = sorted_patches
            .iter()
            .map(|p| p.span.offset() + p.span.len())
            .max()
            .unwrap_or(last_patch.span.offset() + last_patch.span.len());

        // Get the patch span content first to find line boundaries
        let patch_span = SourceSpan::new(start_offset.into(), end_offset - start_offset);
        let span_contents = source.read_span(&patch_span, 0, 0).ok()?;
        let start_line = span_contents.line(); // 0-indexed

        // Now read the full lines by using context lines
        // We need to expand to include the full first and last lines
        // Use read_span with context to get surrounding content
        let full_span_contents = source.read_span(&patch_span, 1, 1).ok()?;

        // Get the actual span that was read (including context)
        let context_span = full_span_contents.span();
        let context_start = context_span.offset();
        let context_len = context_span.len();

        // Read from the context start to find line boundaries
        let full_source_span = SourceSpan::new(context_start.into(), context_len);
        let full_contents = source.read_span(&full_source_span, 0, 0).ok()?;
        let full_bytes = full_contents.data();
        let full_text = std::str::from_utf8(full_bytes).ok()?;

        // Find the line start within the context
        let offset_in_context = start_offset - context_start;
        let line_start_in_context =
            full_text[..offset_in_context].rfind('\n').map(|pos| pos + 1).unwrap_or(0);

        // Find the line end within the context
        let end_in_context = end_offset - context_start;
        let line_end_in_context = full_text[end_in_context..]
            .find('\n')
            .map(|pos| end_in_context + pos + 1)
            .unwrap_or(full_text.len());

        // Extract the full lines
        let original_text = &full_text[line_start_in_context..line_end_in_context];
        let line_start_offset = context_start + line_start_in_context;

        // Apply patches to compute the replacement
        let replacement = Self::apply_patches_to_text(original_text, patches, line_start_offset);

        Some((original_text.to_string(), replacement, start_line + 1)) // 1-indexed
    }

    /// Apply patches to the original text to produce the replacement.
    fn apply_patches_to_text(original: &str, patches: &[Patch], base_offset: usize) -> String {
        let mut sorted_patches: Vec<_> = patches.iter().collect();
        sorted_patches.sort_by_key(|p| p.span.offset());

        let mut result = String::with_capacity(original.len());
        let mut last_end = 0usize;

        for patch in sorted_patches {
            let patch_start = patch.span.offset().saturating_sub(base_offset);
            let patch_end = patch_start + patch.span.len();

            // Add text between last patch and this one
            if patch_start > last_end && patch_start <= original.len() {
                result.push_str(&original[last_end..patch_start]);
            }

            // Add the replacement
            result.push_str(&patch.replacement);

            last_end = patch_end.min(original.len());
        }

        // Add remaining text after last patch
        if last_end < original.len() {
            result.push_str(&original[last_end..]);
        }

        result
    }

    /// Render a single diff block with box-drawing characters and optional colors.
    fn render_diff_block(
        &self,
        output: &mut String,
        original: &str,
        replacement: &str,
        start_line: usize,
        is_first: bool,
        is_last: bool,
    ) -> std::fmt::Result {
        let diff = TextDiff::from_lines(original, replacement);
        let mut has_changes = false;

        // Check if there are any actual changes
        for change in diff.iter_all_changes() {
            if change.tag() != ChangeTag::Equal {
                has_changes = true;
                break;
            }
        }

        if !has_changes {
            return Ok(());
        }

        // Calculate the line number width for alignment
        let max_line = start_line + original.lines().count().max(replacement.lines().count());
        let line_num_width = max_line.to_string().len();

        // Indentation to match miette's source code display
        let indent = "  ";

        // Opening bracket
        let bracket_start = if is_first { "╭" } else { "├" };
        writeln!(output, "{indent}{:width$} {bracket_start}╴", "", width = line_num_width)?;

        let mut old_line_num = start_line;
        let mut new_line_num = start_line;

        // ANSI color codes
        const RED: &str = "\x1b[31m";
        const GREEN: &str = "\x1b[32m";
        const RESET: &str = "\x1b[0m";

        for change in diff.iter_all_changes() {
            let (sign, line_num, color_start, color_end) = match change.tag() {
                ChangeTag::Delete => {
                    let num = old_line_num;
                    old_line_num += 1;
                    if self.use_colors { ("-", num, RED, RESET) } else { ("-", num, "", "") }
                }
                ChangeTag::Insert => {
                    let num = new_line_num;
                    new_line_num += 1;
                    if self.use_colors { ("+", num, GREEN, RESET) } else { ("+", num, "", "") }
                }
                ChangeTag::Equal => {
                    old_line_num += 1;
                    new_line_num += 1;
                    continue; // Skip equal lines in the diff output
                }
            };

            let line_content = change.as_str().unwrap_or("");
            // Remove trailing newline for cleaner output
            let line_content = line_content.trim_end_matches('\n').trim_end_matches('\r');

            writeln!(
                output,
                "{indent}{line_num:>line_num_width$} {color_start}{sign} {line_content}{color_end}",
            )?;
        }

        // Closing bracket
        let bracket_end = if is_last { "╰" } else { "│" };
        writeln!(output, "{indent}{:line_num_width$} {bracket_end}╴", "")?;

        Ok(())
    }
}

impl Default for GraphicalReportHandlerWithDiff {
    fn default() -> Self {
        Self::new()
    }
}
