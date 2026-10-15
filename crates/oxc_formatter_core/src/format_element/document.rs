use std::ops::Deref;

use oxc_allocator::ArenaVec;
use rustc_hash::FxHashMap;

use crate::{PrintResult, Printed, Printer, PrinterOptions};

use super::{
    FormatElement, FormatElements, Interned, LineMode, PrintMode, TextWidth,
    flat::{Flat, Lead},
    tag::{self, LabelId, Tag, TagKind},
};

/// A formatted document.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Document<'a> {
    elements: &'a [FormatElement<'a>],
    sorted_tailwind_classes: Vec<String>,
}

impl<'a> Document<'a> {
    /// Returns the format elements that make up this document.
    pub fn elements(&self) -> &'a [FormatElement<'a>] {
        self.elements
    }

    /// Returns the sorted Tailwind CSS classes referenced by `FormatElement::TailwindClass`.
    pub fn sorted_tailwind_classes(&self) -> &[String] {
        &self.sorted_tailwind_classes
    }
}

impl<'a> Document<'a> {
    /// Creates a new document from the given elements.
    pub fn new(
        elements: ArenaVec<'a, FormatElement<'a>>,
        sorted_tailwind_classes: Vec<String>,
    ) -> Self {
        Self { elements: elements.into_arena_slice(), sorted_tailwind_classes }
    }

    /// Consumes the document and returns its elements and sorted Tailwind CSS classes.
    pub fn into_elements_and_tailwind_classes(self) -> (&'a [FormatElement<'a>], Vec<String>) {
        (self.elements, self.sorted_tailwind_classes)
    }

    /// Finalizes and prints the document: propagates group expansion once
    /// (`Self::propagate_expand`) and hands the elements to the `Printer`.
    ///
    /// The single home of the finalize-once-then-print sequence:
    /// [`crate::Formatted::print`] and standalone raw-IR consumers
    /// (e.g. dispatched embedded IR wrapped into a temporary root) delegate here.
    ///
    /// # Errors
    /// Returns `PrintError` if the document contains invalid structure.
    pub fn print(self, source_size_hint: usize, options: PrinterOptions) -> PrintResult<Printed> {
        self.propagate_expand();
        let (elements, sorted_tailwind_classes) = self.into_elements_and_tailwind_classes();
        Printer::with_capacity(source_size_hint, options, &sorted_tailwind_classes).print(elements)
    }

    /// Like [`Self::print`], but starts at the given indentation level.
    ///
    /// # Errors
    /// Returns `PrintError` if the document contains invalid structure.
    pub fn print_with_indent(
        self,
        source_size_hint: usize,
        options: PrinterOptions,
        indent: u16,
    ) -> PrintResult<Printed> {
        self.propagate_expand();
        let (elements, sorted_tailwind_classes) = self.into_elements_and_tailwind_classes();
        Printer::with_capacity(source_size_hint, options, &sorted_tailwind_classes)
            .print_with_indent(elements, indent)
    }
}

impl Document<'_> {
    /// Sets a group's [`mode`](crate::tag::Group::mode) to [`crate::GroupMode::Propagated`] if the group contains any of:
    /// * a group whose [`mode`](crate::tag::Group::mode) is [`crate::GroupMode::Propagated`] or [`crate::GroupMode::Expand`].
    /// * a non-soft [line break](FormatElement::Line) whose [`LineMode::will_break()`](super::LineMode::will_break) is true.
    /// * a multiline [FormatElement::Text] whose `TextWidth` is not marked `without_expand_parent`.
    /// * a [FormatElement::ExpandParent]
    ///
    /// [`BestFitting`] elements act as expand boundaries, meaning that the fact that a
    /// [`BestFitting`]'s content expands is not propagated past the [`BestFitting`] element.
    ///
    /// [`BestFitting`]: FormatElement::BestFitting
    ///
    /// Also notes what each group's content printed flat does to the line ([`Group::flat`](tag::Group::flat)),
    /// so that a fits measurement can skip it.
    /// See `format_element/flat.rs` for what it leaves out.
    pub(crate) fn propagate_expand(&self) {
        let mut propagate = Propagate {
            enclosing: Vec::new(),
            checked_interned: FxHashMap::default(),
            runs: Vec::new(),
            is_balanced: true,
        };
        propagate.elements(self);
    }
}

#[derive(Debug)]
enum Enclosing<'a> {
    Group(&'a tag::Group),
    BestFitting,
}

struct Propagate<'a> {
    enclosing: Vec<Enclosing<'a>>,
    checked_interned: FxHashMap<&'a Interned<'a>, (bool, Option<Flat>)>,
    /// The runs that enclose the current one.
    runs: Vec<Run<'a>>,
    /// Whether every group ends in the slice it starts in, as builders write them.
    /// If not, no [Flat] is noted any more.
    is_balanced: bool,
}

