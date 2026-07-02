//! Language-agnostic builders for constructing the formatter IR.
//!
//! These builders are generic over the format context type `C`, so they can be
//! reused by every language-specific formatter (JS/TS, CSS, JSON, ...).

use std::num::NonZeroU8;

use oxc_allocator::ArenaVec;

use crate::{
    Argument, Arguments, Buffer, Format, FormatContext, FormatElement, FormatOptions, Formatter,
    GroupId, VecBuffer,
    format::write,
    format_element::{
        self, LineMode, PrintMode, TextWidth,
        tag::{self, Condition, DedentMode, GroupMode, LabelId, Tag},
    },
};

use Tag::{
    EndAlign, EndConditionalContent, EndDedent, EndEntry, EndFill, EndGroup, EndIndent,
    EndIndentIfGroupBreaks, EndLabelled, EndLineSuffix, StartAlign, StartConditionalContent,
    StartDedent, StartEntry, StartFill, StartGroup, StartIndent, StartIndentIfGroupBreaks,
    StartLabelled, StartLineSuffix,
};

// ---------------------------------------------------------------------------
// Line
// ---------------------------------------------------------------------------

/// A line break that only gets printed if the enclosing `Group` doesn't fit on a single line.
#[inline]
pub const fn soft_line_break() -> Line {
    Line::new(LineMode::Soft)
}

/// A forced line break that is always printed.
#[inline]
pub const fn hard_line_break() -> Line {
    Line::new(LineMode::Hard)
}

/// A forced empty line.
#[inline]
pub const fn empty_line() -> Line {
    Line::new(LineMode::Empty)
}

/// A line break if the enclosing `Group` doesn't fit on a single line, a space otherwise.
#[inline]
pub const fn soft_line_break_or_space() -> Line {
    Line::new(LineMode::SoftOrSpace)
}

/// A forced line break that starts the next line at the marked root indention (Prettier's `literalline`).
///
/// Unlike [hard_line_break]:
/// - trailing whitespace on the current line is preserved (never trimmed)
/// - the newline always prints, even on an empty line
/// - the next line starts at the [mark_as_root] indention (column 0 when unmarked)
///   instead of the current indention
///
/// Used for verbatim multi-line content whose line structure is built element by element
/// (e.g. YAML block scalars). For verbatim content held as ONE string,
/// a multiline [text] already prints its embedded newlines with these semantics.
///
/// Known divergence from Prettier:
/// a [hard_line_break] directly after a COLUMN-0 literal line is absorbed
/// by the printer's "only print a newline if the line isn't already empty" rule (Prettier prints both newlines).
/// Use [empty_line] when the extra structural newline is required, it prints exactly one newline in this state.
#[inline]
pub const fn literal_line_break() -> Line {
    Line::new(LineMode::Literal)
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Line {
    mode: LineMode,
}

impl Line {
    const fn new(mode: LineMode) -> Self {
        Self { mode }
    }
}

impl<C> Format<'_, C> for Line {
    fn fmt(&self, f: &mut Formatter<'_, '_, C>) {
        f.write_element(FormatElement::Line(self.mode));
    }
}

impl std::fmt::Debug for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Line").field(&self.mode).finish()
    }
}

// ---------------------------------------------------------------------------
// FormatWith / FormatOnce — closure adapters
// ---------------------------------------------------------------------------
//
// These adapters live here so every language-specific formatter (JS/TS, JSON,
// CSS, ...) shares one definition and one `Format` impl. Each language exposes
// its own context-bound `format_with` / `format_once` wrapper around
// [`FormatWith::new`] / [`FormatOnce::new`] so closures can have their context
// inferred (a fully-generic constructor here would force every call site to
// annotate its closure parameter with the context type).

/// Adapter implementing [`Format`] for a closure. The closure is invoked every
/// time the value is formatted, so it must be re-entrant.
#[derive(Copy, Clone)]
pub struct FormatWith<T> {
    formatter: T,
}

impl<T> FormatWith<T> {
    /// Wraps `formatter` in a [`FormatWith`]. Typically called by a
    /// language-specific `format_with` helper that pins down the context type.
    #[inline]
    pub const fn new(formatter: T) -> Self {
        Self { formatter }
    }
}

