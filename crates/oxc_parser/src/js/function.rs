use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};

use super::FunctionKind;
use crate::{
    Context, ParserConfig as Config, ParserImpl, StatementContext, diagnostics,
    lexer::Kind,
    modifiers::{ModifierKind, ModifierKinds, Modifiers},
};

impl FunctionKind {
    pub(crate) fn is_id_required(self) -> bool {
        matches!(self, Self::Declaration)
    }

    pub(crate) fn is_expression(self) -> bool {
        self == Self::Expression
    }
}

impl<'a, C: Config> ParserImpl<'a, C> {
    pub(crate) fn with_ets_this_return_type<F, T>(&mut self, allow: bool, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let saved = self.state.ets_allow_this_return_type;
        self.state.ets_allow_this_return_type = self.source_type.is_ets_static() && allow;
        let result = f(self);
        self.state.ets_allow_this_return_type = saved;
        result
    }

    pub(crate) fn at_function_with_async(&mut self) -> bool {
        self.at(Kind::Function)
            || self.at(Kind::Async) && {
                let token = self.lexer.peek_token();
                token.kind() == Kind::Function && !token.is_on_new_line()
            }
    }

    pub(crate) fn parse_function_body(&mut self) -> ArenaBox<'a, FunctionBody<'a>> {
        let span = self.start_span();
        let opening_span = self.cur_token().span();
        self.expect(Kind::LCurly);

        // Add Return context, remove TopLevel context
        let saved_ets_loop_depth = self.state.ets_loop_depth;
        let saved_ets_switch_depth = self.state.ets_switch_depth;
        self.state.ets_loop_depth = 0;
        self.state.ets_switch_depth = 0;
        let (directives, statements) = self.context(Context::Return, Context::TopLevel, |p| {
            p.parse_directives_and_statements(/* in_ts_namespace_body */ false)
        });
        self.state.ets_loop_depth = saved_ets_loop_depth;
        self.state.ets_switch_depth = saved_ets_switch_depth;

        self.expect_closing(Kind::RCurly, opening_span);
        FunctionBody::boxed(self.end_span(span), directives, statements, self)
    }

    pub(crate) fn parse_formal_parameters(
        &mut self,
        func_kind: FunctionKind,
        params_kind: FormalParameterKind,
    ) -> (Option<TSThisParameter<'a>>, ArenaBox<'a, FormalParameters<'a>>) {
        let span = self.start_span();
        let opening_span = self.cur_token().span();
        self.expect(Kind::LParen);
        let this_param = if self.is_ts && self.at(Kind::This) {
            let param = self.parse_ts_this_parameter();
            self.bump(Kind::Comma);
            Some(param)
        } else {
            None
        };
        if self.source_type.is_ets_static()
            && this_param.is_some()
            && matches!(func_kind, FunctionKind::ClassMethod | FunctionKind::Constructor)
        {
            self.error(diagnostics::ets_unsupported_syntax(
                "Receiver parameters on class members",
                this_param.as_ref().unwrap().span,
            ));
        }
        let (list, rest) = self.parse_formal_parameters_list(func_kind, opening_span);
        self.expect(Kind::RParen);

        if self.source_type.is_ets_static() {
            let is_arrow = params_kind == FormalParameterKind::ArrowFormalParameters;
            let is_signature = params_kind == FormalParameterKind::Signature;
            for param in &list {
                if is_arrow && !param.pattern.is_binding_identifier() {
                    self.error(diagnostics::ets_unsupported_syntax(
                        "Destructuring lambda parameters",
                        param.pattern.span(),
                    ));
                }
                if (!is_arrow || param.optional || param.initializer.is_some())
                    && param.type_annotation.is_none()
                {
                    self.error(diagnostics::ets_unsupported_syntax(
                        "Parameters without an explicit type annotation in this context",
                        param.span,
                    ));
                }
                if is_signature && param.initializer.is_some() {
                    self.error(diagnostics::ets_unsupported_syntax(
                        "Default values in function type parameters",
                        param.span,
                    ));
                }
            }
            if let Some(rest) = &rest {
                let valid_rest_type = rest.type_annotation.as_ref().is_some_and(|annotation| {
                    matches!(
                        annotation.type_annotation,
                        TSType::TSArrayType(_) | TSType::TSTupleType(_)
                    ) || matches!(
                        &annotation.type_annotation,
                        TSType::TSTypeReference(reference)
                            if matches!(
                                &reference.type_name,
                                TSTypeName::IdentifierReference(identifier)
                                    if matches!(identifier.name.as_str(), "Array" | "FixedArray")
                            )
                    )
                });
                if !valid_rest_type {
                    self.error(diagnostics::ets_unsupported_syntax(
                        "Rest parameters whose type is not an array or tuple",
                        rest.span,
                    ));
                }
            }
        }

        let formal_parameters =
            FormalParameters::boxed(self.end_span(span), params_kind, list, rest, self);
        (this_param, formal_parameters)
    }

