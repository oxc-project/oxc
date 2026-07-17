pub mod debug;
pub mod document;
pub mod tag;

// #[cfg(target_pointer_width = "64")]
// use biome_rowan::static_assert;
use std::hash::{Hash, Hasher};
use std::ptr;
use std::{borrow::Cow, ops::Deref};

use unicode_width::UnicodeWidthStr;

use crate::IndentWidth;

use self::tag::{LabelId, Tag, TagKind};

// In debug builds `GroupId`/`LabelId` carry `&'static str` debug names, so the element is larger.
const _: () = {
    if cfg!(target_pointer_width = "64") && !cfg!(debug_assertions) {
        assert!(
            size_of::<FormatElement>() == 16,
            "`FormatElement` size exceeds 16 bytes on 64-bit platforms"
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
    /// A new line, see [crate::builders::soft_line_break], [crate::builders::hard_line_break], and [crate::builders::soft_line_break_or_space] for documentation.
    Line(LineMode),

    /// Forces the parent group to print in expanded mode.
    ExpandParent,

    /// A short (≤ 7 bytes) ASCII token without line breaks or tab characters, stored inline.
    /// Longer tokens fall back to [`FormatElement::ArenaText`].
    /// Build via [`FormatElement::token`].
    Token { len: u8, bytes: [u8; 7] },

    /// ASCII single-line text borrowed from the document source: display width equals `len`,
    /// the characters are resolved against the printer's source text at print time.
    /// Texts that don't qualify fall back to [`FormatElement::ArenaText`].
    SourceText { offset: u32, len: u32 },

    /// Any other text (multiline, non-ASCII, tabs, owned/normalized or embedded-language
    /// content): copied into the arena together with its precomputed [`TextWidth`].
    ArenaText(ArenaText<'a>),

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

    /// A Tailwind CSS class sorting marker.
    /// The usize is an index into the collected tailwind classes array.
    /// During printing, this will be replaced with the sorted class name.
    TailwindClass(usize),

    /// An embedded-language interpolation placeholder.
    /// The `u32` is the host's expression index (0-based, e.g. the Nth `${expr}` of a css-in-js template).
    /// The host that embeds this IR MUST replace the marker with the interpolation before the document is printed;
    /// it is not meant to reach the printer (which `debug_assert`s if one survives).
    EmbedPlaceholder(u32),
}

impl std::fmt::Debug for FormatElement<'_> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FormatElement::Space => fmt.write_str("Space"),
            FormatElement::Line(mode) => fmt.debug_tuple("Line").field(mode).finish(),
            FormatElement::ExpandParent => fmt.write_str("ExpandParent"),
            FormatElement::Token { .. } => {
                fmt.debug_tuple("Token").field(&self.token_text()).finish()
            }
            FormatElement::SourceText { offset, len } => {
                std::write!(fmt, "SourceText({}..{})", offset, offset + len)
            }
            FormatElement::ArenaText(text) => fmt.debug_tuple("Text").field(&text.text()).finish(),
            FormatElement::LineSuffixBoundary => fmt.write_str("LineSuffixBoundary"),
            FormatElement::BestFitting(best_fitting) => {
                fmt.debug_tuple("BestFitting").field(&best_fitting).finish()
            }
            FormatElement::Interned(interned) => fmt.debug_list().entries(&**interned).finish(),
            FormatElement::Tag(tag) => fmt.debug_tuple("Tag").field(tag).finish(),
            FormatElement::TailwindClass(index) => {
                fmt.debug_tuple("TailwindClass").field(index).finish()
            }
            FormatElement::EmbedPlaceholder(index) => {
                fmt.debug_tuple("EmbedPlaceholder").field(index).finish()
            }
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
    /// See [crate::builders::literal_line_break] for documentation.
    Literal,
}

impl LineMode {
    pub const fn is_hard(self) -> bool {
        matches!(self, LineMode::Hard)
    }

    pub const fn will_break(self) -> bool {
        matches!(self, LineMode::Hard | LineMode::Empty | LineMode::Literal)
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

/// Allocates one arena block holding `header` followed by a bitwise copy of `items`,
/// returning a reference to the written header.
///
/// This is the single building block behind every thin-pointer payload of
/// [`FormatElement`] ([`Interned`], [`BestFittingElement`], [`ArenaText`]): keeping the
/// length (and any metadata) in the arena instead of a fat pointer keeps each variant
/// at one word. Read the payload back with [`trailing`].
///
/// The bitwise copy is sound for any `!needs_drop` payload (checked at compile time);
/// the borrowed `items` are left untouched.
fn alloc_header_with_trailing<'a, H, T>(
    header: H,
    items: &[T],
    allocator: &'a oxc_allocator::Allocator,
) -> &'a H {
    const {
        assert!(!std::mem::needs_drop::<T>());
        assert!(align_of::<T>() <= align_of::<H>());
        assert!(size_of::<H>().is_multiple_of(align_of::<T>()));
    }
    let layout = std::alloc::Layout::from_size_align(
        size_of::<H>().checked_add(size_of_val(items)).expect("thin slice payload too large"),
        align_of::<H>(),
    )
    .expect("thin slice payload too large");
    let ptr = allocator.alloc_layout(layout);
    // SAFETY: `ptr` is a fresh allocation big enough for the header plus the payload,
    // aligned for both (asserted above); the regions cannot overlap the borrowed `items`.
    unsafe {
        let header_ptr = ptr.as_ptr().cast::<H>();
        header_ptr.write(header);
        let target = header_ptr.add(1).cast::<T>();
        ptr::copy_nonoverlapping(items.as_ptr(), target, items.len());
        &*header_ptr
    }
}

/// Reads back the `len` payload items following `header`.
///
/// # Safety
/// `header` must come from [`alloc_header_with_trailing`] called with `len` items of type `T`.
unsafe fn trailing<H, T>(header: &H, len: usize) -> &[T] {
    // SAFETY: `alloc_header_with_trailing` wrote `len` `T`s immediately after the header.
    unsafe {
        let ptr = (&raw const *header).add(1).cast::<T>();
        std::slice::from_raw_parts(ptr, len)
    }
}

/// Header of a thin slice: the payload follows immediately after it in the same arena
/// allocation (see [`alloc_header_with_trailing`]).
#[repr(C, align(8))]
pub(crate) struct ThinSliceHeader {
    len: u32,
}

impl ThinSliceHeader {
    fn alloc_in<'a, T>(items: &[T], allocator: &'a oxc_allocator::Allocator) -> &'a Self {
        let len = u32::try_from(items.len()).expect("thin slice longer than u32::MAX");
        alloc_header_with_trailing(Self { len }, items, allocator)
    }
}

