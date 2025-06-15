use oxc_allocator::{Box, Vec};
use oxc_ast::ast::*;
use oxc_ecmascript::PropName;
use oxc_span::Span;

use crate::{
    Context, ParserImpl, StatementContext, diagnostics,
    lexer::Kind,
    modifiers::{ModifierFlags, ModifierKind, Modifiers},
};

use super::FunctionKind;

type Extends<'a> =
    Vec<'a, (Expression<'a>, Option<Box<'a, TSTypeParameterInstantiation<'a>>>, Span)>;

/// Section 15.7 Class Definitions
impl<'a> ParserImpl<'a> {
    // `start_span` points at the start of all decoractors and `class` keyword.
    pub(crate) fn parse_class_statement(
        &mut self,
        start_span: u32,
        stmt_ctx: StatementContext,
        modifiers: &Modifiers<'a>,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Statement<'a> {
        let decl = self.parse_class_declaration(start_span, modifiers, decorators);
        if stmt_ctx.is_single_statement() {
            self.error(diagnostics::class_declaration(Span::new(
                decl.span.start,
                decl.body.span.start,
            )));
        }
        Statement::ClassDeclaration(decl)
    }

    /// Section 15.7 Class Definitions
    pub(crate) fn parse_class_declaration(
        &mut self,
        start_span: u32,
        modifiers: &Modifiers<'a>,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Box<'a, Class<'a>> {
        self.parse_class(start_span, ClassType::ClassDeclaration, modifiers, decorators)
    }

    /// Section [Class Definitions](https://tc39.es/ecma262/#prod-ClassExpression)
    /// `ClassExpression`[Yield, Await] :
    ///     class `BindingIdentifier`[?Yield, ?Await]opt `ClassTail`[?Yield, ?Await]
    pub(crate) fn parse_class_expression(
        &mut self,
        span: u32,
        modifiers: &Modifiers<'a>,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Expression<'a> {
        let class = self.parse_class(span, ClassType::ClassExpression, modifiers, decorators);
        Expression::ClassExpression(class)
    }

    fn parse_class(
        &mut self,
        start_span: u32,
        r#type: ClassType,
        modifiers: &Modifiers<'a>,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Box<'a, Class<'a>> {
        self.bump_any(); // advance `class`

        // Move span start to decorator position if this is a class expression.
        let mut start_span = start_span;
        if r#type == ClassType::ClassExpression {
            if let Some(d) = decorators.first() {
                start_span = d.span.start;
            }
        }

        let id = if self.cur_kind().is_binding_identifier() && !self.at(Kind::Implements) {
            Some(self.parse_binding_identifier())
        } else {
            None
        };

        let type_parameters = if self.is_ts { self.parse_ts_type_parameters() } else { None };
        let (extends, implements) = self.parse_heritage_clause();
        let mut super_class = None;
        let mut super_type_parameters = None;
        if let Some(mut extends) = extends {
            if !extends.is_empty() {
                let first_extends = extends.remove(0);
                super_class = Some(first_extends.0);
                super_type_parameters = first_extends.1;
            }
        }
        let body = self.parse_class_body();

        self.verify_modifiers(
            modifiers,
            ModifierFlags::DECLARE | ModifierFlags::ABSTRACT,
            diagnostics::modifier_cannot_be_used_here,
        );

        self.ast.alloc_class(
            self.end_span(start_span),
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_parameters,
            implements,
            body,
            modifiers.contains_abstract(),
            modifiers.contains_declare(),
        )
    }

    pub(crate) fn parse_heritage_clause(
        &mut self,
    ) -> (Option<Extends<'a>>, Vec<'a, TSClassImplements<'a>>) {
        let mut extends = None;
        let mut implements = self.ast.vec();

        loop {
            match self.cur_kind() {
                Kind::Extends => {
                    if extends.is_some() {
                        self.error(diagnostics::extends_clause_already_seen(
                            self.cur_token().span(),
                        ));
                    } else if !implements.is_empty() {
                        self.error(diagnostics::extends_clause_must_precede_implements(
                            self.cur_token().span(),
                        ));
                    }
                    extends = Some(self.parse_extends_clause());
                }
                Kind::Implements => {
                    if !implements.is_empty() {
                        self.error(diagnostics::implements_clause_already_seen(
                            self.cur_token().span(),
                        ));
                    }
                    implements.extend(self.parse_ts_implements_clause());
                }
                _ => break,
            }
        }

        (extends, implements)
    }

    /// `ClassHeritage`
    /// extends `LeftHandSideExpression`[?Yield, ?Await]
    fn parse_extends_clause(&mut self) -> Extends<'a> {
        self.bump_any(); // bump `extends`

        let mut extends = self.ast.vec();
        loop {
            let span = self.start_span();
            let mut extend = self.parse_lhs_expression_or_higher();
            let type_argument;
            if let Expression::TSInstantiationExpression(expr) = extend {
                let expr = expr.unbox();
                extend = expr.expression;
                type_argument = Some(expr.type_arguments);
            } else {
                type_argument = self.try_parse_type_arguments();
            }

            extends.push((extend, type_argument, self.end_span(span)));

            if !self.eat(Kind::Comma) {
                break;
            }
        }

        extends
    }

