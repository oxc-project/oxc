use std::iter::FusedIterator;

use oxc_allocator::Vec as ArenaVec;
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};

use crate::QuoteStyle;
use crate::formatter::Comments;
use crate::{
    FormatResult,
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{Formatter, prelude::*},
    write,
};

pub static JSX_WHITESPACE_CHARS: [u8; 4] = [b' ', b'\n', b'\t', b'\r'];

/// Fast lookup table for JSX whitespace characters.
/// This is faster than checking against the array for repeated lookups.
static JSX_WHITESPACE_LOOKUP: [bool; 256] = {
    let mut table = [false; 256];
    table[b' ' as usize] = true;
    table[b'\n' as usize] = true;
    table[b'\t' as usize] = true;
    table[b'\r' as usize] = true;
    table
};

/// Meaningful JSX text is defined to be text that has either non-whitespace
/// characters, or does not contain a newline. Whitespace is defined as ASCII
/// whitespace.
///
/// ```
/// use oxc_formatter::utils::jsx::is_meaningful_jsx_text;
///
/// assert_eq!(is_meaningful_jsx_text("     \t\r   "), true);
/// assert_eq!(is_meaningful_jsx_text("     \n\r   "), false);
/// assert_eq!(is_meaningful_jsx_text("  Alien   "), true);
/// assert_eq!(is_meaningful_jsx_text("\n  Alien   "), true);
/// assert_eq!(is_meaningful_jsx_text("  Alien   \n"), true);
/// assert_eq!(is_meaningful_jsx_text(""), true);
/// ```
pub fn is_meaningful_jsx_text(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut has_newline = false;

    for &byte in bytes {
        // If there is a non-whitespace character
        if !JSX_WHITESPACE_LOOKUP[byte as usize] {
            return true;
        }
        // Track newlines (can use branchless or)
        has_newline |= byte == b'\n';
    }

    !has_newline
}

/// Indicates that an element should always be wrapped in parentheses, should be wrapped
/// only when it's line broken, or should not be wrapped at all.
#[derive(Copy, Clone, Debug)]
pub enum WrapState {
    /// For a JSX element that is never wrapped in parentheses.
    /// For instance, a JSX element that is another element's attribute
    /// should never be wrapped:
    /// ```jsx
    ///  <Route path="/" component={<HomePage />} />
    /// ```
    NoWrap,
    /// For a JSX element that must be wrapped in parentheses when line broken.
    /// For instance, a JSX element nested in a let binding is wrapped on line break:
    /// ```jsx
    ///  let component = <div> La Haine dir. Mathieu Kassovitz </div>;
    ///
    ///  let component = (
    ///   <div> Uncle Boonmee Who Can Recall His Past Lives dir. Apichatpong Weerasethakul </div>
    ///  );
    /// ```
    WrapOnBreak,
}

/// Checks if a JSX Element should be wrapped in parentheses. Returns a [WrapState] which
/// indicates when the element should be wrapped in parentheses.
pub fn get_wrap_state(parent: &AstNodes<'_>) -> WrapState {
    // Call site has ensures that only non-nested JSX elements are passed.
    debug_assert!(!matches!(parent, AstNodes::JSXElement(_) | AstNodes::JSXFragment(_)));

    match parent {
        AstNodes::ArrayExpression(_)
        | AstNodes::JSXAttribute(_)
        | AstNodes::JSXExpressionContainer(_)
        | AstNodes::ConditionalExpression(_) => WrapState::NoWrap,
        AstNodes::StaticMemberExpression(member) => {
            if member.optional {
                WrapState::NoWrap
            } else {
                WrapState::WrapOnBreak
            }
        }
        AstNodes::Argument(argument) if matches!(argument.parent, AstNodes::CallExpression(_)) => {
            WrapState::NoWrap
        }
        AstNodes::ExpressionStatement(stmt) => {
            // `() => <div></div>`
            //        ^^^^^^^^^^^
            if stmt.is_arrow_function_body() { WrapState::WrapOnBreak } else { WrapState::NoWrap }
        }
        AstNodes::ComputedMemberExpression(member) => {
            if member.optional {
                WrapState::NoWrap
            } else {
                WrapState::WrapOnBreak
            }
        }
        _ => WrapState::WrapOnBreak,
    }
}