    fn parse_formal_parameters_list(
        &mut self,
        func_kind: FunctionKind,
        opening_span: Span,
    ) -> (ArenaVec<'a, FormalParameter<'a>>, Option<ArenaBox<'a, FormalParameterRest<'a>>>) {
        let mut list = ArenaVec::new_in(self);
        let mut rest: Option<ArenaBox<'a, FormalParameterRest<'a>>> = None;
        let mut first = true;
        let mut has_optional = false;

        loop {
            let kind = self.cur_kind();
            if kind == Kind::RParen
                || matches!(kind, Kind::Eof | Kind::Undetermined)
                || self.fatal_error.is_some()
            {
                break;
            }

            if first {
                first = false;
            } else {
                let comma_span = self.cur_token().span();
                if kind != Kind::Comma {
                    let error = diagnostics::expect_closing_or_separator(
                        Kind::RParen.to_str(),
                        Kind::Comma.to_str(),
                        kind.to_str(),
                        comma_span,
                        opening_span,
                    );
                    self.set_fatal_error(error);
                    break;
                }
                self.bump_any();
                let kind = self.cur_kind();
                if kind == Kind::RParen {
                    if rest.is_some() && !self.ctx.has_ambient() {
                        self.error(diagnostics::rest_element_trailing_comma(comma_span));
                    }
                    break;
                }
            }

            if let Some(r) = &rest {
                self.set_fatal_error(diagnostics::rest_parameter_last(
                    r.type_annotation.as_ref().map_or_else(
                        || r.rest.span,
                        |type_annotation| r.rest.span.merge(type_annotation.span()),
                    ),
                ));
                break;
            }

            let span = self.start_span();
            let decorators = self.parse_decorators();

            if self.at(Kind::Dot3) {
                let rest_element = self.parse_rest_element_for_formal_parameter();
                let type_annotation =
                    if self.is_ts { self.parse_ts_type_annotation() } else { None };

                let are_decorators_allowed =
                    matches!(func_kind, FunctionKind::ClassMethod | FunctionKind::Constructor)
                        && self.is_ts;
                if !are_decorators_allowed {
                    for decorator in &decorators {
                        self.error(diagnostics::decorators_are_not_valid_here(decorator.span));
                    }
                }

                rest = Some(FormalParameterRest::boxed(
                    self.end_span(span),
                    decorators,
                    rest_element,
                    type_annotation,
                    self,
                ));
            } else {
                let param =
                    self.parse_formal_parameter_with_decorators(func_kind, span, decorators);
                if param.optional
                    || (self.source_type.is_ets_static() && param.initializer.is_some())
                {
                    has_optional = true;
                } else if has_optional && param.initializer.is_none() {
                    self.error(diagnostics::required_parameter_after_optional_parameter(
                        param.span,
                    ));
                }
                list.push(param);
            }
        }

        (list, rest)
    }

    fn parse_formal_parameter_with_decorators(
        &mut self,
        func_kind: FunctionKind,
        span: u32,
        decorators: ArenaVec<'a, Decorator<'a>>,
    ) -> FormalParameter<'a> {
        let modifiers = self.parse_modifiers(false, false);
        if self.source_type.is_ets_static() {
            for modifier in modifiers.iter() {
                self.error(diagnostics::ets_unsupported_syntax(
                    "Parameter-property modifiers",
                    modifier.span(),
                ));
            }
        }
        if self.is_ts {
            let allowed_modifiers = if func_kind == FunctionKind::Constructor {
                ModifierKinds::new([
                    ModifierKind::Public,
                    ModifierKind::Private,
                    ModifierKind::Protected,
                    ModifierKind::Override,
                    ModifierKind::Readonly,
                ])
            } else {
                ModifierKinds::none()
            };
            self.verify_modifiers(
                &modifiers,
                allowed_modifiers,
                true,
                diagnostics::cannot_appear_on_a_parameter,
            );
        } else {
            self.verify_modifiers(
                &modifiers,
                ModifierKinds::none(),
                true,
                diagnostics::parameter_modifiers_in_ts,
            );
        }
        let pattern = self.parse_binding_pattern();

