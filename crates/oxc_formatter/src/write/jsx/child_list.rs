use oxc_allocator::{Allocator, TakeIn, Vec as ArenaVec};
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::AstNode,
    format_args,
    formatter::{
        Comments, FormatElement, Formatter, VecBuffer,
        prelude::{tag::GroupMode, *},
    },
    utils::{
        jsx::{
            JsxChild, JsxChildrenIterator, JsxRawSpace, JsxSpace, is_meaningful_jsx_text,
            is_whitespace_jsx_expression, jsx_split_children,
        },
        suppressed::FormatSuppressedNode,
    },
    write,
};
use std::cell::RefCell;

#[derive(Debug, Clone, Default)]
pub struct FormatJsxChildList {
    layout: JsxChildListLayout,
}

impl FormatJsxChildList {
    pub fn with_options(mut self, options: JsxChildListLayout) -> Self {
        self.layout = options;
        self
    }

    pub fn fmt_children<'a, 'b>(
        &self,
        children: &'b AstNode<'a, ArenaVec<'a, JSXChild<'a>>>,
        f: &mut Formatter<'_, 'a>,
    ) -> FormatChildrenResult<'a, 'b> {
        // Use Biome's exact approach - no need for jsx_split_children at this stage
        let children_meta = Self::children_meta(children, f.context().comments());
        let layout = self.layout(children_meta);

        let multiline_layout = if children_meta.meaningful_text {
            MultilineLayout::Fill
        } else {
            MultilineLayout::NoFill
        };

        let mut force_multiline = layout.is_multiline();
        let mut flat = FlatBuilder::new(force_multiline, f.context().allocator());
        let mut multiline = MultilineBuilder::new(multiline_layout, f.context().allocator());

        let mut children = jsx_split_children(children, f.context().comments());

        // Trim trailing new lines
        if let Some(JsxChild::EmptyLine | JsxChild::Newline) = children.last() {
            children.pop();
        }

        if children.len() == 1 {
            return FormatChildrenResult::SingleChild(FormatSingleChild {
                child: children.pop().unwrap(),
                force_multiline,
            });
        }

        let mut is_next_child_suppressed = false;
        let mut last: Option<&JsxChild> = None;
        let mut children_iter = JsxChildrenIterator::new(children.iter());

        // Trim leading new lines
        if let Some(JsxChild::Newline | JsxChild::EmptyLine) = children_iter.peek() {
            children_iter.next();
        }

        while let Some(child) = children_iter.next() {
            let mut child_breaks = false;

            match &child {
                // A single word: Both `a` and `b` are a word in `a b` because they're separated by JSX Whitespace.
                JsxChild::Word(word) => {
                    let separator = match children_iter.peek() {
                        Some(JsxChild::Word(_)) => {
                            // Separate words by a space or line break in extended mode
                            Some(WordSeparator::BetweenWords)
                        }

                        // Last word or last word before an element without any whitespace in between
                        Some(JsxChild::NonText(next_child)) => Some(WordSeparator::EndOfText {
                            is_soft_line_break: !matches!(
                                next_child.as_ref(),
                                JSXChild::Element(element) if element.closing_element.is_none()
                            ) || word.is_single_character(),
                        }),

                        Some(JsxChild::Newline | JsxChild::Whitespace | JsxChild::EmptyLine) => {
                            None
                        }

                        None => None,
                    };

                    child_breaks = separator.is_some_and(WordSeparator::will_break);

                    flat.write(&format_args!(word, separator), f);

                    if let Some(separator) = separator {
                        multiline.write_with_separator(word, &separator, f);
                    } else {
                        // it's safe to write without a separator because None means that next element is a separator or end of the iterator
                        multiline.write_content(word, f);
                    }
                }

                // * Whitespace after the opening tag and before a meaningful text: `<div> a`
                // * Whitespace before the closing tag: `a </div>`
                // * Whitespace before an opening tag: `a <div>`
                JsxChild::Whitespace => {
                    flat.write(&JsxSpace, f);

                    // `<div>aaa </div>` or `<div> </div>`
                    let is_trailing_or_only_whitespace = children_iter.peek().is_none();

                    if is_trailing_or_only_whitespace {
                        multiline.write_separator_in_last_entry(&JsxRawSpace, f);
                    }
                    // Leading whitespace. Only possible if used together with a expression child
                    //
                    // ```
                    // <div>
                    //
                    //   {' '}
                    //   <b />
                    // </div>
                    // ```
                    else if last.is_none() {
                        multiline.write_with_separator(&JsxRawSpace, &hard_line_break(), f);
                    } else {
                        multiline.write_separator(&JsxSpace, f);
                    }
                }

                // A new line between some JSX text and an element
                JsxChild::Newline => {
                    let is_soft_break = {
                        // Here we handle the case when we have a newline between a single-character word and a jsx element
                        // We need to use the previous and the next element
                        // [JsxChild::Word, JsxChild::Newline, JsxChild::NonText]
                        // ```
                        // <div>
                        //   <div>First</div>,
                        //   <div>Second</div>
                        // </div>
                        // ```
                        if let Some(JsxChild::Word(word)) = last {
                            let is_next_element_self_closing = matches!(
                                children_iter.peek(),
                                Some(JsxChild::NonText(child)) if
                                matches!(child.as_ref(), JSXChild::Element(element) if
                                    element.closing_element.is_none())
                            );
                            !is_next_element_self_closing && word.is_single_character()
                        }
                        // Here we handle the case when we have a single-character word between a new line and a jsx element
                        // Here we need to look ahead two elements
                        // [JsxChild::Newline, JsxChild::Word, JsxChild::NonText]
                        // ```
                        // <div>
                        //   <div>First</div>
                        //   ,<div>Second</div>
                        // </div>
                        // ```
                        else if let Some(JsxChild::Word(next_word)) = children_iter.peek() {
                            let next_next_element = children_iter.peek_next();
                            let is_next_next_element_new_line =
                                matches!(next_next_element, Some(JsxChild::Newline));
                            let is_next_next_element_self_closing = matches!(
                                next_next_element,
                                    Some(JsxChild::NonText(child)) if
                                matches!(child.as_ref(), JSXChild::Element(element) if
                                    element.closing_element.is_none())
                            );
                            let has_new_line_and_self_closing = is_next_next_element_new_line
                                && matches!(
                                    children_iter.peek_next_next(),
                                        Some(JsxChild::NonText(child)) if
                                matches!(child.as_ref(), JSXChild::Element(element) if
                                    element.closing_element.is_none())
                                );

                            !has_new_line_and_self_closing
                                && !is_next_next_element_self_closing
                                && next_word.is_single_character()
                        } else {
                            false
                        }
                    };

                    if is_soft_break {
                        multiline.write_separator(&soft_line_break(), f);
                    } else {
                        child_breaks = true;
                        multiline.write_separator(&hard_line_break(), f);
                    }
                }

                // An empty line between some JSX text and an element
                JsxChild::EmptyLine => {
                    child_breaks = true;

                    // Additional empty lines are not preserved when any of
                    // the children are a meaningful text node.
                    //
                    // <>
                    //   <div>First</div>
                    //
                    //   <div>Second</div>
                    //
                    //   Third
                    // </>
                    //
                    // Becomes:
                    //
                    // <>
                    //   <div>First</div>
                    //   <div>Second</div>
                    //   Third
                    // </>
                    if children_meta.meaningful_text {
                        multiline.write_separator(&hard_line_break(), f);
                    } else {
                        multiline.write_separator(&empty_line(), f);
                    }
                }

                // Any child that isn't text
                JsxChild::NonText(non_text) => {
                    let mut is_non_text_node_next = false;
                    let line_mode = match children_iter.peek() {
                        Some(JsxChild::Word(word)) => {
                            // Break if the current or next element is a self closing element
                            // ```javascript
                            // <pre className="h-screen overflow-y-scroll" />adefg
                            // ```
                            // Becomes
                            // ```javascript
                            // <pre className="h-screen overflow-y-scroll" />
                            // adefg
                            // ```
                            if matches!(non_text.as_ref(), JSXChild::Element(element) if element.closing_element.is_none())
                                && !word.is_single_character()
                            {
                                Some(LineMode::Hard)
                            } else {
                                Some(LineMode::Soft)
                            }
                        }

                        // Add a hard line break if what comes after the element is not a text or is all whitespace
                        Some(JsxChild::NonText(_)) => {
                            is_non_text_node_next = true;
                            Some(LineMode::Hard)
                        }

                        Some(JsxChild::Newline | JsxChild::Whitespace | JsxChild::EmptyLine) => {
                            None
                        }
                        // Don't insert trailing line breaks
                        None => None,
                    };

                    child_breaks = line_mode.is_some_and(LineMode::is_hard);

                    let child_should_be_suppressed = is_next_child_suppressed;
                    let format_child = format_with(|f| {
                        if child_should_be_suppressed {
                            FormatSuppressedNode(non_text.span()).fmt(f);
                        } else {
                            non_text.fmt(f);
                        }
                    });

                    // Tests if a JSX element has a suppression comment or not.
                    //
                    // Suppression for JSX elements differs from regular nodes if they are inside of a JSXFragment or JSXElement children
                    // because they can then not be preceded by a comment.
                    //
                    // A JSX element inside of a JSX children list is suppressed if its first preceding sibling (that contains meaningful text)
                    // is a JSXExpressionContainer, not containing any expression, with a dangling suppression comment.
                    //
                    // ```javascript
                    // <div>
                    //   {/* prettier-ignore */}
                    //   <div a={  some} />
                    //   </div>
                    // ```
                    //
                    is_next_child_suppressed = child_breaks
                        && is_non_text_node_next
                        && matches!(non_text.as_ref(), JSXChild::ExpressionContainer(element) if
                            matches!(&element.expression, JSXExpression::EmptyExpression(_) if
                            f.context().comments().is_suppressed(element.span.end)
                        ));

                    let format_separator = line_mode.map(|mode| {
                        format_with(move |f| f.write_element(FormatElement::Line(mode)))
                    });

                    if force_multiline {
                        if let Some(format_separator) = format_separator {
                            multiline.write_with_separator(&format_child, &format_separator, f);
                        } else {
                            // it's safe to write without a separator because None means that next element is a separator or end of the iterator
                            multiline.write_content(&format_child, f);
                        }
                    } else {
                        let memoized = non_text.memoized();

                        child_breaks = memoized.inspect(f).will_break();

                        if !child_breaks {
                            flat.write(&format_args!(memoized, format_separator), f);
                        }

                        if let Some(format_separator) = format_separator {
                            multiline.write_with_separator(&memoized, &format_separator, f);
                        } else {
                            // it's safe to write without a separator because None means that next element is a separator or end of the iterator
                            multiline.write_content(&memoized, f);
                        }
                    }
                }
            }

            if child_breaks {
                flat.disable();
                force_multiline = true;
            }

            last = Some(child);
        }

        if force_multiline {
            FormatChildrenResult::ForceMultiline(multiline.finish())
        } else {
            FormatChildrenResult::BestFitting {
                flat_children: flat.finish(),
                expanded_children: multiline.finish(),
            }
        }
    }

