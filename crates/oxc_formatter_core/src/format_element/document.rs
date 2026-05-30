use std::ops::Deref;

use oxc_allocator::Vec as ArenaVec;

use super::{
    FormatElement, FormatElements,
    tag::{LabelId, Tag, TagKind},
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

    /// Replaces the document's format elements with new ones.
    ///
    /// If you have modified the elements and want to update the document,
    /// use this method to set the new elements.
    pub fn replace_elements(&mut self, elements: ArenaVec<'a, FormatElement<'a>>) {
        self.elements = elements.into_arena_slice();
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
                // Line suffix
                // Ignore if any of its content breaks
                FormatElement::Tag(StartLineSuffix) => {
                    ignore_depth += 1;
                }
                FormatElement::Tag(EndLineSuffix) => {
                    ignore_depth -= 1;
                }
                FormatElement::Interned(interned) if ignore_depth == 0 && interned.will_break() => {
                    return true;
                }
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
