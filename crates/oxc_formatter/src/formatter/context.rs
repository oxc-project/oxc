use oxc_ast::Comment;
use oxc_formatter_core::{FormatElement, SourceText};
use oxc_span::{GetSpan, SourceType, Span};
use rustc_hash::FxHashMap;

use crate::{
    ExpressionRoot,
    ast_nodes::AstNodes,
    options::{JsFormatOptions, QuoteStyle},
    utils::assignment_like::AssignmentLikeLayout,
};

use super::Comments;

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
        }
    }

    /// Create a new context entry with updated quasi position.
    /// Used when formatting individual quasis to track their position in the template.
    #[must_use]
    pub fn with_quasi_position(mut self, is_first: bool, is_last: bool) -> Self {
        self.is_first_quasi = is_first;
        self.is_last_quasi = is_last;
        self
    }
}

/// Context object storing data relevant when formatting an object.
pub struct JsFormatContext<'ast> {
    options: JsFormatOptions,
    source_text: SourceText<'ast>,
    source_type: SourceType,
    comments: Comments<'ast>,

    cached_elements: FxHashMap<Span, FormatElement<'ast>>,
    /// One-shot handoff of the assignment layout to the arrow expression on the RHS of an assignment-like,
    /// keyed by the arrow's span so no other node can consume it.
    /// Set (and cleared) by `WithAssignmentLayout` around formatting the arrow, taken by the arrow's `write`.
    arrow_assignment_layout: Option<(Span, AssignmentLikeLayout)>,

    /// Tracks whether quotes are needed for properties in the current object-like node.
    ///
    /// When [`JsFormatOptions::quote_properties`] is [`crate::QuoteProperties::Consistent`], each entry indicates
    /// whether at least one property key requires quotes. A stack is used to handle nested object-like
    /// structures (e.g., `{ a: { "b-c": 1 } }` where only the inner object needs quoted keys).
    quote_needed_stack: Vec<bool>,

    /// Stack tracking whether we're inside a Tailwind class context, read by [`Self::tailwind_context`].
    /// - `None` hides the context below it, inside a nested non-Tailwind call
    tailwind_context_stack: Vec<Option<TailwindContextEntry>>,

    embedding: Embedding,
}

/// What an embedded part's host reads of it ([`crate::format_to_ir`]); a file sets none of it.
#[derive(Default)]
struct Embedding {
    /// How its top-level statements end.
    top_level_semicolons: TopLevelSemicolons,
    /// The end of the top-level statement whose `;` [`TopLevelSemicolons::Separated`] writes, until a statement ending there prints it
    /// (see [`JsFormatContext::take_separator_at`]).
    separator_end: Option<u32>,
    /// See [`JsFormatContext::in_html_attribute`].
    in_html_attribute: bool,
    /// The part is an expression ([`crate::JsRoot::Expression`]).
    expression_root: Option<ExpressionRoot>,
}

/// How an embedded program's top-level statements end.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TopLevelSemicolons {
    /// As `semi` says.
    #[default]
    AsOptions,
    /// The expression statement (by its span) prints without its semicolon, and no ASI guard.
    Omitted(Span),
    /// A `;` the program writes ends each statement but the last, so none needs an ASI guard (`semi: false`).
    Separated,
}

impl std::fmt::Debug for JsFormatContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsFormatContext")
            .field("options", &self.options)
            .field("source_text", &self.source_text)
            .field("source_type", &self.source_type)
            .field("comments", &self.comments)
            .field("cached_elements", &self.cached_elements)
            .field("quote_needed_stack", &self.quote_needed_stack)
            .finish()
    }
}

impl oxc_formatter_core::FormatContext for JsFormatContext<'_> {
    type Options = JsFormatOptions;

    fn options(&self) -> &JsFormatOptions {
        &self.options
    }

    fn source_code(&self) -> &str {
        &self.source_text
    }
}

