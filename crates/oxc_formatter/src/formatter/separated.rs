use oxc_span::Span;

use crate::{
    formatter::{
        Buffer, Format, FormatError, FormatResult, Formatter,
        prelude::{group, if_group_breaks, text},
        trivia::format_only_if_breaks,
    },
    options::TrailingSeparator,
};

use super::{GroupId, SyntaxToken};

/// Formats a single element inside a separated list.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormatSeparatedElement<E> {
    element: E,
    is_last: bool,
    /// The separator to write if the element has no separator yet.
    separator: &'static str,
    options: FormatSeparatedOptions,
}

impl<'a, E: Format<'a>> Format<'a> for FormatSeparatedElement<E> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.options.nodes_grouped {
            group(&self.element).fmt(f)?;
        } else {
            self.element.fmt(f)?;
        }
        if self.is_last {
            match self.options.trailing_separator {
                TrailingSeparator::Allowed => format_only_if_breaks(&SyntaxToken, &self.separator)
                    .with_group_id(self.options.group_id)
                    .fmt(f),
                TrailingSeparator::Mandatory => self.separator.fmt(f),
                TrailingSeparator::Disallowed | TrailingSeparator::Omit => Ok(()),
            }
        } else {
            self.separator.fmt(f)
        }
    }
}

/// Iterator for formatting separated elements. Prints the separator between each element and
/// inserts a trailing separator if necessary
pub struct FormatSeparatedIter<I, E> {
    next: Option<E>,
    inner: I,
    separator: &'static str,
    options: FormatSeparatedOptions,
}

impl<I, E> FormatSeparatedIter<I, E> {
    pub fn new(inner: I, separator: &'static str) -> Self {
        Self { inner, separator, next: None, options: FormatSeparatedOptions::default() }
    }

    /// Wraps every node inside of a group
    pub fn nodes_grouped(mut self) -> Self {
        self.options.nodes_grouped = true;
        self
    }

    pub fn with_trailing_separator(mut self, separator: TrailingSeparator) -> Self {
        self.options.trailing_separator = separator;
        self
    }

    pub fn with_group_id(mut self, group_id: Option<GroupId>) -> Self {
        self.options.group_id = group_id;
        self
    }
}

impl<I, E> Iterator for FormatSeparatedIter<I, E>
where
    I: Iterator<Item = E>,
{
    type Item = FormatSeparatedElement<E>;

    fn next(&mut self) -> Option<Self::Item> {
        let element = self.next.take().or_else(|| self.inner.next())?;
        self.next = self.inner.next();
        let is_last = self.next.is_none();
        Some(FormatSeparatedElement {
            element,
            is_last,
            separator: self.separator,
            options: self.options,
        })
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct FormatSeparatedOptions {
    trailing_separator: TrailingSeparator,
    group_id: Option<GroupId>,
    nodes_grouped: bool,
}
