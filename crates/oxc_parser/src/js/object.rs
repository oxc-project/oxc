use oxc_allocator::Box;
use oxc_ast::ast::*;
use oxc_syntax::operator::AssignmentOperator;

use crate::{
    Context, ParserImpl, diagnostics,
    lexer::Kind,
    modifiers::{ModifierFlags, Modifiers},
};

use super::FunctionKind;

impl<'a> ParserImpl<'a> {
    /// [Object Expression](https://tc39.es/ecma262/#sec-object-initializer)
    /// `ObjectLiteral`[Yield, Await] :
    ///     { }
    ///     { `PropertyDefinitionList`[?Yield, ?Await] }
    ///     { `PropertyDefinitionList`[?Yield, ?Await] , }
    pub(crate) fn parse_object_expression(&mut self) -> Box<'a, ObjectExpression<'a>> {
        let span = self.start_span();
        self.expect(Kind::LCurly);
        let (object_expression_properties, _) = self.context_add(Context::In, |p| {
            p.parse_delimited_list(
                Kind::RCurly,
                Kind::Comma,
                Self::parse_object_expression_property,
            )
        });
        self.expect(Kind::RCurly);
        self.ast.alloc_object_expression(self.end_span(span), object_expression_properties)
    }

    fn parse_object_expression_property(&mut self) -> ObjectPropertyKind<'a> {
        match self.cur_kind() {
            Kind::Dot3 => ObjectPropertyKind::SpreadProperty(self.parse_spread_element()),
            _ => ObjectPropertyKind::ObjectProperty(self.parse_object_literal_element()),
        }
    }

    /// `PropertyDefinition`[Yield, Await]
    fn parse_object_literal_element(&mut self) -> Box<'a, ObjectProperty<'a>> {
        let span = self.start_span();

        let modifiers = self.parse_modifiers(
            /* permit_const_as_modifier */ false,
            /* stop_on_start_of_class_static_block */ false,
        );

        if self.parse_contextual_modifier(Kind::Get) {
            return self.parse_method_getter_setter(span, PropertyKind::Get, &modifiers);
        }

        if self.parse_contextual_modifier(Kind::Set) {
            return self.parse_method_getter_setter(span, PropertyKind::Set, &modifiers);
        }

        let asterisk_token = self.eat(Kind::Star);
        let token_is_identifier =
            self.cur_kind().is_identifier_reference(self.ctx.has_yield(), self.ctx.has_await());
        let (key, computed) = self.parse_property_name();

        if asterisk_token || matches!(self.cur_kind(), Kind::LParen | Kind::LAngle) {
            self.verify_modifiers(
                &modifiers,
                ModifierFlags::ASYNC,
                diagnostics::modifier_cannot_be_used_here,
            );
            let method = self.parse_method(
                modifiers.contains_async(),
                asterisk_token,
                FunctionKind::ObjectMethod,
            );
            return self.ast.alloc_object_property(
                self.end_span(span),
                PropertyKind::Init,
                key,
                Expression::FunctionExpression(method),
                /* method */ true,
                /* shorthand */ false,
                computed,
            );
        }

        self.verify_modifiers(
            &modifiers,
            ModifierFlags::empty(),
            diagnostics::modifier_cannot_be_used_here,
        );

        let is_shorthand_property_assignment = token_is_identifier && !self.at(Kind::Colon);

        if is_shorthand_property_assignment {
            if let PropertyKey::StaticIdentifier(identifier_name) = key {
                let identifier_reference =
                    self.ast.identifier_reference(identifier_name.span, identifier_name.name);
                let value = Expression::Identifier(self.alloc(identifier_reference.clone()));
                // CoverInitializedName ({ foo = bar })
                if self.eat(Kind::Eq) {
                    let right = self.parse_assignment_expression_or_higher();
                    let left = AssignmentTarget::AssignmentTargetIdentifier(
                        self.alloc(identifier_reference),
                    );
                    let expr = self.ast.assignment_expression(
                        self.end_span(span),
                        AssignmentOperator::Assign,
                        left,
                        right,
                    );
                    self.state.cover_initialized_name.insert(span, expr);
                }
                self.ast.alloc_object_property(
                    self.end_span(span),
                    PropertyKind::Init,
                    PropertyKey::StaticIdentifier(identifier_name),
                    value,
                    /* method */ false,
                    /* shorthand */ true,
                    computed,
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
    pub(crate) fn parse_spread_element(&mut self) -> Box<'a, SpreadElement<'a>> {
        let span = self.start_span();
        self.bump_any(); // advance `...`
        let argument = self.parse_assignment_expression_or_higher();
        self.ast.alloc_spread_element(self.end_span(span), argument)
    }

    /// `PropertyDefinition`[Yield, Await] :
    ///   `PropertyName`[?Yield, ?Await] : `AssignmentExpression`[+In, ?Yield, ?Await]
    fn parse_property_definition_assignment(
        &mut self,
        span: u32,
        key: PropertyKey<'a>,
        computed: bool,
    ) -> Box<'a, ObjectProperty<'a>> {
        self.expect(Kind::Colon);
        let value = self.parse_assignment_expression_or_higher();
        self.ast.alloc_object_property(
            self.end_span(span),
            PropertyKind::Init,
            key,
            value,
            /* method */ false,
            /* shorthand */ false,
            /* computed */ computed,
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

    /// `MethodDefinition`[Yield, Await] :
    ///   get `ClassElementName`[?Yield, ?Await] ( ) { `FunctionBody`[~Yield, ~Await] }
    ///   set `ClassElementName`[?Yield, ?Await] ( `PropertySetParameterList` ) { `FunctionBody`[~Yield, ~Await] }
    fn parse_method_getter_setter(
        &mut self,
        span: u32,
        kind: PropertyKind,
        modifiers: &Modifiers<'a>,
    ) -> Box<'a, ObjectProperty<'a>> {
        let (key, computed) = self.parse_property_name();
        let function = self.parse_method(false, false, FunctionKind::ObjectMethod);
        match kind {
            PropertyKind::Get => self.check_getter(&function),
            PropertyKind::Set => self.check_setter(&function),
            PropertyKind::Init => {}
        }
        self.verify_modifiers(
            modifiers,
            ModifierFlags::empty(),
            diagnostics::modifier_cannot_be_used_here,
        );
        self.ast.alloc_object_property(
            self.end_span(span),
            kind,
            key,
            Expression::FunctionExpression(function),
            /* method */ false,
            /* shorthand */ false,
            /* computed */ computed,
        )
    }
}