/// Header of an arena text: `len` UTF-8 bytes follow immediately after it in the same
/// arena allocation, with the precomputed [`TextWidth`] kept alongside.
#[repr(C, align(4))]
pub(crate) struct ArenaTextHeader {
    width: TextWidth,
    len: u32,
}

/// A text stored in the arena with its metadata, thin-pointer sized (see `ArenaTextHeader`).
#[derive(Clone, Copy)]
pub struct ArenaText<'a>(&'a ArenaTextHeader);

impl<'a> ArenaText<'a> {
    pub(crate) fn alloc_in(
        text: &str,
        width: TextWidth,
        allocator: &'a oxc_allocator::Allocator,
    ) -> Self {
        let len = u32::try_from(text.len()).expect("text longer than u32::MAX");
        Self(alloc_header_with_trailing(ArenaTextHeader { width, len }, text.as_bytes(), allocator))
    }

    pub fn text(self) -> &'a str {
        // SAFETY: `alloc_in` stored `len` bytes of a valid `&str` after the header.
        unsafe { std::str::from_utf8_unchecked(trailing::<_, u8>(self.0, self.0.len as usize)) }
    }

    pub fn width(self) -> TextWidth {
        self.0.width
    }
}

impl PartialEq for ArenaText<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.width() == other.width() && self.text() == other.text()
    }
}

impl Eq for ArenaText<'_> {}

impl std::fmt::Debug for ArenaText<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.text().fmt(f)
    }
}

/// A shared, immutable slice of format elements, thin-pointer sized (see `ThinSliceHeader`).
#[derive(Clone, Copy)]
pub struct Interned<'a>(&'a ThinSliceHeader);

impl<'a> Interned<'a> {
    pub(crate) fn new_in(
        elements: &[FormatElement<'a>],
        allocator: &'a oxc_allocator::Allocator,
    ) -> Self {
        Self(ThinSliceHeader::alloc_in(elements, allocator))
    }

    fn as_slice(&self) -> &[FormatElement<'a>] {
        // SAFETY: `new_in` stored `len` elements after the header.
        unsafe { trailing(self.0, self.0.len as usize) }
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
        (&raw const *self.0).addr().hash(state);
    }
}

impl std::fmt::Debug for Interned<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(f)
    }
}

