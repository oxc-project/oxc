use oxc_allocator::Box;
use oxc_ast::{ast::*, Span};
use oxc_diagnostics::Result;

use super::list::ObjectExpressionProperties;
use crate::{diagnostics, lexer::Kind, list::SeparatedList, Parser};

impl<'a> Parser<'a> {
    /// [Object Expression](https://tc39.es/ecma262/#sec-object-initializer)
    /// `ObjectLiteral`[Yield, Await] :
    ///     { }
    ///     { `PropertyDefinitionList`[?Yield, ?Await] }
    ///     { `PropertyDefinitionList`[?Yield, ?Await] , }
    pub fn parse_object_expression(&mut self) -> Result<Expression<'a>> {
        let span = self.start_span();

        let has_in = self.ctx.has_in();
        self.ctx = self.ctx.and_in(true);
        let object_expression_properties = ObjectExpressionProperties::parse(self)?;
        self.ctx = self.ctx.and_in(has_in);

        Ok(self.ast.object_expression(
            self.end_span(span),
            object_expression_properties.elements,
            object_expression_properties.trailing_comma,
        ))
    }

    /// `PropertyDefinition`[Yield, Await]
    pub fn parse_property_definition(&mut self) -> Result<Box<'a, Property<'a>>> {
        let peek_kind = self.peek_kind();
        let class_element_name = peek_kind.is_class_element_name_start();
        match self.cur_kind() {
            // get ClassElementName
            Kind::Get if class_element_name => self.parse_method_getter(),
            // set ClassElementName
            Kind::Set if class_element_name => self.parse_method_setter(),
            // AsyncMethod
            // AsyncGeneratorMethod
            Kind::Async
                if (class_element_name || peek_kind == Kind::Star)
                    && !self.peek_token().is_on_new_line =>
            {
                self.parse_property_definition_method()
            }
            // GeneratorMethod
            Kind::Star if class_element_name => self.parse_property_definition_method(),
            // IdentifierReference
            kind if kind.is_identifier_reference(false, false)
                // test Kind::Dot to ignore ({ foo.bar: baz })
                // see https://stackoverflow.com/questions/30285947/syntaxerror-unexpected-token
                && !matches!(
                    peek_kind,
                    Kind::LParen | Kind::Colon | Kind::LAngle | Kind::ShiftLeft | Kind::Dot
                ) =>
            {
                self.parse_property_definition_shorthand()
            }
            _ => {
                let span = self.start_span();
                let (key, computed) = self.parse_property_name()?;

                if self.at(Kind::Colon) {
                    return self.parse_property_definition_assignment(span, key, computed);
                }

                if matches!(self.cur_kind(), Kind::LParen | Kind::LAngle | Kind::ShiftLeft) {
                    let method = self.parse_method(false, false)?;
                    return Ok(self.ast.property(
                        self.end_span(span),
                        PropertyKind::Init,
                        key,
                        PropertyValue::Expression(self.ast.function_expression(method)),
                        /* method */ true,
                        /* shorthand */ false,
                        /* computed */ computed,
                    ));
                }

                self.unexpected()
            }
        }
    }

    /// `PropertyDefinition`[Yield, Await] :
    ///   ... `AssignmentExpression`[+In, ?Yield, ?Await]
    pub fn parse_spread_element(&mut self) -> Result<Box<'a, SpreadElement<'a>>> {
        let span = self.start_span();
        self.bump_any(); // advance `...`
        let argument = self.parse_assignment_expression_base()?;
        Ok(self.ast.spread_element(self.end_span(span), argument))
    }

    /// `PropertyDefinition`[Yield, Await] :
    ///   `IdentifierReference`[?Yield, ?Await]
    ///   `CoverInitializedName`[?Yield, ?Await]
    fn parse_property_definition_shorthand(&mut self) -> Result<Box<'a, Property<'a>>> {
        let span = self.start_span();
        let identifier = self.parse_identifier_reference()?;
        // CoverInitializedName ({ foo = bar })
        let value = if self.eat(Kind::Eq) {
            let right = self.parse_assignment_expression_base()?;
            let left = AssignmentTarget::SimpleAssignmentTarget(
                SimpleAssignmentTarget::AssignmentTargetIdentifier(
                    self.ast.alloc(identifier.clone()),
                ),
            );
            self.ast.assignment_expression(
                self.end_span(span),
                AssignmentOperator::Assign,
                left,
                right,
            )
        } else {
            // IdentifierReference ({ foo })
            Expression::Identifier(self.ast.alloc(identifier.clone()))
        };
        Ok(self.ast.property(
            self.end_span(span),
            PropertyKind::Init,
            PropertyKey::Identifier(
                self.ast.alloc(IdentifierName { span: identifier.span, name: identifier.name }),
            ),
            PropertyValue::Expression(value),
            /* method */ false,
            /* shorthand */ true,
            /* computed */ false,
        ))
    }

    /// `PropertyDefinition`[Yield, Await] :
    ///   `PropertyName`[?Yield, ?Await] : `AssignmentExpression`[+In, ?Yield, ?Await]
    fn parse_property_definition_assignment(
        &mut self,
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
    ) -> Result<Box<'a, Property<'a>>> {
        self.bump_any(); // bump `:`
        let value = self.parse_assignment_expression_base()?;
        Ok(self.ast.property(
            self.end_span(span),
            PropertyKind::Init,
            key,
            PropertyValue::Expression(value),
            /* method */ false,
            /* shorthand */ false,
            /* computed */ computed,
        ))
    }

    /// `PropertyName`[Yield, Await] :
    ///    `LiteralPropertyName`
    ///    `ComputedPropertyName`[?Yield, ?Await]
    pub fn parse_property_name(&mut self) -> Result<(PropertyKey<'a>, bool)> {
        let mut computed = false;
        let key = match self.cur_kind() {
            Kind::Str => self.parse_literal_expression().map(PropertyKey::Expression)?,
            kind if kind.is_number() => {
                self.parse_literal_expression().map(PropertyKey::Expression)?
            }
            // { [foo]() {} }
            Kind::LBrack => {
                computed = true;
                self.parse_computed_property_name().map(PropertyKey::Expression)?
            }
            _ => {
                let ident = self.parse_identifier_name()?;
                PropertyKey::Identifier(self.ast.alloc(ident))
            }
        };
        Ok((key, computed))
    }

    /// `ComputedPropertyName`[Yield, Await] : [ `AssignmentExpression`[+In, ?Yield, ?Await] ]
    pub fn parse_computed_property_name(&mut self) -> Result<Expression<'a>> {
        self.bump_any(); // advance `[`

        let has_in = self.ctx.has_in();
        self.ctx = self.ctx.and_in(true);
        let expression = self.parse_assignment_expression_base()?;
        self.ctx = self.ctx.and_in(has_in);

        self.expect(Kind::RBrack)?;
        Ok(expression)
    }

    /// `PropertyDefinition`[Yield, Await] :
    ///   `MethodDefinition`[?Yield, ?Await]
    fn parse_property_definition_method(&mut self) -> Result<Box<'a, Property<'a>>> {
        let span = self.start_span();
        let r#async = self.eat(Kind::Async);
        let generator = self.eat(Kind::Star);
        let (key, computed) = self.parse_property_name()?;
        let method = self.parse_method(r#async, generator)?;
        let value = PropertyValue::Expression(self.ast.function_expression(method));
        Ok(self.ast.property(
            self.end_span(span),
            PropertyKind::Init,
            key,
            value,
            /* method */ true,
            /* shorthand */ false,
            /* computed */ computed,
        ))
    }

    /// `MethodDefinition`[Yield, Await] :
    ///   get `ClassElementName`[?Yield, ?Await] ( ) { `FunctionBody`[~Yield, ~Await] }
    fn parse_method_getter(&mut self) -> Result<Box<'a, Property<'a>>> {
        let span = self.start_span();
        self.expect(Kind::Get)?;
        let (key, computed) = self.parse_property_name()?;
        let method = self.parse_method(false, false)?;

        if !method.params.is_empty() {
            self.error(diagnostics::GetterParameters(method.params.span));
        }

        let value = PropertyValue::Expression(self.ast.function_expression(method));
        Ok(self.ast.property(
            self.end_span(span),
            PropertyKind::Get,
            key,
            value,
            /* method */ false,
            /* shorthand */ false,
            /* computed */ computed,
        ))
    }

    /// `MethodDefinition`[Yield, Await] :
    /// set `ClassElementName`[?Yield, ?Await] ( `PropertySetParameterList` ) { `FunctionBody`[~Yield, ~Await] }
    fn parse_method_setter(&mut self) -> Result<Box<'a, Property<'a>>> {
        let span = self.start_span();
        self.expect(Kind::Set)?;
        let (key, computed) = self.parse_property_name()?;
        let method = self.parse_method(false, false)?;

        if method.params.items.len() != 1 {
            self.error(diagnostics::SetterParameters(method.params.span));
        }

        if method.params.items.len() == 1 {
            if let BindingPatternKind::RestElement(elem) = &method.params.items[0].pattern.kind {
                self.error(diagnostics::SetterParametersRestPattern(elem.span));
            }
        }

        Ok(self.ast.property(
            self.end_span(span),
            PropertyKind::Set,
            key,
            PropertyValue::Expression(self.ast.function_expression(method)),
            /* method */ false,
            /* shorthand */ false,
            /* computed */ computed,
        ))
    }
}