/// A run of elements whose [Flat] is being noted: a group's content, or a whole slice.
struct Run<'a> {
    /// `None` for a whole slice.
    group: Option<&'a tag::Group>,
    /// Index of the start tag in the slice.
    start: usize,
    /// `None` if its effect depends on where it is printed.
    flat: Option<Flat>,
    /// Open conditional content.
    conditionals: u32,
    /// The depth of the conditional content that a flat print leaves out (`if_group_breaks`), 0 if none.
    left_out: u32,
}

impl<'a> Run<'a> {
    fn new(group: Option<&'a tag::Group>, start: usize) -> Self {
        Self { group, start, flat: Some(Flat::default()), conditionals: 0, left_out: 0 }
    }

    #[inline]
    fn then(&mut self, next: impl FnOnce(Flat) -> Option<Flat>) {
        if self.left_out == 0
            && let Some(flat) = self.flat
        {
            self.flat = next(flat);
        }
    }

    fn then_flat(&mut self, next: Option<Flat>) {
        self.then(|flat| next.map(|next| flat.then(next)));
    }

    /// The run's effect depends on where it is printed.
    fn give_up(&mut self) {
        self.then(|_| None);
    }

    fn start_conditional(&mut self, condition: &tag::Condition) {
        self.conditionals += 1;
        if self.left_out == 0 {
            if condition.group_id().is_some() {
                // Depends on another group
                self.flat = None;
            } else if condition.mode() == PrintMode::Expanded {
                self.left_out = self.conditionals;
            }
        }
    }

    /// Returns `false` if there is no conditional content to end.
    fn end_conditional(&mut self) -> bool {
        if self.left_out == self.conditionals {
            self.left_out = 0;
        }
        let Some(conditionals) = self.conditionals.checked_sub(1) else { return false };
        self.conditionals = conditionals;
        true
    }
}

impl<'a> Propagate<'a> {
    fn expand_parent(&self) {
        if let Some(Enclosing::Group(group)) = self.enclosing.last() {
            group.propagate_expand();
        }
    }

    /// Ends the group whose content `run` is, at `index`, and continues the run that encloses it.
    #[inline]
    fn end_group(&mut self, run: &mut Run<'a>, index: usize) {
        if !self.is_balanced {
            return;
        }
        let (Some(group), 0, Some(parent)) = (run.group, run.conditionals, self.runs.pop()) else {
            // The group started in another slice
            self.is_balanced = false;
            return;
        };
        let content = std::mem::replace(run, parent);
        group.set_flat(content.flat, index - content.start - 1);
        // Skipping a group with an id would not note its print mode
        if group.id().is_some() || !group.mode().is_flat() {
            run.give_up();
        } else {
            run.then_flat(content.flat);
        }
    }

