//! Human-readable debug rendering for [`Document`].
//!
//! This module provides:
//! * The `Format` implementation for `&[FormatElement]` that produces a readable
//!   textual representation of the IR (used by `DisplayDocument`).
//! * The [`DisplayDocument`] wrapper exposed via [`Document::display`].
//!
//! The rendering is language-agnostic and works with any `C: FormatContext`.
//! Languages that support Tailwind sorting can override
//! [`FormatContext::get_tailwind_class`] to surface class names; languages
//! without Tailwind support get the default `None` and an `<UNKNOWN_TAILWIND_CLASS_INDEX<..>>`
//! marker if a `FormatElement::TailwindClass` ever appears.

#![expect(clippy::mutable_key_type)]

use cow_utils::CowUtils;

use oxc_allocator::Allocator;

use crate::{
    Argument, Arguments, Buffer, BufferExtensions, Document, Format, FormatContext, FormatElement,
    FormatOptions, FormatState, Formatter, PrintMode, Printer, PrinterOptions, SimpleFormatContext,
    VecBuffer,
    builders::{hard_line_break, soft_line_break_or_space, space, text, token},
    format::write,
    format_element::{
        LineMode, TextWidth,
        tag::{self, DedentMode, GroupMode, Tag},
    },
    write as w,
};

impl<'a> Document<'a> {
    /// Returns a wrapper that formats the document for human-readable debug display.
    ///
    /// The `source_text` is passed to the temporary `SimpleFormatContext` used
    /// while rendering and is only required so that `Format` implementations that
    /// look up the source text behave sensibly. The debug renderer itself does not
    /// use the source text.
    pub fn display<'src>(&'a self, source_text: &'src str) -> DisplayDocument<'a, 'src> {
        DisplayDocument {
            elements: self.elements(),
            tailwind_classes: self.sorted_tailwind_classes(),
            source_text,
        }
    }
}

/// `Display` wrapper that renders a [`Document`] as a human-readable debug string.
///
/// Obtain one via [`Document::display`].
pub struct DisplayDocument<'a, 'src> {
    elements: &'a [FormatElement<'a>],
    tailwind_classes: &'a [String],
    source_text: &'src str,
}

impl std::fmt::Display for DisplayDocument<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let allocator = Allocator::default();
        let mut context = SimpleFormatContext::default().with_source_code(self.source_text);
        context.set_tailwind_classes(self.tailwind_classes.to_vec());

        let mut state: FormatState<'_, SimpleFormatContext<'_>> =
            FormatState::new(context, &allocator);
        let mut buffer = VecBuffer::new(&mut state);

        let elements = FormatElementSlice(self.elements);
        write(&mut buffer, Arguments::new(&[Argument::new(&elements)]));

        let elements = buffer.into_vec();
        let document = Document::new(elements, Vec::default());

        let printer_options = PrinterOptions::default();
        let (rendered_elements, sorted) = document.into_elements_and_tailwind_classes();
        let printed = Printer::new(printer_options, &sorted)
            .print(rendered_elements)
            .expect("Expected a valid document");

        f.write_str(printed.as_code())
    }
}