/// Creates either a space using an expression child and a string literal,
/// or a regular space, depending on whether the group breaks or not.
///
/// ```jsx
///  <div> Winter Light </div>;
///
///  <div>
///    {" "}Winter Light
///    Through A Glass Darkly
///    The Silence
///    Seventh Seal
///    Wild Strawberries
///  </div>
/// ```
#[derive(Default)]
pub struct JsxSpace;

impl<'a> Format<'a> for JsxSpace {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                if_group_breaks(&format_args!(JsxRawSpace, soft_line_break())),
                if_group_fits_on_line(&space())
            ]
        )
    }
}

pub struct JsxRawSpace;

impl<'a> Format<'a> for JsxRawSpace {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let jsx_space = match f.options().quote_style {
            QuoteStyle::Double => r#"{" "}"#,
            QuoteStyle::Single => "{' '}",
        };

        write!(f, [token(jsx_space)])
    }
}

pub fn is_whitespace_jsx_expression<'a>(
    child: &JSXExpressionContainer<'a>,
    comments: &Comments<'a>,
) -> bool {
    match &child.expression {
        JSXExpression::StringLiteral(literal) => {
            matches!(literal.value.as_str(), " ") && !comments.has_comment_in_span(child.span)
        }
        _ => false,
    }
}

#[derive(Debug, Clone)]
pub enum JsxChild<'a, 'b> {
    /// A Single word in a JSX text. For example, the words for `a b\nc` are `[a, b, c]`
    Word(JsxWord<'a>),

    /// A ` ` or `${" "}` whitespace
    ///
    /// ```javascript
    /// <div> </div>
    /// <div>a </div>
    /// <div> a</div>
    /// <div>{' '}a</div>
    /// <div>a{' '}</div>
    /// <div>{' '}</div>
    /// <div>a
    /// {' '}b</div>
    /// ```
    ///
    /// Whitespace between two words is not represented as whitespace
    /// ```javascript
    /// <div>a b</div>
    /// ```
    /// The space between `a` and `b` is not considered a whitespace.
    Whitespace,

    /// A new line at the start or end of a JSXText with meaningful content. (that isn't all whitespace
    /// and contains a new line).
    ///
    /// ```javascript
    /// <div>
    ///     a
    /// </div>
    /// ```
    Newline,

    /// A JSXText that only consists of whitespace and has at least two line breaks;
    ///
    /// ```javascript
    /// <div>
    ///
    ///   <test />
    /// </div>
    /// ```
    ///
    /// The text between `<div>` and `<test />` is an empty line text.
    EmptyLine,

    /// Any other content that isn't a text. Should be formatted as is.
    NonText(&'b AstNode<'a, JSXChild<'a>>),
}

impl PartialEq for JsxChild<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Word(l0), Self::Word(r0)) => l0 == r0,
            (Self::NonText(_), Self::NonText(_)) => false, // Never equal by structural comparison
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Eq for JsxChild<'_, '_> {}

impl JsxChild<'_, '_> {
    pub const fn is_any_line(&self) -> bool {
        matches!(self, Self::EmptyLine | Self::Newline)
    }
}

/// A word in a Jsx Text. A word is string sequence that isn't separated by any JSX whitespace.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JsxWord<'a> {
    text: &'a str,
}

impl<'a> JsxWord<'a> {
    fn new(text: &'a str) -> Self {
        Self { text }
    }

    pub(crate) fn is_single_character(&self) -> bool {
        self.text.len() == 1
    }
}

impl<'a> Format<'a> for JsxWord<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [text_without_whitespace(self.text)])
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum JsxTextChunk<'a> {
    /// Whitespace chunk with newline count
    /// If newline_count > 0, the whitespace contains newlines
    Whitespace {
        newline_count: u8,
    },
    Word(&'a str),
}

/// Splits a text into whitespace only and non-whitespace chunks.
///
/// Uses byte-based iteration for better performance since JSX whitespace
/// characters are all single-byte ASCII characters.
///
/// See `jsx_split_chunks_iterator` test for examples
struct JsxSplitChunksIterator<'a> {
    position: usize,
    text: &'a str,
    bytes: &'a [u8],
}

