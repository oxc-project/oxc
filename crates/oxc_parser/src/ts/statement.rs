use oxc_allocator::Box;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ParserImpl, diagnostics,
    js::{FunctionKind, VariableDeclarationParent},
    lexer::Kind,
    modifiers::{ModifierFlags, ModifierKind, Modifiers},
};

impl<'a> ParserImpl<'a> {
    /* ------------------- Enum ------------------ */
    /// `https://www.typescriptlang.org/docs/handbook/enums.html`
    pub(crate) fn parse_ts_enum_declaration(
        &mut self,
        span: u32,
        modifiers: &Modifiers<'a>,
    ) -> Declaration<'a> {
        self.bump_any(); // bump `enum`
        let id = self.parse_binding_identifier();
        let body = self.parse_ts_enum_body();
        let span = self.end_span(span);
        self.verify_modifiers(
            modifiers,
            ModifierFlags::DECLARE | ModifierFlags::CONST,
            diagnostics::modifier_cannot_be_used_here,
        );
        self.ast.declaration_ts_enum(
            span,
            id,
            body,
            modifiers.contains_const(),
            modifiers.contains_declare(),
        )
    }

    pub(crate) fn parse_ts_enum_body(&mut self) -> TSEnumBody<'a> {
        let span = self.start_span();
        self.expect(Kind::LCurly);
        let members = self.parse_delimited_list(
            Kind::RCurly,
            Kind::Comma,
            /* trailing_separator */ true,
            Self::parse_ts_enum_member,
        );
        self.expect(Kind::RCurly);
        self.ast.ts_enum_body(self.end_span(span), members)
    }

    pub(crate) fn parse_ts_enum_member(&mut self) -> TSEnumMember<'a> {
        let span = self.start_span();
        let id = self.parse_ts_enum_member_name();
        let initializer = if self.eat(Kind::Eq) {
            Some(self.parse_assignment_expression_or_higher())
        } else {
            None
        };
        self.ast.ts_enum_member(self.end_span(span), id, initializer)
    }

    fn parse_ts_enum_member_name(&mut self) -> TSEnumMemberName<'a> {
        match self.cur_kind() {
            Kind::Str => {
                let literal = self.parse_literal_string();
                TSEnumMemberName::String(self.alloc(literal))
            }
            Kind::LBrack => match self.parse_computed_property_name() {
                Expression::StringLiteral(literal) => TSEnumMemberName::ComputedString(literal),
                Expression::TemplateLiteral(template) if template.is_no_substitution_template() => {
                    TSEnumMemberName::ComputedTemplateString(template)
                }
                Expression::NumericLiteral(literal) => {
                    let error = diagnostics::enum_member_cannot_have_numeric_name(literal.span());
                    self.fatal_error(error)
                }
                expr => {
                    let error =
                        diagnostics::computed_property_names_not_allowed_in_enums(expr.span());
                    self.fatal_error(error)
                }
            },
            Kind::NoSubstitutionTemplate | Kind::TemplateHead => {
                let error = diagnostics::computed_property_names_not_allowed_in_enums(
                    self.cur_token().span(),
                );
                self.fatal_error(error)
            }
            kind if kind.is_number() => {
                let error =
                    diagnostics::enum_member_cannot_have_numeric_name(self.cur_token().span());
                self.fatal_error(error)
            }
            _ => {
                let ident_name = self.parse_identifier_name();
                TSEnumMemberName::Identifier(self.alloc(ident_name))
            }
        }
    }

    /* ------------------- Annotation ----------------- */

    pub(crate) fn parse_ts_type_annotation(&mut self) -> Option<Box<'a, TSTypeAnnotation<'a>>> {
        if !self.is_ts {
            return None;
        }
        if !self.at(Kind::Colon) {
            return None;
        }
        let span = self.start_span();
        self.bump_any(); // bump ':'
        let type_annotation = self.parse_ts_type();
        Some(self.ast.alloc_ts_type_annotation(self.end_span(span), type_annotation))
    }

    pub(crate) fn parse_ts_type_alias_declaration(
        &mut self,
        span: u32,
        modifiers: &Modifiers<'a>,
    ) -> Declaration<'a> {
        self.expect(Kind::Type);

        let id = self.parse_binding_identifier();
        let params = self.parse_ts_type_parameters();
        self.expect(Kind::Eq);

        let ty = if self.at(Kind::Intrinsic) && !self.lookahead(Self::is_next_token_dot) {
            let span = self.start_span();
            self.bump_any();
            self.ast.ts_type_intrinsic_keyword(self.end_span(span))
        } else {
            self.parse_ts_type()
        };

        self.asi();
        let span = self.end_span(span);

        self.verify_modifiers(
            modifiers,
            ModifierFlags::DECLARE,
            diagnostics::modifier_cannot_be_used_here,
        );

        self.ast.declaration_ts_type_alias(span, id, params, ty, modifiers.contains_declare())
    }

    fn is_next_token_dot(&mut self) -> bool {
        self.bump_any();
        self.at(Kind::Dot)
    }

    /* ---------------------  Interface  ------------------------ */

    pub(crate) fn parse_ts_interface_declaration(
        &mut self,
        span: u32,
        modifiers: &Modifiers<'a>,
    ) -> Declaration<'a> {
        self.expect(Kind::Interface); // bump interface
        let id = self.parse_binding_identifier();
        let type_parameters = self.parse_ts_type_parameters();
        let (extends, _) = self.parse_heritage_clause();
        let body = self.parse_ts_interface_body();
        let extends =
            extends.map_or_else(|| self.ast.vec(), |e| self.ast.ts_interface_heritages(e));

        self.verify_modifiers(
            modifiers,
            ModifierFlags::DECLARE,
            diagnostics::modifier_cannot_be_used_here,
        );

        self.ast.declaration_ts_interface(
            self.end_span(span),
            id,
            type_parameters,
            extends,
            body,
            modifiers.contains_declare(),
        )
    }

    fn parse_ts_interface_body(&mut self) -> Box<'a, TSInterfaceBody<'a>> {
        let span = self.start_span();
        let body_list = self.parse_normal_list(Kind::LCurly, Kind::RCurly, |p| {
            Some(Self::parse_ts_type_signature(p))
        });
        self.ast.alloc_ts_interface_body(self.end_span(span), body_list)
    }

    pub(crate) fn parse_ts_type_signature(&mut self) -> TSSignature<'a> {
        if self.lookahead(Self::is_at_ts_index_signature_member) {
            let span = self.start_span();
            let modifiers = self.parse_modifiers(false, false, false);
            let sig = self.parse_index_signature_declaration(span, &modifiers);
            return TSSignature::TSIndexSignature(self.alloc(sig));
        }

        match self.cur_kind() {
            Kind::LParen | Kind::LAngle => self.parse_ts_call_signature_member(),
            Kind::New if self.lookahead(Self::is_next_token_open_paren_or_angle_bracket) => {
                self.parse_ts_constructor_signature_member()
            }
            Kind::Get if self.is_next_at_type_member_name() => {
                self.parse_ts_getter_signature_member()
            }
            Kind::Set if self.is_next_at_type_member_name() => {
                self.parse_ts_setter_signature_member()
            }
            _ => self.parse_ts_property_or_method_signature_member(),
        }
    }

    fn is_next_token_open_paren_or_angle_bracket(&mut self) -> bool {
        self.bump_any();
        matches!(self.cur_kind(), Kind::LParen | Kind::LAngle)
    }

    /// Must be at `[ident:` or `<modifiers> [ident:`
    fn is_at_ts_index_signature_member(&mut self) -> bool {
        if matches!(
            self.cur_kind(),
            Kind::Public
                | Kind::Protected
                | Kind::Private
                | Kind::Static
                | Kind::Abstract
                | Kind::Readonly
                | Kind::Declare
                | Kind::Override
                | Kind::Export
        ) {
            self.bump_any();
        }

        if !self.at(Kind::LBrack) {
            return false;
        }

        self.bump_any();

        if !self.cur_kind().is_identifier() {
            return false;
        }

        self.bump_any();
        self.at(Kind::Colon)
    }

    /* ----------------------- Namespace & Module ----------------------- */

    fn parse_ts_module_block(&mut self) -> Box<'a, TSModuleBlock<'a>> {
        let span = self.start_span();
        self.expect(Kind::LCurly);
        let (directives, statements) =
            self.parse_directives_and_statements(/* is_top_level */ false);
        self.expect(Kind::RCurly);
        self.ast.alloc_ts_module_block(self.end_span(span), directives, statements)
    }

    pub(crate) fn parse_ts_namespace_or_module_declaration_body(
        &mut self,
        span: u32,
        kind: TSModuleDeclarationKind,
        modifiers: &Modifiers<'a>,
    ) -> Box<'a, TSModuleDeclaration<'a>> {
        self.verify_modifiers(
            modifiers,
            ModifierFlags::DECLARE | ModifierFlags::EXPORT,
            diagnostics::modifier_cannot_be_used_here,
        );
        let id = match self.cur_kind() {
            Kind::Str => TSModuleDeclarationName::StringLiteral(self.parse_literal_string()),
            _ => TSModuleDeclarationName::Identifier(self.parse_binding_identifier()),
        };

        let body = if self.eat(Kind::Dot) {
            let span = self.start_span();
            let decl =
                self.parse_ts_namespace_or_module_declaration_body(span, kind, &Modifiers::empty());
            Some(TSModuleDeclarationBody::TSModuleDeclaration(decl))
        } else if self.at(Kind::LCurly) {
            let block = self.parse_ts_module_block();
            Some(TSModuleDeclarationBody::TSModuleBlock(block))
        } else {
            self.asi();
            None
        };

        self.verify_modifiers(
            modifiers,
            ModifierFlags::DECLARE,
            diagnostics::modifier_cannot_be_used_here,
        );

        self.ast.alloc_ts_module_declaration(
            self.end_span(span),
            id,
            body,
            kind,
            modifiers.contains_declare(),
        )
    }

    /* ----------------------- declare --------------------- */

    pub(crate) fn parse_ts_declaration_statement(&mut self, start_span: u32) -> Statement<'a> {
        let reserved_ctx = self.ctx;
        let modifiers = self.eat_modifiers_before_declaration();
        self.ctx = self
            .ctx
            .union_ambient_if(modifiers.contains_declare())
            .and_await(modifiers.contains_async());
        let decl = self.parse_declaration(start_span, &modifiers);
        self.ctx = reserved_ctx;
        Statement::from(decl)
    }

    pub(crate) fn parse_declaration(
        &mut self,
        start_span: u32,
        modifiers: &Modifiers<'a>,
    ) -> Declaration<'a> {
        match self.cur_kind() {
            Kind::Namespace => {
                let kind = TSModuleDeclarationKind::Namespace;
                self.bump_any();
                let decl =
                    self.parse_ts_namespace_or_module_declaration_body(start_span, kind, modifiers);
                Declaration::TSModuleDeclaration(decl)
            }
            Kind::Module => {
                let kind = TSModuleDeclarationKind::Module;
                self.bump_any();
                let decl =
                    self.parse_ts_namespace_or_module_declaration_body(start_span, kind, modifiers);
                Declaration::TSModuleDeclaration(decl)
            }
            Kind::Global => {
                // declare global { }
                let kind = TSModuleDeclarationKind::Global;
                let decl =
                    self.parse_ts_namespace_or_module_declaration_body(start_span, kind, modifiers);
                Declaration::TSModuleDeclaration(decl)
            }
            Kind::Type => self.parse_ts_type_alias_declaration(start_span, modifiers),
            Kind::Enum => self.parse_ts_enum_declaration(start_span, modifiers),
            Kind::Interface => self.parse_ts_interface_declaration(start_span, modifiers),
            Kind::Class => {
                let decl = self.parse_class_declaration(start_span, modifiers);
                Declaration::ClassDeclaration(decl)
            }
            Kind::Import => {
                self.bump_any();
                self.parse_ts_import_equals_declaration(start_span)
            }
            kind if kind.is_variable_declaration() => {
                let decl = self.parse_variable_declaration(
                    start_span,
                    VariableDeclarationParent::Statement,
                    modifiers,
                );
                Declaration::VariableDeclaration(decl)
            }
            _ if self.at_function_with_async() => {
                let declare = modifiers.contains(ModifierKind::Declare);
                if declare {
                    let decl = self.parse_ts_declare_function(start_span, modifiers);
                    Declaration::FunctionDeclaration(decl)
                } else if self.is_ts {
                    let decl = self.parse_ts_function_impl(
                        start_span,
                        FunctionKind::Declaration,
                        modifiers,
                    );
                    Declaration::FunctionDeclaration(decl)
                } else {
                    let decl = self.parse_function_impl(FunctionKind::Declaration);
                    Declaration::FunctionDeclaration(decl)
                }
            }
            _ => self.unexpected(),
        }
    }

    pub(crate) fn parse_ts_declare_function(
        &mut self,
        start_span: u32,
        modifiers: &Modifiers<'a>,
    ) -> Box<'a, Function<'a>> {
        let r#async = modifiers.contains(ModifierKind::Async);
        self.expect(Kind::Function);
        let func_kind = FunctionKind::TSDeclaration;
        let id = self.parse_function_id(func_kind, r#async, false);
        self.parse_function(
            start_span,
            id,
            r#async,
            false,
            func_kind,
            FormalParameterKind::FormalParameter,
            modifiers,
        )
    }

    pub(crate) fn parse_ts_type_assertion(&mut self) -> Expression<'a> {
        let span = self.start_span();
        self.expect(Kind::LAngle);
        let type_annotation = self.parse_ts_type();
        self.expect(Kind::RAngle);
        let lhs_span = self.start_span();
        let expression = self.parse_simple_unary_expression(lhs_span);
        self.ast.expression_ts_type_assertion(self.end_span(span), type_annotation, expression)
    }

    pub(crate) fn parse_ts_import_equals_declaration(&mut self, span: u32) -> Declaration<'a> {
        let import_kind = if !self.lookahead(Self::is_next_token_equals) && self.eat(Kind::Type) {
            ImportOrExportKind::Type
        } else {
            ImportOrExportKind::Value
        };

        let id = self.parse_binding_identifier();

        self.expect(Kind::Eq);

        let reference_span = self.start_span();
        let module_reference = if self.eat(Kind::Require) {
            self.expect(Kind::LParen);
            let expression = self.parse_literal_string();
            self.expect(Kind::RParen);
            self.ast.ts_module_reference_external_module_reference(
                self.end_span(reference_span),
                expression,
            )
        } else {
            let type_name = self.parse_ts_type_name();
            TSModuleReference::from(type_name)
        };

        self.asi();

        self.ast.declaration_ts_import_equals(
            self.end_span(span),
            id,
            module_reference,
            import_kind,
        )
    }

    pub(crate) fn is_next_token_equals(&mut self) -> bool {
        self.bump_any();
        self.at(Kind::Eq)
    }

    pub(crate) fn parse_ts_this_parameter(&mut self) -> TSThisParameter<'a> {
        let span = self.start_span();
        self.parse_class_element_modifiers(true);
        self.eat_decorators();

        let this_span = self.start_span();
        self.bump_any();
        let this = self.end_span(this_span);

        let type_annotation = self.parse_ts_type_annotation();
        self.ast.ts_this_parameter(self.end_span(span), this, type_annotation)
    }

    pub(crate) fn eat_decorators(&mut self) {
        if !self.at(Kind::At) {
            return;
        }

        let mut decorators = self.ast.vec();
        while self.at(Kind::At) {
            let decorator = self.parse_decorator();
            decorators.push(decorator);
        }

        self.state.decorators = decorators;
    }

    pub(crate) fn at_start_of_ts_declaration(&mut self) -> bool {
        self.lookahead(Self::at_start_of_ts_declaration_worker)
    }

    /// Check if the parser is at a start of a ts declaration
    fn at_start_of_ts_declaration_worker(&mut self) -> bool {
        loop {
            match self.cur_kind() {
                Kind::Var | Kind::Let | Kind::Const | Kind::Function | Kind::Class | Kind::Enum => {
                    return true;
                }
                Kind::Interface | Kind::Type => {
                    self.bump_any();
                    return self.cur_kind().is_binding_identifier()
                        && !self.cur_token().is_on_new_line();
                }
                Kind::Module | Kind::Namespace => {
                    self.bump_any();
                    return !self.cur_token().is_on_new_line()
                        && (self.cur_kind().is_binding_identifier()
                            || self.cur_kind() == Kind::Str);
                }
                Kind::Abstract
                | Kind::Accessor
                | Kind::Async
                | Kind::Declare
                | Kind::Private
                | Kind::Protected
                | Kind::Public
                | Kind::Readonly => {
                    self.bump_any();
                    if self.cur_token().is_on_new_line() {
                        return false;
                    }
                }
                Kind::Global => {
                    self.bump_any();
                    return matches!(self.cur_kind(), Kind::Ident | Kind::LCurly | Kind::Export);
                }
                Kind::Import => {
                    self.bump_any();
                    return matches!(self.cur_kind(), Kind::Str | Kind::Star | Kind::LCurly)
                        || self.cur_kind().is_identifier();
                }
                Kind::Export => {
                    self.bump_any();
                    self.bump(Kind::Type); // optional `type` after `export`
                    // This allows constructs like
                    // `export *`, `export default`, `export {}`, `export = {}` along with all
                    // export [declaration]
                    if matches!(
                        self.cur_kind(),
                        Kind::Eq | Kind::Star | Kind::Default | Kind::LCurly | Kind::At | Kind::As
                    ) {
                        return true;
                    }
                    // falls through to check next token
                }
                Kind::Static => {
                    self.bump_any();
                }
                _ => {
                    return false;
                }
            }
        }
    }
}
