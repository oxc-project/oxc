use oxc_allocator::ArenaBox;
use oxc_ast::ast::*;
use oxc_syntax::operator::AssignmentOperator;

use crate::{
    Context, ParserConfig as Config, ParserImpl, diagnostics,
    lexer::Kind,
    modifiers::{ModifierKind, ModifierKinds, Modifiers},
};

use super::FunctionKind;

impl<'a, C: Config> ParserImpl<'a, C> {
    /// [Object Expression](https://tc39.es/ecma262/#sec-object-initializer)
    /// `ObjectLiteral`[Yield, Await] :
    ///     { }
    ///     { `PropertyDefinitionList`[?Yield, ?Await] }
    ///     { `PropertyDefinitionList`[?Yield, ?Await] , }
    pub(crate) fn parse_object_expression(&mut self) -> ArenaBox<'a, ObjectExpression<'a>> {
        let start = self.cur_start();
        let opening_span = self.cur_token().span();
        self.expect(Kind::LCurly);
        let object = ObjectExpression::build(self).span_start(start);
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
        object.properties(object_expression_properties).span_end(self.end_span(start).end).finish()
    }

    fn parse_object_expression_property(&mut self) -> ObjectPropertyKind<'a> {
        match self.cur_kind() {
            Kind::Dot3 => ObjectPropertyKind::SpreadProperty(self.parse_spread_element()),
            _ => ObjectPropertyKind::ObjectProperty(self.parse_object_literal_element()),
        }
    }

    /// `PropertyDefinition`[Yield, Await]
    fn parse_object_literal_element(&mut self) -> ArenaBox<'a, ObjectProperty<'a>> {
        let start = self.cur_start();

        let modifiers = self.parse_modifiers(
            /* permit_const_as_modifier */ false,
            /* stop_on_start_of_class_static_block */ false,
        );

        if self.parse_contextual_modifier(Kind::Get) {
            return self.parse_method_getter_setter(start, PropertyKind::Get, &modifiers);
        }

        if self.parse_contextual_modifier(Kind::Set) {
            return self.parse_method_getter_setter(start, PropertyKind::Set, &modifiers);
        }

        let asterisk_token = self.eat(Kind::Star).then_some(self.prev_token_end - 1);
        let token_is_identifier =
            self.cur_kind().is_identifier_reference(self.ctx.has_yield(), self.ctx.has_await());
        let (key, computed) = self.parse_property_name();

        if asterisk_token.is_some() || matches!(self.cur_kind(), Kind::LParen | Kind::LAngle) {
            self.verify_modifiers(
                &modifiers,
                ModifierKinds::new([ModifierKind::Async]),
                true,
                diagnostics::modifier_cannot_be_used_here,
            );
            let property = ObjectProperty::build(self)
                .span_start(start)
                .kind(PropertyKind::Init)
                .key(key)
                .method(true)
                .shorthand(false)
                .computed(computed);
            let method = self.parse_method(
                modifiers.contains_async(),
                asterisk_token,
                FunctionKind::ObjectMethod,
            );
            return property
                .value(Expression::FunctionExpression(method))
                .span_end(self.end_span(start).end)
                .finish();
        }

        self.verify_modifiers(
            &modifiers,
            ModifierKinds::none(),
            true,
            diagnostics::modifier_cannot_be_used_here,
        );

        let is_shorthand_property_assignment = token_is_identifier && !self.at(Kind::Colon);

        if is_shorthand_property_assignment {
            if let PropertyKey::StaticIdentifier(identifier_name) = key {
                let property = ObjectProperty::build(self)
                    .span_start(start)
                    .kind(PropertyKind::Init)
                    .method(false)
                    .shorthand(true)
                    .computed(computed);
                // CoverInitializedName ({ foo = bar })
                if self.eat(Kind::Eq) {
                    let right = self.parse_assignment_expression_or_higher();
                    let left = AssignmentTarget::AssignmentTargetIdentifier(
                        IdentifierReference::build(self)
                            .span(identifier_name.span)
                            .name(identifier_name.name)
                            .defaults()
                            .finish(),
                    );
                    let expr = AssignmentExpression::new(
                        self.end_span(start),
                        AssignmentOperator::Assign,
                        left,
                        right,
                        self,
                    );
                    self.state.cover_initialized_name.insert(start, expr);
                }
                let value = Expression::Identifier(
                    IdentifierReference::build(self)
                        .span(identifier_name.span)
                        .name(identifier_name.name)
                        .defaults()
                        .finish(),
                );
                property
                    .key(PropertyKey::StaticIdentifier(identifier_name))
                    .value(value)
                    .span_end(self.end_span(start).end)
                    .finish()
            } else {
                self.unexpected()
            }
        } else {
            self.parse_property_definition_assignment(start, key, computed)
        }
    }

    /// `PropertyDefinition`[Yield, Await] :
    ///   ... `AssignmentExpression`[+In, ?Yield, ?Await]
    pub(crate) fn parse_spread_element(&mut self) -> ArenaBox<'a, SpreadElement<'a>> {
        let start = self.cur_start();
        self.bump_any(); // advance `...`
        let spread = SpreadElement::build(self).span_start(start);
        let argument = self.parse_assignment_expression_or_higher();
        spread.argument(argument).span_end(self.end_span(start).end).finish()
    }

    /// `PropertyDefinition`[Yield, Await] :
    ///   `PropertyName`[?Yield, ?Await] : `AssignmentExpression`[+In, ?Yield, ?Await]
    fn parse_property_definition_assignment(
        &mut self,
        start: u32,
        key: PropertyKey<'a>,
        computed: bool,
    ) -> ArenaBox<'a, ObjectProperty<'a>> {
        self.expect(Kind::Colon);
        let property = ObjectProperty::build(self)
            .span_start(start)
            .kind(PropertyKind::Init)
            .key(key)
            .method(false)
            .shorthand(false)
            .computed(computed);
        let value = self.parse_assignment_expression_or_higher();
        property.value(value).span_end(self.end_span(start).end).finish()
    }

    /// `PropertyName`[Yield, Await] :
    ///    `LiteralPropertyName`
    ///    `ComputedPropertyName`[?Yield, ?Await]
    pub(crate) fn parse_property_name(&mut self) -> (PropertyKey<'a>, bool) {
        let mut computed = false;
        let key = match self.cur_kind() {
            Kind::Str => PropertyKey::from(self.parse_literal_expression()),
            kind if kind.is_number() => PropertyKey::from(self.parse_literal_expression()),
            // { [foo]() {} }
            Kind::LBrack => {
                computed = true;
                PropertyKey::from(self.parse_computed_property_name())
            }
            Kind::PrivateIdentifier => {
                let private_ident = self.parse_private_identifier();
                self.error(diagnostics::private_identifier_in_property_name(
                    private_ident.name.as_str(),
                    private_ident.span,
                ));
                PropertyKey::PrivateIdentifier(
                    PrivateIdentifier::build(self)
                        .span(private_ident.span)
                        .name(private_ident.name)
                        .finish(),
                )
            }
            _ => {
                let ident = self.parse_identifier_name();
                PropertyKey::StaticIdentifier(
                    IdentifierName::build(self).span(ident.span).name(ident.name).finish(),
                )
            }
        };
        (key, computed)
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
        kind: PropertyKind,
        modifiers: &Modifiers,
    ) -> ArenaBox<'a, ObjectProperty<'a>> {
        let (key, computed) = self.parse_property_name();
        let property = ObjectProperty::build(self)
            .span_start(start)
            .kind(kind)
            .key(key)
            .method(false)
            .shorthand(false)
            .computed(computed);
        let function = self.parse_method(false, None, FunctionKind::ObjectMethod);
        match kind {
            PropertyKind::Get => self.check_getter(&function),
            PropertyKind::Set => self.check_setter(&function),
            PropertyKind::Init => {}
        }
        self.verify_modifiers(
            modifiers,
            ModifierKinds::none(),
            true,
            diagnostics::modifier_cannot_be_used_here,
        );
        property
            .value(Expression::FunctionExpression(function))
            .span_end(self.end_span(start).end)
            .finish()
    }
}
