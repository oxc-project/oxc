use std::{borrow::Cow, fmt, num::NonZeroU8};

#[derive(Default, Clone, Copy)]
#[non_exhaustive]
#[must_use]
pub struct CodegenOptions {
    /// Use single quotes instead of double quotes.
    ///
    /// Default `false`
    pub single_quote: bool,
    /// What kind of whitespace to use for indentation.
    ///
    /// Default [`Indent::Tabs`]
    pub indent: Indent,
}

impl CodegenOptions {
    pub const fn new() -> Self {
        Self { single_quote: false, indent: Indent::Tabs }
    }

    #[inline]
    pub const fn with_quote(mut self, single_quote: bool) -> Self {
        self.single_quote = single_quote;
        self
    }

    /// Prefer single quotes for strings.
    #[inline]
    pub const fn with_single_quotes(mut self) -> Self {
        self.single_quote = true;
        self
    }

    /// Prefer double quotes for strings.
    #[inline]
    pub const fn with_double_quotes(mut self) -> Self {
        self.single_quote = false;
        self
    }

    /// Configure the indentation style.
    #[inline]
    pub const fn with_indent(mut self, indent: Indent) -> Self {
        self.indent = indent;
        self
    }

    /// Indent with [tabs].
    ///
    /// # Example
    /// ```
    /// use oxc_codegen::CodegenOptions;
    ///
    /// const TABS: CodegenOptions = CodegenOptions::new().with_tabs();
    /// ```
    ///
    /// [tabs]: Indent::Tabs
    #[inline]
    pub const fn with_tabs(mut self) -> Self {
        self.indent = Indent::Tabs;
        self
    }

    /// Indent with `n` [spaces].
    ///
    /// # Example
    /// ```
    /// use oxc_codegen::CodegenOptions;
    ///
    /// const FOUR_SPACES: CodegenOptions = CodegenOptions::new().with_spaces(4);
    /// ```
    ///
    /// [spaces]: Indent::Spaces
    ///
    #[inline]
    pub const fn with_spaces(mut self, spaces: NonZeroU8) -> Self {
        self.indent = Indent::Spaces(spaces);
        self
    }
}

#[derive(Default, Clone, Copy)]
pub struct CommentOptions {
    /// Enable preserve annotate comments, like `/* #__PURE__ */` and `/* #__NO_SIDE_EFFECTS__ */`.
    pub preserve_annotate_comments: bool,
}

/// How to indent code. Supports both tabs and spaces.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Indent {
    /// Indent with tabs
    #[default]
    Tabs,
    /// Indent with `n` spaces.
    ///
    /// If you are creating an [`Indent`] with a predetermined indentation
    /// level, you may find [`Indent::spaces()`] more convenient.
    ///
    /// Note that you cannot indent with zero spaces; in this case, use a
    /// [minifying codegen] instead.
    ///
    /// [minifying codegen]: crate::WhitespaceRemover
    Spaces(NonZeroU8),
}

impl Indent {
    /// Convenience function to create an [`Indent`] with a [`u8`] when you are
    /// certain it will not be zero.
    ///
    /// # Panics
    /// if `n` is zero.
    ///
    /// # Example
    ///
    /// ```
    /// # fn setting_from_user_input() -> u8 {
    /// #   2
    /// # }
    /// use std::num::NonZeroU8;
    /// use oxc_codegen::Indent;
    ///
    /// // It's safe to use `spaces()` b/c we constantly provide a non-zero value.
    /// const TWO_SPACES: Indent = Indent::spaces(2);
    /// // When you're setting indentation at runtime, use Indent::Spaces(n) instead
    /// let size_not_guaranteed: u8 = setting_from_user_input();
    /// let safe_spaces = Indent::Spaces(NonZeroU8::new(size_not_guaranteed).unwrap());
    /// ```
    pub const fn spaces(n: u8) -> Self {
        assert!(n > 0, "Cannot create an Indent with zero spaces");

        #[allow(unsafe_code)]
        // SAFETY: assertion above guarantees `n` is not zero. We cannot use the
        // safe constructor and unwrap here because const usage of unwrap() and
        // expect() requires nightly rust.
        Indent::Spaces(unsafe { NonZeroU8::new_unchecked(n) })
    }

