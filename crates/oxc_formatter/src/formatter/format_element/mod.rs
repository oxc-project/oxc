pub mod document;
pub mod tag;

// #[cfg(target_pointer_width = "64")]
// use biome_rowan::static_assert;
use std::hash::{Hash, Hasher};
use std::ptr;
use std::{borrow::Cow, ops::Deref};

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use oxc_allocator::Vec as ArenaVec;

use crate::IndentWidth;

use super::{
    TagKind,
    format_element::tag::{LabelId, Tag},
};

#[cfg(debug_assertions)]
const _: () = {
    if cfg!(target_pointer_width = "64") {
        assert!(
            size_of::<FormatElement>() == 40,
            "`FormatElement` size exceeds 40 bytes, expected 40 bytes in 64-bit platforms"
        );
    } else if cfg!(target_family = "wasm") || align_of::<u64>() == 8 {
        // Some 32-bit platforms have 8-byte alignment for `u64` and `f64`, while others have 4-byte alignment.
        //
        // Skip these assertions on 32-bit platforms where `u64` / `f64` have 4-byte alignment, because
        // some layout calculations may be incorrect. https://github.com/oxc-project/oxc/pull/13716
        assert!(
            size_of::<FormatElement>() == 24,
            "`FormatElement` size exceeds 24 bytes, expected 24 bytes in 32-bit platforms"
        );
    }
};

#[cfg(not(debug_assertions))]
const _: () = {
    if cfg!(target_pointer_width = "64") {
        assert!(
            size_of::<FormatElement>() == 24,
            "`FormatElement` size exceeds 24 bytes, expected 24 bytes in 64-bit platforms"
        );
    } else if cfg!(target_family = "wasm") || align_of::<u64>() == 8 {
        // Some 32-bit platforms have 8-byte alignment for `u64` and `f64`, while others have 4-byte alignment.
        //
        // Skip these assertions on 32-bit platforms where `u64` / `f64` have 4-byte alignment, because
        // some layout calculations may be incorrect. https://github.com/oxc-project/oxc/pull/13716
        assert!(
            size_of::<FormatElement>() == 16,
            "`FormatElement` size exceeds 16 bytes, expected 16 bytes in 32-bit platforms"
        );
    }
};

