#![expect(clippy::mutable_key_type)]
use cow_utils::CowUtils;
use std::ops::Deref;

use oxc_allocator::Allocator;
use oxc_ast::Comment;
use oxc_span::SourceType;
use rustc_hash::FxHashMap;

use super::super::prelude::*;
use super::tag::Tag;
use crate::formatter::TextSize;
use crate::formatter::prelude::tag::{DedentMode, GroupMode};
use crate::{
    Format, FormatOptions, FormatResult, IndentStyle, IndentWidth, LineEnding, LineWidth,
    formatter::FormatContext, formatter::Formatter,
};

use crate::{format, write};

/// A formatted document.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Document<'a> {
    elements: Vec<FormatElement<'a>>,
}

impl Document<'_> {
    /// Sets [`expand`](tag::Group::expand) to [`GroupMode::Propagated`] if the group contains any of:
    /// * a group with [`expand`](tag::Group::expand) set to [GroupMode::Propagated] or [GroupMode::Expand].
    /// * a non-soft [line break](FormatElement::Line) with mode [LineMode::Hard], [LineMode::Empty], or [LineMode::Literal].
    /// * a [FormatElement::ExpandParent]
    ///
    /// [`BestFitting`] elements act as expand boundaries, meaning that the fact that a
    /// [`BestFitting`]'s content expands is not propagated past the [`BestFitting`] element.
    ///
    /// [`BestFitting`]: FormatElement::BestFitting
    pub(crate) fn propagate_expand(&mut self) {
        #[derive(Debug)]
        enum Enclosing<'a> {
            Group(&'a tag::Group),
            BestFitting,
        }

        fn expand_parent(enclosing: &[Enclosing]) {
            if let Some(Enclosing::Group(group)) = enclosing.last() {
                group.propagate_expand();
            }
        }

        fn propagate_expands<'a>(
            elements: &'a [FormatElement<'a>],
            enclosing: &mut Vec<Enclosing<'a>>,
            checked_interned: &mut FxHashMap<&'a Interned<'a>, bool>,
        ) -> bool {
            let mut expands = false;
            for element in elements {
                let element_expands = match element {
                    FormatElement::Tag(Tag::StartGroup(group)) => {
                        enclosing.push(Enclosing::Group(group));
                        false
                    }
                    FormatElement::Tag(Tag::EndGroup) => match enclosing.pop() {
                        Some(Enclosing::Group(group)) => !group.mode().is_flat(),
                        _ => false,
                    },
                    FormatElement::Interned(interned) => {
                        if let Some(interned_expands) = checked_interned.get(interned) {
                            *interned_expands
                        } else {
                            let interned_expands =
                                propagate_expands(interned, enclosing, checked_interned);
                            checked_interned.insert(interned, interned_expands);
                            interned_expands
                        }
                    }
                    FormatElement::BestFitting(best_fitting) => {
                        enclosing.push(Enclosing::BestFitting);

                        for variant in best_fitting.variants() {
                            propagate_expands(variant, enclosing, checked_interned);
                        }

                        enclosing.pop();
                        // BestFitting acts as a boundary, meaning there is no need to continue
                        // processing this element and we can move onto the next. However, we
                        // _don't_ set `expands = false`, because that ends up negating the
                        // expansion when processing `Interned` elements.
                        //
                        // Only interned lists are affected, because they cache the expansion value
                        // based on the value of `expands` at the end of iterating the children. If
                        // a `best_fitting` element occurs after the last expanding element, and we
                        // end up setting `expands = false` here, then the interned element ends up
                        // thinking that its content doesn't expand, even though it might. Example:
                        //   group(1,
                        //     interned 1 [
                        //       expand_parent,
                        //       best_fitting,
                        //     ]
                        //   )
                        //   group(2,
                        //     [ref interned 1]
                        //   )
                        // Here, `group(1)` gets expanded directly by the `expand_parent` element.
                        // This happens immediately, and then `expands = true` is set. The interned
                        // element continues processing, and encounters the `best_fitting`. If
                        // we set `expands = false` there, then the interned element's result ends
                        // up being `false`, even though it does actually expand. Then, when
                        // `group(2)` checks for expansion, it looks at the ref to `interned 1`,
                        // which thinks it doesn't expand, and so `group(2)` stays flat.
                        //
                        // By _not_ setting `expands = false`, even though `best_fitting` is a
                        // boundary for expansion, we ensure that any references to the interned
                        // element will get the correct value for whether the contained content
                        // actually expands, regardless of the order of elements within it.
                        //
                        // Instead, just returning false here enforces that `best_fitting` doesn't
                        // think it expands _itself_, but allows other sibling elements to still
                        // propagate their expansion.
                        false
                    }
                    FormatElement::StaticText { text }
                    | FormatElement::DynamicText { text, .. } => text.contains('\n'),
                    FormatElement::LocatedTokenText { slice, .. } => slice.contains('\n'),
                    FormatElement::ExpandParent
                    | FormatElement::Line(LineMode::Hard | LineMode::Empty) => true,
                    _ => false,
                };

                if element_expands {
                    expands = true;
                    expand_parent(enclosing);
                }
            }

            expands
        }

        let mut enclosing: Vec<Enclosing> = Vec::new();
        let mut interned = FxHashMap::default();
        propagate_expands(self, &mut enclosing, &mut interned);
    }
}

