use oxc_allocator::ArenaBox;
use oxc_ast::ast::*;

use crate::{
    Context, ParserConfig as Config, ParserImpl, diagnostics,
    lexer::Kind,
    modifiers::{ModifierKind, ModifierKinds, Modifiers},
    state::CoverInitializedName,
};

use super::FunctionKind;

impl<'a, C: Config> ParserImpl<'a, C> {
    /// [Object Expression](https://tc39.es/ecma262/#sec-object-initializer)
    /// `ObjectLiteral`[Yield, Await] :
    ///     { }
    ///     { `PropertyDefinitionList`[?Yield, ?Await] }
    ///     { `PropertyDefinitionList`[?Yield, ?Await] , }
    pub(crate) fn parse_object_expression(&mut self) -> ArenaBox<'a, ObjectExpression<'a>> {
        debug_assert!(self.at(Kind::LCurly));
        let start = self.cur_start();
        let opening_span = self.cur_token().span();
        self.advance(Kind::LCurly);
        let (object_expression_properties, comma_start) = self.context_add(Context::In, |p| {
            p.parse_delimited_list(
                Kind::RCurly,
                Kind::Comma,
                opening_span,
                Self::parse_object_expression_property,
            )
        });
        if let Some(comma_start) = comma_start
            && matches!(
                object_expression_properties.last(),
                Some(ObjectPropertyKind::SpreadProperty(_))
            )
        {
            self.state.trailing_commas.insert(start, self.end_span(comma_start));
        }
        self.expect(Kind::RCurly);
        ObjectExpression::boxed(self.end_span(start), object_expression_properties, self)
    }