/// Language agnostic IR for formatting source code.
///
/// Use the helper functions like [crate::builders::space], [crate::builders::soft_line_break] etc. defined in this file to create elements.
#[derive(Clone, Eq, PartialEq)]
pub enum FormatElement<'a> {
    /// A space token, see [crate::builders::space] for documentation.
    Space,
    HardSpace,
    /// A new line, see [crate::builders::soft_line_break], [crate::builders::hard_line_break], and [crate::builders::soft_line_break_or_space] for documentation.
    Line(LineMode),

    /// Forces the parent group to print in expanded mode.
    ExpandParent,

    /// A ASCII only Token that contains no line breaks or tab characters.
    Token {
        text: &'static str,
    },

    /// An arbitrary text that can contain tabs, newlines, and unicode characters.
    Text {
        text: &'a str,
        width: TextWidth,
    },

    /// Prevents that line suffixes move past this boundary. Forces the printer to print any pending
    /// line suffixes, potentially by inserting a hard line break.
    LineSuffixBoundary,

    /// An interned format element. Useful when the same content must be emitted multiple times to avoid
    /// deep cloning the IR when using the `best_fitting!` macro or `if_group_fits_on_line` and `if_group_breaks`.
    Interned(Interned<'a>),

    /// A list of different variants representing the same content. The printer picks the best fitting content.
    /// Line breaks inside of a best fitting don't propagate to parent groups.
    BestFitting(BestFittingElement<'a>),

    /// A [Tag] that marks the start/end of some content to which some special formatting is applied.
    Tag(Tag),
}

impl std::fmt::Debug for FormatElement<'_> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FormatElement::Space | FormatElement::HardSpace => fmt.write_str("Space"),
            FormatElement::Line(mode) => fmt.debug_tuple("Line").field(mode).finish(),
            FormatElement::ExpandParent => fmt.write_str("ExpandParent"),
            FormatElement::Token { text } => fmt.debug_tuple("Token").field(text).finish(),
            FormatElement::Text { text, .. } => fmt.debug_tuple("Text").field(text).finish(),
            FormatElement::LineSuffixBoundary => fmt.write_str("LineSuffixBoundary"),
            FormatElement::BestFitting(best_fitting) => {
                fmt.debug_tuple("BestFitting").field(&best_fitting).finish()
            }
            FormatElement::Interned(interned) => fmt.debug_list().entries(&**interned).finish(),
            FormatElement::Tag(tag) => fmt.debug_tuple("Tag").field(tag).finish(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LineMode {
    /// See [crate::builders::soft_line_break_or_space] for documentation.
    SoftOrSpace,
    /// See [crate::builders::soft_line_break] for documentation.
    Soft,
    /// See [crate::builders::hard_line_break] for documentation.
    Hard,
    /// See [crate::builders::empty_line] for documentation.
    Empty,
}

impl LineMode {
    pub const fn is_hard(self) -> bool {
        matches!(self, LineMode::Hard)
    }

    pub const fn will_break(self) -> bool {
        matches!(self, LineMode::Hard | LineMode::Empty)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PrintMode {
    /// Omits any soft line breaks
    Flat,
    /// Prints soft line breaks as line breaks
    Expanded,
}

impl PrintMode {
    pub const fn is_flat(self) -> bool {
        matches!(self, PrintMode::Flat)
    }

    pub const fn is_expanded(self) -> bool {
        matches!(self, PrintMode::Expanded)
    }
}

#[derive(Clone)]
pub struct Interned<'a>(&'a [FormatElement<'a>]);

impl<'a> Interned<'a> {
    pub(super) fn new(content: ArenaVec<'a, FormatElement<'a>>) -> Self {
        Self(content.into_bump_slice())
    }
}

impl PartialEq for Interned<'_> {
    fn eq(&self, other: &Interned<'_>) -> bool {
        ptr::eq(self.0, other.0)
    }
}

impl Eq for Interned<'_> {}

impl Hash for Interned<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_ptr().addr().hash(state);
    }
}

impl std::fmt::Debug for Interned<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> Deref for Interned<'a> {
    type Target = [FormatElement<'a>];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

const LINE_SEPARATOR: char = '\u{2028}';

const PARAGRAPH_SEPARATOR: char = '\u{2029}';

#[expect(unused)]
pub const LINE_TERMINATORS: [char; 3] = ['\r', LINE_SEPARATOR, PARAGRAPH_SEPARATOR];

/// Replace the line terminators matching the provided list with "\n"
/// since its the only line break type supported by the printer
#[expect(unused)]
pub fn normalize_newlines<const N: usize>(text: &str, terminators: [char; N]) -> Cow<'_, str> {
    let mut result = String::new();
    let mut last_end = 0;

    for (start, part) in text.match_indices(terminators) {
        result.push_str(&text[last_end..start]);
        result.push('\n');

        last_end = start + part.len();
        // If the current character is \r and the
        // next is \n, skip over the entire sequence
        if part == "\r" && text[last_end..].starts_with('\n') {
            last_end += 1;
        }
    }

    // If the result is empty no line terminators were matched,
    // return the entire input text without allocating a new String
    if result.is_empty() {
        Cow::Borrowed(text)
    } else {
        result.push_str(&text[last_end..text.len()]);
        Cow::Owned(result)
    }
}

impl FormatElement<'_> {
    /// Returns `true` if self is a [FormatElement::Tag]
    pub const fn is_tag(&self) -> bool {
        matches!(self, FormatElement::Tag(_))
    }

    /// Returns `true` if self is a [FormatElement::Tag] and [Tag::is_start] is `true`.
    pub const fn is_start_tag(&self) -> bool {
        match self {
            FormatElement::Tag(tag) => tag.is_start(),
            _ => false,
        }
    }

    /// Returns `true` if self is a [FormatElement::Tag] and [Tag::is_end] is `true`.
    pub const fn is_end_tag(&self) -> bool {
        match self {
            FormatElement::Tag(tag) => tag.is_end(),
            _ => false,
        }
    }

    pub const fn is_text(&self) -> bool {
        matches!(self, FormatElement::Text { .. } | FormatElement::Token { .. })
    }

    pub const fn is_space(&self) -> bool {
        matches!(self, FormatElement::Space)
    }

    pub const fn is_line(&self) -> bool {
        matches!(self, FormatElement::Line(_))
    }
}

impl FormatElements for FormatElement<'_> {
    fn will_break(&self) -> bool {
        match self {
            FormatElement::ExpandParent => true,
            FormatElement::Tag(Tag::StartGroup(group)) => !group.mode().is_flat(),
            FormatElement::Line(line_mode) => line_mode.will_break(),
            FormatElement::Text { text: _, width } => width.is_multiline(),
            FormatElement::Interned(interned) => interned.will_break(),
            // Traverse into the most flat version because the content is guaranteed to expand when even
            // the most flat version contains some content that forces a break.
            FormatElement::BestFitting(best_fitting) => best_fitting.most_flat().will_break(),
            // `FormatElement::Token` cannot contain line breaks
            FormatElement::Token { .. }
            | FormatElement::LineSuffixBoundary
            | FormatElement::Space
            | FormatElement::Tag(_)
            | FormatElement::HardSpace => false,
        }
    }

    fn may_directly_break(&self) -> bool {
        matches!(self, FormatElement::Line(_))
    }

    fn has_label(&self, label_id: LabelId) -> bool {
        match self {
            FormatElement::Tag(Tag::StartLabelled(actual)) => *actual == label_id,
            FormatElement::Interned(interned) => interned.deref().has_label(label_id),
            _ => false,
        }
    }

    fn start_tag(&self, _: TagKind) -> Option<&Tag> {
        None
    }

    fn end_tag(&self, kind: TagKind) -> Option<&Tag> {
        match self {
            FormatElement::Tag(tag) if tag.kind() == kind && tag.is_end() => Some(tag),
            _ => None,
        }
    }
}