impl<'a> Deref for Interned<'a> {
    type Target = [FormatElement<'a>];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

const LINE_SEPARATOR: char = '\u{2028}';

const PARAGRAPH_SEPARATOR: char = '\u{2029}';

pub const LINE_TERMINATORS: [char; 3] = ['\r', LINE_SEPARATOR, PARAGRAPH_SEPARATOR];

/// Replace the line terminators matching the provided list with "\n"
/// since its the only line break type supported by the printer
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

impl<'a> FormatElement<'a> {
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
        matches!(
            self,
            FormatElement::SourceText { .. }
                | FormatElement::ArenaText(_)
                | FormatElement::Token { .. }
        )
    }

    /// Maximum byte length of an inline [`FormatElement::Token`].
    pub const INLINE_TOKEN_MAX: usize = 7;

    /// Builds a token element from a static ASCII string: inline if it fits
    /// [`FormatElement::INLINE_TOKEN_MAX`], otherwise copied into the arena.
    ///
    /// Prefer [`crate::builders::token`] in formatting code — it routes long tokens
    /// through the per-document cache instead of allocating per write.
    pub fn token(text: &'static str, allocator: &'a oxc_allocator::Allocator) -> Self {
        #[expect(clippy::cast_possible_truncation)]
        Self::token_inline(text).unwrap_or_else(|| {
            FormatElement::ArenaText(ArenaText::alloc_in(
                text,
                TextWidth::single(text.len() as u32),
                allocator,
            ))
        })
    }

    /// The inline form of [`FormatElement::token`]; `None` if `text` exceeds
    /// [`FormatElement::INLINE_TOKEN_MAX`].
    pub(crate) fn token_inline(text: &str) -> Option<Self> {
        let len = text.len();
        if len <= Self::INLINE_TOKEN_MAX {
            let mut bytes = [0u8; Self::INLINE_TOKEN_MAX];
            bytes[..len].copy_from_slice(text.as_bytes());
            #[expect(clippy::cast_possible_truncation)]
            Some(FormatElement::Token { len: len as u8, bytes })
        } else {
            None
        }
    }

    /// Builds the right text element for `text`: offset-based [`FormatElement::SourceText`]
    /// if it borrows from the document source and is printable ASCII, arena-copied otherwise.
    /// `width` is only evaluated for the non-ASCII/multiline arena fallback.
    pub fn text<C: crate::FormatContext>(
        text: &str,
        width: impl FnOnce() -> TextWidth,
        state: &crate::FormatState<'a, C>,
    ) -> Self {
        if is_all_printable_ascii(text) {
            #[expect(clippy::cast_possible_truncation)]
            let len = text.len() as u32;
            if let Some(offset) = state.source_offset_of(text) {
                FormatElement::SourceText { offset, len }
            } else {
                FormatElement::ArenaText(ArenaText::alloc_in(
                    text,
                    TextWidth::single(len),
                    state.allocator(),
                ))
            }
        } else {
            FormatElement::ArenaText(ArenaText::alloc_in(text, width(), state.allocator()))
        }
    }

    /// Builds an [`FormatElement::ArenaText`] by copying `text` into the arena together
    /// with its precomputed width. Use [`FormatElement::text`] when the text may borrow
    /// from the document source, and [`FormatElement::arena_text_measured`] when the
    /// width hasn't been computed yet.
    pub fn arena_text(
        text: &str,
        width: TextWidth,
        allocator: &'a oxc_allocator::Allocator,
    ) -> Self {
        FormatElement::ArenaText(ArenaText::alloc_in(text, width, allocator))
    }

    /// [`FormatElement::arena_text`] that measures the width itself, so callers can't
    /// pair a text with a stale width computed from a different string.
    pub fn arena_text_measured(
        text: &str,
        indent_width: IndentWidth,
        allocator: &'a oxc_allocator::Allocator,
    ) -> Self {
        Self::arena_text(text, TextWidth::from_text(text, indent_width), allocator)
    }

    /// The text of a [`FormatElement::Token`] / [`FormatElement::SourceText`] /
    /// [`FormatElement::ArenaText`] element, resolving source offsets against `source`
    /// (the source text of the document the element belongs to).
    /// Returns `None` for non-text elements.
    pub fn text_content<'s>(&'s self, source: &'s str) -> Option<&'s str> {
        match self {
            FormatElement::Token { .. } => Some(self.token_text()),
            _ => self.long_lived_text(source),
        }
    }

    /// Like [`FormatElement::text_content`], but only for the text variants whose text
    /// outlives the element itself (`SourceText` resolves into `source`, `ArenaText`
    /// into the arena). Returns `None` for inline tokens and non-text elements.
    pub fn long_lived_text<'s>(&self, source: &'s str) -> Option<&'s str>
    where
        'a: 's,
    {
        match self {
            FormatElement::SourceText { offset, len } => {
                Some(&source[*offset as usize..(*offset + *len) as usize])
            }
            FormatElement::ArenaText(text) => Some(text.text()),
            _ => None,
        }
    }

