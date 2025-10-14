use cow_utils::CowUtils;
use oxc_allocator::{Box, TakeIn, Vec};
use oxc_ast::ast::*;
#[cfg(feature = "regular_expression")]
use oxc_regular_expression::ast::Pattern;
use oxc_span::{Atom, GetSpan, Span};
use oxc_syntax::{
    number::{BigintBase, NumberBase},
    precedence::Precedence,
};

use super::{
    grammar::CoverGrammar,
    operator::{
        kind_to_precedence, map_assignment_operator, map_binary_operator, map_logical_operator,
        map_unary_operator, map_update_operator,
    },
};
use crate::{
    Context, ParserImpl, diagnostics,
    lexer::{Kind, parse_big_int, parse_float, parse_int},
    modifiers::Modifiers,
};

impl<'a> ParserImpl<'a> {
    pub(crate) fn parse_paren_expression(&mut self) -> Expression<'a> {
        self.expect(Kind::LParen);
        let expression = self.parse_expr();
        self.expect(Kind::RParen);
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
        self.check_identifier(kind, self.ctx);
        let (span, name) = self.parse_identifier_kind(Kind::Ident);
        self.ast.identifier_reference(span, name)
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
        self.ast.binding_identifier(span, name)
    }

    pub(crate) fn parse_label_identifier(&mut self) -> LabelIdentifier<'a> {
        let kind = self.cur_kind();
        if !kind.is_label_identifier(self.ctx.has_yield(), self.ctx.has_await()) {
            return self.unexpected();
        }
        self.check_identifier(kind, self.ctx);
        let (span, name) = self.parse_identifier_kind(Kind::Ident);
        self.ast.label_identifier(span, name)
    }

    pub(crate) fn parse_identifier_name(&mut self) -> IdentifierName<'a> {
        if !self.cur_kind().is_identifier_name() {
            return self.unexpected();
        }
        let (span, name) = self.parse_identifier_kind(Kind::Ident);
        self.ast.identifier_name(span, name)
    }

    /// Parse keyword kind as identifier
    pub(crate) fn parse_keyword_identifier(&mut self, kind: Kind) -> IdentifierName<'a> {
        let (span, name) = self.parse_identifier_kind(kind);
        self.ast.identifier_name(span, name)
    }

    #[inline]
    pub(crate) fn parse_identifier_kind(&mut self, kind: Kind) -> (Span, Atom<'a>) {
        let span = self.cur_token().span();
        let name = self.cur_string();
        self.bump_remap(kind);
        (span, Atom::from(name))
    }

    pub(crate) fn check_identifier(&mut self, kind: Kind, ctx: Context) {
        // It is a Syntax Error if this production has an [Await] parameter.
        if ctx.has_await() && kind == Kind::Await {
            self.error(diagnostics::identifier_async("await", self.cur_token().span()));
        }
        // It is a Syntax Error if this production has a [Yield] parameter.
        if ctx.has_yield() && kind == Kind::Yield {
            self.error(diagnostics::identifier_generator("yield", self.cur_token().span()));
        }
    }

    /// Section [PrivateIdentifier](https://tc39.es/ecma262/#prod-PrivateIdentifier)
    /// `PrivateIdentifier` ::
    ///     # `IdentifierName`
    /// # Panics
    pub(crate) fn parse_private_identifier(&mut self) -> PrivateIdentifier<'a> {
        let span = self.cur_token().span();
        let name = Atom::from(self.cur_string());
        self.bump_any();
        self.ast.private_identifier(span, name)
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
        // FunctionExpression, GeneratorExpression
        // AsyncFunctionExpression, AsyncGeneratorExpression
        if self.at_function_with_async() {
            let span = self.start_span();
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
                self.parse_class_expression(self.start_span(), &Modifiers::empty(), self.ast.vec())
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
            Kind::Slash | Kind::SlashEq => {
                let literal = self.parse_literal_regexp();
                Expression::RegExpLiteral(self.alloc(literal))
            }
            Kind::At => self.parse_decorated_expression(),
            // Literal, RegularExpressionLiteral
            kind if kind.is_literal() => self.parse_literal_expression(),
            _ => self.parse_identifier_expression(),
        }
    }

    fn parse_parenthesized_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        self.bump_any(); // `bump` `(`
        let expr_span = self.start_span();
        let (mut expressions, comma_span) = self.context(Context::In, Context::Decorator, |p| {
            p.parse_delimited_list(
                Kind::RParen,
                Kind::Comma,
                Self::parse_assignment_expression_or_higher,
            )
        });

        if let Some(comma_span) = comma_span {
            let error = diagnostics::expect_token(")", ",", self.end_span(comma_span));
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
            self.ast.expression_sequence(expr_span, expressions)
        };

        match &mut expression {
            Expression::ArrowFunctionExpression(arrow_expr) => arrow_expr.pife = true,
            Expression::FunctionExpression(func_expr) => func_expr.pife = true,
            _ => {}
        }

        if self.options.preserve_parens {
            self.ast.expression_parenthesized(self.end_span(span), expression)
        } else {
            expression
        }
    }

    /// Section 13.2.2 This Expression
    fn parse_this_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        self.bump_any();
        self.ast.expression_this(self.end_span(span))
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
            Kind::True | Kind::False => {
                let lit = self.parse_literal_boolean();
                Expression::BooleanLiteral(self.alloc(lit))
            }
            Kind::Null => {
                let lit = self.parse_literal_null();
                Expression::NullLiteral(self.alloc(lit))
            }
            Kind::DecimalBigInt | Kind::BinaryBigInt | Kind::OctalBigInt | Kind::HexBigInt => {
                let lit = self.parse_literal_bigint();
                Expression::BigIntLiteral(self.alloc(lit))
            }
            kind if kind.is_number() => {
                let lit = self.parse_literal_number();
                Expression::NumericLiteral(self.alloc(lit))
            }
            _ => self.unexpected(),
        }
    }

    pub(crate) fn parse_literal_boolean(&mut self) -> BooleanLiteral {
        let span = self.start_span();
        let value = match self.cur_kind() {
            Kind::True => true,
            Kind::False => false,
            _ => return self.unexpected(),
        };
        self.bump_any();
        self.ast.boolean_literal(self.end_span(span), value)
    }

    pub(crate) fn parse_literal_null(&mut self) -> NullLiteral {
        let span = self.cur_token().span();
        self.bump_any(); // bump `null`
        self.ast.null_literal(span)
    }

    pub(crate) fn parse_literal_number(&mut self) -> NumericLiteral<'a> {
        let token = self.cur_token();
        let span = token.span();
        let kind = token.kind();
        let src = self.cur_src();
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
        self.ast.numeric_literal(span, value, Some(Atom::from(src)), base)
    }

    pub(crate) fn parse_literal_bigint(&mut self) -> BigIntLiteral<'a> {
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
        let value = parse_big_int(src, number_kind, has_separator, self.ast.allocator);

        self.bump_any();
        self.ast.big_int_literal(span, value, Some(Atom::from(raw)), base)
    }

    pub(crate) fn parse_literal_regexp(&mut self) -> RegExpLiteral<'a> {
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

        let pattern = RegExpPattern { text: Atom::from(pattern_text), pattern };

        if flags.contains(RegExpFlags::U | RegExpFlags::V) {
            self.error(diagnostics::reg_exp_flag_u_and_v(span));
        }

        self.ast.reg_exp_literal(span, RegExp { pattern, flags }, Some(Atom::from(raw)))
    }

    #[cfg(feature = "regular_expression")]
    fn parse_regex_pattern(
        &mut self,
        pattern_span_offset: u32,
        pattern: &'a str,
        flags_span_offset: u32,
        flags: &'a str,
    ) -> Option<Box<'a, Pattern<'a>>> {
        use oxc_regular_expression::{LiteralParser, Options};
        match LiteralParser::new(
            self.ast.allocator,
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
        let raw = Atom::from(self.cur_src());
        let value = self.cur_string();
        let lone_surrogates = self.cur_token().lone_surrogates();
        self.bump_any();
        self.ast.string_literal_with_lone_surrogates(span, value, Some(raw), lone_surrogates)
    }

    /// Section [Array Expression](https://tc39.es/ecma262/#prod-ArrayLiteral)
    /// `ArrayLiteral`[Yield, Await]:
    ///     [ Elision opt ]
    ///     [ `ElementList`[?Yield, ?Await] ]
    ///     [ `ElementList`[?Yield, ?Await] , Elisionopt ]
    pub(crate) fn parse_array_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        self.expect(Kind::LBrack);
        let (elements, comma_span) = self.context_add(Context::In, |p| {
            p.parse_delimited_list(Kind::RBrack, Kind::Comma, Self::parse_array_expression_element)
        });
        if let Some(comma_span) = comma_span {
            self.state.trailing_commas.insert(span, self.end_span(comma_span));
        }
        self.expect(Kind::RBrack);
        self.ast.expression_array(self.end_span(span), elements)
    }

    fn parse_array_expression_element(&mut self) -> ArrayExpressionElement<'a> {
        match self.cur_kind() {
            Kind::Comma => self.parse_elision(),
            Kind::Dot3 => ArrayExpressionElement::SpreadElement(self.parse_spread_element()),
            _ => ArrayExpressionElement::from(self.parse_assignment_expression_or_higher()),
        }
    }

    /// Elision :
    ///     ,
    ///    Elision ,
    pub(crate) fn parse_elision(&self) -> ArrayExpressionElement<'a> {
        self.ast.array_expression_element_elision(self.cur_token().span())
    }

    /// Section [Template Literal](https://tc39.es/ecma262/#prod-TemplateLiteral)
    /// `TemplateLiteral`[Yield, Await, Tagged] :
    ///     `NoSubstitutionTemplate`
    ///     `SubstitutionTemplate`[?Yield, ?Await, ?Tagged]
    pub(crate) fn parse_template_literal(&mut self, tagged: bool) -> TemplateLiteral<'a> {
        let span = self.start_span();

        let (quasis, expressions) = match self.cur_kind() {
            Kind::NoSubstitutionTemplate => {
                let quasis = self.ast.vec1(self.parse_template_element(tagged));
                (quasis, self.ast.vec())
            }
            Kind::TemplateHead => {
                let mut expressions = self.ast.vec_with_capacity(1);
                let mut quasis = self.ast.vec_with_capacity(2);

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

        self.ast.template_literal(self.end_span(span), quasis, expressions)
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
        type_arguments: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
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
        self.ast.expression_tagged_template(span, lhs, type_arguments, quasi)
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
        let mut raw = Atom::from(
            &self.source_text[raw_span.start as usize + 1..(raw_span.end - end_offset) as usize],
        );

        // Get `cooked`.
        // Also replace `\r` with `\n` in `raw`.
        // If contains `\r`, then `escaped` must be `true` (because `\r` needs unescaping),
        // so we can skip searching for `\r` in common case where contains no escapes.
        let (cooked, lone_surrogates) = if self.cur_token().escaped() {
            // `cooked = None` when template literal has invalid escape sequence
            let cooked = self.cur_template_string().map(Atom::from);
            if cooked.is_some() && raw.contains('\r') {
                raw = self.ast.atom(&raw.cow_replace("\r\n", "\n").cow_replace('\r', "\n"));
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
        self.ast.template_element_with_lone_surrogates(
            span,
            TemplateElementValue { raw, cooked },
            tail,
            lone_surrogates,
        )
    }

    /// Section 13.3 ImportCall or ImportMeta
    fn parse_import_meta_or_call(&mut self) -> Expression<'a> {
        let span = self.start_span();
        let meta = self.parse_keyword_identifier(Kind::Import);
        match self.cur_kind() {
            Kind::Dot => {
                self.bump_any(); // bump `.`
                match self.cur_kind() {
                    // `import.meta`
                    Kind::Meta => {
                        let property = self.parse_keyword_identifier(Kind::Meta);
                        let span = self.end_span(span);
                        self.module_record_builder.visit_import_meta(span);
                        self.ast.expression_meta_property(span, meta, property)
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
                        self.fatal_error(diagnostics::import_meta(self.end_span(span)))
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

        self.expect(Kind::LParen);
        let (arguments, _) = self.context(Context::In, Context::Decorator, |p| {
            p.parse_delimited_list(Kind::RParen, Kind::Comma, Self::parse_v8_intrinsic_argument)
        });
        self.expect(Kind::RParen);
        self.ast.expression_v_8_intrinsic(self.end_span(span), name, arguments)
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
        let lhs = self.parse_member_expression_or_higher(&mut in_optional_chain);
        let lhs = self.parse_call_expression_rest(span, lhs, &mut in_optional_chain);
        if !in_optional_chain {
            return lhs;
        }
        if self.ctx.has_decorator() {
            self.error(diagnostics::decorator_optional(lhs.span()));
        }
        // Add `ChainExpression` to `a?.c?.b<c>`;
        if let Expression::TSInstantiationExpression(mut expr) = lhs {
            expr.expression = self
                .map_to_chain_expression(expr.expression.span(), expr.expression.take_in(self.ast));
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
                self.ast.expression_chain(span, ChainElement::from(member_expr))
            }
            Expression::CallExpression(e) => {
                self.ast.expression_chain(span, ChainElement::CallExpression(e))
            }
            Expression::TSNonNullExpression(e) => {
                self.ast.expression_chain(span, ChainElement::TSNonNullExpression(e))
            }
            expr => expr,
        }
    }

    /// Section 13.3 Member Expression
    fn parse_member_expression_or_higher(
        &mut self,
        in_optional_chain: &mut bool,
    ) -> Expression<'a> {
        let span = self.start_span();
        let lhs = self.parse_primary_expression();
        self.parse_member_expression_rest(
            span,
            lhs,
            in_optional_chain,
            /* allow_optional_chain */ true,
        )
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

        self.ast.expression_super(span)
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

            let mut question_dot = false;
            let is_property_access = if allow_optional_chain && self.at(Kind::QuestionDot) {
                // ?.
                // Fast check to avoid checkpoint/rewind in common cases
                let next_kind = self.lexer.peek_token().kind();
                if next_kind == Kind::LBrack
                    || next_kind.is_identifier_or_keyword()
                    || next_kind.is_template_start_of_tagged_template()
                {
                    // This is likely a valid optional chain, proceed with normal parsing
                    self.bump_any(); // consume ?.
                    let kind = self.cur_kind();
                    let is_identifier_or_keyword = kind.is_identifier_or_keyword();
                    // ?.[
                    // ?.something
                    // ?.template`...`
                    *in_optional_chain = true;
                    question_dot = true;
                    is_identifier_or_keyword
                } else {
                    // This is not a valid optional chain pattern, don't consume ?.
                    // Should be a cold branch here, as most real-world optional chaining will look like
                    // `?.something` or `?.[expr]`
                    false
                }
            } else {
                self.eat(Kind::Dot)
            };

            if is_property_access {
                lhs = self.parse_static_member_expression(lhs_span, lhs, question_dot);
                continue;
            }

            if (question_dot || !self.ctx.has_decorator()) && self.at(Kind::LBrack) {
                lhs = self.parse_computed_member_expression(lhs_span, lhs, question_dot);
                continue;
            }

            if self.cur_kind().is_template_start_of_tagged_template() {
                let (expr, type_arguments) =
                    if let Expression::TSInstantiationExpression(instantiation_expr) = lhs {
                        let expr = instantiation_expr.unbox();
                        (expr.expression, Some(expr.type_arguments))
                    } else {
                        (lhs, None)
                    };
                lhs =
                    self.parse_tagged_template(lhs_span, expr, *in_optional_chain, type_arguments);
                continue;
            }

            if !question_dot && self.is_ts {
                if !self.cur_token().is_on_new_line() && self.eat(Kind::Bang) {
                    lhs = self.ast.expression_ts_non_null(self.end_span(lhs_span), lhs);
                    continue;
                }

                if matches!(self.cur_kind(), Kind::LAngle | Kind::ShiftLeft)
                    && let Some(arguments) =
                        self.try_parse(Self::parse_type_arguments_in_expression)
                {
                    lhs = self.ast.expression_ts_instantiation(
                        self.end_span(lhs_span),
                        lhs,
                        arguments,
                    );
                    continue;
                }
            }

            return lhs;
        }
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
            self.ast.member_expression_private_field_expression(
                self.end_span(lhs_span),
                lhs,
                private_ident,
                optional,
            )
        } else {
            let ident = self.parse_identifier_name();
            self.ast.member_expression_static(self.end_span(lhs_span), lhs, ident, optional)
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
        self.bump_any(); // advance `[`
        let property = self.context_add(Context::In, Self::parse_expr);
        self.expect(Kind::RBrack);
        self.ast.member_expression_computed(self.end_span(lhs_span), lhs, property, optional).into()
    }

    /// [NewExpression](https://tc39.es/ecma262/#sec-new-operator)
    fn parse_new_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        let identifier = self.parse_keyword_identifier(Kind::New);

        if self.eat(Kind::Dot) {
            return if self.at(Kind::Target) {
                let property = self.parse_keyword_identifier(Kind::Target);
                self.ast.expression_meta_property(self.end_span(span), identifier, property)
            } else {
                self.bump_any();
                self.fatal_error(diagnostics::new_target(self.end_span(span)))
            };
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

        // parse `new ident` without arguments
        let arguments = if self.eat(Kind::LParen) {
            // ArgumentList[Yield, Await] :
            //   AssignmentExpression[+In, ?Yield, ?Await]
            let (call_arguments, _) = self.context_add(Context::In, |p| {
                p.parse_delimited_list(Kind::RParen, Kind::Comma, Self::parse_call_argument)
            });
            self.expect(Kind::RParen);
            call_arguments
        } else {
            self.ast.vec()
        };

        if is_import && matches!(callee, Expression::ImportExpression(_)) {
            self.error(diagnostics::new_dynamic_import(self.end_span(rhs_span)));
        }

        let span = self.end_span(span);

        if optional {
            self.error(diagnostics::new_optional_chain(span));
        }

        self.ast.expression_new(span, callee, type_arguments, arguments)
    }

    /// Section 13.3 Call Expression
    fn parse_call_expression_rest(
        &mut self,
        lhs_span: u32,
        lhs: Expression<'a>,
        in_optional_chain: &mut bool,
    ) -> Expression<'a> {
        let mut lhs = lhs;
        while self.fatal_error.is_none() {
            lhs = self.parse_member_expression_rest(
                lhs_span,
                lhs,
                in_optional_chain,
                /* allow_optional_chain */ true,
            );
            let question_dot_span = self.at(Kind::QuestionDot).then(|| self.cur_token().span());
            let question_dot = question_dot_span.is_some();
            if question_dot {
                self.bump_any();
                *in_optional_chain = true;
            }

            let mut type_arguments = None;
            if question_dot {
                if self.is_ts
                    && let Some(args) = self.try_parse(Self::parse_type_arguments_in_expression)
                {
                    type_arguments = Some(args);
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
                let error = diagnostics::unexpected_token(span);
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
        type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    ) -> Expression<'a> {
        // ArgumentList[Yield, Await] :
        //   AssignmentExpression[+In, ?Yield, ?Await]
        self.expect(Kind::LParen);
        let (call_arguments, _) = self.context(Context::In, Context::Decorator, |p| {
            p.parse_delimited_list(Kind::RParen, Kind::Comma, Self::parse_call_argument)
        });
        self.expect(Kind::RParen);
        self.ast.expression_call(
            self.end_span(lhs_span),
            lhs,
            type_parameters,
            call_arguments,
            optional,
        )
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
            let argument = SimpleAssignmentTarget::cover(argument, self);
            return self.ast.expression_update(self.end_span(lhs_span), operator, true, argument);
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
            let lhs = SimpleAssignmentTarget::cover(lhs, self);
            return self.ast.expression_update(self.end_span(span), operator, false, lhs);
        }
        lhs
    }

    /// Section 13.5 Unary Expression
    pub(crate) fn parse_unary_expression_or_higher(&mut self, lhs_span: u32) -> Expression<'a> {
        // ++ -- prefix update expressions
        if self.is_update_expression() {
            return self.parse_update_expression(lhs_span);
        }
        self.parse_simple_unary_expression(lhs_span)
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
                self.unexpected()
            }
            Kind::Await if self.is_await_expression() => self.parse_await_expression(lhs_span),
            _ => self.parse_update_expression(lhs_span),
        }
    }

    fn parse_unary_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        let operator = map_unary_operator(self.cur_kind());
        self.bump_any();
        let has_pure_comment = self.lexer.trivia_builder.previous_token_has_pure_comment();
        let mut argument = self.parse_simple_unary_expression(self.start_span());
        if has_pure_comment {
            Self::set_pure_on_call_or_new_expr(&mut argument);
        }
        self.ast.expression_unary(self.end_span(span), operator, argument)
    }

    pub(crate) fn parse_binary_expression_or_higher(
        &mut self,
        lhs_precedence: Precedence,
    ) -> Expression<'a> {
        let lhs_span = self.start_span();

        let lhs_parenthesized = self.at(Kind::LParen);
        // [+In] PrivateIdentifier in ShiftExpression[?Yield, ?Await]
        let lhs = if self.ctx.has_in() && self.at(Kind::PrivateIdentifier) {
            let left = self.parse_private_identifier();
            self.expect(Kind::In);
            let right = self.parse_binary_expression_or_higher(Precedence::Compare);
            if let Expression::PrivateInExpression(private_in_expr) = right {
                let error = diagnostics::private_in_private(private_in_expr.span);
                return self.fatal_error(error);
            }
            self.ast.expression_private_in(self.end_span(lhs_span), left, right)
        } else {
            let has_pure_comment = self.lexer.trivia_builder.previous_token_has_pure_comment();
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
        loop {
            // re-lex for `>=` `>>` `>>>`
            // This is need for jsx `<div>=</div>` case
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
                let type_annotation = self.parse_ts_type();
                let span = self.end_span(lhs_span);
                lhs = if kind == Kind::As {
                    if !self.is_ts {
                        self.error(diagnostics::as_in_ts(span));
                    }
                    self.ast.expression_ts_as(span, lhs, type_annotation)
                } else {
                    if !self.is_ts {
                        self.error(diagnostics::satisfies_in_ts(span));
                    }
                    self.ast.expression_ts_satisfies(span, lhs, type_annotation)
                };
                continue;
            }

            self.bump_any(); // bump operator
            let rhs_parenthesized = self.at(Kind::LParen);
            let rhs = self.parse_binary_expression_or_higher(left_precedence);

            lhs = if kind.is_logical_operator() {
                let span = self.end_span(lhs_span);
                let op = map_logical_operator(kind);
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
                self.ast.expression_logical(span, lhs, op, rhs)
            } else if kind.is_binary_operator() {
                let span = self.end_span(lhs_span);
                let op = map_binary_operator(kind);
                if op == BinaryOperator::Exponential
                    && !lhs_parenthesized
                    && let Some(key) = match lhs {
                        Expression::AwaitExpression(_) => Some("await"),
                        Expression::UnaryExpression(_) => Some("unary"),
                        _ => None,
                    }
                {
                    self.error(diagnostics::unexpected_exponential(key, lhs.span()));
                }
                self.ast.expression_binary(span, lhs, op, rhs)
            } else {
                break;
            };
        }

        lhs
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
        if !self.eat(Kind::Question) {
            return lhs;
        }
        let consequent = self.context_add(Context::In, |p| {
            p.parse_assignment_expression_or_higher_impl(
                /* allow_return_type_in_arrow_function */ false,
            )
        });
        self.expect(Kind::Colon);
        let alternate =
            self.parse_assignment_expression_or_higher_impl(allow_return_type_in_arrow_function);
        self.ast.expression_conditional(self.end_span(lhs_span), lhs, consequent, alternate)
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
        let has_pure_comment = self.lexer.trivia_builder.previous_token_has_pure_comment();
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

        if has_pure_comment {
            Self::set_pure_on_call_or_new_expr(&mut expr);
        }

        if has_no_side_effects_comment {
            Self::set_pure_on_function_expr(&mut expr);
        }

        expr
    }

    fn set_pure_on_call_or_new_expr(expr: &mut Expression<'a>) {
        match &mut expr.get_inner_expression_mut() {
            Expression::CallExpression(call_expr) => {
                call_expr.pure = true;
            }
            Expression::NewExpression(new_expr) => {
                new_expr.pure = true;
            }
            Expression::BinaryExpression(binary_expr) => {
                Self::set_pure_on_call_or_new_expr(&mut binary_expr.left);
            }
            Expression::LogicalExpression(logical_expr) => {
                Self::set_pure_on_call_or_new_expr(&mut logical_expr.left);
            }
            Expression::ConditionalExpression(conditional_expr) => {
                Self::set_pure_on_call_or_new_expr(&mut conditional_expr.test);
            }
            Expression::ChainExpression(chain_expr) => {
                if let ChainElement::CallExpression(call_expr) = &mut chain_expr.expression {
                    call_expr.pure = true;
                }
            }
            _ => {}
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
        let left = AssignmentTarget::cover(lhs, self);
        self.bump_any();
        let right =
            self.parse_assignment_expression_or_higher_impl(allow_return_type_in_arrow_function);
        self.ast.expression_assignment(self.end_span(span), operator, left, right)
    }

    /// Section 13.16 Sequence Expression
    fn parse_sequence_expression(
        &mut self,
        span: u32,
        first_expression: Expression<'a>,
    ) -> Expression<'a> {
        let mut expressions = self.ast.vec1(first_expression);
        while self.eat(Kind::Comma) {
            let expression = self.parse_assignment_expression_or_higher();
            expressions.push(expression);
        }
        self.ast.expression_sequence(self.end_span(span), expressions)
    }

    /// ``AwaitExpression`[Yield]` :
    ///     await `UnaryExpression`[?Yield, +Await]
    fn parse_await_expression(&mut self, lhs_span: u32) -> Expression<'a> {
        let span = self.start_span();
        if !self.ctx.has_await() {
            self.error(diagnostics::await_expression(self.cur_token().span()));
        }
        self.bump_any();
        let argument =
            self.context_add(Context::Await, |p| p.parse_simple_unary_expression(lhs_span));
        self.ast.expression_await(self.end_span(span), argument)
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

    pub(crate) fn parse_decorators(&mut self) -> Vec<'a, Decorator<'a>> {
        if self.at(Kind::At) {
            let mut decorators = self.ast.vec_with_capacity(1);
            while self.at(Kind::At) {
                decorators.push(self.parse_decorator());
            }
            decorators
        } else {
            self.ast.vec()
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
        self.ast.decorator(self.end_span(span), expr)
    }

    fn is_update_expression(&self) -> bool {
        match self.cur_kind() {
            kind if kind.is_unary_operator() => false,
            Kind::Await => false,
            Kind::LAngle => {
                if !self.source_type.is_jsx() {
                    return false;
                }
                true
            }
            _ => true,
        }
    }

    fn is_await_expression(&mut self) -> bool {
        if self.at(Kind::Await) {
            if self.ctx.has_await() {
                return true;
            }
            return self.lookahead(|p| {
                Self::next_token_is_identifier_or_keyword_or_literal_on_same_line(p, true)
            });
        }
        false
    }

    fn is_yield_expression(&mut self) -> bool {
        if self.at(Kind::Yield) {
            if self.ctx.has_yield() {
                return true;
            }
            return self.lookahead(|p| {
                Self::next_token_is_identifier_or_keyword_or_literal_on_same_line(p, false)
            });
        }
        false
    }

    fn next_token_is_identifier_or_keyword_or_literal_on_same_line(
        &mut self,
        is_await: bool,
    ) -> bool {
        self.bump_any();

        // NOTE: This check implementation is based on TypeScript parser.
        // But TS does not have this exception.
        // This is needed to pass parser_babel test to parse:
        // `for (await of [])` with `sourceType: script`
        if !self.is_ts && is_await && self.at(Kind::Of) {
            return false;
        }

        let token = self.cur_token();
        let kind = token.kind();
        !token.is_on_new_line() && kind.is_after_await_or_yield()
    }
}