        let optional = self.is_ts && self.eat(Kind::Question);
        let type_annotation = self.parse_ts_type_annotation();

        // Now parse the initializer if present
        let init = if self.eat(Kind::Eq) {
            let init =
                self.context_add(Context::In, ParserImpl::parse_assignment_expression_or_higher);
            if optional {
                self.error(diagnostics::a_parameter_cannot_have_question_mark_and_initializer(
                    pattern.span(),
                ));
            }
            Some(init)
        } else {
            None
        };

        let is_parameter_property = modifiers.contains_accessibility()
            || modifiers.contains_readonly()
            || modifiers.contains_override();
        if is_parameter_property {
            if let Some(ident) = pattern.get_binding_identifier() {
                if func_kind == FunctionKind::Constructor && ident.name == "constructor" {
                    self.error(diagnostics::constructor_cannot_be_parameter_property_name(
                        ident.span,
                    ));
                }
            } else {
                self.error(diagnostics::parameter_property_cannot_be_binding_pattern(Span::new(
                    span,
                    self.prev_token_end,
                )));
            }
        }

        let are_decorators_allowed =
            matches!(func_kind, FunctionKind::ClassMethod | FunctionKind::Constructor)
                && self.is_ts;
        if !are_decorators_allowed {
            for decorator in &decorators {
                self.error(diagnostics::decorators_are_not_valid_here(decorator.span));
            }
        }
        FormalParameter::new(
            self.end_span(span),
            decorators,
            pattern,
            type_annotation,
            init,
            optional,
            modifiers.accessibility(),
            modifiers.contains_readonly(),
            modifiers.contains_override(),
            self,
        )
    }

    pub(crate) fn parse_function(
        &mut self,
        span: u32,
        id: Option<BindingIdentifier<'a>>,
        r#async: bool,
        generator: bool,
        func_kind: FunctionKind,
        param_kind: FormalParameterKind,
        modifiers: &Modifiers,
        decorators: ArenaVec<'a, Decorator<'a>>,
    ) -> ArenaBox<'a, Function<'a>> {
        if self.source_type.is_ets_static() && generator {
            self.error(diagnostics::ets_unsupported_syntax(
                "Generator functions",
                Span::empty(span),
            ));
        }
        if self.source_type.is_ets_static() && r#async {
            if let Some(native) = modifiers.get(ModifierKind::Native) {
                self.error(diagnostics::ets_unsupported_syntax(
                    "Async native functions",
                    native.span(),
                ));
            }
            if let Some(declare) = modifiers.get(ModifierKind::Declare) {
                self.error(diagnostics::ets_unsupported_syntax(
                    "Async ambient functions",
                    declare.span(),
                ));
            }
        }
        let ctx = self.ctx;
        // `new.target` is allowed in a function's parameters and body (but not arrow
        // functions, which are parsed via `parse_function_body` directly).
        self.ctx =
            self.ctx.and_in(true).and_await(r#async).and_yield(generator).and_new_target(true);
        let type_parameters = self.parse_ts_type_parameters();
        let (this_param, params) = self.parse_formal_parameters(func_kind, param_kind);
        let allow_this_return_type = self.state.ets_allow_this_return_type || this_param.is_some();
        let return_type = if self.is_ts {
            if self.source_type.is_ets_static() && allow_this_return_type {
                self.context_add(Context::EtsAllowThisType, Self::parse_ts_return_type_annotation)
            } else {
                self.parse_ts_return_type_annotation()
            }
        } else {
            None
        };
        let body = if self.at(Kind::LCurly) || func_kind == FunctionKind::Expression {
            let is_arkui_dsl_function = self.take_next_arkui_dsl_function()
                || self.source_type.is_arkui()
                    && self.decorators_enable_arkui_dsl(decorators.as_slice());
            Some(if is_arkui_dsl_function {
                self.in_arkui_dsl_context(Self::parse_function_body)
            } else {
                self.without_arkui_dsl_context(Self::parse_function_body)
            })
        } else {
            None
        };
        self.ctx = self
            .ctx
            .and_in(ctx.has_in())
            .and_await(ctx.has_await())
            .and_yield(ctx.has_yield())
            .and_new_target(ctx.has_new_target());
        if (!self.is_ts || matches!(func_kind, FunctionKind::ObjectMethod)) && body.is_none() {
            return self.fatal_error(diagnostics::expect_function_body(self.end_span(span)));
        }
        let function_type = match func_kind {
            FunctionKind::Declaration | FunctionKind::DefaultExport => {
                if body.is_none() {
                    FunctionType::TSDeclareFunction
                } else {
                    FunctionType::FunctionDeclaration
                }
            }
            FunctionKind::Expression
            | FunctionKind::ClassMethod
            | FunctionKind::Constructor
            | FunctionKind::ObjectMethod => {
                if body.is_none() {
                    FunctionType::TSEmptyBodyFunctionExpression
                } else {
                    FunctionType::FunctionExpression
                }
            }
            FunctionKind::TSDeclaration => FunctionType::TSDeclareFunction,
        };

        if FunctionType::TSDeclareFunction == function_type
            || FunctionType::TSEmptyBodyFunctionExpression == function_type
        {
            // Static ETS permits comma-separated ambient/interface members.
            // Keep TypeScript's ASI behavior unchanged in every other mode.
            if !(self.source_type.is_ets_static() && self.eat(Kind::Comma)) {
                self.asi();
            }
        }

        // A function declaration's implementation (body) cannot be declared in an ambient context,
        // whether the ambient context comes from the function's own `declare` modifier or is
        // inherited from an enclosing `declare module`/`declare namespace` or a `.d.ts` file
        // (TS1183). Class methods are checked separately in `check_method_definition`, so they are
        // excluded here to avoid a duplicate diagnostic.
        if ctx.has_ambient()
            && (!self.source_type.is_ets_static() || !modifiers.contains_declare())
            && matches!(
                func_kind,
                FunctionKind::Declaration
                    | FunctionKind::DefaultExport
                    | FunctionKind::TSDeclaration
            )
            && let Some(body) = &body
        {
            self.error(diagnostics::implementation_in_ambient(Span::empty(body.span.start)));
        }

        if generator && !self.source_type.is_ets_static() {
            if ctx.has_ambient() {
                self.error(diagnostics::generator_in_ambient_context(self.end_span(span)));
            } else if body.is_none() {
                self.error(diagnostics::overload_signature_generator(self.end_span(span)));
            }
        }
        self.verify_modifiers(
            modifiers,
            ModifierKinds::new([ModifierKind::Declare, ModifierKind::Async]),
            true,
            diagnostics::modifier_cannot_be_used_here,
        );

        let mut function = Function::boxed_with_decorators(
            self.end_span(span),
            function_type,
            decorators,
            id,
            generator,
            r#async,
            modifiers.contains_declare(),
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            self,
        );
        if self.source_type.is_ets_static() {
            function.r#final = modifiers.contains(ModifierKind::Final);
            function.native = modifiers.contains(ModifierKind::Native);
        }
        function
    }

    /// [Function Declaration](https://tc39.es/ecma262/#prod-FunctionDeclaration)
    pub(crate) fn parse_function_declaration(
        &mut self,
        span: u32,
        r#async: bool,
        stmt_ctx: StatementContext,
        decorators: ArenaVec<'a, Decorator<'a>>,
    ) -> Statement<'a> {
        if self.source_type.is_ets_static() && !self.state.ets_in_declaration_scope {
            self.error(diagnostics::ets_nested_declaration("Function", Span::empty(span)));
        }
        let func_kind = FunctionKind::Declaration;
        let decl = self.parse_function_impl(span, r#async, func_kind, decorators);
        if stmt_ctx.is_single_statement() {
            if decl.r#async {
                self.error(diagnostics::async_function_declaration(Span::new(
                    decl.span.start,
                    decl.params.span.end,
                )));
            } else if decl.generator {
                self.error(diagnostics::generator_function_declaration(Span::new(
                    decl.span.start,
                    decl.params.span.end,
                )));
            }
        }
        Statement::FunctionDeclaration(decl)
    }

    /// Parse function implementation in Javascript, cursor
    /// at `function` or `async function`
    pub(crate) fn parse_function_impl(
        &mut self,
        span: u32,
        r#async: bool,
        func_kind: FunctionKind,
        decorators: ArenaVec<'a, Decorator<'a>>,
    ) -> ArenaBox<'a, Function<'a>> {
        self.expect(Kind::Function);
        let generator = self.eat(Kind::Star);
        let id = self.parse_function_id(func_kind, r#async, generator);
        self.parse_function(
            span,
            id,
            r#async,
            generator,
            func_kind,
            FormalParameterKind::FormalParameter,
            &Modifiers::empty(),
            decorators,
        )
    }

    /// Parse function implementation in Typescript, cursor
    /// at `function`
    pub(crate) fn parse_ts_function_impl(
        &mut self,
        start_span: u32,
        func_kind: FunctionKind,
        modifiers: &Modifiers,
        decorators: ArenaVec<'a, Decorator<'a>>,
    ) -> ArenaBox<'a, Function<'a>> {
        let r#async = modifiers.contains(ModifierKind::Async);
        self.expect(Kind::Function);
        let generator = self.eat(Kind::Star);
        let id = self.parse_function_id(func_kind, r#async, generator);
        self.parse_function(
            start_span,
            id,
            r#async,
            generator,
            func_kind,
            FormalParameterKind::FormalParameter,
            modifiers,
            decorators,
        )
    }

    /// [Function Expression](https://tc39.es/ecma262/#prod-FunctionExpression)
    pub(crate) fn parse_function_expression(&mut self, span: u32, r#async: bool) -> Expression<'a> {
        let func_kind = FunctionKind::Expression;
        self.expect(Kind::Function);

        let generator = self.eat(Kind::Star);
        let id = self.parse_function_id(func_kind, r#async, generator);
        let function = self.parse_function(
            span,
            id,
            r#async,
            generator,
            func_kind,
            FormalParameterKind::FormalParameter,
            &Modifiers::empty(),
            ArenaVec::new_in(self), // decorators
        );
        Expression::FunctionExpression(function)
    }

    /// Section 15.4 Method Definitions
    /// `ClassElementName` ( `UniqueFormalParameters` ) { `FunctionBody` }
    /// * `GeneratorMethod`
    ///   * `ClassElementName`
    /// * `AsyncMethod`
    ///   async `ClassElementName`
    /// * `AsyncGeneratorMethod`
    ///   async * `ClassElementName`
    pub(crate) fn parse_method(
        &mut self,
        r#async: bool,
        generator: bool,
        func_kind: FunctionKind,
    ) -> ArenaBox<'a, Function<'a>> {
        let span = self.start_span();
        self.parse_function(
            span,
            None,
            r#async,
            generator,
            func_kind,
            FormalParameterKind::UniqueFormalParameters,
            &Modifiers::empty(),
            ArenaVec::new_in(self), // decorators
        )
    }

    /// Section 15.5 Yield Expression
    /// yield
    /// yield [no `LineTerminator` here] `AssignmentExpression`
    /// yield [no `LineTerminator` here] * `AssignmentExpression`
    pub(crate) fn parse_yield_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        self.bump_any(); // advance `yield`

        let has_yield = self.ctx.has_yield();
        if !has_yield {
            self.error(diagnostics::yield_expression(Span::sized(span, 5)));
        }

        let mut delegate = false;
        let mut argument = None;

        if !self.cur_token().is_on_new_line() {
            delegate = self.eat(Kind::Star);
            let not_assignment_expr = matches!(
                self.cur_kind(),
                Kind::Semicolon
                    | Kind::Eof
                    | Kind::RCurly
                    | Kind::RParen
                    | Kind::RBrack
                    | Kind::Colon
                    | Kind::Comma
            );
            if !not_assignment_expr || delegate {
                self.ctx = self.ctx.union_yield_if(true);
                argument = Some(self.parse_assignment_expression_or_higher());
                self.ctx = self.ctx.and_yield(has_yield);
            }
        }

        Expression::new_yield_expression(self.end_span(span), delegate, argument, self)
    }

    // id: None - for AnonymousDefaultExportedFunctionDeclaration
    pub(crate) fn parse_function_id(
        &mut self,
        func_kind: FunctionKind,
        r#async: bool,
        generator: bool,
    ) -> Option<BindingIdentifier<'a>> {
        let kind = self.cur_kind();
        if kind.is_binding_identifier() {
            let mut ctx = self.ctx;
            if func_kind.is_expression() {
                ctx = ctx.and_await(r#async).and_yield(generator);
            }
            self.check_identifier(kind, ctx);

            let (span, name) = self.parse_identifier_kind(Kind::Ident);
            Some(BindingIdentifier::new(span, name, self))
        } else {
            if func_kind.is_id_required() {
                match self.cur_kind() {
                    Kind::LParen => {
                        self.error(diagnostics::expect_function_name(self.cur_token().span()));
                    }
                    kind if kind.is_reserved_keyword() => self.expect_without_advance(Kind::Ident),
                    _ => {}
                }
            }

            None
        }
    }
}
