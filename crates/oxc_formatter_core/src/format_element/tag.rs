use std::{
    cell::Cell,
    num::{NonZeroU8, NonZeroU32},
};

use crate::GroupId;

use super::PrintMode;

/// A Tag marking the start and end of some content to which some special formatting should be applied.
///
/// Tags always come in pairs of a start and an end tag and the styling defined by this tag
/// will be applied to all elements in between the start/end tags.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Tag {
    /// Indents the content one level deeper, see [crate::builders::indent] for documentation and examples.
    StartIndent,
    EndIndent,

    /// Variant of `Indent` that indents content by a number of spaces. For example, `Align(2)`
    /// indents any content following a line break by an additional two spaces.
    ///
    /// Nesting Aligns has the effect that all except the most inner align are handled as `Indent`.
    StartAlign(Align),
    EndAlign,

    /// Reduces the indention of the specified content either by one level or to the root, depending on the mode.
    /// Reverse operation of `Indent` and can be used to *undo* an `Align` for nested content.
    StartDedent(DedentMode),
    EndDedent(DedentMode),

    /// Creates a logical group where its content is either consistently printed:
    /// * on a single line: Omitting `LineMode::Soft` line breaks and printing spaces for `LineMode::SoftOrSpace`
    /// * on multiple lines: Printing all line breaks
    ///
    /// See [crate::builders::group] for documentation and examples.
    StartGroup(Group),
    EndGroup,

    /// Allows to specify content that gets printed depending on whatever the enclosing group
    /// is printed on a single line or multiple lines. See [crate::builders::if_group_breaks] for examples.
    StartConditionalContent(Condition),
    EndConditionalContent,

    /// Optimized version of [Tag::StartConditionalContent] for the case where some content
    /// should be indented if the specified group breaks.
    StartIndentIfGroupBreaks(GroupId),
    EndIndentIfGroupBreaks(GroupId),

    /// Concatenates multiple elements together with a given separator printed in either
    /// flat or expanded mode to fill the print width. Expect that the content is a list of alternating
    /// [element, separator] See [crate::Formatter::fill].
    StartFill,
    EndFill,

    /// Entry inside of a [Tag::StartFill]
    StartEntry,
    EndEntry,

    /// Delay the printing of its content until the next line break
    StartLineSuffix,
    EndLineSuffix,

    /// Special semantic element marking the content with a label.
    /// This does not directly influence how the content will be printed.
    ///
    /// See [crate::builders::labelled] for documentation.
    StartLabelled(LabelId),
    EndLabelled,

    /// Marks the current indention as the root that [LineMode::Literal](super::LineMode::Literal)
    /// line breaks and [DedentMode::Root] dedents return to.
    /// See [crate::builders::mark_as_root] for documentation.
    StartMarkAsRoot,
    EndMarkAsRoot,
}

impl Tag {
    /// Returns `true` if `self` is any start tag.
    pub const fn is_start(&self) -> bool {
        matches!(
            self,
            Tag::StartIndent
                | Tag::StartAlign(_)
                | Tag::StartDedent(_)
                | Tag::StartGroup { .. }
                | Tag::StartConditionalContent(_)
                | Tag::StartIndentIfGroupBreaks(_)
                | Tag::StartFill
                | Tag::StartEntry
                | Tag::StartLineSuffix
                | Tag::StartLabelled(_)
                | Tag::StartMarkAsRoot
        )
    }

    /// Returns `true` if `self` is any end tag.
    pub const fn is_end(&self) -> bool {
        !self.is_start()
    }