    /// The text of an inline [`FormatElement::Token`].
    ///
    /// # Panics
    /// Panics if `self` is not a [`FormatElement::Token`].
    #[inline]
    pub fn token_text(&self) -> &str {
        match self {
            FormatElement::Token { len, bytes } => {
                // SAFETY: `token` copies a prefix of a valid ASCII `&str` into `bytes`.
                unsafe { std::str::from_utf8_unchecked(&bytes[..*len as usize]) }
            }
            _ => panic!("`token_text` called on a non-token element"),
        }
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
            FormatElement::ArenaText(text) => text.width().propagates_expand(),
            FormatElement::Interned(interned) => interned.will_break(),
            // Traverse into the most flat version because the content is guaranteed to expand when even
            // the most flat version contains some content that forces a break.
            FormatElement::BestFitting(best_fitting) => best_fitting.most_flat().will_break(),
            // Tokens and source texts cannot contain line breaks
            FormatElement::Token { .. }
            | FormatElement::SourceText { .. }
            | FormatElement::LineSuffixBoundary
            | FormatElement::Space
            | FormatElement::Tag(_)
            | FormatElement::TailwindClass(_)
            | FormatElement::EmbedPlaceholder(_) => false,
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
/// Thin-pointer sized: the header's payload is the list of fat variant slices
/// (see `alloc_header_with_trailing`).
#[derive(Clone, Copy)]
pub struct BestFittingElement<'a>(&'a ThinSliceHeader);

impl PartialEq for BestFittingElement<'_> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.0, other.0)
    }
}

impl Eq for BestFittingElement<'_> {}

impl<'a> BestFittingElement<'a> {
    /// Creates a new best fitting IR with the given variants. The method itself isn't unsafe
    /// but it is to discourage people from using it because the printer will panic if
    /// the slice doesn't contain at least the least and most expanded variants.
    ///
    /// The first variant must be the one that takes up the most horizontal space (the most flat),
    /// the last the one that takes up the least (the most expanded).
    ///
    /// You're looking for a way to create a `BestFitting` object, use the `best_fitting![least_expanded, most_expanded]` macro.
    ///
    /// ## Safety
    /// The slice must contain at least two variants.
    #[doc(hidden)]
    pub unsafe fn from_slices_unchecked(
        variants: &[&'a [FormatElement<'a>]],
        allocator: &'a oxc_allocator::Allocator,
    ) -> Self {
        debug_assert!(
            variants.len() >= 2,
            "Requires at least the least expanded and most expanded variants"
        );

        Self(ThinSliceHeader::alloc_in(variants, allocator))
    }

    /// Returns the most expanded variant
    ///
    /// # Panics
    ///
    /// Panics if there are no variants. The constructor guarantees at least two variants.
    pub fn most_expanded(&self) -> &[FormatElement<'a>] {
        self.variants().last().expect(
            "Most contain at least two elements, as guaranteed by the best fitting builder.",
        )
    }

    /// Splits the variants into the most expanded and the remaining flat variants
    pub fn split_to_most_expanded_and_flat_variants(
        &self,
    ) -> (&&'a [FormatElement<'a>], &[&'a [FormatElement<'a>]]) {
        // SAFETY: We have already asserted that there are at least two variants for creating this struct.
        unsafe { self.variants().split_last().unwrap_unchecked() }
    }

    pub fn variants(self) -> &'a [&'a [FormatElement<'a>]] {
        // SAFETY: `from_slices_unchecked` stored `len` fat variant slices after the header.
        unsafe { trailing(self.0, self.0.len as usize) }
    }

    /// Returns the least expanded variant
    ///
    /// # Panics
    ///
    /// Panics if there are no variants. The constructor guarantees at least two variants.
    pub fn most_flat(&self) -> &[FormatElement<'a>] {
        self.variants().first().expect(
            "Most contain at least two elements, as guaranteed by the best fitting builder.",
        )
    }
}

