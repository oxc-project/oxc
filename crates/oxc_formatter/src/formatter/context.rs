use std::mem;

use oxc_allocator::Allocator;
use oxc_ast::Comment;
use oxc_span::{GetSpan, SourceType, Span};
use rustc_hash::FxHashMap;

use crate::{
    external_formatter::ExternalCallbacks, formatter::FormatElement, options::FormatOptions,
};

use super::{Comments, SourceText};

/// Entry in the Tailwind context stack, tracking whether we're inside a Tailwind class context.
#[derive(Clone, Copy, Debug)]
pub struct TailwindContextEntry {
    /// Whether to preserve whitespace (newlines) in template literals.
    pub preserve_whitespace: bool,
    /// Whether we're inside a template literal expression (between `${` and `}`).
    /// If true, we need to consider whitespace in adjacent quasis.
    pub in_template_expression: bool,
    /// Whether the quasi before this expression ends with whitespace.
    /// Only relevant when `in_template_expression` is true.
    pub quasi_before_has_trailing_ws: bool,
    /// Whether the quasi after this expression starts with whitespace.
    /// Only relevant when `in_template_expression` is true.
    pub quasi_after_has_leading_ws: bool,
    /// Whether this is the first quasi in a template literal.
    /// Used for template element boundary detection.
    pub is_first_quasi: bool,
    /// Whether this is the last quasi in a template literal.
    /// Used for template element boundary detection.
    pub is_last_quasi: bool,
    /// Whether Tailwind sorting is disabled in this context.
    /// Used to prevent sorting strings inside nested non-Tailwind call expressions.
    /// For example, in `classNames("a", x.includes("\n") ? "b" : "c")`, the `"\n"`
    /// inside `includes()` should NOT be sorted as a Tailwind class.
    pub disabled: bool,
}

impl TailwindContextEntry {
    /// Create a new context entry for JSX attributes or function calls.
    pub fn new(preserve_whitespace: bool) -> Self {
        Self {
            preserve_whitespace,
            in_template_expression: false,
            quasi_before_has_trailing_ws: true, // Default: can collapse
            quasi_after_has_leading_ws: true,   // Default: can collapse
            is_first_quasi: true,
            is_last_quasi: true,
            disabled: false,
        }
    }

    /// Create a new context entry for template literal expressions.
    /// Inherits `preserve_whitespace` from the parent context.
    pub fn template_expression(
        parent: TailwindContextEntry,
        quasi_before_has_trailing_ws: bool,
        quasi_after_has_leading_ws: bool,
    ) -> Self {
        Self {
            preserve_whitespace: parent.preserve_whitespace,
            in_template_expression: true,
            quasi_before_has_trailing_ws,
            quasi_after_has_leading_ws,
            is_first_quasi: true,
            is_last_quasi: true,
            disabled: false,
        }
    }

    /// Create a new context entry with updated quasi position.
    /// Used when formatting individual quasis to track their position in the template.
    pub fn with_quasi_position(mut self, is_first: bool, is_last: bool) -> Self {
        self.is_first_quasi = is_first;
        self.is_last_quasi = is_last;
        self
    }
}

/// Context object storing data relevant when formatting an object.
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

    /// Collected Tailwind CSS class strings from JSX attributes.
    /// These will be sorted by an external callback and replaced during printing.
    tailwind_classes: Vec<String>,

    /// Stack tracking whether we're inside a Tailwind class context.
    /// When non-empty, StringLiterals should be sorted as Tailwind classes.
    tailwind_context_stack: Vec<TailwindContextEntry>,

    external_callbacks: ExternalCallbacks,

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
            .field("quote_needed_stack", &self.quote_needed_stack)
            .field("tailwind_classes", &self.tailwind_classes)
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
        external_callbacks: Option<ExternalCallbacks>,
    ) -> Self {
        let source_text = SourceText::new(source_text);
        Self {
            options,
            source_text,
            source_type,
            comments: Comments::new(source_text, comments),
            cached_elements: FxHashMap::default(),
            quote_needed_stack: Vec::new(),
            tailwind_classes: Vec::new(),
            tailwind_context_stack: Vec::new(),
            external_callbacks: external_callbacks.unwrap_or_default(),
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
            tailwind_classes: Vec::new(),
            tailwind_context_stack: Vec::new(),
            external_callbacks: ExternalCallbacks::default(),
            allocator,
        }
    }

    /// Get the external callbacks if set
    pub fn external_callbacks(&self) -> &ExternalCallbacks {
        &self.external_callbacks
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

    /// Add a Tailwind CSS class string found in JSX attributes.
    /// Returns the index where the class was stored.
    pub fn add_tailwind_class(&mut self, class: String) -> usize {
        let index = self.tailwind_classes.len();
        self.tailwind_classes.push(class);
        index
    }
    pub fn get_tailwind_class(&self, index: usize) -> Option<&String> {
        self.tailwind_classes.get(index)
    }

    /// Take all collected Tailwind classes, clearing the internal storage.
    pub fn take_tailwind_classes(&mut self) -> Vec<String> {
        mem::take(&mut self.tailwind_classes)
    }

    /// Set the collected Tailwind CSS classes.
    pub fn set_tailwind_classes(&mut self, classes: Vec<String>) {
        self.tailwind_classes = classes;
    }

    /// Push a Tailwind context entry onto the stack.
    /// Call this when entering a JSXAttribute or CallExpression with Tailwind class context.
    pub fn push_tailwind_context(&mut self, entry: TailwindContextEntry) {
        self.tailwind_context_stack.push(entry);
    }

    /// Pop a Tailwind context entry from the stack.
    /// Call this when leaving a JSXAttribute or CallExpression with Tailwind class context.
    pub fn pop_tailwind_context(&mut self) {
        self.tailwind_context_stack.pop();
    }

    /// Get the current Tailwind context, if any.
    /// Returns `Some` if we're inside a Tailwind class context (JSXAttribute or CallExpression).
    pub fn tailwind_context(&self) -> Option<&TailwindContextEntry> {
        self.tailwind_context_stack.last()
    }

    /// Get a mutable reference to the current Tailwind context, if any.
    pub fn tailwind_context_mut(&mut self) -> Option<&mut TailwindContextEntry> {
        self.tailwind_context_stack.last_mut()
    }
}