    pub const fn kind(&self) -> TagKind {
        use Tag::{
            EndAlign, EndConditionalContent, EndDedent, EndEntry, EndFill, EndGroup, EndIndent,
            EndIndentIfGroupBreaks, EndLabelled, EndLineSuffix, EndMarkAsRoot, StartAlign,
            StartConditionalContent, StartDedent, StartEntry, StartFill, StartGroup, StartIndent,
            StartIndentIfGroupBreaks, StartLabelled, StartLineSuffix, StartMarkAsRoot,
        };

        match self {
            StartIndent | EndIndent => TagKind::Indent,
            StartAlign(_) | EndAlign => TagKind::Align,
            StartDedent(_) | EndDedent(_) => TagKind::Dedent,
            StartGroup(_) | EndGroup => TagKind::Group,
            StartConditionalContent(_) | EndConditionalContent => TagKind::ConditionalContent,
            StartIndentIfGroupBreaks(_) | EndIndentIfGroupBreaks(_) => TagKind::IndentIfGroupBreaks,
            StartFill | EndFill => TagKind::Fill,
            StartEntry | EndEntry => TagKind::Entry,
            StartLineSuffix | EndLineSuffix => TagKind::LineSuffix,
            StartLabelled(_) | EndLabelled => TagKind::Labelled,
            StartMarkAsRoot | EndMarkAsRoot => TagKind::MarkAsRoot,
        }
    }
}

/// The kind of a [Tag].
///
/// Each start end tag pair has its own [tag kind](TagKind).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TagKind {
    Indent,
    Align,
    Dedent,
    Group,
    ConditionalContent,
    IndentIfGroupBreaks,
    Fill,
    Entry,
    LineSuffix,
    Labelled,
    TailwindClass,
    MarkAsRoot,
}

// The discriminants are the bit encoding used by `Group`'s packed representation.
#[derive(Debug, Copy, Default, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum GroupMode {
    /// Print group in flat mode.
    #[default]
    Flat = 0,

    /// The group should be printed in expanded mode
    Expand = 1,

    /// Expand mode has been propagated from an enclosing group to this group.
    Propagated = 2,
}

impl GroupMode {
    pub const fn is_flat(self) -> bool {
        matches!(self, GroupMode::Flat)
    }
}

/// Debug-name storage for a bit-packed group id: zero-sized in release builds,
/// mirroring how [`GroupId`] itself only carries its name in debug builds.
#[derive(Debug, Clone, Copy, Default)]
struct PackedIdName {
    #[cfg(debug_assertions)]
    name: Option<&'static str>,
}

impl PackedIdName {
    #[cfg_attr(not(debug_assertions), expect(unused_variables))]
    fn of(id: Option<GroupId>) -> Self {
        Self {
            #[cfg(debug_assertions)]
            name: id.map(GroupId::debug_name),
        }
    }

    #[cfg(debug_assertions)]
    fn get(self) -> &'static str {
        self.name.unwrap_or("group")
    }

    #[cfg(not(debug_assertions))]
    fn get(self) -> &'static str {
        "group"
    }
}

/// Bit position of the group id shared by [`Group`] and [`Condition`]:
/// the id occupies bits 2.., mode flags live in bits 0..2.
const PACKED_ID_SHIFT: u32 = 2;

/// Packs a group id into bits 2.. of a `u32` (0 = none).
/// The 30-bit limit is enforced where ids are minted ([`crate::UniqueGroupIdBuilder`]).
fn pack_group_id(id: Option<GroupId>) -> u32 {
    let id_bits = id.map_or(0, |id| id.value().get());
    debug_assert!(id_bits < (1 << 30), "group id exceeds the 30 bits available when packed");
    id_bits << PACKED_ID_SHIFT
}

fn unpack_group_id(packed: u32, name: PackedIdName) -> Option<GroupId> {
    NonZeroU32::new(packed >> PACKED_ID_SHIFT).map(|value| GroupId::from_value(value, name.get()))
}

/// Group id and mode, bit-packed to keep [`Tag`] (and with it [`super::FormatElement`]) small.
///
/// Bits 2.. hold the id value (see `pack_group_id`), bits 0..2 the [`GroupMode`].
/// Interior-mutable ([`Cell`]) so [`Group::propagate_expand`] can flip the mode through a `&self`
/// reference while the element sits in the document.
#[derive(Debug, Clone, Default)]
pub struct Group {
    packed: Cell<u32>,
    id_name: PackedIdName,
}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.packed == other.packed
    }
}

impl Eq for Group {}

impl Group {
    const MODE_MASK: u32 = 0b11;

    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_id(mut self, id: Option<GroupId>) -> Self {
        self.packed.set(pack_group_id(id) | (self.packed.get() & Self::MODE_MASK));
        self.id_name = PackedIdName::of(id);
        self
    }