impl std::fmt::Debug for BestFittingElement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.variants()).finish()
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
    fn start_tag(&self, kind: TagKind) -> Option<&Tag>;

    /// Returns the end tag if:
    /// * the last element is an end tag of `kind`
    fn end_tag(&self, kind: TagKind) -> Option<&Tag>;
}

/// New-type wrapper for a text Unicode width. Mainly to prevent access to the inner value.
///
/// ## Representation
///
/// Uses a single `u32` to efficiently store the width value and two flags:
///
/// - Bit 31: Multiline flag (1 = multiline, 0 = single line)
/// - Bit 30: No-expand flag (1 = multiline text that does not expand enclosing groups)
/// - Bits 0-29: Width value (stored directly)
///
/// This encoding allows `TextWidth` to fit in 4 bytes while supporting both a width value
/// and boolean flags. The maximum representable width is 2^30 - 1 (0x3FFFFFFF).
///
/// The maximum value is sufficient in practice as texts that long would exceed any
/// reasonable line width configuration (typically < 500 columns).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TextWidth(u32);

/// Every byte in `0x20..=0x7E` is a single-width ASCII character,
/// so display width equals byte length and the text is guaranteed single-line.
fn is_all_printable_ascii(s: &str) -> bool {
    s.as_bytes().iter().all(|&b| matches!(b, 0x20..=0x7e))
}

impl TextWidth {
    /// Bit mask for the multiline flag (highest bit)
    const MULTILINE_MASK: u32 = 1 << 31;

    /// Bit mask for the "multiline but does not expand enclosing groups" flag.
    /// Equivalent to Prettier's `literallineWithoutBreakParent`: the newlines still
    /// print literally, but `Document::propagate_expand` / `will_break` ignore them.
    const NO_EXPAND_MASK: u32 = 1 << 30;

    /// Bit mask for extracting the width value (all bits except the flags)
    const WIDTH_MASK: u32 = Self::NO_EXPAND_MASK - 1;

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
    /// NOTE: Uses `UnicodeWidthStr::width()` for accurate emoji sequence handling.
    /// Counting by `char` can lead to incorrect widths for complex Unicode sequences.
    /// e.g. "🗑️" (U+1F5D1 U+FE0F) is a single emoji with width 2, but counting chars gives width 1.
    #[expect(clippy::cast_possible_truncation)]
    pub fn from_text(text: &str, indent_width: IndentWidth) -> TextWidth {
        // Fast path for empty text
        if text.is_empty() {
            return Self::single(0);
        }

        // Excludes `\t`, `\n`, control bytes and multi-byte UTF-8,
        // those fall through to the scan below.
        if is_all_printable_ascii(text) {
            return Self::single(text.len() as u32);
        }

        let mut width = 0u32;
        let mut segment_start = 0;
        for (i, c) in text.char_indices() {
            match c {
                '\t' => {
                    width += text[segment_start..i].width() as u32;
                    width += u32::from(indent_width.value());
                    segment_start = i + 1; // Skip the tab character
                }
                '\n' => {
                    width += text[segment_start..i].width() as u32;
                    return Self::multiline(width);
                }
                _ => {}
            }
        }
        width += text[segment_start..].width() as u32;

        Self::single(width)
    }

    /// Creates width from a string known to not contain whitespace.
    /// More efficient than `from_text` when whitespace is guaranteed absent.
    #[expect(clippy::cast_possible_truncation)]
    pub fn from_non_whitespace_str(name: &str) -> TextWidth {
        if is_all_printable_ascii(name) {
            return Self::single(name.len() as u32);
        }
        Self::single(name.width() as u32)
    }

    /// Returns true if the text contains newlines.
    pub const fn is_multiline(self) -> bool {
        (self.0 & Self::MULTILINE_MASK) != 0
    }

    /// Marks a multiline width so its newlines do NOT expand enclosing groups
    /// (Prettier's `literallineWithoutBreakParent`). No-op for single-line widths.
    #[must_use]
    pub const fn without_expand_parent(self) -> Self {
        if self.is_multiline() { Self(self.0 | Self::NO_EXPAND_MASK) } else { self }
    }

