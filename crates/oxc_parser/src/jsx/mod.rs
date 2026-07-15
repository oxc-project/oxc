//! [JSX](https://facebook.github.io/jsx)

use oxc_allocator::{Allocator, ArenaBox, ArenaVec, Dummy, GetAllocator};
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};
use oxc_str::Str;

use crate::{ParserConfig as Config, ParserImpl, diagnostics, lexer::Kind};

/// Represents either a closing JSX element or fragment.
enum JSXClosing<'a> {
    /// [`JSXClosingElement`]
    Element(ArenaBox<'a, JSXClosingElement<'a>>),
    /// [`JSXClosingFragment`]
    Fragment(JSXClosingFragment),
}

impl<'a> Dummy<'a> for JSXClosing<'a> {
    fn dummy(allocator: &'a Allocator) -> Self {
        JSXClosing::Fragment(Dummy::dummy(allocator))
    }
}

impl<'a, C: Config> ParserImpl<'a, C> {
    pub(crate) fn parse_jsx_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `<`
        let kind = self.cur_kind();
        let expr = if kind == Kind::RAngle {
            Expression::JSXFragment(self.parse_jsx_fragment(span, false))
        } else if kind.is_identifier_or_keyword() {
            Expression::JSXElement(self.parse_jsx_element(span, false))
        } else {
            return self.unexpected();
        };

        // A top-level JSX element/fragment immediately followed by `<` is a second,
        // adjacent JSX element that isn't wrapped in an enclosing tag, e.g.
        // `<div></div><span></span>`.
        if self.at(Kind::LAngle) && self.fatal_error.is_none() {
            self.set_fatal_error(diagnostics::adjacent_jsx_elements(self.cur_token().span()));
        }