    /// Get the number of bytes that this indentation represents.
    ///
    /// Indentation length will never be 0, so it is safe to create non-zero
    /// numeric types from. This is also why [`Indent`] has no `is_empty` method.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_codegen::Indent;
    ///
    /// assert_eq!(Indent::Tabs.len(), 1);
    /// assert_eq!(Indent::spaces(2).len(), 2);
    /// assert_eq!(Indent::spaces(4).len(), 4);
    /// ```
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        match self {
            Indent::Tabs => 1,
            Indent::Spaces(n) => n.get() as usize,
        }
    }

    /// Check if this indentation is tabs.
    ///
    /// # Example
    /// ```
    /// use oxc_codegen::Indent;
    ///
    /// assert!(Indent::Tabs.is_tabs());
    /// assert!(!Indent::spaces(2).is_tabs());
    /// ```
    pub const fn is_tabs(&self) -> bool {
        matches!(self, Indent::Tabs)
    }

    /// Check if this indentation is using spaces.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_codegen::Indent;
    ///
    /// assert!(!Indent::Tabs.is_spaces());
    /// assert!(Indent::spaces(2).is_spaces());
    /// ```
    pub const fn is_spaces(&self) -> bool {
        matches!(self, Indent::Spaces(_))
    }

    /// Get a string representation of this indentation.
    ///
    /// # Example
    /// ```
    /// use oxc_codegen::Indent;
    ///
    /// assert_eq!(Indent::Tabs.as_str(), "\t");
    /// assert_eq!(Indent::spaces(2).as_str(), "  ");
    /// ```
    pub fn as_str(self) -> Cow<'static, str> {
        match self {
            Indent::Tabs => Cow::Borrowed("\t"),
            Indent::Spaces(n) => {
                match n.get() {
                    // Create borrowed strings for common cases
                    0 => unreachable!(),
                    2 => Cow::Borrowed("  "),
                    4 => Cow::Borrowed("    "),
                    n => Cow::Owned(" ".repeat(n as usize)),
                }
            }
        }
    }
}

impl fmt::Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Indent::Tabs => write!(f, "\t"),
            Indent::Spaces(n) => write!(f, "{:width$}", "", width = n.get() as usize),
        }
    }
}

impl From<u8> for Indent {
    fn from(n: u8) -> Self {
        if n == 0 {
            Indent::Tabs
        } else {
            #[allow(unsafe_code)]
            // SAFETY: `n` is definitely not zero because of the check above
            Indent::Spaces(unsafe { NonZeroU8::new_unchecked(n) })
        }
    }
}

impl IntoIterator for Indent {
    type Item = u8;
    type IntoIter = IndentIter;

    /// Creates an iterator that yields ASCII bytes for indentation whitespace.
    ///
    /// See the [iter module documentation] for more information.
    ///
    /// [iter module documentation]: core::iter
    fn into_iter(self) -> Self::IntoIter {
        IndentIter::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct IndentIter {
    indent: Indent,
    index: u8,
}
impl IndentIter {
    fn new(indent: Indent) -> Self {
        Self { indent, index: 0 }
    }
}

impl Iterator for IndentIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.indent {
            Indent::Tabs => {
                if self.index == 0 {
                    self.index = 1;
                    Some(b'\t')
                } else {
                    None
                }
            }
            Indent::Spaces(n) => {
                if self.index < n.get() {
                    self.index += 1;
                    Some(b' ')
                } else {
                    None
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.indent {
            Indent::Tabs => (1, Some(1)),
            Indent::Spaces(n) => (n.get() as usize, Some(n.get() as usize)),
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        match self.indent {
            Indent::Tabs => {
                debug_assert!(self.index <= 1);
                (1 - self.index) as usize
            }
            Indent::Spaces(n) => {
                let n = n.get();
                debug_assert!(self.index <= n);
                (n - self.index) as usize
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default() {
        let default = CodegenOptions::default();
        assert!(!default.single_quote);
        assert!(default.indent.is_tabs());
    }

    #[test]
    #[should_panic]
    fn zero_spaces() {
        let _ = Indent::spaces(0);
    }

    #[test]
    fn indent_iter() {
        let tabs: Vec<u8> = Indent::Tabs.into_iter().collect();
        assert_eq!(tabs, vec![b'\t']);

        let spaces: Vec<u8> = Indent::spaces(2).into_iter().collect();
        assert_eq!(spaces, vec![b' ', b' ']);

        let spaces: Vec<u8> = Indent::spaces(4).into_iter().collect();
        assert_eq!(spaces, vec![b' ', b' ', b' ', b' ']);
    }

    #[test]
    fn test_equality_and_len() {
        let tabs = Indent::Tabs;
        let spaces2 = Indent::spaces(2);
        let spaces4 = Indent::spaces(4);

        assert!(tabs.is_tabs());
        assert_eq!(tabs.len(), 1);

        assert!(spaces2.is_spaces());
        assert_eq!(spaces2.len(), 2);
        assert_eq!(spaces4.len(), 4);

        assert_eq!(tabs, tabs);
        assert_eq!(tabs, Indent::Tabs);
        assert_eq!(spaces2, Indent::spaces(2));

        assert_ne!(spaces2, tabs);
        assert_ne!(spaces2, spaces4);
    }

    #[test]
    fn test_printing() {
        assert_eq!(format!("{}", Indent::Tabs), "\t");
        assert_eq!(Indent::Tabs.as_str(), "\t");

        assert_eq!(format!("{}", Indent::spaces(2)), "  ");
        assert_eq!(Indent::spaces(2).as_str(), "  ");

        assert_eq!(format!("{}", Indent::spaces(4)), "    ");
        assert_eq!(Indent::spaces(4).as_str(), "    ");
    }
}