impl<'a> From<Vec<FormatElement<'a>>> for Document<'a> {
    fn from(elements: Vec<FormatElement<'a>>) -> Self {
        Self { elements }
    }
}

impl<'a> Deref for Document<'a> {
    type Target = [FormatElement<'a>];

    fn deref(&self) -> &Self::Target {
        self.elements.as_slice()
    }
}

impl std::fmt::Display for Document<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let allocator = Allocator::default();
        let source_text = allocator.alloc_str("");
        let source_type = SourceType::default();
        let comments: oxc_allocator::Vec<Comment> = oxc_allocator::Vec::new_in(&allocator);
        let options = FormatOptions {
            line_width: LineWidth::default(),
            indent_style: IndentStyle::Space,
            indent_width: IndentWidth::default(),
            line_ending: LineEnding::Lf,
            ..Default::default()
        };
        let context =
            FormatContext::new(source_text, source_type, comments.as_ref(), &allocator, options);

        let formatted = format!(context, [self.elements.as_slice()])
            .expect("Formatting not to throw any FormatErrors");

        f.write_str(formatted.print().expect("Expected a valid document").as_code())
    }
}

impl<'a> Format<'a> for &[FormatElement<'a>] {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        use Tag::{
            EndAlign, EndConditionalContent, EndDedent, EndEntry, EndFill, EndGroup, EndIndent,
            EndIndentIfGroupBreaks, EndLabelled, EndLineSuffix, EndVerbatim, StartAlign,
            StartConditionalContent, StartDedent, StartEntry, StartFill, StartGroup, StartIndent,
            StartIndentIfGroupBreaks, StartLabelled, StartLineSuffix, StartVerbatim,
        };

        write!(f, [ContentArrayStart])?;

        let mut tag_stack = Vec::new();
        let mut first_element = true;
        let mut in_text = false;

        let mut iter = self.iter().peekable();

        while let Some(element) = iter.next() {
            if !first_element && !in_text && !element.is_end_tag() {
                // Write a separator between every two elements
                write!(f, [text(","), soft_line_break_or_space()])?;
            }

            first_element = false;

            match element {
                element @ (FormatElement::Space
                | FormatElement::HardSpace
                | FormatElement::StaticText { .. }
                | FormatElement::DynamicText { .. }
                | FormatElement::LocatedTokenText { .. }) => {
                    if !in_text {
                        write!(f, [text("\"")])?;
                    }

                    in_text = true;

                    match element {
                        FormatElement::Space | FormatElement::HardSpace => {
                            write!(f, [text(" ")])?;
                        }
                        element if element.is_text() => {
                            // escape quotes
                            let new_element = match element {
                                // except for static text because source_position is unknown
                                FormatElement::StaticText { .. } => element.clone(),
                                FormatElement::DynamicText { text } => {
                                    let text = text.cow_replace('"', "\\\"");
                                    FormatElement::DynamicText {
                                        text: f.context().allocator().alloc_str(&text),
                                    }
                                }
                                FormatElement::LocatedTokenText { slice, source_position } => {
                                    let text = slice.cow_replace('"', "\\\"");
                                    FormatElement::DynamicText {
                                        text: f.context().allocator().alloc_str(&text),
                                    }
                                }
                                _ => unreachable!(),
                            };
                            f.write_element(new_element)?;
                        }
                        _ => unreachable!(),
                    }

                    let is_next_text = iter.peek().is_some_and(|e| e.is_text() || e.is_space());

                    if !is_next_text {
                        write!(f, [text("\"")])?;
                        in_text = false;
                    }
                }

                FormatElement::Line(mode) => match mode {
                    LineMode::SoftOrSpace => {
                        write!(f, [text("soft_line_break_or_space")])?;
                    }
                    LineMode::Soft => {
                        write!(f, [text("soft_line_break")])?;
                    }
                    LineMode::Hard => {
                        write!(f, [text("hard_line_break")])?;
                    }
                    LineMode::Empty => {
                        write!(f, [text("empty_line")])?;
                    }
                },
                FormatElement::ExpandParent => {
                    write!(f, [text("expand_parent")])?;
                }

                FormatElement::LineSuffixBoundary => {
                    write!(f, [text("line_suffix_boundary")])?;
                }

                FormatElement::BestFitting(best_fitting) => {
                    write!(f, [text("best_fitting([")])?;
                    f.write_elements([
                        FormatElement::Tag(StartIndent),
                        FormatElement::Line(LineMode::Hard),
                    ])?;

                    for variant in best_fitting.variants() {
                        write!(f, [&**variant, hard_line_break()])?;
                    }

                    f.write_elements([
                        FormatElement::Tag(EndIndent),
                        FormatElement::Line(LineMode::Hard),
                    ])?;

                    write!(f, [text("])")])?;
                }

                FormatElement::Interned(interned) => {
                    let mut interned_elements = f.state_mut().printed_interned_elements();
                    match interned_elements.get(interned).copied() {
                        None => {
                            let index = interned_elements.len();
                            interned_elements.insert(interned.clone(), index);

                            write!(
                                f,
                                [
                                    dynamic_text(
                                        f.context()
                                            .allocator()
                                            .alloc_str(&std::format!("<interned {index}>"))
                                    ),
                                    space(),
                                    &&**interned,
                                ]
                            )?;
                        }
                        Some(reference) => {
                            write!(
                                f,
                                [dynamic_text(
                                    f.context()
                                        .allocator()
                                        .alloc_str(&std::format!("<ref interned *{reference}>"))
                                )]
                            )?;
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
                                write!(
                                    f,
                                    [
                                        text("<END_TAG_WITHOUT_START<"),
                                        dynamic_text(
                                            f.context()
                                                .allocator()
                                                .alloc_str(&std::format!("{:?}", tag.kind()))
                                        ),
                                        text(">>"),
                                    ]
                                )?;
                                first_element = false;
                                continue;
                            }
                            Some(start_kind) if start_kind != tag.kind() => {
                                write!(
                                    f,
                                    [
                                        ContentArrayEnd,
                                        text(")"),
                                        soft_line_break_or_space(),
                                        text("ERROR<START_END_TAG_MISMATCH<start: "),
                                        dynamic_text(
                                            f.context()
                                                .allocator()
                                                .alloc_str(&std::format!("{start_kind:?}"))
                                        ),
                                        text(", end: "),
                                        dynamic_text(
                                            f.context()
                                                .allocator()
                                                .alloc_str(&std::format!("{:?}", tag.kind()))
                                        ),
                                        text(">>")
                                    ]
                                )?;
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
                            write!(f, [text("indent(")])?;
                        }

                        StartDedent(mode) => {
                            let label = match mode {
                                DedentMode::Level => "dedent",
                                DedentMode::Root => "dedentRoot",
                            };

                            write!(f, [text(label), text("(")])?;
                        }

                        StartAlign(tag::Align(count)) => {
                            write!(
                                f,
                                [
                                    text("align("),
                                    dynamic_text(
                                        f.context().allocator().alloc_str(&count.to_string()),
                                    ),
                                    text(","),
                                    space(),
                                ]
                            )?;
                        }

                        StartLineSuffix => {
                            write!(f, [text("line_suffix(")])?;
                        }

                        StartVerbatim(_) => {
                            write!(f, [text("verbatim(")])?;
                        }

                        StartGroup(group) => {
                            write!(f, [text("group(")])?;

                            if let Some(group_id) = group.id() {
                                write!(
                                    f,
                                    [
                                        dynamic_text(
                                            f.context()
                                                .allocator()
                                                .alloc_str(&std::format!("\"{group_id:?}\""))
                                        ),
                                        text(","),
                                        space(),
                                    ]
                                )?;
                            }

                            match group.mode() {
                                GroupMode::Flat => {}
                                GroupMode::Expand => {
                                    write!(f, [text("expand: true,"), space()])?;
                                }
                                GroupMode::Propagated => {
                                    write!(f, [text("expand: propagated,"), space()])?;
                                }
                            }
                        }

                        StartIndentIfGroupBreaks(id) => {
                            write!(
                                f,
                                [
                                    text("indent_if_group_breaks("),
                                    dynamic_text(
                                        f.context()
                                            .allocator()
                                            .alloc_str(&std::format!("\"{id:?}\"")),
                                    ),
                                    text(","),
                                    space(),
                                ]
                            )?;
                        }

                        StartConditionalContent(condition) => {
                            match condition.mode {
                                PrintMode::Flat => {
                                    write!(f, [text("if_group_fits_on_line(")])?;
                                }
                                PrintMode::Expanded => {
                                    write!(f, [text("if_group_breaks(")])?;
                                }
                            }

                            if let Some(group_id) = condition.group_id {
                                write!(
                                    f,
                                    [
                                        dynamic_text(
                                            f.context()
                                                .allocator()
                                                .alloc_str(&std::format!("\"{group_id:?}\"")),
                                        ),
                                        text(","),
                                        space(),
                                    ]
                                )?;
                            }
                        }

                        StartLabelled(label_id) => {
                            write!(
                                f,
                                [
                                    text("label("),
                                    dynamic_text(
                                        f.context()
                                            .allocator()
                                            .alloc_str(&std::format!("\"{label_id:?}\"")),
                                    ),
                                    text(","),
                                    space(),
                                ]
                            )?;
                        }

                        StartFill => {
                            write!(f, [text("fill(")])?;
                        }

                        StartEntry => {
                            // handled after the match for all start tags
                        }
                        EndEntry => write!(f, [ContentArrayEnd])?,

                        EndFill
                        | EndLabelled
                        | EndConditionalContent
                        | EndIndentIfGroupBreaks(_)
                        | EndAlign
                        | EndIndent
                        | EndGroup
                        | EndLineSuffix
                        | EndDedent(_)
                        | EndVerbatim => {
                            write!(f, [ContentArrayEnd, text(")")])?;
                        }
                    }

                    if tag.is_start() {
                        write!(f, [ContentArrayStart])?;
                    }
                }
            }
        }

        while let Some(top) = tag_stack.pop() {
            write!(
                f,
                [
                    ContentArrayEnd,
                    text(")"),
                    soft_line_break_or_space(),
                    dynamic_text(
                        f.context()
                            .allocator()
                            .alloc_str(&std::format!("<START_WITHOUT_END<{top:?}>>"))
                    ),
                ]
            )?;
        }

        write!(f, [ContentArrayEnd])
    }
}

