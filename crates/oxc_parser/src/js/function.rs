use oxc_allocator::Box;
use oxc_ast::{
    ast::*,
    context::{Context, StatementContext},
    GetNode, Node,
};
use oxc_diagnostics::{Diagnostic, Result};

use super::list::FormalParameterList;
use crate::lexer::Kind;
use crate::list::SeparatedList;
use crate::Parser;

type ArrowFunctionHead<'a> = (
    Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    Box<'a, FormalParameters<'a>>,
    Option<TSTypeAnnotation<'a>>,
    bool,
    Node,
);

#[derive(Debug, Copy, Clone)]
pub enum IsParenthesizedArrowFunction {
    True,
    False,
    Maybe,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum FunctionKind {
    Declaration { single_statement: bool },
    Expression,
    DefaultExport,
    TSDeclaration,
}

impl FunctionKind {
    pub const fn is_id_required(self) -> bool {
        matches!(self, Self::Declaration { single_statement: true })
    }

    pub fn is_expression(self) -> bool {
        self == Self::Expression
    }
}

impl<'a> Parser<'a> {
    pub fn at_function_with_async(&mut self) -> bool {
        self.at(Kind::Function)
            || self.at(Kind::Async)
                && self.peek_at(Kind::Function)
                && !self.peek_token().is_on_new_line
    }

    pub fn at_async_no_new_line(&mut self) -> bool {
        self.at(Kind::Async) && !self.cur_token().escaped && !self.peek_token().is_on_new_line
    }

    pub fn parse_function_body(&mut self) -> Result<Box<'a, FunctionBody<'a>>> {
        let node = self.start_node();
        self.expect(Kind::LCurly)?;

        // We may be in a [Decorator] context when parsing a function expression or
        // arrow function. The body of the function is not in [Decorator] context.
        let save_decorator_context = self.ctx.has_decorator();
        if save_decorator_context {
            self.ctx = self.ctx.and_decorator(false);
        }

        let (directives, statements) = self.with_context(Context::Return, |p| {
            p.parse_directives_and_statements(/* is_top_level */ false)
        })?;

        if save_decorator_context {
            self.ctx = self.ctx.and_decorator(true);
        }

        self.expect(Kind::RCurly)?;
        Ok(self.ast.function_body(self.end_node(node), directives, statements))
    }

    pub fn parse_formal_parameters(
        &mut self,
        params_kind: FormalParameterKind,
    ) -> Result<Box<'a, FormalParameters<'a>>> {
        let node = self.start_node();
        let elements = FormalParameterList::parse(self)?.elements;
        Ok(self.ast.formal_parameters(self.end_node(node), params_kind, elements))
    }

    pub fn parse_function(
        &mut self,
        node: Node,
        id: Option<BindingIdentifier>,
        r#async: bool,
        generator: bool,
        func_kind: FunctionKind,
    ) -> Result<Box<'a, Function<'a>>> {
        let has_await = self.ctx.has_await();
        let has_yield = self.ctx.has_yield();
        self.ctx = self.ctx.and_await(r#async).and_yield(generator);

        let type_parameters = self.parse_ts_type_parameters()?;

        let params = self.parse_formal_parameters(FormalParameterKind::FormalParameter)?;

        let return_type = self.parse_ts_return_type_annotation()?;

        let body = if self.at(Kind::LCurly) { Some(self.parse_function_body()?) } else { None };

        self.ctx = self.ctx.and_await(has_await).and_yield(has_yield);

        if !self.ts_enabled() && body.is_none() {
            return self.unexpected();
        }

        let function_type = if body.is_none() {
            FunctionType::TSDeclareFunction
        } else {
            match func_kind {
                FunctionKind::Declaration { .. } | FunctionKind::DefaultExport => {
                    FunctionType::FunctionDeclaration
                }
                FunctionKind::Expression { .. } => FunctionType::FunctionExpression,
                FunctionKind::TSDeclaration { .. } => FunctionType::TSDeclareFunction,
            }
        };

        if FunctionType::TSDeclareFunction == function_type {
            self.asi()?;
        }

        Ok(self.ast.function(
            function_type,
            self.end_node(node),
            id,
            false, // expression
            generator,
            r#async,
            params,
            body,
            type_parameters,
            return_type,
            false,
        ))
    }

    /// [Function Declaration](https://tc39.es/ecma262/#prod-FunctionDeclaration)
    pub fn parse_function_declaration(
        &mut self,
        stmt_ctx: StatementContext,
    ) -> Result<Statement<'a>> {
        let func_kind =
            FunctionKind::Declaration { single_statement: stmt_ctx.is_single_statement() };
        let decl = self.parse_function_impl(func_kind)?;
        if stmt_ctx.is_single_statement() {
            if decl.r#async {
                self.error(Diagnostic::AsyncFunctionDeclaration(Node::new(
                    decl.node.start,
                    decl.params.node.end,
                )));
            } else if decl.generator {
                self.error(Diagnostic::GeneratorFunctionDeclaration(Node::new(
                    decl.node.start,
                    decl.params.node.end,
                )));
            }
        }

