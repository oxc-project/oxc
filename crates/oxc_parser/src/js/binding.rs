use oxc_ast::{NONE, ast::*};
use oxc_span::GetSpan;

use crate::{Context, ParserImpl, diagnostics, lexer::Kind};

impl<'a> ParserImpl<'a> {
    /// `BindingElement`
    ///     `SingleNameBinding`
    ///     `BindingPattern`[?Yield, ?Await] `Initializer`[+In, ?Yield, ?Await]opt
    pub(super) fn parse_binding_pattern_with_initializer(&mut self) -> BindingPattern<'a> {
        let span = self.start_span();
        let pattern = self.parse_binding_pattern(true);
        self.context_add(Context::In, |p| p.parse_initializer(span, pattern))
    }

    pub(super) fn parse_binding_pattern(&mut self, allow_question: bool) -> BindingPattern<'a> {
        let mut kind = self.parse_binding_pattern_kind();
        let optional = if allow_question && self.is_ts { self.eat(Kind::Question) } else { false };
        let type_annotation = self.parse_ts_type_annotation();
        if let Some(type_annotation) = &type_annotation {
            Self::extend_binding_pattern_span_end(type_annotation.span.end, &mut kind);
        } else if optional {
            Self::extend_binding_pattern_span_end(self.prev_token_end, &mut kind);
        }
        self.ast.binding_pattern(kind, type_annotation, optional)
    }

    pub(crate) fn parse_binding_pattern_kind(&mut self) -> BindingPatternKind<'a> {
        match self.cur_kind() {
            Kind::LCurly => self.parse_object_binding_pattern(),
            Kind::LBrack => self.parse_array_binding_pattern(),
            _ => self.parse_binding_pattern_identifier(),
        }
    }

    fn parse_binding_pattern_identifier(&mut self) -> BindingPatternKind<'a> {
        let ident = self.parse_binding_identifier();
        BindingPatternKind::BindingIdentifier(self.alloc(ident))
    }

    /// Section 14.3.3 Object Binding Pattern
    fn parse_object_binding_pattern(&mut self) -> BindingPatternKind<'a> {
        let span = self.start_span();
        let opening_span = self.cur_token().span();
        self.expect(Kind::LCurly);
        let (list, rest) = self.parse_delimited_list_with_rest(
            Kind::RCurly,
            opening_span,
            Self::parse_binding_property,
            diagnostics::binding_rest_element_last,
        );
        if let Some(rest) = &rest {
            if let Some(ty) = &rest.argument.type_annotation {
                self.error(diagnostics::rest_element_property_name(ty.span));
            }
            if !matches!(&rest.argument.kind, BindingPatternKind::BindingIdentifier(_)) {
                let error = diagnostics::invalid_binding_rest_element(rest.argument.span());
                return self.fatal_error(error);
            }
        }
        self.expect(Kind::RCurly);
        self.ast.binding_pattern_kind_object_pattern(
            self.end_span(span),
            list,
            rest.map(|r| self.alloc(r)),
        )
    }

    /// Section 14.3.3 Array Binding Pattern
    fn parse_array_binding_pattern(&mut self) -> BindingPatternKind<'a> {
        let span = self.start_span();
        let opening_span = self.cur_token().span();
        self.expect(Kind::LBrack);
        let (list, rest) = self.parse_delimited_list_with_rest(
            Kind::RBrack,
            opening_span,
            Self::parse_array_binding_element,
            diagnostics::binding_rest_element_last,
        );
        if let Some(rest) = &rest
            && let Some(ty) = &rest.argument.type_annotation
        {
            self.error(diagnostics::rest_element_property_name(ty.span));
        }
        self.expect(Kind::RBrack);
        self.ast.binding_pattern_kind_array_pattern(
            self.end_span(span),
            list,
            rest.map(|r| self.alloc(r)),
        )
    }

    fn parse_array_binding_element(&mut self) -> Option<BindingPattern<'a>> {
        if self.at(Kind::Comma) {
            None
        } else {
            Some(self.parse_binding_pattern_with_initializer())
        }
    }

    /// Section 14.3.3 Binding Rest Property
    pub(crate) fn parse_rest_element(&mut self) -> BindingRestElement<'a> {
        let span = self.start_span();
        self.bump_any(); // advance `...`
        let init_span = self.start_span();

        let kind = self.parse_binding_pattern_kind();
        // Rest element does not allow `?`, checked in checker/typescript.rs
        if self.at(Kind::Question) && self.is_ts {
            let span = self.cur_token().span();
            self.bump_any();
            self.error(diagnostics::a_rest_parameter_cannot_be_optional(span));
        }
        // The span is not extended to its type_annotation
        let type_annotation = self.parse_ts_type_annotation();
        let pattern = self.ast.binding_pattern(kind, type_annotation, false);

        // Rest element does not allow `= initializer`
        // function foo([...x = []]) { }
        //                    ^^^^ A rest element cannot have an initializer
        let argument = self.context_add(Context::In, |p| p.parse_initializer(init_span, pattern));
        if let BindingPatternKind::AssignmentPattern(pat) = &argument.kind {
            self.error(diagnostics::a_rest_element_cannot_have_an_initializer(pat.span));
        }

        self.ast.binding_rest_element(self.end_span(span), argument)
    }

    /// `BindingProperty`[Yield, Await] :
    ///     `SingleNameBinding`[?Yield, ?Await]
    ///     `PropertyName`[?Yield, ?Await] : `BindingElement`[?Yield, ?Await]
    pub(super) fn parse_binding_property(&mut self) -> BindingProperty<'a> {
        let span = self.start_span();

        let mut shorthand = false;
        let is_binding_identifier = self.cur_kind().is_binding_identifier();
        let key_cur_kind = self.cur_kind();
        let (key, computed) = self.parse_property_name();

        let value = if is_binding_identifier && !self.at(Kind::Colon) {
            // let { a = b } = c
            // let { a } = b
            //       ^ BindingIdentifier
            if let PropertyKey::StaticIdentifier(ident) = &key {
                shorthand = true;
                self.check_identifier_with_span(key_cur_kind, self.ctx, ident.span);
                let identifier =
                    self.ast.binding_pattern_kind_binding_identifier(ident.span, ident.name);
                let left = self.ast.binding_pattern(identifier, NONE, false);
                self.context_add(Context::In, |p| p.parse_initializer(span, left))
            } else {
                return self.unexpected();
            }
        } else {
            // let { a: b } = c
            //       ^ IdentifierReference
            self.expect(Kind::Colon);
            self.parse_binding_pattern_with_initializer()
        };

        self.ast.binding_property(self.end_span(span), key, value, shorthand, computed)
    }

    /// Initializer[In, Yield, Await] :
    ///   = `AssignmentExpression`[?In, ?Yield, ?Await]
    fn parse_initializer(&mut self, span: u32, left: BindingPattern<'a>) -> BindingPattern<'a> {
        if self.eat(Kind::Eq) {
            let expr = self.parse_assignment_expression_or_higher();
            self.ast.binding_pattern(
                self.ast.binding_pattern_kind_assignment_pattern(self.end_span(span), left, expr),
                NONE,
                false,
            )
        } else {
            left
        }
    }

    pub(super) fn extend_binding_pattern_span_end(end: u32, kind: &mut BindingPatternKind<'a>) {
        let pat_span = match kind {
            BindingPatternKind::BindingIdentifier(pat) => &mut pat.span,
            BindingPatternKind::ObjectPattern(pat) => &mut pat.span,
            BindingPatternKind::ArrayPattern(pat) => &mut pat.span,
            BindingPatternKind::AssignmentPattern(pat) => &mut pat.span,
        };
        pat_span.end = end;
    }
}
