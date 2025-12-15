use oxc_allocator::Allocator;
use oxc_ast::Comment;
use oxc_span::{GetSpan, SourceType, Span};
use rustc_hash::FxHashMap;

use crate::{
    embedded_formatter::EmbeddedFormatter, formatter::FormatElement, options::FormatOptions,
};

use super::{Comments, SourceText};

/// Context object storing data relevant when formatting an object.
#[derive(Clone)]
pub struct FormatContext<'ast> {
    options: FormatOptions,

    source_text: SourceText<'ast>,

    source_type: SourceType,

    comments: Comments<'ast>,

    cached_elements: FxHashMap<Span, FormatElement<'ast>>,

    /// Tracks whether quotes are needed for properties in the current object-like node.
    ///
    /// When [`FormatOptions::quote_properties`] is [`crate::QuoteProperties::Consistent`], each entry indicates
    /// whether at least one property key requires quotes. A stack is used to handle nested object-like
    /// structures (e.g., `{ a: { "b-c": 1 } }` where only the inner object needs quoted keys).
    quote_needed_stack: Vec<bool>,

    embedded_formatter: Option<EmbeddedFormatter>,

    allocator: &'ast Allocator,
}

impl std::fmt::Debug for FormatContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormatContext")
            .field("options", &self.options)
            .field("source_text", &self.source_text)
            .field("source_type", &self.source_type)
            .field("comments", &self.comments)
            .field("cached_elements", &self.cached_elements)
            .finish()
    }
}

impl<'ast> FormatContext<'ast> {
    pub fn new(
        source_text: &'ast str,
        source_type: SourceType,
        comments: &'ast [Comment],
        allocator: &'ast Allocator,
        options: FormatOptions,
        embedded_formatter: Option<EmbeddedFormatter>,
    ) -> Self {
        let source_text = SourceText::new(source_text);
        Self {
            options,
            source_text,
            source_type,
            comments: Comments::new(source_text, comments),
            cached_elements: FxHashMap::default(),
            quote_needed_stack: Vec::new(),
            embedded_formatter,
            allocator,
        }
    }

    pub(crate) fn dummy(allocator: &'ast Allocator) -> Self {
        Self {
            options: FormatOptions::default(),
            source_text: SourceText::new(""),
            source_type: SourceType::default(),
            comments: Comments::new(SourceText::new(""), &[]),
            cached_elements: FxHashMap::default(),
            quote_needed_stack: Vec::new(),
            embedded_formatter: None,
            allocator,
        }
    }

    /// Get the embedded formatter if one is set
    pub fn embedded_formatter(&self) -> Option<&EmbeddedFormatter> {
        self.embedded_formatter.as_ref()
    }

    /// Returns the formatting options
    pub fn options(&self) -> &FormatOptions {
        &self.options
    }

    /// Returns a reference to the program's comments.
    pub fn comments(&self) -> &Comments<'ast> {
        &self.comments
    }

    /// Returns a reference to the program's comments.
    pub fn comments_mut(&mut self) -> &mut Comments<'ast> {
        &mut self.comments
    }

    /// Returns the source text wrapper
    pub fn source_text(&self) -> SourceText<'ast> {
        self.source_text
    }

    /// Returns the source type
    pub fn source_type(&self) -> SourceType {
        self.source_type
    }

    /// Returns the cached formatted element for the given key.
    pub(crate) fn get_cached_element<T: GetSpan>(&self, key: &T) -> Option<FormatElement<'ast>> {
        self.cached_elements.get(&key.span()).cloned()
    }

    /// Caches the formatted element for the given key.
    pub(crate) fn cache_element<T: GetSpan>(&mut self, key: &T, formatted: FormatElement<'ast>) {
        self.cached_elements.insert(key.span(), formatted);
    }

    /// Pushes a new quote needed state onto the stack.
    pub fn push_quote_needed(&mut self, needed: bool) {
        debug_assert!(
            self.options.quote_properties.is_consistent(),
            "`push_quote_needed` should only be used when `self.options.quote_properties.is_consistent()` is true"
        );
        self.quote_needed_stack.push(needed);
    }

    /// Pops the top quote needed state from the stack.
    pub fn pop_quote_needed(&mut self) {
        debug_assert!(
            self.options.quote_properties.is_consistent(),
            "`pop_quote_needed` should only be used when `self.options.quote_properties.is_consistent()` is true"
        );
        self.quote_needed_stack.pop();
    }

    pub fn is_quote_needed(&self) -> bool {
        *self.quote_needed_stack.last().unwrap_or(&false)
    }

    pub fn allocator(&self) -> &'ast Allocator {
        self.allocator
    }
}
