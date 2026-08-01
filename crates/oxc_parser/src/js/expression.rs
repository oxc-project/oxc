use cow_utils::CowUtils;
use oxc_allocator::{ArenaBox, ArenaVec, CloneIn, GetAllocator, ReplaceWith};
use oxc_ast::{ast::*, builder::NONE};
#[cfg(feature = "regular_expression")]
use oxc_regular_expression::ast::Pattern;
use oxc_span::{GetSpan, GetSpanMut, Span};
use oxc_str::{Ident, Str};
use oxc_syntax::{
    number::{BigintBase, NumberBase},
    precedence::Precedence,
};

use super::{
    arkui::ArkUIArgumentContext,
    grammar::CoverGrammar,
    operator::{
        kind_to_precedence, map_assignment_operator, map_binary_operator, map_logical_operator,
        map_unary_operator, map_update_operator,
    },
};
use crate::{
    Context, ParserConfig as Config, ParserImpl, diagnostics,
    lexer::{Kind, parse_big_int, parse_float, parse_int},
    modifiers::Modifiers,
};

fn is_import_expression_or_member_access_on_import_expression(callee: &Expression<'_>) -> bool {
    let mut expr = callee;
    loop {
        if matches!(expr, Expression::ImportExpression(_)) {
            return true;
        }
        let Some(member_expr) = expr.as_member_expression() else {
            expr = match expr {
                Expression::TaggedTemplateExpression(tagged_template) => &tagged_template.tag,
                Expression::TSNonNullExpression(non_null) => &non_null.expression,
                _ => return false,
            };
            continue;
        };
        expr = member_expr.object();
    }
}