    fn children_meta(
        children: &AstNode<'_, ArenaVec<'_, JSXChild<'_>>>,
        comments: &Comments<'_>,
    ) -> ChildrenMeta {
        let mut meta = ChildrenMeta::default();
        let mut has_expression = false;

        for child in children {
            match child.as_ref() {
                JSXChild::Element(_) | JSXChild::Fragment(_) => {
                    meta.any_tag = true;
                }
                JSXChild::ExpressionContainer(expression) => {
                    if is_whitespace_jsx_expression(expression, comments) {
                        meta.meaningful_text = true;
                    } else {
                        meta.multiple_expressions = has_expression;
                        has_expression = true;
                    }
                }
                JSXChild::Text(text) => {
                    meta.meaningful_text =
                        meta.meaningful_text || is_meaningful_jsx_text(&text.value);
                }
                JSXChild::Spread(_) => {}
            }
        }

        meta
    }

    fn layout(&self, meta: ChildrenMeta) -> JsxChildListLayout {
        match self.layout {
            JsxChildListLayout::BestFitting => {
                if meta.any_tag || meta.multiple_expressions {
                    JsxChildListLayout::Multiline
                } else {
                    JsxChildListLayout::BestFitting
                }
            }
            JsxChildListLayout::Multiline => JsxChildListLayout::Multiline,
        }
    }
}

