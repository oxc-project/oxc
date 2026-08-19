#![expect(clippy::mutable_key_type)]
use std::ops::Deref;

use oxc_allocator::ArenaVec;
use rustc_hash::FxHashMap;

use crate::{PrintResult, Printed, Printer, PrinterOptions};

use super::{
    FormatElement, FormatElements, Interned,
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
    pub(crate) fn propagate_expand(&self) {
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
                    // `FormatElement::Token` cannot contain line breaks
                    FormatElement::Text { text: _, width } => width.propagates_expand(),
                    FormatElement::ExpandParent => true,
                    FormatElement::Line(mode) => mode.propagates_expand(),
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
                FormatElement::Interned(interned) if ignore_depth == 0 && interned.will_break() => {
                    return true;
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
