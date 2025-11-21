use oxc_allocator::Box;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{Context, ParserImpl, diagnostics, lexer::Kind};

impl<'a> ParserImpl<'a> {
    /// `BindingElement`
    ///     `SingleNameBinding`
    ///     `BindingPattern`[?Yield, ?Await] `Initializer`[+In, ?Yield, ?Await]opt
    pub(super) fn parse_binding_pattern_with_initializer(&mut self) -> BindingPattern<'a> {
        let span = self.start_span();
        let pattern = self.parse_binding_pattern();
        self.context_add(Context::In, |p| p.parse_initializer(span, pattern))
    }

    pub(super) fn parse_binding_pattern(&mut self) -> BindingPattern<'a> {
        self.parse_binding_pattern_kind()
    }

    pub(super) fn parse_binding_pattern_with_type_annotation(
        &mut self,
    ) -> (BindingPattern<'a>, Option<Box<'a, TSTypeAnnotation<'a>>>) {
        let pattern = self.parse_binding_pattern_kind();
        let type_annotation = self.parse_ts_type_annotation();
        (pattern, type_annotation)
    }

    pub(crate) fn parse_binding_pattern_kind(&mut self) -> BindingPattern<'a> {
        match self.cur_kind() {
            Kind::LCurly => self.parse_object_binding_pattern(),
            Kind::LBrack => self.parse_array_binding_pattern(),
            _ => self.parse_binding_pattern_identifier(),
        }
    }

    fn parse_binding_pattern_identifier(&mut self) -> BindingPattern<'a> {
        let ident = self.parse_binding_identifier();
        BindingPattern::BindingIdentifier(self.alloc(ident))
    }

    /// Section 14.3.3 Object Binding Pattern
    fn parse_object_binding_pattern(&mut self) -> BindingPattern<'a> {
        let span = self.start_span();
        let opening_span = self.cur_token().span();
        self.expect(Kind::LCurly);
        let (list, rest) = self.parse_delimited_list_with_rest(
            Kind::RCurly,
            opening_span,
            Self::parse_binding_property,
            Self::parse_rest_element,
            diagnostics::binding_rest_element_last,
        );
        if let Some(rest) = &rest
            && !matches!(&rest.argument, BindingPattern::BindingIdentifier(_))
        {
            let error = diagnostics::invalid_binding_rest_element(rest.argument.span());
            return self.fatal_error(error);
        }

        self.expect(Kind::RCurly);
        self.ast.binding_pattern_object_pattern(
            self.end_span(span),
            list,
            rest.map(|r| self.alloc(r)),
        )
    }

    /// Section 14.3.3 Array Binding Pattern
    fn parse_array_binding_pattern(&mut self) -> BindingPattern<'a> {
        let span = self.start_span();
        let opening_span = self.cur_token().span();
        self.expect(Kind::LBrack);
        let (list, rest) = self.parse_delimited_list_with_rest(
            Kind::RBrack,
            opening_span,
            Self::parse_array_binding_element,
            Self::parse_rest_element,
            diagnostics::binding_rest_element_last,
        );
        self.expect(Kind::RBrack);
        self.ast.binding_pattern_array_pattern(
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

    /// Section 14.3.3 Binding Rest Property (for binding patterns - errors on type annotation)
    pub(crate) fn parse_rest_element(&mut self) -> BindingRestElement<'a> {
        let span = self.start_span();
        self.bump_any(); // advance `...`
        let init_span = self.start_span();

        let pattern = self.parse_binding_pattern_kind();
        // Rest element does not allow `?`, checked in checker/typescript.rs
        if self.at(Kind::Question) && self.is_ts {
            let span = self.cur_token().span();
            self.bump_any();
            self.error(diagnostics::a_rest_parameter_cannot_be_optional(span));
        }
        // In binding patterns, type annotation syntax is invalid (looks like property name)
        // Example: const { ...a: b } = {}; // Error: rest element cannot have property name
        if self.is_ts && self.at(Kind::Colon) {
            let type_annotation = self.parse_ts_type_annotation();
            if let Some(ty) = type_annotation {
                self.error(diagnostics::rest_element_property_name(ty.span));
            }
        }

        // Rest element does not allow `= initializer`
        // function foo([...x = []]) { }
        //                    ^^^^ A rest element cannot have an initializer
        let argument = self.context_add(Context::In, |p| p.parse_initializer(init_span, pattern));
        if let BindingPattern::AssignmentPattern(pat) = &argument {
            self.error(diagnostics::a_rest_element_cannot_have_an_initializer(pat.span));
        }

        self.ast.binding_rest_element(self.end_span(span), argument)
    }

    /// Parse rest element for function parameters (type annotation NOT consumed)
    /// The type annotation will be parsed by the caller and stored on FormalParameterRest
    /// We don't consume it here so the caller can access it
    pub(crate) fn parse_rest_element_for_formal_parameter(&mut self) -> BindingRestElement<'a> {
        let span = self.start_span();
        self.bump_any(); // advance `...`
        let init_span = self.start_span();

        let pattern = self.parse_binding_pattern_kind();
        // Rest element does not allow `?`, checked in checker/typescript.rs
        if self.at(Kind::Question) && self.is_ts {
            let span = self.cur_token().span();
            self.bump_any();
            self.error(diagnostics::a_rest_parameter_cannot_be_optional(span));
        }
        // For formal parameters, type annotation is NOT consumed here
        // It will be parsed by the caller (parse_formal_parameters_list) and attached to FormalParameterRest

        // Rest element does not allow `= initializer`
        // function foo([...x = []]) { }
        //                    ^^^^ A rest element cannot have an initializer
        let argument = self.context_add(Context::In, |p| p.parse_initializer(init_span, pattern));
        if let BindingPattern::AssignmentPattern(pat) = &argument {
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
                    self.ast.binding_pattern_binding_identifier(ident.span, ident.name);
                self.context_add(Context::In, |p| p.parse_initializer(span, identifier))
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
            self.ast.binding_pattern_assignment_pattern(self.end_span(span), left, expr)
        } else {
            left
        }
    }
}
