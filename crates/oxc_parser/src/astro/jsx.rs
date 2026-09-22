//! Astro-specific JSX parsing.
//!
//! This module contains Astro-specific overrides of JSX parsing functions.
//! These duplicate the logic from `jsx/mod.rs` with Astro-specific behavior
//! (void elements, raw text elements, foreign content, HTML comments, etc.)
//! so that the standard JSX code remains free of `#[cfg(feature = "astro")]` blocks.
//!
//! Functions are named `parse_astro_*` to avoid conflicts with the standard JSX
//! functions. The Astro entry points call these instead of the standard versions.

use oxc_allocator::{Allocator, Box, Dummy, Vec};
use oxc_ast::ast::*;
use oxc_span::{Atom, GetSpan, Span};

use crate::{ParserImpl, config::ParserConfig, diagnostics, lexer::Kind};

/// Represents either a closing JSX element or fragment (Astro copy).
enum JSXClosing<'a> {
    /// [`JSXClosingElement`]
    Element(Box<'a, JSXClosingElement<'a>>),
    /// [`JSXClosingFragment`]
    Fragment(JSXClosingFragment),
}

impl<'a> Dummy<'a> for JSXClosing<'a> {
    fn dummy(allocator: &'a Allocator) -> Self {
        JSXClosing::Fragment(Dummy::dummy(allocator))
    }
}

impl<'a, C: ParserConfig> ParserImpl<'a, C> {
    // ==================== Astro-specific JSX entry points ====================
    //
    // These are Astro-specific versions of the core JSX parsing functions.
    // They handle void elements, raw text elements, foreign content, HTML comments,
    // `<script>` special handling, shorthand attributes, etc.
    //
    // They are named `parse_astro_*` to avoid conflicts with the standard
    // `parse_jsx_*` functions in `jsx/mod.rs`.