/// The result of formatting the children of a JSX child list.
#[derive(Debug)]
pub enum FormatChildrenResult<'a, 'b> {
    /// Force the children to be formatted over multiple lines.
    ///
    /// For example:
    /// ```jsx
    /// <div>
    ///     <div>1</div>
    ///     <div>2</div>
    /// </div>
    /// ```
    ///
    /// This usually occurs when the children are already formatted over multiple lines, or when the children contains another tag.
    ForceMultiline(FormatMultilineChildren<'a>),

    /// Let the formatter determine whether the children should be formatted over multiple lines or if they can be kept on a single line.
    BestFitting {
        flat_children: FormatFlatChildren<'a>,
        expanded_children: FormatMultilineChildren<'a>,
    },

    SingleChild(FormatSingleChild<'a, 'b>),
}

#[derive(Debug, Default, Copy, Clone)]
pub enum JsxChildListLayout {
    /// Prefers to format the children on a single line if possible.
    #[default]
    BestFitting,

    /// Forces the children to be formatted over multiple lines
    Multiline,
}

impl JsxChildListLayout {
    const fn is_multiline(self) -> bool {
        matches!(self, Self::Multiline)
    }
}

#[derive(Copy, Clone, Debug, Default)]
struct ChildrenMeta {
    /// `true` if children contains a [`JSXElement`] or [`JSXFragment`]
    any_tag: bool,