impl<'a, C: Config> ParserImpl<'a, C> {
    pub(crate) fn parse_paren_expression(&mut self) -> Expression<'a> {
        let opening_span = self.cur_token().span();
        self.expect(Kind::LParen);
        let expression = self.parse_expr();
        self.expect_closing(Kind::RParen, opening_span);
        expression
    }

    /// Section [Expression](https://tc39.es/ecma262/#sec-ecmascript-language-expressions)
    pub(crate) fn parse_expr(&mut self) -> Expression<'a> {
        let span = self.start_span();

        let has_decorator = self.ctx.has_decorator();
        if has_decorator {
            self.ctx = self.ctx.and_decorator(false);
        }

        let lhs = self.parse_assignment_expression_or_higher();
        if !self.at(Kind::Comma) {
            return lhs;
        }

        let expr = self.parse_sequence_expression(span, lhs);

        if has_decorator {
            self.ctx = self.ctx.and_decorator(true);
        }

        expr
    }

    /// `PrimaryExpression`: Identifier Reference
    pub(crate) fn parse_identifier_expression(&mut self) -> Expression<'a> {
        let ident = self.parse_identifier_reference();
        Expression::Identifier(self.alloc(ident))
    }

    pub(crate) fn parse_identifier_reference(&mut self) -> IdentifierReference<'a> {
        // allow `await` and `yield`, let semantic analysis report error
        let kind = self.cur_kind();
        if !kind.is_identifier_reference(false, false) {
            return self.unexpected();
        }
        // Track await identifier for potential reparsing in unambiguous mode
        if kind == Kind::Await && !self.ctx.has_await() {
            self.state.encountered_await_identifier = true;
        }
        self.check_identifier(kind, self.ctx);
        let (span, name) = self.parse_identifier_kind(Kind::Ident);
        IdentifierReference::new(span, name, self)
    }

    /// `BindingIdentifier` : Identifier
    pub(crate) fn parse_binding_identifier(&mut self) -> BindingIdentifier<'a> {
        let cur = self.cur_kind();
        if !cur.is_binding_identifier() {
            return if cur.is_reserved_keyword() {
                let error =
                    diagnostics::identifier_reserved_word(self.cur_token().span(), cur.to_str());
                self.fatal_error(error)
            } else {
                self.unexpected()
            };
        }
        self.check_identifier(cur, self.ctx);
        let (span, name) = self.parse_identifier_kind(Kind::Ident);
        self.check_ets_binding_name(&name, span);
        BindingIdentifier::new(span, name, self)
    }

    pub(crate) fn parse_label_identifier(&mut self) -> LabelIdentifier<'a> {
        let kind = self.cur_kind();
        if !kind.is_label_identifier(self.ctx.has_yield(), self.ctx.has_await()) {
            return self.unexpected();
        }
        self.check_identifier(kind, self.ctx);
        let (span, name) = self.parse_identifier_kind(Kind::Ident);
        LabelIdentifier::new(span, name, self)
    }

    pub(crate) fn parse_identifier_name(&mut self) -> IdentifierName<'a> {
        if !self.cur_kind().is_identifier_name() {
            return self.unexpected();
        }
        let (span, name) = self.parse_identifier_kind(Kind::Ident);
        IdentifierName::new(span, name, self)
    }

    #[inline]
    pub(crate) fn parse_identifier_kind(&mut self, kind: Kind) -> (Span, Ident<'a>) {
        let token = self.cur_token();
        let span = token.span();
        // Fast path: most identifiers are not escaped, so we can slice directly
        // from source text without going through `get_string`'s kind matching.
        let name = if token.escaped() { self.cur_string() } else { self.token_source(&token) };
        self.advance(kind);
        (span, self.ident(name))
    }

    /// Create an [`Ident`], respecting [`ParseOptions::enable_ident_hashes`].
    ///
    /// All parser-created [`Ident`]s must be built through this method (or copied from another
    /// `Ident` that was), so that [`ParseOptions::enable_ident_hashes`] applies uniformly.
    /// Building an `Ident` from a `&str`/`Str` via `Into` instead would always hash it, producing
    /// an AST where some `Ident`s are hashed and some are not.
    ///
    /// [`ParseOptions::enable_ident_hashes`]: crate::ParseOptions::enable_ident_hashes
    #[inline]
    pub(crate) fn ident(&self, name: &'a str) -> Ident<'a> {
        if self.options.enable_ident_hashes { Ident::from(name) } else { Ident::new_unhashed(name) }
    }

    pub(crate) fn check_identifier(&mut self, kind: Kind, ctx: Context) {
        self.check_identifier_with_span(kind, ctx, self.cur_token().span());
    }

    pub(crate) fn check_identifier_with_span(&mut self, kind: Kind, ctx: Context, span: Span) {
        match kind {
            // It is a Syntax Error if this production has an [Await] parameter.
            Kind::Await if ctx.has_await() => {
                self.error(diagnostics::identifier_async("await", span));
            }
            // It is a Syntax Error if this production has a [Yield] parameter.
            Kind::Yield if ctx.has_yield() => {
                let next_token = self.lexer.peek_token();
                let looks_like_yield_expression =
                    !next_token.is_on_new_line() && next_token.kind().is_after_await_or_yield();
                self.error(diagnostics::identifier_generator(
                    "yield",
                    span,
                    looks_like_yield_expression,
                ));
            }
            _ => {}
        }
    }

    /// Section [PrivateIdentifier](https://tc39.es/ecma262/#prod-PrivateIdentifier)
    /// `PrivateIdentifier` ::
    ///     # `IdentifierName`
    /// # Panics
    pub(crate) fn parse_private_identifier(&mut self) -> PrivateIdentifier<'a> {
        let span = self.cur_token().span();
        let name = self.ident(self.cur_string());
        self.bump_any();
        PrivateIdentifier::new(span, name, self)
    }

    /// [+In] PrivateIdentifier in ShiftExpression[?Yield, ?Await]
    fn parse_private_in_expression(
        &mut self,
        lhs_span: u32,
        lhs_precedence: Precedence,
    ) -> Expression<'a> {
        let left = self.parse_private_identifier();
        // Check if `in` operator precedence is allowed at current level.
        // For `1 + #a in b`, when parsing RHS of `+`, lhs_precedence is `Add` which is
        // higher than `Compare` (the precedence of `in`), so `#a in` cannot be parsed here.
        if lhs_precedence >= Precedence::Compare {
            return self.fatal_error(diagnostics::unexpected_private_identifier(left.span));
        }
        self.expect(Kind::In);
        let right = self.parse_binary_expression_or_higher(Precedence::Compare);
        if let Expression::PrivateInExpression(private_in_expr) = right {
            return self.fatal_error(diagnostics::private_in_private(private_in_expr.span));
        }
        Expression::new_private_in_expression(self.end_span(lhs_span), left, right, self)
    }

    /// Section [Primary Expression](https://tc39.es/ecma262/#sec-primary-expression)
    /// `PrimaryExpression`[Yield, Await] :
    ///     this
    ///     `IdentifierReference`[?Yield, ?Await]
    ///     Literal
    ///     `ArrayLiteral`[?Yield, ?Await]
    ///     `ObjectLiteral`[?Yield, ?Await]
    ///     `FunctionExpression`
    ///     `ClassExpression`[?Yield, ?Await]
    ///     `GeneratorExpression`
    ///     `AsyncFunctionExpression`
    ///     `AsyncGeneratorExpression`
    ///     `RegularExpressionLiteral`
    ///     `TemplateLiteral`[?Yield, ?Await, ~Tagged]
    ///     `CoverParenthesizedExpressionAndArrowParameterList`[?Yield, ?Await]
    fn parse_primary_expression(&mut self) -> Expression<'a> {
        // Handle ArkUI expressions starting with dots (e.g., in object literals or function bodies)
        // Example: { focused: { .backgroundColor('#ffffeef0') } }
        if self.source_type.is_arkui() && self.is_in_arkui_dsl_context() && self.at(Kind::Dot) {
            return self.parse_leading_dot_expression();
        }

        // FunctionExpression, GeneratorExpression
        // AsyncFunctionExpression, AsyncGeneratorExpression
        if self.at_function_with_async() {
            let span = self.start_span();
            if self.source_type.is_ets_static() {
                self.error(diagnostics::ets_unsupported_syntax(
                    "Function expressions",
                    self.cur_token().span(),
                ));
            }
            let r#async = self.eat(Kind::Async);
            return self.parse_function_expression(span, r#async);
        }

        let kind = self.cur_kind();
        match kind {
            Kind::Ident => self.parse_identifier_expression(), // fast path, keywords are checked at the end
            // ArrayLiteral
            Kind::LBrack => self.parse_array_expression(),
            // ObjectLiteral
            Kind::LCurly => Expression::ObjectExpression(self.parse_object_expression()),
            // ClassExpression
            Kind::Class => {
                if self.source_type.is_ets_static() {
                    self.error(diagnostics::ets_unsupported_syntax(
                        "Class expressions",
                        self.cur_token().span(),
                    ));
                }
                self.parse_class_expression(
                    self.start_span(),
                    &Modifiers::empty(),
                    ArenaVec::new_in(self),
                )
            }
            // This
            Kind::This => self.parse_this_expression(),
            // TemplateLiteral
            Kind::NoSubstitutionTemplate | Kind::TemplateHead => {
                self.parse_template_literal_expression(false)
            }
            Kind::Percent if self.options.allow_v8_intrinsics => {
                self.parse_v8_intrinsic_expression()
            }
            Kind::New => self.parse_new_expression(),
            Kind::Super => self.parse_super(),
            Kind::Import => self.parse_import_meta_or_call(),
            Kind::LParen => self.parse_parenthesized_expression(),
            Kind::Slash | Kind::SlashEq => Expression::RegExpLiteral(self.parse_literal_regexp()),
            Kind::At => self.parse_decorated_expression(),
            // Literal, RegularExpressionLiteral
            kind if kind.is_literal() => self.parse_literal_expression(),
            _ => self.parse_identifier_expression(),
        }
    }

    /// Parse ArkUI leading-dot expression (e.g., `.backgroundColor('#ffffeef0')`)
    /// Creates a LeadingDotExpression, similar to CallExpression but starting with a dot
    pub(crate) fn parse_leading_dot_expression(&mut self) -> Expression<'a> {
        use crate::lexer::Kind;
        let span = self.start_span();

        // Consume the leading dot
        self.expect(Kind::Dot);

        // Check for optional chaining
        let optional = self.eat(Kind::QuestionDot);

        if !self.cur_kind().is_identifier_or_keyword() {
            // Invalid syntax, should have an identifier after the dot
            let error = diagnostics::identifier_expected(self.cur_token().span());
            return self.fatal_error(error);
        }

        // Parse the property name (we'll use it to build the initial call expression)
        let _property = self.parse_identifier_name();

        // Parse type arguments if present (TypeScript)
        let type_arguments =
            if self.is_ts { self.parse_type_arguments_in_expression() } else { None };

        // Parse arguments (optional - can be property access without parentheses)
        let arguments = if self.at(Kind::LParen) {
            let opening_span = self.cur_token().span();
            self.expect(Kind::LParen);
            let (exprs, _) = self.parse_delimited_list(
                Kind::RParen,
                Kind::Comma,
                opening_span,
                Self::parse_assignment_expression_or_higher,
            );
            let mut args = ArenaVec::new_in(self);
            for expr in exprs {
                args.push(Argument::from(expr));
            }
            self.expect(Kind::RParen);
            args
        } else {
            // No parentheses - this is a property access, not a call
            ArenaVec::new_in(self)
        };

        // In ArkUI function bodies, parse the entire chain and store in expression field
        // LeadingDotExpression's arguments field should be empty - arguments are in the expression field
        let type_arguments_for_chain = type_arguments.clone_in(self.ast.allocator());
        let empty_arguments = ArenaVec::new_in(self); // LeadingDotExpression should have empty arguments

        if self.source_type.is_arkui()
            && self.is_in_arkui_dsl_context()
            && (self.at(Kind::Dot)
                || self.at(Kind::QuestionDot)
                || self.at(Kind::LParen)
                || self.at(Kind::LBrack))
        {
            // Parse the entire chain starting from the leading dot
            // The expression field should start from fontSize(size) (without the leading dot)
            // So we create a CallExpression with Identifier as callee
            // Start the expression span from the property (not including the leading dot)
            // Use _property.span.start as the expression start to ensure correct span
            let expression_span_start = _property.span.start;
            let property_ident = Expression::new_identifier(_property.span, _property.name, self);

            // Create the initial CallExpression: fontSize(size)
            // Note: callee is Identifier, not StaticMemberExpression
            // Use _property.span.start and current prev_token_end to create valid span
            let initial_call_end = self.prev_token_end.max(expression_span_start);
            let initial_call_span = Span::new(expression_span_start, initial_call_end);
            let initial_call = Expression::new_call_expression(
                initial_call_span,
                property_ident,
                type_arguments,
                arguments,
                optional,
                self,
            );

            let mut in_optional_chain = false;
            let mut chained_expr = initial_call;
            if self.at(Kind::Dot) || self.at(Kind::QuestionDot) || self.at(Kind::LBrack) {
                chained_expr = self.parse_member_expression_rest(
                    expression_span_start,
                    chained_expr,
                    &mut in_optional_chain,
                    /* allow_optional_chain */ true,
                );
            }
            if self.at(Kind::LParen) || self.at(Kind::QuestionDot) {
                chained_expr = self.parse_call_expression_rest(
                    expression_span_start,
                    chained_expr,
                    &mut in_optional_chain,
                );
            }

            // Create LeadingDotExpression with the entire chain in expression field
            // The expression field contains fontSize(size).fancy(12).hello() (without leading dot)
            // The expression span should start from property, not including the leading dot
            // Ensure end >= start to avoid assertion failures
            let mut chained_expr_with_correct_span = chained_expr;
            let chained_expr_end = chained_expr_with_correct_span.span().end;
            // Ensure the end is at least as large as the start
            let expression_span_end = chained_expr_end.max(expression_span_start);
            *chained_expr_with_correct_span.span_mut() =
                Span::new(expression_span_start, expression_span_end);

            // Create leading_dot_span: from the dot start to the end of the chained expression
            // Ensure end >= start to avoid assertion failures
            let leading_dot_start = span;
            let leading_dot_end =
                self.prev_token_end.max(expression_span_end).max(leading_dot_start);
            let leading_dot_span = Span::new(leading_dot_start, leading_dot_end);
            return Expression::new_leading_dot_expression(
                leading_dot_span,
                optional,
                type_arguments_for_chain,
                empty_arguments, // LeadingDotExpression should have empty arguments
                chained_expr_with_correct_span,
                self,
            );
        }

        // No chaining - create a simple LeadingDotExpression with just the initial call
        // For non-chained case, we still need to create the expression
        // The expression field should start from fontSize(size) (without the leading dot)
        // Start the expression span from the property (not including the leading dot)
        let expression_start_span = self.start_span();
        let property_ident = Expression::new_identifier(_property.span, _property.name, self);

        // Create CallExpression with Identifier as callee (not StaticMemberExpression)
        let mut initial_call = Expression::new_call_expression(
            self.end_span(expression_start_span),
            property_ident,
            type_arguments,
            arguments,
            optional,
            self,
        );

        // Update the expression span to exclude the leading dot
        // Ensure end >= start to avoid assertion failures
        let initial_call_end = initial_call.span().end;
        let expression_span_end = initial_call_end.max(_property.span.start);
        *initial_call.span_mut() = Span::new(_property.span.start, expression_span_end);

        // LeadingDotExpression span includes the leading dot, but expression span doesn't
        // Ensure end >= start to avoid assertion failures
        // LeadingDotExpression's arguments field should be empty - arguments are in the expression field
        let leading_dot_start = span;
        let leading_dot_end = self.prev_token_end.max(expression_span_end).max(leading_dot_start);
        let leading_dot_span = Span::new(leading_dot_start, leading_dot_end);
        Expression::new_leading_dot_expression(
            leading_dot_span,
            optional,
            type_arguments_for_chain,
            empty_arguments, // LeadingDotExpression should have empty arguments
            initial_call,
            self,
        )
    }

    /// Parse member expression rest starting from a given LHS for primary expressions
    /// Used for ArkUI expressions starting with dots in object literals and other contexts
    #[allow(dead_code)]
    pub(crate) fn parse_member_expression_rest_from_lhs_for_primary(
        &mut self,
        _lhs_span: u32,
        lhs: Expression<'a>,
    ) -> Expression<'a> {
        use crate::lexer::Kind;
        let mut lhs = lhs;

        loop {
            if self.fatal_error.is_some() {
                return lhs;
            }

            // Check if we should stop parsing the chain
            // Stop if we encounter semicolon, comma, or closing brace
            // Comma stops the chain in object literal contexts (indicates next property)
            // Note: comma inside function arguments won't reach here as it's handled by parse_delimited_list
            if matches!(self.cur_kind(), Kind::Semicolon | Kind::Comma | Kind::RCurly) {
                break;
            }

            let is_property_access = self.eat(Kind::Dot);
            if !is_property_access {
                break;
            }

            if self.cur_kind().is_identifier_or_keyword() {
                let ident_span = self.start_span();
                let ident = self.parse_identifier_name();
                if self.at(Kind::LParen) {
                    // Method call: .methodName(...)
                    let member_expr = Expression::new_static_member_expression(
                        self.end_span(ident_span),
                        lhs,
                        ident,
                        false,
                        self,
                    );
                    // Parse call arguments
                    let call_span = self.start_span();
                    let opening_span = self.cur_token().span();
                    self.expect(Kind::LParen);
                    let (exprs, _) = self.parse_delimited_list(
                        Kind::RParen,
                        Kind::Comma,
                        opening_span,
                        Self::parse_assignment_expression_or_higher,
                    );
                    let mut call_args = ArenaVec::new_in(self);
                    for expr in exprs {
                        call_args.push(Argument::from(expr));
                    }
                    self.expect(Kind::RParen);
                    // Create call expression
                    lhs = Expression::new_call_expression(
                        self.end_span(call_span),
                        member_expr,
                        NONE,
                        call_args,
                        false,
                        self,
                    );
                    // Continue parsing more chain expressions
                    // Check if next token is a dot (chain continues) or semicolon/comma/brace (chain ends)
                    if !self.at(Kind::Dot)
                        || matches!(self.cur_kind(), Kind::Semicolon | Kind::Comma | Kind::RCurly)
                    {
                        break;
                    }
                    continue;
                } else {
                    // Property access: .propertyName
                    lhs = Expression::new_static_member_expression(
                        self.end_span(ident_span),
                        lhs,
                        ident,
                        false,
                        self,
                    );
                    // Check if there are more chain expressions
                    // Stop if next token is not a dot, or if it's semicolon/comma/brace
                    if !self.at(Kind::Dot)
                        || matches!(self.cur_kind(), Kind::Semicolon | Kind::Comma | Kind::RCurly)
                    {
                        break;
                    }
                    continue;
                }
            }

            break;
        }

        lhs
    }

    fn parse_parenthesized_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        let opening_span = self.cur_token().span();
        // Capture annotation flags before bumping `(` since bump resets them
        let has_no_side_effects_comment =
            self.lexer.trivia_builder.previous_token_has_no_side_effects_comment();
        self.bump_any(); // `bump` `(`
        let expr_span = self.start_span();
        let (mut expressions, comma_span) = self.context(Context::In, Context::Decorator, |p| {
            p.parse_delimited_list(
                Kind::RParen,
                Kind::Comma,
                opening_span,
                Self::parse_assignment_expression_or_higher,
            )
        });

        if let Some(comma_span) = comma_span {
            let error = diagnostics::unexpected_trailing_comma(
                "Parenthesized expressions",
                self.end_span(comma_span),
            );
            return self.fatal_error(error);
        }

        if expressions.is_empty() {
            self.expect(Kind::RParen);
            let error = diagnostics::empty_parenthesized_expression(self.end_span(span));
            return self.fatal_error(error);
        }

        let expr_span = self.end_span(expr_span);
        self.expect(Kind::RParen);

        // ParenthesizedExpression is from acorn --preserveParens
        let mut expression = if expressions.len() == 1 {
            expressions.remove(0)
        } else {
            if self.source_type.is_ets_static() && !self.ctx.ets_allows_sequence() {
                self.error(diagnostics::ets_unsupported_syntax(
                    "The comma operator outside a `for` statement",
                    expr_span,
                ));
            }
            Expression::new_sequence_expression(expr_span, expressions, self)
        };

        match &mut expression {
            Expression::ArrowFunctionExpression(arrow_expr) => {
                arrow_expr.pife = true;
                if has_no_side_effects_comment {
                    arrow_expr.pure = true;
                }
            }
            Expression::FunctionExpression(func_expr) => {
                func_expr.pife = true;
                if has_no_side_effects_comment {
                    func_expr.pure = true;
                }
            }
            _ => {}
        }

        if self.options.preserve_parens {
            Expression::new_parenthesized_expression(self.end_span(span), expression, self)
        } else {
            expression
        }
    }

    /// Section 13.2.2 This Expression
    fn parse_this_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        self.bump_any();
        Expression::new_this_expression(self.end_span(span), self)
    }

    /// [Literal Expression](https://tc39.es/ecma262/#prod-Literal)
    /// parses string | true | false | null | number
    pub(crate) fn parse_literal_expression(&mut self) -> Expression<'a> {
        let kind = self.cur_kind();
        match kind {
            Kind::Str => {
                let lit = self.parse_literal_string();
                Expression::StringLiteral(self.alloc(lit))
            }
            Kind::CharLiteral => Expression::CharLiteral(self.parse_ets_char_literal()),
            Kind::True | Kind::False => Expression::BooleanLiteral(self.parse_literal_boolean()),
            Kind::Null => Expression::NullLiteral(self.parse_literal_null()),
            Kind::DecimalBigInt | Kind::BinaryBigInt | Kind::OctalBigInt | Kind::HexBigInt => {
                Expression::BigIntLiteral(self.parse_literal_bigint())
            }
            kind if kind.is_number() => Expression::NumericLiteral(self.parse_literal_number()),
            _ => self.unexpected(),
        }
    }

    pub(crate) fn parse_literal_boolean(&mut self) -> ArenaBox<'a, BooleanLiteral> {
        let span = self.start_span();
        let value = match self.cur_kind() {
            Kind::True => true,
            Kind::False => false,
            _ => return self.unexpected(),
        };
        self.bump_any();
        BooleanLiteral::boxed(self.end_span(span), value, self)
    }

    fn parse_ets_char_literal(&mut self) -> ArenaBox<'a, CharLiteral<'a>> {
        let token = self.cur_token();
        let span = token.span();
        let raw = self.cur_src();
        let value = self.cur_string();
        let mut utf16 = value.encode_utf16();
        let first = utf16.next();
        let second = utf16.next();
        let value = match (first, second) {
            (Some(value), None) => u32::from(value),
            _ => {
                self.error(diagnostics::ets_char_literal_length(span));
                0
            }
        };
        self.bump_any();
        CharLiteral::boxed(span, value, Some(Str::from(raw)), self)
    }

    pub(crate) fn parse_literal_null(&mut self) -> ArenaBox<'a, NullLiteral> {
        let span = self.cur_token().span();
        self.bump_any(); // bump `null`
        NullLiteral::boxed(span, self)
    }

    pub(crate) fn parse_literal_number(&mut self) -> ArenaBox<'a, NumericLiteral<'a>> {
        let token = self.cur_token();
        let span = token.span();
        let kind = token.kind();
        let raw = self.cur_src();
        let src = if self.source_type.is_ets_static() {
            raw.strip_suffix('f').unwrap_or(raw)
        } else {
            raw
        };
        let has_separator = token.has_separator();
        let value = match kind {
            Kind::Decimal | Kind::Binary | Kind::Octal | Kind::Hex => {
                parse_int(src, kind, has_separator)
            }
            Kind::Float | Kind::PositiveExponential | Kind::NegativeExponential => {
                parse_float(src, has_separator)
            }
            _ => unreachable!(),
        };
        let value = value.unwrap_or_else(|err| {
            self.set_fatal_error(diagnostics::invalid_number(err, span));
            0.0 // Dummy value
        });
        let base = match kind {
            Kind::Decimal => NumberBase::Decimal,
            Kind::Float => NumberBase::Float,
            Kind::Binary => NumberBase::Binary,
            Kind::Octal => NumberBase::Octal,
            Kind::Hex => NumberBase::Hex,
            Kind::PositiveExponential | Kind::NegativeExponential => {
                if value.fract() == 0.0 {
                    NumberBase::Decimal
                } else {
                    NumberBase::Float
                }
            }
            _ => return self.unexpected(),
        };
        self.bump_any();
        NumericLiteral::boxed(span, value, Some(Str::from(raw)), base, self)
    }

    pub(crate) fn parse_literal_bigint(&mut self) -> ArenaBox<'a, BigIntLiteral<'a>> {
        let token = self.cur_token();
        let kind = token.kind();
        let has_separator = token.has_separator();
        let (base, number_kind) = match kind {
            Kind::DecimalBigInt => (BigintBase::Decimal, Kind::Decimal),
            Kind::BinaryBigInt => (BigintBase::Binary, Kind::Binary),
            Kind::OctalBigInt => (BigintBase::Octal, Kind::Octal),
            Kind::HexBigInt => (BigintBase::Hex, Kind::Hex),
            _ => return self.unexpected(),
        };
        let span = token.span();
        let raw = self.cur_src();
        let src = raw.strip_suffix('n').unwrap();
        let value = parse_big_int(src, number_kind, has_separator, self.allocator());

        self.bump_any();
        BigIntLiteral::boxed(span, value, Some(Str::from(raw)), base, self)
    }

    pub(crate) fn parse_literal_regexp(&mut self) -> ArenaBox<'a, RegExpLiteral<'a>> {
        let (pattern_end, flags, flags_error) = self.read_regex();
        if !self.lexer.errors.is_empty() {
            return self.unexpected();
        }
        let span = self.cur_token().span();
        let pattern_start = span.start + 1; // +1 to exclude left `/`
        let pattern_text = &self.source_text[pattern_start as usize..pattern_end as usize];
        let flags_start = pattern_end + 1; // +1 to include right `/`
        let flags_text = &self.source_text[flags_start as usize..span.end as usize];
        let raw = self.cur_src();
        self.bump_any();

        // Parse pattern if options is enabled and also flags are valid
        #[cfg(feature = "regular_expression")]
        let pattern = if self.options.parse_regular_expression && !flags_error {
            self.parse_regex_pattern(pattern_start, pattern_text, flags_start, flags_text)
        } else {
            None
        };
        #[cfg(not(feature = "regular_expression"))]
        let pattern = {
            let _ = (flags_text, flags_error);
            None
        };

        let pattern = RegExpPattern { text: Str::from(pattern_text), pattern };

        if flags.contains(RegExpFlags::U | RegExpFlags::V) {
            self.error(diagnostics::reg_exp_flag_u_and_v(span));
        }

        RegExpLiteral::boxed(span, RegExp { pattern, flags }, Some(Str::from(raw)), self)
    }

    #[cfg(feature = "regular_expression")]
    fn parse_regex_pattern(
        &mut self,
        pattern_span_offset: u32,
        pattern: &'a str,
        flags_span_offset: u32,
        flags: &'a str,
    ) -> Option<ArenaBox<'a, Pattern<'a>>> {
        use oxc_regular_expression::{LiteralParser, Options};
        match LiteralParser::new(
            self.allocator(),
            pattern,
            Some(flags),
            Options { pattern_span_offset, flags_span_offset },
        )
        .parse()
        {
            Ok(regular_expression) => Some(self.alloc(regular_expression)),
            Err(diagnostic) => {
                self.error(diagnostic);
                None
            }
        }
    }

    pub(crate) fn parse_literal_string(&mut self) -> StringLiteral<'a> {
        if !self.at(Kind::Str) {
            return self.unexpected();
        }
        let span = self.cur_token().span();
        let raw = Str::from(self.cur_src());
        let value = self.cur_string();
        let lone_surrogates = self.cur_token().lone_surrogates();
        self.bump_any();
        StringLiteral::new_with_lone_surrogates(span, value, Some(raw), lone_surrogates, self)
    }

    /// Section [Array Expression](https://tc39.es/ecma262/#prod-ArrayLiteral)
    /// `ArrayLiteral`[Yield, Await]:
    ///     [ Elision opt ]
    ///     [ `ElementList`[?Yield, ?Await] ]
    ///     [ `ElementList`[?Yield, ?Await] , Elisionopt ]
    pub(crate) fn parse_array_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        let opening_span = self.cur_token().span();
        self.expect(Kind::LBrack);
        let (elements, comma_span) = self.context_add(Context::In, |p| {
            p.parse_delimited_list(
                Kind::RBrack,
                Kind::Comma,
                opening_span,
                Self::parse_array_expression_element,
            )
        });
        if let Some(comma_span) = comma_span {
            self.state.trailing_commas.insert(span, self.end_span(comma_span));
        }
        self.expect(Kind::RBrack);
        Expression::new_array_expression(self.end_span(span), elements, self)
    }

    fn parse_array_expression_element(&mut self) -> ArrayExpressionElement<'a> {
        match self.cur_kind() {
            Kind::Comma => {
                if self.source_type.is_ets_static() {
                    self.error(diagnostics::ets_unsupported_syntax(
                        "Omitted array elements",
                        self.cur_token().span(),
                    ));
                }
                self.parse_elision()
            }
            Kind::Dot3 => ArrayExpressionElement::SpreadElement(self.parse_spread_element()),
            _ => ArrayExpressionElement::from(self.parse_assignment_expression_or_higher()),
        }
    }

    /// Elision :
    ///     ,
    ///    Elision ,
    pub(crate) fn parse_elision(&self) -> ArrayExpressionElement<'a> {
        ArrayExpressionElement::new_elision(self.cur_token().span(), self)
    }

    /// Section [Template Literal](https://tc39.es/ecma262/#prod-TemplateLiteral)
    /// `TemplateLiteral`[Yield, Await, Tagged] :
    ///     `NoSubstitutionTemplate`
    ///     `SubstitutionTemplate`[?Yield, ?Await, ?Tagged]
    pub(crate) fn parse_template_literal(&mut self, tagged: bool) -> TemplateLiteral<'a> {
        let span = self.start_span();

        let (quasis, expressions) = match self.cur_kind() {
            Kind::NoSubstitutionTemplate => {
                let quasis = ArenaVec::from_value_in(self.parse_template_element(tagged), self);
                (quasis, ArenaVec::new_in(self))
            }
            Kind::TemplateHead => {
                let mut expressions = ArenaVec::with_capacity_in(1, self);
                let mut quasis = ArenaVec::with_capacity_in(2, self);

                quasis.push(self.parse_template_element(tagged));
                // TemplateHead Expression[+In, ?Yield, ?Await]
                let expr = self.context_add(Context::In, Self::parse_expr);
                expressions.push(expr);
                self.re_lex_template_substitution_tail();
                while self.fatal_error.is_none() {
                    match self.cur_kind() {
                        Kind::TemplateTail => {
                            quasis.push(self.parse_template_element(tagged));
                            break;
                        }
                        Kind::TemplateMiddle => {
                            quasis.push(self.parse_template_element(tagged));
                            // TemplateMiddle Expression[+In, ?Yield, ?Await]
                            let expr = self.context_add(Context::In, Self::parse_expr);
                            expressions.push(expr);
                            self.re_lex_template_substitution_tail();
                        }
                        _ => {
                            self.expect(Kind::TemplateTail);
                            break;
                        }
                    }
                }

                (quasis, expressions)
            }
            _ => unreachable!("parse_template_literal"),
        };

        TemplateLiteral::new(self.end_span(span), quasis, expressions, self)
    }

    pub(crate) fn parse_template_literal_expression(&mut self, tagged: bool) -> Expression<'a> {
        let template_lit = self.parse_template_literal(tagged);
        Expression::TemplateLiteral(self.alloc(template_lit))
    }

    fn parse_tagged_template(
        &mut self,
        span: u32,
        lhs: Expression<'a>,
        in_optional_chain: bool,
        type_arguments: Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Expression<'a> {
        let quasi = self.parse_template_literal(true);
        let span = self.end_span(span);
        // OptionalChain :
        //   ?. TemplateLiteral
        //   OptionalChain TemplateLiteral
        // It is a Syntax Error if any source text is matched by this production.
        // <https://tc39.es/ecma262/#sec-left-hand-side-expressions-static-semantics-early-errors>
        if in_optional_chain {
            self.error(diagnostics::optional_chain_tagged_template(quasi.span));
        }
        Expression::new_tagged_template_expression(span, lhs, type_arguments, quasi, self)
    }

    pub(crate) fn parse_template_element(&mut self, tagged: bool) -> TemplateElement<'a> {
        let span = self.start_span();
        let cur_kind = self.cur_kind();
        let end_offset: u32 = match cur_kind {
            Kind::TemplateHead | Kind::TemplateMiddle => 2,
            Kind::NoSubstitutionTemplate | Kind::TemplateTail => 1,
            _ => unreachable!(),
        };

        // Get `raw`
        let raw_span = self.cur_token().span();
        let mut raw = Str::from(
            &self.source_text[raw_span.start as usize + 1..(raw_span.end - end_offset) as usize],
        );

        // Get `cooked`.
        // Also replace `\r` with `\n` in `raw`.
        // If contains `\r`, then `escaped` must be `true` (because `\r` needs unescaping),
        // so we can skip searching for `\r` in common case where contains no escapes.
        let (cooked, lone_surrogates) = if self.cur_token().escaped() {
            // `cooked = None` when template literal has invalid escape sequence
            let cooked = self.cur_template_string().map(Str::from);
            if cooked.is_some() && raw.contains('\r') {
                raw =
                    Str::from_str_in(&raw.cow_replace("\r\n", "\n").cow_replace('\r', "\n"), self);
            }
            (cooked, self.cur_token().lone_surrogates())
        } else {
            (Some(raw), false)
        };

        self.bump_any();

        let mut span = self.end_span(span);
        span.start += 1;
        span.end -= end_offset;

        if !tagged && cooked.is_none() {
            self.error(diagnostics::template_literal(span));
        }

        let tail = matches!(cur_kind, Kind::TemplateTail | Kind::NoSubstitutionTemplate);
        // Parser provides already-escaped values from source, so no escaping needed here
        TemplateElement::new_with_lone_surrogates(
            span,
            TemplateElementValue { raw, cooked },
            tail,
            lone_surrogates,
            self,
        )
    }

    /// Section 13.3 ImportCall or ImportMeta
    fn parse_import_meta_or_call(&mut self) -> Expression<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `import`
        match self.cur_kind() {
            Kind::Dot => {
                self.bump_any(); // bump `.`
                match self.cur_kind() {
                    // `import.meta`
                    Kind::Meta => {
                        self.bump_any(); // bump `meta`
                        let span = self.end_span(span);
                        self.module_record_builder.visit_import_meta(span);
                        // `import.meta` is only allowed in module code.
                        if !self.source_type.is_module() {
                            self.error_on_script(diagnostics::import_meta(span));
                        }
                        Expression::new_import_meta(span, self)
                    }
                    // `import.source(expr)`
                    Kind::Source => {
                        self.bump_any();
                        self.parse_import_expression(span, Some(ImportPhase::Source))
                    }
                    // `import.defer(expr)`
                    Kind::Defer => {
                        self.bump_any();
                        self.parse_import_expression(span, Some(ImportPhase::Defer))
                    }
                    _ => {
                        self.bump_any();
                        self.fatal_error(diagnostics::invalid_import_property(self.end_span(span)))
                    }
                }
            }
            Kind::LParen => self.parse_import_expression(span, None),
            _ => self.unexpected(),
        }
    }

    /// V8 Runtime calls.
    /// See: [runtime.h](https://github.com/v8/v8/blob/5fe0aa3bc79c0a9d3ad546b79211f07105f09585/src/runtime/runtime.h#L43)
    pub(crate) fn parse_v8_intrinsic_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        self.expect(Kind::Percent);
        let name = self.parse_identifier_name();

        let opening_span = self.cur_token().span();
        self.expect(Kind::LParen);
        let (arguments, _) = self.context(Context::In, Context::Decorator, |p| {
            p.parse_delimited_list(
                Kind::RParen,
                Kind::Comma,
                opening_span,
                Self::parse_v8_intrinsic_argument,
            )
        });
        self.expect(Kind::RParen);
        Expression::new_v8_intrinsic_expression(self.end_span(span), name, arguments, self)
    }

    fn parse_v8_intrinsic_argument(&mut self) -> Argument<'a> {
        if self.at(Kind::Dot3) {
            self.error(diagnostics::v8_intrinsic_spread_elem(self.cur_token().span()));
            Argument::SpreadElement(self.parse_spread_element())
        } else {
            Argument::from(self.parse_assignment_expression_or_higher())
        }
    }

    /// Section 13.3 Left-Hand-Side Expression
    pub(crate) fn parse_lhs_expression_or_higher(&mut self) -> Expression<'a> {
        let span = self.start_span();
        let mut in_optional_chain = false;
        // `MemberExpression`
        let primary = self.parse_primary_expression();
        let member_expression = self.parse_member_expression_rest(
            span,
            primary,
            &mut in_optional_chain,
            /* allow_optional_chain */ true,
        );
        // A fully-parsed `MemberExpression` only extends into a `LeftHandSideExpression` via
        // `Arguments` (`(`) or an `OptionalChain` (`?.`); see <https://tc39.es/ecma262/#sec-left-hand-side-expressions>.
        // So skip `parse_call_expression_rest` (and its redundant member-rest re-scan) otherwise.
        let lhs = if matches!(self.cur_kind(), Kind::LParen | Kind::QuestionDot) {
            self.parse_call_expression_rest(span, member_expression, &mut in_optional_chain)
        } else {
            member_expression
        };
        if !in_optional_chain {
            return lhs;
        }
        if self.ctx.has_decorator() {
            self.error(diagnostics::decorator_optional(lhs.span()));
        }
        // Add `ChainExpression` to `a?.c?.b<c>`;
        if let Expression::TSInstantiationExpression(mut expr) = lhs {
            expr.expression.replace_with(|expr| self.map_to_chain_expression(expr.span(), expr));
            Expression::TSInstantiationExpression(expr)
        } else {
            let span = self.end_span(span);
            self.map_to_chain_expression(span, lhs)
        }
    }

    fn map_to_chain_expression(&self, span: Span, expr: Expression<'a>) -> Expression<'a> {
        match expr {
            match_member_expression!(Expression) => {
                let member_expr = expr.into_member_expression();
                Expression::new_chain_expression(span, ChainElement::from(member_expr), self)
            }
            Expression::CallExpression(e) => {
                Expression::new_chain_expression(span, ChainElement::CallExpression(e), self)
            }
            Expression::TSNonNullExpression(e) => {
                Expression::new_chain_expression(span, ChainElement::TSNonNullExpression(e), self)
            }
            expr => expr,
        }
    }

    /// Section 13.3 Super Call
    fn parse_super(&mut self) -> Expression<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `super`
        let span = self.end_span(span);

        // The `super` keyword can appear at below:
        // SuperProperty:
        //     super [ Expression ]
        //     super . IdentifierName
        // SuperCall:
        //     super ( Arguments )
        if !matches!(self.cur_kind(), Kind::Dot | Kind::LBrack | Kind::LParen) {
            self.error(diagnostics::unexpected_super(span));
        }

        Expression::new_super(span, self)
    }

    /// An instantiation expression (`MemberExpression TypeArguments`) directly followed by a
    /// member access is TS1477; a parenthesized one is allowed:
    ///
    /// ```text
    /// a<b>.c      // error
    /// a<b>?.[c]   // error
    /// (a<b>).c    // ok
    /// ```
    fn error_if_unparenthesized_instantiation_expression(
        &mut self,
        lhs: &Expression<'a>,
        lhs_span: u32,
    ) {
        if matches!(lhs, Expression::TSInstantiationExpression(e) if e.span.start == lhs_span) {
            self.error(
                diagnostics::ts_instantiation_expression_cannot_be_followed_by_property_access(
                    self.end_span(lhs_span),
                ),
            );
        }
    }

    /// parse rhs of a member expression, starting from lhs
    fn parse_member_expression_rest(
        &mut self,
        lhs_span: u32,
        lhs: Expression<'a>,
        in_optional_chain: &mut bool,
        allow_optional_chain: bool,
    ) -> Expression<'a> {
        let mut lhs = lhs;
        loop {
            if self.fatal_error.is_some() {
                return lhs;
            }

            match self.cur_kind() {
                Kind::Dot => {
                    self.bump_any();
                    self.error_if_unparenthesized_instantiation_expression(&lhs, lhs_span);
                    lhs = self.parse_static_member_expression(lhs_span, lhs, false);
                }
                Kind::QuestionDot if allow_optional_chain => {
                    // Fast check to avoid checkpoint/rewind in common cases
                    let next_kind = self.lexer.peek_token().kind();
                    if next_kind == Kind::LBrack
                        || next_kind.is_identifier_or_keyword()
                        || next_kind.is_template_start_of_tagged_template()
                    {
                        // This is likely a valid optional chain, proceed with normal parsing
                        self.bump_any(); // consume ?.
                        *in_optional_chain = true;
                        if self.cur_kind().is_identifier_or_keyword() {
                            // ?.something
                            self.error_if_unparenthesized_instantiation_expression(&lhs, lhs_span);
                            lhs = self.parse_static_member_expression(lhs_span, lhs, true);
                        } else if self.at(Kind::LBrack) {
                            // ?.[
                            self.error_if_unparenthesized_instantiation_expression(&lhs, lhs_span);
                            lhs = self.parse_computed_member_expression(lhs_span, lhs, true);
                        } else {
                            // ?.template`...`
                            lhs =
                                self.parse_tagged_template_rest(lhs_span, lhs, *in_optional_chain);
                        }
                    } else {
                        // This is not a valid optional chain pattern, don't consume ?.
                        // Should be a cold branch here, as most real-world optional chaining will look like
                        // `?.something` or `?.[expr]`
                        return lhs;
                    }
                }
                Kind::LBrack if !self.ctx.has_decorator() => {
                    self.error_if_unparenthesized_instantiation_expression(&lhs, lhs_span);
                    lhs = self.parse_computed_member_expression(lhs_span, lhs, false);
                }
                kind if kind.is_template_start_of_tagged_template() => {
                    lhs = self.parse_tagged_template_rest(lhs_span, lhs, *in_optional_chain);
                }
                Kind::Bang if self.is_ts && !self.cur_token().is_on_new_line() => {
                    self.bump_any();
                    lhs =
                        Expression::new_ts_non_null_expression(self.end_span(lhs_span), lhs, self);
                }
                Kind::LAngle | Kind::ShiftLeft if self.is_ts => {
                    if let Some(arguments) = self.parse_type_arguments_in_expression() {
                        lhs = Expression::new_ts_instantiation_expression(
                            self.end_span(lhs_span),
                            lhs,
                            arguments,
                            self,
                        );
                    } else {
                        // `re_lex_as_typescript_l_angle` may have popped the original token
                        // (e.g. `<<`) from the collected token stream. Rewind restored the
                        // parser's current token, so write it back to the stream.
                        // This is a no-op when tokens are statically disabled (`NoTokensLexerConfig`).
                        self.lexer.rewrite_last_collected_token(self.token);
                        return lhs;
                    }
                }
                _ => return lhs,
            }
        }
    }

    /// Parse the tagged template continuation of a member expression,
    /// unwrapping a `TSInstantiationExpression` lhs (`` foo<T>`...` ``) into its type arguments.
    fn parse_tagged_template_rest(
        &mut self,
        lhs_span: u32,
        lhs: Expression<'a>,
        in_optional_chain: bool,
    ) -> Expression<'a> {
        let (expr, type_arguments) =
            if let Expression::TSInstantiationExpression(instantiation_expr) = lhs {
                let expr = instantiation_expr.unbox();
                (expr.expression, Some(expr.type_arguments))
            } else {
                (lhs, None)
            };
        self.parse_tagged_template(lhs_span, expr, in_optional_chain, type_arguments)
    }

    /// Section 13.3 `MemberExpression`
    /// static member `a.b`
    fn parse_static_member_expression(
        &mut self,
        lhs_span: u32,
        lhs: Expression<'a>,
        optional: bool,
    ) -> Expression<'a> {
        Expression::from(if self.cur_kind() == Kind::PrivateIdentifier {
            let private_ident = self.parse_private_identifier();
            let span = self.end_span(lhs_span);
            // `super.#field` is not allowed.
            if lhs.is_super() {
                self.error(diagnostics::super_private(span));
            }
            MemberExpression::new_private_field_expression(span, lhs, private_ident, optional, self)
        } else {
            let ident = self.parse_identifier_name();
            if self.source_type.is_ets_static() && ident.name == "prototype" {
                self.error(diagnostics::ets_unsupported_syntax(
                    "Runtime prototype access",
                    ident.span,
                ));
            }
            MemberExpression::new_static_member_expression(
                self.end_span(lhs_span),
                lhs,
                ident,
                optional,
                self,
            )
        })
    }

    /// Section 13.3 `MemberExpression`
    /// `MemberExpression`[Yield, Await] :
    ///   `MemberExpression`[?Yield, ?Await] [ Expression[+In, ?Yield, ?Await] ]
    fn parse_computed_member_expression(
        &mut self,
        lhs_span: u32,
        lhs: Expression<'a>,
        optional: bool,
    ) -> Expression<'a> {
        self.check_ets_type_value(&lhs);
        self.bump_any(); // advance `[`
        let property = self.context_add(Context::In, Self::parse_expr);
        self.expect(Kind::RBrack);
        Expression::new_computed_member_expression(
            self.end_span(lhs_span),
            lhs,
            property,
            optional,
            self,
        )
    }

    /// [NewExpression](https://tc39.es/ecma262/#sec-new-operator)
    fn parse_new_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `new`

        if self.eat(Kind::Dot) {
            return if self.at(Kind::Target) {
                self.bump_any(); // bump `target`
                let span = self.end_span(span);
                if !self.ctx.has_new_target() {
                    self.error(diagnostics::new_target_outside_function(span));
                }
                Expression::new_new_target(span, self)
            } else {
                self.bump_any();
                self.fatal_error(diagnostics::new_target(self.end_span(span)))
            };
        }

        if self.source_type.is_ets_static() {
            return self.parse_ets_new_expression(span);
        }

        let rhs_span = self.start_span();
        let is_import = self.at(Kind::Import); // Syntax Error for `new import('mod')` but not `new (import('mod'))`.
        let mut optional = false;
        let mut callee = {
            let lhs = self.parse_primary_expression();
            self.parse_member_expression_rest(
                rhs_span,
                lhs,
                &mut optional,
                /* allow_optional_chain */ false,
            )
        };

        let mut type_arguments = None;
        if let Expression::TSInstantiationExpression(instantiation_expr) = callee {
            let instantiation_expr = instantiation_expr.unbox();
            type_arguments.replace(instantiation_expr.type_arguments);
            callee = instantiation_expr.expression;
        }

        if self.at(Kind::QuestionDot) {
            let error = diagnostics::invalid_new_optional_chain(self.cur_token().span());
            return self.fatal_error(error);
        }

        let opening_span = self.cur_token().span();

        // parse `new ident` without arguments
        let arguments = if self.eat(Kind::LParen) {
            // ArgumentList[Yield, Await] :
            //   AssignmentExpression[+In, ?Yield, ?Await]
            let (call_arguments, _) = self.context_add(Context::In, |p| {
                p.parse_delimited_list(
                    Kind::RParen,
                    Kind::Comma,
                    opening_span,
                    Self::parse_call_argument,
                )
            });
            self.expect(Kind::RParen);
            call_arguments
        } else {
            ArenaVec::new_in(self)
        };

        if is_import && is_import_expression_or_member_access_on_import_expression(&callee) {
            self.error(diagnostics::new_dynamic_import(self.end_span(rhs_span)));
        }

        if matches!(callee, Expression::Super(_)) {
            self.error(diagnostics::new_super(self.end_span(rhs_span)));
        }

        let span = self.end_span(span);

        if optional {
            self.error(diagnostics::new_optional_chain(span));
        }

        Expression::new_new_expression(span, callee, type_arguments, arguments, self)
    }

    fn parse_ets_new_expression(&mut self, span: u32) -> Expression<'a> {
        let type_annotation = self.parse_non_array_type();

        if self.at(Kind::LBrack) {
            let mut dimensions = ArenaVec::with_capacity_in(1, self);
            while self.eat(Kind::LBrack) {
                if self.at(Kind::RBrack) {
                    let error_span = self.cur_token().span();
                    self.error(diagnostics::ets_array_dimension_required(error_span));
                    dimensions.push(Expression::new_numeric_literal(
                        error_span,
                        0.0,
                        None,
                        NumberBase::Decimal,
                        self,
                    ));
                } else {
                    dimensions.push(
                        self.context_add(Context::In, Self::parse_assignment_expression_or_higher),
                    );
                }
                self.expect(Kind::RBrack);
            }

            if dimensions.len() == 1 {
                return Expression::ETSNewArrayInstanceExpression(
                    ETSNewArrayInstanceExpression::boxed(
                        self.end_span(span),
                        type_annotation,
                        dimensions.remove(0),
                        self,
                    ),
                );
            }

            return Expression::ETSNewMultiDimArrayInstanceExpression(
                ETSNewMultiDimArrayInstanceExpression::boxed(
                    self.end_span(span),
                    type_annotation,
                    dimensions,
                    self,
                ),
            );
        }

        let has_arguments = self.at(Kind::LParen);
        let arguments = if has_arguments {
            let opening_span = self.cur_token().span();
            self.expect(Kind::LParen);
            let (arguments, _) = self.context_add(Context::In, |p| {
                p.parse_delimited_list(
                    Kind::RParen,
                    Kind::Comma,
                    opening_span,
                    Self::parse_call_argument,
                )
            });
            self.expect(Kind::RParen);
            arguments
        } else {
            ArenaVec::new_in(self)
        };

        Expression::ETSNewClassInstanceExpression(ETSNewClassInstanceExpression::boxed(
            self.end_span(span),
            type_annotation,
            arguments,
            has_arguments,
            self,
        ))
    }

    /// Section 13.3 Call Expression
    pub(super) fn parse_call_expression_rest(
        &mut self,
        lhs_span: u32,
        lhs: Expression<'a>,
        in_optional_chain: &mut bool,
    ) -> Expression<'a> {
        let mut lhs = lhs;
        // The caller only enters here with the current token being `(` or `?.`. For `(`,
        // `parse_member_expression_rest` is a no-op (`(` ∉ its FIRST set: `?.`, `.`, `[`,
        // template, TS `!`/`<`), so skip the redundant re-scan on entry; entering on `?.`, and
        // every later iteration (after `Arguments`), still needs it.
        debug_assert!(
            matches!(self.cur_kind(), Kind::LParen | Kind::QuestionDot),
            "parse_call_expression_rest is only entered on `(` or `?.`",
        );
        let mut rescan_members = !self.at(Kind::LParen);
        while self.fatal_error.is_none() {
            if rescan_members {
                lhs = self.parse_member_expression_rest(
                    lhs_span,
                    lhs,
                    in_optional_chain,
                    /* allow_optional_chain */ true,
                );
            }
            rescan_members = true;
            let question_dot_span = self.at(Kind::QuestionDot).then(|| self.cur_token().span());
            let question_dot = question_dot_span.is_some();
            if question_dot {
                self.bump_any();
                *in_optional_chain = true;
            }

            let mut type_arguments = None;
            if question_dot {
                if self.is_ts {
                    if let Some(args) = self.parse_type_arguments_in_expression() {
                        type_arguments = Some(args);
                    } else {
                        // `re_lex_as_typescript_l_angle` may have popped the original token
                        // (e.g. `<<`) from the collected token stream. Rewind restored the
                        // parser's current token, so write it back to the stream.
                        // This is a no-op when tokens are statically disabled (`NoTokensLexerConfig`).
                        self.lexer.rewrite_last_collected_token(self.token);
                    }
                }
                if self.cur_kind().is_template_start_of_tagged_template() {
                    lhs = self.parse_tagged_template(lhs_span, lhs, question_dot, type_arguments);
                    continue;
                }
            }

            if type_arguments.is_some() || self.at(Kind::LParen) {
                if !question_dot && let Expression::TSInstantiationExpression(expr) = lhs {
                    let expr = expr.unbox();
                    type_arguments.replace(expr.type_arguments);
                    lhs = expr.expression;
                }

                lhs = self.parse_call_arguments(lhs_span, lhs, question_dot, type_arguments.take());
                continue;
            }

            if let Some(span) = question_dot_span {
                // We parsed `?.` but then failed to parse anything, so report a missing identifier here.
                let error = diagnostics::identifier_expected_after_question_dot(span);
                return self.fatal_error(error);
            }

            break;
        }

        lhs
    }

    fn parse_call_arguments(
        &mut self,
        lhs_span: u32,
        lhs: Expression<'a>,
        optional: bool,
        type_parameters: Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Expression<'a> {
        // ArgumentList[Yield, Await] :
        //   AssignmentExpression[+In, ?Yield, ?Await]
        let arkui_argument_context =
            if self.source_type.is_arkui() && self.is_in_arkui_dsl_context() {
                self.arkui_argument_context(&lhs)
            } else {
                ArkUIArgumentContext::None
            };
        let opening_span = self.cur_token().span();
        self.expect(Kind::LParen);
        let mut argument_index = 0usize;
        let (call_arguments, trailing_comma) = self.context(Context::In, Context::Decorator, |p| {
            p.parse_delimited_list(Kind::RParen, Kind::Comma, opening_span, |p| {
                let is_ui_callback = match arkui_argument_context {
                    ArkUIArgumentContext::None => false,
                    ArkUIArgumentContext::SkipFirst => argument_index > 0,
                    ArkUIArgumentContext::All => true,
                };
                let previous = p.state.arkui_dsl_next_function;
                p.state.arkui_dsl_next_function = is_ui_callback;
                let argument = p.parse_call_argument();
                p.state.arkui_dsl_next_function = previous;
                argument_index += 1;
                argument
            })
        });
        self.expect(Kind::RParen);

        // ETS stays on the TypeScript path unless the enclosing AST is known to be ArkUI DSL.
        if self.source_type.is_arkui()
            && self.is_in_arkui_dsl_context()
            && (self.at(Kind::LCurly) || self.is_builtin_arkui_component(&lhs))
        {
            return self.parse_arkui_component_expression_after_args(
                lhs_span,
                lhs,
                type_parameters,
                call_arguments,
            );
        }

        let call = Expression::new_call_expression(
            self.end_span(lhs_span),
            lhs,
            type_parameters,
            call_arguments,
            optional,
            self,
        );

        if self.source_type.is_ets_static() && self.at(Kind::LCurly) {
            let is_block_on_new_line = self.cur_token().is_on_new_line();
            let block = self.parse_block();
            let Expression::CallExpression(call) = call else { unreachable!() };
            return Expression::new_ets_trailing_block_expression(
                self.end_span(lhs_span),
                call,
                block,
                false,
                is_block_on_new_line,
                trailing_comma.is_some(),
                self,
            );
        }

        call
    }

    fn parse_call_argument(&mut self) -> Argument<'a> {
        if self.at(Kind::Dot3) {
            Argument::SpreadElement(self.parse_spread_element())
        } else {
            Argument::from(self.parse_assignment_expression_or_higher())
        }
    }

    /// Section 13.4 Update Expression
    fn parse_update_expression(&mut self, lhs_span: u32) -> Expression<'a> {
        let kind = self.cur_kind();
        // ++ -- prefix update expressions
        if kind.is_update_operator() {
            let operator = map_update_operator(kind);
            self.bump_any();
            let argument = self.parse_unary_expression_or_higher(lhs_span);
            self.check_ets_type_value(&argument);
            let argument = SimpleAssignmentTarget::cover(argument, self);
            return Expression::new_update_expression(
                self.end_span(lhs_span),
                operator,
                true,
                argument,
                self,
            );
        }

        if self.source_type.is_jsx() && self.at(Kind::LAngle) {
            return self.parse_jsx_expression();
        }

        let span = self.start_span();
        let lhs = self.parse_lhs_expression_or_higher();
        // ++ -- postfix update expressions
        let post_kind = self.cur_kind();
        if post_kind.is_update_operator() && !self.cur_token().is_on_new_line() {
            let operator = map_update_operator(post_kind);
            self.bump_any();
            self.check_ets_type_value(&lhs);
            let lhs = SimpleAssignmentTarget::cover(lhs, self);
            return Expression::new_update_expression(
                self.end_span(span),
                operator,
                false,
                lhs,
                self,
            );
        }
        lhs
    }

    /// Section 13.5 Unary Expression
    pub(crate) fn parse_unary_expression_or_higher(&mut self, lhs_span: u32) -> Expression<'a> {
        match self.cur_kind() {
            // `UnaryExpression : (delete | void | typeof | + | - | ~ | !) UnaryExpression`
            // e.g. `!x`, `-1`, `typeof y`, `void 0`, `delete a.b`
            kind if kind.is_unary_operator() => self.parse_unary_expression(),
            // TS type assertion, a modified `UnaryExpression`: `< Type > UnaryExpression`, e.g. `<T>x`.
            // In a non-JSX, non-TS file a leading `<` is instead a JSX-in-non-JSX error, e.g. `<div/>`.
            // (`<` in a JSX file is not matched here; it falls through to the `UpdateExpression` arm,
            // which parses the JSX element.)
            Kind::LAngle if !self.source_type.is_jsx() => {
                if self.is_ts {
                    self.parse_ts_type_assertion()
                } else {
                    self.parse_jsx_in_non_jsx_error()
                }
            }
            // `UnaryExpression : [+Await] AwaitExpression`, with `AwaitExpression : await UnaryExpression`
            // e.g. `await foo`
            Kind::Await => self.parse_await_expression(lhs_span),
            // `UnaryExpression : UpdateExpression` — a `LeftHandSideExpression` with an optional
            // prefix or postfix `++`/`--`. e.g. `a`, `f()`, `a++`, `++a`
            _ => self.parse_update_expression(lhs_span),
        }
    }

    pub(crate) fn parse_simple_unary_expression(&mut self, lhs_span: u32) -> Expression<'a> {
        match self.cur_kind() {
            kind if kind.is_unary_operator() => self.parse_unary_expression(),
            Kind::LAngle => {
                if self.source_type.is_jsx() {
                    return self.parse_jsx_expression();
                }
                if self.is_ts {
                    return self.parse_ts_type_assertion();
                }
                self.parse_jsx_in_non_jsx_error()
            }
            Kind::Await => self.parse_await_expression(lhs_span),
            _ => self.parse_update_expression(lhs_span),
        }
    }

    /// A leading `<` in a file where JSX is disabled and which is not TypeScript (so it is not a
    /// type assertion either) is always an error. Report it against the `<` token with a help to
    /// enable JSX in the parser options.
    fn parse_jsx_in_non_jsx_error(&mut self) -> Expression<'a> {
        self.fatal_error(diagnostics::jsx_in_non_jsx(self.cur_token().span()))
    }

    fn parse_unary_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        let operator = map_unary_operator(self.cur_kind());
        self.bump_any();
        let pure_comment_index = self.lexer.trivia_builder.previous_token_has_pure_comment();
        let mut argument = self.parse_simple_unary_expression(self.start_span());
        if let Some(index) = pure_comment_index
            && !Self::set_pure_on_call_or_new_expr(&mut argument)
        {
            self.lexer.trivia_builder.mark_pure_comment_not_applied(index);
        }
        Expression::new_unary_expression(self.end_span(span), operator, argument, self)
    }

    pub(crate) fn parse_binary_expression_or_higher(
        &mut self,
        lhs_precedence: Precedence,
    ) -> Expression<'a> {
        let lhs_span = self.start_span();

        let lhs_parenthesized = self.at(Kind::LParen);
        // [+In] PrivateIdentifier in ShiftExpression[?Yield, ?Await]
        let lhs = if self.ctx.has_in() && self.at(Kind::PrivateIdentifier) {
            self.parse_private_in_expression(lhs_span, lhs_precedence)
        } else {
            let has_pure_comment =
                self.lexer.trivia_builder.previous_token_has_pure_comment().is_some();
            let mut expr = self.parse_unary_expression_or_higher(lhs_span);
            if has_pure_comment {
                Self::set_pure_on_call_or_new_expr(&mut expr);
            }
            expr
        };

        self.parse_binary_expression_rest(lhs_span, lhs, lhs_parenthesized, lhs_precedence)
    }

    /// Section 13.6 - 13.13 Binary Expression
    fn parse_binary_expression_rest(
        &mut self,
        lhs_span: u32,
        lhs: Expression<'a>,
        lhs_parenthesized: bool,
        min_precedence: Precedence,
    ) -> Expression<'a> {
        // Pratt Parsing Algorithm
        // <https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html>
        let mut lhs = lhs;
        // Precedence of the operator that produced the running left-hand operand, used to detect
        // `as`/`satisfies` assertions that cannot be erased. `None` until a binary/logical operator
        // has been consumed in this loop, matching TypeScript where the initial operand (from unary
        // parsing) is never a binary expression.
        let mut last_operand_precedence: Option<Precedence> = None;
        loop {
            // re-lex for `>=` `>>` `>>>`
            // This is needed for jsx `<div>=</div>` case
            let kind = self.re_lex_right_angle();

            let Some(left_precedence) = kind_to_precedence(kind) else { break };

            let stop = if left_precedence.is_right_associative() {
                left_precedence < min_precedence
            } else {
                left_precedence <= min_precedence
            };

            if stop {
                break;
            }

            // Omit the In keyword for the grammar in 13.10 Relational Operators
            // RelationalExpression[In, Yield, Await] :
            // [+In] RelationalExpression[+In, ?Yield, ?Await] in ShiftExpression[?Yield, ?Await]
            if kind == Kind::In && !self.ctx.has_in() {
                break;
            }

            if matches!(kind, Kind::As | Kind::Satisfies) {
                if self.cur_token().is_on_new_line() {
                    break;
                }
                self.bump_any();
                if self.source_type.is_ets_static()
                    && (kind == Kind::Satisfies || self.at(Kind::Const))
                {
                    let feature = if kind == Kind::Satisfies {
                        "The `satisfies` operator"
                    } else {
                        "`as const` assertions"
                    };
                    self.error(diagnostics::ets_unsupported_syntax(
                        feature,
                        self.cur_token().span(),
                    ));
                }
                let type_annotation = self.parse_ts_type();
                let span = self.end_span(lhs_span);
                lhs = if kind == Kind::As {
                    if !self.is_ts {
                        self.error(diagnostics::as_in_ts(span));
                    }
                    Expression::new_ts_as_expression(span, lhs, type_annotation, self)
                } else {
                    if !self.is_ts {
                        self.error(diagnostics::satisfies_in_ts(span));
                    }
                    Expression::new_ts_satisfies_expression(span, lhs, type_annotation, self)
                };
                // When we have `a ## b as T` or `a ## b satisfies T`, where `##` is some binary
                // operator, stop parsing on any following operator with higher precedence than `##`
                // because continuing would make it impossible to erase the `as` or `satisfies`
                // without changing the meaning of the expression.
                // See <https://github.com/microsoft/TypeScript/issues/63527>.
                if let Some(last_precedence) = last_operand_precedence
                    && let Some(next_precedence) = kind_to_precedence(self.re_lex_right_angle())
                    && next_precedence > last_precedence
                {
                    break;
                }
                continue;
            }

            self.bump_any(); // bump operator

            if self.source_type.is_ets_static() && kind == Kind::Instanceof {
                self.check_ets_type_value(&lhs);
                let right = self.parse_ts_type();
                lhs = Expression::ETSInstanceOfExpression(ETSInstanceOfExpression::boxed(
                    self.end_span(lhs_span),
                    lhs,
                    right,
                    self,
                ));
                last_operand_precedence = Some(left_precedence);
                continue;
            }

            let rhs_parenthesized = self.at(Kind::LParen);
            let rhs = self.parse_binary_expression_or_higher(left_precedence);

            lhs = if kind.is_logical_operator() {
                let span = self.end_span(lhs_span);
                let op = map_logical_operator(kind);
                if self.source_type.is_ets_static() {
                    self.check_ets_type_value(&lhs);
                    self.check_ets_type_value(&rhs);
                }
                // check mixed coalesce
                if op == LogicalOperator::Coalesce {
                    let mut maybe_mixed_coalesce_expr = None;
                    if let Expression::LogicalExpression(rhs) = &rhs {
                        if !rhs_parenthesized {
                            maybe_mixed_coalesce_expr = Some(rhs);
                        }
                    } else if let Expression::LogicalExpression(lhs) = &lhs
                        && !lhs_parenthesized
                    {
                        maybe_mixed_coalesce_expr = Some(lhs);
                    }
                    if let Some(expr) = maybe_mixed_coalesce_expr
                        && matches!(expr.operator, LogicalOperator::And | LogicalOperator::Or)
                    {
                        self.error(diagnostics::mixed_coalesce(span));
                    }
                }
                Expression::new_logical_expression(span, lhs, op, rhs, self)
            } else if kind.is_binary_operator() {
                let span = self.end_span(lhs_span);
                let op = map_binary_operator(kind);
                if self.source_type.is_ets_static() {
                    self.check_ets_type_value(&lhs);
                    self.check_ets_type_value(&rhs);
                }
                if self.source_type.is_ets_static()
                    && matches!(op, BinaryOperator::Division | BinaryOperator::Remainder)
                    && rhs.is_number_0()
                    && Self::ets_expression_is_integer(&lhs, self.source_text)
                {
                    self.error(diagnostics::ets_unsupported_syntax(
                        "Division by zero in integer arithmetic",
                        rhs.span(),
                    ));
                }
                if op == BinaryOperator::Exponential && !lhs_parenthesized {
                    let diagnostic = match &lhs {
                        Expression::AwaitExpression(_) => Some(
                            diagnostics::unary_exponentiation_left_operand("await", lhs.span()),
                        ),
                        Expression::UnaryExpression(unary) => {
                            Some(diagnostics::unary_exponentiation_left_operand(
                                unary.operator.as_str(),
                                lhs.span(),
                            ))
                        }
                        Expression::TSTypeAssertion(_) => Some(
                            diagnostics::type_assertion_exponentiation_left_operand(lhs.span()),
                        ),
                        _ => None,
                    };
                    if let Some(diagnostic) = diagnostic {
                        self.error(diagnostic);
                    }
                }
                Expression::new_binary_expression(span, lhs, op, rhs, self)
            } else {
                break;
            };
            last_operand_precedence = Some(left_precedence);
        }

        lhs
    }

    fn ets_expression_is_integer(expression: &Expression<'a>, source_text: &str) -> bool {
        match expression {
            Expression::NumericLiteral(literal) => {
                let text = &source_text[literal.span.start as usize..literal.span.end as usize];
                !text.bytes().any(|byte| matches!(byte, b'.' | b'e' | b'E' | b'f'))
            }
            Expression::UnaryExpression(unary) => {
                Self::ets_expression_is_integer(&unary.argument, source_text)
            }
            Expression::BinaryExpression(binary) => {
                Self::ets_expression_is_integer(&binary.left, source_text)
                    && Self::ets_expression_is_integer(&binary.right, source_text)
            }
            Expression::ParenthesizedExpression(parenthesized) => {
                Self::ets_expression_is_integer(&parenthesized.expression, source_text)
            }
            _ => false,
        }
    }

    /// Section 13.14 Conditional Expression
    /// `ConditionalExpression`[In, Yield, Await] :
    ///     `ShortCircuitExpression`[?In, ?Yield, ?Await]
    ///     `ShortCircuitExpression`[?In, ?Yield, ?Await] ? `AssignmentExpression`[+In, ?Yield, ?Await] : `AssignmentExpression`[?In, ?Yield, ?Await]
    fn parse_conditional_expression_rest(
        &mut self,
        lhs_span: u32,
        lhs: Expression<'a>,
        allow_return_type_in_arrow_function: bool,
    ) -> Expression<'a> {
        let question_span = self.token.span();
        if !self.eat(Kind::Question) {
            return lhs;
        }
        let consequent = self.context_add(Context::In, |p| {
            p.parse_assignment_expression_or_higher_impl(
                /* allow_return_type_in_arrow_function */ false,
            )
        });
        self.expect_conditional_alternative(question_span);
        let alternate =
            self.parse_assignment_expression_or_higher_impl(allow_return_type_in_arrow_function);
        Expression::new_conditional_expression(
            self.end_span(lhs_span),
            lhs,
            consequent,
            alternate,
            self,
        )
    }

    /// `AssignmentExpression`[In, Yield, Await] :
    pub(crate) fn parse_assignment_expression_or_higher(&mut self) -> Expression<'a> {
        self.parse_assignment_expression_or_higher_impl(
            /* allow_return_type_in_arrow_function */ true,
        )
    }

    pub(crate) fn parse_assignment_expression_or_higher_impl(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Expression<'a> {
        let has_no_side_effects_comment =
            self.lexer.trivia_builder.previous_token_has_no_side_effects_comment();
        let pure_comment_index = self.lexer.trivia_builder.previous_token_has_pure_comment();
        // [+Yield] YieldExpression
        if self.is_yield_expression() {
            return self.parse_yield_expression();
        }
        // `() => {}`, `(x) => {}`
        if let Some(mut arrow_expr) = self
            .try_parse_parenthesized_arrow_function_expression(allow_return_type_in_arrow_function)
        {
            if has_no_side_effects_comment
                && let Expression::ArrowFunctionExpression(func) = &mut arrow_expr
            {
                func.pure = true;
            }
            return arrow_expr;
        }
        // `async x => {}`
        if let Some(mut arrow_expr) = self
            .try_parse_async_simple_arrow_function_expression(allow_return_type_in_arrow_function)
        {
            if has_no_side_effects_comment
                && let Expression::ArrowFunctionExpression(func) = &mut arrow_expr
            {
                func.pure = true;
            }
            return arrow_expr;
        }

        let span = self.start_span();
        let lhs_parenthesized = self.at(Kind::LParen);
        let lhs = self.parse_binary_expression_or_higher(Precedence::Comma);
        let lhs_parenthesized_span = lhs_parenthesized.then(|| self.end_span(span));
        let kind = self.cur_kind();

        // `x => {}`
        if kind == Kind::Arrow
            && let Expression::Identifier(ident) = &lhs
        {
            let mut arrow_expr = self.parse_simple_arrow_function_expression(
                span,
                ident,
                /* async */ false,
                allow_return_type_in_arrow_function,
            );
            if has_no_side_effects_comment
                && let Expression::ArrowFunctionExpression(func) = &mut arrow_expr
            {
                func.pure = true;
            }
            return arrow_expr;
        }

        if kind.is_assignment_operator() {
            return self.parse_assignment_expression_recursive(
                span,
                lhs,
                lhs_parenthesized_span,
                allow_return_type_in_arrow_function,
            );
        }

        let mut expr =
            self.parse_conditional_expression_rest(span, lhs, allow_return_type_in_arrow_function);

        if let Some(index) = pure_comment_index
            && !Self::set_pure_on_call_or_new_expr(&mut expr)
        {
            self.lexer.trivia_builder.mark_pure_comment_not_applied(index);
        }

        if has_no_side_effects_comment {
            Self::set_pure_on_function_expr(&mut expr);
        }

        expr
    }

    fn set_pure_on_call_or_new_expr(expr: &mut Expression<'a>) -> bool {
        match &mut expr.get_inner_expression_mut() {
            Expression::CallExpression(call_expr) => {
                call_expr.pure = true;
                true
            }
            Expression::NewExpression(new_expr) => {
                new_expr.pure = true;
                true
            }
            Expression::BinaryExpression(binary_expr) => {
                Self::set_pure_on_call_or_new_expr(&mut binary_expr.left)
            }
            Expression::LogicalExpression(logical_expr) => {
                Self::set_pure_on_call_or_new_expr(&mut logical_expr.left)
            }
            Expression::ConditionalExpression(conditional_expr) => {
                Self::set_pure_on_call_or_new_expr(&mut conditional_expr.test)
            }
            // Recurse through member-access chains: `/* #__PURE__ */ foo().a.b.c`
            // applies PURE to the underlying call/new (Rollup/esbuild semantics).
            expr @ match_member_expression!(Expression) => {
                Self::set_pure_on_call_or_new_expr(expr.to_member_expression_mut().object_mut())
            }
            Expression::ChainExpression(chain_expr) => match &mut chain_expr.expression {
                ChainElement::CallExpression(call_expr) => {
                    call_expr.pure = true;
                    true
                }
                element @ match_member_expression!(ChainElement) => {
                    Self::set_pure_on_call_or_new_expr(
                        element.to_member_expression_mut().object_mut(),
                    )
                }
                ChainElement::TSNonNullExpression(non_null_expr) => {
                    Self::set_pure_on_call_or_new_expr(&mut non_null_expr.expression)
                }
            },
            _ => false,
        }
    }

    pub(crate) fn set_pure_on_function_expr(expr: &mut Expression<'a>) {
        match expr {
            Expression::FunctionExpression(func) => {
                func.pure = true;
            }
            Expression::ArrowFunctionExpression(func) => {
                func.pure = true;
            }
            _ => {}
        }
    }

    fn parse_assignment_expression_recursive(
        &mut self,
        span: u32,
        lhs: Expression<'a>,
        left_parenthesized_span: Option<Span>,
        allow_return_type_in_arrow_function: bool,
    ) -> Expression<'a> {
        let operator = map_assignment_operator(self.cur_kind());
        if self.source_type.is_ets_static() {
            self.check_ets_type_value(&lhs);
        }
        if self.source_type.is_ets_static() && operator.is_logical() {
            self.error(diagnostics::ets_unsupported_syntax(
                "Logical assignment operators",
                self.cur_token().span(),
            ));
        }
        // 13.15.5 Destructuring Assignment
        // LeftHandSideExpression = AssignmentExpression
        // is converted to
        // AssignmentPattern[Yield, Await] :
        //    ObjectAssignmentPattern
        //    ArrayAssignmentPattern
        if let Some(span) = left_parenthesized_span {
            //  `({}) = x;`, `([]) = x;`
            if matches!(lhs, Expression::ObjectExpression(_) | Expression::ArrayExpression(_)) {
                self.error(diagnostics::invalid_assignment(span));
            }
        }
        // A destructuring pattern target is only valid with `=`, not a compound operator.
        if operator != AssignmentOperator::Assign
            && matches!(lhs, Expression::ObjectExpression(_) | Expression::ArrayExpression(_))
        {
            self.error(diagnostics::assignment_is_not_simple(lhs.span()));
        }
        let left = AssignmentTarget::cover(lhs, self);
        self.bump_any();
        let right =
            self.parse_assignment_expression_or_higher_impl(allow_return_type_in_arrow_function);
        if self.source_type.is_ets_static() {
            self.check_ets_type_value(&right);
        }
        Expression::new_assignment_expression(self.end_span(span), operator, left, right, self)
    }

    /// Section 13.16 Sequence Expression
    fn parse_sequence_expression(
        &mut self,
        span: u32,
        first_expression: Expression<'a>,
    ) -> Expression<'a> {
        if self.source_type.is_ets_static() && !self.ctx.ets_allows_sequence() {
            self.error(diagnostics::ets_unsupported_syntax(
                "The comma operator outside a `for` statement",
                self.cur_token().span(),
            ));
        }
        let mut expressions = ArenaVec::with_capacity_in(2, self);
        expressions.push(first_expression);
        while self.eat(Kind::Comma) {
            let expression = self.parse_assignment_expression_or_higher();
            expressions.push(expression);
        }
        Expression::new_sequence_expression(self.end_span(span), expressions, self)
    }

    /// Check if the current `await` token is unambiguously an await expression.
    ///
    /// Based on Babel's `isAmbiguousPrefixOrIdentifier` (inverted) and
    /// TypeScript's `nextTokenIsIdentifierOrKeywordOrLiteralOnSameLine`.
    ///
    /// Returns `true` when await is definitely an await expression (not ambiguous).
    ///
    /// Unambiguous cases (returns `true`):
    /// - Next token is identifier, keyword (except `of`), or literal on same line
    ///
    /// Ambiguous cases (returns `false`):
    /// - Line break after `await` (could be ASI)
    /// - Next token is `+` `-` (could be binary operator or unary prefix)
    /// - Next token is `(` `[` (could be call/member or grouping/array)
    /// - Next token is template literal
    /// - Next token is `of` (for-await-of ambiguity: `for (await of [])`)
    /// - Next token is `/` (division or regex literal)
    /// - Next token cannot start an expression (`)`, `}`, `;`, etc.)
    fn is_unambiguous_await(&mut self) -> bool {
        let token = self.lexer.peek_token();

        // Line break after await makes it ambiguous (could be ASI)
        if token.is_on_new_line() {
            return false;
        }

        let kind = token.kind();

        // Special case: `await of` is ambiguous (for-await-of loop)
        // Special case: `await using` should be handled as a declaration, not `await (using)`
        if matches!(kind, Kind::Of | Kind::Using) {
            return false;
        }

        // Returns true for identifiers, keywords, and literals (not binary operators)
        kind.is_after_await_or_yield()
    }

    /// ``AwaitExpression`[Yield]` :
    ///     await `UnaryExpression`[?Yield, +Await]
    fn parse_await_expression(&mut self, lhs_span: u32) -> Expression<'a> {
        // es2panda parses `await` independently of whether the enclosing
        // function is marked `async`; contextual validity is checked after
        // parsing. Keep the JavaScript/TypeScript diagnostics unchanged.
        if self.source_type.is_ets_static() {
            let span = self.start_span();
            self.bump_any(); // consume `await`
            let argument = self.context_add(Context::Await, |p| {
                p.parse_unary_expression_or_higher(p.start_span())
            });
            return Expression::new_await_expression(self.end_span(span), argument, self);
        }

        // Case 1: In await context (async function, module top-level, unambiguous mode top-level)
        // Always parse as await expression
        if self.ctx.has_await() {
            let span = self.start_span();
            self.bump_any(); // consume `await`
            let argument = self.parse_unary_expression_or_higher(self.start_span());
            return Expression::new_await_expression(self.end_span(span), argument, self);
        }

        // Case 2: Not in await context, but unambiguously an await expression
        // Parse as await expression and report error for better diagnostics
        // This matches Babel's behavior: report "await only allowed in async" error
        //
        // At top level in unambiguous mode: unambiguous await upgrades the file to ESM
        // (like Babel's `sawUnambiguousESM`). We defer the error with `error_on_script` -
        // it will be discarded when we upgrade to ESM.
        //
        // Inside a function: await is always invalid in non-async functions, even in ESM.
        // Report error immediately.
        if self.is_unambiguous_await() {
            let span = self.start_span();

            if self.ctx.has_top_level() {
                // At top level - upgrade to ESM immediately (like Babel's `sawUnambiguousESM`)
                self.module_record_builder.set_module_syntax();
                // Defer error - will be discarded when we upgrade to ESM
                self.error_on_script(diagnostics::await_expression(self.cur_token().span()));
            } else {
                // Inside a function - await is always invalid in non-async function
                self.error(diagnostics::await_expression(self.cur_token().span()));
            }

            self.bump_any(); // consume `await`
            // Parse argument with await context enabled for this expression
            self.ctx = self.ctx.and_await(true);
            let argument = self.parse_unary_expression_or_higher(self.start_span());
            self.ctx = self.ctx.and_await(false);
            return Expression::new_await_expression(self.end_span(span), argument, self);
        }

        // Case 3: Ambiguous - parse `await` as identifier
        // This applies to scripts where `await` might be identifier or keyword
        // The statement-level checkpoint system handles reparsing if ESM detected
        self.parse_update_expression(lhs_span)
    }

    fn parse_decorated_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        let decorators = self.parse_decorators();
        let modifiers = self.parse_modifiers(false, false);
        if self.at(Kind::Class) {
            self.parse_class_expression(span, &modifiers, decorators)
        } else {
            self.unexpected()
        }
    }

    pub(crate) fn parse_decorators(&mut self) -> ArenaVec<'a, Decorator<'a>> {
        if self.at(Kind::At) {
            let mut decorators = ArenaVec::with_capacity_in(1, self);
            while self.at(Kind::At) && !self.at_arkts_annotation_declaration() {
                decorators.push(self.parse_decorator());
            }
            decorators
        } else {
            ArenaVec::new_in(self)
        }
    }

    /// `Decorator`[Yield, Await]:
    ///   `DecoratorMemberExpression`[?Yield, ?Await]
    ///   ( `Expression`[+In, ?Yield, ?Await] )
    ///   `DecoratorCallExpression`
    pub(crate) fn parse_decorator(&mut self) -> Decorator<'a> {
        let span = self.start_span();
        self.bump_any(); // bump @
        let expr = self.context_add(Context::Decorator, Self::parse_lhs_expression_or_higher);
        if self.source_type.is_ets_static() {
            self.check_ets_annotation_usage(&expr);
        }
        Decorator::new(self.end_span(span), expr, self)
    }

    fn is_yield_expression(&mut self) -> bool {
        if self.at(Kind::Yield) {
            if self.ctx.has_yield() {
                return true;
            }
            // Outside a generator, `yield` is an `IdentifierReference` unless the next token (on the
            // same line) can start the operand of a `YieldExpression`. That is a single-token
            // decision, so peek instead of running a full `lookahead` checkpoint/rewind. With
            // `is_await = false` the worker reduces to exactly this (mirrors `is_unambiguous_await`).
            let token = self.lexer.peek_token();
            return !token.is_on_new_line() && token.kind().is_after_await_or_yield();
        }
        false
    }
}