    fn parse_class_body(&mut self) -> Box<'a, ClassBody<'a>> {
        let span = self.start_span();
        let class_elements = self.parse_normal_list(Kind::LCurly, Kind::RCurly, |p| {
            // Skip empty class element `;`
            if p.eat(Kind::Semicolon) {
                while p.eat(Kind::Semicolon) {}
                if p.at(Kind::RCurly) {
                    return None;
                }
            }
            Some(Self::parse_class_element(p))
        });
        self.ast.alloc_class_body(self.end_span(span), class_elements)
    }

    fn parse_class_element(&mut self) -> ClassElement<'a> {
        let elem = self.parse_class_element_impl();
        if let ClassElement::MethodDefinition(def) = &elem {
            if def.value.body.is_none() && !def.decorators.is_empty() {
                for decorator in &def.decorators {
                    self.error(diagnostics::decorator_on_overload(decorator.span));
                }
            }
        }
        elem
    }

    fn parse_class_element_impl(&mut self) -> ClassElement<'a> {
        let span = self.start_span();

        let decorators = self.parse_decorators();
        let modifiers = self.parse_modifiers(
            /* permit_const_as_modifier */ true,
            /* stop_on_start_of_class_static_block */ true,
        );

        // static { block }
        if self.at(Kind::Static) && self.lookahead(Self::next_token_is_open_brace) {
            for decorator in decorators {
                self.error(diagnostics::decorators_are_not_valid_here(decorator.span));
            }
            for modifier in modifiers.iter() {
                self.error(diagnostics::modifiers_cannot_appear_here(modifier.span));
            }
            return self.parse_class_static_block(span);
        }

        let r#abstract = modifiers.contains(ModifierKind::Abstract);

        let r#type = if r#abstract {
            MethodDefinitionType::TSAbstractMethodDefinition
        } else {
            MethodDefinitionType::MethodDefinition
        };

        if self.parse_contextual_modifier(Kind::Get) {
            return self.parse_accessor_declaration(
                span,
                r#type,
                MethodDefinitionKind::Get,
                &modifiers,
                decorators,
            );
        }

        if self.parse_contextual_modifier(Kind::Set) {
            return self.parse_accessor_declaration(
                span,
                r#type,
                MethodDefinitionKind::Set,
                &modifiers,
                decorators,
            );
        }

        if matches!(self.cur_kind(), Kind::Constructor | Kind::Str)
            && !modifiers.contains(ModifierKind::Static)
        {
            if let Some(name) = self.parse_constructor_name() {
                return self
                    .parse_constructor_declaration(span, r#type, name, &modifiers, decorators);
            }
        }

        if self.is_index_signature() {
            for decorator in decorators {
                self.error(diagnostics::decorators_are_not_valid_here(decorator.span));
            }
            return ClassElement::TSIndexSignature(
                self.parse_index_signature_declaration(span, &modifiers),
            );
        }

        let kind = self.cur_kind();
        if kind.is_identifier_or_keyword() || kind == Kind::Star || kind == Kind::LBrack {
            let is_ambient = modifiers.contains(ModifierKind::Declare);
            return if is_ambient {
                self.context(Context::Ambient, Context::empty(), |p| {
                    p.parse_property_or_method_declaration(span, r#type, &modifiers, decorators)
                })
            } else {
                self.parse_property_or_method_declaration(span, r#type, &modifiers, decorators)
            };
        }

        self.unexpected()
    }

    fn parse_class_element_name(&mut self, modifiers: &Modifiers<'a>) -> (PropertyKey<'a>, bool) {
        if let Some(modifier) = modifiers.iter().find(|m| m.kind == ModifierKind::Const) {
            self.error(diagnostics::const_class_member(modifier.span));
        }
        match self.cur_kind() {
            Kind::PrivateIdentifier => {
                let private_ident = self.parse_private_identifier();
                // `private #foo`, etc. is illegal
                if self.is_ts {
                    self.verify_modifiers(
                        modifiers,
                        ModifierFlags::all() - ModifierFlags::ACCESSIBILITY,
                        diagnostics::accessibility_modifier_on_private_property,
                    );
                }
                if private_ident.name == "constructor" {
                    self.error(diagnostics::private_name_constructor(private_ident.span));
                }
                (PropertyKey::PrivateIdentifier(self.alloc(private_ident)), false)
            }
            _ => self.parse_property_name(),
        }
    }

    /// `ClassStaticBlockStatementList` :
    ///    `StatementList`[~Yield, +Await, ~Return]
    fn parse_class_static_block(&mut self, span: u32) -> ClassElement<'a> {
        self.bump_any(); // bump `static`
        let block =
            self.context(Context::Await, Context::Yield | Context::Return, Self::parse_block);
        self.ast.class_element_static_block(self.end_span(span), block.unbox().body)
    }

    /// <https://github.com/tc39/proposal-decorators>
    fn parse_class_accessor_property(
        &mut self,
        span: u32,
        key: PropertyKey<'a>,
        computed: bool,
        definite: bool,
        modifiers: &Modifiers<'a>,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        let type_annotation = if self.is_ts { self.parse_ts_type_annotation() } else { None };
        let value = self.eat(Kind::Eq).then(|| self.parse_assignment_expression_or_higher());
        self.asi();
        let r#type = if modifiers.contains(ModifierKind::Abstract) {
            AccessorPropertyType::TSAbstractAccessorProperty
        } else {
            AccessorPropertyType::AccessorProperty
        };
        self.verify_modifiers(
            modifiers,
            ModifierFlags::ACCESSIBILITY
                | ModifierFlags::ACCESSOR
                | ModifierFlags::STATIC
                | ModifierFlags::ABSTRACT
                | ModifierFlags::OVERRIDE,
            diagnostics::accessor_modifier,
        );
        self.ast.class_element_accessor_property(
            self.end_span(span),
            r#type,
            decorators,
            key,
            type_annotation,
            value,
            computed,
            modifiers.contains(ModifierKind::Static),
            modifiers.contains(ModifierKind::Override),
            definite,
            modifiers.accessibility(),
        )
    }

    fn parse_accessor_declaration(
        &mut self,
        span: u32,
        r#type: MethodDefinitionType,
        kind: MethodDefinitionKind,
        modifiers: &Modifiers<'a>,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        let (name, computed) = self.parse_class_element_name(modifiers);
        let value = self.parse_method(
            modifiers.contains(ModifierKind::Async),
            false,
            FunctionKind::ClassMethod,
        );
        let method_definition = self.ast.alloc_method_definition(
            self.end_span(span),
            r#type,
            decorators,
            name,
            value,
            kind,
            computed,
            modifiers.contains(ModifierKind::Static),
            modifiers.contains(ModifierKind::Override),
            false,
            modifiers.accessibility(),
        );
        self.check_method_definition(&method_definition);
        self.verify_modifiers(
            modifiers,
            ModifierFlags::all() - ModifierFlags::ASYNC - ModifierFlags::DECLARE,
            diagnostics::modifier_cannot_be_used_here,
        );
        ClassElement::MethodDefinition(method_definition)
    }

    fn parse_constructor_declaration(
        &mut self,
        span: u32,
        r#type: MethodDefinitionType,
        name: PropertyKey<'a>,
        modifiers: &Modifiers<'a>,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        let value = self.parse_method(
            modifiers.contains(ModifierKind::Async),
            false,
            FunctionKind::ClassMethod,
        );
        let method_definition = self.ast.alloc_method_definition(
            self.end_span(span),
            r#type,
            decorators,
            name,
            value,
            MethodDefinitionKind::Constructor,
            false,
            modifiers.contains(ModifierKind::Static),
            modifiers.contains(ModifierKind::Override),
            false,
            modifiers.accessibility(),
        );
        self.check_method_definition(&method_definition);
        ClassElement::MethodDefinition(method_definition)
    }

    fn parse_constructor_name(&mut self) -> Option<PropertyKey<'a>> {
        if self.at(Kind::Constructor) {
            let ident = self.parse_identifier_name();
            return Some(PropertyKey::StaticIdentifier(self.alloc(ident)));
        }
        if self.at(Kind::Str)
            && self.lookahead(|p| {
                p.bump_any();
                p.at(Kind::LParen)
            })
        {
            return self.try_parse(|p| {
                let string_literal = p.parse_literal_string();
                if string_literal.value != "constructor" {
                    return p.unexpected();
                }
                PropertyKey::StringLiteral(p.alloc(string_literal))
            });
        }
        None
    }

    fn parse_property_or_method_declaration(
        &mut self,
        span: u32,
        r#type: MethodDefinitionType,
        modifiers: &Modifiers<'a>,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        let generator = self.eat(Kind::Star);
        let (name, computed) = self.parse_class_element_name(modifiers);

        let cur_token = self.cur_token();
        let optional_span = (cur_token.kind() == Kind::Question).then(|| {
            let span = cur_token.span();
            self.bump_any();
            span
        });

        let optional = optional_span.is_some();

        if generator || matches!(self.cur_kind(), Kind::LParen | Kind::LAngle) {
            return self.parse_method_declaration(
                span, r#type, generator, name, computed, optional, modifiers, decorators,
            );
        }

        let definite = self.eat(Kind::Bang);

        if definite {
            if let Some(optional_span) = optional_span {
                self.error(diagnostics::optional_definite_property(optional_span.expand_right(1)));
            }
        }

        if modifiers.contains(ModifierKind::Accessor) {
            if let Some(optional_span) = optional_span {
                self.error(diagnostics::optional_accessor_property(optional_span));
            }
            return self.parse_class_accessor_property(
                span, name, computed, definite, modifiers, decorators,
            );
        }

        self.parse_property_declaration(
            span,
            name,
            computed,
            optional_span,
            definite,
            modifiers,
            decorators,
        )
    }

    fn parse_method_declaration(
        &mut self,
        span: u32,
        r#type: MethodDefinitionType,
        generator: bool,
        name: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        modifiers: &Modifiers<'a>,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        let value = self.parse_method(
            modifiers.contains(ModifierKind::Async),
            generator,
            FunctionKind::ClassMethod,
        );
        let method_definition = self.ast.alloc_method_definition(
            self.end_span(span),
            r#type,
            decorators,
            name,
            value,
            MethodDefinitionKind::Method,
            computed,
            modifiers.contains(ModifierKind::Static),
            modifiers.contains(ModifierKind::Override),
            optional,
            modifiers.accessibility(),
        );
        self.check_method_definition(&method_definition);
        ClassElement::MethodDefinition(method_definition)
    }

    fn parse_property_declaration(
        &mut self,
        span: u32,
        name: PropertyKey<'a>,
        computed: bool,
        optional_span: Option<Span>,
        definite: bool,
        modifiers: &Modifiers<'a>,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        let type_annotation = if self.is_ts { self.parse_ts_type_annotation() } else { None };
        // Initializer[+In, ?Yield, ?Await]opt
        let initializer = self
            .eat(Kind::Eq)
            .then(|| self.context(Context::In, Context::Yield | Context::Await, Self::parse_expr));

        // Handle trailing `;` or newline
        let cur_token = self.cur_token();
        if cur_token.kind() == Kind::Semicolon {
            self.bump_any();
        } else if !self.can_insert_semicolon() {
            let error = diagnostics::expect_token(";", cur_token.kind().to_str(), cur_token.span());
            return self.fatal_error(error);
        }

        let r#abstract = modifiers.contains(ModifierKind::Abstract);
        let r#type = if r#abstract {
            PropertyDefinitionType::TSAbstractPropertyDefinition
        } else {
            PropertyDefinitionType::PropertyDefinition
        };
        let r#static = modifiers.contains(ModifierKind::Static);
        if !computed {
            if let Some((name, span)) = name.prop_name() {
                if name == "constructor" {
                    self.error(diagnostics::field_constructor(span));
                }
                if r#static && name == "prototype" && !self.ctx.has_ambient() {
                    self.error(diagnostics::static_prototype(span));
                }
            }
        }
        self.ast.class_element_property_definition(
            self.end_span(span),
            r#type,
            decorators,
            name,
            type_annotation,
            initializer,
            computed,
            r#static,
            modifiers.contains(ModifierKind::Declare),
            modifiers.contains(ModifierKind::Override),
            optional_span.is_some(),
            definite,
            modifiers.contains(ModifierKind::Readonly),
            modifiers.accessibility(),
        )
    }

    pub(crate) fn check_getter(&mut self, function: &Function<'a>) {
        if !function.params.items.is_empty() {
            self.error(diagnostics::getter_parameters(function.params.span));
        }
    }

    pub(crate) fn check_setter(&mut self, function: &Function<'a>) {
        if let Some(rest) = &function.params.rest {
            self.error(diagnostics::setter_with_rest_parameter(rest.span));
        } else if function.params.parameters_count() != 1 {
            self.error(diagnostics::setter_with_parameters(function.params.span));
        }
    }

    fn check_method_definition(&mut self, method: &MethodDefinition<'a>) {
        let function = &method.value;
        match method.kind {
            MethodDefinitionKind::Get => self.check_getter(function),
            MethodDefinitionKind::Set => self.check_setter(function),
            _ => {}
        }
        if !method.computed {
            if let Some((name, span)) = method.key.prop_name() {
                if method.r#static && name == "prototype" && !self.ctx.has_ambient() {
                    self.error(diagnostics::static_prototype(span));
                }
                if !method.r#static && name == "constructor" {
                    if method.kind == MethodDefinitionKind::Get
                        || method.kind == MethodDefinitionKind::Set
                    {
                        self.error(diagnostics::constructor_getter_setter(span));
                    }
                    if method.value.r#async {
                        self.error(diagnostics::constructor_async(span));
                    }
                    if method.value.generator {
                        self.error(diagnostics::constructor_generator(span));
                    }
                }
            }
        }
        if method.kind == MethodDefinitionKind::Constructor {
            if let Some(this_param) = &method.value.this_param {
                // class Foo { constructor(this: number) {} }
                self.error(diagnostics::ts_constructor_this_parameter(this_param.span));
            }

            if let Some(type_sig) = &method.value.type_parameters {
                // class Foo { constructor<T>(param: T ) {} }
                self.error(diagnostics::ts_constructor_type_parameter(type_sig.span));
            }
        }
    }
}