    /// Astro-specific version of `parse_jsx_expression`.
    /// Handles `<script>` special handling in expression context.
    pub(crate) fn parse_astro_jsx_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `<`
        let kind = self.cur_kind();
        if kind == Kind::RAngle {
            Expression::JSXFragment(self.parse_astro_jsx_fragment(span, false))
        } else if kind.is_identifier_or_keyword() {
            // In Astro, check for <script> which needs special handling
            if self.cur_src() == "script" {
                // parse_astro_script_in_jsx returns JSXChild, convert to Expression
                return match self.parse_astro_script_in_jsx(span, false) {
                    JSXChild::Element(el) => Expression::JSXElement(el),
                    JSXChild::AstroScript(script) => {
                        // Wrap AstroScript in a synthetic JSX element for expression context
                        let script_span = script.span;
                        let name = self
                            .ast
                            .jsx_identifier(oxc_span::Span::new(span + 1, span + 7), "script");
                        let elem_name = JSXElementName::Identifier(self.alloc(name));
                        let opening = self.ast.alloc_jsx_opening_element(
                            script_span,
                            elem_name,
                            Option::<Box<'a, oxc_ast::ast::TSTypeParameterInstantiation<'a>>>::None,
                            self.ast.vec(),
                        );
                        Expression::JSXElement(self.ast.alloc_jsx_element(
                            script_span,
                            opening,
                            self.ast.vec1(JSXChild::AstroScript(script)),
                            Option::<Box<'a, JSXClosingElement<'a>>>::None,
                        ))
                    }
                    _ => unreachable!("parse_astro_script_in_jsx returns Element or AstroScript"),
                };
            }
            Expression::JSXElement(self.parse_astro_jsx_element(span, false))
        } else {
            self.unexpected()
        }
    }

    /// Astro-specific version of `parse_jsx_fragment`.
    /// Delegates to the standard `parse_jsx_fragment` for most logic but uses
    /// Astro-specific children parsing.
    pub(crate) fn parse_astro_jsx_fragment(
        &mut self,
        span: u32,
        in_jsx_child: bool,
    ) -> Box<'a, JSXFragment<'a>> {
        self.expect_jsx_child(Kind::RAngle);
        let opening_fragment = self.ast.jsx_opening_fragment(self.end_span(span));
        let (children, closing) = self.parse_astro_jsx_children_and_closing(in_jsx_child);
        let closing_fragment = match closing {
            JSXClosing::Fragment(f) => f,
            JSXClosing::Element(e) => {
                self.error(diagnostics::jsx_fragment_no_match(
                    opening_fragment.span,
                    e.name.span(),
                ));
                self.ast.jsx_closing_fragment(e.span)
            }
        };
        self.ast.alloc_jsx_fragment(
            self.end_span(span),
            opening_fragment,
            children,
            closing_fragment,
        )
    }

    /// Astro-specific version of `parse_jsx_element`.
    /// Handles raw text elements, void elements, and foreign content.
    pub(crate) fn parse_astro_jsx_element(
        &mut self,
        span: u32,
        in_jsx_child: bool,
    ) -> Box<'a, JSXElement<'a>> {
        let (opening_element, self_closing, is_raw_text_element, prev_no_expression) =
            self.parse_astro_jsx_opening_element(span, in_jsx_child);

        let (children, closing_element) = if self_closing {
            (self.ast.vec(), None)
        } else {
            // Track this element as open so nested children can detect stray
            // closing tags (e.g. an extra `</div>`) that match no open element.
            self.astro_open_elements
                .push(opening_element.name.span().source_text(self.source_text));

            // Raw text elements (script/style/is:raw): content is raw text, not JSX children.
            let (children, closing) = if is_raw_text_element {
                let children =
                    self.skip_astro_raw_text_element_content(&opening_element.name, in_jsx_child);
                // Restore the no-expression flag
                self.lexer.no_expression_in_jsx_children = prev_no_expression;
                // Parse `</name>` closing tag
                let closing_span = self.start_span();
                self.bump_any(); // bump `<`
                self.bump_any(); // bump `/`
                let closing = self.parse_astro_jsx_closing_inline(closing_span, in_jsx_child);
                (children, closing)
            } else {
                let result = self.parse_astro_jsx_children_and_closing(in_jsx_child);
                self.lexer.no_expression_in_jsx_children = prev_no_expression;
                result
            };

            // This element is no longer open.
            let _ = self.astro_open_elements.pop();

            let closing_element = match closing {
                JSXClosing::Element(e) => {
                    if !Self::astro_jsx_element_name_eq(&opening_element.name, &e.name) {
                        self.error(diagnostics::jsx_element_no_match(
                            opening_element.name.span(),
                            e.name.span(),
                            opening_element.name.span().source_text(self.source_text),
                        ));
                    }
                    e
                }
                JSXClosing::Fragment(f) => {
                    return self.fatal_error(diagnostics::jsx_element_no_match(
                        opening_element.name.span(),
                        f.span,
                        opening_element.name.span().source_text(self.source_text),
                    ));
                }
            };
            (children, Some(closing_element))
        };
        self.ast.alloc_jsx_element(self.end_span(span), opening_element, children, closing_element)
    }

    /// Astro-specific version of `parse_jsx_opening_element`.
    /// Returns (opening_element, self_closing, is_raw_text_element, prev_no_expression).
    fn parse_astro_jsx_opening_element(
        &mut self,
        span: u32,
        in_jsx_child: bool,
    ) -> (
        Box<'a, JSXOpeningElement<'a>>,
        bool, // `true` if self-closing
        bool, // `true` if raw text element (script/style)
        bool, // previous value of no_expression_in_jsx_children (to restore on close)
    ) {
        let name = self.parse_astro_jsx_element_name();
        let type_arguments = if self.is_ts { self.try_parse_type_arguments() } else { None };
        let attributes = self.parse_astro_jsx_attributes();
        let explicit_self_closing = self.eat(Kind::Slash);

        // HTML void elements are implicitly self-closing even without `/`
        let self_closing = explicit_self_closing || Self::is_astro_void_element(&name);

        // Check if this is a raw text element
        let is_raw_text =
            Self::is_astro_raw_text_element(&name) || Self::has_is_raw_attribute(&attributes);

        // For foreign content elements like <math>, set the no-expression flag
        let is_foreign = Self::is_foreign_content_element(&name);
        let prev_no_expression = self.lexer.no_expression_in_jsx_children;
        if is_foreign && !self_closing {
            self.lexer.no_expression_in_jsx_children = true;
        }

        // For raw text elements, don't advance past `>` — the caller will reposition the lexer.
        if is_raw_text && !explicit_self_closing {
            self.expect_without_advance(Kind::RAngle);
            self.prev_token_end = self.cur_token().span().end;
        } else if !self_closing || in_jsx_child {
            self.expect_jsx_child(Kind::RAngle);
        } else {
            self.expect(Kind::RAngle);
        }
        let elem = self.ast.alloc_jsx_opening_element(
            self.end_span(span),
            name,
            type_arguments,
            attributes,
        );
        (elem, self_closing, is_raw_text, prev_no_expression)
    }

    /// Astro-specific version of `parse_jsx_element_name`.
    /// In Astro, namespaced element names are not supported.
    fn parse_astro_jsx_element_name(&mut self) -> JSXElementName<'a> {
        let span = self.start_span();
        let identifier = self.parse_jsx_identifier();

        // In Astro, namespaced element names are not supported — don't parse `<ns:tag>`.

        // <member.foo.bar />
        if self.at(Kind::Dot) {
            return JSXElementName::MemberExpression(
                self.parse_jsx_member_expression(span, &identifier),
            );
        }

        if self.fatal_error.is_some() {
            return JSXElementName::dummy(self.ast.allocator);
        }

        let name = identifier.name.as_str();
        let is_reference = match name.as_bytes()[0] {
            b if b.is_ascii() => !b.is_ascii_lowercase(),
            _ => true,
        } && !name.contains('-');

        if is_reference {
            let identifier = self.ast.alloc_identifier_reference(identifier.span, identifier.name);
            JSXElementName::IdentifierReference(identifier)
        } else if name == "this" {
            JSXElementName::ThisExpression(self.ast.alloc_this_expression(identifier.span))
        } else {
            JSXElementName::Identifier(self.alloc(identifier))
        }
    }

    /// Astro-specific version of `parse_jsx_children_and_closing`.
    /// Handles `<script>` tags, HTML comments, and `<!` constructs.
    fn parse_astro_jsx_children_and_closing(
        &mut self,
        in_jsx_child: bool,
    ) -> (Vec<'a, JSXChild<'a>>, JSXClosing<'a>) {
        let mut children = self.ast.vec();
        loop {
            if self.fatal_error.is_some() {
                let closing = self.ast.jsx_closing_fragment(self.cur_token().span());
                return (children, JSXClosing::Fragment(closing));
            }

            match self.cur_kind() {
                Kind::LAngle => {
                    let span = self.start_span();
                    self.bump_any(); // bump `<`
                    let kind = self.cur_kind();

                    // <> open nested fragment
                    if kind == Kind::RAngle {
                        children
                            .push(JSXChild::Fragment(self.parse_astro_jsx_fragment(span, true)));
                        continue;
                    }

                    // <ident open nested element
                    if kind == Kind::Ident || kind.is_any_keyword() {
                        // Check for <script> which needs special handling
                        if self.cur_src() == "script" {
                            children.push(self.parse_astro_script_in_jsx(span, true));
                            continue;
                        }
                        children.push(JSXChild::Element(self.parse_astro_jsx_element(span, true)));
                        continue;
                    }

                    // <! in Astro - HTML comment or doctype
                    if kind == Kind::Bang {
                        if let Some(child) = self.parse_html_comment_in_jsx(span) {
                            children.push(child);
                            continue;
                        }
                        return (children, self.unexpected());
                    }

                    // </ closing tag
                    if kind == Kind::Slash {
                        self.bump_any(); // bump `/`
                        let closing = self.parse_astro_jsx_closing_inline(span, in_jsx_child);
                        // A named closing tag that matches no currently-open element is
                        // stray (e.g. an extra `</div>`). Report it at its own location
                        // and skip it, so the surrounding elements still pair up correctly
                        // instead of blaming an unrelated ancestor.
                        if let JSXClosing::Element(e) = &closing {
                            let closing_name = e.name.span().source_text(self.source_text);
                            if !self.astro_open_elements.contains(&closing_name) {
                                self.error(diagnostics::jsx_unexpected_closing_tag(
                                    e.name.span(),
                                    closing_name,
                                ));
                                continue;
                            }
                        }
                        return (children, closing);
                    }

                    return (children, self.unexpected());
                }
                Kind::LCurly => {
                    let span_start = self.start_span();
                    self.bump_any(); // bump `{`

                    if self.eat(Kind::Dot3) {
                        children.push(JSXChild::Spread(self.parse_jsx_spread_child(span_start)));
                        continue;
                    }
                    children.push(JSXChild::ExpressionContainer(
                        self.parse_astro_jsx_expression_container(
                            span_start, /* in_jsx_child */ true,
                        ),
                    ));
                }
                Kind::JSXText => {
                    children.push(JSXChild::Text(self.parse_jsx_text()));
                }
                _ => {
                    return (children, self.unexpected());
                }
            }
        }
    }

    /// Astro-specific version of `parse_jsx_closing_inline`.
    fn parse_astro_jsx_closing_inline(
        &mut self,
        open_angle_span: u32,
        in_jsx_child: bool,
    ) -> JSXClosing<'a> {
        if self.at(Kind::RAngle) {
            if in_jsx_child {
                self.expect_jsx_child(Kind::RAngle);
            } else {
                self.expect(Kind::RAngle);
            }
            JSXClosing::Fragment(self.ast.jsx_closing_fragment(self.end_span(open_angle_span)))
        } else {
            let name = self.parse_astro_jsx_element_name();
            if in_jsx_child {
                self.expect_jsx_child(Kind::RAngle);
            } else {
                self.expect(Kind::RAngle);
            }
            JSXClosing::Element(
                self.ast.alloc_jsx_closing_element(self.end_span(open_angle_span), name),
            )
        }
    }

    /// Astro-specific version of `parse_jsx_expression_container`.
    /// Handles Astro JSX children in expressions and allows empty expressions.
    pub(crate) fn parse_astro_jsx_expression_container(
        &mut self,
        span_start: u32,
        in_jsx_child: bool,
    ) -> Box<'a, JSXExpressionContainer<'a>> {
        // In Astro mode, `{<div/>}` and `{<!-- comment -->}` are treated symmetrically:
        // both route through the JSX-children-in-expression path.
        // `<!-- -->` produces an `AstroComment` JSXChild, just as it does in a JSX
        // children position.  Trailing content after the comment (e.g. `{<!-- --> x}`)
        // is an error, exactly like `{<div/> x}` would be.
        if in_jsx_child && self.at(Kind::LAngle) {
            let expr = self.parse_astro_jsx_children_in_expression(span_start);
            return self.ast.alloc_jsx_expression_container(self.end_span(span_start), expr);
        }

        let expr = if self.at(Kind::RCurly) {
            if in_jsx_child {
                self.expect_jsx_child(Kind::RCurly);
            } else {
                self.expect(Kind::RCurly);
            }
            let span = self.end_span(span_start);

            // In Astro mode, empty expressions are allowed — don't emit error
            if !in_jsx_child {
                // Still allowed in Astro, skip the error
            }

            let expr = self.ast.jsx_empty_expression(Span::new(span.start + 1, span.end - 1));
            JSXExpression::EmptyExpression(expr)
        } else {
            let expr = JSXExpression::from(self.parse_expr());
            if in_jsx_child {
                self.expect_jsx_child(Kind::RCurly);
            } else {
                self.expect(Kind::RCurly);
            }
            expr
        };

        self.ast.alloc_jsx_expression_container(self.end_span(span_start), expr)
    }

    /// Astro-specific version of `parse_jsx_attributes`.
    /// Handles shorthand attributes, special attribute names, and quotes in attribute names.
    pub(crate) fn parse_astro_jsx_attributes(&mut self) -> Vec<'a, JSXAttributeItem<'a>> {
        let mut attributes = self.ast.vec();
        loop {
            let kind = self.cur_kind();
            if matches!(kind, Kind::Eof | Kind::LAngle | Kind::RAngle | Kind::Slash)
                || self.fatal_error.is_some()
            {
                break;
            }
            let attribute = match kind {
                Kind::LCurly => {
                    // Check for empty expression container: {} or {/* comment */}
                    if self.lexer.peek_token().kind() == Kind::RCurly {
                        self.bump_any(); // bump `{`
                        self.bump_any(); // bump `}`
                        continue;
                    }
                    // `{...spread}`
                    if self.lexer.peek_token().kind() == Kind::Dot3 {
                        JSXAttributeItem::SpreadAttribute(self.parse_jsx_spread_attribute())
                    } else if let Some(attr) = self.try_parse_astro_shorthand_attribute() {
                        // `{prop}` shorthand for `prop={prop}`
                        attributes.push(JSXAttributeItem::Attribute(attr));
                        continue;
                    } else {
                        // Any other `{expr}` (e.g. `{{ answer: 1 }}`) is shorthand
                        JSXAttributeItem::Attribute(
                            self.parse_astro_expression_shorthand_attribute(),
                        )
                    }
                }
                // Quotes can appear in Astro attribute names
                Kind::Str | Kind::Undetermined => {
                    // Pop the "unterminated string" error that the lexer added
                    self.lexer.errors.pop();
                    JSXAttributeItem::Attribute(self.parse_astro_attribute())
                }
                _ => {
                    // Use permissive attribute name parsing
                    JSXAttributeItem::Attribute(self.parse_astro_attribute())
                }
            };
            attributes.push(attribute);
        }
        attributes
    }

    /// Astro-specific: Peek ahead past `<` to check if the next token starts a JSX element
    /// or fragment. Used in binary expression context to distinguish `<div>` from `< comparison`.
    ///
    /// Returns `true` if `<` is followed by an identifier, keyword, `>` (fragment), or a
    /// complete `<!-- -->` comment. Does not consume any tokens.
    pub(crate) fn is_astro_jsx_after_lt(&mut self) -> bool {
        let checkpoint = self.checkpoint();
        self.bump_any(); // bump `<`
        let next_kind = self.cur_kind();
        let is_jsx = next_kind == Kind::RAngle
            || next_kind == Kind::Ident
            || next_kind.is_any_keyword()
            || self.at_astro_html_comment();
        self.rewind(checkpoint);
        is_jsx
    }

    /// `true` when the bytes right after the just-bumped `<` open a complete
    /// `<!-- … -->` comment, letting it continue a run of sibling JSX.
    ///
    /// Requiring a real `-->` keeps the JS `< !--x` (negated pre-decrement) a
    /// comparison, and stops the binary parser looping on a `<!--` it can't consume.
    fn at_astro_html_comment(&self) -> bool {
        // `prev_token_end` is the byte right after `<` (no whitespace skipped),
        // so a `< !--` comparison is excluded; only `<!--` matches.
        self.source_text
            .get(self.prev_token_end as usize..)
            .is_some_and(|rest| rest.starts_with("!--") && rest[3..].contains("-->"))
    }

    /// Astro-specific: Parse multiple JSX elements in binary expression context.
    pub(crate) fn parse_astro_multiple_jsx_in_expression(
        &mut self,
        span_start: u32,
        first_element: Expression<'a>,
    ) -> Expression<'a> {
        let mut children = self.ast.vec();

        match first_element {
            Expression::JSXElement(el) => children.push(JSXChild::Element(el)),
            Expression::JSXFragment(frag) => children.push(JSXChild::Fragment(frag)),
            _ => unreachable!(
                "parse_astro_multiple_jsx_in_expression called with non-JSX expression"
            ),
        }

        while self.at(Kind::LAngle) {
            // The JS lexer skips whitespace between siblings; record where the
            // gap starts so it can be preserved (see the helper call below).
            let ws_start = self.prev_token_end;
            let checkpoint = self.checkpoint();
            let child_span = self.start_span();
            self.bump_any(); // bump `<`

            let kind = self.cur_kind();
            if kind == Kind::RAngle {
                self.push_astro_inter_sibling_whitespace(&mut children, ws_start, child_span);
                let fragment = self.parse_astro_jsx_fragment(child_span, false);
                children.push(JSXChild::Fragment(fragment));
            } else if kind == Kind::Ident || kind.is_any_keyword() {
                self.push_astro_inter_sibling_whitespace(&mut children, ws_start, child_span);
                // `<script>` needs the dedicated raw-text path like every other
                // JSX-children site; otherwise its body parses as JSX and a close
                // tag in a template literal (`` `</article>` ``) becomes a stray error.
                if self.cur_src() == "script" {
                    children.push(self.parse_astro_script_in_jsx(child_span, false));
                } else {
                    let element = self.parse_astro_jsx_element(child_span, false);
                    children.push(JSXChild::Element(element));
                }
            } else if kind == Kind::Bang
                && let Some(comment) = self.parse_html_comment_in_jsx(child_span)
            {
                // HTML comment in a sibling run. The comment parser leaves the lexer
                // in JSX-child mode, but this loop is JS-expression context. Resync to
                // JS tokens so the closing `)` ends the list instead of being read as text.
                self.push_astro_inter_sibling_whitespace(&mut children, ws_start, child_span);
                let comment_end = comment.span().end;
                self.lexer.set_position_for_astro(comment_end);
                self.token = self.lexer.next_token();
                // Resync bypasses the parser's bump bookkeeping; point `prev_token_end`
                // at the comment end so the next iteration captures the whitespace
                // between the comment and the following sibling.
                self.prev_token_end = comment_end;
                children.push(comment);
            } else {
                self.rewind(checkpoint);
                break;
            }
        }

        if children.len() == 1 {
            return match children.pop().unwrap() {
                JSXChild::Element(el) => Expression::JSXElement(el),
                JSXChild::Fragment(frag) => Expression::JSXFragment(frag),
                _ => unreachable!(),
            };
        }

        let fragment_span = Span::new(span_start, self.prev_token_end);
        let opening = self.ast.jsx_opening_fragment(Span::empty(span_start));
        let closing = self.ast.jsx_closing_fragment(Span::empty(self.prev_token_end));
        let fragment = self.ast.alloc_jsx_fragment(fragment_span, opening, children, closing);
        Expression::JSXFragment(fragment)
    }

    /// Push the source text in `[start, end)` as a `JSXText` child when it is
    /// non-empty whitespace. Used to preserve whitespace between bare JSX
    /// siblings in an expression, which the JS lexer would otherwise drop.
    fn push_astro_inter_sibling_whitespace(
        &self,
        children: &mut Vec<'a, JSXChild<'a>>,
        start: u32,
        end: u32,
    ) {
        if end <= start {
            return;
        }
        let span = Span::new(start, end);
        let ws = span.source_text(self.source_text);
        if !ws.bytes().all(|b| b.is_ascii_whitespace()) {
            return;
        }
        let text = self.ast.alloc_jsx_text(span, Atom::from(ws), Some(Atom::from(ws)));
        children.push(JSXChild::Text(text));
    }

    // ==================== Astro-specific helpers ====================

    /// Try to parse Astro shorthand attribute `{prop}` -> `prop={prop}`
    fn try_parse_astro_shorthand_attribute(&mut self) -> Option<Box<'a, JSXAttribute<'a>>> {
        let checkpoint = self.checkpoint();
        let attr_span = self.start_span();
        self.bump_any(); // bump `{`

        if self.at(Kind::Ident) || self.cur_kind().is_any_keyword() {
            let ident_span = self.start_span();
            let name = self.cur_src();
            self.bump_any();
            let ident_span = self.end_span(ident_span);

            if self.at(Kind::RCurly) {
                self.bump_any(); // bump `}`
                let attr_span = self.end_span(attr_span);
                let name = Atom::from(name);
                let identifier = self.ast.jsx_identifier(ident_span, name);
                let attr_name = JSXAttributeName::Identifier(self.alloc(identifier));

                let ident_ref = self.ast.identifier_reference(ident_span, name);
                let expr = Expression::Identifier(self.alloc(ident_ref));
                let expr_container =
                    self.ast.alloc_jsx_expression_container(attr_span, JSXExpression::from(expr));
                let value = JSXAttributeValue::ExpressionContainer(expr_container);

                return Some(self.ast.alloc_jsx_attribute(attr_span, attr_name, Some(value)));
            }
        }

        self.rewind(checkpoint);
        None
    }

    /// Parse an Astro shorthand attribute whose expression is not a bare
    /// identifier, e.g. `{{ answer: sum(2, 4) }}`. The attribute name is the
    /// source text of the inner expression (matching the Go compiler).
    fn parse_astro_expression_shorthand_attribute(&mut self) -> Box<'a, JSXAttribute<'a>> {
        let span = self.start_span();
        self.bump_any(); // bump `{`

        let expr_start = self.cur_token().span().start;
        let expr = self.parse_expr();
        // `parse_expr` can fail without advancing (e.g. the next token is `)`),
        // leaving `prev_token_end < expr_start`. Clamp so the name span is never
        // inverted; `source_text()` panics on an inverted span.
        let name_span = Span::new(expr_start, self.prev_token_end.max(expr_start));
        self.expect(Kind::RCurly);

        let name = Atom::from(name_span.source_text(self.source_text).trim());
        let identifier = self.ast.jsx_identifier(name_span, name);
        let attr_name = JSXAttributeName::Identifier(self.alloc(identifier));

        let expr_container =
            self.ast.alloc_jsx_expression_container(name_span, JSXExpression::from(expr));
        let value = JSXAttributeValue::ExpressionContainer(expr_container);

        self.ast.alloc_jsx_attribute(self.end_span(span), attr_name, Some(value))
    }

    /// Parse an Astro attribute which can have special characters in the name.
    fn parse_astro_attribute(&mut self) -> Box<'a, JSXAttribute<'a>> {
        let span = self.start_span();

        let token_start = self.cur_token().span().start;
        self.lexer.set_position_for_astro(token_start);

        self.read_astro_attribute_name();
        let name_span = self.cur_token().span();
        let name = name_span.source_text(self.source_text);
        self.bump_any();

        let identifier = self.ast.jsx_identifier(name_span, name);
        let attr_name = JSXAttributeName::Identifier(self.alloc(identifier));

        let value = if self.at(Kind::Eq) { Some(self.parse_astro_attribute_value()) } else { None };

        self.ast.alloc_jsx_attribute(self.end_span(span), attr_name, value)
    }

    /// Parse an Astro attribute value. Called while positioned at `=`.
    ///
    /// The lexing mode is chosen from the raw bytes after `=` *before* the JS
    /// lexer runs, so unquoted HTML values reach the Astro reader instead of
    /// being rejected as malformed JS tokens (e.g. `color=#18b218`). Quoted
    /// strings, `{expr}` and templates still lex as JS/JSX.
    fn parse_astro_attribute_value(&mut self) -> JSXAttributeValue<'a> {
        let bytes = self.source_text.as_bytes();
        let mut value_start = self.cur_token().end() as usize;
        while matches!(bytes.get(value_start), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            value_start += 1;
        }
        // A structural terminator (`>`, `}`, EOF) means the value is missing
        // (`attr=` before `>`); fall through to the JS path to report it there.
        // `/` is a valid first value char (HTML5): `href=/about` is value `/about`,
        // and even `attr=/>` is value `/`, not a self-close.
        let is_unquoted = bytes
            .get(value_start)
            .is_some_and(|b| !matches!(b, b'"' | b'\'' | b'{' | b'`' | b'<' | b'>' | b'}'));

        if is_unquoted {
            self.prev_token_end = self.cur_token().end(); // consume `=`
            self.lexer.set_position_for_astro(value_start as u32);
            self.read_astro_unquoted_attribute_value();
            let value_span = self.cur_token().span();
            let value = Atom::from(value_span.source_text(self.source_text));
            self.bump_any();
            let str_lit = self.ast.string_literal(value_span, value, None);
            return JSXAttributeValue::StringLiteral(self.alloc(str_lit));
        }

        self.expect_jsx_attribute_value(Kind::Eq);
        match self.cur_kind() {
            Kind::NoSubstitutionTemplate | Kind::TemplateHead => {
                let span = self.start_span();
                let template_lit = self.parse_template_literal(false);
                let expr = Expression::TemplateLiteral(self.alloc(template_lit));
                let expr_container = self
                    .ast
                    .alloc_jsx_expression_container(self.end_span(span), JSXExpression::from(expr));
                JSXAttributeValue::ExpressionContainer(expr_container)
            }
            Kind::LCurly => {
                let span_start = self.start_span();
                self.bump_any(); // bump `{`
                let expr = self.parse_astro_jsx_expression_container(
                    span_start, /* in_jsx_child */ false,
                );
                JSXAttributeValue::ExpressionContainer(expr)
            }
            Kind::Str | Kind::LAngle => self.parse_jsx_attribute_value(),
            // Reached only via the JS path for a token that isn't a recognized
            // value start (e.g. a comment before the value). Bail on a structural
            // terminator, otherwise re-read under HTML rules.
            _ => {
                let token_start = self.cur_token().span().start as usize;
                let starts_with_terminator = self
                    .source_text
                    .as_bytes()
                    .get(token_start)
                    .is_none_or(|b| matches!(b, b'/' | b'>' | b'<' | b'}'));
                if starts_with_terminator {
                    return self.unexpected();
                }
                self.lexer.set_position_for_astro(token_start as u32);
                self.read_astro_unquoted_attribute_value();
                let value_span = self.cur_token().span();
                let value = Atom::from(value_span.source_text(self.source_text));
                self.bump_any();
                let str_lit = self.ast.string_literal(value_span, value, None);
                JSXAttributeValue::StringLiteral(self.alloc(str_lit))
            }
        }
    }

    /// Check if the element name is an HTML void element.
    pub(crate) fn is_astro_void_element(name: &JSXElementName<'a>) -> bool {
        match name {
            JSXElementName::Identifier(ident) => {
                matches!(
                    ident.name.as_str(),
                    "area"
                        | "base"
                        | "br"
                        | "col"
                        | "embed"
                        | "hr"
                        | "img"
                        | "input"
                        | "link"
                        | "meta"
                        | "param"
                        | "source"
                        | "track"
                        | "wbr"
                )
            }
            _ => false,
        }
    }

    /// Check if the element name is a raw text element (style only in Astro).
    fn is_astro_raw_text_element(name: &JSXElementName<'a>) -> bool {
        match name {
            JSXElementName::Identifier(ident) => matches!(ident.name.as_str(), "style"),
            _ => false,
        }
    }

    /// Determine whether a `<script>` tag's content should be treated as raw text
    /// (i.e. NOT parsed as JavaScript/TypeScript).
    ///
    /// Rules:
    /// - If there is **no `type` attribute** → parse as TS/JS (return `false`)
    /// - If `type` is a JS/module MIME type → parse as TS/JS (return `false`)
    ///   - Recognised JS MIME types: `text/javascript`, `text/ecmascript`,
    ///     `application/javascript`, `application/ecmascript`,
    ///     `application/x-javascript`, `module`
    /// - If `type` is anything else (e.g. `text/css`, `application/json`) →
    ///   treat as raw text (return `true`)
    ///
    /// Other attributes (e.g. `defer`, `src`, `is:inline`, `asdf`) do not affect
    /// parsing — only the presence and value of `type` matters.
    pub(crate) fn is_raw_text_script(attributes: &[JSXAttributeItem<'a>]) -> bool {
        for attr in attributes {
            let JSXAttributeItem::Attribute(attr) = attr else { continue };
            let attr_name = match &attr.name {
                JSXAttributeName::Identifier(ident) => ident.name.as_str(),
                JSXAttributeName::NamespacedName(_) => continue,
            };
            if attr_name != "type" {
                continue;
            }
            // Found a `type` attribute — check its value.
            let type_value = match &attr.value {
                None => {
                    // `<script type>` with no value — treat as raw text
                    return true;
                }
                Some(JSXAttributeValue::StringLiteral(s)) => s.value.as_str(),
                Some(JSXAttributeValue::ExpressionContainer(container)) => {
                    // Dynamic type attribute — we can't determine it statically,
                    // so conservatively treat as raw text.
                    let _ = container;
                    return true;
                }
                Some(_) => return true,
            };
            // Check against known JS MIME types.
            return !matches!(
                type_value,
                "text/javascript"
                    | "text/ecmascript"
                    | "application/javascript"
                    | "application/ecmascript"
                    | "application/x-javascript"
                    | "module"
            );
        }
        // No `type` attribute found — parse as TS/JS.
        false
    }

    /// Check if the element is a foreign content element where `{` is literal text.
    fn is_foreign_content_element(name: &JSXElementName<'a>) -> bool {
        match name {
            JSXElementName::Identifier(ident) => ident.name.as_str() == "math",
            _ => false,
        }
    }

    /// Check if attributes contain `is:raw` directive.
    fn has_is_raw_attribute(attributes: &[JSXAttributeItem<'a>]) -> bool {
        attributes.iter().any(|attr| {
            if let JSXAttributeItem::Attribute(attr) = attr {
                match &attr.name {
                    JSXAttributeName::Identifier(ident) => ident.name.as_str() == "is:raw",
                    JSXAttributeName::NamespacedName(ns) => {
                        ns.namespace.name.as_str() == "is" && ns.name.name.as_str() == "raw"
                    }
                }
            } else {
                false
            }
        })
    }

    /// Astro-specific element name equality check.
    fn astro_jsx_element_name_eq(lhs: &JSXElementName<'a>, rhs: &JSXElementName<'a>) -> bool {
        match (lhs, rhs) {
            (JSXElementName::Identifier(lhs), JSXElementName::Identifier(rhs)) => {
                lhs.name == rhs.name
            }
            (
                JSXElementName::IdentifierReference(lhs),
                JSXElementName::IdentifierReference(rhs),
            ) => lhs.name == rhs.name,
            (JSXElementName::NamespacedName(lhs), JSXElementName::NamespacedName(rhs)) => {
                lhs.namespace.name == rhs.namespace.name && lhs.name.name == rhs.name.name
            }
            (JSXElementName::MemberExpression(lhs), JSXElementName::MemberExpression(rhs)) => {
                Self::astro_jsx_member_expression_eq(lhs, rhs)
            }
            (JSXElementName::ThisExpression(_), JSXElementName::ThisExpression(_)) => true,
            _ => false,
        }
    }

    fn astro_jsx_member_expression_eq(
        lhs: &JSXMemberExpression<'a>,
        rhs: &JSXMemberExpression<'a>,
    ) -> bool {
        if lhs.property.name != rhs.property.name {
            return false;
        }
        match (&lhs.object, &rhs.object) {
            (
                JSXMemberExpressionObject::IdentifierReference(lhs),
                JSXMemberExpressionObject::IdentifierReference(rhs),
            ) => lhs.name == rhs.name,
            (
                JSXMemberExpressionObject::MemberExpression(lhs),
                JSXMemberExpressionObject::MemberExpression(rhs),
            ) => Self::astro_jsx_member_expression_eq(lhs, rhs),
            (
                JSXMemberExpressionObject::ThisExpression(_),
                JSXMemberExpressionObject::ThisExpression(_),
            ) => true,
            _ => false,
        }
    }

    /// Parse a `<script>` element encountered inside JSX children (Astro-specific).
    ///
    /// See [`Self::is_raw_text_script`] for the rules on when the content is
    /// parsed vs. treated as raw text.
    #[expect(clippy::cast_possible_truncation)]
    pub(crate) fn parse_astro_script_in_jsx(
        &mut self,
        span: u32,
        in_jsx_child: bool,
    ) -> JSXChild<'a> {
        self.bump_any(); // skip `script`

        let attributes = self.parse_astro_jsx_attributes();
        let has_attributes = Self::is_raw_text_script(&attributes);

        let is_self_closing = if self.at(Kind::Slash) {
            self.bump_any();
            true
        } else {
            false
        };

        if self.at(Kind::RAngle) {
            self.bump_any();
        }

        if is_self_closing {
            let end = self.prev_token_end;
            let script_span = oxc_span::Span::new(span, end);

            if has_attributes {
                let name =
                    self.ast.jsx_identifier(oxc_span::Span::new(span + 1, span + 7), "script");
                let elem_name = JSXElementName::Identifier(self.alloc(name));
                let opening = self.ast.alloc_jsx_opening_element(
                    script_span,
                    elem_name,
                    Option::<Box<'a, oxc_ast::ast::TSTypeParameterInstantiation<'a>>>::None,
                    attributes,
                );
                return JSXChild::Element(self.ast.alloc_jsx_element(
                    script_span,
                    opening,
                    self.ast.vec(),
                    Option::<Box<'a, JSXClosingElement<'a>>>::None,
                ));
            }

            let program = self.ast.program(
                oxc_span::Span::empty(end),
                self.source_type,
                "",
                self.ast.vec(),
                None,
                self.ast.vec(),
                self.ast.vec(),
            );
            let astro_script =
                JSXChild::AstroScript(self.ast.alloc_astro_script(script_span, program));

            let name = self.ast.jsx_identifier(oxc_span::Span::new(span + 1, span + 7), "script");
            let elem_name = JSXElementName::Identifier(self.alloc(name));
            let opening = self.ast.alloc_jsx_opening_element(
                script_span,
                elem_name,
                Option::<Box<'a, oxc_ast::ast::TSTypeParameterInstantiation<'a>>>::None,
                attributes,
            );
            return JSXChild::Element(self.ast.alloc_jsx_element(
                script_span,
                opening,
                self.ast.vec1(astro_script),
                Option::<Box<'a, JSXClosingElement<'a>>>::None,
            ));
        }

        // Use `prev_token_end` (right after `>`) instead of `cur_token().span().start` so
        // that leading trivia (e.g. `// eslint-disable-next-line` comments) before the first
        // real token is included in the content span and picked up by the parser.
        let content_start = self.prev_token_end as usize;
        let closing_tag = "</script";

        if let Some(rest) = self.source_text.get(content_start..)
            && let Some(end_offset) = rest.find(closing_tag)
        {
            let content_end = content_start + end_offset;

            self.lexer.set_position_for_astro(content_end as u32);
            self.token = self.lexer.next_jsx_child();

            self.bump_any(); // skip `<`
            self.bump_any(); // skip `/`
            self.bump_any(); // Skip `script`
            let end = if self.at(Kind::RAngle) {
                self.bump_any();
                self.prev_token_end
            } else {
                self.cur_token().span().end
            };

            // The closing tag was consumed with the regular lexer, which skips
            // trivia. Re-enter JSX-child mode so whitespace after the raw-text
            // element is preserved as a JSXText child of its parent.
            self.lexer.set_position_for_astro(end);
            self.token =
                if in_jsx_child { self.lexer.next_jsx_child() } else { self.lexer.next_token() };

            let full_span = oxc_span::Span::new(span, end);

            if has_attributes {
                let opening_name =
                    self.ast.jsx_identifier(oxc_span::Span::new(span + 1, span + 7), "script");
                let opening_elem_name = JSXElementName::Identifier(self.alloc(opening_name));
                let opening = self.ast.alloc_jsx_opening_element(
                    oxc_span::Span::new(span, content_start as u32),
                    opening_elem_name,
                    Option::<Box<'a, oxc_ast::ast::TSTypeParameterInstantiation<'a>>>::None,
                    attributes,
                );

                let closing_name = self
                    .ast
                    .jsx_identifier(oxc_span::Span::new(content_end as u32 + 2, end - 1), "script");
                let closing_elem_name = JSXElementName::Identifier(self.alloc(closing_name));
                let closing = self.ast.alloc_jsx_closing_element(
                    oxc_span::Span::new(content_end as u32, end),
                    closing_elem_name,
                );

                let raw_text = &self.source_text[content_start..content_end];
                let text_span = oxc_span::Span::new(content_start as u32, content_end as u32);
                let text = self.ast.alloc_jsx_text(
                    text_span,
                    oxc_span::Atom::from(raw_text),
                    Some(oxc_span::Atom::from(raw_text)),
                );

                return JSXChild::Element(self.ast.alloc_jsx_element(
                    full_span,
                    opening,
                    self.ast.vec1(JSXChild::Text(text)),
                    Some(closing),
                ));
            }

            let script_content_span = oxc_span::Span::new(content_start as u32, content_end as u32);
            let program = self.ast.program(
                script_content_span,
                self.source_type,
                "",
                self.ast.vec(),
                None,
                self.ast.vec(),
                self.ast.vec(),
            );
            let astro_script =
                JSXChild::AstroScript(self.ast.alloc_astro_script(full_span, program));

            let opening_name =
                self.ast.jsx_identifier(oxc_span::Span::new(span + 1, span + 7), "script");
            let opening_elem_name = JSXElementName::Identifier(self.alloc(opening_name));
            let opening = self.ast.alloc_jsx_opening_element(
                oxc_span::Span::new(span, content_start as u32),
                opening_elem_name,
                Option::<Box<'a, oxc_ast::ast::TSTypeParameterInstantiation<'a>>>::None,
                attributes,
            );

            let closing_name = self
                .ast
                .jsx_identifier(oxc_span::Span::new(content_end as u32 + 2, end - 1), "script");
            let closing_elem_name = JSXElementName::Identifier(self.alloc(closing_name));
            let closing = self.ast.alloc_jsx_closing_element(
                oxc_span::Span::new(content_end as u32, end),
                closing_elem_name,
            );

            return JSXChild::Element(self.ast.alloc_jsx_element(
                full_span,
                opening,
                self.ast.vec1(astro_script),
                Some(closing),
            ));
        }

        // No closing tag found - error recovery
        let end = self.prev_token_end;
        let script_span = oxc_span::Span::new(span, end);
        let program = self.ast.program(
            oxc_span::Span::empty(end),
            self.source_type,
            "",
            self.ast.vec(),
            None,
            self.ast.vec(),
            self.ast.vec(),
        );
        let astro_script = JSXChild::AstroScript(self.ast.alloc_astro_script(script_span, program));

        let name = self.ast.jsx_identifier(oxc_span::Span::new(span + 1, span + 7), "script");
        let elem_name = JSXElementName::Identifier(self.alloc(name));
        let opening = self.ast.alloc_jsx_opening_element(
            script_span,
            elem_name,
            Option::<Box<'a, oxc_ast::ast::TSTypeParameterInstantiation<'a>>>::None,
            self.ast.vec(),
        );
        JSXChild::Element(self.ast.alloc_jsx_element(
            script_span,
            opening,
            self.ast.vec1(astro_script),
            Option::<Box<'a, JSXClosingElement<'a>>>::None,
        ))
    }

    /// Skip the raw text content of a raw text element in Astro.
    pub(crate) fn skip_astro_raw_text_element_content(
        &mut self,
        name: &JSXElementName<'a>,
        in_jsx_child: bool,
    ) -> Vec<'a, JSXChild<'a>> {
        let tag_name = match name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            JSXElementName::IdentifierReference(ident_ref) => ident_ref.name.as_str(),
            JSXElementName::MemberExpression(_) => name.span().source_text(self.source_text),
            JSXElementName::NamespacedName(_) | JSXElementName::ThisExpression(_) => {
                name.span().source_text(self.source_text)
            }
        };

        let closing_tag = format!("</{tag_name}");

        let start_pos = self.prev_token_end as usize;
        if let Some(rest) = self.source_text.get(start_pos..)
            && let Some(end_pos) = rest.find(&closing_tag)
        {
            #[expect(clippy::cast_possible_truncation)]
            let content_end = (start_pos + end_pos) as u32;
            let content_start = self.prev_token_end;

            self.lexer.set_position_for_astro(content_end);
            if in_jsx_child {
                self.token = self.lexer.next_jsx_child();
            } else {
                self.token = self.lexer.next_token();
            }

            if content_end > content_start {
                let span = Span::new(content_start, content_end);
                let raw_text = span.source_text(self.source_text);
                let jsx_text = self.ast.alloc_jsx_text(span, raw_text, Some(Atom::from(raw_text)));
                return self.ast.vec1(JSXChild::Text(jsx_text));
            }
        }

        self.ast.vec()
    }

    /// Parse HTML comment in JSX (Astro-specific).
    #[expect(clippy::cast_possible_truncation)]
    pub(crate) fn parse_html_comment_in_jsx(&mut self, span: u32) -> Option<JSXChild<'a>> {
        let start_pos = self.prev_token_end as usize;

        // Search for the closing `-->` *after* the opening `!--` (offset >= 3).
        // Otherwise an overlapping marker like `<!-->` would match `-->` at
        // offset 1 and slice `rest[3..1]`, panicking the parser on malformed input.
        if let Some(rest) = self.source_text.get(start_pos..)
            && rest.starts_with("!--")
            && let Some(rel) = rest[3..].find("-->")
        {
            let end_offset = rel + 3;
            let comment_end = (start_pos + end_offset + 3) as u32;
            let comment_start = span;

            let content = &rest[3..end_offset];
            let value = oxc_span::Atom::from(content);

            let comment_span = oxc_span::Span::new(comment_start, comment_end);
            let comment = self.ast.alloc_astro_comment(comment_span, value);

            self.lexer.set_position_for_astro(comment_end);
            self.token = self.lexer.next_jsx_child();

            return Some(JSXChild::AstroComment(comment));
        }

        None
    }

    /// Parse JSX children in an expression container (Astro-specific).
    fn parse_astro_jsx_children_in_expression(&mut self, span_start: u32) -> JSXExpression<'a> {
        let fragment_span_start = span_start + 1;
        let mut children = self.ast.vec();

        loop {
            if self.at(Kind::Eof) {
                break;
            }

            match self.cur_kind() {
                Kind::LAngle => {
                    let child_span = self.start_span();
                    self.bump_any();

                    let kind = self.cur_kind();

                    if kind == Kind::RAngle {
                        let fragment = self.parse_astro_jsx_fragment(child_span, true);
                        children.push(JSXChild::Fragment(fragment));
                        if self.at(Kind::Eof) {
                            self.lexer.errors.pop();
                            self.token = self.lexer.next_token();
                        }
                    } else if kind == Kind::Ident || kind.is_any_keyword() {
                        if self.cur_src() == "script" {
                            children.push(self.parse_astro_script_in_jsx(child_span, true));
                        } else {
                            let element = self.parse_astro_jsx_element(child_span, true);
                            children.push(JSXChild::Element(element));
                        }
                        if self.at(Kind::Eof) {
                            self.lexer.errors.pop();
                            self.token = self.lexer.next_token();
                        }
                    } else if kind == Kind::Slash {
                        let _: () = self.unexpected();
                        break;
                    } else if kind == Kind::Bang {
                        // `<!--` HTML comment — produce an AstroComment child.
                        // `parse_html_comment_in_jsx` repositions the lexer and
                        // calls `next_jsx_child()` internally, so after returning
                        // the loop continues in JSX-child mode.
                        if let Some(comment) = self.parse_html_comment_in_jsx(child_span) {
                            children.push(comment);
                        } else {
                            // Malformed `<!…>` — skip and continue.
                            self.token = self.lexer.next_jsx_child();
                        }
                    } else {
                        let _: () = self.unexpected();
                        break;
                    }
                }
                Kind::JSXText => {
                    let text = self.parse_jsx_text();
                    children.push(JSXChild::Text(text));
                }
                Kind::LCurly => {
                    let nested_span = self.start_span();
                    self.bump_any();

                    if self.eat(Kind::Dot3) {
                        let spread = self.parse_jsx_spread_child(nested_span);
                        children.push(JSXChild::Spread(spread));
                    } else {
                        let expr = JSXExpression::from(self.parse_expr());
                        self.expect_jsx_child(Kind::RCurly);
                        let container = self
                            .ast
                            .alloc_jsx_expression_container(self.end_span(nested_span), expr);
                        children.push(JSXChild::ExpressionContainer(container));
                    }
                    self.token = self.lexer.next_jsx_child();
                }
                Kind::RCurly => {
                    break;
                }
                _ => {
                    self.token = self.lexer.next_jsx_child();
                    if self.at(Kind::Eof) {
                        break;
                    }
                }
            }
        }

        self.expect_jsx_child(Kind::RCurly);

        // Strip trailing whitespace-only text node produced by the JSX child
        // lexer scanning whitespace between the last real child and `}`.
        if let Some(JSXChild::Text(t)) = children.last() {
            if t.value.as_str().chars().all(|ch| ch.is_ascii_whitespace()) {
                children.pop();
            }
        }

        if children.len() == 1 {
            match children.pop().unwrap() {
                JSXChild::Element(el) => return JSXExpression::JSXElement(el),
                JSXChild::Fragment(frag) => return JSXExpression::JSXFragment(frag),
                other => children.push(other),
            }
        }

        let fragment_span = Span::new(fragment_span_start, self.prev_token_end);
        let opening = self.ast.jsx_opening_fragment(Span::empty(fragment_span_start));
        let closing = self.ast.jsx_closing_fragment(Span::empty(self.prev_token_end));
        let fragment = self.ast.alloc_jsx_fragment(fragment_span, opening, children, closing);
        JSXExpression::JSXFragment(fragment)
    }
}