        expr
    }

    /// `JSXFragment` :
    ///   < > `JSXChildren_opt` < / >
    fn parse_jsx_fragment(
        &mut self,
        span: u32,
        in_jsx_child: bool,
    ) -> ArenaBox<'a, JSXFragment<'a>> {
        self.expect_jsx_child(Kind::RAngle);
        let opening_fragment = JSXOpeningFragment::new(self.end_span(span), self);
        let (children, closing) = self.parse_jsx_children_and_closing(in_jsx_child);
        let closing_fragment = match closing {
            JSXClosing::Fragment(f) => f,
            JSXClosing::Element(e) => {
                // Got a closing element when expecting a closing fragment
                self.error(diagnostics::jsx_fragment_no_match(
                    opening_fragment.span,
                    e.name.span(),
                ));
                JSXClosingFragment::new(e.span, self)
            }
        };
        JSXFragment::boxed(self.end_span(span), opening_fragment, children, closing_fragment, self)
    }

    /// `JSXElement` :
    ///   `JSXSelfClosingElement`
    ///   `JSXOpeningElement` `JSXChildren_opt` `JSXClosingElement`
    /// `in_jsx_child`:
    ///     used for telling `JSXClosingElement` to parse the next jsx child or not
    ///     true when inside jsx element, false when at top level expression
    fn parse_jsx_element(&mut self, span: u32, in_jsx_child: bool) -> ArenaBox<'a, JSXElement<'a>> {
        let (opening_element, self_closing) = self.parse_jsx_opening_element(span, in_jsx_child);
        let (children, closing_element) = if self_closing {
            (ArenaVec::new_in(self), None)
        } else {
            let (children, closing) = self.parse_jsx_children_and_closing(in_jsx_child);
            let closing_element = match closing {
                JSXClosing::Element(e) => {
                    if !Self::jsx_element_name_eq(&opening_element.name, &e.name) {
                        self.error(diagnostics::jsx_element_no_match(
                            opening_element.name.span(),
                            e.name.span(),
                            opening_element.name.span().source_text(self.source_text),
                        ));
                    }
                    e
                }
                JSXClosing::Fragment(f) => {
                    // Got a closing fragment when expecting a closing element
                    return self.fatal_error(diagnostics::jsx_element_no_match(
                        opening_element.name.span(),
                        f.span,
                        opening_element.name.span().source_text(self.source_text),
                    ));
                }
            };
            (children, Some(closing_element))
        };
        JSXElement::boxed(self.end_span(span), opening_element, children, closing_element, self)
    }

    /// `JSXOpeningElement` :
    /// < `JSXElementName` `JSXAttributes_opt` >
    fn parse_jsx_opening_element(
        &mut self,
        span: u32,
        in_jsx_child: bool,
    ) -> (
        ArenaBox<'a, JSXOpeningElement<'a>>,
        bool, // `true` if self-closing
    ) {
        let name = self.parse_jsx_element_name();
        // <Component<TsType> for tsx
        let type_arguments = if self.is_ts { self.try_parse_type_arguments() } else { None };
        let attributes = self.parse_jsx_attributes();
        let self_closing = self.eat(Kind::Slash);
        if !self_closing || in_jsx_child {
            self.expect_jsx_child(Kind::RAngle);
        } else {
            self.expect(Kind::RAngle);
        }
        let elem =
            JSXOpeningElement::boxed(self.end_span(span), name, type_arguments, attributes, self);
        (elem, self_closing)
    }

    /// `JSXElementName` :
    ///   `JSXIdentifier`
    ///   `JSXNamespacedName`
    ///   `JSXMemberExpression`
    fn parse_jsx_element_name(&mut self) -> JSXElementName<'a> {
        let span = self.start_span();
        let (identifier, contains_dash) = self.parse_jsx_identifier();

        // <namespace:property />
        if self.eat(Kind::Colon) {
            let (property, _) = self.parse_jsx_identifier();
            return JSXElementName::new_namespaced_name(
                self.end_span(span),
                identifier,
                property,
                self,
            );
        }

        // <member.foo.bar />
        if self.at(Kind::Dot) {
            return JSXElementName::MemberExpression(
                self.parse_jsx_member_expression(span, &identifier),
            );
        }

        if self.fatal_error.is_some() {
            return JSXElementName::dummy(self.allocator());
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
        let is_reference = !contains_dash // Exclude hyphenated custom elements
            && match name.as_bytes()[0] {
                b if b.is_ascii() => !b.is_ascii_lowercase(), // Matches A-Z, _, $
                _ => true, // Non-ASCII characters are always treated as references
            };

        if is_reference {
            JSXElementName::new_identifier_reference(
                identifier.span,
                self.ident(identifier.name.as_str()),
                self,
            )
        } else if name == "this" {
            JSXElementName::new_this_expression(identifier.span, self)
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
    ) -> ArenaBox<'a, JSXMemberExpression<'a>> {
        let mut object = if object.name == "this" {
            JSXMemberExpressionObject::new_this_expression(object.span, self)
        } else {
            JSXMemberExpressionObject::new_identifier_reference(
                object.span,
                self.ident(object.name.as_str()),
                self,
            )
        };

        let mut span = Span::new(span, 0);
        let mut property = None;

        while self.eat(Kind::Dot) && self.fatal_error.is_none() {
            // <foo.bar.baz>
            if let Some(prop) = property {
                object = JSXMemberExpressionObject::new_member_expression(span, object, prop, self);
            }

            // <foo.bar>
            let (ident, contains_dash) = self.parse_jsx_identifier();
            // `<foo.bar- />` is a syntax error.
            if contains_dash {
                let error = diagnostics::identifier_expected_jsx_no_hyphen(ident.span);
                return self.fatal_error(error);
            }
            property = Some(ident);
            span = self.end_span(span.start);
        }

        if let Some(property) = property {
            return JSXMemberExpression::boxed(self.end_span(span.start), object, property, self);
        }

        self.unexpected()
    }

    /// `JSXChildren` :
    ///   `JSXChild` `JSXChildren_opt`
    /// Parses children and the closing element/fragment in one pass.
    /// Returns `(children, closing)` where closing is either a `JSXClosingElement` or `JSXClosingFragment`.
    fn parse_jsx_children_and_closing(
        &mut self,
        in_jsx_child: bool,
    ) -> (ArenaVec<'a, JSXChild<'a>>, JSXClosing<'a>) {
        let mark = self.scratch_mark::<JSXChild<'a>>();
        loop {
            if self.fatal_error.is_some() {
                // Return dummy closing fragment on fatal error
                let closing = JSXClosingFragment::new(self.cur_token().span(), self);
                return (self.scratch_take(mark), JSXClosing::Fragment(closing));
            }

            match self.cur_kind() {
                Kind::LAngle => {
                    let span = self.start_span();
                    self.bump_any(); // bump `<`
                    let kind = self.cur_kind();

                    // <> open nested fragment
                    if kind == Kind::RAngle {
                        let child = JSXChild::Fragment(self.parse_jsx_fragment(span, true));
                        self.scratch_push(child);
                        continue;
                    }

                    // <ident open nested element
                    if kind == Kind::Ident || kind.is_any_keyword() {
                        let child = JSXChild::Element(self.parse_jsx_element(span, true));
                        self.scratch_push(child);
                        continue;
                    }

                    // </ closing tag - parse it inline and return
                    if kind == Kind::Slash {
                        self.bump_any(); // bump `/`
                        let closing = self.parse_jsx_closing_inline(span, in_jsx_child);
                        return (self.scratch_take(mark), closing);
                    }

                    // Unexpected token after `<`
                    return (self.scratch_take(mark), self.unexpected());
                }
                Kind::LCurly => {
                    let span_start = self.start_span();
                    self.bump_any(); // bump `{`

                    // {...expr}
                    if self.eat(Kind::Dot3) {
                        let child = JSXChild::Spread(self.parse_jsx_spread_child(span_start));
                        self.scratch_push(child);
                        continue;
                    }
                    // {expr}
                    let child =
                        JSXChild::ExpressionContainer(self.parse_jsx_expression_container(
                            span_start, /* in_jsx_child */ true,
                        ));
                    self.scratch_push(child);
                }
                // text
                Kind::JSXText => {
                    let child = JSXChild::Text(self.parse_jsx_text());
                    self.scratch_push(child);
                }
                _ => {
                    // Unexpected token in JSX children
                    return (self.scratch_take(mark), self.unexpected());
                }
            }
        }
    }

    /// Parses the closing element or fragment after `</` has been consumed.
    fn parse_jsx_closing_inline(
        &mut self,
        open_angle_span: u32,
        in_jsx_child: bool,
    ) -> JSXClosing<'a> {
        if self.at(Kind::RAngle) {
            // Closing fragment: </>
            if in_jsx_child {
                self.expect_jsx_child(Kind::RAngle);
            } else {
                self.expect(Kind::RAngle);
            }
            JSXClosing::Fragment(JSXClosingFragment::new(self.end_span(open_angle_span), self))
        } else {
            // Closing element: </name>
            let name = self.parse_jsx_element_name();
            if in_jsx_child {
                self.expect_jsx_child(Kind::RAngle);
            } else {
                self.expect(Kind::RAngle);
            }
            JSXClosing::Element(JSXClosingElement::boxed(
                self.end_span(open_angle_span),
                name,
                self,
            ))
        }
    }

    ///   { `JSXChildExpression_opt` }
    fn parse_jsx_expression_container(
        &mut self,
        span_start: u32,
        in_jsx_child: bool,
    ) -> ArenaBox<'a, JSXExpressionContainer<'a>> {
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
            JSXExpression::new_empty_expression(Span::new(span.start + 1, span.end - 1), self)
        } else {
            let expr = JSXExpression::from(self.parse_expr());
            // JSX expressions may not use the comma operator.
            if matches!(expr, JSXExpression::SequenceExpression(_)) {
                self.error(diagnostics::jsx_expressions_may_not_use_the_comma_operator(
                    expr.span(),
                ));
            }
            if in_jsx_child {
                self.expect_jsx_child(Kind::RCurly);
            } else {
                self.expect(Kind::RCurly);
            }
            expr
        };

        JSXExpressionContainer::boxed(self.end_span(span_start), expr, self)
    }

    /// `JSXChildExpression` :
    ///   { ... `AssignmentExpression` }
    fn parse_jsx_spread_child(&mut self, span_start: u32) -> ArenaBox<'a, JSXSpreadChild<'a>> {
        let expr = self.parse_expr();
        self.expect_jsx_child(Kind::RCurly);
        JSXSpreadChild::boxed(self.end_span(span_start), expr, self)
    }

    /// `JSXAttributes` :
    ///   `JSXSpreadAttribute` `JSXAttributes_opt`
    ///   `JSXAttribute` `JSXAttributes_opt`
    fn parse_jsx_attributes(&mut self) -> ArenaVec<'a, JSXAttributeItem<'a>> {
        let mark = self.scratch_mark::<JSXAttributeItem<'a>>();
        loop {
            let kind = self.cur_kind();
            if matches!(kind, Kind::LAngle | Kind::RAngle | Kind::Slash)
                || self.fatal_error.is_some()
            {
                break;
            }
            let attribute = match kind {
                Kind::LCurly => {
                    JSXAttributeItem::SpreadAttribute(self.parse_jsx_spread_attribute())
                }
                _ => JSXAttributeItem::Attribute(self.parse_jsx_attribute()),
            };
            self.scratch_push(attribute);
        }
        self.scratch_take(mark)
    }

    /// `JSXAttribute` :
    ///   `JSXAttributeName` `JSXAttributeInitializer_opt`
    fn parse_jsx_attribute(&mut self) -> ArenaBox<'a, JSXAttribute<'a>> {
        let span = self.start_span();
        let name = self.parse_jsx_attribute_name();
        let value = if self.at(Kind::Eq) {
            self.advance_for_jsx_attribute_value();
            Some(self.parse_jsx_attribute_value())
        } else {
            None
        };
        JSXAttribute::boxed(self.end_span(span), name, value, self)
    }

    /// `JSXSpreadAttribute` :
    ///   { ... `AssignmentExpression` }
    fn parse_jsx_spread_attribute(&mut self) -> ArenaBox<'a, JSXSpreadAttribute<'a>> {
        let span = self.start_span();
        self.bump_any(); // bump `{`
        self.expect(Kind::Dot3);
        let argument = self.parse_expr();
        self.expect(Kind::RCurly);
        JSXSpreadAttribute::boxed(self.end_span(span), argument, self)
    }

    /// `JSXAttributeName` :
    ///   `JSXIdentifier`
    ///   `JSXNamespacedName`
    fn parse_jsx_attribute_name(&mut self) -> JSXAttributeName<'a> {
        let span = self.start_span();
        let (identifier, _) = self.parse_jsx_identifier();

        if self.eat(Kind::Colon) {
            let (property, _) = self.parse_jsx_identifier();
            return JSXAttributeName::new_namespaced_name(
                self.end_span(span),
                identifier,
                property,
                self,
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
            _ => self.unexpected(),
        }
    }

    /// `JSXIdentifier` :
    ///   `IdentifierStart`
    ///   `JSXIdentifier` `IdentifierPart`
    ///   `JSXIdentifier` [no `WhiteSpace` or Comment here] -
    fn parse_jsx_identifier(
        &mut self,
    ) -> (
        JSXIdentifier<'a>, // JSX identifier
        bool,              // `true` if contains `-`
    ) {
        let span = self.start_span();
        let kind = self.cur_kind();
        if kind != Kind::Ident && !kind.is_any_keyword() {
            return (self.unexpected(), false);
        }
        // Currently at a valid normal Ident or Keyword, keep on lexing for `-` in `<component-name />`
        let contains_dash = self.continue_lex_jsx_identifier();
        self.bump_any();
        let span = self.end_span(span);
        let name = span.source_text(self.source_text);
        let identifier = JSXIdentifier::new(span, name, self);
        (identifier, contains_dash)
    }

    fn parse_jsx_text(&mut self) -> ArenaBox<'a, JSXText<'a>> {
        let span = self.cur_token().span();
        let raw = Str::from(self.cur_src());
        let value = Str::from(self.cur_string());
        self.bump_any();
        JSXText::boxed(span, value, Some(raw), self)
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
}
