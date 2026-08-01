use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::ast::*;
use oxc_ecmascript::PropName;
use oxc_span::{GetSpan, Span};

use crate::{
    Context, ParserConfig as Config, ParserImpl, StatementContext, diagnostics,
    lexer::Kind,
    modifiers::{ModifierKind, ModifierKinds, Modifiers},
};

use super::FunctionKind;

type ImplementsWithKeywordSpan<'a> = (Span, ArenaVec<'a, TSClassImplements<'a>>);

/// Section 15.7 Class Definitions
impl<'a, C: Config> ParserImpl<'a, C> {
    // `start_span` points at the start of all decoractors and `class` keyword.
    pub(crate) fn parse_class_statement(
        &mut self,
        start_span: u32,
        stmt_ctx: StatementContext,
        modifiers: &Modifiers,
        decorators: ArenaVec<'a, Decorator<'a>>,
    ) -> Statement<'a> {
        if self.source_type.is_ets_static() && !self.state.ets_in_declaration_scope {
            self.error(diagnostics::ets_nested_declaration("Class", Span::empty(start_span)));
        }
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
        modifiers: &Modifiers,
        decorators: ArenaVec<'a, Decorator<'a>>,
    ) -> ArenaBox<'a, Class<'a>> {
        self.parse_class(start_span, ClassType::ClassDeclaration, modifiers, decorators)
    }

    /// Section [Class Definitions](https://tc39.es/ecma262/#prod-ClassExpression)
    /// `ClassExpression`[Yield, Await] :
    ///     class `BindingIdentifier`[?Yield, ?Await]opt `ClassTail`[?Yield, ?Await]
    pub(crate) fn parse_class_expression(
        &mut self,
        span: u32,
        modifiers: &Modifiers,
        decorators: ArenaVec<'a, Decorator<'a>>,
    ) -> Expression<'a> {
        let class = self.parse_class(span, ClassType::ClassExpression, modifiers, decorators);
        Expression::ClassExpression(class)
    }

    fn parse_class(
        &mut self,
        start_span: u32,
        r#type: ClassType,
        modifiers: &Modifiers,
        decorators: ArenaVec<'a, Decorator<'a>>,
    ) -> ArenaBox<'a, Class<'a>> {
        self.bump_any(); // advance `class`

        // Move span start to decorator position if this is a class expression.
        let mut start_span = start_span;
        if r#type == ClassType::ClassExpression
            && let Some(d) = decorators.first()
        {
            start_span = d.span.start;
        }

        let id = if self.cur_kind().is_binding_identifier()
            && !(self.at(Kind::Implements)
                && self.lexer.peek_token().kind().is_identifier_or_keyword())
        {
            Some(self.parse_binding_identifier())
        } else {
            None
        };
        // A class name may not be a reserved type name, but only in TypeScript
        // (`class string {}` is valid JavaScript).
        if self.is_ts
            && let Some(id) = &id
        {
            self.check_reserved_type_name(id, "Class");
        }
        if let Some(id) = &id {
            self.register_ets_type_name(id.name);
        }

        let type_parameters =
            if self.is_ts { self.parse_ts_type_parameters_with_variance() } else { None };
        let (extends, implements) = self.parse_heritage_clause(Self::parse_class_extends_clause);
        let mut super_class = None;
        let mut super_type_parameters = None;
        if let Some(mut extends) = extends
            && !extends.is_empty()
        {
            let (expression, type_arguments) = extends.remove(0);
            super_class = Some(expression);
            super_type_parameters = type_arguments;
            for (expression, type_arguments) in extends {
                let expression_span = expression.span();
                let span = type_arguments.map_or(expression_span, |type_arguments| {
                    expression_span.merge(type_arguments.span)
                });
                self.error(diagnostics::classes_can_only_extend_single_class(span));
            }
        }
        let body = self.parse_class_body();

        if self.source_type.is_ets_static() {
            for modifier in modifiers.iter() {
                let allowed = matches!(
                    modifier.kind,
                    ModifierKind::Declare
                        | ModifierKind::Abstract
                        | ModifierKind::Final
                        | ModifierKind::Export
                        | ModifierKind::Default
                );
                if !allowed {
                    self.error(diagnostics::ets_modifier_not_allowed(&modifier, "a class"));
                }
            }
        }

        self.verify_modifiers(
            modifiers,
            ModifierKinds::new([ModifierKind::Declare, ModifierKind::Abstract]),
            true,
            diagnostics::modifier_cannot_be_used_here,
        );

        let mut class = Class::boxed(
            self.end_span(start_span),
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_parameters,
            implements.map_or_else(|| ArenaVec::new_in(self), |(_, implements)| implements),
            body,
            modifiers.contains_abstract(),
            modifiers.contains_declare(),
            self,
        );
        if self.source_type.is_ets_static() {
            class.r#final = modifiers.contains(ModifierKind::Final);
            class.native = modifiers.contains(ModifierKind::Native);
            class.r#static = modifiers.contains(ModifierKind::Static);
        }
        class
    }

    pub(crate) fn parse_heritage_clause<T, F>(
        &mut self,
        mut parse_extends_clause: F,
    ) -> (Option<ArenaVec<'a, T>>, Option<ImplementsWithKeywordSpan<'a>>)
    where
        F: FnMut(&mut Self) -> ArenaVec<'a, T>,
    {
        let mut extends: Option<ArenaVec<'a, T>> = None;
        let mut implements: Option<ImplementsWithKeywordSpan> = None;

        loop {
            match self.cur_kind() {
                Kind::Extends => {
                    if extends.is_some() {
                        self.error(diagnostics::extends_clause_already_seen(
                            self.cur_token().span(),
                        ));
                    } else if let Some((implements_span, _)) = implements {
                        self.error(diagnostics::extends_clause_must_precede_implements(
                            self.cur_token().span(),
                            implements_span,
                        ));
                    }
                    extends = Some(parse_extends_clause(self));
                }
                Kind::Implements => {
                    if let Some((implements_span, _)) = implements {
                        self.error(diagnostics::implements_clause_already_seen(
                            self.cur_token().span(),
                            implements_span,
                        ));
                    }
                    let implements_kw_span = self.cur_token().span();
                    if !self.is_ts {
                        self.error(diagnostics::implements_clause_in_ts(implements_kw_span));
                    }
                    if let Some((_, implements)) = implements.as_mut() {
                        implements.extend(self.parse_ts_implements_clause());
                    } else {
                        implements = Some((implements_kw_span, self.parse_ts_implements_clause()));
                    }
                }
                _ => break,
            }
        }

        (extends, implements)
    }

    /// `ClassHeritage`
    /// extends `LeftHandSideExpression`[?Yield, ?Await]
    pub(crate) fn parse_class_extends_clause(
        &mut self,
    ) -> ArenaVec<'a, (Expression<'a>, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>)>
    {
        self.bump_any(); // bump `extends`

        let mut extends = ArenaVec::with_capacity_in(1, self);
        loop {
            let mut extend = self.parse_lhs_expression_or_higher();
            if self.fatal_error.is_some() {
                break;
            }
            let type_argument;
            if let Expression::TSInstantiationExpression(expr) = extend {
                let expr = expr.unbox();
                extend = expr.expression;
                type_argument = Some(expr.type_arguments);
            } else {
                type_argument = self.try_parse_type_arguments();
            }

            extends.push((extend, type_argument));

            if !self.eat(Kind::Comma) {
                break;
            }
        }

        extends
    }

    fn parse_class_body(&mut self) -> ArenaBox<'a, ClassBody<'a>> {
        let span = self.start_span();
        let class_elements = self.parse_normal_list_breakable(Kind::LCurly, Kind::RCurly, |p| {
            // Skip empty class element `;`
            if p.eat(Kind::Semicolon) {
                while p.eat(Kind::Semicolon) {}
                if p.at(Kind::RCurly) {
                    return None;
                }
            }
            Some(Self::parse_class_element(p))
        });
        if self.source_type.is_ets_static() {
            let mut seen_index_signature = false;
            for element in &class_elements {
                if matches!(element, ClassElement::TSIndexSignature(_)) {
                    if seen_index_signature {
                        self.error(diagnostics::ets_unsupported_syntax(
                            "Multiple index signatures in one class",
                            element.span(),
                        ));
                    }
                    seen_index_signature = true;
                }
            }
        }
        ClassBody::boxed(self.end_span(span), class_elements, self)
    }

    fn parse_class_element(&mut self) -> ClassElement<'a> {
        let elem = self.parse_class_element_impl();
        if let ClassElement::MethodDefinition(def) = &elem
            && def.value.body.is_none()
            && !def.decorators.is_empty()
            && !self.source_type.is_ets_static()
        {
            for decorator in &def.decorators {
                self.error(diagnostics::decorator_on_overload(decorator.span));
            }
        }
        elem
    }

    fn parse_class_element_impl(&mut self) -> ClassElement<'a> {
        let span = self.start_span();

        if self.source_type.is_ets_static()
            && matches!(self.cur_kind(), Kind::LParen | Kind::LAngle)
        {
            let signature_span = self.cur_token().span();
            if !self.ctx.has_ambient() {
                self.error(diagnostics::ets_unsupported_syntax(
                    "Call signatures in non-ambient classes",
                    signature_span,
                ));
            }
            let signature =
                self.parse_signature_member(crate::ts::CallOrConstructorSignature::Call);
            let TSSignature::TSCallSignatureDeclaration(signature) = signature else {
                unreachable!()
            };
            return ClassElement::TSCallSignatureDeclaration(signature);
        }

        let decorators = self.parse_decorators();
        let modifiers = self.parse_modifiers(
            /* permit_const_as_modifier */ true,
            /* stop_on_start_of_class_static_block */ true,
        );

        if self.source_type.is_ets_static() && self.at(Kind::Overload) {
            return ClassElement::ETSOverloadDeclaration(self.parse_ets_overload_declaration(
                span,
                decorators,
                &modifiers,
                ETSOverloadDeclarationKind::ClassMethod,
            ));
        }

        // static { block }
        if self.at(Kind::Static) && self.lexer.peek_token().kind() == Kind::LCurly {
            for decorator in decorators {
                self.error(diagnostics::decorators_are_not_valid_here(decorator.span));
            }
            self.verify_modifiers(
                &modifiers,
                ModifierKinds::none(),
                false,
                diagnostics::modifiers_cannot_appear_here,
            );
            return self.parse_class_static_block(span);
        }

        self.verify_modifiers(
            &modifiers,
            ModifierKinds::all_except([ModifierKind::Export]),
            false,
            diagnostics::cannot_appear_on_class_elements,
        );

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
            && let Some(name) = self.parse_constructor_name()
        {
            return self.parse_constructor_declaration(span, r#type, name, &modifiers, decorators);
        }

        if self.is_index_signature() {
            for decorator in decorators {
                self.error(diagnostics::decorators_are_not_valid_here(decorator.span));
            }

            // No modifiers except `static` and `readonly` are valid here
            self.verify_modifiers(
                &modifiers,
                ModifierKinds::new([ModifierKind::Readonly, ModifierKind::Static]),
                true,
                diagnostics::cannot_appear_on_an_index_signature,
            );

            return ClassElement::TSIndexSignature(
                self.parse_index_signature_declaration(span, &modifiers),
            );
        }

        let kind = self.cur_kind();
        if kind.is_identifier_or_keyword() || kind == Kind::Star || kind == Kind::LBrack {
            let is_ambient = modifiers.contains(ModifierKind::Declare);
            return if is_ambient {
                self.context_add(Context::Ambient, |p| {
                    p.parse_property_or_method_declaration(span, r#type, &modifiers, decorators)
                })
            } else {
                self.parse_property_or_method_declaration(span, r#type, &modifiers, decorators)
            };
        }

        self.unexpected()
    }

    fn parse_class_element_name(&mut self, modifiers: &Modifiers) -> (PropertyKey<'a>, bool) {
        self.verify_modifiers(
            modifiers,
            ModifierKinds::all_except([ModifierKind::Const, ModifierKind::In, ModifierKind::Out]),
            false,
            |modifier, _| {
                match modifier.kind {
                    ModifierKind::Const => diagnostics::const_class_member(modifier.span()),
                    ModifierKind::In | ModifierKind::Out => {
                        diagnostics::can_only_appear_on_a_type_parameter_of_a_class_interface_or_type_alias(modifier.kind, modifier.span())
                    }
                    _ => unreachable!(),
                }
            },
        );

        match self.cur_kind() {
            Kind::PrivateIdentifier => {
                let private_ident = self.parse_private_identifier();
                // `private #foo`, etc. is illegal
                if self.is_ts {
                    self.verify_modifiers(
                        modifiers,
                        ModifierKinds::all_except([
                            ModifierKind::Public,
                            ModifierKind::Private,
                            ModifierKind::Protected,
                        ]),
                        false,
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
    pub(crate) fn parse_class_static_block(&mut self, span: u32) -> ClassElement<'a> {
        self.bump_any(); // bump `static`
        let block = self.context(
            Context::Await | Context::NewTarget,
            Context::Yield | Context::Return,
            Self::parse_block,
        );
        ClassElement::new_static_block(self.end_span(span), block.unbox().body, self)
    }

    /// <https://github.com/tc39/proposal-decorators>
    pub(crate) fn parse_class_accessor_property(
        &mut self,
        span: u32,
        key: PropertyKey<'a>,
        computed: bool,
        definite: Option<u32>,
        modifiers: &Modifiers,
        decorators: ArenaVec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        let type_annotation = if self.is_ts { self.parse_ts_type_annotation() } else { None };
        // `new.target` is allowed in a class accessor field initializer.
        let value = self.eat(Kind::Eq).then(|| {
            self.context_add(Context::NewTarget, Self::parse_assignment_expression_or_higher)
        });
        self.asi();
        let r#type = if modifiers.contains(ModifierKind::Abstract) {
            AccessorPropertyType::TSAbstractAccessorProperty
        } else {
            AccessorPropertyType::AccessorProperty
        };
        self.verify_modifiers(
            modifiers,
            ModifierKinds::new([
                ModifierKind::Public,
                ModifierKind::Private,
                ModifierKind::Protected,
                ModifierKind::Accessor,
                ModifierKind::Static,
                ModifierKind::Abstract,
                ModifierKind::Override,
            ]),
            true,
            diagnostics::accessor_modifier,
        );
        if let Some(definite_token_start) = definite
            && !modifiers.contains(ModifierKind::Declare)
        {
            let definite_span = Span::sized(definite_token_start, 1);
            if value.is_some() {
                self.error(diagnostics::variable_declarator_definite(definite_span));
            } else if type_annotation.is_none() {
                self.error(diagnostics::variable_declarator_definite_type_assertion(definite_span));
            } else if self.ctx.has_ambient()
                || modifiers.contains(ModifierKind::Static)
                || r#type.is_abstract()
            {
                self.error(diagnostics::definite_assignment_assertion_not_permitted(definite_span));
            }
        }
        ClassElement::new_accessor_property(
            self.end_span(span),
            r#type,
            decorators,
            key,
            type_annotation,
            value,
            computed,
            modifiers.contains(ModifierKind::Static),
            modifiers.contains(ModifierKind::Override),
            definite.is_some(),
            modifiers.accessibility(),
            self,
        )
    }

    fn parse_accessor_declaration(
        &mut self,
        span: u32,
        r#type: MethodDefinitionType,
        kind: MethodDefinitionKind,
        modifiers: &Modifiers,
        decorators: ArenaVec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        let (name, computed) = self.parse_class_element_name(modifiers);
        let mut value =
            self.with_ets_this_return_type(!modifiers.contains(ModifierKind::Static), |p| {
                p.parse_method(
                    modifiers.contains(ModifierKind::Async),
                    false,
                    FunctionKind::ClassMethod,
                )
            });
        if self.source_type.is_ets_static() {
            value.r#final = modifiers.contains(ModifierKind::Final);
            value.native = modifiers.contains(ModifierKind::Native);
        }
        let mut method_definition = MethodDefinition::boxed(
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
            self,
        );
        if self.source_type.is_ets_static() {
            method_definition.r#final = modifiers.contains(ModifierKind::Final);
            method_definition.native = modifiers.contains(ModifierKind::Native);
            self.check_ets_class_method_modifiers(
                modifiers,
                MethodDefinitionKind::Get,
                method_definition.value.body.is_some(),
            );
        }
        self.check_method_definition_accessor(&method_definition);
        self.verify_modifiers(
            modifiers,
            ModifierKinds::all_except([ModifierKind::Async, ModifierKind::Declare]),
            false,
            diagnostics::modifier_cannot_be_used_here,
        );
        ClassElement::MethodDefinition(method_definition)
    }

    pub(crate) fn parse_constructor_declaration(
        &mut self,
        span: u32,
        r#type: MethodDefinitionType,
        name: PropertyKey<'a>,
        modifiers: &Modifiers,
        decorators: ArenaVec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        if let Some(modifier) = modifiers.get(ModifierKind::Declare) {
            self.error(diagnostics::declare_constructor(modifier.span()));
        }

        let mut value = self.parse_method(
            modifiers.contains(ModifierKind::Async),
            false,
            FunctionKind::Constructor,
        );
        if self.source_type.is_ets_static() {
            value.r#final = modifiers.contains(ModifierKind::Final);
            value.native = modifiers.contains(ModifierKind::Native);
        }
        let mut method_definition = MethodDefinition::boxed(
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
            self,
        );
        if self.source_type.is_ets_static() {
            method_definition.r#final = modifiers.contains(ModifierKind::Final);
            method_definition.native = modifiers.contains(ModifierKind::Native);
            self.check_ets_class_method_modifiers(
                modifiers,
                MethodDefinitionKind::Constructor,
                method_definition.value.body.is_some(),
            );
        }
        self.check_method_definition_constructor(&method_definition);
        ClassElement::MethodDefinition(method_definition)
    }

    pub(crate) fn parse_constructor_name(&mut self) -> Option<PropertyKey<'a>> {
        if self.at(Kind::Constructor) {
            let ident = self.parse_identifier_name();
            if self.source_type.is_ets_static()
                && self.cur_kind().is_binding_identifier()
                && self.lexer.peek_token().kind() == Kind::LParen
            {
                let named_constructor = self.parse_identifier_name();
                return Some(PropertyKey::StaticIdentifier(self.alloc(named_constructor)));
            }
            return Some(PropertyKey::StaticIdentifier(self.alloc(ident)));
        }
        if self.at(Kind::Str)
            && self.cur_string() == "constructor"
            && self.lexer.peek_token().kind() == Kind::LParen
        {
            let string_literal = self.parse_literal_string();
            return Some(PropertyKey::StringLiteral(self.alloc(string_literal)));
        }
        None
    }

    fn parse_property_or_method_declaration(
        &mut self,
        span: u32,
        r#type: MethodDefinitionType,
        modifiers: &Modifiers,
        decorators: ArenaVec<'a, Decorator<'a>>,
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
            self.verify_modifiers(
                modifiers,
                ModifierKinds::all_except([ModifierKind::Declare, ModifierKind::Readonly]),
                false,
                |modifier, _| {
                    const ALLOWED: ModifierKinds = ModifierKinds::new([
                        ModifierKind::Public,
                        ModifierKind::Private,
                        ModifierKind::Protected,
                        ModifierKind::Static,
                        ModifierKind::Abstract,
                        ModifierKind::Override,
                        ModifierKind::Async,
                    ]);

                    match modifier.kind {
                        ModifierKind::Declare => {
                            diagnostics::cannot_appear_on_class_elements(modifier, Some(ALLOWED))
                        }
                        ModifierKind::Readonly => {
                            diagnostics::modifier_only_on_property_declaration_or_index_signature(
                                modifier,
                                Some(ALLOWED),
                            )
                        }
                        _ => unreachable!(),
                    }
                },
            );
            return self.parse_method_declaration(
                span, r#type, generator, name, computed, optional, modifiers, decorators,
            );
        }

        let is_definite = self.eat(Kind::Bang);
        let definite = is_definite.then_some(self.prev_token_end - 1);

        if is_definite && let Some(optional_span) = optional_span {
            self.error(diagnostics::optional_definite_property(optional_span.expand_right(1)));
        }

        if modifiers.contains(ModifierKind::Accessor) {
            if let Some(optional_span) = optional_span {
                self.error(diagnostics::optional_accessor_property(optional_span));
            }
            if name.is_specific_string_literal("constructor") && !computed {
                self.error(diagnostics::constructor_accessor(name.span()));
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
        modifiers: &Modifiers,
        decorators: ArenaVec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        let is_arkui_dsl_method =
            self.source_type.is_arkui() && self.decorators_enable_arkui_dsl(decorators.as_slice());
        let mut value =
            self.with_ets_this_return_type(!modifiers.contains(ModifierKind::Static), |p| {
                if is_arkui_dsl_method {
                    p.next_function_in_arkui_dsl(|p| {
                        p.parse_method(
                            modifiers.contains(ModifierKind::Async),
                            generator,
                            FunctionKind::ClassMethod,
                        )
                    })
                } else {
                    p.parse_method(
                        modifiers.contains(ModifierKind::Async),
                        generator,
                        FunctionKind::ClassMethod,
                    )
                }
            });
        if self.source_type.is_ets_static() {
            value.r#final = modifiers.contains(ModifierKind::Final);
            value.native = modifiers.contains(ModifierKind::Native);
        }
        let mut method_definition = MethodDefinition::boxed(
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
            self,
        );
        if self.source_type.is_ets_static() {
            method_definition.r#final = modifiers.contains(ModifierKind::Final);
            method_definition.native = modifiers.contains(ModifierKind::Native);
            self.check_ets_class_method_modifiers(
                modifiers,
                MethodDefinitionKind::Method,
                method_definition.value.body.is_some(),
            );
            if method_definition.key.is_specific_static_name("new") {
                self.error(diagnostics::ets_unsupported_syntax(
                    "Constructor signatures in classes",
                    method_definition.key.span(),
                ));
            }
        }
        self.check_method_definition_method(&method_definition);
        ClassElement::MethodDefinition(method_definition)
    }

    fn parse_property_declaration(
        &mut self,
        span: u32,
        name: PropertyKey<'a>,
        computed: bool,
        optional_span: Option<Span>,
        definite: Option<u32>,
        modifiers: &Modifiers,
        decorators: ArenaVec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        let type_annotation = if self.is_ts { self.parse_ts_type_annotation() } else { None };
        // Initializer[+In, ?Yield, ?Await]opt
        // `new.target` is allowed in a class field initializer.
        let initializer = self.eat(Kind::Eq).then(|| {
            self.context(
                Context::In | Context::NewTarget,
                Context::Yield | Context::Await,
                Self::parse_expr,
            )
        });

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
        if !self.source_type.is_ets_static()
            && !computed
            && let Some((name, span)) = name.prop_name()
        {
            if name == "constructor" {
                self.error(diagnostics::field_constructor(span));
            }
            if r#static && name == "prototype" && !self.ctx.has_ambient() {
                self.error(diagnostics::static_prototype(span));
            }
        }
        if !self.source_type.is_ets_static() && r#abstract && name.is_private_identifier() {
            self.error(diagnostics::abstract_with_private_identifier(name.span()));
        }
        if !self.source_type.is_ets_static() && r#abstract && initializer.is_some() {
            let (name, span) = name.prop_name().unwrap_or_else(|| {
                let span = name.span();
                (&self.source_text[span], span)
            });
            self.error(diagnostics::abstract_property_cannot_have_initializer(name, span));
        }
        if !self.source_type.is_ets_static()
            && self.ctx.has_ambient()
            && let Some(initializer) = &initializer
            && !(modifiers.contains(ModifierKind::Readonly) && type_annotation.is_none())
        {
            self.error(diagnostics::initializers_not_allowed_in_ambient_contexts(
                initializer.span(),
            ));
        }
        if let Some(definite_token_start) = definite {
            let definite_span = Span::sized(definite_token_start, 1);
            if self.source_type.is_ets_static() && r#static {
                self.error(diagnostics::ets_unsupported_syntax(
                    "Static late-initialized fields",
                    definite_span,
                ));
            }
            if initializer.is_some() {
                self.error(diagnostics::variable_declarator_definite(definite_span));
            } else if type_annotation.is_none() {
                self.error(diagnostics::variable_declarator_definite_type_assertion(definite_span));
            } else if !self.source_type.is_ets_static()
                && (self.ctx.has_ambient() || r#static || r#abstract)
            {
                self.error(diagnostics::definite_assignment_assertion_not_permitted(definite_span));
            }
            if self.source_type.is_ets_static()
                && (optional_span.is_some() || modifiers.contains(ModifierKind::Readonly))
            {
                self.error(diagnostics::ets_unsupported_syntax(
                    "Optional or readonly late-initialized fields",
                    definite_span,
                ));
            }
        }
        if self.source_type.is_ets_static()
            && r#static
            && modifiers.contains(ModifierKind::Override)
        {
            self.error(diagnostics::ets_unsupported_syntax("Static override fields", name.span()));
        }
        ClassElement::new_property_definition(
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
            definite.is_some(),
            modifiers.contains(ModifierKind::Readonly),
            modifiers.accessibility(),
            self,
        )
    }

    #[cold]
    pub(crate) fn check_getter(&mut self, function: &Function<'a>) {
        if let Some(type_parameters) = &function.type_parameters {
            self.error(diagnostics::accessor_cannot_have_type_parameters(type_parameters.span));
        } else if !function.params.items.is_empty() {
            self.error(diagnostics::getter_parameters(function.params.span));
        }
    }

    #[cold]
    pub(crate) fn check_setter(&mut self, function: &Function<'a>) {
        if let Some(type_parameters) = &function.type_parameters {
            self.error(diagnostics::accessor_cannot_have_type_parameters(type_parameters.span));
        } else if function.params.parameters_count() != 1 {
            self.error(diagnostics::setter_with_parameters(
                function.params.span,
                function.params.parameters_count(),
            ));
        } else if let Some(rest) = &function.params.rest {
            self.error(diagnostics::setter_with_rest_parameter(rest.span));
        } else if self.is_ts {
            let param = function.params.items.first().unwrap();
            if let Some(return_type) = &function.return_type {
                self.error(diagnostics::a_set_accessor_cannot_have_a_return_type_annotation(
                    return_type.span(),
                ));
            } else if param.optional {
                self.error(diagnostics::setter_with_optional_parameter(param.span));
            } else if param.initializer.is_some() {
                self.error(diagnostics::setter_with_initializer(function.params.span));
            }
        }
    }

    fn check_method_definition(&mut self, method: &MethodDefinition<'a>) {
        if self.source_type.is_ets_static() {
            if self.ctx.has_ambient()
                && method.accessibility.is_some()
                && let Some(body) = &method.value.body
            {
                self.error(diagnostics::implementation_in_ambient(Span::empty(body.span.start)));
            }
            if !method.computed
                && method.r#static
                && method.key.is_specific_static_name("prototype")
            {
                self.error(diagnostics::ets_unsupported_syntax(
                    "Runtime prototype members",
                    method.key.span(),
                ));
            }
            return;
        }
        if method.r#type.is_abstract() && method.key.is_private_identifier() {
            self.error(diagnostics::abstract_with_private_identifier(method.key.span()));
        }

        if !method.computed
            && let Some((name, span)) = method.key.prop_name()
        {
            if method.r#static {
                if name == "prototype" && !self.ctx.has_ambient() {
                    self.error(diagnostics::static_prototype(span));
                }
            } else if name == "constructor" {
                if matches!(method.kind, MethodDefinitionKind::Get | MethodDefinitionKind::Set) {
                    self.error(diagnostics::constructor_getter_setter(span));
                }
                if method.value.r#async {
                    self.error(diagnostics::constructor_async(span));
                }
                if method.value.generator {
                    self.error(diagnostics::constructor_generator(span));
                }
                if method.r#type.is_abstract() {
                    self.error(diagnostics::illegal_abstract_modifier(span));
                }
            }
        }

        if self.ctx.has_ambient()
            && let Some(body) = &method.value.body
        {
            self.error(diagnostics::implementation_in_ambient(Span::empty(body.span.start)));
        }
    }

    fn check_method_definition_accessor(&mut self, method: &MethodDefinition<'a>) {
        self.check_method_definition(method);

        match method.kind {
            MethodDefinitionKind::Get => self.check_getter(&method.value),
            MethodDefinitionKind::Set => self.check_setter(&method.value),
            _ => {}
        }
        if !self.source_type.is_ets_static()
            && method.r#type.is_abstract()
            && method.value.body.is_some()
        {
            let (name, span) = method.key.prop_name().unwrap_or_else(|| {
                let span = method.key.span();
                (&self.source_text[span], span)
            });
            self.error(diagnostics::abstract_accessor_cannot_have_implementation(name, span));
        }
    }

    fn check_method_definition_method(&mut self, method: &MethodDefinition<'a>) {
        self.check_method_definition(method);

        if self.source_type.is_ets_static() {
            let expected = if method.key.is_specific_static_name("$_get") {
                Some(1)
            } else if method.key.is_specific_static_name("$_set") {
                Some(2)
            } else {
                None
            };
            if let Some(expected) = expected {
                let all_required = method.value.params.rest.is_none()
                    && method
                        .value
                        .params
                        .items
                        .iter()
                        .all(|param| !param.optional && param.initializer.is_none());
                if method.value.params.parameters_count() != expected || !all_required {
                    self.error(diagnostics::ets_unsupported_syntax(
                        "Invalid predefined index-access method signature",
                        method.key.span(),
                    ));
                }
            }
        }

        if !self.source_type.is_ets_static()
            && method.r#type.is_abstract()
            && method.value.body.is_some()
        {
            let (name, span) = method.key.prop_name().unwrap_or_else(|| {
                let span = method.key.span();
                (&self.source_text[span], span)
            });
            self.error(diagnostics::abstract_method_cannot_have_implementation(name, span));
        }
    }

    fn check_method_definition_constructor(&mut self, method: &MethodDefinition<'a>) {
        self.check_method_definition(method);

        if let Some(this_param) = &method.value.this_param {
            // class Foo { constructor(this: number) {} }
            self.error(diagnostics::ts_constructor_this_parameter(this_param.span));
        }
        if let Some(type_sig) = &method.value.type_parameters {
            // class Foo { constructor<T>(param: T ) {} }
            self.error(diagnostics::ts_constructor_type_parameter(type_sig.span));
        }
        if method.value.body.is_some()
            && let Some(return_type) = &method.value.return_type
        {
            self.error(diagnostics::constructor_return_type(return_type.span));
        }
    }

    fn check_ets_class_method_modifiers(
        &mut self,
        modifiers: &Modifiers,
        kind: MethodDefinitionKind,
        _has_body: bool,
    ) {
        debug_assert!(self.source_type.is_ets_static());

        let allowed = match kind {
            MethodDefinitionKind::Constructor => ModifierKinds::new([
                ModifierKind::Public,
                ModifierKind::Private,
                ModifierKind::Protected,
                ModifierKind::Native,
                ModifierKind::Declare,
            ]),
            MethodDefinitionKind::Get | MethodDefinitionKind::Set => ModifierKinds::new([
                ModifierKind::Public,
                ModifierKind::Private,
                ModifierKind::Protected,
                ModifierKind::Abstract,
                ModifierKind::Static,
                ModifierKind::Final,
                ModifierKind::Override,
                ModifierKind::Native,
            ]),
            MethodDefinitionKind::Method => ModifierKinds::new([
                ModifierKind::Public,
                ModifierKind::Private,
                ModifierKind::Protected,
                ModifierKind::Abstract,
                ModifierKind::Static,
                ModifierKind::Final,
                ModifierKind::Override,
                ModifierKind::Native,
                ModifierKind::Async,
                ModifierKind::Declare,
            ]),
        };
        for modifier in modifiers.iter() {
            if !allowed.contains(modifier.kind) {
                self.error(diagnostics::ets_modifier_not_allowed(
                    &modifier,
                    match kind {
                        MethodDefinitionKind::Constructor => "a constructor",
                        MethodDefinitionKind::Get | MethodDefinitionKind::Set => "an accessor",
                        MethodDefinitionKind::Method => "a class method",
                    },
                ));
            }
        }

        if modifiers.contains(ModifierKind::Async) {
            if let Some(modifier) = modifiers.get(ModifierKind::Native) {
                self.error(diagnostics::ets_unsupported_syntax(
                    "Async native methods",
                    modifier.span(),
                ));
            }
            if let Some(modifier) = modifiers.get(ModifierKind::Abstract) {
                self.error(diagnostics::ets_unsupported_syntax(
                    "Async abstract methods",
                    modifier.span(),
                ));
            }
        }
    }
}