impl<'ast, C, T> Format<'ast, C> for FormatWith<T>
where
    T: Fn(&mut Formatter<'_, 'ast, C>),
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        (self.formatter)(f);
    }
}

impl<T> std::fmt::Debug for FormatWith<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FormatWith").field(&"{{formatter}}").finish()
    }
}

/// Like [`FormatWith`] but consumes the closure on first format. Useful when the
/// closure can't be re-entered (e.g. it captures-by-move).
pub struct FormatOnce<T> {
    formatter: std::cell::Cell<Option<T>>,
}

impl<T> FormatOnce<T> {
    /// Wraps `formatter` in a [`FormatOnce`].
    #[inline]
    pub const fn new(formatter: T) -> Self {
        Self { formatter: std::cell::Cell::new(Some(formatter)) }
    }
}

impl<'ast, C, T> Format<'ast, C> for FormatOnce<T>
where
    T: FnOnce(&mut Formatter<'_, 'ast, C>),
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        let formatter = self.formatter.take().expect(
            "Tried to format a `FormatOnce` at least twice. \
             This is not allowed. Use `FormatWith` for re-entrant closures.",
        );
        (formatter)(f);
    }
}

impl<T> std::fmt::Debug for FormatOnce<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FormatOnce").field(&"{{formatter}}").finish()
    }
}

// ---------------------------------------------------------------------------
// Token
// ---------------------------------------------------------------------------

/// Creates a [`FormatElement::Token`] that gets written as is to the output.
///
/// # SAFETY
///
/// This function is safe to use only if the provided text contains no line breaks, tab characters,
/// or other non-ASCII characters.
#[inline]
pub fn token(text: &'static str) -> Token {
    debug_assert_token_ascii_only_and_no_linebreaks(text);
    Token { text }
}

fn debug_assert_token_ascii_only_and_no_linebreaks(text: &str) {
    debug_assert!(
        text.as_bytes().iter().all(|&c| c.is_ascii() && !matches!(c, b'\r' | b'\n' | b'\t')),
        "`FormatElement::Token` can only contain ASCII characters without line breaks or tab characters. Found invalid content: '{text}'"
    );
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Token {
    text: &'static str,
}

impl<C> Format<'_, C> for Token {
    fn fmt(&self, f: &mut Formatter<'_, '_, C>) {
        f.write_element(FormatElement::Token { text: self.text });
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "Token({})", self.text)
    }
}

// ---------------------------------------------------------------------------
// Text
// ---------------------------------------------------------------------------

/// Creates a text from a dynamic string and a range of the input source
pub fn text(text: &str) -> Text<'_> {
    debug_assert_no_cr_line_break(text);
    Text { text, width: None, expand_parent: true }
}

/// Creates a text from a dynamic string that contains no whitespace characters
pub fn text_without_whitespace(text: &str) -> Text<'_> {
    debug_assert!(
        text.as_bytes().iter().all(|&b| !b.is_ascii_whitespace()),
        "The content '{text}' contains whitespace characters but text must not contain any whitespace characters."
    );
    Text { text, width: Some(TextWidth::from_non_whitespace_str(text)), expand_parent: true }
}

#[derive(Eq, PartialEq)]
pub struct Text<'a> {
    #[expect(clippy::struct_field_names)] // Keep the name the same as it is in the original source
    text: &'a str,
    width: Option<TextWidth>,
    expand_parent: bool,
}

impl Text<'_> {
    /// Prints embedded newlines literally but does NOT force enclosing groups to expand
    /// (Prettier's `replaceEndOfLine(..., literallineWithoutBreakParent)`).
    ///
    /// Fits measurement still only counts the first line, and each newline resets the line width.
    #[must_use]
    pub fn without_expand_parent(mut self) -> Self {
        self.expand_parent = false;
        self
    }
}