    /// `true` if children contains more than one [`JSXExpressionContainer`]
    multiple_expressions: bool,

    /// `true` if any child contains meaningful a [`JSXText`] with meaningful text.
    meaningful_text: bool,
}

#[derive(Copy, Clone, Debug)]
enum WordSeparator {
    /// Separator between two words. Creates a soft line break or space.
    ///
    /// `a b`
    BetweenWords,

    /// A separator of a word at the end of a [`JSXText`] element. Either because it is the last
    /// child in its parent OR it is right before the start of another child (element, expression, ...).
    ///
    /// ```javascript
    /// <div>a</div>; // last element of parent
    /// <div>a<other /></div> // last element before another element
    /// <div>a{expression}</div> // last element before expression
    /// ```
    ///
    /// Creates a soft line break EXCEPT if the next element is a self closing element
    /// or the previous word was an ascii punctuation, which results in a hard line break:
    ///
    /// ```javascript
    /// a = <div>ab<br/></div>;
    ///
    /// // becomes
    ///
    /// a = (
    ///     <div>
    ///         ab
    ///         <br />
    ///     </div>
    /// );
    /// ```
    EndOfText { is_soft_line_break: bool },
}

impl WordSeparator {
    /// Returns if formatting this separator will result in a child that expands
    fn will_break(self) -> bool {
        matches!(self, Self::EndOfText { is_soft_line_break: false })
    }
}

impl<'a> Format<'a> for WordSeparator {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        match self {
            Self::BetweenWords => soft_line_break_or_space().fmt(f),
            Self::EndOfText { is_soft_line_break } => {
                if *is_soft_line_break {
                    soft_line_break().fmt(f);
                }
                // ```javascript
                // <div>ab<br/></div>
                // ```
                // Becomes
                //
                // ```javascript
                // <div>
                //  ab
                //  <br />
                // </div>
                // ```
                else {
                    hard_line_break().fmt(f);
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
enum MultilineLayout {
    Fill,
    #[default]
    NoFill,
}

/// Builder that helps to create the output for the multiline layout.
///
/// The multiline layout may use [`FormatElement::Tag`] element and [`Tag::StartFill`] tag that requires that its children
/// are an alternating sequence of `[element, separator, element, separator, ...]`.
///
/// This requires that each element is wrapped inside of a list if it emits more than one element to uphold
/// the constraints of [`FormatElement::Tag`].
///
/// However, the wrapping is only necessary for [`MultilineLayout::Fill`] for when the [`FormatElement::Tag`] element is used.
///
/// This builder takes care of doing the least amount of work necessary for the chosen layout while also guaranteeing
/// that the written element is valid
#[derive(Debug)]
struct MultilineBuilder<'a> {
    layout: MultilineLayout,
    result: ArenaVec<'a, FormatElement<'a>>,
}

impl<'a> MultilineBuilder<'a> {
    fn new(layout: MultilineLayout, allocator: &'a Allocator) -> Self {
        Self { layout, result: ArenaVec::new_in(allocator) }
    }

    /// Formats an element that does not require a separator
    /// It is safe to omit the separator because at the call side we must guarantee that we have reached the end of the iterator
    /// or the next element is a space/newline that should be written into the separator "slot".
    fn write_content(&mut self, content: &dyn Format<'a>, f: &mut Formatter<'_, 'a>) {
        self.write(content, None, f);
    }

    /// Formatting a separator does not require any element in the separator slot
    fn write_separator(&mut self, separator: &dyn Format<'a>, f: &mut Formatter<'_, 'a>) {
        self.write(separator, None, f);
    }

    fn write_with_separator(
        &mut self,
        content: &dyn Format<'a>,
        separator: &dyn Format<'a>,
        f: &mut Formatter<'_, 'a>,
    ) {
        self.write(content, Some(separator), f);
    }

    fn write(
        &mut self,
        content: &dyn Format<'a>,
        separator: Option<&dyn Format<'a>>,
        f: &mut Formatter<'_, 'a>,
    ) {
        let elements =
            std::mem::replace(&mut self.result, ArenaVec::new_in(f.context().allocator()));

        self.result = {
            let mut buffer = VecBuffer::new_with_vec(f.state_mut(), elements);
            match self.layout {
                MultilineLayout::Fill => {
                    // Make sure that the separator and content only ever write a single element
                    buffer.write_element(FormatElement::Tag(Tag::StartEntry));
                    write!(buffer, [content]);
                    buffer.write_element(FormatElement::Tag(Tag::EndEntry));

                    if let Some(separator) = separator {
                        buffer.write_element(FormatElement::Tag(Tag::StartEntry));
                        write!(buffer, [separator]);
                        buffer.write_element(FormatElement::Tag(Tag::EndEntry));
                    }
                }
                MultilineLayout::NoFill => {
                    write!(buffer, [content, separator]);
                }
            }
            buffer.into_vec()
        };
    }

    /// Writes a separator into the last entry if it is an entry.
    fn write_separator_in_last_entry(
        &mut self,
        separator: &dyn Format<'a>,
        f: &mut Formatter<'_, 'a>,
    ) {
        if self.result.last().is_some_and(|element| element.end_tag(TagKind::Entry).is_some()) {
            let last_index = self.result.len() - 1;
            self.result.insert(last_index, f.intern(separator).unwrap());
        } else {
            self.write_content(separator, f);
        }
    }

    fn finish(self) -> FormatMultilineChildren<'a> {
        FormatMultilineChildren { layout: self.layout, elements: RefCell::new(self.result) }
    }
}

#[derive(Debug)]
pub struct FormatMultilineChildren<'a> {
    layout: MultilineLayout,
    elements: RefCell<ArenaVec<'a, FormatElement<'a>>>,
}

impl<'a> Format<'a> for FormatMultilineChildren<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let format_inner = format_with(|f| {
            if let Some(elements) =
                f.intern_vec(self.elements.borrow_mut().take_in(f.context().allocator()))
            {
                match self.layout {
                    MultilineLayout::Fill => f.write_elements([
                        FormatElement::Tag(Tag::StartFill),
                        elements,
                        FormatElement::Tag(Tag::EndFill),
                    ]),
                    MultilineLayout::NoFill => f.write_elements([
                        FormatElement::Tag(Tag::StartGroup(
                            tag::Group::new().with_mode(GroupMode::Expand),
                        )),
                        elements,
                        FormatElement::Tag(Tag::EndGroup),
                    ]),
                }
            }
        });

