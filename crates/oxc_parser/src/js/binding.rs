use oxc_allocator::Box;
use oxc_ast::ast::*;
use oxc_diagnostics::Result;
use oxc_span::Span;

use super::list::{ArrayPatternList, ObjectPatternProperties};
use crate::{diagnostics, lexer::Kind, list::SeparatedList, Context, ParserImpl};

impl<'a> ParserImpl<'a> {
    /// `BindingElement`
    ///     `SingleNameBinding`
    ///     `BindingPattern`[?Yield, ?Await] `Initializer`[+In, ?Yield, ?Await]opt
    pub(super) fn parse_binding_pattern_with_initializer(&mut self) -> Result<BindingPattern<'a>> {
        let span = self.start_span();
        let pattern = self.parse_binding_pattern(true)?;
        self.context(Context::In, Context::empty(), |p| p.parse_initializer(span, pattern))
    }

    pub(super) fn parse_binding_pattern(
        &mut self,
        allow_question: bool,
    ) -> Result<BindingPattern<'a>> {
        let mut kind = self.parse_binding_pattern_kind()?;
        let optional =
            if allow_question && self.ts_enabled() { self.eat(Kind::Question) } else { false };
        let type_annotation = self.parse_ts_type_annotation()?;
        if let Some(type_annotation) = &type_annotation {
            Self::extend_binding_pattern_span_end(type_annotation.span, &mut kind);
        }
        Ok(self.ast.binding_pattern(kind, type_annotation, optional))
    }

    pub(crate) fn parse_binding_pattern_kind(&mut self) -> Result<BindingPatternKind<'a>> {
        match self.cur_kind() {
            Kind::LCurly => self.parse_object_binding_pattern(),
            Kind::LBrack => self.parse_array_binding_pattern(),
            _ => self.parse_binding_pattern_identifier(),
        }
    }

    fn parse_binding_pattern_identifier(&mut self) -> Result<BindingPatternKind<'a>> {
        let ident = self.parse_binding_identifier()?;
        Ok(self.ast.binding_pattern_identifier(ident))
    }

    /// Section 14.3.3 Object Binding Pattern
    fn parse_object_binding_pattern(&mut self) -> Result<BindingPatternKind<'a>> {
        let span = self.start_span();
        let props = ObjectPatternProperties::parse(self)?;
        Ok(self.ast.object_pattern(self.end_span(span), props.elements, props.rest))
    }

    /// Section 14.3.3 Array Binding Pattern
    fn parse_array_binding_pattern(&mut self) -> Result<BindingPatternKind<'a>> {
        let span = self.start_span();
        let list = ArrayPatternList::parse(self)?;
        Ok(self.ast.array_pattern(self.end_span(span), list.elements, list.rest))
    }

    /// Section 14.3.3 Binding Rest Property
    pub(super) fn parse_rest_element(&mut self) -> Result<Box<'a, BindingRestElement<'a>>> {
        let span = self.start_span();
        self.bump_any(); // advance `...`
        let init_span = self.start_span();

        let kind = self.parse_binding_pattern_kind()?;
        // Rest element does not allow `?`, checked in checker/typescript.rs
        if self.at(Kind::Question) && self.ts_enabled() {
            let span = self.cur_token().span();
            self.bump_any();
            self.error(diagnostics::a_rest_parameter_cannot_be_optional(span));
        }
        // The span is not extended to its type_annotation
        let type_annotation = self.parse_ts_type_annotation()?;
        let pattern = self.ast.binding_pattern(kind, type_annotation, false);
        // Rest element does not allow `= initializer`, .
        let argument = self
            .context(Context::In, Context::empty(), |p| p.parse_initializer(init_span, pattern))?;
        let span = self.end_span(span);

        if self.at(Kind::Comma) {
            if self.peek_at(Kind::RBrack) {
                self.error(diagnostics::binding_rest_element_trailing_comma(
                    self.cur_token().span(),
                ));
            } else if !self.ctx.has_ambient() {
                self.error(diagnostics::binding_rest_element_last(span));
            }
        }

        Ok(self.ast.rest_element(span, argument))
    }

    /// `BindingProperty`[Yield, Await] :
    ///     `SingleNameBinding`[?Yield, ?Await]
    ///     `PropertyName`[?Yield, ?Await] : `BindingElement`[?Yield, ?Await]
    pub(super) fn parse_binding_property(&mut self) -> Result<BindingProperty<'a>> {
        let span = self.start_span();

        let mut shorthand = false;
        let is_binding_identifier = self.cur_kind().is_binding_identifier();
        let (key, computed) = self.parse_property_name()?;

        let value = if is_binding_identifier && !self.at(Kind::Colon) {
            // let { a = b } = c
            // let { a } = b
            //       ^ BindingIdentifier
            if let PropertyKey::StaticIdentifier(ident) = &key {
                shorthand = true;
                let binding_identifier = BindingIdentifier::new(ident.span, ident.name.clone());
                let identifier = self.ast.binding_pattern_identifier(binding_identifier);
                let left = self.ast.binding_pattern(identifier, None, false);
                self.context(Context::In, Context::empty(), |p| p.parse_initializer(span, left))?
            } else {
                return Err(self.unexpected());
            }
        } else {
            // let { a: b } = c
            //       ^ IdentifierReference
            self.expect(Kind::Colon)?;
            self.parse_binding_pattern_with_initializer()?
        };

        Ok(self.ast.binding_property(self.end_span(span), key, value, shorthand, computed))
    }

    /// Initializer[In, Yield, Await] :
    ///   = `AssignmentExpression`[?In, ?Yield, ?Await]
    fn parse_initializer(
        &mut self,
        span: Span,
        left: BindingPattern<'a>,
    ) -> Result<BindingPattern<'a>> {
        if self.eat(Kind::Eq) {
            let expr = self.parse_assignment_expression_or_higher()?;
            Ok(self.ast.assignment_pattern(self.end_span(span), left, expr))
        } else {
            Ok(left)
        }
    }

    pub(super) fn extend_binding_pattern_span_end(span: Span, kind: &mut BindingPatternKind<'a>) {
        let pat_span = match kind {
            BindingPatternKind::BindingIdentifier(pat) => &mut pat.span,
            BindingPatternKind::ObjectPattern(pat) => &mut pat.span,
            BindingPatternKind::ArrayPattern(pat) => &mut pat.span,
            BindingPatternKind::AssignmentPattern(pat) => &mut pat.span,
        };
        pat_span.end = span.end;
    }
}