impl<'a, C> Format<'a, C> for Text<'a>
where
    C: FormatContext,
{
    fn fmt(&self, f: &mut Formatter<'_, 'a, C>) {
        let width = self
            .width
            .unwrap_or_else(|| TextWidth::from_text(self.text, f.options().indent_width()));
        let width = if self.expand_parent { width } else { width.without_expand_parent() };
        f.write_element(FormatElement::Text { text: self.text, width });
    }
}

impl std::fmt::Debug for Text<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "Text({})", self.text)
    }
}

/// Debug assert that the given text contains no `\r` line terminator characters.
//
// `#[inline(always)]` because this is a no-op in release mode
#[inline(always)]
#[expect(clippy::inline_always)]
#[track_caller]
fn debug_assert_no_cr_line_break(text: &str) {
    debug_assert!(
        !text.contains('\r'),
        "The content `{text}` contains an unsupported `\\r` line terminator character but text must only use line feeds `\\n` as line separator. Use `\\n` instead of `\\r` and `\\r\\n` to insert a line break in strings."
    );
}

// ---------------------------------------------------------------------------
// Space / maybe_space
// ---------------------------------------------------------------------------

/// Inserts a single space.
#[inline]
pub const fn space() -> Space {
    Space
}

/// Optionally inserts a single space if the given condition is true.
#[inline]
pub fn maybe_space(should_insert: bool) -> Option<Space> {
    if should_insert { Some(Space) } else { None }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Space;

impl<C> Format<'_, C> for Space {
    fn fmt(&self, f: &mut Formatter<'_, '_, C>) {
        f.write_element(FormatElement::Space);
    }
}

// ---------------------------------------------------------------------------
// LineSuffix
// ---------------------------------------------------------------------------

/// Pushes some content to the end of the current line.
#[inline]
pub fn line_suffix<'a, 'ast, C, Content>(inner: &'a Content) -> LineSuffix<'a, 'ast, C>
where
    Content: Format<'ast, C>,
{
    LineSuffix { content: Argument::new(inner) }
}

#[derive(Copy, Clone)]
pub struct LineSuffix<'a, 'ast, C> {
    content: Argument<'a, 'ast, C>,
}

impl<'ast, C> Format<'ast, C> for LineSuffix<'_, 'ast, C> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        f.write_element(FormatElement::Tag(StartLineSuffix));
        Arguments::from(&self.content).fmt(f);
        f.write_element(FormatElement::Tag(EndLineSuffix));
    }
}

impl<C> std::fmt::Debug for LineSuffix<'_, '_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LineSuffix").field(&"{{content}}").finish()
    }
}

// ---------------------------------------------------------------------------
// LineSuffixBoundary
// ---------------------------------------------------------------------------

/// Inserts a boundary for line suffixes.
pub const fn line_suffix_boundary() -> LineSuffixBoundary {
    LineSuffixBoundary
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LineSuffixBoundary;

impl<C> Format<'_, C> for LineSuffixBoundary {
    fn fmt(&self, f: &mut Formatter<'_, '_, C>) {
        f.write_element(FormatElement::LineSuffixBoundary);
    }
}

// ---------------------------------------------------------------------------
// FormatLabelled
// ---------------------------------------------------------------------------

/// Marks some content with a label.
#[inline]
pub fn labelled<'a, 'ast, C, Content>(
    label_id: LabelId,
    content: &'a Content,
) -> FormatLabelled<'a, 'ast, C>
where
    Content: Format<'ast, C>,
{
    FormatLabelled { label_id, content: Argument::new(content) }
}

#[derive(Copy, Clone)]
pub struct FormatLabelled<'a, 'ast, C> {
    label_id: LabelId,
    content: Argument<'a, 'ast, C>,
}

impl<'ast, C> Format<'ast, C> for FormatLabelled<'_, 'ast, C> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        f.write_element(FormatElement::Tag(StartLabelled(self.label_id)));
        Arguments::from(&self.content).fmt(f);
        f.write_element(FormatElement::Tag(EndLabelled));
    }
}

impl<C> std::fmt::Debug for FormatLabelled<'_, '_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Label").field(&self.label_id).field(&"{{content}}").finish()
    }
}

// ---------------------------------------------------------------------------
// Indent
// ---------------------------------------------------------------------------