/// Provides the printer with different representations for the same element so that the printer
/// can pick the best fitting variant.
///
/// Best fitting is defined as the variant that takes the most horizontal space but fits on the line.
#[derive(Clone, Eq, PartialEq)]
pub struct BestFittingElement<'a> {
    /// The different variants for this element.
    /// The first element is the one that takes up the most space horizontally (the most flat),
    /// The last element takes up the least space horizontally (but most horizontal space).
    variants: &'a [&'a [FormatElement<'a>]],
}

impl<'a> BestFittingElement<'a> {
    /// Creates a new best fitting IR with the given variants. The method itself isn't unsafe
    /// but it is to discourage people from using it because the printer will panic if
    /// the slice doesn't contain at least the least and most expanded variants.
    ///
    /// You're looking for a way to create a `BestFitting` object, use the `best_fitting![least_expanded, most_expanded]` macro.
    ///
    /// ## Safety
    /// The slice must contain at least two variants.
    #[doc(hidden)]
    pub unsafe fn from_vec_unchecked(variants: ArenaVec<'a, &'a [FormatElement<'a>]>) -> Self {
        debug_assert!(
            variants.len() >= 2,
            "Requires at least the least expanded and most expanded variants"
        );

        Self { variants: variants.into_bump_slice() }
    }

    /// Returns the most expanded variant
    pub fn most_expanded(&self) -> &[FormatElement<'a>] {
        self.variants.last().expect(
            "Most contain at least two elements, as guaranteed by the best fitting builder.",
        )
    }

    /// Splits the variants into the most expanded and the remaining flat variants
    pub fn split_to_most_expanded_and_flat_variants(
        &self,
    ) -> (&&[FormatElement<'a>], &[&[FormatElement<'a>]]) {
        // SAFETY: We have already asserted that there are at least two variants for creating this struct.
        unsafe { self.variants.split_last().unwrap_unchecked() }
    }

    pub fn variants(&self) -> &[&'a [FormatElement<'a>]] {
        self.variants
    }

    /// Returns the least expanded variant
    pub fn most_flat(&self) -> &[FormatElement<'a>] {
        self.variants.first().expect(
            "Most contain at least two elements, as guaranteed by the best fitting builder.",
        )
    }
}

impl std::fmt::Debug for BestFittingElement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.variants).finish()
    }
}

pub trait FormatElements {
    /// Returns true if this [FormatElement] is guaranteed to break across multiple lines by the printer.
    /// This is the case if this format element recursively contains a:
    /// * [crate::builders::empty_line] or [crate::builders::hard_line_break]
    /// * A token containing '\n'
    ///
    /// Use this with caution, this is only a heuristic and the printer may print the element over multiple
    /// lines if this element is part of a group and the group doesn't fit on a single line.
    fn will_break(&self) -> bool;

    /// Returns true if this [FormatElement] has the potential to break across multiple lines when printed.
    /// This is the case _only_ if this format element recursively contains a [FormatElement::Line].
    ///
    /// It's possible for [FormatElements::will_break] to return true while this function returns false,
    /// such as when the group contains a [crate::builders::expand_parent] or some text within the group
    /// contains a newline. Neither of those cases directly contain a [FormatElement::Line], and so they
    /// do not _directly_ break.
    fn may_directly_break(&self) -> bool;

    /// Returns true if the element has the given label.
    fn has_label(&self, label: LabelId) -> bool;

    /// Returns the start tag of `kind` if:
    /// * the last element is an end tag of `kind`.
    /// * there's a matching start tag in this document (may not be true if this slice is an interned element and the `start` is in the document storing the interned element).
    #[expect(unused)]
    fn start_tag(&self, kind: TagKind) -> Option<&Tag>;

    /// Returns the end tag if:
    /// * the last element is an end tag of `kind`
    fn end_tag(&self, kind: TagKind) -> Option<&Tag>;
}

