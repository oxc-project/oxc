use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::{ast::*, builder::NONE};
use oxc_span::GetSpan;
use oxc_syntax::operator::UnaryOperator;

use crate::{
    Context, ParserConfig as Config, ParserImpl, diagnostics,
    lexer::Kind,
    modifiers::{ModifierKind, ModifierKinds, Modifiers},
};

use super::{super::js::FunctionKind, statement::CallOrConstructorSignature};

impl<'a, C: Config> ParserImpl<'a, C> {
    pub(crate) fn parse_ts_type(&mut self) -> TSType<'a> {
        if self.is_start_of_function_type_or_constructor_type() {
            return self.parse_function_or_constructor_type();
        }
        let span = self.start_span();
        let ty = self.parse_union_type_or_higher();
        if !self.ctx.has_disallow_conditional_types()
            && !self.cur_token().is_on_new_line()
            && self.eat(Kind::Extends)
        {
            let extends_type =
                self.context_add(Context::DisallowConditionalTypes, Self::parse_ts_type);
            let question_span = self.token.span();
            self.expect(Kind::Question);
            let true_type =
                self.context_remove(Context::DisallowConditionalTypes, Self::parse_ts_type);
            self.expect_conditional_alternative(question_span);
            let false_type =
                self.context_remove(Context::DisallowConditionalTypes, Self::parse_ts_type);
            return TSType::new_ts_conditional_type(
                self.end_span(span),
                ty,
                extends_type,
                true_type,
                false_type,
                self,
            );
        }
        ty
    }

    fn parse_function_or_constructor_type(&mut self) -> TSType<'a> {
        let span = self.start_span();
        let r#abstract = self.eat(Kind::Abstract);
        let is_constructor_type = self.eat(Kind::New);
        let type_parameters = self.parse_ts_type_parameters();
        let (this_param, params) = self.context_remove(Context::DisallowConditionalTypes, |p| {
            p.parse_formal_parameters(FunctionKind::Declaration, FormalParameterKind::Signature)
        });
        let return_type = {
            let return_type_span = self.start_span();
            let return_type = self.parse_return_type();
            TSTypeAnnotation::new(self.end_span(return_type_span), return_type, self)
        };

        let span = self.end_span(span);
        if is_constructor_type {
            if let Some(this_param) = &this_param {
                // type Foo = new (this: number) => any;
                self.error(diagnostics::ts_constructor_this_parameter(this_param.span));
            }
            TSType::new_ts_constructor_type(
                span,
                r#abstract,
                type_parameters,
                params,
                return_type,
                self,
            )
        } else {
            TSType::new_ts_function_type(
                span,
                type_parameters,
                this_param,
                params,
                return_type,
                self,
            )
        }
    }

    fn is_start_of_function_type_or_constructor_type(&mut self) -> bool {
        if self.at(Kind::LAngle) {
            return true;
        }
        let kind = self.cur_kind();
        if kind == Kind::New {
            return true;
        }
        if kind != Kind::LParen && kind != Kind::Abstract {
            return false;
        }
        let checkpoint = self.checkpoint();
        self.bump_any();

        match kind {
            Kind::Abstract => {
                // `abstract new ...`
                if self.at(Kind::New) {
                    self.rewind(checkpoint);
                    return true;
                }
            }
            Kind::LParen => {
                // `( ...`
                if matches!(self.cur_kind(), Kind::RParen | Kind::Dot3) {
                    self.rewind(checkpoint);
                    return true;
                }
                if self.skip_parameter_start() {
                    // ( xxx :
                    // ( xxx ,
                    // ( xxx ?
                    // ( xxx =
                    if matches!(
                        self.cur_kind(),
                        Kind::Colon | Kind::Comma | Kind::Question | Kind::Eq
                    ) {
                        self.rewind(checkpoint);
                        return true;
                    }
                    // ( xxx ) =>
                    if self.eat(Kind::RParen) && self.at(Kind::Arrow) {
                        self.rewind(checkpoint);
                        return true;
                    }
                }
            }
            _ => unreachable!(),
        }

        self.rewind(checkpoint);
        false
    }

    fn skip_parameter_start(&mut self) -> bool {
        // Skip modifiers
        if self.cur_kind().is_modifier_kind() {
            self.parse_modifiers(false, false);
        }
        let kind = self.cur_kind();
        if kind.is_identifier() || kind == Kind::This {
            self.bump_any();
            return true;
        }
        if matches!(kind, Kind::LBrack | Kind::LCurly) {
            let errors_count = self.errors_count();
            self.parse_binding_pattern_kind();
            if !self.has_fatal_error() && errors_count == self.errors_count() {
                return true;
            }
        }
        false
    }

    pub(crate) fn parse_ts_type_parameters(
        &mut self,
    ) -> Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>> {
        self.parse_ts_type_parameters_impl(false).0
    }

    pub(crate) fn parse_ts_type_parameters_with_variance(
        &mut self,
    ) -> Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>> {
        self.parse_ts_type_parameters_impl(true).0
    }

    /// Parse TypeScript type parameters and return whether there was a trailing comma.
    /// Used for arrow functions to check for TS7060 (JSX-like type parameters in .mts/.cts).
    pub(crate) fn parse_ts_type_parameters_with_trailing_comma(
        &mut self,
    ) -> (Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>, bool) {
        self.parse_ts_type_parameters_impl(false)
    }

    fn parse_ts_type_parameters_impl(
        &mut self,
        allow_variance: bool,
    ) -> (Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>, bool) {
        if !self.is_ts {
            return (None, false);
        }
        if !self.at(Kind::LAngle) {
            return (None, false);
        }
        let span = self.start_span();
        let opening_span = self.cur_token().span();
        self.expect(Kind::LAngle);
        let (params, trailing_comma) =
            self.parse_delimited_list(Kind::RAngle, Kind::Comma, opening_span, |p| {
                p.parse_ts_type_parameter(allow_variance)
            });
        self.expect(Kind::RAngle);
        let span = self.end_span(span);
        if params.is_empty() {
            self.error(diagnostics::ts_empty_type_parameter_list(span));
        }
        (Some(TSTypeParameterDeclaration::boxed(span, params, self)), trailing_comma.is_some())
    }

    pub(crate) fn parse_ts_implements_clause(&mut self) -> ArenaVec<'a, TSClassImplements<'a>> {
        self.expect(Kind::Implements);
        let first = self.parse_ts_implement_name();
        let mut implements = ArenaVec::from_value_in(first, self);
        while self.eat(Kind::Comma) {
            implements.push(self.parse_ts_implement_name());
        }
        implements
    }

    fn parse_ts_type_parameter(&mut self, allow_variance: bool) -> TSTypeParameter<'a> {
        let span = self.start_span();

        let modifiers = self.parse_modifiers(true, false);
        let allowed_modifiers = if allow_variance {
            ModifierKinds::new([ModifierKind::In, ModifierKind::Out, ModifierKind::Const])
        } else {
            ModifierKinds::new([ModifierKind::Const])
        };
        self.verify_modifiers(
            &modifiers,
            allowed_modifiers,
            false,
            |modifier, allowed| match modifier.kind {
                ModifierKind::In | ModifierKind::Out => {
                    diagnostics::can_only_appear_on_a_type_parameter_of_a_class_interface_or_type_alias(
                        modifier.kind,
                        modifier.span(),
                    )
                }
                _ => diagnostics::cannot_appear_on_a_type_parameter(modifier, allowed),
            },
        );

        let name = self.parse_binding_identifier();
        self.check_reserved_type_name(&name, "Type parameter");
        let constraint = self.parse_ts_type_constraint();
        let default = self.parse_ts_default_type();

        TSTypeParameter::new(
            self.end_span(span),
            name,
            constraint,
            default,
            modifiers.contains(ModifierKind::In),
            modifiers.contains(ModifierKind::Out),
            modifiers.contains(ModifierKind::Const),
            self,
        )
    }

    fn parse_intersection_type_or_higher(&mut self) -> TSType<'a> {
        self.parse_union_type_or_intersection_type(Kind::Amp, Self::parse_type_operator_or_higher)
    }

    fn parse_union_type_or_higher(&mut self) -> TSType<'a> {
        self.parse_union_type_or_intersection_type(
            Kind::Pipe,
            Self::parse_intersection_type_or_higher,
        )
    }

    fn parse_union_type_or_intersection_type<F>(
        &mut self,
        kind: Kind,
        parse_constituent_type: F,
    ) -> TSType<'a>
    where
        F: Fn(&mut Self) -> TSType<'a>,
    {
        let span = self.start_span();
        let has_leading_operator = self.eat(kind);
        /* hasLeadingOperator && parseFunctionOrConstructorTypeToError(isUnionType) ||*/
        let mut ty = parse_constituent_type(self);
        if self.at(kind) || has_leading_operator {
            let mut types = ArenaVec::from_value_in(ty, self);
            while self.eat(kind) {
                types.push(
                    /*parseFunctionOrConstructorTypeToError(isUnionType) || */
                    parse_constituent_type(self),
                );
            }
            let span = self.end_span(span);
            ty = match kind {
                Kind::Pipe => TSType::new_ts_union_type(span, types, self),
                Kind::Amp => TSType::new_ts_intersection_type(span, types, self),
                _ => unreachable!(),
            };
        }
        ty
    }

    fn parse_type_operator_or_higher(&mut self) -> TSType<'a> {
        match self.cur_kind() {
            Kind::KeyOf => self.parse_type_operator(TSTypeOperatorOperator::Keyof),
            Kind::Unique => self.parse_type_operator(TSTypeOperatorOperator::Unique),
            Kind::Readonly => self.parse_type_operator(TSTypeOperatorOperator::Readonly),
            Kind::Infer => self.parse_infer_type(),
            _ => self.context_remove(
                Context::DisallowConditionalTypes,
                Self::parse_postfix_type_or_higher,
            ),
        }
    }

    fn parse_type_operator(&mut self, operator: TSTypeOperatorOperator) -> TSType<'a> {
        let span = self.start_span();
        self.bump_any(); // bump operator
        let operator_span = self.end_span(span);
        let ty = self.parse_type_operator_or_higher();
        if operator == TSTypeOperatorOperator::Readonly
            && !matches!(ty, TSType::TSArrayType(_))
            && !matches!(ty, TSType::TSTupleType(_))
        {
            self.error(diagnostics::readonly_in_array_or_tuple_type(operator_span));
        }
        TSType::new_ts_type_operator_type(self.end_span(span), operator, ty, self)
    }

    fn parse_infer_type(&mut self) -> TSType<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `infer`
        let type_parameter = self.parse_type_parameter_of_infer_type();
        TSType::new_ts_infer_type(self.end_span(span), type_parameter, self)
    }

    fn parse_type_parameter_of_infer_type(&mut self) -> ArenaBox<'a, TSTypeParameter<'a>> {
        let span = self.start_span();
        let name = self.parse_binding_identifier();
        self.check_reserved_type_name(&name, "Type parameter");
        let constraint = self.parse_constraint_of_infer_type();
        let span = self.end_span(span);

        TSTypeParameter::boxed(span, name, constraint, None, false, false, false, self)
    }

    /// Parse the `extends U` constraint of an `infer T extends U` type.
    ///
    /// Returns `None` when:
    ///
    ///   * the current token is not `extends`, or
    ///   * we're in a conditional-type-allowed context and the constraint
    ///     we'd have parsed is followed by `?`, meaning `extends` actually
    ///     belongs to an enclosing conditional (`infer T extends U ? A : B`).
    ///     In this case the parsed constraint is rewound.
    fn parse_constraint_of_infer_type(&mut self) -> Option<TSType<'a>> {
        if !self.at(Kind::Extends) {
            return None;
        }
        // When conditional types are already disallowed by the enclosing context — the normal case,
        // since `infer` lives in a conditional's `extends` clause which is parsed with
        // `DisallowConditionalTypes` — a trailing `?` cannot reinterpret `extends` as a conditional.
        // The constraint is then unambiguous, so parse it without a checkpoint/rewind.
        if self.ctx.has_disallow_conditional_types() {
            self.bump_any();
            return Some(self.context_add(Context::DisallowConditionalTypes, Self::parse_ts_type));
        }
        let checkpoint = self.checkpoint();
        self.bump_any();
        let constraint = self.context_add(Context::DisallowConditionalTypes, Self::parse_ts_type);
        if self.at(Kind::Question) {
            self.rewind(checkpoint);
            None
        } else {
            Some(constraint)
        }
    }

    fn parse_postfix_type_or_higher(&mut self) -> TSType<'a> {
        let span = self.start_span();
        let mut ty = self.parse_non_array_type();

        while !self.cur_token().is_on_new_line() {
            match self.cur_kind() {
                Kind::Bang => {
                    self.bump_any();
                    ty = TSType::new_js_doc_non_nullable_type(
                        self.end_span(span),
                        ty,
                        /* postfix */ true,
                        self,
                    );
                }
                Kind::Question => {
                    // If next token is start of a type we have a conditional type
                    if self.lookahead(|p| {
                        p.bump_any();
                        p.is_start_of_type(false)
                    }) {
                        return ty;
                    }
                    self.bump_any();
                    ty = TSType::new_js_doc_nullable_type(
                        self.end_span(span),
                        ty,
                        /* postfix */ true,
                        self,
                    );
                }
                Kind::LBrack => {
                    self.bump_any();
                    if self.is_start_of_type(/* in_start_of_parameter */ false) {
                        let index_type = self.parse_ts_type();
                        self.expect(Kind::RBrack);
                        ty = TSType::new_ts_indexed_access_type(
                            self.end_span(span),
                            ty,
                            index_type,
                            self,
                        );
                    } else {
                        self.expect(Kind::RBrack);
                        ty = TSType::new_ts_array_type(self.end_span(span), ty, self);
                    }
                }
                _ => return ty,
            }
        }
        ty
    }

    fn parse_non_array_type(&mut self) -> TSType<'a> {
        match self.cur_kind() {
            Kind::Any
            | Kind::Unknown
            | Kind::String
            | Kind::Number
            | Kind::BigInt
            | Kind::Symbol
            | Kind::Boolean
            | Kind::Undefined
            | Kind::Never
            | Kind::Object
            // Parse `null` as `TSNullKeyword` instead of null literal to align with typescript eslint.
            | Kind::Null => {
                if self.lexer.peek_token().kind() == Kind::Dot {
                    self.parse_type_reference()
                } else {
                    self.parse_keyword_type()
                }
            }
            // TODO: js doc types: `JSDocAllType`, `JSDocFunctionType`
            // Kind::StarEq => {
            // scanner.reScanAsteriskEqualsToken();
            // falls through
            // }
            // Kind::Star => {
            // return parseJSDocAllType();
            // }
            // case SyntaxKind.QuestionQuestionToken:
            // // If there is '??', treat it as prefix-'?' in JSDoc type.
            // scanner.reScanQuestionToken();
            // // falls through
            // case SyntaxKind.FunctionKeyword:
            // return parseJSDocFunctionType();
            Kind::Question => self.parse_js_doc_unknown_or_nullable_type(),
            Kind::Bang => self.parse_js_doc_non_nullable_type(),
            Kind::Str | Kind::True | Kind::False => self.parse_literal_type(),
            kind if kind.is_number() => self.parse_literal_type(),
            Kind::NoSubstitutionTemplate => {
                let span = self.start_span();
                let literal = self.parse_template_literal(false);
                let span = self.end_span(span);
                TSType::new_ts_literal_type(span, TSLiteral::TemplateLiteral(self.alloc(literal)), self)
            }
            Kind::Minus => {
                if self.lexer.peek_token().kind().is_number() {
                    let minus_start_span = self.start_span();
                    self.bump_any(); // bump `-`
                    self.parse_literal_type_negative(minus_start_span)
                } else {
                    self.parse_type_reference()
                }
            }
            Kind::Void => {
                let span = self.start_span();
                self.bump_any();
                TSType::new_ts_void_keyword(self.end_span(span), self)
            }
            Kind::This => {
                let span = self.start_span();
                self.bump_any(); // bump `this`
                let this_type = TSThisType::boxed(self.end_span(span), self);
                if self.at(Kind::Is) && !self.cur_token().is_on_new_line() {
                    self.parse_this_type_predicate(span, this_type)
                } else {
                    TSType::TSThisType(this_type)
                }
            }
            Kind::Typeof => {
                self.parse_type_query()
            }
            Kind::LCurly => {
                if self.lookahead(Self::is_start_of_mapped_type) {
                    self.parse_mapped_type()
                } else {
                    self.parse_type_literal()
                }
            }
            Kind::LBrack => self.parse_tuple_type(),
            Kind::LParen => self.parse_parenthesized_type(),
            Kind::Import => TSType::TSImportType(self.parse_ts_import_type()),
            Kind::Asserts => {
                // Peek the token after `asserts` to check if this is an asserts type predicate.
                let next = self.lexer.peek_token();
                if next.kind().is_identifier_name() && !next.is_on_new_line() {
                    let asserts_start_span = self.start_span();
                    self.bump_any(); // bump `asserts`
                    self.parse_asserts_type_predicate(asserts_start_span)
                } else {
                    self.parse_type_reference()
                }
            }
            Kind::TemplateHead => self.parse_template_type(false),
            _ => self.parse_type_reference(),
        }
    }

    fn parse_keyword_type(&mut self) -> TSType<'a> {
        let span = self.start_span();
        match self.cur_kind() {
            Kind::Any => {
                self.bump_any();
                TSType::new_ts_any_keyword(self.end_span(span), self)
            }
            Kind::BigInt => {
                self.bump_any();
                TSType::new_ts_big_int_keyword(self.end_span(span), self)
            }
            Kind::Boolean => {
                self.bump_any();
                TSType::new_ts_boolean_keyword(self.end_span(span), self)
            }
            Kind::Never => {
                self.bump_any();
                TSType::new_ts_never_keyword(self.end_span(span), self)
            }
            Kind::Number => {
                self.bump_any();
                TSType::new_ts_number_keyword(self.end_span(span), self)
            }
            Kind::Object => {
                self.bump_any();
                TSType::new_ts_object_keyword(self.end_span(span), self)
            }
            Kind::String => {
                self.bump_any();
                TSType::new_ts_string_keyword(self.end_span(span), self)
            }
            Kind::Symbol => {
                self.bump_any();
                TSType::new_ts_symbol_keyword(self.end_span(span), self)
            }
            Kind::Undefined => {
                self.bump_any();
                TSType::new_ts_undefined_keyword(self.end_span(span), self)
            }
            Kind::Unknown => {
                self.bump_any();
                TSType::new_ts_unknown_keyword(self.end_span(span), self)
            }
            Kind::Null => {
                self.bump_any();
                TSType::new_ts_null_keyword(self.end_span(span), self)
            }
            _ => self.unexpected(),
        }
    }

    fn is_start_of_type(&mut self, in_start_of_parameter: bool) -> bool {
        match self.cur_kind() {
            kind if kind.is_number() => true,
            Kind::Any
            | Kind::Unknown
            | Kind::String
            | Kind::Number
            | Kind::BigInt
            | Kind::Boolean
            | Kind::Readonly
            | Kind::Symbol
            | Kind::Unique
            | Kind::Void
            | Kind::Undefined
            | Kind::Null
            | Kind::This
            | Kind::Typeof
            | Kind::Never
            | Kind::LCurly
            | Kind::LBrack
            | Kind::LAngle
            | Kind::Pipe
            | Kind::Amp
            | Kind::New
            | Kind::Str
            | Kind::True
            | Kind::False
            | Kind::Object
            | Kind::Star
            | Kind::Question
            | Kind::Break
            | Kind::Dot3
            | Kind::Infer
            | Kind::Import
            | Kind::Asserts
            | Kind::NoSubstitutionTemplate
            | Kind::TemplateHead => true,
            Kind::Function => !in_start_of_parameter,
            Kind::Minus => !in_start_of_parameter && self.lexer.peek_token().kind().is_number(),
            Kind::LParen => {
                !in_start_of_parameter
                    && self.lookahead(Self::is_start_of_parenthesized_or_function_type)
            }
            kind => kind.is_identifier(),
        }
    }

    fn is_start_of_mapped_type(&mut self) -> bool {
        self.bump_any();
        let kind = self.cur_kind();
        if kind == Kind::Plus || kind == Kind::Minus {
            self.bump_any();
            return self.at(Kind::Readonly);
        }

        self.bump(Kind::Readonly);

        if !self.eat(Kind::LBrack) && self.cur_kind().is_identifier_name() {
            return false;
        }

        self.bump_any();
        self.at(Kind::In)
    }

    fn is_start_of_parenthesized_or_function_type(&mut self) -> bool {
        self.bump_any();
        self.at(Kind::RParen)
            || self.is_start_of_parameter(/* is_js_doc_parameter */ false)
            || self.is_start_of_type(false)
    }

    fn is_start_of_parameter(&mut self, is_js_doc_parameter: bool) -> bool {
        let kind = self.cur_kind();
        kind == Kind::Dot3
            || kind.is_binding_identifier_or_private_identifier_or_pattern()
            || kind.is_modifier_kind()
            || kind == Kind::At
            || self.is_start_of_type(!is_js_doc_parameter)
    }

    fn parse_mapped_type(&mut self) -> TSType<'a> {
        let span = self.start_span();
        self.expect(Kind::LCurly);
        let mut readonly = None;
        if self.eat(Kind::Readonly) {
            readonly = Some(TSMappedTypeModifierOperator::True);
        } else if self.eat(Kind::Plus) && self.eat(Kind::Readonly) {
            readonly = Some(TSMappedTypeModifierOperator::Plus);
        } else if self.eat(Kind::Minus) && self.eat(Kind::Readonly) {
            readonly = Some(TSMappedTypeModifierOperator::Minus);
        }

        self.expect(Kind::LBrack);
        if !self.cur_kind().is_identifier_name() {
            return self.unexpected();
        }
        let key = self.parse_binding_identifier();
        self.expect(Kind::In);
        let constraint = self.parse_ts_type();

        let name_type = if self.eat(Kind::As) { Some(self.parse_ts_type()) } else { None };
        self.expect(Kind::RBrack);

        let optional = match self.cur_kind() {
            Kind::Question => {
                self.bump_any();
                Some(TSMappedTypeModifierOperator::True)
            }
            Kind::Minus => {
                self.bump_any();
                self.expect(Kind::Question);
                Some(TSMappedTypeModifierOperator::Minus)
            }
            Kind::Plus => {
                self.bump_any();
                self.expect(Kind::Question);
                Some(TSMappedTypeModifierOperator::Plus)
            }
            _ => None,
        };

        let type_annotation = self.eat(Kind::Colon).then(|| self.parse_ts_type());
        self.bump(Kind::Semicolon);
        self.expect(Kind::RCurly);

        TSType::new_ts_mapped_type(
            self.end_span(span),
            key,
            constraint,
            name_type,
            type_annotation,
            optional,
            readonly,
            self,
        )
    }

    fn parse_type_literal(&mut self) -> TSType<'a> {
        let span = self.start_span();
        let member_list =
            self.parse_normal_list(Kind::LCurly, Kind::RCurly, Self::parse_ts_type_signature);
        TSType::new_ts_type_literal(self.end_span(span), member_list, self)
    }

    fn parse_type_query(&mut self) -> TSType<'a> {
        let span = self.start_span();
        self.bump_any(); // `bump `typeof`
        let (entity_name, type_arguments) = if self.at(Kind::Import) {
            let entity_name = TSTypeQueryExprName::TSImportType(self.parse_ts_import_type());
            (entity_name, None)
        } else {
            let entity_name = self.parse_ts_type_name(); // TODO: parseEntityName
            let entity_name = TSTypeQueryExprName::from(entity_name);
            let type_arguments = if self.cur_token().is_on_new_line() {
                None
            } else {
                self.try_parse_type_arguments()
            };
            (entity_name, type_arguments)
        };
        TSType::new_ts_type_query(self.end_span(span), entity_name, type_arguments, self)
    }

    fn parse_this_type_predicate(
        &mut self,
        span: u32,
        this_ty: ArenaBox<'a, TSThisType>,
    ) -> TSType<'a> {
        self.bump_any(); // bump `is`
        let ty = self.parse_ts_type();
        let type_annotation = Some(TSTypeAnnotation::new(ty.span(), ty, self));
        TSType::new_ts_type_predicate(
            self.end_span(span),
            TSTypePredicateName::This(this_ty),
            false,
            type_annotation,
            self,
        )
    }

    fn parse_this_type_node(&mut self) -> ArenaBox<'a, TSThisType> {
        let span = self.start_span();
        self.bump_any(); // bump `this`
        TSThisType::boxed(self.end_span(span), self)
    }

    fn parse_ts_type_constraint(&mut self) -> Option<TSType<'a>> {
        if !self.at(Kind::Extends) {
            return None;
        }
        self.bump_any();
        Some(self.parse_ts_type())
    }

    fn parse_ts_default_type(&mut self) -> Option<TSType<'a>> {
        if !self.at(Kind::Eq) {
            return None;
        }
        self.bump_any();
        Some(self.parse_ts_type())
    }

    fn parse_template_type(&mut self, tagged: bool) -> TSType<'a> {
        let span = self.start_span();
        let mut types = ArenaVec::new_in(self);
        let mut quasis = ArenaVec::new_in(self);
        match self.cur_kind() {
            Kind::NoSubstitutionTemplate => {
                quasis.push(self.parse_template_element(tagged));
            }
            Kind::TemplateHead => {
                quasis.push(self.parse_template_element(tagged));
                types.push(self.parse_ts_type());
                self.re_lex_template_substitution_tail();
                while self.fatal_error.is_none() {
                    match self.cur_kind() {
                        Kind::TemplateTail => {
                            quasis.push(self.parse_template_element(tagged));
                            break;
                        }
                        Kind::TemplateMiddle => {
                            quasis.push(self.parse_template_element(tagged));
                        }
                        Kind::Eof => {
                            self.expect(Kind::TemplateTail);
                            break;
                        }
                        _ => {
                            types.push(self.parse_ts_type());
                            self.re_lex_template_substitution_tail();
                        }
                    }
                }
            }
            _ => unreachable!("parse_template_literal"),
        }

        TSType::new_ts_template_literal_type(self.end_span(span), quasis, types, self)
    }

    fn parse_asserts_type_predicate(&mut self, asserts_start_span: u32) -> TSType<'a> {
        let parameter_name = if self.at(Kind::This) {
            TSTypePredicateName::This(self.parse_this_type_node())
        } else {
            let ident_name = self.parse_identifier_name();
            TSTypePredicateName::Identifier(self.alloc(ident_name))
        };
        let mut type_annotation = None;
        if self.eat(Kind::Is) {
            let type_span = self.start_span();
            let ty = self.parse_ts_type();
            type_annotation = Some(TSTypeAnnotation::new(self.end_span(type_span), ty, self));
        }
        TSType::new_ts_type_predicate(
            self.end_span(asserts_start_span),
            parameter_name,
            /* asserts */ true,
            type_annotation,
            self,
        )
    }

    pub(crate) fn parse_type_reference(&mut self) -> TSType<'a> {
        let span = self.start_span();
        let type_name = self.parse_ts_type_name();
        let type_parameters = self.parse_type_arguments_of_type_reference();
        TSType::new_ts_type_reference(self.end_span(span), type_name, type_parameters, self)
    }

    fn parse_ts_implement_name(&mut self) -> TSClassImplements<'a> {
        let span = self.start_span();
        let type_name = self.parse_ts_type_name();
        let type_parameters = self.parse_type_arguments_of_type_reference();
        TSClassImplements::new(self.end_span(span), type_name, type_parameters, self)
    }

    pub(crate) fn parse_ts_type_name(&mut self) -> TSTypeName<'a> {
        let span = self.start_span();
        let left = if self.at(Kind::This) {
            self.bump_any();
            TSTypeName::new_this_expression(self.end_span(span), self)
        } else {
            let ident = self.parse_identifier_name();
            TSTypeName::new_identifier_reference(ident.span, ident.name, self)
        };
        if self.at(Kind::Dot) { self.parse_ts_qualified_type_name(span, left) } else { left }
    }

    pub(crate) fn parse_ts_qualified_type_name(
        &mut self,
        span: u32,
        mut left_name: TSTypeName<'a>,
    ) -> TSTypeName<'a> {
        while self.eat(Kind::Dot) {
            let right = self.parse_identifier_name();
            left_name = TSTypeName::new_qualified_name(self.end_span(span), left_name, right, self);
        }
        left_name
    }

    pub(crate) fn try_parse_type_arguments(
        &mut self,
    ) -> Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>> {
        if self.re_lex_ts_l_angle() {
            let span = self.start_span();
            let opening_span = self.cur_token().span();
            self.expect(Kind::LAngle);
            let (params, _) = self.parse_delimited_list(
                Kind::RAngle,
                Kind::Comma,
                opening_span,
                Self::parse_ts_type,
            );
            self.expect(Kind::RAngle);
            let span = self.end_span(span);
            if params.is_empty() {
                self.error(diagnostics::ts_empty_type_argument_list(span));
            }
            return Some(TSTypeParameterInstantiation::boxed(span, params, self));
        }
        None
    }

    pub(crate) fn parse_type_arguments_of_type_reference(
        &mut self,
    ) -> Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>> {
        if !self.cur_token().is_on_new_line() && self.re_lex_ts_l_angle() {
            let span = self.start_span();
            let opening_span = self.cur_token().span();
            self.expect(Kind::LAngle);
            let (params, _) = self.parse_delimited_list(
                Kind::RAngle,
                Kind::Comma,
                opening_span,
                Self::parse_ts_type,
            );
            self.expect(Kind::RAngle);
            let span = self.end_span(span);
            if params.is_empty() {
                self.error(diagnostics::ts_empty_type_argument_list(span));
            }
            return Some(TSTypeParameterInstantiation::boxed(span, params, self));
        }
        None
    }

    /// Speculatively parse a `<T, U>` type-argument list in an expression
    /// position (e.g. `foo<T>(arg)` vs `foo < T`). Returns `None` and rewinds
    /// any parser/lexer state when the upcoming tokens turn out not to be a
    /// valid type-argument list.
    pub(crate) fn parse_type_arguments_in_expression(
        &mut self,
    ) -> Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>> {
        // A type-argument list can only open with `<`, or `<<` for nested generics like
        // `f<<T>() => U>()`. This mirrors TypeScript's `reScanLessThanToken`, which re-scans only
        // `<`/`<<`. `<=`/`<<=` can never open one — splitting off the leading `<` leaves a `=`, and
        // no type starts with `=` — so although `re_lex_ts_l_angle` would accept them (it is shared
        // with type-context callers), speculating here can only fail and rewind to `None`. Bail
        // before the checkpoint for any non-`<`-opening token (the common `a?.(`, `a?.b` paths),
        // avoiding a checkpoint/rewind round-trip that returns `None` anyway.
        if !matches!(self.cur_kind(), Kind::LAngle | Kind::ShiftLeft) {
            return None;
        }
        let checkpoint = self.checkpoint();
        let span = self.start_span();
        if !self.re_lex_ts_l_angle() {
            self.rewind(checkpoint);
            return None;
        }
        let opening_span = self.cur_token().span();
        self.expect(Kind::LAngle);
        let (params, _) =
            self.parse_delimited_list(Kind::RAngle, Kind::Comma, opening_span, Self::parse_ts_type);
        // `a < b> = c` is valid but `a < b >= c` is BinaryExpression
        if matches!(self.re_lex_right_angle(), Kind::GtEq) {
            self.rewind(checkpoint);
            return None;
        }
        self.re_lex_ts_r_angle();
        self.expect(Kind::RAngle);
        if self.fatal_error.is_some() || !self.can_follow_type_arguments_in_expr() {
            self.rewind(checkpoint);
            return None;
        }
        let span = self.end_span(span);
        if params.is_empty() {
            self.error(diagnostics::ts_empty_type_argument_list(span));
        }
        Some(TSTypeParameterInstantiation::boxed(span, params, self))
    }

    fn can_follow_type_arguments_in_expr(&mut self) -> bool {
        match self.cur_kind() {
            Kind::LParen | Kind::NoSubstitutionTemplate | Kind::TemplateHead => true,
            Kind::LAngle | Kind::RAngle | Kind::Plus | Kind::Minus => false,
            _ => {
                self.cur_token().is_on_new_line()
                    || self.is_binary_operator()
                    || !self.is_start_of_expression()
            }
        }
    }

    fn parse_tuple_type(&mut self) -> TSType<'a> {
        let span = self.start_span();
        let opening_span = self.cur_token().span();
        self.expect(Kind::LBrack);

        let mut seen_rest_span: Option<Span> = None;
        let mut seen_optional_span: Option<Span> = None;
        let (elements, _) =
            self.parse_delimited_list(Kind::RBrack, Kind::Comma, opening_span, |me| {
                let tuple = me.parse_tuple_element();
                // check for array type, because unknown types can be destructed, example of valid code:
                // type C<T extends unknown[]> = [...string[], ...T];
                // example of invalid code:
                // type C<T extends unknown[]> = [...string[], ...T[]];
                if let TSTupleElement::TSRestType(rest) = &tuple
                    && match &rest.type_annotation {
                        TSType::TSArrayType(_) => true,
                        // Check for `Array<...>` type
                        TSType::TSTypeReference(ts_ref) => match &ts_ref.type_name {
                            TSTypeName::IdentifierReference(id_ref) => id_ref.name == "Array",
                            _ => false,
                        },
                        _ => false,
                    }
                {
                    if let Some(seen_span) = seen_rest_span {
                        me.error(diagnostics::rest_element_cannot_follow_another_rest_element(
                            seen_span,
                            tuple.span(),
                        ));
                    }
                    seen_rest_span = Some(tuple.span());
                }

                if !match &tuple {
                    TSTupleElement::TSOptionalType(_) | TSTupleElement::TSRestType(_) => true,
                    TSTupleElement::TSNamedTupleMember(named) => named.optional,
                    _ => false,
                } && let Some(seen_optional_span) = seen_optional_span
                {
                    me.error(diagnostics::required_element_cannot_follow_optional_element(
                        tuple.span(),
                        seen_optional_span,
                    ));
                }

                if match &tuple {
                    TSTupleElement::TSOptionalType(_) => true,
                    TSTupleElement::TSNamedTupleMember(named) => named.optional,
                    _ => false,
                } {
                    if let Some(seen_rest_span) = seen_rest_span {
                        me.error(diagnostics::optional_element_cannot_follow_rest_element(
                            tuple.span(),
                            seen_rest_span,
                        ));
                    }
                    seen_optional_span = Some(tuple.span());
                }

                tuple
            });
        self.expect(Kind::RBrack);
        TSType::new_ts_tuple_type(self.end_span(span), elements, self)
    }

    pub(super) fn parse_tuple_element(&mut self) -> TSTupleElement<'a> {
        let span_start = self.start_span();
        let is_rest_type = self.eat(Kind::Dot3);

        if self.cur_kind().is_identifier_name()
            && self.lookahead(Self::is_next_token_colon_or_question_colon)
        {
            let member_span_start = self.start_span();
            let label = self.parse_identifier_name();
            let optional = self.eat(Kind::Question);
            self.expect(Kind::Colon);
            let type_span_start = self.start_span();
            let rest_after_tuple_member_name = self.eat(Kind::Dot3);
            let ty = self.parse_ts_type();
            let optional_after_tuple_member_name = matches!(ty, TSType::JSDocNullableType(_));
            let tuple_element = self.convert_type_to_tuple_element(ty);
            let member_span = self.end_span(member_span_start);
            let named_tuple_member = TSType::new_ts_named_tuple_member(
                member_span,
                label,
                tuple_element,
                optional,
                self,
            );
            if rest_after_tuple_member_name {
                self.error(diagnostics::rest_after_tuple_member_name(
                    self.end_span(type_span_start),
                ));
            }
            if optional_after_tuple_member_name {
                self.error(diagnostics::optional_after_tuple_member_name(
                    self.end_span(type_span_start),
                ));
            }
            if is_rest_type {
                let span = self.end_span(span_start);
                if optional {
                    self.error(diagnostics::optional_and_rest_tuple_member(span));
                }
                TSTupleElement::new_ts_rest_type(span, named_tuple_member, self)
            } else {
                TSTupleElement::from(named_tuple_member)
            }
        } else {
            let ty = self.parse_ts_type();
            if is_rest_type {
                TSTupleElement::new_ts_rest_type(self.end_span(span_start), ty, self)
            } else {
                self.convert_type_to_tuple_element(ty)
            }
        }
    }

    fn is_next_token_colon_or_question_colon(&mut self) -> bool {
        self.bump_any();
        self.bump(Kind::Question);
        self.at(Kind::Colon)
    }

    fn convert_type_to_tuple_element(&self, ty: TSType<'a>) -> TSTupleElement<'a> {
        if let TSType::JSDocNullableType(ty) = ty {
            if ty.postfix {
                TSTupleElement::new_ts_optional_type(ty.span, ty.unbox().type_annotation, self)
            } else {
                TSTupleElement::JSDocNullableType(ty)
            }
        } else {
            TSTupleElement::from(ty)
        }
    }

    fn parse_parenthesized_type(&mut self) -> TSType<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `(`
        let ty = self.parse_ts_type();
        self.expect(Kind::RParen);
        if self.options.preserve_parens {
            TSType::new_ts_parenthesized_type(self.end_span(span), ty, self)
        } else {
            ty
        }
    }

    fn parse_literal_type(&mut self) -> TSType<'a> {
        let span = self.start_span();
        let expression = self.parse_literal_expression();
        let span = self.end_span(span);
        let literal = match expression {
            Expression::BooleanLiteral(literal) => TSLiteral::BooleanLiteral(literal),
            Expression::NumericLiteral(literal) => TSLiteral::NumericLiteral(literal),
            Expression::BigIntLiteral(literal) => TSLiteral::BigIntLiteral(literal),
            Expression::StringLiteral(literal) => TSLiteral::StringLiteral(literal),
            _ => return self.unexpected(),
        };
        TSType::new_ts_literal_type(span, literal, self)
    }

    fn parse_literal_type_negative(&mut self, span: u32) -> TSType<'a> {
        let literal_expr = self.parse_literal_expression();
        let span = self.end_span(span);
        let literal =
            TSLiteral::new_unary_expression(span, UnaryOperator::UnaryNegation, literal_expr, self);
        TSType::new_ts_literal_type(span, literal, self)
    }

    fn parse_ts_import_type(&mut self) -> ArenaBox<'a, TSImportType<'a>> {
        let span = self.start_span();
        self.expect(Kind::Import);
        self.expect(Kind::LParen);

        let source = if self.at(Kind::Str) {
            self.parse_literal_string()
        } else {
            // `StringLiteral` is the only valid type for `TSImportType`'s `source` field. So this is an error.
            // Fallback to parsing a `TSType`, to obtain a good span for the diagnostic.
            // It's possible for `parse_ts_type` to produce an invalid span. Fallback to the current token span if so.
            let mut span = self.parse_ts_type().span();
            if span.end <= span.start {
                span = self.cur_token().span();
            }
            self.error(diagnostics::ts_string_literal_expected(span));
            StringLiteral::new(span, "", None, self)
        };

        let options =
            if self.eat(Kind::Comma) { Some(self.parse_ts_import_type_options()) } else { None };
        self.expect(Kind::RParen);
        let qualifier =
            if self.eat(Kind::Dot) { Some(self.parse_ts_import_type_qualifier()) } else { None };
        let type_arguments = self.parse_type_arguments_of_type_reference();
        TSImportType::boxed(self.end_span(span), source, options, qualifier, type_arguments, self)
    }

    fn parse_ts_import_type_qualifier(&mut self) -> TSImportTypeQualifier<'a> {
        let span = self.start_span();
        let ident = self.parse_identifier_name();
        let mut left = TSImportTypeQualifier::new_identifier(ident.span, ident.name, self);

        while self.eat(Kind::Dot) {
            let right = self.parse_identifier_name();
            left =
                TSImportTypeQualifier::new_qualified_name(self.end_span(span), left, right, self);
        }

        left
    }

    /// Parse TypeScript import type options: `{ with: { type: "json" } }` or `{ assert: { ... } }`
    ///
    /// The options must have a property with key `with` or `assert` (as identifier, not string).
    /// If the value is an object literal, it must have only static key-value pairs
    /// (no computed keys, no spread elements).
    fn parse_ts_import_type_options(&mut self) -> ArenaBox<'a, ObjectExpression<'a>> {
        let span = self.start_span();
        self.expect(Kind::LCurly);

        // Expect `with` or `assert` as identifier (not string, not escaped)
        // TypeScript supports both: `with` is the current standard, `assert` is the older syntax
        let key_span = self.cur_token().span();
        let is_with = self.at(Kind::With);
        let is_assert = self.at(Kind::Assert);
        if (!is_with && !is_assert) || self.cur_token().escaped() {
            self.error(diagnostics::ts_import_type_options_expected_with(key_span));
        }
        // Use the actual string from the source (not a static string) to ensure it's in the arena
        let key_name = self.cur_string();
        let with_key_span = self.start_span();
        self.bump_any();
        let with_key = IdentifierName::boxed(self.end_span(with_key_span), key_name, self);

        self.expect(Kind::Colon);

        // Parse the value - if it's an object literal, validate it
        let value = if self.at(Kind::LCurly) {
            Expression::ObjectExpression(self.parse_ts_import_type_attributes())
        } else {
            // Allow any expression (e.g., super.foo)
            self.parse_assignment_expression_or_higher()
        };

        // Create the outer `with: { ... }` property
        let with_property = ObjectProperty::boxed(
            self.end_span(with_key_span),
            PropertyKind::Init,
            PropertyKey::StaticIdentifier(with_key),
            value,
            false,
            false,
            false,
            self,
        );

        let outer_properties =
            ArenaVec::from_value_in(ObjectPropertyKind::ObjectProperty(with_property), self);

        // Allow optional trailing comma: `{ with: { type: "json" }, }`
        let _ = self.eat(Kind::Comma);

        self.expect(Kind::RCurly);
        ObjectExpression::boxed(self.end_span(span), outer_properties, self)
    }

    /// Parse TypeScript import type attributes object: `{ type: "json" }`
    /// Only allows static key-value pairs (no computed keys, no spread elements).
    fn parse_ts_import_type_attributes(&mut self) -> ArenaBox<'a, ObjectExpression<'a>> {
        let span = self.start_span();
        self.expect(Kind::LCurly);

        let mut properties = ArenaVec::new_in(self);
        let mut first = true;
        while !self.at(Kind::RCurly) && !self.at(Kind::Eof) {
            if first {
                first = false;
            } else {
                self.expect(Kind::Comma);
                if self.at(Kind::RCurly) {
                    break;
                }
            }

            if properties.capacity() == 0 {
                properties.reserve(4);
            }

            // Check for spread element
            if self.at(Kind::Dot3) {
                let spread_span = self.cur_token().span();
                self.error(diagnostics::ts_import_type_options_no_spread(spread_span));
                // Skip the spread and parse the expression to recover
                self.bump_any();
                self.parse_assignment_expression_or_higher();
                continue;
            }

            let prop_span = self.start_span();

            // Check for computed property
            if self.at(Kind::LBrack) {
                let bracket_span = self.cur_token().span();
                self.error(diagnostics::ts_import_type_options_invalid_key(bracket_span));
                // Parse as computed to recover
                self.bump_any();
                self.parse_assignment_expression_or_higher();
                self.expect(Kind::RBrack);
                self.expect(Kind::Colon);
                let value = self.parse_assignment_expression_or_higher();
                let key = PropertyKey::new_string_literal(bracket_span, "", None, self);
                properties.push(ObjectPropertyKind::new_object_property(
                    self.end_span(prop_span),
                    PropertyKind::Init,
                    key,
                    value,
                    false,
                    false,
                    true, /* computed */
                    self,
                ));
                continue;
            }

            // Parse identifier or string key
            let key = if self.at(Kind::Str) {
                let string_literal = self.parse_literal_string();
                PropertyKey::StringLiteral(self.alloc(string_literal))
            } else {
                let ident = self.parse_identifier_name();
                PropertyKey::StaticIdentifier(self.alloc(ident))
            };

            self.expect(Kind::Colon);
            let value = self.parse_assignment_expression_or_higher();

            properties.push(ObjectPropertyKind::new_object_property(
                self.end_span(prop_span),
                PropertyKind::Init,
                key,
                value,
                false,
                false,
                false,
                self,
            ));
        }

        self.expect(Kind::RCurly);
        ObjectExpression::boxed(self.end_span(span), properties, self)
    }

    pub(crate) fn parse_ts_return_type_annotation(
        &mut self,
    ) -> Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        if !self.at(Kind::Colon) {
            return None;
        }
        let span = self.start_span();
        let return_type = self.parse_return_type();
        Some(TSTypeAnnotation::boxed(self.end_span(span), return_type, self))
    }

    fn parse_return_type(&mut self) -> TSType<'a> {
        self.bump_any();
        self.context_remove(Context::DisallowConditionalTypes, Self::parse_type_or_type_predicate)
    }

    fn parse_type_or_type_predicate(&mut self) -> TSType<'a> {
        let span = self.start_span();
        let type_predicate_variable = self.parse_type_predicate_prefix();

        let ty = self.parse_ts_type();
        if let Some(parameter_name) = type_predicate_variable {
            let type_annotation = Some(TSTypeAnnotation::new(ty.span(), ty, self));
            return TSType::new_ts_type_predicate(
                self.end_span(span),
                parameter_name,
                false,
                type_annotation,
                self,
            );
        }
        ty
    }

    /// Parse `<ident> is` or `this is` prefix of a type predicate.
    /// Returns `None` (without consuming anything) when the current token is
    /// not followed by `is` on the same line.
    fn parse_type_predicate_prefix(&mut self) -> Option<TSTypePredicateName<'a>> {
        if !self.cur_kind().is_identifier_name() {
            return None;
        }
        let next = self.lexer.peek_token();
        if next.kind() != Kind::Is || next.is_on_new_line() {
            return None;
        }
        let parameter_name = if self.at(Kind::This) {
            TSTypePredicateName::This(self.parse_this_type_node())
        } else {
            let ident_name = self.parse_identifier_name();
            TSTypePredicateName::Identifier(self.alloc(ident_name))
        };
        self.bump_any(); // bump `is`
        Some(parameter_name)
    }

    pub(super) fn parse_signature_member(
        &mut self,
        kind: CallOrConstructorSignature,
    ) -> TSSignature<'a> {
        let span = self.start_span();
        if kind == CallOrConstructorSignature::Constructor {
            self.expect(Kind::New);
        }
        let type_parameters = self.parse_ts_type_parameters();
        let (this_param, params) =
            self.parse_formal_parameters(FunctionKind::Declaration, FormalParameterKind::Signature);
        if kind == CallOrConstructorSignature::Constructor
            && let Some(this_param) = &this_param
        {
            // interface Foo { new(this: number): Foo }
            self.error(diagnostics::ts_constructor_this_parameter(this_param.span));
        }
        let return_type = self.parse_ts_return_type_annotation();
        self.parse_type_member_semicolon();
        match kind {
            CallOrConstructorSignature::Call => TSSignature::new_ts_call_signature_declaration(
                self.end_span(span),
                type_parameters,
                this_param,
                params,
                return_type,
                self,
            ),
            CallOrConstructorSignature::Constructor => {
                TSSignature::new_ts_construct_signature_declaration(
                    self.end_span(span),
                    type_parameters,
                    params,
                    return_type,
                    self,
                )
            }
        }
    }

    pub(crate) fn parse_getter_setter_signature_member(
        &mut self,
        span: u32,
        kind: TSMethodSignatureKind,
    ) -> TSSignature<'a> {
        let (key, computed) = self.parse_property_name();
        let (this_param, params) =
            self.parse_formal_parameters(FunctionKind::Declaration, FormalParameterKind::Signature);
        let return_type = self.parse_ts_return_type_annotation();
        self.parse_type_member_semicolon();

        if let Some(this_param) = &this_param {
            self.error(diagnostics::accessor_cannot_have_this_parameter(this_param.span));
        }

        match kind {
            TSMethodSignatureKind::Get => {
                if !params.items.is_empty() {
                    self.error(diagnostics::getter_parameters(params.span));
                }
            }
            TSMethodSignatureKind::Set => {
                if let Some(return_type) = return_type.as_ref() {
                    self.error(diagnostics::a_set_accessor_cannot_have_a_return_type_annotation(
                        return_type.span,
                    ));
                }
                if let Some(rest) = &params.rest {
                    self.error(diagnostics::setter_with_rest_parameter(rest.span));
                }
                if params.items.len() != 1 {
                    self.error(diagnostics::setter_with_parameters(
                        params.span,
                        params.items.len(),
                    ));
                } else if let Some(param) = params.items.first()
                    && param.optional
                {
                    self.error(diagnostics::setter_with_optional_parameter(param.span));
                }
            }
            TSMethodSignatureKind::Method => {}
        }

        TSSignature::new_ts_method_signature(
            self.end_span(span),
            key,
            computed,
            /* optional */ false,
            kind,
            NONE,
            this_param,
            params,
            return_type,
            self,
        )
    }

    pub(super) fn parse_property_or_method_signature(
        &mut self,
        span: u32,
        modifiers: &Modifiers,
    ) -> TSSignature<'a> {
        let (key, computed) = self.parse_property_name();
        let optional = self.eat(Kind::Question);

        let kind = self.cur_kind();
        if kind == Kind::LParen || kind == Kind::LAngle {
            self.verify_modifiers(
                modifiers,
                ModifierKinds::all_except([ModifierKind::Readonly]),
                false,
                diagnostics::modifier_only_on_property_declaration_or_index_signature,
            );

            let type_parameters = self.parse_ts_type_parameters();
            let (this_param, params) = self
                .parse_formal_parameters(FunctionKind::Declaration, FormalParameterKind::Signature);
            let return_type = self.parse_ts_return_type_annotation();
            self.parse_type_member_semicolon();
            TSSignature::new_ts_method_signature(
                self.end_span(span),
                key,
                computed,
                optional,
                TSMethodSignatureKind::Method,
                type_parameters,
                this_param,
                params,
                return_type,
                self,
            )
        } else {
            let type_annotation = self.parse_ts_type_annotation();
            self.parse_type_member_semicolon();
            TSSignature::new_ts_property_signature(
                self.end_span(span),
                computed,
                optional,
                modifiers.contains_readonly(),
                key,
                type_annotation,
                self,
            )
        }
    }

    pub(crate) fn parse_index_signature_declaration(
        &mut self,
        span: u32,
        modifiers: &Modifiers,
    ) -> ArenaBox<'a, TSIndexSignature<'a>> {
        let opening_span = self.cur_token().span();
        self.expect(Kind::LBrack);
        let (params, comma_span) = self.parse_delimited_list(
            Kind::RBrack,
            Kind::Comma,
            opening_span,
            Self::parse_ts_index_signature_name,
        );
        if let Some(comma_span) = comma_span {
            self.error(diagnostics::unexpected_trailing_comma(
                "Index signature declarations",
                self.end_span(comma_span),
            ));
        }
        self.expect(Kind::RBrack);
        match params.as_slice() {
            [param] => match &param.type_annotation.type_annotation {
                TSType::TSLiteralType(ty) => {
                    self.error(diagnostics::index_signature_parameter_literal_type(ty.span));
                }
                TSType::TSStringKeyword(_)
                | TSType::TSNumberKeyword(_)
                | TSType::TSSymbolKeyword(_)
                | TSType::TSAnyKeyword(_) => {}
                ty if ty.is_keyword() => {
                    self.error(diagnostics::index_signature_parameter_type(param.span));
                }
                _ => {}
            },
            _ => self.error(diagnostics::index_signature_one_parameter(self.end_span(span))),
        }
        let Some(type_annotation) = self.parse_ts_type_annotation() else {
            return self
                .fatal_error(diagnostics::index_signature_type_annotation(self.end_span(span)));
        };
        self.parse_type_member_semicolon();
        TSIndexSignature::boxed(
            self.end_span(span),
            params,
            type_annotation,
            modifiers.contains(ModifierKind::Readonly),
            modifiers.contains(ModifierKind::Static),
            self,
        )
    }

    fn parse_type_member_semicolon(&mut self) {
        // We allow type members to be separated by commas or (possibly ASI) semicolons.
        // First check if it was a comma.  If so, we're done with the member.
        if self.eat(Kind::Comma) {
            return;
        }
        // Didn't have a comma.  We must have a (possible ASI) semicolon.
        self.bump(Kind::Semicolon);
    }

    fn parse_ts_index_signature_name(&mut self) -> TSIndexSignatureName<'a> {
        let span = self.start_span();
        let name = self.parse_identifier_name().name;
        if self.at(Kind::Question) {
            self.error(diagnostics::index_signature_question_mark(self.cur_token().span()));
            self.bump_any();
        }
        let type_annotation = self.parse_ts_type_annotation();
        if let Some(type_annotation) = type_annotation {
            TSIndexSignatureName::new(self.end_span(span), name, type_annotation, self)
        } else {
            self.unexpected()
        }
    }

    fn parse_js_doc_unknown_or_nullable_type(&mut self) -> TSType<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `?`
        if matches!(
            self.cur_kind(),
            Kind::Comma | Kind::RCurly | Kind::RParen | Kind::RAngle | Kind::Eq | Kind::Pipe
        ) {
            return TSType::new_js_doc_unknown_type(self.end_span(span), self);
        }
        let type_annotation = self.parse_ts_type();
        TSType::new_js_doc_nullable_type(
            self.end_span(span),
            type_annotation,
            /* postfix */ false,
            self,
        )
    }

    fn parse_js_doc_non_nullable_type(&mut self) -> TSType<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `!`
        let ty = self.parse_non_array_type();
        TSType::new_js_doc_non_nullable_type(
            self.end_span(span),
            ty,
            /* postfix */ false,
            self,
        )
    }

    fn is_binary_operator(&self) -> bool {
        if self.ctx.has_in() && self.at(Kind::In) {
            return false;
        }
        self.cur_kind().is_binary_operator()
    }

    fn is_start_of_expression(&mut self) -> bool {
        if self.is_start_of_left_hand_side_expression() {
            return true;
        }
        match self.cur_kind() {
            kind if kind.is_unary_operator() => true,
            kind if kind.is_update_operator() => true,
            Kind::LAngle | Kind::Await | Kind::Yield | Kind::Private | Kind::At => true,
            kind if kind.is_binary_operator() => true,
            kind => kind.is_ts_identifier(self.ctx.has_yield(), self.ctx.has_await()),
        }
    }

    fn is_start_of_left_hand_side_expression(&mut self) -> bool {
        match self.cur_kind() {
            kind if kind.is_literal() => true,
            kind if kind.is_template_start_of_tagged_template() => true,
            Kind::This
            | Kind::Super
            | Kind::LParen
            | Kind::LBrack
            | Kind::LCurly
            | Kind::Function
            | Kind::Class
            | Kind::New
            | Kind::Slash
            | Kind::SlashEq => true,
            Kind::Import => {
                matches!(self.lexer.peek_token().kind(), Kind::LParen | Kind::LAngle | Kind::Dot)
            }
            _ => false,
        }
    }
}