/// Adds a level of indentation to the given content.
#[inline]
pub fn indent<'a, 'ast, C, Content>(content: &'a Content) -> Indent<'a, 'ast, C>
where
    Content: Format<'ast, C>,
{
    Indent { content: Argument::new(content) }
}

#[derive(Copy, Clone)]
pub struct Indent<'a, 'ast, C> {
    content: Argument<'a, 'ast, C>,
}

impl<'ast, C> Format<'ast, C> for Indent<'_, 'ast, C> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        f.write_element(FormatElement::Tag(StartIndent));

        let elements_length = f.elements().len();

        Arguments::from(&self.content).fmt(f);

        debug_assert_ne!(
            elements_length,
            f.elements().len(),
            "Indent's content must produce at least one element"
        );

        f.write_element(FormatElement::Tag(EndIndent));
    }
}

impl<C> std::fmt::Debug for Indent<'_, '_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Indent").field(&"{{content}}").finish()
    }
}

// ---------------------------------------------------------------------------
// Dedent
// ---------------------------------------------------------------------------

/// Reduces the indentation for the given content.
#[inline]
pub fn dedent<'ast, C, Content>(content: &Content) -> Dedent<'_, 'ast, C>
where
    Content: Format<'ast, C>,
{
    Dedent { content: Argument::new(content), mode: DedentMode::Level }
}

/// Resets the indention so that the content prints at the [mark_as_root] indention,
/// or at the start of the line when no `mark_as_root` is active (Prettier's `dedentToRoot`).
#[inline]
pub fn dedent_to_root<'ast, C, Content>(content: &Content) -> Dedent<'_, 'ast, C>
where
    Content: Format<'ast, C>,
{
    Dedent { content: Argument::new(content), mode: DedentMode::Root }
}

/// Marks the current indention as the root that [literal_line_break] and [dedent_to_root]
/// inside `content` return to (Prettier's `markAsRoot`).
///
/// Without an enclosing `mark_as_root`, the root is column 0.
/// e.g. YAML block scalars wrap each line boundary in `mark_as_root(&literal_line_break())`
/// so continuation lines keep the block's base indention.
#[inline]
pub fn mark_as_root<'ast, C, Content>(content: &Content) -> MarkAsRoot<'_, 'ast, C>
where
    Content: Format<'ast, C>,
{
    MarkAsRoot { content: Argument::new(content) }
}

#[derive(Copy, Clone)]
pub struct MarkAsRoot<'a, 'ast, C> {
    content: Argument<'a, 'ast, C>,
}

impl<'ast, C> Format<'ast, C> for MarkAsRoot<'_, 'ast, C> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        f.write_element(FormatElement::Tag(Tag::StartMarkAsRoot));
        Arguments::from(&self.content).fmt(f);
        f.write_element(FormatElement::Tag(Tag::EndMarkAsRoot));
    }
}

impl<C> std::fmt::Debug for MarkAsRoot<'_, '_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MarkAsRoot").field(&"{{content}}").finish()
    }
}

#[derive(Copy, Clone)]
pub struct Dedent<'a, 'ast, C> {
    content: Argument<'a, 'ast, C>,
    mode: DedentMode,
}

impl<'ast, C> Format<'ast, C> for Dedent<'_, 'ast, C> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        f.write_element(FormatElement::Tag(StartDedent(self.mode)));
        Arguments::from(&self.content).fmt(f);
        f.write_element(FormatElement::Tag(EndDedent(self.mode)));
    }
}

impl<C> std::fmt::Debug for Dedent<'_, '_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Dedent").field(&"{{content}}").finish()
    }
}

// ---------------------------------------------------------------------------
// Align
// ---------------------------------------------------------------------------

/// Aligns its content by indenting the content by `count` spaces.
///
/// # Panics
///
/// Panics if `count` is `0`.
pub fn align<'ast, C, Content>(count: u8, content: &Content) -> Align<'_, 'ast, C>
where
    Content: Format<'ast, C>,
{
    Align {
        count: NonZeroU8::new(count).expect("Alignment count must be a non-zero number."),
        content: Argument::new(content),
    }
}

