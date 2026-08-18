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

use owo_colors::OwoColorize;
use smallvec::SmallVec;

use super::{
    handler::GraphicalReportHandler,
    label::write_repeated_char,
    span::{FancySpan, LabelRenderMode},
};
use crate::{
    Diagnostic, LabeledSpan,
    source_impls::{SpanContents, SpanScanner},
};

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

        for line in &lines {
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