impl<'ast> JsFormatContext<'ast> {
    pub fn new(
        source_text: &'ast str,
        source_type: SourceType,
        comments: &'ast [Comment],
        options: JsFormatOptions,
    ) -> Self {
        let source_text = SourceText::new(source_text);
        Self {
            options,
            source_text,
            source_type,
            comments: Comments::new(source_text, comments),
            cached_elements: FxHashMap::default(),
            arrow_assignment_layout: None,
            quote_needed_stack: Vec::new(),
            tailwind_context_stack: Vec::new(),
            embedding: Embedding::default(),
        }
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

    /// See the [`Self::arrow_assignment_layout`] field.
    pub(crate) fn set_arrow_assignment_layout(&mut self, span: Span, layout: AssignmentLikeLayout) {
        debug_assert!(
            self.arrow_assignment_layout.is_none(),
            "a previous arrow assignment layout was neither taken nor cleared"
        );
        self.arrow_assignment_layout = Some((span, layout));
    }

    /// See the [`Self::arrow_assignment_layout`] field.
    pub(crate) fn take_arrow_assignment_layout(
        &mut self,
        span: Span,
    ) -> Option<AssignmentLikeLayout> {
        match self.arrow_assignment_layout {
            Some((key, layout)) if key == span => {
                self.arrow_assignment_layout = None;
                Some(layout)
            }
            _ => None,
        }
    }

    /// See the [`Self::arrow_assignment_layout`] field.
    /// Clears a layout left behind when the arrow was printed without running
    /// `write` (a suppressed arrow prints its source verbatim instead).
    pub(crate) fn clear_arrow_assignment_layout(&mut self) {
        self.arrow_assignment_layout = None;
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

    // An embedded part's state (`Embedding`)

    pub(crate) fn set_top_level_semicolons(&mut self, semicolons: TopLevelSemicolons) {
        self.embedding.top_level_semicolons = semicolons;
    }

    pub(crate) fn top_level_semicolons(&self) -> TopLevelSemicolons {
        self.embedding.top_level_semicolons
    }

    pub(crate) fn set_separator_end(&mut self, end: u32) {
        self.embedding.separator_end = Some(end);
    }

    /// A statement ending at `end` prints the `;` [`TopLevelSemicolons::Separated`] writes as its own,
    /// whatever `semi` says, so its comments move behind it as any terminator's do.
    /// `true` once, for the innermost statement ending there (`if (a) b()`'s `b()`).
    pub(crate) fn take_separator_at(&mut self, end: u32) -> bool {
        let taken = self.embedding.separator_end == Some(end);
        if taken {
            self.embedding.separator_end = None;
        }
        taken
    }

    pub(crate) fn set_in_html_attribute(&mut self, in_html_attribute: bool) {
        self.embedding.in_html_attribute = in_html_attribute;
    }

    /// The part is a `"`-quoted HTML attribute's value ([`crate::JsEmbeddedIn::in_html_attribute`]):
    /// strings print in `'`, whatever they hold.
    pub(crate) fn in_html_attribute(&self) -> bool {
        self.embedding.in_html_attribute
    }

    pub(crate) fn set_expression_root(&mut self, root: ExpressionRoot) {
        self.embedding.expression_root = Some(root);
    }

    /// `node` is the parent of an embedded expression's root ([`crate::JsRoot::Expression`]): the `Program` it prints under,
    /// whose wrapping statement is not printed. A file has none.
    ///
    /// Not an `AstNodes` variant of its own: one costs every node, as `AstNodes::span` no longer inlines.
    #[inline]
    pub(crate) fn is_embedded_root(&self, node: &AstNodes<'_, '_>) -> bool {
        matches!(node, AstNodes::Program(_)) && self.embedding.expression_root.is_some()
    }

    pub(crate) fn expression_root(&self) -> Option<ExpressionRoot> {
        self.embedding.expression_root
    }

    /// The quote a string prints in, the formatter's own too (a quoted property key):
    /// the configured one, but `'` in an HTML attribute.
    /// A JSX attribute and a directive have their own (`FormatLiteralStringToken`).
    pub(crate) fn string_quote_style(&self) -> QuoteStyle {
        if self.in_html_attribute() { QuoteStyle::Single } else { self.options.quote_style }
    }

    /// Push a Tailwind context entry onto the stack.
    /// Call this when entering a JSXAttribute or CallExpression with Tailwind class context.
    pub fn push_tailwind_context(&mut self, entry: TailwindContextEntry) {
        self.tailwind_context_stack.push(Some(entry));
    }

    /// Hide the current Tailwind context until the matching [`Self::pop_tailwind_context`].
    /// Call this when entering a nested non-Tailwind CallExpression.
    pub fn hide_tailwind_context(&mut self) {
        self.tailwind_context_stack.push(None);
    }

    /// Pop a Tailwind context entry from the stack.
    /// Call this when leaving a JSXAttribute or CallExpression with Tailwind class context.
    pub fn pop_tailwind_context(&mut self) {
        self.tailwind_context_stack.pop();
    }

    /// Get the current Tailwind context, if any.
    /// Returns `Some` if we're inside a Tailwind class context (JSXAttribute or CallExpression),
    /// and not inside a nested non-Tailwind call (see [`Self::hide_tailwind_context`]).
    pub fn tailwind_context(&self) -> Option<&TailwindContextEntry> {
        self.tailwind_context_stack.last().and_then(Option::as_ref)
    }
}