#[derive(Copy, Clone)]
pub struct Align<'a, 'ast, C> {
    count: NonZeroU8,
    content: Argument<'a, 'ast, C>,
}

impl<'ast, C> Format<'ast, C> for Align<'_, 'ast, C> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        f.write_element(FormatElement::Tag(StartAlign(tag::Align(self.count))));
        Arguments::from(&self.content).fmt(f);
        f.write_element(FormatElement::Tag(EndAlign));
    }
}

impl<C> std::fmt::Debug for Align<'_, '_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Align")
            .field("count", &self.count)
            .field("content", &"{{content}}")
            .finish()
    }
}

// ---------------------------------------------------------------------------
// BlockIndent
// ---------------------------------------------------------------------------

/// Inserts a hard line break before and after the content and increases the indention level by one.
#[inline]
pub fn block_indent<'ast, C>(content: &impl Format<'ast, C>) -> BlockIndent<'_, 'ast, C> {
    BlockIndent { content: Argument::new(content), mode: IndentMode::Block }
}

/// Indents the content by inserting a line break before and after the content and increasing
/// the indention level for the content by one if the enclosing group doesn't fit on a single line.
#[inline]
pub fn soft_block_indent<'ast, C>(content: &impl Format<'ast, C>) -> BlockIndent<'_, 'ast, C> {
    BlockIndent { content: Argument::new(content), mode: IndentMode::Soft }
}

/// Conditionally adds spaces around the content if its enclosing group fits within a single line.
pub fn soft_block_indent_with_maybe_space<'ast, C>(
    content: &impl Format<'ast, C>,
    should_add_space: bool,
) -> BlockIndent<'_, 'ast, C> {
    if should_add_space { soft_space_or_block_indent(content) } else { soft_block_indent(content) }
}

/// If the enclosing `Group` doesn't fit on a single line, inserts a line break and indent.
/// Otherwise, just inserts a space.
#[inline]
pub fn soft_line_indent_or_space<'ast, C>(
    content: &impl Format<'ast, C>,
) -> BlockIndent<'_, 'ast, C> {
    BlockIndent { content: Argument::new(content), mode: IndentMode::SoftLineOrSpace }
}

/// Adds spaces around the content if its enclosing group fits on a line, otherwise indents the content and separates it by line breaks.
pub fn soft_space_or_block_indent<'ast, C>(
    content: &impl Format<'ast, C>,
) -> BlockIndent<'_, 'ast, C> {
    BlockIndent { content: Argument::new(content), mode: IndentMode::SoftSpace }
}

#[derive(Copy, Clone)]
pub struct BlockIndent<'fmt, 'ast, C> {
    content: Argument<'fmt, 'ast, C>,
    mode: IndentMode,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum IndentMode {
    Soft,
    Block,
    SoftSpace,
    SoftLineOrSpace,
}

impl<'ast, C> Format<'ast, C> for BlockIndent<'_, 'ast, C> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        f.write_element(FormatElement::Tag(StartIndent));

        match self.mode {
            IndentMode::Soft => {
                let line = soft_line_break();
                write(f, Arguments::new(&[Argument::new(&line)]));
            }
            IndentMode::Block => {
                let line = hard_line_break();
                write(f, Arguments::new(&[Argument::new(&line)]));
            }
            IndentMode::SoftLineOrSpace | IndentMode::SoftSpace => {
                let line = soft_line_break_or_space();
                write(f, Arguments::new(&[Argument::new(&line)]));
            }
        }

        let elements_length = f.elements().len();

        Arguments::from(&self.content).fmt(f);

        debug_assert_ne!(
            elements_length,
            f.elements().len(),
            "BlockIndent's content must produce at least one element"
        );

        f.write_element(FormatElement::Tag(EndIndent));

        match self.mode {
            IndentMode::Soft => {
                let line = soft_line_break();
                write(f, Arguments::new(&[Argument::new(&line)]));
            }
            IndentMode::Block => {
                let line = hard_line_break();
                write(f, Arguments::new(&[Argument::new(&line)]));
            }
            IndentMode::SoftSpace => {
                let line = soft_line_break_or_space();
                write(f, Arguments::new(&[Argument::new(&line)]));
            }
            IndentMode::SoftLineOrSpace => (),
        }
    }
}