    fn parse_object_expression_property(&mut self) -> ObjectPropertyKind<'a> {
        match self.cur_kind() {
            Kind::Ident => self.parse_object_literal_element_with_identifier_key(),
            Kind::Dot3 => ObjectPropertyKind::SpreadProperty(self.parse_spread_element()),
            _ => self.parse_object_literal_element(),
        }
    }

    /// Parse a property whose key is an identifier that is not a keyword in any context,
    /// so it is not a modifier, `get` or `set`: `a`, `a: b`, `a = b` or `a() {}`.
    fn parse_object_literal_element_with_identifier_key(&mut self) -> ObjectPropertyKind<'a> {
        let start = self.cur_start();
        let (span, name) = self.parse_identifier_kind(Kind::Ident);
        let key = PropertyKey::StaticIdentifier(IdentifierName::boxed(span, name, self));

        let kind = self.cur_kind();
        let (value, shorthand) = if kind == Kind::Colon {
            self.bump_any(); // `:`
            (self.parse_assignment_expression_or_higher(), false)
        } else if matches!(kind, Kind::LParen | Kind::LAngle) {
            let method =
                self.parse_method(/* async */ false, None, FunctionKind::ObjectMethod);
            return ObjectPropertyKind::MethodProperty(ObjectMethodProperty::boxed(
                self.end_span(start),
                ObjectMethodKind::Method,
                key,
                method,
                self,
            ));
        } else {
            // CoverInitializedName ({ foo = bar })
            if kind == Kind::Eq {
                self.bump_any(); // `=`
                let initializer = self.parse_assignment_expression_or_higher();
                let cover = CoverInitializedName { span: self.end_span(start), initializer };
                self.state.cover_initialized_name.insert(start, cover);
            }
            (Expression::new_identifier(span, name, self), true)
        };
        ObjectPropertyKind::ObjectProperty(ObjectProperty::boxed(
            self.end_span(start),
            key,
            value,
            shorthand,
            self,
        ))
    }

    /// `PropertyDefinition`[Yield, Await]
    fn parse_object_literal_element(&mut self) -> ObjectPropertyKind<'a> {
        let start = self.cur_start();

        let modifiers = self.parse_modifiers(
            /* permit_const_as_modifier */ false,
            /* stop_on_start_of_class_static_block */ false,
        );

        if self.parse_contextual_modifier(Kind::Get) {
            return self.parse_method_getter_setter(start, ObjectMethodKind::Get, &modifiers);
        }

        if self.parse_contextual_modifier(Kind::Set) {
            return self.parse_method_getter_setter(start, ObjectMethodKind::Set, &modifiers);
        }

        let asterisk_token = self.eat(Kind::Star).then_some(self.prev_token_end - 1);
        let key_kind = self.cur_kind();
        let token_is_identifier = key_kind
            .is_identifier_reference(self.ctx.has_yield(), self.ctx.has_await())
            // In a list that may be the parameters of an arrow function, `({ await }) => {}`,
            // a reserved `await` or `yield` is reported as the binding it would be.
            || (matches!(key_kind, Kind::Await | Kind::Yield) && self.state.cover_depth > 0);
        let key = self.parse_property_name();

        if asterisk_token.is_some() || matches!(self.cur_kind(), Kind::LParen | Kind::LAngle) {
            self.verify_modifiers(
                &modifiers,
                ModifierKinds::new([ModifierKind::Async]),
                true,
                diagnostics::modifier_cannot_be_used_here,
            );
            let method = self.parse_method(
                modifiers.contains_async(),
                asterisk_token,
                FunctionKind::ObjectMethod,
            );
            return ObjectPropertyKind::MethodProperty(ObjectMethodProperty::boxed(
                self.end_span(start),
                ObjectMethodKind::Method,
                key,
                method,
                self,
            ));
        }

        self.verify_modifiers(
            &modifiers,
            ModifierKinds::none(),
            true,
            diagnostics::modifier_cannot_be_used_here,
        );

        let shorthand = token_is_identifier && !self.at(Kind::Colon);
        let value = if shorthand {
            let PropertyKey::StaticIdentifier(identifier_name) = &key else {
                return self.unexpected();
            };
            if matches!(key_kind, Kind::Await | Kind::Yield) {
                self.check_identifier_with_span(key_kind, self.ctx, identifier_name.span);
            }
            // CoverInitializedName ({ foo = bar })
            if self.eat(Kind::Eq) {
                let initializer = self.parse_assignment_expression_or_higher();
                let cover = CoverInitializedName { span: self.end_span(start), initializer };
                self.state.cover_initialized_name.insert(start, cover);
            }
            Expression::new_identifier(identifier_name.span, identifier_name.name, self)
        } else {
            self.expect(Kind::Colon);
            self.parse_assignment_expression_or_higher()
        };
        ObjectPropertyKind::ObjectProperty(ObjectProperty::boxed(
            self.end_span(start),
            key,
            value,
            shorthand,
            self,
        ))
    }

    /// `PropertyDefinition`[Yield, Await] :
    ///   ... `AssignmentExpression`[+In, ?Yield, ?Await]
    pub(crate) fn parse_spread_element(&mut self) -> ArenaBox<'a, SpreadElement<'a>> {
        let start = self.cur_start();
        self.bump_any(); // advance `...`
        let argument = self.parse_assignment_expression_or_higher();
        SpreadElement::boxed(self.end_span(start), argument, self)
    }

    /// `PropertyName`[Yield, Await] :
    ///    `LiteralPropertyName`
    ///    `ComputedPropertyName`[?Yield, ?Await]
    pub(crate) fn parse_property_name(&mut self) -> PropertyKey<'a> {
        match self.cur_kind() {
            Kind::Ident => {
                let ident = self.parse_known_identifier_name();
                PropertyKey::StaticIdentifier(self.alloc(ident))
            }
            Kind::Str => {
                let literal = self.parse_known_literal_string();
                PropertyKey::StaticStringLiteral(self.alloc(literal))
            }
            Kind::DecimalBigInt | Kind::BinaryBigInt | Kind::OctalBigInt | Kind::HexBigInt => {
                PropertyKey::StaticBigIntLiteral(self.parse_literal_bigint())
            }
            kind if kind.is_number() => {
                PropertyKey::StaticNumericLiteral(self.parse_literal_number())
            }
            // { [foo]() {} }
            Kind::LBrack => PropertyKey::from(self.parse_computed_property_name()),
            Kind::PrivateIdentifier => {
                let private_ident = self.parse_private_identifier();
                self.error(diagnostics::private_identifier_in_property_name(
                    private_ident.name.as_str(),
                    private_ident.span,
                ));
                PropertyKey::PrivateIdentifier(self.alloc(private_ident))
            }
            _ => {
                let ident = self.parse_identifier_name();
                PropertyKey::StaticIdentifier(self.alloc(ident))
            }
        }
    }

    /// `ComputedPropertyName`[Yield, Await] : [ `AssignmentExpression`[+In, ?Yield, ?Await] ]
    pub(crate) fn parse_computed_property_name(&mut self) -> Expression<'a> {
        self.bump_any(); // advance `[`

        let expression = self.context_add(Context::In, Self::parse_assignment_expression_or_higher);

        self.expect(Kind::RBrack);
        expression
    }

    /// `MethodDefinition`[Yield, Await] :
    ///   get `ClassElementName`[?Yield, ?Await] ( ) { `FunctionBody`[~Yield, ~Await] }
    ///   set `ClassElementName`[?Yield, ?Await] ( `PropertySetParameterList` ) { `FunctionBody`[~Yield, ~Await] }
    fn parse_method_getter_setter(
        &mut self,
        start: u32,
        kind: ObjectMethodKind,
        modifiers: &Modifiers,
    ) -> ObjectPropertyKind<'a> {
        let key = self.parse_property_name();
        let function = self.parse_method(false, None, FunctionKind::ObjectMethod);
        match kind {
            ObjectMethodKind::Get => self.check_getter(&function),
            ObjectMethodKind::Set => self.check_setter(&function),
            ObjectMethodKind::Method => {}
        }
        self.verify_modifiers(
            modifiers,
            ModifierKinds::none(),
            true,
            diagnostics::modifier_cannot_be_used_here,
        );
        ObjectPropertyKind::MethodProperty(ObjectMethodProperty::boxed(
            self.end_span(start),
            kind,
            key,
            function,
            self,
        ))
    }
}
