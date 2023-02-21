use oxc_allocator::Box;
use oxc_ast::{
    ast::*,
    context::{Context, StatementContext},
    Span,
};
use oxc_diagnostics::Result;

use super::list::{TSEnumMemberList, TSInterfaceOrObjectBodyList};
use crate::js::declaration::{VariableDeclarationContext, VariableDeclarationParent};
use crate::js::function::FunctionKind;
use crate::lexer::Kind;
use crate::list::{NormalList, SeparatedList};
use crate::Parser;

impl<'a> Parser<'a> {
    /** ------------------- Enum ------------------ */

    pub fn is_at_enum_declaration(&mut self) -> bool {
        self.at(Kind::Enum) || (self.at(Kind::Const) && self.peek_at(Kind::Enum))
    }

    /// `https://www.typescriptlang.org/docs/handbook/enums.html`
    pub fn parse_ts_enum_declaration(
        &mut self,
        declare: bool,
        span: Span,
    ) -> Result<Declaration<'a>> {
        let r#const = self.eat(Kind::Const);
        self.expect(Kind::Enum)?;

        let id = self.parse_binding_identifier()?;
        let members = TSEnumMemberList::parse(self)?.members;
        Ok(self.ast.ts_enum_declaration(span, id, members, declare, r#const))
    }

    pub fn parse_ts_enum_member(&mut self) -> Result<TSEnumMember<'a>> {
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
                Ok(TSEnumMemberName::NumberLiteral(self.parse_literal_number()?))
            }
            _ => Ok(TSEnumMemberName::Identifier(self.parse_identifier_name()?)),
        }
    }

    /** ------------------- Annotation ----------------- */

    pub fn parse_ts_type_annotation(&mut self) -> Result<Option<TSTypeAnnotation<'a>>> {
        if self.at(Kind::Colon) {
            let span = self.start_span();
            self.bump_any(); // bump ':'
            let type_annotation = self.parse_ts_type()?;
            Ok(Some(self.ast.ts_type_annotation(self.end_span(span), type_annotation)))
        } else {
            Ok(None)
        }
    }

    pub fn parse_ts_variable_annotation(&mut self) -> Result<(Option<TSTypeAnnotation<'a>>, bool)> {
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
            self.unexpected()
        }
    }

    pub fn parse_ts_type_alias_declaration(
        &mut self,
        declare: bool,
        span: Span,
    ) -> Result<Declaration<'a>> {
        self.expect(Kind::Type)?;

        let id = self.parse_binding_identifier()?;
        let params = self.parse_ts_type_parameters()?;
        self.expect(Kind::Eq)?;

        let annotation = self.parse_ts_type()?;

        self.asi()?;
        Ok(self.ast.ts_type_alias_declaration(span, id, annotation, params, declare))
    }

    /** ---------------------  Interface  ------------------------ */

    pub fn parse_ts_interface_declaration(
        &mut self,
        declare: bool,
        span: Span,
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
            declare,
        ))
    }

    fn parse_ts_interface_body(&mut self) -> Result<Box<'a, TSInterfaceBody<'a>>> {
        let span = self.start_span();
        let mut body_list = TSInterfaceOrObjectBodyList::new(self);
        body_list.parse(self)?;
        Ok(self.ast.ts_interface_body(self.end_span(span), body_list.body))
    }

    pub fn is_at_interface_declaration(&mut self) -> bool {
        if !self.at(Kind::Interface) || self.peek_token().is_on_new_line {
            false
        } else {
            self.peek_token().kind.is_binding_identifier() || self.peek_at(Kind::LCurly)
        }
    }

    pub fn parse_ts_type_signature(&mut self) -> Result<TSSignature<'a>> {
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
    pub fn is_at_ts_index_signature_member(&mut self) -> bool {
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

    pub fn is_nth_at_modifier(&mut self, n: u8, is_constructor_parameter: bool) -> bool {
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
        match self.cur_kind() {
            Kind::Import if !matches!(self.peek_kind(), Kind::Dot | Kind::LParen) => {
                self.parse_import_declaration()
            }
            Kind::Export => self.parse_export_declaration(),
            Kind::At => {
                self.eat_decorators()?;
                self.parse_ts_module_item()
            }
            _ => self.parse_statement_list_item(StatementContext::StatementList),
        }
    }

    pub fn parse_ts_namespace_or_module_declaration_body(
        &mut self,
        span: Span,
        declare: bool,
    ) -> Result<Box<'a, TSModuleDeclaration<'a>>> {
        let id = match self.cur_kind() {
            Kind::Str => self.parse_literal_string().map(TSModuleDeclarationName::StringLiteral),
            _ => self.parse_identifier_name().map(TSModuleDeclarationName::Identifier),
        }?;

        let body = if self.eat(Kind::Dot) {
            let span = self.start_span();
            let decl = self.parse_ts_namespace_or_module_declaration_body(span, false)?;
            TSModuleDeclarationBody::TSModuleDeclaration(decl)
        } else {
            let block = self.parse_ts_module_block()?;
            self.asi()?;
            TSModuleDeclarationBody::TSModuleBlock(block)
        };

        Ok(self.ast.ts_module_declaration(self.end_span(span), id, body, declare))
    }

    pub fn parse_ts_namespace_or_module_declaration(
        &mut self,
        declare: bool,
    ) -> Result<Box<'a, TSModuleDeclaration<'a>>> {
        let span = self.start_span();
        self.expect(Kind::Namespace).or_else(|_| self.expect(Kind::Module))?;
        self.parse_ts_namespace_or_module_declaration_body(span, declare)
    }

    pub fn parse_ts_global_declaration(&mut self) -> Result<Box<'a, TSModuleDeclaration<'a>>> {
        let span = self.start_span();
        self.parse_ts_namespace_or_module_declaration_body(span, false)
    }

    pub fn parse_ts_namespace_or_module_statement(
        &mut self,
        declare: bool,
    ) -> Result<Statement<'a>> {
        self.parse_ts_namespace_or_module_declaration(declare)
            .map(|decl| Statement::Declaration(Declaration::TSModuleDeclaration(decl)))
    }

    pub fn parse_ts_global_statement(&mut self) -> Result<Statement<'a>> {
        self.parse_ts_global_declaration()
            .map(|decl| Statement::Declaration(Declaration::TSModuleDeclaration(decl)))
    }

    pub fn is_nth_at_ts_namespace_declaration(&mut self, n: u8) -> bool {
        if self.nth(n + 1).is_on_new_line {
            return false;
        }

        if self.nth_at(n, Kind::Module) || self.nth_at(n, Kind::Namespace) {
            return self.nth_kind(n + 1).is_identifier() || self.nth_at(n + 1, Kind::Str);
        }

        if self.nth_at(n, Kind::Global) {
            return self.nth_at(n + 1, Kind::LCurly);
        }

        false
    }

    /** ----------------------- declaration --------------------- */

    pub fn is_at_ts_declaration_clause(&mut self) -> bool {
        if !self.at(Kind::Declare) || self.peek_token().is_on_new_line {
            return false;
        }

        if matches!(
            self.peek_kind(),
            Kind::Function | Kind::Const | Kind::Enum | Kind::Class | Kind::Import
        ) {
            return true;
        }

        if self.peek_kind().is_variable_declaration() {
            return true;
        }

        if self.nth(2).is_on_new_line {
            return false;
        }

        if self.peek_at(Kind::Type) || self.peek_at(Kind::Interface) {
            return true;
        }

        if self.peek_at(Kind::Async) && self.nth_at(2, Kind::Function) {
            return true;
        }

        if self.is_nth_at_ts_namespace_declaration(1) {
            return true;
        }

        if self.peek_at(Kind::Abstract) && self.nth_at(2, Kind::Class) {
            return true;
        }

        false
    }

    pub fn parse_ts_declare_statement(&mut self) -> Result<Statement<'a>> {
        let declaration = self.parse_declaration_clause()?;
        Ok(Statement::Declaration(declaration))
    }

    pub fn parse_declaration_clause(&mut self) -> Result<Declaration<'a>> {
        let has_ambient = self.ctx.has_ambient();
        let declare = self.eat(Kind::Declare);
        if declare {
            self.ctx = self.ctx.and_ambient(true);
        }

        let start_span = self.start_span();

        let result = match self.cur_kind() {
            Kind::Namespace | Kind::Module => self
                .parse_ts_namespace_or_module_declaration(declare)
                .map(Declaration::TSModuleDeclaration),
            Kind::Global => {
                let decl = if self.peek_at(Kind::LCurly) {
                    // valid syntax for
                    // declare global { }
                    self.parse_ts_namespace_or_module_declaration_body(start_span, declare)
                } else {
                    self.parse_ts_global_declaration()
                }?;
                Ok(Declaration::TSModuleDeclaration(decl))
            }
            Kind::Type => self.parse_ts_type_alias_declaration(declare, start_span),
            Kind::Const | Kind::Enum if self.is_at_enum_declaration() => {
                self.parse_ts_enum_declaration(declare, start_span)
            }
            Kind::Interface if self.is_at_interface_declaration() => {
                self.parse_ts_interface_declaration(declare, start_span)
            }
            Kind::Class | Kind::Abstract => {
                self.parse_class_declaration(declare).map(Declaration::ClassDeclaration)
            }
            Kind::Import => {
                self.bump_any();
                self.parse_ts_import_equals_declaration(start_span, true)
            }
            kind if kind.is_variable_declaration() => self
                .parse_variable_declaration(VariableDeclarationContext::new(
                    VariableDeclarationParent::Clause,
                ))
                .map(Declaration::VariableDeclaration),
            _ if self.at_function_with_async() => {
                if declare {
                    self.parse_ts_declare_function().map(Declaration::FunctionDeclaration)
                } else {
                    self.parse_function_impl(FunctionKind::Declaration { single_statement: true })
                        .map(Declaration::FunctionDeclaration)
                }
            }
            _ => self.unexpected(),
        };

        self.ctx = self.ctx.and_ambient(has_ambient);
        result
    }

    pub fn parse_ts_declare_function(&mut self) -> Result<Box<'a, Function<'a>>> {
        let span = self.start_span();
        let r#async = self.eat(Kind::Async);
        self.expect(Kind::Function)?;
        let func_kind = FunctionKind::TSDeclaration;
        let id = self.parse_function_id(func_kind, r#async, false);
        self.parse_function(span, id, r#async, false, func_kind)
    }

    pub fn parse_ts_type_assertion(&mut self) -> Result<Expression<'a>> {
        let span = self.start_span();
        self.re_lex_ts_l_angle();
        self.expect(Kind::LAngle)?;
        let type_annotation = self.parse_ts_type()?;
        self.expect(Kind::RAngle)?;
        let lhs_span = self.start_span();
        let expression = self.parse_unary_expression_base(lhs_span)?;
        Ok(self.ast.ts_type_assertion(self.end_span(span), type_annotation, expression))
    }

    pub fn parse_ts_import_equals_declaration(
        &mut self,
        span: Span,
        is_export: bool,
    ) -> Result<Declaration<'a>> {
        let import_kind = if !self.peek_at(Kind::Eq) && self.eat(Kind::Type) {
            ImportOrExportKind::Value
        } else {
            ImportOrExportKind::Type
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
            TSModuleReference::TypeName(self.parse_ts_qualified_name()?)
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

    pub fn parse_ts_this_parameter(&mut self) -> Result<()> {
        let _ident = self.parse_identifier_kind(Kind::Ident);
        let _ = self.parse_ts_type_annotation()?;
        Ok(())
    }

    pub fn eat_decorators(&mut self) -> Result<()> {
        if !self.at(Kind::At) {
            return Ok(());
        }

        let in_decorator = self.ctx.has_decorator();
        self.ctx = self.ctx.and_decorator(true);

        let mut decorators = self.ast.new_vec();
        while self.at(Kind::At) {
            let decorator = self.parse_decorator()?;
            decorators.push(decorator);
        }

        self.ctx = self.ctx.and_decorator(in_decorator);

        self.state.decorators = Some(decorators);
        Ok(())
    }

    fn parse_decorator(&mut self) -> Result<Decorator<'a>> {
        self.bump_any(); // bump @
        let span = self.start_span();
        let expr = self.with_context(Context::Decorator, Self::parse_lhs_expression)?;
        Ok(self.ast.decorator(self.end_span(span), expr))
    }
}