impl<C> std::fmt::Debug for BlockIndent<'_, '_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self.mode {
            IndentMode::Soft => "SoftBlockIndent",
            IndentMode::Block => "HardBlockIndent",
            IndentMode::SoftLineOrSpace => "SoftLineIndentOrSpace",
            IndentMode::SoftSpace => "SoftSpaceBlockIndent",
        };

        f.debug_tuple(name).field(&"{{content}}").finish()
    }
}

// ---------------------------------------------------------------------------
// Group
// ---------------------------------------------------------------------------

/// Creates a logical `Group` around the content.
#[inline]
pub fn group<'ast, C>(content: &impl Format<'ast, C>) -> Group<'_, 'ast, C> {
    Group { content: Argument::new(content), group_id: None, should_expand: false }
}

#[derive(Copy, Clone)]
pub struct Group<'fmt, 'ast, C> {
    content: Argument<'fmt, 'ast, C>,
    #[expect(clippy::struct_field_names)] // Keep the name the same as it is in the original source
    group_id: Option<GroupId>,
    should_expand: bool,
}

impl<C> Group<'_, '_, C> {
    #[must_use]
    pub fn with_group_id(mut self, group_id: Option<GroupId>) -> Self {
        self.group_id = group_id;
        self
    }

    /// Changes the [PrintMode] of the group from `Flat` to `Expanded`.
    #[must_use]
    pub fn should_expand(mut self, should_expand: bool) -> Self {
        self.should_expand = should_expand;
        self
    }
}

impl<'ast, C> Format<'ast, C> for Group<'_, 'ast, C> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        let mode = if self.should_expand { GroupMode::Expand } else { GroupMode::Flat };

        f.write_element(FormatElement::Tag(StartGroup(
            tag::Group::new().with_id(self.group_id).with_mode(mode),
        )));

        Arguments::from(&self.content).fmt(f);

        f.write_element(FormatElement::Tag(EndGroup));
    }
}

impl<C> std::fmt::Debug for Group<'_, '_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroupElements")
            .field("group_id", &self.group_id)
            .field("should_expand", &self.should_expand)
            .field("content", &"{{content}}")
            .finish()
    }
}

// ---------------------------------------------------------------------------
// ExpandParent
// ---------------------------------------------------------------------------

/// IR element that forces the parent group to print in expanded mode.
pub const fn expand_parent() -> ExpandParent {
    ExpandParent
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ExpandParent;

impl<C> Format<'_, C> for ExpandParent {
    fn fmt(&self, f: &mut Formatter<'_, '_, C>) {
        f.write_element(FormatElement::ExpandParent);
    }
}

// ---------------------------------------------------------------------------
// IfGroupBreaks
// ---------------------------------------------------------------------------

/// Adds a conditional content that is emitted only if it isn't inside an enclosing `Group` that
/// is printed on a single line.
#[inline]
pub fn if_group_breaks<'ast, C, Content>(content: &Content) -> IfGroupBreaks<'_, 'ast, C>
where
    Content: Format<'ast, C>,
{
    IfGroupBreaks { content: Argument::new(content), group_id: None, mode: PrintMode::Expanded }
}

/// Adds a conditional content specific for `Group`s that fit on a single line.
#[inline]
pub fn if_group_fits_on_line<'ast, C, Content>(flat_content: &Content) -> IfGroupBreaks<'_, 'ast, C>
where
    Content: Format<'ast, C>,
{
    IfGroupBreaks { mode: PrintMode::Flat, group_id: None, content: Argument::new(flat_content) }
}

#[derive(Copy, Clone)]
pub struct IfGroupBreaks<'a, 'ast, C> {
    content: Argument<'a, 'ast, C>,
    group_id: Option<GroupId>,
    mode: PrintMode,
}

