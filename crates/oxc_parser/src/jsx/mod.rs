//! [JSX](https://facebook.github.io/jsx)

use oxc_allocator::{Box, Dummy, Vec};
use oxc_ast::ast::*;
use oxc_span::{Atom, GetSpan, Span};

use crate::{ParserImpl, diagnostics, lexer::Kind};

impl<'a> ParserImpl<'a> {
    pub(crate) fn parse_jsx_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `<`
        let kind = self.cur_kind();
        if kind == Kind::RAngle {
            Expression::JSXFragment(self.parse_jsx_fragment(span, false))
        } else if kind.is_identifier_or_keyword() {
            Expression::JSXElement(self.parse_jsx_element(span, false))
        } else {
            self.unexpected()
        }
    }

    /// `JSXFragment` :
    ///   < > `JSXChildren_opt` < / >
    pub(crate) fn parse_jsx_fragment(
        &mut self,
        span: u32,
        in_jsx_child: bool,
    ) -> Box<'a, JSXFragment<'a>> {
        self.expect_jsx_child(Kind::RAngle);
        let opening_fragment = self.ast.jsx_opening_fragment(self.end_span(span));
        let children = self.parse_jsx_children();
        let closing_fragment = self.parse_jsx_closing_fragment(in_jsx_child);
        self.ast.alloc_jsx_fragment(
            self.end_span(span),
            opening_fragment,
            children,
            closing_fragment,
        )
    }

    /// </>
    fn parse_jsx_closing_fragment(&mut self, in_jsx_child: bool) -> JSXClosingFragment {
        let span = self.start_span();
        self.expect(Kind::LAngle);
        self.expect(Kind::Slash);
        if in_jsx_child {
            self.expect_jsx_child(Kind::RAngle);
        } else {
            self.expect(Kind::RAngle);
        }
        self.ast.jsx_closing_fragment(self.end_span(span))
    }

    /// `JSXElement` :
    ///   `JSXSelfClosingElement`
    ///   `JSXOpeningElement` `JSXChildren_opt` `JSXClosingElement`
    /// `in_jsx_child`:
    ///     used for telling `JSXClosingElement` to parse the next jsx child or not
    ///     true when inside jsx element, false when at top level expression
    pub(crate) fn parse_jsx_element(
        &mut self,
        span: u32,
        in_jsx_child: bool,
    ) -> Box<'a, JSXElement<'a>> {
        let (opening_element, self_closing, is_raw_text_element) =
            self.parse_jsx_opening_element(span, in_jsx_child);
        let (children, closing_element) = if self_closing {
            (self.ast.vec(), None)
        } else {
            let children = if is_raw_text_element {
                self.skip_raw_text_element_content(&opening_element.name, in_jsx_child)
            } else {
                self.parse_jsx_children()
            };
            let closing_element = self.parse_jsx_closing_element(in_jsx_child);
            if !Self::jsx_element_name_eq(&opening_element.name, &closing_element.name) {
                self.error(diagnostics::jsx_element_no_match(
                    opening_element.name.span(),
                    closing_element.name.span(),
                    opening_element.name.span().source_text(self.source_text),
                ));
            }
            (children, Some(closing_element))
        };
        self.ast.alloc_jsx_element(self.end_span(span), opening_element, children, closing_element)
    }

    /// Check if the element name is a raw text element (script, style, etc.)
    /// These elements contain raw text content, not JSX children.
    fn is_raw_text_element(name: &JSXElementName<'a>) -> bool {
        match name {
            JSXElementName::Identifier(ident) => {
                matches!(ident.name.as_str(), "script" | "style")
            }
            _ => false,
        }
    }

    /// Check if the element name is an HTML void element.
    /// Void elements cannot have content and don't require a closing tag.
    /// <https://html.spec.whatwg.org/multipage/syntax.html#void-elements>
    fn is_html_void_element(name: &JSXElementName<'a>) -> bool {
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

    /// Skip the raw text content of a script or style element.
    /// Returns an empty children vector since we don't parse the content as JSX.
    ///
    /// Note: This is called AFTER the opening element's `>` has been consumed,
    /// but BEFORE the lexer has advanced to parse JSX children.
    /// The `prev_token_end` points to right after the `>`.
    fn skip_raw_text_element_content(
        &mut self,
        name: &JSXElementName<'a>,
        in_jsx_child: bool,
    ) -> Vec<'a, JSXChild<'a>> {
        let tag_name = match name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            _ => return self.ast.vec(),
        };

        // Build the closing tag pattern: </script> or </style>
        let closing_tag = format!("</{tag_name}");

        // Start searching from after the `>` we just consumed
        let start_pos = self.prev_token_end as usize;
        if let Some(rest) = self.source_text.get(start_pos..)
            && let Some(end_pos) = rest.find(&closing_tag)
        {
            // Move the lexer position to the closing tag
            #[expect(clippy::cast_possible_truncation)]
            let new_pos = (start_pos + end_pos) as u32;
            self.lexer.set_position_for_astro(new_pos);
            // Read the `<` token to prepare for parsing the closing element
            // Use next_jsx_child if in_jsx_child to maintain consistent lexer state
            if in_jsx_child {
                self.token = self.lexer.next_jsx_child();
            } else {
                self.token = self.lexer.next_token();
            }
        }

        self.ast.vec()
    }

    /// `JSXOpeningElement` :
    /// < `JSXElementName` `JSXAttributes_opt` >
    ///
    /// Returns (opening_element, self_closing, is_raw_text_element)
    fn parse_jsx_opening_element(
        &mut self,
        span: u32,
        in_jsx_child: bool,
    ) -> (
        Box<'a, JSXOpeningElement<'a>>,
        bool, // `true` if self-closing
        bool, // `true` if raw text element (script/style)
    ) {
        let name = self.parse_jsx_element_name();
        // <Component<TsType> for tsx
        let type_arguments = if self.is_ts { self.try_parse_type_arguments() } else { None };
        let attributes = self.parse_jsx_attributes();
        let explicit_self_closing = self.eat(Kind::Slash);

        // In Astro, HTML void elements are implicitly self-closing even without `/`
        let self_closing = explicit_self_closing
            || (self.source_type.is_astro() && Self::is_html_void_element(&name));

        // Check if this is a raw text element BEFORE advancing the lexer
        let is_raw_text = Self::is_raw_text_element(&name);

        // For raw text elements, we must NOT call next_token() or next_jsx_child()
        // because the content should not be parsed as JSX/JS. We just check for `>`
        // without advancing, then record prev_token_end manually.
        // The caller (skip_raw_text_element_content) will reposition the lexer.
        if is_raw_text && !explicit_self_closing {
            self.expect_without_advance(Kind::RAngle);
            // Manually update prev_token_end to point after the `>`
            self.prev_token_end = self.cur_token().span().end;
        } else if !explicit_self_closing || in_jsx_child {
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
        (elem, self_closing, is_raw_text)
    }

    fn parse_jsx_closing_element(&mut self, in_jsx_child: bool) -> Box<'a, JSXClosingElement<'a>> {
        let span = self.start_span();
        self.expect(Kind::LAngle);
        self.expect(Kind::Slash);
        let name = self.parse_jsx_element_name();
        if in_jsx_child {
            self.expect_jsx_child(Kind::RAngle);
        } else {
            self.expect(Kind::RAngle);
        }
        self.ast.alloc_jsx_closing_element(self.end_span(span), name)
    }

    /// `JSXElementName` :
    ///   `JSXIdentifier`
    ///   `JSXNamespacedName`
    ///   `JSXMemberExpression`
    fn parse_jsx_element_name(&mut self) -> JSXElementName<'a> {
        let span = self.start_span();
        let identifier = self.parse_jsx_identifier();

        // <namespace:property />
        // In Astro, `:` followed by whitespace is NOT a namespace separator.
        // `<div:foo>` is a namespaced name, but `<div :foo>` has `:foo` as an attribute.
        // We detect this by checking if there's whitespace between the identifier and `:`.
        let colon_directly_follows = self.at(Kind::Colon)
            && self.prev_token_end == self.cur_token().span().start;
        if colon_directly_follows && self.eat(Kind::Colon) {
            let property = self.parse_jsx_identifier();
            return self.ast.jsx_element_name_namespaced_name(
                self.end_span(span),
                identifier,
                property,
            );
        }

        // <member.foo.bar />
        if self.at(Kind::Dot) {
            return JSXElementName::MemberExpression(
                self.parse_jsx_member_expression(span, &identifier),
            );
        }

        if self.fatal_error.is_some() {
            return JSXElementName::dummy(self.ast.allocator);
        }

        // Determine if this JSX element name is a reference (component) or an intrinsic element.
        // References (components) are:
        // - ASCII names that start with uppercase letter, `_` or `$`: `<Foo>`, `<_foo>`, `<$foo>`
        // - All non-ASCII names (e.g., Unicode identifiers like `<테스트>`)
        // - Names without hyphens (hyphenated names like `<my-element>` are custom elements)
        // https://babeljs.io/repl#?code_lz=DwMQ9mAED0B8DcAoYAzCMHIPpqnJwAJLhkkA&presets=react
        //
        // The identifier has already been validated by the parser, so for ASCII characters
        // we know it can only be `a-z`, `A-Z`, `_` or `$`.
        // Use a fast path for ASCII to avoid expensive Unicode operations in the common case.
        let name = identifier.name.as_str();
        let is_reference = match name.as_bytes()[0] {
            b if b.is_ascii() => !b.is_ascii_lowercase(), // Matches A-Z, _, $
            _ => true, // Non-ASCII characters are always treated as references
        } && !name.contains('-'); // Exclude hyphenated custom elements

        if is_reference {
            let identifier = self.ast.alloc_identifier_reference(identifier.span, identifier.name);
            JSXElementName::IdentifierReference(identifier)
        } else if name == "this" {
            JSXElementName::ThisExpression(self.ast.alloc_this_expression(identifier.span))
        } else {
            JSXElementName::Identifier(self.alloc(identifier))
        }
    }

    /// `JSXMemberExpression` :
    /// `JSXIdentifier` . `JSXIdentifier`
    /// `JSXMemberExpression` . `JSXIdentifier`
    fn parse_jsx_member_expression(
        &mut self,
        span: u32,
        object: &JSXIdentifier<'a>,
    ) -> Box<'a, JSXMemberExpression<'a>> {
        let mut object = if object.name == "this" {
            self.ast.jsx_member_expression_object_this_expression(object.span)
        } else {
            self.ast.jsx_member_expression_object_identifier_reference(object.span, object.name)
        };

        let mut span = Span::new(span, 0);
        let mut property = None;

        while self.eat(Kind::Dot) && self.fatal_error.is_none() {
            // <foo.bar.baz>
            if let Some(prop) = property {
                object =
                    self.ast.jsx_member_expression_object_member_expression(span, object, prop);
            }

            // <foo.bar>
            let ident = self.parse_jsx_identifier();
            // `<foo.bar- />` is a syntax error.
            if ident.name.contains('-') {
                let error = diagnostics::identifier_expected_jsx_no_hyphen(ident.span);
                return self.fatal_error(error);
            }
            property = Some(ident);
            span = self.end_span(span.start);
        }

        if let Some(property) = property {
            return self.ast.alloc_jsx_member_expression(
                self.end_span(span.start),
                object,
                property,
            );
        }

        self.unexpected()
    }

    /// `JSXChildren` :
    ///   `JSXChild` `JSXChildren_opt`
    fn parse_jsx_children(&mut self) -> Vec<'a, JSXChild<'a>> {
        let mut children = self.ast.vec();
        while self.fatal_error.is_none() {
            if let Some(child) = self.parse_jsx_child() {
                children.push(child);
            } else {
                break;
            }
        }
        children
    }

    /// `JSXChild` :
    ///   `JSXText`
    ///   `JSXElement`
    ///   `JSXFragment`
    ///   { `JSXChildExpression_opt` }
    fn parse_jsx_child(&mut self) -> Option<JSXChild<'a>> {
        match self.cur_kind() {
            Kind::LAngle => {
                let span = self.start_span();
                let checkpoint = self.checkpoint();
                self.bump_any(); // bump `<`
                let kind = self.cur_kind();
                // <> open fragment
                if kind == Kind::RAngle {
                    return Some(JSXChild::Fragment(self.parse_jsx_fragment(span, true)));
                }
                // <ident open element
                if kind == Kind::Ident || kind.is_any_keyword() {
                    return Some(JSXChild::Element(self.parse_jsx_element(span, true)));
                }
                // </ close fragment
                if kind == Kind::Slash {
                    self.rewind(checkpoint);
                    return None;
                }
                // <! HTML comment - skip and continue parsing
                if kind == Kind::Bang {
                    self.skip_jsx_html_comment();
                    return self.parse_jsx_child();
                }
                self.unexpected()
            }
            Kind::LCurly => {
                let span_start = self.start_span();
                self.bump_any(); // bump `{`

                // {...expr}
                if self.eat(Kind::Dot3) {
                    return Some(JSXChild::Spread(self.parse_jsx_spread_child(span_start)));
                }
                // {expr}
                Some(JSXChild::ExpressionContainer(
                    self.parse_jsx_expression_container(span_start, /* in_jsx_child */ true),
                ))
            }
            // text
            Kind::JSXText => Some(JSXChild::Text(self.parse_jsx_text())),
            _ => self.unexpected(),
        }
    }

    ///   { `JSXChildExpression_opt` }
    pub(crate) fn parse_jsx_expression_container(
        &mut self,
        span_start: u32,
        in_jsx_child: bool,
    ) -> Box<'a, JSXExpressionContainer<'a>> {
        let expr = if self.at(Kind::RCurly) {
            if in_jsx_child {
                self.expect_jsx_child(Kind::RCurly);
            } else {
                self.expect(Kind::RCurly);
            }
            let span = self.end_span(span_start);

            // Empty expression is not allowed in JSX attribute value
            // e.g. `<C attr={} />`
            if !in_jsx_child {
                self.error(diagnostics::jsx_attribute_value_empty_expression(span));
            }

            // Handle comment between curly braces (ex. `{/* comment */}`)
            //                                            ^^^^^^^^^^^^^ span
            let expr = self.ast.jsx_empty_expression(Span::new(span.start + 1, span.end - 1));
            JSXExpression::EmptyExpression(expr)
        } else {
            // In Astro mode with JSX children starting with `<`, parse as JSX children directly.
            // This allows: { <div>1</div> <div>2</div> } without explicit fragments.
            // We must detect this BEFORE calling parse_expr() because parse_expr() would
            // misinterpret the `<` after the first element as a binary comparison operator.
            if self.source_type.is_astro() && in_jsx_child && self.at(Kind::LAngle) {
                let expr = self.parse_astro_jsx_children_in_expression(span_start);
                return self.ast.alloc_jsx_expression_container(self.end_span(span_start), expr);
            }

            // Parse the expression normally
            let first_expr = self.parse_expr();

            if in_jsx_child {
                self.expect_jsx_child(Kind::RCurly);
            } else {
                self.expect(Kind::RCurly);
            }
            JSXExpression::from(first_expr)
        };

        self.ast.alloc_jsx_expression_container(self.end_span(span_start), expr)
    }

    /// Parse JSX children in an expression container (Astro-specific).
    /// Called when we see `{` followed by `<` in Astro mode.
    /// This allows: `{ <div>1</div> <div>2</div> }` without explicit fragments.
    ///
    /// We parse JSX children similarly to parse_jsx_children, but we stop at `}`
    /// instead of `</`. After each child, we re-lex as JSX child to get the next token.
    fn parse_astro_jsx_children_in_expression(&mut self, span_start: u32) -> JSXExpression<'a> {
        let fragment_span_start = span_start + 1; // After the `{`
        let mut children = self.ast.vec();

        // We're currently at `<` (LAngle). Parse JSX children until we hit `}`.
        loop {
            if self.at(Kind::Eof) {
                break;
            }

            match self.cur_kind() {
                Kind::LAngle => {
                    let child_span = self.start_span();
                    // Use regular bump_any() after `<` - we expect identifier/keyword/slash/etc
                    self.bump_any();

                    let kind = self.cur_kind();

                    if kind == Kind::RAngle {
                        // `<>` - fragment
                        let fragment = self.parse_jsx_fragment(child_span, true);
                        children.push(JSXChild::Fragment(fragment));
                        // parse_jsx_fragment with in_jsx_child=true calls expect_jsx_child(RAngle)
                        // Same Eof handling as elements (see below).
                        if self.at(Kind::Eof) {
                            self.lexer.errors.pop();
                            self.token = self.lexer.next_token();
                        }
                    } else if kind == Kind::Ident || kind.is_any_keyword() {
                        // `<ident` - element
                        let element = self.parse_jsx_element(child_span, true);
                        children.push(JSXChild::Element(element));
                        // parse_jsx_element with in_jsx_child=true calls expect_jsx_child(RAngle)
                        // which calls next_jsx_child(). But next_jsx_child() returns Eof with error
                        // when text is followed by `}` (since `}` is unexpected in normal JSX).
                        // In our Astro context, `}` is the valid terminator, so if we got Eof,
                        // re-lex with regular tokenization to check for `}`.
                        if self.at(Kind::Eof) {
                            // Pop the spurious "unexpected `}`" error from lexer
                            self.lexer.errors.pop();
                            self.token = self.lexer.next_token();
                        }
                    } else if kind == Kind::Slash {
                        // `</` - unexpected closing tag, error
                        let _: () = self.unexpected();
                        break;
                    } else if kind == Kind::Bang {
                        // `<!` - HTML comment, skip
                        self.skip_jsx_html_comment();
                        // Re-lex for next child
                        self.token = self.lexer.next_jsx_child();
                    } else {
                        let _: () = self.unexpected();
                        break;
                    }
                    // Note: Don't call next_jsx_child() here - parse_jsx_element/fragment
                    // with in_jsx_child=true already advances to the next JSX child token
                }
                Kind::JSXText => {
                    let text = self.parse_jsx_text();
                    children.push(JSXChild::Text(text));
                    // parse_jsx_text uses bump_any() which calls next_token().
                    // The token is already set to the next token (LAngle, LCurly, RCurly, etc.)
                    // Don't call next_jsx_child() - we're already positioned correctly.
                }
                Kind::LCurly => {
                    // Nested expression container
                    let nested_span = self.start_span();
                    self.bump_any(); // bump `{`

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
                    // Re-lex for next child
                    self.token = self.lexer.next_jsx_child();
                }
                Kind::RCurly => {
                    // End of expression container - don't consume here, let caller handle it
                    break;
                }
                _ => {
                    // For other tokens (like Eof from next_jsx_child hitting `}`),
                    // try to re-lex and check again
                    self.token = self.lexer.next_jsx_child();
                    if self.at(Kind::Eof) {
                        break;
                    }
                }
            }
        }

        self.expect_jsx_child(Kind::RCurly);

        // If only one child and it's an element or fragment, return it directly
        // (no need for implicit fragment wrapper)
        if children.len() == 1 {
            match children.pop().unwrap() {
                JSXChild::Element(el) => return JSXExpression::JSXElement(el),
                JSXChild::Fragment(frag) => return JSXExpression::JSXFragment(frag),
                other => children.push(other),
            }
        }

        // Wrap all children in an implicit fragment
        let fragment_span = Span::new(fragment_span_start, self.prev_token_end);
        let opening = self.ast.jsx_opening_fragment(Span::empty(fragment_span_start));
        let closing = self.ast.jsx_closing_fragment(Span::empty(self.prev_token_end));
        let fragment = self.ast.alloc_jsx_fragment(fragment_span, opening, children, closing);
        JSXExpression::JSXFragment(fragment)
    }

    /// Parse multiple JSX elements in expression context (Astro-specific).
    /// Called from binary expression parsing when we have a JSX element followed by `<`.
    /// This allows: `<div>1</div><div>2</div>` without explicit fragments in any expression.
    pub(crate) fn parse_astro_multiple_jsx_in_expression(
        &mut self,
        span_start: u32,
        first_element: Expression<'a>,
    ) -> Expression<'a> {
        let mut children = self.ast.vec();

        // Add first element as a child
        match first_element {
            Expression::JSXElement(el) => children.push(JSXChild::Element(el)),
            Expression::JSXFragment(frag) => children.push(JSXChild::Fragment(frag)),
            _ => unreachable!(
                "parse_astro_multiple_jsx_in_expression called with non-JSX expression"
            ),
        }

        // Parse additional JSX elements while we see `<` followed by JSX content
        while self.at(Kind::LAngle) {
            let checkpoint = self.checkpoint();
            let child_span = self.start_span();
            self.bump_any(); // bump `<`

            let kind = self.cur_kind();
            if kind == Kind::RAngle {
                // `<>` - fragment
                let fragment = self.parse_jsx_fragment(child_span, false);
                children.push(JSXChild::Fragment(fragment));
            } else if kind == Kind::Ident || kind.is_any_keyword() {
                // `<ident` - element
                let element = self.parse_jsx_element(child_span, false);
                children.push(JSXChild::Element(element));
            } else {
                // Not a JSX element/fragment, rewind and stop
                self.rewind(checkpoint);
                break;
            }
        }

        // If only one child (shouldn't happen but handle gracefully), return it directly
        if children.len() == 1 {
            return match children.pop().unwrap() {
                JSXChild::Element(el) => Expression::JSXElement(el),
                JSXChild::Fragment(frag) => Expression::JSXFragment(frag),
                _ => unreachable!(),
            };
        }

        // Wrap all children in an implicit fragment
        let fragment_span = Span::new(span_start, self.prev_token_end);
        let opening = self.ast.jsx_opening_fragment(Span::empty(span_start));
        let closing = self.ast.jsx_closing_fragment(Span::empty(self.prev_token_end));
        let fragment = self.ast.alloc_jsx_fragment(fragment_span, opening, children, closing);
        Expression::JSXFragment(fragment)
    }

    /// `JSXChildExpression` :
    ///   { ... `AssignmentExpression` }
    pub(crate) fn parse_jsx_spread_child(
        &mut self,
        span_start: u32,
    ) -> Box<'a, JSXSpreadChild<'a>> {
        let expr = self.parse_expr();
        self.expect_jsx_child(Kind::RCurly);
        self.ast.alloc_jsx_spread_child(self.end_span(span_start), expr)
    }

    /// `JSXAttributes` :
    ///   `JSXSpreadAttribute` `JSXAttributes_opt`
    ///   `JSXAttribute` `JSXAttributes_opt`
    fn parse_jsx_attributes(&mut self) -> Vec<'a, JSXAttributeItem<'a>> {
        let mut attributes = self.ast.vec();
        let is_astro = self.source_type.is_astro();
        loop {
            let kind = self.cur_kind();
            if matches!(kind, Kind::Eof | Kind::LAngle | Kind::RAngle | Kind::Slash)
                || self.fatal_error.is_some()
            {
                break;
            }
            let attribute = match kind {
                Kind::LCurly => {
                    // In Astro, `{prop}` can be shorthand for `prop={prop}`
                    if is_astro {
                        let checkpoint = self.checkpoint();
                        self.bump_any(); // bump `{`

                        // Check if this is a shorthand: `{identifier}`
                        if self.at(Kind::Ident) || self.cur_kind().is_any_keyword() {
                            let ident_span = self.start_span();
                            let name = self.cur_src();
                            self.bump_any();

                            if self.at(Kind::RCurly) {
                                // This is shorthand: `{prop}` -> `prop={prop}`
                                self.bump_any(); // bump `}`
                                let ident_span = self.end_span(ident_span);
                                let name = Atom::from(name);
                                let identifier = self.ast.jsx_identifier(ident_span, name);
                                let attr_name =
                                    JSXAttributeName::Identifier(self.alloc(identifier));

                                // Create expression for value: the identifier reference
                                let ident_ref = self.ast.identifier_reference(ident_span, name);
                                let expr = Expression::Identifier(self.alloc(ident_ref));
                                let expr_container = self.ast.alloc_jsx_expression_container(
                                    ident_span,
                                    JSXExpression::from(expr),
                                );
                                let value = JSXAttributeValue::ExpressionContainer(expr_container);

                                let attr = self.ast.alloc_jsx_attribute(
                                    ident_span,
                                    attr_name,
                                    Some(value),
                                );
                                attributes.push(JSXAttributeItem::Attribute(attr));
                                continue;
                            }
                        }

                        // Not shorthand, rewind and parse as spread
                        self.rewind(checkpoint);
                    }
                    JSXAttributeItem::SpreadAttribute(self.parse_jsx_spread_attribute())
                }
                _ => {
                    // In Astro, use permissive attribute name parsing
                    if is_astro {
                        JSXAttributeItem::Attribute(self.parse_astro_attribute())
                    } else {
                        JSXAttributeItem::Attribute(self.parse_jsx_attribute())
                    }
                }
            };
            attributes.push(attribute);
        }
        attributes
    }

    /// `JSXAttribute` :
    ///   `JSXAttributeName` `JSXAttributeInitializer_opt`
    fn parse_jsx_attribute(&mut self) -> Box<'a, JSXAttribute<'a>> {
        let span = self.start_span();
        let name = self.parse_jsx_attribute_name();
        let value = if self.at(Kind::Eq) {
            self.expect_jsx_attribute_value(Kind::Eq);
            Some(self.parse_jsx_attribute_value())
        } else {
            None
        };
        self.ast.alloc_jsx_attribute(self.end_span(span), name, value)
    }

    /// Parse an Astro attribute which can have special characters in the name.
    /// Examples: `@click="handler"`, `x.data="value"`, `:any`, `class:list`
    fn parse_astro_attribute(&mut self) -> Box<'a, JSXAttribute<'a>> {
        let span = self.start_span();

        // The current token might be a special character like `:` or `@` that was
        // tokenized separately. We need to reposition the lexer to the start of
        // the current token and re-read the full attribute name.
        let token_start = self.cur_token().span().start;
        self.lexer.set_position_for_astro(token_start);

        // Read the full attribute name including special chars
        self.read_astro_attribute_name();
        let name_span = self.cur_token().span();
        let name = name_span.source_text(self.source_text);
        self.bump_any(); // consume the identifier token

        let identifier = self.ast.jsx_identifier(name_span, name);
        let attr_name = JSXAttributeName::Identifier(self.alloc(identifier));

        let value = if self.at(Kind::Eq) {
            self.expect_jsx_attribute_value(Kind::Eq);
            Some(self.parse_jsx_attribute_value())
        } else {
            None
        };

        self.ast.alloc_jsx_attribute(self.end_span(span), attr_name, value)
    }

    /// `JSXSpreadAttribute` :
    ///   { ... `AssignmentExpression` }
    fn parse_jsx_spread_attribute(&mut self) -> Box<'a, JSXSpreadAttribute<'a>> {
        let span = self.start_span();
        self.bump_any(); // bump `{`
        self.expect(Kind::Dot3);
        let argument = self.parse_expr();
        self.expect(Kind::RCurly);
        self.ast.alloc_jsx_spread_attribute(self.end_span(span), argument)
    }

    /// `JSXAttributeName` :
    ///   `JSXIdentifier`
    ///   `JSXNamespacedName`
    fn parse_jsx_attribute_name(&mut self) -> JSXAttributeName<'a> {
        let span = self.start_span();
        let identifier = self.parse_jsx_identifier();

        if self.eat(Kind::Colon) {
            let property = self.parse_jsx_identifier();
            return self.ast.jsx_attribute_name_namespaced_name(
                self.end_span(span),
                identifier,
                property,
            );
        }

        JSXAttributeName::Identifier(self.alloc(identifier))
    }

    fn parse_jsx_attribute_value(&mut self) -> JSXAttributeValue<'a> {
        match self.cur_kind() {
            Kind::Str => {
                let str_lit = self.parse_literal_string();
                JSXAttributeValue::StringLiteral(self.alloc(str_lit))
            }
            Kind::LCurly => {
                let span_start = self.start_span();
                self.bump_any(); // bump `{`

                let expr =
                    self.parse_jsx_expression_container(span_start, /* in_jsx_child */ false);
                JSXAttributeValue::ExpressionContainer(expr)
            }
            Kind::LAngle => match self.parse_jsx_expression() {
                Expression::JSXFragment(fragment) => JSXAttributeValue::Fragment(fragment),
                Expression::JSXElement(element) => JSXAttributeValue::Element(element),
                _ => self.unexpected(),
            },
            // In Astro, template literals can be used directly as attribute values
            // e.g., <Component attr=`hello ${value}` />
            Kind::NoSubstitutionTemplate | Kind::TemplateHead if self.source_type.is_astro() => {
                let span = self.start_span();
                let template_lit = self.parse_template_literal(false);
                let expr = Expression::TemplateLiteral(self.alloc(template_lit));
                let expr_container = self
                    .ast
                    .alloc_jsx_expression_container(self.end_span(span), JSXExpression::from(expr));
                JSXAttributeValue::ExpressionContainer(expr_container)
            }
            _ => self.unexpected(),
        }
    }

    /// `JSXIdentifier` :
    ///   `IdentifierStart`
    ///   `JSXIdentifier` `IdentifierPart`
    ///   `JSXIdentifier` [no `WhiteSpace` or Comment here] -
    fn parse_jsx_identifier(&mut self) -> JSXIdentifier<'a> {
        let span = self.start_span();
        let kind = self.cur_kind();
        if kind != Kind::Ident && !kind.is_any_keyword() {
            return self.unexpected();
        }
        // Currently at a valid normal Ident or Keyword, keep on lexing for `-` in `<component-name />`
        self.continue_lex_jsx_identifier();
        self.bump_any();
        let span = self.end_span(span);
        let name = span.source_text(self.source_text);
        self.ast.jsx_identifier(span, name)
    }

    pub(crate) fn parse_jsx_text(&mut self) -> Box<'a, JSXText<'a>> {
        let span = self.cur_token().span();
        let raw = Atom::from(self.cur_src());
        let value = Atom::from(self.cur_string());
        self.bump_any();
        self.ast.alloc_jsx_text(span, value, Some(raw))
    }

    fn jsx_element_name_eq(lhs: &JSXElementName<'a>, rhs: &JSXElementName<'a>) -> bool {
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
                Self::jsx_member_expression_eq(lhs, rhs)
            }
            (JSXElementName::ThisExpression(_), JSXElementName::ThisExpression(_)) => true,
            _ => false,
        }
    }

    fn jsx_member_expression_eq(
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
            ) => Self::jsx_member_expression_eq(lhs, rhs),
            (
                JSXMemberExpressionObject::ThisExpression(_),
                JSXMemberExpressionObject::ThisExpression(_),
            ) => true,
            _ => false,
        }
    }

    /// Skip an HTML comment `<!-- ... -->` inside JSX.
    ///
    /// HTML comments are valid in Astro files and some JSX contexts.
    /// We skip them entirely rather than producing an AST node.
    fn skip_jsx_html_comment(&mut self) {
        // We're at `!` after `<`
        self.bump_any(); // skip `!`

        // Look for `--` to start comment
        if self.at(Kind::Minus2) {
            self.bump_any(); // skip `--`
        } else if self.at(Kind::Minus) {
            self.bump_any(); // skip first `-`
            if self.at(Kind::Minus) {
                self.bump_any(); // skip second `-`
            }
        }

        // Skip until we find `-->`
        // The lexer is in JSX mode, so we need to scan the source directly
        let start_pos = self.cur_token().span().start as usize;
        if let Some(rest) = self.source_text.get(start_pos..)
            && let Some(end_pos) = rest.find("-->")
        {
            // Move past the `-->` by setting the lexer position
            #[expect(clippy::cast_possible_truncation)]
            let new_pos = (start_pos + end_pos + 3) as u32;
            self.lexer.set_position_for_astro(new_pos);
            self.token = self.lexer.next_jsx_child();
            return;
        }

        // If no closing `-->` found, skip tokens until EOF (error recovery)
        while !self.at(Kind::Eof) {
            if self.at(Kind::Minus2) || self.at(Kind::Minus) {
                self.bump_any();
                if self.at(Kind::Minus) {
                    self.bump_any();
                }
                if self.at(Kind::RAngle) {
                    self.bump_any();
                    break;
                }
            } else {
                self.bump_any();
            }
        }
    }
}