/// Wrapper around a [`FormatElement`] slice so we can provide a `Format`
/// implementation without conflicting with the blanket `Format` impl for `&T`.
struct FormatElementSlice<'a>(&'a [FormatElement<'a>]);

impl<'a, C> Format<'a, C> for FormatElementSlice<'a>
where
    C: FormatContext,
{
    fn fmt(&self, f: &mut Formatter<'_, 'a, C>) {
        use Tag::{
            EndAlign, EndConditionalContent, EndDedent, EndEntry, EndFill, EndGroup, EndIndent,
            EndIndentIfGroupBreaks, EndLabelled, EndLineSuffix, EndMarkAsRoot, StartAlign,
            StartConditionalContent, StartDedent, StartEntry, StartFill, StartGroup, StartIndent,
            StartIndentIfGroupBreaks, StartLabelled, StartLineSuffix, StartMarkAsRoot,
        };

        w!(f, [ContentArrayStart]);

        let mut tag_stack = Vec::new();
        let mut first_element = true;
        let mut in_text = false;

        let mut iter = self.0.iter().peekable();

        while let Some(element) = iter.next() {
            if !first_element && !in_text && !element.is_end_tag() {
                // Write a separator between every two elements
                w!(f, [token(","), soft_line_break_or_space()]);
            }

            first_element = false;

            match element {
                element @ (FormatElement::Space
                | FormatElement::Token { .. }
                | FormatElement::Text { .. }) => {
                    if !in_text {
                        w!(f, [token("\"")]);
                    }

                    in_text = true;

                    match element {
                        FormatElement::Space => {
                            w!(f, [token(" ")]);
                        }
                        element if element.is_text() => {
                            // escape quotes
                            let new_element = match element {
                                // except for static text because source_position is unknown
                                FormatElement::Token { .. } => element.clone(),
                                FormatElement::Text { text, width: _ } => {
                                    let text = text.cow_replace('"', "\\\"");
                                    FormatElement::Text {
                                        text: f.allocator().alloc_str(&text),
                                        width: TextWidth::from_text(
                                            &text,
                                            f.options().indent_width(),
                                        ),
                                    }
                                }
                                _ => unreachable!(),
                            };
                            f.write_element(new_element);
                        }
                        _ => unreachable!(),
                    }

                    let is_next_text = iter.peek().is_some_and(|e| e.is_text() || e.is_space());

                    if !is_next_text {
                        w!(f, [token("\"")]);
                        in_text = false;
                    }
                }

                FormatElement::Line(mode) => match mode {
                    LineMode::SoftOrSpace => {
                        w!(f, [token("soft_line_break_or_space")]);
                    }
                    LineMode::Soft => {
                        w!(f, [token("soft_line_break")]);
                    }
                    LineMode::Hard => {
                        w!(f, [token("hard_line_break")]);
                    }
                    LineMode::Empty => {
                        w!(f, [token("empty_line")]);
                    }
                    LineMode::Literal => {
                        w!(f, [token("literal_line_break")]);
                    }
                },
                FormatElement::ExpandParent => {
                    w!(f, [token("expand_parent")]);
                }

                FormatElement::LineSuffixBoundary => {
                    w!(f, [token("line_suffix_boundary")]);
                }

                FormatElement::BestFitting(best_fitting) => {
                    w!(f, [token("best_fitting([")]);
                    f.write_elements([
                        FormatElement::Tag(StartIndent),
                        FormatElement::Line(LineMode::Hard),
                    ]);

                    for variant in best_fitting.variants() {
                        w!(f, [FormatElementSlice(variant), hard_line_break()]);
                    }

                    f.write_elements([
                        FormatElement::Tag(EndIndent),
                        FormatElement::Line(LineMode::Hard),
                    ]);

                    w!(f, [token("])")]);
                }

                FormatElement::Interned(interned) => {
                    let interned_elements = f.state_mut().printed_interned_elements();
                    match interned_elements.get(interned).copied() {
                        None => {
                            let index = interned_elements.len();
                            interned_elements.insert(interned.clone(), index);

                            w!(
                                f,
                                [
                                    text(
                                        f.allocator()
                                            .alloc_str(&std::format!("<interned {index}>"))
                                    ),
                                    space(),
                                    FormatElementSlice(interned),
                                ]
                            );
                        }
                        Some(reference) => {
                            w!(
                                f,
                                [text(
                                    f.allocator()
                                        .alloc_str(&std::format!("<ref interned *{reference}>"))
                                )]
                            );
                        }
                    }
                }

                FormatElement::Tag(tag) => {
                    if tag.is_start() {
                        first_element = true;
                        tag_stack.push(tag.kind());
                    }
                    // Handle documents with mismatching start/end or superfluous end tags
                    else {
                        match tag_stack.pop() {
                            None => {
                                // Only write the end tag without any indent to ensure the output document is valid.
                                w!(
                                    f,
                                    [
                                        token("<END_TAG_WITHOUT_START<"),
                                        text(
                                            f.allocator()
                                                .alloc_str(&std::format!("{:?}", tag.kind()))
                                        ),
                                        token(">>"),
                                    ]
                                );
                                first_element = false;
                                continue;
                            }
                            Some(start_kind) if start_kind != tag.kind() => {
                                w!(
                                    f,
                                    [
                                        ContentArrayEnd,
                                        token(")"),
                                        soft_line_break_or_space(),
                                        token("ERROR<START_END_TAG_MISMATCH<start: "),
                                        text(
                                            f.allocator()
                                                .alloc_str(&std::format!("{start_kind:?}"))
                                        ),
                                        token(", end: "),
                                        text(
                                            f.allocator()
                                                .alloc_str(&std::format!("{:?}", tag.kind()))
                                        ),
                                        token(">>")
                                    ]
                                );
                                first_element = false;
                                continue;
                            }
                            _ => {
                                // all ok
                            }
                        }
                    }

                    match tag {
                        StartIndent => {
                            w!(f, [token("indent(")]);
                        }

                        StartDedent(mode) => {
                            let label = match mode {
                                DedentMode::Level => "dedent",
                                DedentMode::Root => "dedentRoot",
                            };

                            w!(f, [token(label), token("(")]);
                        }

                        StartAlign(align) => {
                            w!(
                                f,
                                [
                                    token("align("),
                                    text(f.allocator().alloc_str(&align.count().to_string()),),
                                    token(","),
                                    space(),
                                ]
                            );
                        }

                        StartLineSuffix => {
                            w!(f, [token("line_suffix(")]);
                        }

                        StartGroup(group) => {
                            w!(f, [token("group(")]);

                            if let Some(group_id) = group.id() {
                                w!(
                                    f,
                                    [
                                        text(
                                            f.allocator()
                                                .alloc_str(&std::format!("\"{group_id:?}\""))
                                        ),
                                        token(","),
                                        space(),
                                    ]
                                );
                            }

                            match group.mode() {
                                GroupMode::Flat => {}
                                GroupMode::Expand => {
                                    w!(f, [token("expand: true,"), space()]);
                                }
                                GroupMode::Propagated => {
                                    w!(f, [token("expand: propagated,"), space()]);
                                }
                            }
                        }

                        StartIndentIfGroupBreaks(id) => {
                            w!(
                                f,
                                [
                                    token("indent_if_group_breaks("),
                                    text(f.allocator().alloc_str(&std::format!("\"{id:?}\"")),),
                                    token(","),
                                    space(),
                                ]
                            );
                        }

                        StartConditionalContent(condition) => {
                            match condition.mode() {
                                PrintMode::Flat => {
                                    w!(f, [token("if_group_fits_on_line(")]);
                                }
                                PrintMode::Expanded => {
                                    w!(f, [token("if_group_breaks(")]);
                                }
                            }

                            if let Some(group_id) = condition.group_id() {
                                w!(
                                    f,
                                    [
                                        text(
                                            f.allocator()
                                                .alloc_str(&std::format!("\"{group_id:?}\"")),
                                        ),
                                        token(","),
                                        space(),
                                    ]
                                );
                            }
                        }

                        StartLabelled(label_id) => {
                            w!(
                                f,
                                [
                                    token("label("),
                                    text(
                                        f.allocator().alloc_str(&std::format!("\"{label_id:?}\"")),
                                    ),
                                    token(","),
                                    space(),
                                ]
                            );
                        }

                        StartFill => {
                            w!(f, [token("fill(")]);
                        }

                        StartMarkAsRoot => {
                            w!(f, [token("mark_as_root(")]);
                        }

                        StartEntry => {
                            // handled after the match for all start tags
                        }
                        EndEntry => w!(f, [ContentArrayEnd]),

                        EndFill
                        | EndLabelled
                        | EndConditionalContent
                        | EndIndentIfGroupBreaks(_)
                        | EndAlign
                        | EndIndent
                        | EndGroup
                        | EndLineSuffix
                        | EndMarkAsRoot
                        | EndDedent(_) => {
                            w!(f, [ContentArrayEnd, token(")")]);
                        }
                    }

                    if tag.is_start() {
                        w!(f, [ContentArrayStart]);
                    }
                }
                FormatElement::TailwindClass(index) => {
                    let class = f.context().get_tailwind_class(*index);
                    if let Some(class) = class {
                        w!(f, [text(f.allocator().alloc_str(class))]);
                    } else {
                        w!(
                            f,
                            [
                                token("<UNKNOWN_TAILWIND_CLASS_INDEX<"),
                                text(f.allocator().alloc_str(&std::format!("{index}"))),
                                token(">>"),
                            ]
                        );
                    }
                }
                FormatElement::EmbedPlaceholder(index) => {
                    w!(
                        f,
                        [
                            token("<embed-placeholder<"),
                            text(f.allocator().alloc_str(&std::format!("{index}"))),
                            token(">>"),
                        ]
                    );
                }
            }
        }

        while let Some(top) = tag_stack.pop() {
            w!(
                f,
                [
                    ContentArrayEnd,
                    token(")"),
                    soft_line_break_or_space(),
                    text(f.allocator().alloc_str(&std::format!("<START_WITHOUT_END<{top:?}>>"))),
                ]
            );
        }

        w!(f, [ContentArrayEnd]);
    }
}

struct ContentArrayStart;

impl<'a, C> Format<'a, C> for ContentArrayStart {
    fn fmt(&self, f: &mut Formatter<'_, 'a, C>) {
        use Tag::{StartGroup, StartIndent};

        w!(f, [token("[")]);

        f.write_elements([
            FormatElement::Tag(StartGroup(tag::Group::new())),
            FormatElement::Tag(StartIndent),
            FormatElement::Line(LineMode::Soft),
        ]);
    }
}

struct ContentArrayEnd;

impl<'a, C> Format<'a, C> for ContentArrayEnd {
    fn fmt(&self, f: &mut Formatter<'_, 'a, C>) {
        use Tag::{EndGroup, EndIndent};
        f.write_elements([
            FormatElement::Tag(EndIndent),
            FormatElement::Line(LineMode::Soft),
            FormatElement::Tag(EndGroup),
        ]);

        w!(f, [token("]")]);
    }
}