    /// Returns whether the elements expand, and their effect printed flat.
    fn elements(&mut self, elements: &'a [FormatElement<'a>]) -> (bool, Option<Flat>) {
        let mut run = Run::new(None, 0);
        let mut expands = false;
        for (index, element) in elements.iter().enumerate() {
            let element_expands = match element {
                FormatElement::Tag(Tag::StartGroup(group)) => {
                    self.enclosing.push(Enclosing::Group(group));
                    if self.is_balanced {
                        let mut content = Run::new(Some(group), index);
                        // Never printed or measured flat
                        if !group.mode().is_flat() {
                            content.flat = None;
                        }
                        self.runs.push(std::mem::replace(&mut run, content));
                    }
                    false
                }
                FormatElement::Tag(Tag::EndGroup) => {
                    let expands = match self.enclosing.pop() {
                        Some(Enclosing::Group(group)) => !group.mode().is_flat(),
                        _ => false,
                    };
                    self.end_group(&mut run, index);
                    expands
                }
                FormatElement::Interned(interned) => {
                    let (interned_expands, flat) =
                        if let Some(checked) = self.checked_interned.get(interned) {
                            *checked
                        } else {
                            let checked = self.elements(interned);
                            self.checked_interned.insert(interned, checked);
                            checked
                        };
                    run.then_flat(flat);
                    interned_expands
                }
                FormatElement::BestFitting(best_fitting) => {
                    self.enclosing.push(Enclosing::BestFitting);

                    // Printed flat, the most flat variant is printed
                    let (_, most_flat) = self.elements(best_fitting.most_flat());
                    for variant in &best_fitting.variants()[1..] {
                        self.elements(variant);
                    }

                    self.enclosing.pop();
                    run.then_flat(most_flat);
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
                FormatElement::Text { text: _, width } => {
                    run.then(|flat| flat.text(*width));
                    width.propagates_expand()
                }
                // `FormatElement::Token` cannot contain line breaks
                FormatElement::Token { text } => {
                    #[expect(clippy::cast_possible_truncation)]
                    let width = TextWidth::single(text.len() as u32);
                    run.then(|flat| flat.text(width));
                    false
                }
                FormatElement::Space => {
                    run.then(|flat| Some(flat.space(Lead::IfLineNotEmpty)));
                    false
                }
                FormatElement::Line(mode) => {
                    match mode {
                        LineMode::Soft => {}
                        LineMode::SoftOrSpace => run.then(|flat| Some(flat.space(Lead::Always))),
                        _ => run.give_up(),
                    }
                    mode.propagates_expand()
                }
                FormatElement::ExpandParent => {
                    run.give_up();
                    true
                }
                FormatElement::Tag(Tag::StartConditionalContent(condition)) => {
                    run.start_conditional(condition);
                    false
                }
                FormatElement::Tag(Tag::EndConditionalContent) => {
                    if !run.end_conditional() {
                        self.is_balanced = false;
                    }
                    false
                }
                FormatElement::Tag(Tag::StartLineSuffix)
                | FormatElement::LineSuffixBoundary
                | FormatElement::TailwindClass(_) => {
                    run.give_up();
                    false
                }
                // Nothing on one line: indents only apply after a line break, a fill fits on the line as a whole.
                // Listed in full, so that a new element is decided here (and in `Printer::print_flat`)
                FormatElement::Tag(
                    Tag::StartIndent
                    | Tag::EndIndent
                    | Tag::StartAlign(_)
                    | Tag::EndAlign
                    | Tag::StartPrefix(_)
                    | Tag::EndPrefix
                    | Tag::StartDedent(_)
                    | Tag::EndDedent(_)
                    | Tag::StartIndentIfGroupBreaks(_)
                    | Tag::EndIndentIfGroupBreaks(_)
                    | Tag::StartFill
                    | Tag::EndFill
                    | Tag::StartEntry
                    | Tag::EndEntry
                    | Tag::EndLineSuffix
                    | Tag::StartLabelled(_)
                    | Tag::EndLabelled
                    | Tag::StartMarkAsRoot
                    | Tag::EndMarkAsRoot,
                )
                // Replaced before printing
                | FormatElement::EmbedPlaceholder(_) => false,
            };

            if element_expands {
                expands = true;
                self.expand_parent();
            }
        }

        if run.group.is_some() || run.conditionals != 0 {
            // A group or conditional content that ends in another slice
            self.is_balanced = false;
        }
        let flat = if self.is_balanced { run.flat } else { None };

        (expands, flat)
    }
}

impl<'a> Deref for Document<'a> {
    type Target = [FormatElement<'a>];

    fn deref(&self) -> &Self::Target {
        self.elements
    }
}

impl FormatElements for [FormatElement<'_>] {
    fn will_break(&self) -> bool {
        use Tag::{EndLineSuffix, StartLineSuffix};
        let mut ignore_depth = 0usize;

        for element in self {
            match element {
                // Line suffix: Ignore its content, except for direct `Line` elements (see below)
                FormatElement::Tag(StartLineSuffix) => {
                    ignore_depth += 1;
                }
                FormatElement::Tag(EndLineSuffix) => {
                    ignore_depth -= 1;
                }
                // No `ignore_depth` guard on purpose: like Prettier's `willBreak`,
                // any always-breaking line counts — even directly inside a line suffix,
                // and independently of whether it propagates expansion
                // (`HardWithoutExpand` answers "will this print a newline" with yes too).
                FormatElement::Line(line) if line.will_break() => {
                    return true;
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
                FormatElement::Interned(interned)
                    if ignore_depth == 0 && interned.may_directly_break() =>
                {
                    return true;
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
        let _ = self.end_tag(kind);

        let mut depth = 0usize;

        traverse_slice(self, kind, &mut depth)
    }

    fn end_tag(&self, kind: TagKind) -> Option<&Tag> {
        self.last().and_then(|element| element.end_tag(kind))
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::{Allocator, ArenaVec};

    use crate::{FormatElement, FormatElements, LineMode, format_element::Interned};

    #[test]
    fn will_break_deeply_nested_interned() {
        let allocator = &Allocator::new();

        for (line, expected) in [(LineMode::Soft, false), (LineMode::Hard, true)] {
            let mut elements = ArenaVec::from_array_in([FormatElement::Line(line)], &allocator);

            // The non-breaking path must traverse each interned slice only once.
            // Repeating it after a failed match guard makes this exponential in depth.
            for _ in 0..64 {
                elements = ArenaVec::from_array_in(
                    [FormatElement::Interned(Interned::new(elements))],
                    &allocator,
                );
            }

            assert_eq!(elements.will_break(), expected);
        }
    }
}
