use oxc_allocator::{ArenaBox, ArenaVec, Slot, SlotFilled};
use oxc_ast::{ast::*, builder::builders::traits::SlotBuild};
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
    pub(crate) fn at_function_with_async(&mut self) -> bool {
        self.at(Kind::Function)
            || self.at(Kind::Async) && {
                let token = self.lexer.peek_token();
                token.kind() == Kind::Function && !token.is_on_new_line()
            }
    }

    pub(crate) fn parse_function_body(&mut self) -> ArenaBox<'a, FunctionBody<'a>> {
        FunctionBody::uninit(self).fill_with(|slot| self.parse_function_body_into(slot))
    }

    fn parse_function_body_into<'slot>(
        &mut self,
        slot: Slot<'slot, FunctionBody<'a>>,
    ) -> SlotFilled<'slot> {
        let start = self.cur_start();
        let opening_span = self.cur_token().span();
        self.expect(Kind::LCurly);

        // Add Return context, remove TopLevel context
        let (directives, statements) = self.context(Context::Return, Context::TopLevel, |p| {
            p.parse_directives_and_statements(/* in_ts_namespace_body */ false)
        });

        self.expect_closing(Kind::RCurly, opening_span);
        slot.build(self)
            .span_start(start)
            .directives(directives)
            .statements(statements)
            .span_end(self.end_span(start).end)
            .finish()
    }

    pub(crate) fn parse_formal_parameters(
        &mut self,
        func_kind: FunctionKind,
        params_kind: FormalParameterKind,
    ) -> (Option<ArenaBox<'a, TSThisParameter<'a>>>, ArenaBox<'a, FormalParameters<'a>>) {
        let mut this_param = None;
        let formal_parameters = FormalParameters::uninit(self).fill_with(|slot| {
            self.parse_formal_parameters_into(slot, func_kind, params_kind, &mut this_param)
        });
        (this_param, formal_parameters)
    }

    fn parse_formal_parameters_into<'slot>(
        &mut self,
        slot: Slot<'slot, FormalParameters<'a>>,
        func_kind: FunctionKind,
        params_kind: FormalParameterKind,
        this_param: &mut Option<ArenaBox<'a, TSThisParameter<'a>>>,
    ) -> SlotFilled<'slot> {
        let start = self.cur_start();
        let opening_span = self.cur_token().span();
        self.expect(Kind::LParen);
        if self.is_ts && self.at(Kind::This) {
            let param = self.parse_ts_this_parameter();
            self.bump(Kind::Comma);
            *this_param = Some(param);
        }
        let mut rest = None;
        slot.build(self)
            .span_start(start)
            .kind(params_kind)
            .items_with(|slot| {
                let mut items = ArenaVec::new_in(self);
                rest = self.parse_formal_parameters_list_into(&mut items, func_kind, opening_span);
                slot.fill(items)
            })
            .rest(rest)
            .span_end({
                self.expect(Kind::RParen);
                self.end_span(start).end
            })
            .finish()
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn parse_formal_parameters_list_into(
        &mut self,
        list: &mut ArenaVec<'a, FormalParameter<'a>>,
        func_kind: FunctionKind,
        opening_span: Span,
    ) -> Option<ArenaBox<'a, FormalParameterRest<'a>>> {
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

            let start = self.cur_start();
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

                rest = Some(
                    FormalParameterRest::build(self)
                        .span_start(start)
                        .decorators(decorators)
                        .rest(rest_element)
                        .type_annotation(type_annotation)
                        .span_end(self.end_span(start).end)
                        .finish(),
                );
            } else {
                list.push_with(|slot| {
                    self.parse_formal_parameter_with_decorators_into(
                        slot,
                        func_kind,
                        start,
                        decorators,
                        &mut has_optional,
                    )
                });
            }
        }

        rest
    }

    fn parse_formal_parameter_with_decorators_into<'slot>(
        &mut self,
        slot: Slot<'slot, FormalParameter<'a>>,
        func_kind: FunctionKind,
        start: u32,
        decorators: ArenaVec<'a, Decorator<'a>>,
        has_optional: &mut bool,
    ) -> SlotFilled<'slot> {
        let modifiers = self.parse_modifiers(false, false);
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
            Some(ArenaBox::new_in(init, self))
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
                    start,
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
        let span = self.end_span(start);
        if optional {
            *has_optional = true;
        } else if *has_optional && init.is_none() {
            self.error(diagnostics::required_parameter_after_optional_parameter(span));
        }

        slot.build(self)
            .span(span)
            .decorators(decorators)
            .pattern(pattern)
            .type_annotation(type_annotation)
            .initializer(init)
            .optional(optional)
            .accessibility(modifiers.accessibility())
            .readonly(modifiers.contains_readonly())
            .r#override(modifiers.contains_override())
            .finish()
    }

    pub(crate) fn parse_function(
        &mut self,
        start: u32,
        id: Option<BindingIdentifier<'a>>,
        r#async: bool,
        generator: Option<u32>,
        func_kind: FunctionKind,
        param_kind: FormalParameterKind,
        modifiers: &Modifiers,
    ) -> ArenaBox<'a, Function<'a>> {
        let ctx = self.ctx;
        // `new.target` is allowed in a function's parameters and body (but not arrow
        // functions, which are parsed via `parse_function_body` directly).
        self.ctx = self
            .ctx
            .and_in(true)
            .and_await(r#async)
            .and_yield(generator.is_some())
            .and_new_target(true);
        let type_parameters = self.parse_ts_type_parameters();
        let mut this_param = None;
        let function = Function::build(self)
            .span_start(start)
            .id(id)
            .generator(generator.is_some())
            .r#async(r#async)
            .declare(modifiers.contains_declare())
            .type_parameters(type_parameters)
            .params_with(|slot| {
                self.parse_formal_parameters_into(
                    slot.into_contents(self),
                    func_kind,
                    param_kind,
                    &mut this_param,
                )
            })
            .this_param(this_param);
        let return_type = if self.is_ts { self.parse_ts_return_type_annotation() } else { None };
        let mut body_span = None;
        let parse_body = self.at(Kind::LCurly) || func_kind == FunctionKind::Expression;
        let function = function.return_type(return_type).body_with(|slot| {
            if parse_body {
                let start = self.cur_start();
                let filled = self.parse_function_body_into(slot.into_some().into_contents(self));
                body_span = Some(Span::new(start, self.prev_token_end));
                filled
            } else {
                slot.fill(None)
            }
        });
        self.ctx = self
            .ctx
            .and_in(ctx.has_in())
            .and_await(ctx.has_await())
            .and_yield(ctx.has_yield())
            .and_new_target(ctx.has_new_target());
        if (!self.is_ts || matches!(func_kind, FunctionKind::ObjectMethod)) && body_span.is_none() {
            return self.fatal_error(diagnostics::expect_function_body(self.end_span(start)));
        }
        let function_type = match func_kind {
            FunctionKind::Declaration | FunctionKind::DefaultExport => {
                if body_span.is_none() {
                    FunctionType::TSDeclareFunction
                } else {
                    FunctionType::FunctionDeclaration
                }
            }
            FunctionKind::Expression
            | FunctionKind::ClassMethod
            | FunctionKind::Constructor
            | FunctionKind::ObjectMethod => {
                if body_span.is_none() {
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
            self.asi();
        }

        // A function declaration's implementation (body) cannot be declared in an ambient context,
        // whether the ambient context comes from the function's own `declare` modifier or is
        // inherited from an enclosing `declare module`/`declare namespace` or a `.d.ts` file
        // (TS1183). Class methods are checked separately in `check_method_definition`, so they are
        // excluded here to avoid a duplicate diagnostic.
        if ctx.has_ambient()
            && matches!(
                func_kind,
                FunctionKind::Declaration
                    | FunctionKind::DefaultExport
                    | FunctionKind::TSDeclaration
            )
            && let Some(body_span) = body_span
        {
            self.error(diagnostics::implementation_in_ambient(Span::empty(body_span.start)));
        }

        if let Some(generator) = generator {
            if ctx.has_ambient() {
                self.error(diagnostics::generator_in_ambient_context(self.end_span(generator)));
            } else if body_span.is_none() {
                self.error(diagnostics::overload_signature_generator(self.end_span(start)));
            }
        }
        self.verify_modifiers(
            modifiers,
            ModifierKinds::new([ModifierKind::Declare, ModifierKind::Async]),
            true,
            diagnostics::modifier_cannot_be_used_here,
        );

        function.r#type(function_type).defaults().span_end(self.end_span(start).end).finish()
    }

    /// [Function Declaration](https://tc39.es/ecma262/#prod-FunctionDeclaration)
    pub(crate) fn parse_function_declaration(
        &mut self,
        start: u32,
        r#async: bool,
        stmt_ctx: StatementContext,
    ) -> Statement<'a> {
        let func_kind = FunctionKind::Declaration;
        let decl = self.parse_function_impl(start, r#async, func_kind);
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
        start: u32,
        r#async: bool,
        func_kind: FunctionKind,
    ) -> ArenaBox<'a, Function<'a>> {
        self.expect(Kind::Function);
        let generator = self.eat(Kind::Star).then_some(self.prev_token_end - 1);
        let id = self.parse_function_id(func_kind, r#async, generator.is_some());
        self.parse_function(
            start,
            id,
            r#async,
            generator,
            func_kind,
            FormalParameterKind::FormalParameter,
            &Modifiers::empty(),
        )
    }

    /// Parse function implementation in Typescript, cursor
    /// at `function`
    pub(crate) fn parse_ts_function_impl(
        &mut self,
        start: u32,
        func_kind: FunctionKind,
        modifiers: &Modifiers,
    ) -> ArenaBox<'a, Function<'a>> {
        let r#async = modifiers.contains(ModifierKind::Async);
        self.expect(Kind::Function);
        let generator = self.eat(Kind::Star).then_some(self.prev_token_end - 1);
        let id = self.parse_function_id(func_kind, r#async, generator.is_some());
        self.parse_function(
            start,
            id,
            r#async,
            generator,
            func_kind,
            FormalParameterKind::FormalParameter,
            modifiers,
        )
    }

    /// [Function Expression](https://tc39.es/ecma262/#prod-FunctionExpression)
    pub(crate) fn parse_function_expression(
        &mut self,
        start: u32,
        r#async: bool,
    ) -> Expression<'a> {
        let func_kind = FunctionKind::Expression;
        self.expect(Kind::Function);

        let generator = self.eat(Kind::Star).then_some(self.prev_token_end - 1);
        let id = self.parse_function_id(func_kind, r#async, generator.is_some());
        let function = self.parse_function(
            start,
            id,
            r#async,
            generator,
            func_kind,
            FormalParameterKind::FormalParameter,
            &Modifiers::empty(),
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
        generator: Option<u32>,
        func_kind: FunctionKind,
    ) -> ArenaBox<'a, Function<'a>> {
        let start = self.cur_start();
        self.parse_function(
            start,
            None,
            r#async,
            generator,
            func_kind,
            FormalParameterKind::UniqueFormalParameters,
            &Modifiers::empty(),
        )
    }

    /// Section 15.5 Yield Expression
    /// yield
    /// yield [no `LineTerminator` here] `AssignmentExpression`
    /// yield [no `LineTerminator` here] * `AssignmentExpression`
    pub(crate) fn parse_yield_expression(&mut self) -> Expression<'a> {
        let start = self.cur_start();
        self.bump_any(); // advance `yield`

        let has_yield = self.ctx.has_yield();
        if !has_yield {
            self.error(diagnostics::yield_expression(Span::sized(start, 5)));
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

        Expression::new_yield_expression(self.end_span(start), delegate, argument, self)
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