impl<C> IfGroupBreaks<'_, '_, C> {
    /// Inserts some content that the printer only prints if the group with the specified `group_id`
    /// is printed in multiline mode.
    #[must_use]
    pub fn with_group_id(mut self, group_id: Option<GroupId>) -> Self {
        self.group_id = group_id;
        self
    }
}

impl<'ast, C> Format<'ast, C> for IfGroupBreaks<'_, 'ast, C> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        f.write_element(FormatElement::Tag(StartConditionalContent(
            Condition::new(self.mode).with_group_id(self.group_id),
        )));
        self.content.fmt(f);
        f.write_element(FormatElement::Tag(EndConditionalContent));
    }
}

impl<C> std::fmt::Debug for IfGroupBreaks<'_, '_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self.mode {
            PrintMode::Flat => "IfGroupFitsOnLine",
            PrintMode::Expanded => "IfGroupBreaks",
        };

        f.debug_struct(name)
            .field("group_id", &self.group_id)
            .field("content", &"{{content}}")
            .finish()
    }
}

// ---------------------------------------------------------------------------
// IndentIfGroupBreaks
// ---------------------------------------------------------------------------

/// Increases the indent level by one if the group with the specified id breaks.
#[inline]
pub fn indent_if_group_breaks<'a, 'ast, C, Content>(
    content: &'a Content,
    group_id: GroupId,
) -> IndentIfGroupBreaks<'a, 'ast, C>
where
    Content: Format<'ast, C>,
{
    IndentIfGroupBreaks { group_id, content: Argument::new(content) }
}

#[derive(Copy, Clone)]
pub struct IndentIfGroupBreaks<'a, 'ast, C> {
    content: Argument<'a, 'ast, C>,
    group_id: GroupId,
}

impl<'ast, C> Format<'ast, C> for IndentIfGroupBreaks<'_, 'ast, C> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        f.write_element(FormatElement::Tag(StartIndentIfGroupBreaks(self.group_id)));
        Arguments::from(&self.content).fmt(f);
        f.write_element(FormatElement::Tag(EndIndentIfGroupBreaks(self.group_id)));
    }
}

impl<C> std::fmt::Debug for IndentIfGroupBreaks<'_, '_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndentIfGroupBreaks")
            .field("group_id", &self.group_id)
            .field("content", &"{{content}}")
            .finish()
    }
}

// ---------------------------------------------------------------------------
// BestFitting
// ---------------------------------------------------------------------------

/// The first variant is the most flat, and the last is the most expanded variant.
/// See [`best_fitting!`] macro for a more in-detail documentation
#[derive(Copy, Clone)]
pub struct BestFitting<'fmt, 'ast, C> {
    variants: Arguments<'fmt, 'ast, C>,
}

impl<'fmt, 'ast, C> BestFitting<'fmt, 'ast, C> {
    /// Creates a new best fitting IR with the given variants. The method itself isn't unsafe
    /// but it is to discourage people from using it because the printer will panic if
    /// the slice doesn't contain at least the least and most expanded variants.
    ///
    /// You're looking for a way to create a `BestFitting` object, use the `best_fitting![least_expanded, most_expanded]` macro.
    ///
    /// ## Safety
    /// The slice must contain at least two variants.
    #[doc(hidden)]
    pub fn from_arguments_unchecked(variants: Arguments<'fmt, 'ast, C>) -> Self {
        assert!(
            variants.0.len() >= 2,
            "Requires at least the least expanded and most expanded variants"
        );

        Self { variants }
    }
}

impl<'ast, C> Format<'ast, C> for BestFitting<'_, 'ast, C> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        let mut buffer = VecBuffer::new(f.state_mut());
        let variants = self.variants.items();

        let mut formatted_variants = Vec::with_capacity(variants.len());

        for variant in variants {
            buffer.write_element(FormatElement::Tag(StartEntry));
            buffer.write_fmt(Arguments::from(variant));
            buffer.write_element(FormatElement::Tag(EndEntry));

            formatted_variants.push(buffer.take_vec().into_arena_slice());
        }

        let formatted_variants = ArenaVec::from_iter_in(formatted_variants, f);

        // SAFETY: The constructor guarantees that there are always at least two variants. It's, therefore,
        // safe to call into the unsafe `from_vec_unchecked` function
        let element = unsafe {
            FormatElement::BestFitting(format_element::BestFittingElement::from_vec_unchecked(
                formatted_variants,
            ))
        };

        f.write_element(element);
    }
}