        Ok(self.ast.function_declaration(decl))
    }

    pub fn parse_function_impl(
        &mut self,
        func_kind: FunctionKind,
    ) -> Result<Box<'a, Function<'a>>> {
        let node = self.start_node();
        let r#async = self.eat(Kind::Async);
        self.expect(Kind::Function)?;
        let generator = self.eat(Kind::Star);
        let id = self.parse_function_id(func_kind, r#async, generator);
        self.parse_function(node, id, r#async, generator, func_kind)
    }

    /// [Function Expression](https://tc39.es/ecma262/#prod-FunctionExpression)
    pub fn parse_function_expression(
        &mut self,
        node: Node,
        r#async: bool,
    ) -> Result<Expression<'a>> {
        let func_kind = FunctionKind::Expression;
        self.expect(Kind::Function)?;

        let save_decorator_context = self.ctx.has_decorator();
        self.ctx = self.ctx.and_decorator(false);

        let generator = self.eat(Kind::Star);
        let id = self.parse_function_id(func_kind, r#async, generator);
        let function = self.parse_function(node, id, r#async, generator, func_kind)?;

        self.ctx = self.ctx.and_decorator(save_decorator_context);

        Ok(self.ast.function_expression(function))
    }

    pub fn parse_single_param_function_expression(
        &mut self,
        node: Node,
        r#async: bool,
        generator: bool,
    ) -> Result<Expression<'a>> {
        let has_await = self.ctx.has_await();
        let has_yield = self.ctx.has_yield();

        self.ctx = self.ctx.union_await_if(r#async).union_yield_if(generator);
        let params_node = self.start_node();
        let param = self.parse_binding_identifier()?;
        let ident = self.ast.binding_identifier(param);
        let pattern = self.ast.binding_pattern(ident, None, false);
        let params_node = self.end_node(params_node);
        let formal_parameter = self.ast.formal_parameter(params_node, pattern, None, false, None);
        let params = self.ast.formal_parameters(
            params_node,
            FormalParameterKind::ArrowFormalParameters,
            self.ast.new_vec_single(formal_parameter),
        );

        self.expect(Kind::Arrow)?;

        self.ctx = self.ctx.and_await(r#async).and_yield(generator);
        let expression = !self.at(Kind::LCurly);
        let body = if expression {
            let expr = self.parse_assignment_expression_base()?;
            let node = expr.node();
            let expr_stmt = self.ast.expression_statement(node, expr);
            self.ast.function_body(node, self.ast.new_vec(), self.ast.new_vec_single(expr_stmt))
        } else {
            self.parse_function_body()?
        };
        self.ctx = self.ctx.and_await(has_await).and_yield(has_yield);

        Ok(self.ast.arrow_expression(
            self.end_node(node),
            expression,
            false,
            r#async,
            params,
            body,
            None,
            None,
        ))
    }

    /// Section 15.4 Method Definitions
    /// `ClassElementName` ( `UniqueFormalParameters` ) { `FunctionBody` }
    /// `GeneratorMethod`
    ///   * `ClassElementName`
    /// `AsyncMethod`
    ///   async `ClassElementName`
    /// `AsyncGeneratorMethod`
    ///   async * `ClassElementName`
    pub fn parse_method(
        &mut self,
        r#async: bool,
        generator: bool,
    ) -> Result<Box<'a, Function<'a>>> {
        let node = self.start_node();
        self.parse_function(node, None, r#async, generator, FunctionKind::Expression)
    }

    /// Section 15.5 Yield Expression
    /// yield
    /// yield [no `LineTerminator` here] `AssignmentExpression`
    /// yield [no `LineTerminator` here] * `AssignmentExpression`
    pub fn parse_yield_expression(&mut self) -> Result<Expression<'a>> {
        let node = self.start_node();
        self.bump_any(); // advance `yield`

        let mut delegate = false;
        let mut argument = None;

        if !self.cur_token().is_on_new_line {
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
                let has_yield = self.ctx.has_yield();
                self.ctx = self.ctx.union_yield_if(true);
                argument = Some(self.parse_assignment_expression_base()?);
                self.ctx = self.ctx.and_yield(has_yield);
            }
        }

        Ok(self.ast.yield_expression(self.end_node(node), delegate, argument))
    }

    // id: None - for AnonymousDefaultExportedFunctionDeclaration
    pub fn parse_function_id(
        &mut self,
        kind: FunctionKind,
        r#async: bool,
        generator: bool,
    ) -> Option<BindingIdentifier> {
        let ctx = self.ctx;
        if kind.is_expression() {
            self.ctx = self.ctx.and_await(r#async).and_yield(generator);
        }
        let id = self.cur_kind().is_binding_identifier().then(|| {
            let (node, name) = self.parse_identifier_kind(Kind::Ident);
            BindingIdentifier { node, name }
        });
        self.ctx = ctx;

        if kind.is_id_required() && id.is_none() {
            self.error(Diagnostic::ExpectFunctionName(self.cur_token().node()));
        }

        id
    }

    pub fn is_parenthesized_arrow_function_expression(
        &mut self,
        r#async: bool,
    ) -> IsParenthesizedArrowFunction {
        let offset = u8::from(r#async);

        match self.nth_kind(offset) {
            Kind::LParen => match self.nth_kind(offset + 1) {
                // '()' is an arrow expression if followed by an '=>', a type annotation or body.
                // Otherwise, a parenthesized expression with a missing inner expression
                Kind::RParen
                    if matches!(
                        self.nth_kind(offset + 2),
                        Kind::Arrow | Kind::Colon | Kind::LCurly
                    ) =>
                {
                    IsParenthesizedArrowFunction::True
                }
                // Rest parameter '(...a' is certainly not a parenthesized expression
                Kind::Dot3 => IsParenthesizedArrowFunction::True,
                // '([ ...', '({ ... } can either be a parenthesized object or array expression or a destructing parameter
                Kind::LBrack | Kind::LCurly => IsParenthesizedArrowFunction::Maybe,
                _ if self.nth_kind(offset + 1).is_binding_identifier()
                    || self.nth_at(offset + 1, Kind::This) =>
                {
                    match self.nth_kind(offset + 2) {
                        // '(a: ' must be a type annotation
                        Kind::Colon => IsParenthesizedArrowFunction::True,
                        // * '(a = ': an initializer or a parenthesized assignment expression
                        // * '(a, ': separator to next parameter or a parenthesized sequence expression
                        // * '(a)': a single parameter OR a parenthesized expression
                        Kind::Eq | Kind::Comma | Kind::RParen => {
                            IsParenthesizedArrowFunction::Maybe
                        }
                        // '(a?:' | '(a?,' | '(a?=' | '(a?)'
                        Kind::Question
                            if matches!(
                                self.nth_kind(offset + 3),
                                Kind::Colon | Kind::Comma | Kind::Eq | Kind::RParen
                            ) =>
                        {
                            IsParenthesizedArrowFunction::True
                        }
                        _ => IsParenthesizedArrowFunction::False,
                    }
                }
                _ => IsParenthesizedArrowFunction::False,
            },
            Kind::LAngle => {
                if !self.nth_kind(offset + 1).is_identifier() {
                    return IsParenthesizedArrowFunction::False;
                }

                if self.source_type.is_jsx() {
                    return match self.nth_kind(offset + 2) {
                        Kind::Extends => {
                            let third_kind = self.nth_kind(offset + 3);
                            if matches!(third_kind, Kind::Eq | Kind::RAngle) {
                                IsParenthesizedArrowFunction::False
                            } else if third_kind.is_identifier() {
                                IsParenthesizedArrowFunction::Maybe
                            } else {
                                IsParenthesizedArrowFunction::True
                            }
                        }
                        Kind::Eq | Kind::Comma => IsParenthesizedArrowFunction::True,
                        _ => IsParenthesizedArrowFunction::False,
                    };
                }

                IsParenthesizedArrowFunction::Maybe
            }
            _ => IsParenthesizedArrowFunction::False,
        }
    }

    pub fn is_parenthesized_arrow_function(&mut self) -> IsParenthesizedArrowFunction {
        match self.cur_kind() {
            Kind::LAngle | Kind::LParen => self.is_parenthesized_arrow_function_expression(false),
            Kind::Async => {
                let peeked = self.peek_token();
                if !peeked.is_on_new_line && matches!(peeked.kind, Kind::LAngle | Kind::LParen) {
                    self.is_parenthesized_arrow_function_expression(true)
                } else {
                    IsParenthesizedArrowFunction::False
                }
            }
            _ => IsParenthesizedArrowFunction::False,
        }
    }

    pub fn parse_parenthesized_arrow_function_head(&mut self) -> Result<ArrowFunctionHead<'a>> {
        let node = self.start_node();
        let r#async = self.eat(Kind::Async);

        let has_await = self.ctx.has_await();
        self.ctx = self.ctx.union_await_if(r#async);

        let type_parameters = self.parse_ts_type_parameters()?;

        let params = self.parse_formal_parameters(FormalParameterKind::ArrowFormalParameters)?;

        let return_type = self.parse_ts_return_type_annotation()?;

        self.ctx = self.ctx.and_await(has_await);

        if self.cur_token().is_on_new_line {
            self.error(Diagnostic::LineterminatorBeforeArrow(self.cur_token().node()));
        }

        self.expect(Kind::Arrow)?;

        Ok((type_parameters, params, return_type, r#async, node))
    }

    /// [`ConciseBody`](https://tc39.es/ecma262/#prod-ConciseBody)
    ///     [lookahead ≠ {] `ExpressionBody`[?In, ~Await]
    ///     { `FunctionBody`[~Yield, ~Await] }
    /// `ExpressionBody`[In, Await] :
    ///     `AssignmentExpression`[?In, ~Yield, ?Await]
    pub fn parse_arrow_function_body(
        &mut self,
        node: Node,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Option<TSTypeAnnotation<'a>>,
        r#async: bool,
    ) -> Result<Expression<'a>> {
        let has_await = self.ctx.has_await();
        let has_yield = self.ctx.has_yield();
        self.ctx = self.ctx.and_await(r#async).and_yield(false);

        let expression = !self.at(Kind::LCurly);
        let body = if expression {
            let expr = self.parse_assignment_expression_base()?;
            let node = expr.node();
            let expr_stmt = self.ast.expression_statement(node, expr);
            self.ast.function_body(node, self.ast.new_vec(), self.ast.new_vec_single(expr_stmt))
        } else {
            self.parse_function_body()?
        };

        self.ctx = self.ctx.and_await(has_await).and_yield(has_yield);

        Ok(self.ast.arrow_expression(
            self.end_node(node),
            expression,
            false,
            r#async,
            params,
            body,
            type_parameters,
            return_type,
        ))
    }

    /// Section Arrow Function `https://tc39.es/ecma262/#sec-arrow-function-definitions`
    /// `ArrowFunction`[In, Yield, Await] :
    ///     `ArrowParameters`[?Yield, ?Await] [no `LineTerminator` here] => `ConciseBody`[?In]
    pub fn parse_parenthesized_arrow_function(&mut self) -> Result<Expression<'a>> {
        let (type_parameters, params, return_type, r#async, node) =
            self.parse_parenthesized_arrow_function_head()?;
        self.parse_arrow_function_body(node, type_parameters, params, return_type, r#async)
    }
}