struct ContentArrayStart;

impl<'a> Format<'a> for ContentArrayStart {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        use Tag::{StartGroup, StartIndent};

        write!(f, [text("[")])?;

        f.write_elements([
            FormatElement::Tag(StartGroup(tag::Group::new())),
            FormatElement::Tag(StartIndent),
            FormatElement::Line(LineMode::Soft),
        ])
    }
}

struct ContentArrayEnd;

impl<'a> Format<'a> for ContentArrayEnd {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        use Tag::{EndGroup, EndIndent};
        f.write_elements([
            FormatElement::Tag(EndIndent),
            FormatElement::Line(LineMode::Soft),
            FormatElement::Tag(EndGroup),
        ])?;

        write!(f, [text("]")])
    }
}

impl FormatElements for [FormatElement<'_>] {
    fn will_break(&self) -> bool {
        use Tag::{EndLineSuffix, StartLineSuffix};
        let mut ignore_depth = 0usize;

        for element in self {
            match element {
                // Line suffix
                // Ignore if any of its content breaks
                FormatElement::Tag(StartLineSuffix) => {
                    ignore_depth += 1;
                }
                FormatElement::Tag(EndLineSuffix) => {
                    ignore_depth -= 1;
                }
                FormatElement::Interned(interned) if ignore_depth == 0 => {
                    if interned.will_break() {
                        return true;
                    }
                }

                element if ignore_depth == 0 && element.will_break() => {
                    return true;
                }
                _ => {}
            }
        }

        debug_assert_eq!(ignore_depth, 0, "Unclosed start container");

        false
    }

    fn may_directly_break(&self) -> bool {
        use Tag::{EndLineSuffix, StartLineSuffix};
        let mut ignore_depth = 0usize;

        for element in self {
            match element {
                // Line suffix
                // Ignore if any of its content breaks
                FormatElement::Tag(StartLineSuffix) => {
                    ignore_depth += 1;
                }
                FormatElement::Tag(EndLineSuffix) => {
                    ignore_depth -= 1;
                }
                FormatElement::Interned(interned) if ignore_depth == 0 => {
                    if interned.may_directly_break() {
                        return true;
                    }
                }

                element if ignore_depth == 0 && element.may_directly_break() => {
                    return true;
                }
                _ => {}
            }
        }

        debug_assert_eq!(ignore_depth, 0, "Unclosed start container");

        false
    }

    fn has_label(&self, expected: LabelId) -> bool {
        self.first().is_some_and(|element| element.has_label(expected))
    }

    fn start_tag(&self, kind: TagKind) -> Option<&Tag> {
        fn traverse_slice<'a>(
            slice: &'a [FormatElement],
            kind: TagKind,
            depth: &mut usize,
        ) -> Option<&'a Tag> {
            for element in slice.iter().rev() {
                match element {
                    FormatElement::Tag(tag) if tag.kind() == kind => {
                        if tag.is_start() {
                            if *depth == 0 {
                                // Invalid document
                                return None;
                            } else if *depth == 1 {
                                return Some(tag);
                            }
                            *depth -= 1;
                        } else {
                            *depth += 1;
                        }
                    }
                    FormatElement::Interned(interned) => {
                        match traverse_slice(interned, kind, depth) {
                            Some(start) => {
                                return Some(start);
                            }
                            // Reached end or invalid document
                            None if *depth == 0 => {
                                return None;
                            }
                            _ => {
                                // continue with other elements
                            }
                        }
                    }
                    _ => {}
                }
            }

            None
        }

        // Assert that the document ends at a tag with the specified kind;
        let _ = self.end_tag(kind)?;

        let mut depth = 0usize;

        traverse_slice(self, kind, &mut depth)
    }

    fn end_tag(&self, kind: TagKind) -> Option<&Tag> {
        self.last().and_then(|element| element.end_tag(kind))
    }
}