impl<'a> JsxSplitChunksIterator<'a> {
    fn new(text: &'a str) -> Self {
        Self { position: 0, text, bytes: text.as_bytes() }
    }

    #[inline]
    fn is_jsx_whitespace(byte: u8) -> bool {
        JSX_WHITESPACE_LOOKUP[byte as usize]
    }
}

impl<'a> Iterator for JsxSplitChunksIterator<'a> {
    type Item = (usize, JsxTextChunk<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        // Early return if we've consumed all bytes
        if self.position >= self.bytes.len() {
            return None;
        }

        let start = self.position;
        let first_byte = self.bytes[start];
        let is_whitespace = Self::is_jsx_whitespace(first_byte);

        if is_whitespace {
            // For whitespace, count newlines as we scan
            let mut newline_count = u8::from(first_byte == b'\n');
            self.position += 1;

            while self.position < self.bytes.len() {
                let byte = self.bytes[self.position];
                if !Self::is_jsx_whitespace(byte) {
                    break;
                }
                if byte == b'\n' {
                    newline_count = newline_count.saturating_add(1);
                }
                self.position += 1;
            }

            Some((start, JsxTextChunk::Whitespace { newline_count }))
        } else {
            // For words, just scan until we hit whitespace
            self.position += 1;

            while self.position < self.bytes.len() {
                if Self::is_jsx_whitespace(self.bytes[self.position]) {
                    break;
                }
                self.position += 1;
            }

            let slice = &self.text[start..self.position];
            Some((start, JsxTextChunk::Word(slice)))
        }
    }
}

impl FusedIterator for JsxSplitChunksIterator<'_> {}

pub fn jsx_split_children<'a, 'b>(
    children: &'b AstNode<'a, ArenaVec<'a, JSXChild<'a>>>,
    comments: &Comments<'a>,
) -> Vec<JsxChild<'a, 'b>> {
    // Early return for empty children
    if children.is_empty() {
        return Vec::new();
    }

    let mut builder = JsxSplitChildrenBuilder::with_capacity(children.len());

    for child in children {
        match child.as_ref() {
            JSXChild::Text(text) => {
                // Split the text into words
                // Keep track if there's any leading/trailing empty line, new line or whitespace

                let text_value = &text.value;
                let mut chunks = JsxSplitChunksIterator::new(text_value).peekable();

                // Text starting with a whitespace
                if let Some((_, JsxTextChunk::Whitespace { .. })) = chunks.peek() {
                    match chunks.next() {
                        Some((_, JsxTextChunk::Whitespace { newline_count })) => {
                            if newline_count > 0 {
                                if chunks.peek().is_none() {
                                    // A text only consisting of whitespace that also contains a new line isn't considered meaningful text.
                                    // It can be entirely removed from the content without changing the semantics.

                                    // Keep up to one blank line between tags/expressions and text.
                                    // ```javascript
                                    // <div>
                                    //
                                    //   <MyElement />
                                    // </div>
                                    // ```
                                    if newline_count > 1 {
                                        builder.entry(JsxChild::EmptyLine);
                                    }

                                    continue;
                                }

                                builder.entry(JsxChild::Newline);
                            } else {
                                builder.entry(JsxChild::Whitespace);
                            }
                        }
                        _ => unreachable!(),
                    }
                }

                while let Some(chunk) = chunks.next() {
                    match chunk {
                        (_, JsxTextChunk::Whitespace { newline_count }) => {
                            // Only handle trailing whitespace. Words must always be joined by new lines
                            if chunks.peek().is_none() {
                                if newline_count > 0 {
                                    builder.entry(JsxChild::Newline);
                                } else {
                                    builder.entry(JsxChild::Whitespace);
                                }
                            }
                        }

                        (_, JsxTextChunk::Word(word)) => {
                            builder.entry(JsxChild::Word(JsxWord::new(word)));
                        }
                    }
                }
            }

            expr_container @ JSXChild::ExpressionContainer(container) => {
                if is_whitespace_jsx_expression(container.as_ref(), comments) {
                    builder.entry(JsxChild::Whitespace);
                } else {
                    builder.entry(JsxChild::NonText(child));
                }
            }
            _ => {
                builder.entry(JsxChild::NonText(child));
            }
        }
    }

    builder.finish()
}