        // This indent is wrapped with a group to ensure that the print mode is
        // set to `Expanded` when the group prints and will guarantee that the
        // content _does not_ fit when printed as part of a `Fill`. Example:
        //   <div>
        //     <span a b>
        //       <Foo />
        //     </span>{" "}
        //     ({variable})
        //   </div>
        // The `<span>...</span>` is the element that gets wrapped in the group
        // by this line. Importantly, it contains a hard line break, and because
        // [FitsMeasurer::fits_element] considers all hard lines as `Fits::Yes`,
        // it will cause the element and the following separator to be printed
        // in flat mode due to the logic of `Fill`. But because the we know the
        // item breaks over multiple lines, we want it to _not_ fit and print
        // both the content and the separator in Expanded mode, keeping the
        // formatting as shown above.
        //
        // The `group` here allows us to opt-in to telling the `FitsMeasurer`
        // that content that breaks shouldn't be considered flat and should be
        // expanded. This is in contrast to something like a concise array fill,
        // which _does_ allow breaks to fit and preserves density.
        write!(f, [group(&block_indent(&format_inner))]);
    }
}

#[derive(Debug)]
struct FlatBuilder<'a> {
    result: ArenaVec<'a, FormatElement<'a>>,
    disabled: bool,
}

impl<'a> FlatBuilder<'a> {
    fn new(disabled: bool, allocator: &'a Allocator) -> Self {
        Self { result: ArenaVec::new_in(allocator), disabled }
    }

