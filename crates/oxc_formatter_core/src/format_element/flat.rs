//! What a run of elements printed flat does to the line, for a fits measurement to skip a group's content.
//! See [`Group::flat`](super::tag::Group::flat).
//!
//! A run has a [`Flat`] only if its effect is the same wherever it is printed:
//! nothing in it reads the printer's state other than the line width and the pending space.
//! That leaves out:
//! - groups with an id (a measurement notes their print mode), conditional content with a group id
//! - line suffixes and their boundaries, Tailwind classes
//! - line breaks that a flat print still prints, multiline texts
//! - a first text without width (whether the line is still empty after it depends on what precedes it)
//!
//! `IndentIfGroupBreaks` reads another group's mode too, but an indent only applies after a line break, which a run with a [`Flat`] has none of.

use super::TextWidth;

/// The space that a run of elements leaves pending before its first text.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Lead {
    #[default]
    None = 0,
    /// A [`Space`](super::FormatElement::Space): only if the line is not empty.
    IfLineNotEmpty = 1,
    /// A [`SoftOrSpace`](super::LineMode::SoftOrSpace) line.
    Always = 2,
}

/// The effect of a run of elements printed flat on a fits measurement.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Flat {
    pub has_text: bool,
    lead: Lead,
    /// From the start of the first text to the end of the last one.
    pub width: u32,
    /// Whether a space is pending after the last text.
    pub trailing: bool,
}

impl Flat {
    const KNOWN: u16 = 1 << 15;
    const HAS_TEXT: u16 = 1 << 14;
    const TRAILING: u16 = 1 << 13;
    const LEAD_SHIFT: u16 = 11;
    const MAX_WIDTH: u16 = (1 << Self::LEAD_SHIFT) - 1;

    /// Whether a space is pending before the first text,
    /// given what was pending and the line width before the run.
    pub fn leads_with_space(self, pending_space: bool, line_width: usize) -> bool {
        pending_space
            || match self.lead {
                Lead::None => false,
                Lead::IfLineNotEmpty => line_width > 0,
                Lead::Always => true,
            }
    }

    /// Appends a text.
    /// `None` if its effect depends on where it is printed.
    pub fn text(self, width: TextWidth) -> Option<Self> {
        if width.is_multiline() {
            return None;
        }
        let width = width.value();
        if self.has_text {
            let width = self.width.saturating_add(u32::from(self.trailing)).saturating_add(width);
            Some(Self { width, trailing: false, ..self })
        } else if width == 0 {
            None
        } else {
            Some(Self { has_text: true, width, trailing: false, ..self })
        }
    }

    /// Appends a space.
    pub fn space(self, lead: Lead) -> Self {
        if self.has_text {
            // The line is not empty after the first text
            Self { trailing: true, ..self }
        } else {
            Self { lead: self.lead.max(lead), ..self }
        }
    }

    /// Appends another run.
    pub fn then(self, next: Self) -> Self {
        match (self.has_text, next.has_text) {
            (_, false) if next.lead == Lead::None => self,
            (_, false) => self.space(next.lead),
            (true, true) => Self {
                width: self
                    .width
                    .saturating_add(u32::from(self.trailing || next.lead != Lead::None))
                    .saturating_add(next.width),
                trailing: next.trailing,
                ..self
            },
            (false, true) => Self { lead: self.lead.max(next.lead), ..next },
        }
    }

    /// `KNOWN | HAS_TEXT? | TRAILING? | lead << LEAD_SHIFT | width`, 0 for `None`.
    /// `None` too if the width does not fit: saturating it would not do, as it is compared with any print width.
    pub fn pack(flat: Option<Self>) -> u16 {
        let Some(flat) = flat else { return 0 };
        let Some(width) = u16::try_from(flat.width).ok().filter(|width| *width <= Self::MAX_WIDTH)
        else {
            return 0;
        };
        Self::KNOWN
            | if flat.has_text { Self::HAS_TEXT } else { 0 }
            | if flat.trailing { Self::TRAILING } else { 0 }
            | ((flat.lead as u16) << Self::LEAD_SHIFT)
            | width
    }

    pub fn unpack(bits: u16) -> Option<Self> {
        if bits & Self::KNOWN == 0 {
            return None;
        }
        let lead = match (bits >> Self::LEAD_SHIFT) & 0b11 {
            0 => Lead::None,
            1 => Lead::IfLineNotEmpty,
            _ => Lead::Always,
        };
        Some(Self {
            has_text: bits & Self::HAS_TEXT != 0,
            lead,
            width: u32::from(bits & Self::MAX_WIDTH),
            trailing: bits & Self::TRAILING != 0,
        })
    }
}