/// The builder is used to:
/// 1. Remove [JsxChild::EmptyLine], [JsxChild::Newline], [JsxChild::Whitespace] if a next element is [JsxChild::Whitespace]
/// 2. Don't push a new element [JsxChild::EmptyLine], [JsxChild::Newline], [JsxChild::Whitespace] if previous one is [JsxChild::EmptyLine], [JsxChild::Newline], [JsxChild::Whitespace]
///
/// [Prettier applies]: https://github.com/prettier/prettier/blob/b0d9387b95cdd4e9d50f5999d3be53b0b5d03a97/src/language-js/print/jsx.js#L144-L180
#[derive(Debug)]
struct JsxSplitChildrenBuilder<'a, 'b> {
    buffer: Vec<JsxChild<'a, 'b>>,
}

impl<'a, 'b> JsxSplitChildrenBuilder<'a, 'b> {
    fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self { buffer: Vec::with_capacity(capacity) }
    }

    fn entry(&mut self, child: JsxChild<'a, 'b>) {
        match self.buffer.last_mut() {
            Some(last @ (JsxChild::EmptyLine | JsxChild::Newline | JsxChild::Whitespace)) => {
                if matches!(child, JsxChild::Whitespace) {
                    *last = child;
                } else if matches!(child, JsxChild::NonText(_) | JsxChild::Word(_)) {
                    self.buffer.push(child);
                }
            }
            _ => self.buffer.push(child),
        }
    }

    fn finish(self) -> Vec<JsxChild<'a, 'b>> {
        self.buffer
    }
}

/// An iterator adaptor that allows a lookahead of three tokens
///
/// # Examples
/// ```
/// use oxc_formatter::utils::jsx::JsxChildrenIterator;
///
/// let buffer = vec![1, 2, 3, 4];
///
/// let mut iter = JsxChildrenIterator::new(buffer.iter());
///
/// assert_eq!(iter.peek(), Some(&&1));
/// assert_eq!(iter.peek_next(), Some(&&2));
/// assert_eq!(iter.peek_next_next(), Some(&&3));
/// assert_eq!(iter.next(), Some(&1));
/// assert_eq!(iter.next(), Some(&2));
/// assert_eq!(iter.next(), Some(&3));
/// ```
#[derive(Clone, Debug)]
#[expect(clippy::option_option)]
pub struct JsxChildrenIterator<I: Iterator> {
    iter: I,

    peeked: Option<Option<I::Item>>,
    peeked_next: Option<Option<I::Item>>,
    peeked_next_next: Option<Option<I::Item>>,
}

impl<I: Iterator> JsxChildrenIterator<I> {
    pub fn new(iter: I) -> Self {
        Self { iter, peeked: None, peeked_next: None, peeked_next_next: None }
    }

    pub fn peek(&mut self) -> Option<&I::Item> {
        let iter = &mut self.iter;
        self.peeked.get_or_insert_with(|| iter.next()).as_ref()
    }

    pub fn peek_next(&mut self) -> Option<&I::Item> {
        let iter = &mut self.iter;
        let peeked = &mut self.peeked;

        self.peeked_next
            .get_or_insert_with(|| {
                peeked.get_or_insert_with(|| iter.next());
                iter.next()
            })
            .as_ref()
    }

    pub fn peek_next_next(&mut self) -> Option<&I::Item> {
        let iter = &mut self.iter;
        let peeked = &mut self.peeked;
        let peeked_next = &mut self.peeked_next;

        self.peeked_next_next
            .get_or_insert_with(|| {
                peeked.get_or_insert_with(|| iter.next());
                peeked_next.get_or_insert_with(|| iter.next());
                iter.next()
            })
            .as_ref()
    }
}

impl<I: Iterator> Iterator for JsxChildrenIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.peeked.take() {
            Some(peeked) => {
                self.peeked = self.peeked_next.take();
                self.peeked_next = self.peeked_next_next.take();
                peeked
            }
            None => self.iter.next(),
        }
    }
}
