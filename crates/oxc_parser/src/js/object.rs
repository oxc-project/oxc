use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_str::Ident;
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
        let span = self.start_span();
        let opening_span = self.cur_token().span();
        self.expect(Kind::LCurly);

        // Check if this is an ArkUI object literal with expression statements (e.g., { .backgroundColor(...) })
        // In ArkUI, object literals can contain expression statements starting with dots
        if self.source_type.is_arkui() && self.is_in_arkui_dsl_context() && self.at(Kind::Dot) {
            // Parse as ArkUI object literal with expression statements
            return self.parse_arkui_object_expression_with_statements(span, opening_span);
        }

        let (object_expression_properties, comma_span) = self.context_add(Context::In, |p| {
            p.parse_delimited_list(
                Kind::RCurly,
                Kind::Comma,
                opening_span,
                Self::parse_object_expression_property,
            )
        });
        if let Some(comma_span) = comma_span {
            self.state.trailing_commas.insert(span, self.end_span(comma_span));
        }
        self.expect(Kind::RCurly);
        ObjectExpression::boxed(self.end_span(span), object_expression_properties, self)
    }

    /// Parse ArkUI object literal with expression statements
    /// Example: { .backgroundColor('#ffffeef0') }
    /// In ArkUI, object literals can contain expression statements starting with dots
    fn parse_arkui_object_expression_with_statements(
        &mut self,
        span: u32,
        _opening_span: Span,
    ) -> ArenaBox<'a, ObjectExpression<'a>> {
        use crate::lexer::Kind;
        let mut properties = ArenaVec::new_in(self);

        // Parse expression statements until closing brace
        while !self.at(Kind::RCurly) && !self.has_fatal_error() {
            if self.at(Kind::Dot) {
                // Parse expression statement starting with dot as LeadingDotExpression
                let expr_span = self.start_span();
                let expr = self.parse_leading_dot_expression();
                let expr_end_span = self.end_span(expr_span);

                // Create a property with the expression as value
                // Use a synthetic key for the expression statement
                let key = PropertyKey::StaticIdentifier(IdentifierName::boxed(
                    expr_end_span,
                    Ident::from(""),
                    self,
                ));
                let property = ObjectProperty::boxed(
                    expr_end_span,
                    PropertyKind::Init,
                    key,
                    expr,
                    false, // not a method
                    false, // not shorthand
                    false, // not computed
                    self,
                );
                properties.push(ObjectPropertyKind::ObjectProperty(property));

                // Optional semicolon or comma (both are allowed after expression statements)
                if self.eat(Kind::Semicolon) {
                    // Semicolon consumed
                } else if self.at(Kind::Comma) {
                    // Comma after expression statement (e.g., .method1().method2(),)
                    self.expect(Kind::Comma);
                }
                // If neither semicolon nor comma, continue to next token (could be closing brace or next property)
            } else {
                // Parse as normal object property
                let property = self.parse_object_expression_property();
                properties.push(property);

                // Expect comma or closing brace
                if !self.at(Kind::RCurly) {
                    self.expect(Kind::Comma);
                }
            }
        }

        self.expect(Kind::RCurly);
        ObjectExpression::boxed(self.end_span(span), properties, self)
    }

    fn parse_object_expression_property(&mut self) -> ObjectPropertyKind<'a> {
        match self.cur_kind() {
            Kind::Dot3 => ObjectPropertyKind::SpreadProperty(self.parse_spread_element()),
            _ => ObjectPropertyKind::ObjectProperty(self.parse_object_literal_element()),
        }
    }

    /// `PropertyDefinition`[Yield, Await]
    fn parse_object_literal_element(&mut self) -> ArenaBox<'a, ObjectProperty<'a>> {
        let span = self.start_span();

        let modifiers = self.parse_modifiers(
            /* permit_const_as_modifier */ false,
            /* stop_on_start_of_class_static_block */ false,
        );

        if self.parse_contextual_modifier(Kind::Get) {
            if self.source_type.is_ets_static() {
                self.error(diagnostics::ets_unsupported_syntax(
                    "Methods in object literals",
                    self.cur_token().span(),
                ));
            }
            return self.parse_method_getter_setter(span, PropertyKind::Get, &modifiers);
        }

        if self.parse_contextual_modifier(Kind::Set) {
            if self.source_type.is_ets_static() {
                self.error(diagnostics::ets_unsupported_syntax(
                    "Methods in object literals",
                    self.cur_token().span(),
                ));
            }
            return self.parse_method_getter_setter(span, PropertyKind::Set, &modifiers);
        }

        let asterisk_token = self.eat(Kind::Star);
        let token_is_identifier =
            self.cur_kind().is_identifier_reference(self.ctx.has_yield(), self.ctx.has_await());
        let (key, computed) = self.parse_property_name();

        if asterisk_token || matches!(self.cur_kind(), Kind::LParen | Kind::LAngle) {
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
            return ObjectProperty::boxed(
                self.end_span(span),
                PropertyKind::Init,
                key,
                Expression::FunctionExpression(method),
                /* method */ true,
                /* shorthand */ false,
                computed,
                self,
            );
        }

        self.verify_modifiers(
            &modifiers,
            ModifierKinds::none(),
            true,
            diagnostics::modifier_cannot_be_used_here,
        );

        if self.source_type.is_ets_static() && computed {
            self.error(diagnostics::ets_unsupported_syntax(
                "Computed properties in object literals",
                key.span(),
            ));
        }

        if self.source_type.is_ets_static() && token_is_identifier && self.eat(Kind::Eq) {
            let value = self.parse_assignment_expression_or_higher();
            return ObjectProperty::boxed(
                self.end_span(span),
                PropertyKind::EtsEquals,
                key,
                value,
                /* method */ false,
                /* shorthand */ false,
                computed,
                self,
            );
        }

        let is_shorthand_property_assignment = token_is_identifier && !self.at(Kind::Colon);

        if is_shorthand_property_assignment {
            if let PropertyKey::StaticIdentifier(identifier_name) = key {
                // CoverInitializedName ({ foo = bar })
                if self.eat(Kind::Eq) {
                    let right = self.parse_assignment_expression_or_higher();
                    let left = AssignmentTarget::new_assignment_target_identifier(
                        identifier_name.span,
                        identifier_name.name,
                        self,
                    );
                    let expr = AssignmentExpression::new(
                        self.end_span(span),
                        AssignmentOperator::Assign,
                        left,
                        right,
                        self,
                    );
                    self.state.cover_initialized_name.insert(span, expr);
                }
                let value =
                    Expression::new_identifier(identifier_name.span, identifier_name.name, self);
                ObjectProperty::boxed(
                    self.end_span(span),
                    PropertyKind::Init,
                    PropertyKey::StaticIdentifier(identifier_name),
                    value,
                    /* method */ false,
                    /* shorthand */ true,
                    computed,
                    self,
                )
            } else {
                self.unexpected()
            }
        } else {
            self.parse_property_definition_assignment(span, key, computed)
        }
    }

    /// `PropertyDefinition`[Yield, Await] :
    ///   ... `AssignmentExpression`[+In, ?Yield, ?Await]
    pub(crate) fn parse_spread_element(&mut self) -> ArenaBox<'a, SpreadElement<'a>> {
        let span = self.start_span();
        self.bump_any(); // advance `...`
        let argument = self.parse_assignment_expression_or_higher();
        SpreadElement::boxed(self.end_span(span), argument, self)
    }

    /// `PropertyDefinition`[Yield, Await] :
    ///   `PropertyName`[?Yield, ?Await] : `AssignmentExpression`[+In, ?Yield, ?Await]
    fn parse_property_definition_assignment(
        &mut self,
        span: u32,
        key: PropertyKey<'a>,
        computed: bool,
    ) -> ArenaBox<'a, ObjectProperty<'a>> {
        self.expect(Kind::Colon);
        let value = if self.source_type.is_arkui()
            && self.is_in_arkui_dsl_context()
            && self.at(Kind::LCurly)
        {
            let obj_span = self.start_span();
            let opening_span = self.cur_token().span();
            self.expect(Kind::LCurly);

            let object = if self.at(Kind::Dot) {
                self.parse_arkui_object_expression_with_statements(obj_span, opening_span)
            } else {
                let (properties, comma_span) = self.context_add(Context::In, |p| {
                    p.parse_delimited_list(
                        Kind::RCurly,
                        Kind::Comma,
                        opening_span,
                        Self::parse_object_expression_property,
                    )
                });
                if let Some(comma_span) = comma_span {
                    self.state.trailing_commas.insert(obj_span, self.end_span(comma_span));
                }
                self.expect(Kind::RCurly);
                ObjectExpression::boxed(self.end_span(obj_span), properties, self)
            };

            self.parse_type_assertion_if_present(Expression::ObjectExpression(object))
        } else {
            self.parse_assignment_expression_or_higher()
        };
        ObjectProperty::boxed(
            self.end_span(span),
            PropertyKind::Init,
            key,
            value,
            /* method */ false,
            /* shorthand */ false,
            /* computed */ computed,
            self,
        )
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
                    &private_ident.name,
                    private_ident.span,
                ));
                PropertyKey::PrivateIdentifier(self.alloc(private_ident))
            }
            _ => {
                let ident = self.parse_identifier_name();
                PropertyKey::StaticIdentifier(self.alloc(ident))
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

    /// Parse type assertion (as or satisfies) if present after an expression
    /// Returns the expression wrapped in type assertion if found, otherwise returns the original expression
    fn parse_type_assertion_if_present(&mut self, expr: Expression<'a>) -> Expression<'a> {
        let kind = self.cur_kind();
        if matches!(kind, Kind::As | Kind::Satisfies) {
            if self.cur_token().is_on_new_line() {
                expr
            } else {
                let lhs_span = self.start_span();
                self.bump_any();
                let type_annotation = self.parse_ts_type();
                let span = self.end_span(lhs_span);
                if kind == Kind::As {
                    if !self.is_ts {
                        self.error(diagnostics::as_in_ts(span));
                    }
                    Expression::new_ts_as_expression(span, expr, type_annotation, self)
                } else {
                    if !self.is_ts {
                        self.error(diagnostics::satisfies_in_ts(span));
                    }
                    Expression::new_ts_satisfies_expression(span, expr, type_annotation, self)
                }
            }
        } else {
            expr
        }
    }

    /// `MethodDefinition`[Yield, Await] :
    ///   get `ClassElementName`[?Yield, ?Await] ( ) { `FunctionBody`[~Yield, ~Await] }
    ///   set `ClassElementName`[?Yield, ?Await] ( `PropertySetParameterList` ) { `FunctionBody`[~Yield, ~Await] }
    fn parse_method_getter_setter(
        &mut self,
        span: u32,
        kind: PropertyKind,
        modifiers: &Modifiers,
    ) -> ArenaBox<'a, ObjectProperty<'a>> {
        let (key, computed) = self.parse_property_name();
        let function = self.parse_method(false, false, FunctionKind::ObjectMethod);
        match kind {
            PropertyKind::Get => self.check_getter(&function),
            PropertyKind::Set => self.check_setter(&function),
            PropertyKind::Init | PropertyKind::EtsEquals => {}
        }
        self.verify_modifiers(
            modifiers,
            ModifierKinds::none(),
            true,
            diagnostics::modifier_cannot_be_used_here,
        );
        ObjectProperty::boxed(
            self.end_span(span),
            kind,
            key,
            Expression::FunctionExpression(function),
            /* method */ false,
            /* shorthand */ false,
            /* computed */ computed,
            self,
        )
    }
}