    /// Returns true if the text forces enclosing groups to expand:
    /// it contains newlines and is not marked [`Self::without_expand_parent`].
    pub const fn propagates_expand(self) -> bool {
        (self.0 & (Self::MULTILINE_MASK | Self::NO_EXPAND_MASK)) == Self::MULTILINE_MASK
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
    fn without_expand_parent_disables_propagation_keeping_width() {
        let multi = TextWidth::multiline(3);
        debug_assert!(multi.propagates_expand());

        let no_expand = multi.without_expand_parent();
        debug_assert!(no_expand.is_multiline());
        debug_assert!(!no_expand.propagates_expand());
        debug_assert_eq!(no_expand.value(), 3);
    }

    #[test]
    fn without_expand_parent_is_noop_for_single_line() {
        let single = TextWidth::single(3);
        debug_assert!(!single.propagates_expand());
        debug_assert_eq!(single.without_expand_parent(), single);
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
    fn from_text_handles_emoji_sequences() {
        use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

        // Emoji with variation selector: 🗑️ = U+1F5D1 + U+FE0F
        let emoji = "🗑️";

        // Counting by char gives wrong width
        let wrong: usize = emoji.chars().filter_map(UnicodeWidthChar::width).sum();
        debug_assert_eq!(wrong, 1);
        // Need to count by str for correct width
        debug_assert_eq!(emoji.width(), 2);
        // Verify `TextWidth` also gets it right
        let width = TextWidth::from_text(emoji, indent_width(2));
        debug_assert_eq!(width.value(), 2);

        // Emoji with text
        let width = TextWidth::from_text("🗑️ DELETE", indent_width(2));
        debug_assert_eq!(width.value(), 9); // 2 (emoji) + 1 (space) + 6 (DELETE)

        // Another emoji with variation selector: ⚠️ = U+26A0 + U+FE0F
        let width = TextWidth::from_text("⚠️", indent_width(2));
        debug_assert_eq!(width.value(), 2);
    }

    #[test]
    fn from_text_empty_returns_zero() {
        let width = TextWidth::from_text("", indent_width(2));
        debug_assert_eq!(width.value(), 0);
        debug_assert!(!width.is_multiline());
    }

    #[test]
    fn ascii_fast_path_is_byte_identical_to_slow_path() {
        use unicode_width::UnicodeWidthStr;

        // Reference: the original `from_text` scan, without the ASCII fast path.
        #[expect(clippy::cast_possible_truncation)]
        fn from_text_slow(text: &str, indent_width: IndentWidth) -> TextWidth {
            if text.is_empty() {
                return TextWidth::single(0);
            }
            let mut width = 0u32;
            let mut segment_start = 0;
            for (i, c) in text.char_indices() {
                match c {
                    '\t' => {
                        width += text[segment_start..i].width() as u32;
                        width += u32::from(indent_width.value());
                        segment_start = i + 1;
                    }
                    '\n' => {
                        width += text[segment_start..i].width() as u32;
                        return TextWidth::multiline(width);
                    }
                    _ => {}
                }
            }
            width += text[segment_start..].width() as u32;
            TextWidth::single(width)
        }

        let cases = [
            "",
            "abc",
            "a b c",
            "className",
            "onClick",
            "~!@#$%^&*()_+-=[]{}|;':\",./<>?",
            " ",
            "  leading-and-trailing  ",
            "\t",
            "a\tb",
            "a\nb",
            "a\r\nb",
            "line1\nline2",
            "café",
            "日本語",
            "🗑️ DELETE",
            "⚠️",
            "a\u{0b}b", // vertical tab
            "a\u{0c}b", // form feed
            "a\u{7f}b", // DEL
            "\u{1f}",   // unit separator
            "mix café \t 日 \n end",
        ];
        let w = indent_width(4);
        for &s in &cases {
            debug_assert_eq!(
                TextWidth::from_text(s, w).0,
                from_text_slow(s, w).0,
                "from_text mismatch for {s:?}"
            );
        }

        // `from_non_whitespace_str` must equal an unconditional `UnicodeWidthStr::width`.
        let names = [
            "",
            "x",
            "className",
            "onClick",
            "snake_case_$id123",
            "~!@#",
            "café",
            "日本語",
            "🗑️",
            "a\u{7f}b",
            "\u{0b}",
        ];
        for &n in &names {
            #[expect(clippy::cast_possible_truncation)]
            let expected = TextWidth::single(n.width() as u32);
            debug_assert_eq!(
                TextWidth::from_non_whitespace_str(n).0,
                expected.0,
                "from_non_whitespace_str mismatch for {n:?}"
            );
        }
    }
}
