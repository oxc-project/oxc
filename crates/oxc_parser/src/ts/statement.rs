use oxc_allocator::Box;
use oxc_ast::ast::*;
use oxc_diagnostics::Result;
use oxc_span::Span;

use super::{
    list::{TSEnumMemberList, TSInterfaceOrObjectBodyList},
    types::ModifierFlags,
};
use crate::{
    js::{
        declaration::{VariableDeclarationContext, VariableDeclarationParent},
        function::FunctionKind,
    },
    lexer::Kind,
    list::{NormalList, SeparatedList},
    ParserImpl, StatementContext,
};

impl<'a> ParserImpl<'a> {
    /** ------------------- Enum ------------------ */

    pub(crate) fn is_at_enum_declaration(&mut self) -> bool {
        self.at(Kind::Enum) || (self.at(Kind::Const) && self.peek_at(Kind::Enum))
    }

    /// `https://www.typescriptlang.org/docs/handbook/enums.html`
    pub(crate) fn parse_ts_enum_declaration(
        &mut self,
        span: Span,
        modifiers: Modifiers<'a>,
    ) -> Result<Declaration<'a>> {
        self.bump_any(); // bump `enum`

        let id = self.parse_binding_identifier()?;
        let members = TSEnumMemberList::parse(self)?.members;
        let span = self.end_span(span);
        Ok(self.ast.ts_enum_declaration(span, id, members, modifiers))
    }

    pub(crate) fn parse_ts_enum_member(&mut self) -> Result<TSEnumMember<'a>> {
        let span = self.start_span();
        let id = self.parse_ts_enum_member_name()?;

        let initializer =
            if self.eat(Kind::Eq) { Some(self.parse_assignment_expression_base()?) } else { None };

        Ok(TSEnumMember { span: self.end_span(span), id, initializer })
    }

    fn parse_ts_enum_member_name(&mut self) -> Result<TSEnumMemberName<'a>> {
        match self.cur_kind() {
            Kind::LBrack => {
                Ok(TSEnumMemberName::ComputedPropertyName(self.parse_computed_property_name()?))
            }
            Kind::Str => Ok(TSEnumMemberName::StringLiteral(self.parse_literal_string()?)),
            kind if kind.is_number() => {
                Ok(TSEnumMemberName::NumericLiteral(self.parse_literal_number()?))
            }
            _ => Ok(TSEnumMemberName::Identifier(self.parse_identifier_name()?)),
        }
    }

    /** ------------------- Annotation ----------------- */

    pub(crate) fn parse_ts_type_annotation(
        &mut self,
    ) -> Result<Option<Box<'a, TSTypeAnnotation<'a>>>> {
        if self.at(Kind::Colon) {
            let span = self.start_span();
            self.bump_any(); // bump ':'
            let type_annotation = self.parse_ts_type()?;
            Ok(Some(self.ast.ts_type_annotation(self.end_span(span), type_annotation)))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn parse_ts_variable_annotation(
        &mut self,
    ) -> Result<(Option<Box<'a, TSTypeAnnotation<'a>>>, bool)> {
        if !self.at(Kind::Bang) {
            return Ok((self.parse_ts_type_annotation()?, false));
        }

        if self.cur_token().is_on_new_line {
            return Ok((None, false));
        }

        let span = self.start_span();
        self.bump(Kind::Bang);

        if self.eat(Kind::Colon) {
            let type_annotation = self.parse_ts_type()?;
            Ok((Some(self.ast.ts_type_annotation(self.end_span(span), type_annotation)), true))
        } else {
            Err(self.unexpected())
        }
    }

    pub(crate) fn parse_ts_type_alias_declaration(
        &mut self,
        span: Span,
        modifiers: Modifiers<'a>,
    ) -> Result<Declaration<'a>> {
        self.expect(Kind::Type)?;

        let id = self.parse_binding_identifier()?;
        let params = self.parse_ts_type_parameters()?;
        self.expect(Kind::Eq)?;

        let annotation = self.parse_ts_type()?;

        self.asi()?;
        let span = self.end_span(span);
        Ok(self.ast.ts_type_alias_declaration(span, id, annotation, params, modifiers))
    }

    /** ---------------------  Interface  ------------------------ */

    pub(crate) fn parse_ts_interface_declaration(
        &mut self,
        span: Span,
        modifiers: Modifiers<'a>,
    ) -> Result<Declaration<'a>> {
        self.expect(Kind::Interface)?; // bump interface
        let id = self.parse_binding_identifier()?;
        let type_parameters = self.parse_ts_type_parameters()?;
        let (extends, _) = self.parse_heritage_clause()?;
        let body = self.parse_ts_interface_body()?;
        let extends = extends.map(|e| self.ast.ts_interface_heritages(e));
        Ok(self.ast.ts_interface_declaration(
            self.end_span(span),
            id,
            body,
            type_parameters,
            extends,
            modifiers,
        ))
    }

    fn parse_ts_interface_body(&mut self) -> Result<Box<'a, TSInterfaceBody<'a>>> {
        let span = self.start_span();
        let mut body_list = TSInterfaceOrObjectBodyList::new(self);
        body_list.parse(self)?;
        Ok(self.ast.ts_interface_body(self.end_span(span), body_list.body))
    }

    pub(crate) fn is_at_interface_declaration(&mut self) -> bool {
        if !self.at(Kind::Interface) || self.peek_token().is_on_new_line {
            false
        } else {
            self.peek_token().kind.is_binding_identifier() || self.peek_at(Kind::LCurly)
        }
    }

    pub(crate) fn parse_ts_type_signature(&mut self) -> Result<TSSignature<'a>> {
        if self.is_at_ts_index_signature_member() {
            return self.parse_ts_index_signature_member();
        }

        match self.cur_kind() {
            Kind::LParen | Kind::LAngle => self.parse_ts_call_signature_member(),
            Kind::New if self.peek_at(Kind::LParen) || self.peek_at(Kind::LAngle) => {
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

    /// Must be at `[ident:` or `<modifiers> [ident:`
    pub(crate) fn is_at_ts_index_signature_member(&mut self) -> bool {
        let mut offset = 0;
        while self.is_nth_at_modifier(offset, false) {
            offset += 1;
        }

        if !self.nth_at(offset, Kind::LBrack) {
            return false;
        }

        if !self.nth_kind(offset + 1).is_identifier() {
            return false;
        }

        self.nth_at(offset + 2, Kind::Colon)
    }

    pub(crate) fn is_nth_at_modifier(&mut self, n: u8, is_constructor_parameter: bool) -> bool {
        let nth = self.nth(n);
        if !(matches!(
            nth.kind,
            Kind::Public
                | Kind::Protected
                | Kind::Private
                | Kind::Static
                | Kind::Abstract
                | Kind::Readonly
                | Kind::Declare
                | Kind::Override
        )) {
            return false;
        }

        let next = self.nth(n + 1);

        if next.is_on_new_line {
            false
        } else {
            let followed_by_any_member =
                matches!(next.kind, Kind::PrivateIdentifier | Kind::LBrack)
                    || next.kind.is_literal_property_name();
            let followed_by_class_member = !is_constructor_parameter && next.kind == Kind::Star;
            // allow `...` for error recovery
            let followed_by_parameter = is_constructor_parameter
                && matches!(next.kind, Kind::LCurly | Kind::LBrack | Kind::Dot3);

            followed_by_any_member || followed_by_class_member || followed_by_parameter
        }
    }

    /** ----------------------- Namespace & Module ----------------------- */

    fn parse_ts_module_block(&mut self) -> Result<Box<'a, TSModuleBlock<'a>>> {
        let span = self.start_span();

        let mut statements = self.ast.new_vec();

        if self.at(Kind::LCurly) {
            self.expect(Kind::LCurly)?;

            while !self.eat(Kind::RCurly) && !self.at(Kind::Eof) {
                let stmt = self.parse_ts_module_item()?;
                statements.push(stmt);
            }
        }

        Ok(self.ast.ts_module_block(self.end_span(span), statements))
    }

    fn parse_ts_module_item(&mut self) -> Result<Statement<'a>> {
        self.parse_statement_list_item(StatementContext::StatementList)
    }

    pub(crate) fn parse_ts_namespace_or_module_declaration_body(
        &mut self,
        span: Span,
        kind: TSModuleDeclarationKind,
        modifiers: Modifiers<'a>,
    ) -> Result<Box<'a, TSModuleDeclaration<'a>>> {
        let id = match self.cur_kind() {
            Kind::Str => self.parse_literal_string().map(TSModuleDeclarationName::StringLiteral),
            _ => self.parse_identifier_name().map(TSModuleDeclarationName::Identifier),
        }?;

        let body = if self.eat(Kind::Dot) {
            let span = self.start_span();
            let decl =
                self.parse_ts_namespace_or_module_declaration_body(span, kind, Modifiers::empty())?;
            TSModuleDeclarationBody::TSModuleDeclaration(decl)
        } else {
            let block = self.parse_ts_module_block()?;
            self.asi()?;
            TSModuleDeclarationBody::TSModuleBlock(block)
        };

        Ok(self.ast.ts_module_declaration(self.end_span(span), id, body, kind, modifiers))
    }

    /** ----------------------- declare --------------------- */

    pub(crate) fn parse_ts_declaration_statement(
        &mut self,
        start_span: Span,
    ) -> Result<Statement<'a>> {
        let reserved_ctx = self.ctx;
        let (flags, modifiers) = self.eat_modifiers_before_declaration();
        self.ctx = self.ctx.union_ambient_if(flags.declare()).and_await(flags.r#async());
        let result = self.parse_declaration(start_span, modifiers);
        self.ctx = reserved_ctx;
        result.map(Statement::Declaration)
    }

    pub(crate) fn parse_declaration(
        &mut self,
        start_span: Span,
        modifiers: Modifiers<'a>,
    ) -> Result<Declaration<'a>> {
        match self.cur_kind() {
            Kind::Namespace => {
                let kind = TSModuleDeclarationKind::Namespace;
                let span = self.start_span();
                self.bump_any();
                self.parse_ts_namespace_or_module_declaration_body(span, kind, modifiers)
                    .map(Declaration::TSModuleDeclaration)
            }
            Kind::Module => {
                let kind = TSModuleDeclarationKind::Module;
                let span = self.start_span();
                self.bump_any();
                self.parse_ts_namespace_or_module_declaration_body(span, kind, modifiers)
                    .map(Declaration::TSModuleDeclaration)
            }
            Kind::Global => {
                // declare global { }
                let kind = TSModuleDeclarationKind::Global;
                self.parse_ts_namespace_or_module_declaration_body(start_span, kind, modifiers)
                    .map(Declaration::TSModuleDeclaration)
            }
            Kind::Type => self.parse_ts_type_alias_declaration(start_span, modifiers),
            Kind::Enum => self.parse_ts_enum_declaration(start_span, modifiers),
            Kind::Interface if self.is_at_interface_declaration() => {
                self.parse_ts_interface_declaration(start_span, modifiers)
            }
            Kind::Class => self
                .parse_class_declaration(start_span, modifiers)
                .map(Declaration::ClassDeclaration),
            Kind::Import => {
                self.bump_any();
                self.parse_ts_import_equals_declaration(start_span, true)
            }
            kind if kind.is_variable_declaration() => self
                .parse_variable_declaration(
                    start_span,
                    VariableDeclarationContext::new(VariableDeclarationParent::Clause),
                    modifiers,
                )
                .map(Declaration::VariableDeclaration),
            _ if self.at_function_with_async() => {
                let declare = modifiers.contains(ModifierKind::Declare);
                if declare {
                    self.parse_ts_declare_function(start_span, modifiers)
                        .map(Declaration::FunctionDeclaration)
                } else if self.ts_enabled() {
                    self.parse_ts_function_impl(
                        start_span,
                        FunctionKind::Declaration { single_statement: true },
                        modifiers,
                    )
                    .map(Declaration::FunctionDeclaration)
                } else {
                    self.parse_function_impl(FunctionKind::Declaration { single_statement: true })
                        .map(Declaration::FunctionDeclaration)
                }
            }
            _ => Err(self.unexpected()),
        }
    }

    pub(crate) fn parse_ts_declare_function(
        &mut self,
        start_span: Span,
        modifiers: Modifiers<'a>,
    ) -> Result<Box<'a, Function<'a>>> {
        let r#async = modifiers.contains(ModifierKind::Async);
        self.expect(Kind::Function)?;
        let func_kind = FunctionKind::TSDeclaration;
        let id = self.parse_function_id(func_kind, r#async, false);
        self.parse_function(start_span, id, r#async, false, func_kind, modifiers)
    }

    pub(crate) fn parse_ts_type_assertion(&mut self) -> Result<Expression<'a>> {
        let span = self.start_span();
        self.re_lex_ts_l_angle();
        self.expect(Kind::LAngle)?;
        let type_annotation = self.parse_ts_type()?;
        self.expect(Kind::RAngle)?;
        let lhs_span = self.start_span();
        let expression = self.parse_unary_expression_base(lhs_span)?;
        Ok(self.ast.ts_type_assertion(self.end_span(span), type_annotation, expression))
    }

    pub(crate) fn parse_ts_import_equals_declaration(
        &mut self,
        span: Span,
        is_export: bool,
    ) -> Result<Declaration<'a>> {
        let import_kind = if !self.peek_at(Kind::Eq) && self.eat(Kind::Type) {
            ImportOrExportKind::Type
        } else {
            ImportOrExportKind::Value
        };

        let id = self.parse_binding_identifier()?;

        self.expect(Kind::Eq)?;

        let reference_span = self.start_span();
        let module_reference = if self.eat(Kind::Require) {
            self.expect(Kind::LParen)?;
            let expression = self.parse_literal_string()?;
            self.expect(Kind::RParen)?;
            TSModuleReference::ExternalModuleReference(TSExternalModuleReference {
                span: self.end_span(reference_span),
                expression,
            })
        } else {
            TSModuleReference::TypeName(self.parse_ts_type_name()?)
        };

        self.asi()?;

        Ok(self.ast.ts_import_equals_declaration(
            self.end_span(span),
            id,
            module_reference,
            is_export,
            import_kind,
        ))
    }

    pub(crate) fn parse_ts_this_parameter(&mut self) -> Result<TSThisParameter<'a>> {
        let span = self.start_span();

        let this = {
            let (span, name) = self.parse_identifier_kind(Kind::This);
            IdentifierName { span, name }
        };

        let type_annotation = self.parse_ts_type_annotation()?;
        Ok(self.ast.ts_this_parameter(self.end_span(span), this, type_annotation))
    }

    pub(crate) fn eat_decorators(&mut self) -> Result<()> {
        if !self.at(Kind::At) {
            return Ok(());
        }

        let mut decorators = self.ast.new_vec();
        while self.at(Kind::At) {
            let decorator = self.parse_decorator()?;
            decorators.push(decorator);
        }

        self.state.decorators = decorators;
        Ok(())
    }

    pub(crate) fn eat_modifiers_before_declaration(&mut self) -> (ModifierFlags, Modifiers<'a>) {
        let mut flags = ModifierFlags::empty();
        let mut modifiers = self.ast.new_vec();
        while self.at_modifier() {
            let span = self.start_span();
            let modifier_flag = self.cur_kind().into();
            flags.set(modifier_flag, true);
            let kind = self.cur_kind();
            self.bump_any();
            modifiers.push(Self::modifier(kind, self.end_span(span)));
        }

        (flags, Modifiers::new(modifiers))
    }

    fn at_modifier(&mut self) -> bool {
        self.lookahead(Self::at_modifier_worker)
    }

    fn at_modifier_worker(&mut self) -> bool {
        if !self.cur_kind().is_modifier_kind() {
            return false;
        }

        match self.cur_kind() {
            Kind::Const => !self.peek_token().is_on_new_line && self.peek_kind() == Kind::Enum,
            Kind::Export => {
                self.bump_any();
                match self.cur_kind() {
                    Kind::Default => {
                        self.bump_any();
                        self.can_follow_default()
                    }
                    Kind::Type => {
                        self.bump_any();
                        self.can_follow_export()
                    }
                    _ => self.can_follow_export(),
                }
            }
            Kind::Default => {
                self.bump_any();
                self.can_follow_default()
            }
            Kind::Accessor | Kind::Static | Kind::Get | Kind::Set => {
                // These modifiers can cross line.
                self.bump_any();
                Self::can_follow_modifier(self.cur_kind())
            }
            // Rest modifiers cannot cross line
            _ => {
                self.bump_any();
                Self::can_follow_modifier(self.cur_kind()) && !self.cur_token().is_on_new_line
            }
        }
    }

    fn can_follow_default(&mut self) -> bool {
        let at_declaration =
            matches!(self.cur_kind(), Kind::Class | Kind::Function | Kind::Interface);
        let at_abstract_declaration = self.at(Kind::Abstract)
            && self.peek_at(Kind::Class)
            && !self.peek_token().is_on_new_line;
        let at_async_function = self.at(Kind::Async)
            && self.peek_at(Kind::Function)
            && !self.peek_token().is_on_new_line;
        at_declaration | at_abstract_declaration | at_async_function
    }

    fn can_follow_export(&mut self) -> bool {
        // Note that the `export` in export assignment is not a modifier
        // and are handled explicitly in the parser.
        !matches!(self.cur_kind(), Kind::Star | Kind::As | Kind::LCurly)
            && Self::can_follow_modifier(self.cur_kind())
    }

    fn can_follow_modifier(kind: Kind) -> bool {
        kind.is_literal_property_name()
            || matches!(kind, Kind::LCurly | Kind::LBrack | Kind::Star | Kind::Dot3)
    }

    fn modifier(kind: Kind, span: Span) -> Modifier {
        let modifier_kind = match kind {
            Kind::Abstract => ModifierKind::Abstract,
            Kind::Declare => ModifierKind::Declare,
            Kind::Private => ModifierKind::Private,
            Kind::Protected => ModifierKind::Protected,
            Kind::Public => ModifierKind::Public,
            Kind::Static => ModifierKind::Static,
            Kind::Readonly => ModifierKind::Readonly,
            Kind::Override => ModifierKind::Override,
            Kind::Async => ModifierKind::Async,
            Kind::Const => ModifierKind::Const,
            Kind::In => ModifierKind::In,
            Kind::Out => ModifierKind::Out,
            Kind::Export => ModifierKind::Export,
            Kind::Default => ModifierKind::Default,
            Kind::Accessor => ModifierKind::Accessor,
            _ => unreachable!(),
        };
        Modifier { span, kind: modifier_kind }
    }
}