// ---------------------------------------------------------------------------
// JoinBuilder
// ---------------------------------------------------------------------------

/// Builder to join together a sequence of content.
/// See [Formatter::join]
#[must_use = "must eventually call `finish()` on Format builders"]
pub struct JoinBuilder<'fmt, 'buf, 'ast, Separator, C> {
    pub(crate) fmt: &'fmt mut Formatter<'buf, 'ast, C>,
    with: Option<Separator>,
    has_elements: bool,
}

impl<'fmt, 'buf, 'ast, Separator, C> JoinBuilder<'fmt, 'buf, 'ast, Separator, C>
where
    Separator: Format<'ast, C>,
{
    /// Creates a new instance that joins the elements without a separator
    pub fn new(fmt: &'fmt mut Formatter<'buf, 'ast, C>) -> Self {
        Self { fmt, has_elements: false, with: None }
    }

    /// Creates a new instance that prints the passed separator between every two entries.
    pub fn with_separator(fmt: &'fmt mut Formatter<'buf, 'ast, C>, with: Separator) -> Self {
        Self { fmt, has_elements: false, with: Some(with) }
    }

    /// Adds a new entry to the join output.
    pub fn entry(&mut self, entry: &dyn Format<'ast, C>) -> &mut Self {
        if let Some(with) = &self.with
            && self.has_elements
        {
            with.fmt(self.fmt);
        }
        self.has_elements = true;

        entry.fmt(self.fmt);

        self
    }

    /// Adds the contents of an iterator of entries to the join output.
    pub fn entries<F, I>(&mut self, entries: I) -> &mut Self
    where
        F: Format<'ast, C>,
        I: IntoIterator<Item = F>,
    {
        for entry in entries {
            self.entry(&entry);
        }

        self
    }

    #[expect(clippy::unused_self)]
    pub fn finish(self) {}
}

// ---------------------------------------------------------------------------
// FillBuilder
// ---------------------------------------------------------------------------

/// Builder to fill as many elements as possible on a single line.
#[must_use = "must eventually call `finish()` on Format builders"]
pub struct FillBuilder<'fmt, 'buf, 'ast, C> {
    fmt: &'fmt mut Formatter<'buf, 'ast, C>,
    empty: bool,
}

impl<'fmt, 'buf, 'ast, C> FillBuilder<'fmt, 'buf, 'ast, C> {
    pub fn new(fmt: &'fmt mut Formatter<'buf, 'ast, C>) -> Self {
        fmt.write_element(FormatElement::Tag(StartFill));

        Self { fmt, empty: true }
    }

    /// Adds an iterator of entries to the fill output. Uses the passed `separator` to separate any two items.
    pub fn entries<F, I>(&mut self, separator: &dyn Format<'ast, C>, entries: I) -> &mut Self
    where
        F: Format<'ast, C>,
        I: IntoIterator<Item = F>,
    {
        for entry in entries {
            self.entry(separator, &entry);
        }

        self
    }

    /// Adds a new entry to the fill output. The `separator` isn't written if this is the first element in the list.
    pub fn entry(
        &mut self,
        separator: &dyn Format<'ast, C>,
        entry: &dyn Format<'ast, C>,
    ) -> &mut Self {
        if self.empty {
            self.empty = false;
        } else {
            self.fmt.write_element(FormatElement::Tag(StartEntry));
            separator.fmt(self.fmt);
            self.fmt.write_element(FormatElement::Tag(EndEntry));
        }

        self.fmt.write_element(FormatElement::Tag(StartEntry));
        entry.fmt(self.fmt);
        self.fmt.write_element(FormatElement::Tag(EndEntry));

        self
    }

    /// Finishes the output and returns any error encountered
    pub fn finish(&mut self) {
        self.fmt.write_element(FormatElement::Tag(EndFill));
    }
}