    #[must_use]
    pub fn with_mode(self, mode: GroupMode) -> Self {
        self.set_mode(mode);
        self
    }

    fn set_mode(&self, mode: GroupMode) {
        self.packed.set((self.packed.get() & !Self::MODE_MASK) | mode as u32);
    }

    pub fn mode(&self) -> GroupMode {
        match self.packed.get() & Self::MODE_MASK {
            0 => GroupMode::Flat,
            1 => GroupMode::Expand,
            _ => GroupMode::Propagated,
        }
    }

    pub fn propagate_expand(&self) {
        if self.mode() == GroupMode::Flat {
            self.set_mode(GroupMode::Propagated);
        }
    }

    pub fn id(&self) -> Option<GroupId> {
        unpack_group_id(self.packed.get(), self.id_name)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DedentMode {
    /// Reduces the indent by a level (if the current indent is > 0)
    Level,

    /// Reduces the indent to the root
    Root,
}

/// Print-mode condition, bit-packed like [`Group`]: bit 0 holds the [`PrintMode`],
/// bits 2.. the referenced group id (0 = the enclosing group).
///
/// * `Flat` -> Omitted if the enclosing group is a multiline group, printed for groups fitting on a single line
/// * `Expanded` -> Omitted if the enclosing group fits on a single line, printed if the group breaks over multiple lines.
#[derive(Debug, Clone)]
pub struct Condition {
    packed: u32,
    id_name: PackedIdName,
}

impl PartialEq for Condition {
    fn eq(&self, other: &Self) -> bool {
        self.packed == other.packed
    }
}

impl Eq for Condition {}

impl Condition {
    const MODE_MASK: u32 = 0b1;

    pub fn new(mode: PrintMode) -> Self {
        Self {
            packed: match mode {
                PrintMode::Flat => 0,
                PrintMode::Expanded => 1,
            },
            id_name: PackedIdName::default(),
        }
    }

    #[must_use]
    pub fn with_group_id(mut self, id: Option<GroupId>) -> Self {
        self.packed = pack_group_id(id) | (self.packed & Self::MODE_MASK);
        self.id_name = PackedIdName::of(id);
        self
    }

    pub fn mode(&self) -> PrintMode {
        if self.packed & Self::MODE_MASK == 0 { PrintMode::Flat } else { PrintMode::Expanded }
    }

    pub fn group_id(&self) -> Option<GroupId> {
        unpack_group_id(self.packed, self.id_name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Align(pub(crate) NonZeroU8);

impl Align {
    pub fn new(count: NonZeroU8) -> Self {
        Self(count)
    }

    pub fn count(&self) -> NonZeroU8 {
        self.0
    }
}

#[derive(Debug, Eq, Copy, Clone)]
pub struct LabelId {
    // `u32` keeps `Tag` small; label ids are tiny per-language enum discriminants.
    value: u32,
    #[cfg(debug_assertions)]
    name: &'static str,
}

impl PartialEq for LabelId {
    fn eq(&self, other: &Self) -> bool {
        let is_equal = self.value == other.value;

        #[cfg(debug_assertions)]
        {
            if is_equal {
                assert_eq!(
                    self.name, other.name,
                    "Two `LabelId`s with different names have the same `value`. Are you mixing labels of two different `LabelDefinition` or are the values returned by the `LabelDefinition` not unique?"
                );
            }
        }

        is_equal
    }
}

impl LabelId {
    #[expect(clippy::needless_pass_by_value)] // The `Label` trait is unnecessary, would refactor it later.
    pub fn of<T: Label>(label: T) -> Self {
        let value = label.id();
        debug_assert!(u32::try_from(value).is_ok(), "label id exceeds `u32`");
        Self {
            #[expect(clippy::cast_possible_truncation)]
            value: value as u32,
            #[cfg(debug_assertions)]
            name: label.debug_name(),
        }
    }
}

/// Defines the valid labels of a language. You want to have at most one implementation per formatter
/// project.
pub trait Label {
    /// Returns the `u64` uniquely identifying this specific label.
    fn id(&self) -> u64;

    /// Returns the name of the label that is shown in debug builds.
    fn debug_name(&self) -> &'static str;
}