/// New-type wrapper for a text Unicode width. Mainly to prevent access to the inner value.
///
/// ## Representation
///
/// Uses a single `u32` to efficiently store both the width value and a multiline flag:
///
/// - Bit 31: Multiline flag (1 = multiline, 0 = single line)
/// - Bits 0-30: Width value (stored directly)
///
/// This encoding allows `TextWidth` to fit in 4 bytes while supporting both a width value
/// and a boolean flag. The maximum representable width is 2^31 - 1 (0x7FFFFFFF).
///
/// The maximum value is sufficient in practice as texts that long would exceed any
/// reasonable line width configuration (typically < 500 columns).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TextWidth(u32);

impl TextWidth {
    /// Bit mask for the multiline flag (highest bit)
    const MULTILINE_MASK: u32 = 1 << 31;

    /// Bit mask for extracting the width value (all bits except the highest)
    const WIDTH_MASK: u32 = Self::MULTILINE_MASK - 1;

    /// Encodes width and multiline flag into a single u32.
    const fn encode(width: u32, multiline: bool) -> u32 {
        debug_assert!(width <= Self::WIDTH_MASK, "width exceeds maximum representable value");

        // Set multiline flag if needed
        if multiline { width | Self::MULTILINE_MASK } else { width }
    }

    /// Creates a single-line text width.
    pub const fn single(width: u32) -> Self {
        Self(Self::encode(width, false))
    }

    /// Creates a multi-line text width.
    pub const fn multiline(width: u32) -> Self {
        Self(Self::encode(width, true))
    }

    /// Returns the width value.
    pub const fn value(self) -> u32 {
        self.0 & Self::WIDTH_MASK
    }

    /// Calculates width from text, handling tabs, newlines, and Unicode.
    ///
    /// Returns early on newline detection for efficiency.
    pub fn from_text(text: &str, indent_width: IndentWidth) -> TextWidth {
        // Fast path for empty text
        if text.is_empty() {
            return Self::single(0);
        }

        let mut width = 0u32;

        #[expect(clippy::cast_lossless)]
        for c in text.chars() {
            let char_width = match c {
                '\t' => indent_width.value(),
                '\n' => return Self::multiline(width),
                #[expect(clippy::cast_possible_truncation)]
                c => c.width().unwrap_or(0) as u8,
            };
            width += char_width as u32;
        }

        Self::single(width)
    }

    /// Creates width from a string known to not contain whitespace.
    /// More efficient than `from_text` when whitespace is guaranteed absent.
    pub fn from_non_whitespace_str(name: &str) -> TextWidth {
        #[expect(clippy::cast_possible_truncation)]
        Self::single(name.width() as u32)
    }

    /// Returns true if the text contains newlines.
    pub(crate) const fn is_multiline(self) -> bool {
        (self.0 & Self::MULTILINE_MASK) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn indent_width(value: u8) -> IndentWidth {
        IndentWidth::try_from(value).expect("valid indent width")
    }

    #[test]
    fn single_width_round_trips_zero() {
        let width = TextWidth::single(0);
        debug_assert_eq!(width.value(), 0);
        debug_assert!(!width.is_multiline());
    }

    #[test]
    fn multiline_sets_flag_without_touching_width() {
        let width = TextWidth::multiline(3);
        debug_assert_eq!(width.value(), 3);
        debug_assert!(width.is_multiline());
    }

    #[test]
    fn from_text_uses_indent_width_for_tabs() {
        let width = TextWidth::from_text("\t", indent_width(4));
        debug_assert_eq!(width.value(), 4);
        debug_assert!(!width.is_multiline());
    }

    #[test]
    fn from_text_marks_multiline_on_newline() {
        let width = TextWidth::from_text("ab\nc", indent_width(2));
        debug_assert_eq!(width.value(), 2);
        debug_assert!(width.is_multiline());
    }

    #[test]
    fn from_non_whitespace_and_len_match() {
        let name = "hello";
        #[expect(clippy::cast_possible_truncation)]
        let name_len = name.len() as u32;
        debug_assert_eq!(TextWidth::from_non_whitespace_str(name).value(), name_len);
    }

    #[test]
    fn is_single_line_inverse_of_is_multiline() {
        let single = TextWidth::single(10);
        debug_assert!(!single.is_multiline());

        let multi = TextWidth::multiline(10);
        debug_assert!(multi.is_multiline());
    }

    #[test]
    fn from_text_handles_unicode() {
        // Emoji width
        let width = TextWidth::from_text("👍", indent_width(2));
        debug_assert_eq!(width.value(), 2); // Most emojis are width 2
        debug_assert!(!width.is_multiline());

        // Chinese characters
        let width = TextWidth::from_text("你好", indent_width(2));
        debug_assert_eq!(width.value(), 4); // Each CJK char is width 2
        debug_assert!(!width.is_multiline());
    }

    #[test]
    fn from_text_empty_returns_zero() {
        let width = TextWidth::from_text("", indent_width(2));
        debug_assert_eq!(width.value(), 0);
        debug_assert!(!width.is_multiline());
    }
}
