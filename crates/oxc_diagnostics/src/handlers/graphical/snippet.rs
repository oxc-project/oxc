//! Source-snippet layout.
//!
//! [`render_snippets`](GraphicalReportHandler::render_snippets) reads every
//! label's span in a single forward scan and merges overlapping spans into
//! contexts.
//! [`render_context`](GraphicalReportHandler::render_context) then draws one
//! context: the `[file:line:col]` header, each source line (via
//! [`render_line_text`](GraphicalReportHandler::render_line_text)), and the
//! gutters/underlines/labels delegated to the sibling modules.

use std::{borrow::Cow, cmp::max, fmt};

use smallvec::SmallVec;

use super::{
    handler::GraphicalReportHandler,
    label::write_repeated_char,
    span::{FancySpan, LabelRenderMode},
};
use crate::{
    Diagnostic, LabeledSpan,
    handlers::theme::DiagnosticColorize,
    source_impls::{SpanContents, SpanScanner},
};

const MIN_COLLAPSED_LINES: usize = 3;

impl GraphicalReportHandler {
    pub(super) fn render_snippets(
        &self,
        f: &mut impl fmt::Write,
        diagnostic: &dyn Diagnostic,
        scanner: Option<&mut SpanScanner<'_>>,
        source_name: Option<&str>,
    ) -> fmt::Result {
        let Some(scanner) = scanner else { return Ok(()) };
        let labels = diagnostic.labels();
        if labels.is_empty() {
            return Ok(());
        }

        let mut read = |span| scanner.read_span(span);

        if let [label] = labels {
            let contents = read(label.span()).ok_or(fmt::Error)?;
            return self.render_context(f, label, &contents, &[label], source_name);
        }

        let mut inline_labels = [&labels[0], &labels[1]];
        let mut heap_labels = Vec::new();
        let labels = if labels.len() == 2 {
            inline_labels.sort_unstable_by_key(|label| label.offset());
            inline_labels.as_slice()
        } else {
            heap_labels.extend(labels);
            heap_labels.sort_unstable_by_key(|label| label.offset());
            heap_labels.as_slice()
        };
        let mut contexts: Vec<(Cow<'_, LabeledSpan>, _)> = Vec::with_capacity(labels.len());
        for &right in labels {
            let right_conts = read(right.span()).ok_or(fmt::Error)?;

            let Some((left, left_contents)) = contexts.last() else {
                contexts.push((Cow::Borrowed(right), right_conts));
                continue;
            };

            if left_contents.line() + left_contents.line_count() >= right_conts.line() {
                // Merge overlapping snippets into one context.
                let left_end = left.offset() + left.len();
                let right_end = right.offset() + right.len();
                let new_end = max(left_end, right_end);

                let new_span = LabeledSpan::new(
                    left.label().map(String::from),
                    left.offset(),
                    new_end - left.offset(),
                );
                // Check that the two contexts can be combined.
                if let Some(new_conts) = read(new_span.span()) {
                    contexts.pop();
                    contexts.push((Cow::Owned(new_span), new_conts));
                    continue;
                }
            }

            contexts.push((Cow::Borrowed(right), right_conts));
        }
        for (ctx, conts) in contexts {
            self.render_context(f, &ctx, &conts, labels, source_name)?;
        }

        Ok(())
    }

    pub(super) fn render_context(
        &self,
        f: &mut impl fmt::Write,
        context: &LabeledSpan,
        contents: &SpanContents<'_>,
        labels: &[&LabeledSpan],
        source_name: Option<&str>,
    ) -> fmt::Result {
        let lines = Self::get_lines(contents);

        // Only labels within this context can be its primary label.
        let mut ctx_labels =
            labels.iter().filter(|label| context.span().contains_inclusive(label.span()));
        let primary_label =
            ctx_labels.clone().find(|label| label.primary()).or_else(|| ctx_labels.next());

        // Assign styles after sorting labels by source position.
        let labels = labels
            .iter()
            .copied()
            .zip(self.theme.styles.highlights.iter().copied().cycle())
            .map(|(label, style)| FancySpan::new(label.label(), label.span(), style))
            .collect::<SmallVec<[_; 2]>>();

        // Find the maximum number of active gutter lines to determine indentation.
        let mut max_gutter = 0usize;
        for line in &lines {
            let mut num_highlights = 0;
            for hl in &labels {
                if !line.span_line_only(hl) && line.span_applies_gutter(hl) {
                    num_highlights += 1;
                }
            }
            max_gutter = max(max_gutter, num_highlights);
        }

        // Determine the width of the line-number column.
        let linum_width = lines
            .last()
            .map_or(1, |line| line.number.checked_ilog10().map_or(1, |width| width as usize + 1));

        // Header
        write_repeated_char(f, ' ', linum_width + 2)?;
        f.write_char(self.theme.characters.ltop)?;
        f.write_char(self.theme.characters.hbar)?;

        // Derive the primary label location from `contents`: its data begins
        // at a line boundary at `contents.line()`, and the primary label always
        // lies within it, so only the short prefix needs to be walked.
        let (primary_line, primary_column) = match primary_label {
            Some(label) => {
                contents.line_column_at(label.span().start as usize).ok_or(fmt::Error)?
            }
            None => (contents.line(), contents.column()),
        };

        match source_name {
            Some(source_name) => {
                let style = self.theme.styles.link;
                if style.is_plain() {
                    Self::write_location(
                        f,
                        Some(source_name),
                        primary_line + 1,
                        primary_column + 1,
                    )?;
                } else {
                    let source_name = source_name.style(style);
                    writeln!(f, "[{}:{}:{}]", source_name, primary_line + 1, primary_column + 1)?;
                }
            }
            _ => {
                if lines.len() <= 1 {
                    write_repeated_char(f, self.theme.characters.hbar, 3)?;
                    f.write_char('\n')?;
                } else {
                    Self::write_location(f, None, primary_line + 1, primary_column + 1)?;
                }
            }
        }

        let mut visible = vec![false; lines.len()];
        for (index, line) in lines.iter().enumerate() {
            if index == 0
                || index + 1 == lines.len()
                || labels.iter().any(|label| line.span_applies(label) && !line.span_flyby(label))
            {
                let start = index.saturating_sub(4);
                let end = (index + 5).min(lines.len());
                visible[start..end].fill(true);
            }
        }

        for run in visible.chunk_by_mut(|left, right| left == right) {
            if !run[0] && run.len() < MIN_COLLAPSED_LINES {
                run.fill(true);
            }
        }

        let mut skipped = false;
        for (line, visible) in lines.iter().zip(visible) {
            if !visible {
                if !skipped {
                    write_repeated_char(f, ' ', (linum_width + 1).saturating_sub(3))?;
                    write!(f, "{}", "...".style(self.theme.styles.linum))?;
                    f.write_char(' ')?;
                    f.write_char(self.theme.characters.vbar_break)?;
                    f.write_char(' ')?;
                    self.render_line_gutter(f, max_gutter, line, &labels)?;
                    f.write_char('\n')?;
                }
                skipped = true;
                continue;
            }
            skipped = false;
            self.write_linum(f, linum_width, line.number)?;
            self.render_line_gutter(f, max_gutter, line, &labels)?;
            Self::render_line_text(f, line.text)?;

            let (single_line, multi_line): (SmallVec<[_; 2]>, SmallVec<[_; 2]>) = labels
                .iter()
                .filter(|hl| line.span_applies(hl))
                .partition(|hl| line.span_line_only(hl));
            if !single_line.is_empty() {
                self.write_no_linum(f, linum_width)?;
                self.render_highlight_gutter(
                    f,
                    max_gutter,
                    line,
                    &labels,
                    LabelRenderMode::SingleLine,
                )?;
                self.render_single_line_highlights(
                    f,
                    line,
                    linum_width,
                    max_gutter,
                    &single_line,
                    &labels,
                )?;
            }
            for hl in multi_line {
                if hl.has_label() && line.span_ends(hl) && !line.span_starts(hl) {
                    self.render_multi_line_end(f, &labels, max_gutter, linum_width, line, hl)?;
                }
            }
        }
        write_repeated_char(f, ' ', linum_width + 2)?;
        f.write_char(self.theme.characters.lbot)?;
        write_repeated_char(f, self.theme.characters.hbar, 4)?;
        f.write_char('\n')?;
        Ok(())
    }

    fn write_location(
        f: &mut impl fmt::Write,
        source_name: Option<&str>,
        line: usize,
        column: usize,
    ) -> fmt::Result {
        let mut line_buffer = itoa::Buffer::new();
        let mut column_buffer = itoa::Buffer::new();
        let line = line_buffer.format(line);
        let column = column_buffer.format(column);

        f.write_char('[')?;
        if let Some(source_name) = source_name {
            f.write_str(source_name)?;
            f.write_char(':')?;
        }
        f.write_str(line)?;
        f.write_char(':')?;
        f.write_str(column)?;
        f.write_str("]\n")
    }

    /// Renders a line to the output formatter, replacing tabs with spaces.
    pub(super) fn render_line_text(f: &mut impl fmt::Write, text: &str) -> fmt::Result {
        if !text.contains('\t') {
            f.write_str(text)?;
            return f.write_char('\n');
        }

        for (c, width) in text.chars().zip(Self::line_visual_char_width(text)) {
            if c == '\t' {
                for _ in 0..width {
                    f.write_char(' ')?;
                }
            } else {
                f.write_char(c)?;
            }
        }
        f.write_char('\n')?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::GraphicalReportHandler;

    fn render_span(line_count: usize, inner_label: bool) -> String {
        render_span_themed(line_count, inner_label, crate::GraphicalTheme::none())
    }

    fn render_span_themed(
        line_count: usize,
        inner_label: bool,
        theme: crate::GraphicalTheme,
    ) -> String {
        use std::fmt::Write;

        use oxc_span::Span;

        use crate::OxcDiagnostic;

        let mut source = String::new();
        for line in 1..=line_count {
            writeln!(source, "line{line:04}();").unwrap();
        }
        let mut labels = vec![Span::new(0, u32::try_from(source.len()).unwrap() - 1)];
        if inner_label {
            labels.push(Span::new(12 * 14, 12 * 14 + 4));
        }
        let diagnostic =
            OxcDiagnostic::warn("long span").with_labels(labels).with_source_code(source);
        let mut output = String::new();
        GraphicalReportHandler::new_themed(theme)
            .render_report(&mut output, diagnostic.as_ref())
            .unwrap();
        output
    }

    #[test]
    fn long_spans_collapse_without_losing_endpoints() {
        let output = render_span(1000, false);
        assert!(output.contains("..."));
        assert!(output.contains("line0001();"));
        assert!(output.contains("line0005();"));
        assert!(!output.contains("line0006();"));
        assert!(output.contains("line0996();"));
        assert!(output.contains("line1000();"));
        assert!(output.lines().count() < 20);
        insta::assert_snapshot!("collapsed_long_span", output);
    }

    #[test]
    fn short_spans_remain_complete() {
        for count in 1..=12 {
            let output = render_span(count, false);
            assert!(!output.contains("..."), "{output}");
            for line in 1..=count {
                assert!(output.contains(&format!("line{line:04}();")));
            }
        }
        insta::assert_snapshot!("span_at_collapse_threshold", render_span(12, false));
        insta::assert_snapshot!("span_above_collapse_threshold", render_span(13, false));
    }

    #[test]
    fn omission_gutters_align_with_source_rows() {
        for count in [13, 100, 1000] {
            let output = render_span(count, false);
            let source_row = output.lines().find(|line| line.contains("line0002")).unwrap();
            let omission = output.lines().find(|line| line.contains("...")).unwrap();
            assert_eq!(omission.find(':'), source_row.find('|'), "{output}");
        }
    }

    #[test]
    fn labels_inside_long_spans_remain_visible() {
        let output = render_span(40, true);
        assert!(output.contains("line0015();"));
        assert!(output.contains("^^^^"));
        assert_eq!(output.matches("...").count(), 2);
        insta::assert_snapshot!("collapsed_span_with_inner_label", output);
    }

    #[test]
    fn omission_marker_uses_line_number_style() {
        let output = render_span_themed(13, false, crate::GraphicalTheme::unicode());
        assert!(output.contains("\x1b[2m...\x1b[0m"));
        assert!(output.contains("\x1b[2m 1\x1b[0m"));
        insta::assert_snapshot!("colored_collapsed_span", output);
    }

    #[test]
    fn separate_labels_keep_context_around_collapsed_gap() {
        use oxc_span::Span;

        use crate::{GraphicalTheme, LabeledSpan, source_impls::SpanScanner};

        let source = "statement();\n".repeat(30);
        let span = Span::new(0, u32::try_from(source.len()).unwrap());
        let context = LabeledSpan::new(None, 0, span.end);
        let first = LabeledSpan::new(Some("first".into()), 13, 9);
        let last = LabeledSpan::new(Some("last".into()), 13 * 28, 9);
        let mut scanner = SpanScanner::new(source.as_bytes(), 0, 0);
        let contents = scanner.read_span(span).unwrap();
        let mut output = String::new();
        GraphicalReportHandler::new_themed(GraphicalTheme::none())
            .render_context(&mut output, &context, &contents, &[&first, &last], None)
            .unwrap();
        assert_eq!(output.matches("...").count(), 1);
        assert!(output.contains("first"));
        assert!(output.contains("last"));
        insta::assert_snapshot!("separate_labels_with_collapsed_gap", output);
    }

    #[test]
    fn direct_locations_match_standard_formatting() {
        for (source_name, line, column) in
            [(None, 1, 1), (Some("src/火.ts"), usize::MAX, usize::MAX)]
        {
            let expected = source_name.map_or_else(
                || format!("[{line}:{column}]\n"),
                |source_name| format!("[{source_name}:{line}:{column}]\n"),
            );
            let mut actual = String::new();
            GraphicalReportHandler::write_location(&mut actual, source_name, line, column).unwrap();
            assert_eq!(actual, expected);
        }
    }
}
