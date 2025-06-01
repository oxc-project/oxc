use oxc_allocator::Box;
use oxc_ast::ast::*;
use oxc_syntax::operator::AssignmentOperator;

use crate::{Context, ParserImpl, diagnostics, lexer::Kind, modifiers::Modifier};

impl<'a> ParserImpl<'a> {
    /// [Object Expression](https://tc39.es/ecma262/#sec-object-initializer)
    /// `ObjectLiteral`[Yield, Await] :
    ///     { }
    ///     { `PropertyDefinitionList`[?Yield, ?Await] }
    ///     { `PropertyDefinitionList`[?Yield, ?Await] , }
    pub(crate) fn parse_object_expression(&mut self) -> Box<'a, ObjectExpression<'a>> {
        let span = self.start_span();
        self.expect(Kind::LCurly);
        let (object_expression_properties, _) = self.context(Context::In, Context::empty(), |p| {
            p.parse_delimited_list(
                Kind::RCurly,
                Kind::Comma,
                Self::parse_object_expression_property,
            )
        });
        self.bump(Kind::Comma); // Trailing Comma
        self.expect(Kind::RCurly);
        self.ast.alloc_object_expression(self.end_span(span), object_expression_properties)
    }

    fn parse_object_expression_property(&mut self) -> ObjectPropertyKind<'a> {
        match self.cur_kind() {
            Kind::Dot3 => ObjectPropertyKind::SpreadProperty(self.parse_spread_element()),
            _ => ObjectPropertyKind::ObjectProperty(self.parse_property_definition()),
        }
    }

    /// `PropertyDefinition`[Yield, Await]
    pub(crate) fn parse_property_definition(&mut self) -> Box<'a, ObjectProperty<'a>> {
        let checkpoint = self.checkpoint();
        self.bump_any();
        let peek_kind = self.cur_kind();
        let peek_token = self.cur_token();
        let class_element_name = peek_kind.is_class_element_name_start();
        self.rewind(checkpoint);
        match self.cur_kind() {
            // get ClassElementName
            Kind::Get if class_element_name => self.parse_method_getter(),
            // set ClassElementName
            Kind::Set if class_element_name => self.parse_method_setter(),
            // AsyncMethod
            // AsyncGeneratorMethod
            Kind::Async
                if (class_element_name || peek_kind == Kind::Star)
                    && !peek_token.is_on_new_line() =>
            {
                self.parse_property_definition_method()
            }
            // GeneratorMethod
            Kind::Star if class_element_name => self.parse_property_definition_method(),
            // Report and handle illegal modifiers
            // e.g. const x = { public foo() {} }
            modifier_kind
                if self.is_ts
                    && modifier_kind.is_modifier_kind()
                    && peek_kind.is_identifier_or_keyword() =>
            {
                if let Ok(modifier) = Modifier::try_from(self.cur_token()) {
                    self.error(diagnostics::modifier_cannot_be_used_here(&modifier));
                } else {
                    #[cfg(debug_assertions)]
                    panic!(
                        "Kind::is_modifier_kind() is true but the token could not be converted to a Modifier."
                    )
                }
                // re-parse
                self.bump_any();
                self.parse_property_definition()
            }
            // IdentifierReference
            kind if kind.is_identifier_reference(false, false)
                // test Kind::Dot to ignore ({ foo.bar: baz })
                // see <https://stackoverflow.com/questions/30285947/syntaxerror-unexpected-token>
                && !matches!(
                    peek_kind,
                    Kind::LParen | Kind::Colon | Kind::LAngle | Kind::ShiftLeft | Kind::Dot
                ) =>
            {
                self.parse_property_definition_shorthand()
            }
            _ => {
                let span = self.start_span();
                let (key, computed) = self.parse_property_name();

                if self.at(Kind::Colon) {
                    return self.parse_property_definition_assignment(span, key, computed);
                }

                if matches!(self.cur_kind(), Kind::LParen | Kind::LAngle | Kind::ShiftLeft) {
                    let method = self.parse_method(false, false);
                    return self.ast.alloc_object_property(
                        self.end_span(span),
                        PropertyKind::Init,
                        key,
                        Expression::FunctionExpression(method),
                        /* method */ true,
                        /* shorthand */ false,
                        /* computed */ computed,
                    );
                }

                self.unexpected()
            }
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
    ///   `IdentifierReference`[?Yield, ?Await]
    ///   `CoverInitializedName`[?Yield, ?Await]
    fn parse_property_definition_shorthand(&mut self) -> Box<'a, ObjectProperty<'a>> {
        let span = self.start_span();
        let identifier = self.parse_identifier_reference();
        let key = self.ast.alloc_identifier_name(identifier.span, identifier.name);
        // IdentifierReference ({ foo })
        let value = Expression::Identifier(self.alloc(identifier.clone()));
        // CoverInitializedName ({ foo = bar })
        if self.eat(Kind::Eq) {
            let right = self.parse_assignment_expression_or_higher();
            let left = AssignmentTarget::AssignmentTargetIdentifier(self.alloc(identifier));
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
            PropertyKey::StaticIdentifier(key),
            value,
            /* method */ false,
            /* shorthand */ true,
            /* computed */ false,
        )
    }

    /// `PropertyDefinition`[Yield, Await] :
    ///   `PropertyName`[?Yield, ?Await] : `AssignmentExpression`[+In, ?Yield, ?Await]
    fn parse_property_definition_assignment(
        &mut self,
        span: u32,
        key: PropertyKey<'a>,
        computed: bool,
    ) -> Box<'a, ObjectProperty<'a>> {
        self.bump_any(); // bump `:`
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

        let expression = self.context(
            Context::In,
            Context::empty(),
            Self::parse_assignment_expression_or_higher,
        );

        self.expect(Kind::RBrack);
        expression
    }

    /// `PropertyDefinition`[Yield, Await] :
    ///   `MethodDefinition`[?Yield, ?Await]
    fn parse_property_definition_method(&mut self) -> Box<'a, ObjectProperty<'a>> {
        let span = self.start_span();
        let r#async = self.eat(Kind::Async);
        let generator = self.eat(Kind::Star);
        let (key, computed) = self.parse_property_name();
        let method = self.parse_method(r#async, generator);
        let value = Expression::FunctionExpression(method);
        self.ast.alloc_object_property(
            self.end_span(span),
            PropertyKind::Init,
            key,
            value,
            /* method */ true,
            /* shorthand */ false,
            /* computed */ computed,
        )
    }

    /// `MethodDefinition`[Yield, Await] :
    ///   get `ClassElementName`[?Yield, ?Await] ( ) { `FunctionBody`[~Yield, ~Await] }
    fn parse_method_getter(&mut self) -> Box<'a, ObjectProperty<'a>> {
        let span = self.start_span();
        self.expect(Kind::Get);
        let (key, computed) = self.parse_property_name();
        let method = self.parse_method(false, false);
        let value = Expression::FunctionExpression(method);
        self.ast.alloc_object_property(
            self.end_span(span),
            PropertyKind::Get,
            key,
            value,
            /* method */ false,
            /* shorthand */ false,
            /* computed */ computed,
        )
    }

    /// `MethodDefinition`[Yield, Await] :
    /// set `ClassElementName`[?Yield, ?Await] ( `PropertySetParameterList` ) { `FunctionBody`[~Yield, ~Await] }
    fn parse_method_setter(&mut self) -> Box<'a, ObjectProperty<'a>> {
        let span = self.start_span();
        self.expect(Kind::Set);
        let (key, computed) = self.parse_property_name();
        let method = self.parse_method(false, false);

        self.ast.alloc_object_property(
            self.end_span(span),
            PropertyKind::Set,
            key,
            Expression::FunctionExpression(method),
            /* method */ false,
            /* shorthand */ false,
            /* computed */ computed,
        )
    }
}
