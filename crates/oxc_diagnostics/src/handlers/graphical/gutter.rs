//! The gutter: everything drawn to the left of the source text.
//!
//! That is the line-number column ([`write_linum`](GraphicalReportHandler::write_linum) /
//! [`write_no_linum`](GraphicalReportHandler::write_no_linum)) and the vertical
//! "gutter" lines that connect the start and end of a multi-line span
//! ([`render_line_gutter`](GraphicalReportHandler::render_line_gutter) /
//! [`render_highlight_gutter`](GraphicalReportHandler::render_highlight_gutter)).

use std::fmt::{self, Write};

use owo_colors::OwoColorize;

use super::{
    handler::GraphicalReportHandler,
    label::write_repeated_char,
    line::Line,
    span::{FancySpan, LabelRenderMode},
};

impl GraphicalReportHandler {
    pub(super) fn write_linum(
        &self,
        f: &mut impl fmt::Write,
        width: usize,
        linum: usize,
    ) -> fmt::Result {
        f.write_char(' ')?;
        if self.theme.styles.linum.is_plain() {
            let mut buffer = itoa::Buffer::new();
            let linum = buffer.format(linum);
            write_repeated_char(f, ' ', width.saturating_sub(linum.len()))?;
            f.write_str(linum)?;
        } else {
            write!(f, "{:width$}", linum.style(self.theme.styles.linum), width = width)?;
        }
        f.write_char(' ')?;
        f.write_char(self.theme.characters.vbar)?;
        f.write_char(' ')
    }

    pub(super) fn write_no_linum(&self, f: &mut impl fmt::Write, width: usize) -> fmt::Result {
        write_repeated_char(f, ' ', width + 2)?;
        f.write_char(self.theme.characters.vbar_break)?;
        f.write_char(' ')
    }

    /// Draws the gutter alongside the source line itself: for each active
    /// multi-line span this is the corner/arrow that opens or closes it, or a
    /// vertical bar for spans that fly past this line.
    pub(super) fn render_line_gutter(
        &self,
        f: &mut impl fmt::Write,
        max_gutter: usize,
        line: &Line<'_>,
        highlights: &[FancySpan],
    ) -> fmt::Result {
        if max_gutter == 0 {
            return Ok(());
        }
        let chars = &self.theme.characters;
        let mut gutter = String::new();
        let applicable = highlights.iter().filter(|hl| line.span_applies_gutter(hl));
        let mut arrow = false;
        for (i, hl) in applicable.enumerate() {
            if line.span_starts(hl) {
                write!(gutter, "{}", chars.ltop.style(hl.style))?;
                write!(
                    gutter,
                    "{}",
                    chars.hbar.to_string().repeat(max_gutter.saturating_sub(i)).style(hl.style)
                )?;
                write!(gutter, "{}", chars.rarrow.style(hl.style))?;
                arrow = true;
                break;
            } else if line.span_ends(hl) {
                if hl.has_label() {
                    write!(gutter, "{}", chars.lcross.style(hl.style))?;
                } else {
                    write!(gutter, "{}", chars.lbot.style(hl.style))?;
                }
                write!(
                    gutter,
                    "{}",
                    chars.hbar.to_string().repeat(max_gutter.saturating_sub(i)).style(hl.style)
                )?;
                write!(gutter, "{}", chars.rarrow.style(hl.style))?;
                arrow = true;
                break;
            } else if line.span_flyby(hl) {
                write!(gutter, "{}", chars.vbar.style(hl.style))?;
            } else {
                gutter.push(' ');
            }
        }
        write!(
            f,
            "{}{}",
            gutter,
            " ".repeat(
                if arrow { 1 } else { 3 } + max_gutter.saturating_sub(gutter.chars().count())
            )
        )?;
        Ok(())
    }

    /// Draws the gutter alongside a highlight/label line (i.e. the lines below
    /// the source text). Unlike [`Self::render_line_gutter`] it must keep an
    /// explicit column count because the styled string contains ANSI escapes,
    /// so its byte length is not its rendered width.
    pub(super) fn render_highlight_gutter(
        &self,
        f: &mut impl fmt::Write,
        max_gutter: usize,
        line: &Line<'_>,
        highlights: &[FancySpan],
        render_mode: LabelRenderMode,
    ) -> fmt::Result {
        if max_gutter == 0 {
            return Ok(());
        }

        // keeps track of how many columns wide the gutter is
        // important for ansi since simply measuring the size of the final string
        // gives the wrong result when the string contains ansi codes.
        let mut gutter_cols = 0;

        let chars = &self.theme.characters;
        let mut gutter = String::new();
        let applicable = highlights.iter().filter(|hl| line.span_applies_gutter(hl));
        for (i, hl) in applicable.enumerate() {
            if !line.span_line_only(hl) && line.span_ends(hl) {
                if render_mode == LabelRenderMode::BlockRest {
                    // this is to make multiline labels work. We want to make the right amount
                    // of horizontal space for them, but not actually draw the lines
                    let horizontal_space = max_gutter.saturating_sub(i) + 2;
                    for _ in 0..horizontal_space {
                        gutter.push(' ');
                    }
                    // account for one more horizontal space, since in multiline mode
                    // we also add in the vertical line before the label like this:
                    // 2 │ ╭─▶   text
                    // 3 │ ├─▶     here
                    //   · ╰──┤ these two lines
                    //   ·    │ are the problem
                    //        ^this
                    gutter_cols += horizontal_space + 1;
                } else {
                    let num_repeat = max_gutter.saturating_sub(i) + 2;

                    write!(gutter, "{}", chars.lbot.style(hl.style))?;

                    write!(
                        gutter,
                        "{}",
                        chars
                            .hbar
                            .to_string()
                            .repeat(
                                num_repeat
                                    // if we are rendering a multiline label, then leave a bit of space for the
                                    // rcross character
                                    - usize::from(render_mode == LabelRenderMode::BlockFirst),
                            )
                            .style(hl.style)
                    )?;

                    // we count 1 for the lbot char, and then a few more, the same number
                    // as we just repeated for. For each repeat we only add 1, even though
                    // due to ansi escape codes the number of bytes in the string could grow
                    // a lot each time.
                    gutter_cols += num_repeat + 1;
                }
                break;
            }
            write!(gutter, "{}", chars.vbar.style(hl.style))?;

            // we may push many bytes for the ansi escape codes style adds,
            // but we still only add a single character-width to the string in a terminal
            gutter_cols += 1;
        }

        // now calculate how many spaces to add based on how many columns we just created.
        // it's the max width of the gutter, minus how many character-widths we just generated
        // capped at 0 (though this should never go below in reality), and then we add 3 to
        // account for arrowheads when a gutter line ends
        let num_spaces = (max_gutter + 3).saturating_sub(gutter_cols);
        // we then write the gutter and as many spaces as we need
        write!(f, "{}{:width$}", gutter, "", width = num_spaces)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GraphicalTheme;

    #[test]
    fn direct_plain_line_numbers_match_standard_formatting() {
        let handler = GraphicalReportHandler::new_themed(GraphicalTheme::none());
        for (width, linum) in [(3, 1), (1, usize::MAX)] {
            let mut actual = String::new();
            handler.write_linum(&mut actual, width, linum).unwrap();
            assert_eq!(actual, format!(" {linum:width$} | "));
        }
    }
}