    fn write(&mut self, content: &dyn Format<'a>, f: &mut Formatter<'_, 'a>) {
        if self.disabled {
            return;
        }

        let result = std::mem::replace(&mut self.result, ArenaVec::new_in(f.context().allocator()));

        self.result = {
            let elements = result;
            let mut buffer = VecBuffer::new_with_vec(f.state_mut(), elements);

            write!(buffer, [content]);

            buffer.into_vec()
        }
    }

    fn disable(&mut self) {
        self.disabled = true;
    }

    fn finish(self) -> FormatFlatChildren<'a> {
        assert!(
            !self.disabled,
            "The flat builder has been disabled and thus, does no longer store any elements. Make sure you don't call disable if you later intend to format the flat content."
        );

        FormatFlatChildren { elements: RefCell::new(self.result) }
    }
}

#[derive(Debug)]
pub struct FormatFlatChildren<'a> {
    elements: RefCell<ArenaVec<'a, FormatElement<'a>>>,
}

impl<'a> Format<'a> for FormatFlatChildren<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        if let Some(elements) =
            f.intern_vec(self.elements.borrow_mut().take_in(f.context().allocator()))
        {
            f.write_element(elements);
        }
    }
}

/// Optimized formatting for a single JSX child.
///
/// When there is only a single child, we do not need to write the child into a temporary buffer
/// and then take it when formatting. Instead, we can directly write the child into the root buffer.
///
/// Also, this can avoid calling costly `best_fitting!` formatting in some situations.
#[derive(Debug)]
pub struct FormatSingleChild<'a, 'b> {
    child: JsxChild<'a, 'b>,
    force_multiline: bool,
}

impl<'a> Format<'a> for FormatSingleChild<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let format_inner = format_with(|f| match &self.child {
            JsxChild::Word(word) => {
                word.fmt(f);
            }
            JsxChild::Whitespace => {
                JsxSpace.fmt(f);
            }
            JsxChild::NonText(non_text) => {
                non_text.fmt(f);
            }
            JsxChild::Newline | JsxChild::EmptyLine => {
                unreachable!(
                    "`Newline` or `EmptyLine` should have been trimmed for single child formatting"
                );
            }
        });

        if self.force_multiline {
            write!(f, [block_indent(&format_inner)]);
        } else {
            write!(f, [soft_block_indent(&format_inner)]);
        }
    }
}
