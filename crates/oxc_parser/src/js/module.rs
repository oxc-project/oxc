use oxc_allocator::{Box, Vec};
use oxc_ast::ast::*;
use oxc_diagnostics::Result;
use oxc_span::Span;

use super::{
    function::FunctionKind,
    list::{AssertEntries, ExportNamedSpecifiers, ImportSpecifierList},
};
use crate::{diagnostics, lexer::Kind, list::SeparatedList, Context, Parser};

impl<'a> Parser<'a> {
    /// [Import Call](https://tc39.es/ecma262/#sec-import-calls)
    /// `ImportCall` : import ( `AssignmentExpression` )
    pub(crate) fn parse_import_expression(&mut self, span: Span) -> Result<Expression<'a>> {
        self.bump_any(); // advance '('

        let has_in = self.ctx.has_in();
        self.ctx = self.ctx.and_in(true);

        let expression = self.parse_assignment_expression_base()?;
        let mut arguments = self.ast.new_vec();
        if self.eat(Kind::Comma) && !self.at(Kind::RParen) {
            arguments.push(self.parse_assignment_expression_base()?);
        }

        self.ctx = self.ctx.and_in(has_in);
        self.ctx = self.ctx.and_in(has_in);
        self.bump(Kind::Comma);
        self.expect(Kind::RParen)?;
        Ok(self.ast.import_expression(self.end_span(span), expression, arguments))
    }

    /// Section 16.2.2 Import Declaration
    pub(crate) fn parse_import_declaration(&mut self) -> Result<Statement<'a>> {
        let span = self.start_span();

        self.bump_any(); // advance `import`

        if self.ts_enabled()
            && ((self.cur_kind().is_binding_identifier() && self.peek_at(Kind::Eq))
                || (self.at(Kind::Type)
                    && self.peek_kind().is_binding_identifier()
                    && self.nth_at(2, Kind::Eq)))
        {
            let decl = self.parse_ts_import_equals_declaration(span, false)?;
            return Ok(Statement::Declaration(decl));
        }

        // `import type ...`
        let import_kind = self.parse_import_or_export_kind();

        let specifiers = if self.at(Kind::Str) {
            // import "source"
            None
        } else {
            Some(self.parse_import_declaration_specifiers()?)
        };

        let source = self.parse_literal_string()?;
        let assertions = self.parse_import_attributes()?;
        self.asi()?;
        let span = self.end_span(span);
        let decl = ModuleDeclaration::ImportDeclaration(self.ast.import_declaration(
            span,
            specifiers,
            source,
            assertions,
            import_kind,
        ));
        Ok(self.ast.module_declaration(decl))
    }

    // Full Syntax: <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/import#syntax>
    fn parse_import_declaration_specifiers(
        &mut self,
    ) -> Result<Vec<'a, ImportDeclarationSpecifier>> {
        let mut specifiers = self.ast.new_vec();
        // import defaultExport from "module-name";
        if self.cur_kind().is_binding_identifier() {
            specifiers.push(self.parse_import_default_specifier()?);
            if self.eat(Kind::Comma) {
                match self.cur_kind() {
                    // import defaultExport, * as name from "module-name";
                    Kind::Star => specifiers.push(self.parse_import_namespace_specifier()?),
                    // import defaultExport, { export1 [ , [...] ] } from "module-name";
                    Kind::LCurly => {
                        let mut import_specifiers = self.parse_import_specifiers()?;
                        specifiers.append(&mut import_specifiers);
                    }
                    _ => return Err(self.unexpected()),
                }
            }
        // import * as name from "module-name";
        } else if self.at(Kind::Star) {
            specifiers.push(self.parse_import_namespace_specifier()?);
        // import { export1 , export2 as alias2 , [...] } from "module-name";
        } else if self.at(Kind::LCurly) {
            let mut import_specifiers = self.parse_import_specifiers()?;
            specifiers.append(&mut import_specifiers);
        };

        self.expect(Kind::From)?;
        Ok(specifiers)
    }

    // import default from "module-name"
    fn parse_import_default_specifier(&mut self) -> Result<ImportDeclarationSpecifier> {
        let span = self.start_span();
        let local = self.parse_binding_identifier()?;
        Ok(ImportDeclarationSpecifier::ImportDefaultSpecifier(ImportDefaultSpecifier {
            span: self.end_span(span),
            local,
        }))
    }

    // import * as name from "module-name"
    fn parse_import_namespace_specifier(&mut self) -> Result<ImportDeclarationSpecifier> {
        let span = self.start_span();
        self.bump_any(); // advance `*`
        self.expect(Kind::As)?;
        let local = self.parse_binding_identifier()?;
        Ok(ImportDeclarationSpecifier::ImportNamespaceSpecifier(ImportNamespaceSpecifier {
            span: self.end_span(span),
            local,
        }))
    }

    // import { export1 , export2 as alias2 , [...] } from "module-name";
    fn parse_import_specifiers(&mut self) -> Result<Vec<'a, ImportDeclarationSpecifier>> {
        let ctx = self.ctx;
        self.ctx = Context::default();
        let specifiers = ImportSpecifierList::parse(self)?.import_specifiers;
        self.ctx = ctx;
        Ok(specifiers)
    }

    /// [Import assertion](https://tc39.es/proposal-import-assertions)
    fn parse_import_attributes(&mut self) -> Result<Option<Vec<'a, ImportAttribute>>> {
        if !self.at(Kind::Assert) || self.cur_token().is_on_new_line {
            return Ok(None);
        }
        self.bump_any();

        let ctx = self.ctx;
        self.ctx = Context::default();
        let entries = AssertEntries::parse(self)?.elements;
        self.ctx = ctx;

        Ok(Some(entries))
    }

    pub(crate) fn parse_ts_export_assignment_declaration(
        &mut self,
    ) -> Result<Box<'a, TSExportAssignment<'a>>> {
        let span = self.start_span();
        self.expect(Kind::Eq)?;

        let expression = self.parse_assignment_expression_base()?;
        self.asi()?;

        Ok(self.ast.alloc(TSExportAssignment { span: self.end_span(span), expression }))
    }

    pub(crate) fn parse_ts_export_namespace(
        &mut self,
    ) -> Result<Box<'a, TSNamespaceExportDeclaration>> {
        let span = self.start_span();
        self.expect(Kind::As)?;
        self.expect(Kind::Namespace)?;

        let id = self.parse_identifier_name()?;
        self.asi()?;

        Ok(self.ast.alloc(TSNamespaceExportDeclaration { span: self.end_span(span), id }))
    }

    /// [Exports](https://tc39.es/ecma262/#sec-exports)
    pub(crate) fn parse_export_declaration(&mut self) -> Result<Statement<'a>> {
        let span = self.start_span();
        self.bump_any(); // advance `export`

        let decl = match self.cur_kind() {
            Kind::Eq if self.ts_enabled() => self
                .parse_ts_export_assignment_declaration()
                .map(ModuleDeclaration::TSExportAssignment),
            Kind::As if self.peek_at(Kind::Namespace) && self.ts_enabled() => self
                .parse_ts_export_namespace()
                .map(ModuleDeclaration::TSNamespaceExportDeclaration),
            Kind::Default => self
                .parse_export_default_declaration(span)
                .map(ModuleDeclaration::ExportDefaultDeclaration),
            Kind::Star => {
                self.parse_export_all_declaration(span).map(ModuleDeclaration::ExportAllDeclaration)
            }
            Kind::LCurly => self
                .parse_export_named_specifiers(span)
                .map(ModuleDeclaration::ExportNamedDeclaration),
            Kind::Type if self.peek_at(Kind::LCurly) && self.ts_enabled() => self
                .parse_export_named_specifiers(span)
                .map(ModuleDeclaration::ExportNamedDeclaration),
            Kind::Type if self.peek_at(Kind::Star) => {
                self.parse_export_all_declaration(span).map(ModuleDeclaration::ExportAllDeclaration)
            }
            _ => self
                .parse_export_named_declaration(span)
                .map(ModuleDeclaration::ExportNamedDeclaration),
        }?;
        Ok(self.ast.module_declaration(decl))
    }

    // export NamedExports ;
    // NamedExports :
    //   { }
    //   { ExportsList }
    //   { ExportsList , }
    // ExportsList :
    //   ExportSpecifier
    //   ExportsList , ExportSpecifier
    // ExportSpecifier :
    //   ModuleExportName
    //   ModuleExportName as ModuleExportName
    fn parse_export_named_specifiers(
        &mut self,
        span: Span,
    ) -> Result<Box<'a, ExportNamedDeclaration<'a>>> {
        let export_kind = self.parse_import_or_export_kind();

        let ctx = self.ctx;
        self.ctx = Context::default();
        let specifiers = ExportNamedSpecifiers::parse(self)?.elements;
        self.ctx = ctx;

        let source = if self.eat(Kind::From) && self.cur_kind().is_literal() {
            let source = self.parse_literal_string()?;
            Some(source)
        } else {
            None
        };

        // ExportDeclaration : export NamedExports ;
        if source.is_none() {
            for specifier in &specifiers {
                match &specifier.local {
                    // It is a Syntax Error if ReferencedBindings of NamedExports contains any StringLiterals.
                    ModuleExportName::StringLiteral(literal) => {
                        self.error(diagnostics::ExportNamedString(
                            specifier.local.to_string(),
                            specifier.exported.to_string(),
                            literal.span,
                        ));
                    }
                    // For each IdentifierName n in ReferencedBindings of NamedExports:
                    // It is a Syntax Error if StringValue of n is a ReservedWord or the StringValue of n
                    // is one of "implements", "interface", "let", "package", "private", "protected", "public", or "static".
                    ModuleExportName::Identifier(id) => {
                        let match_result = Kind::match_keyword(&id.name);
                        if match_result.is_reserved_keyword()
                            || match_result.is_future_reserved_keyword()
                        {
                            self.error(diagnostics::ExportReservedWord(
                                specifier.local.to_string(),
                                specifier.exported.to_string(),
                                id.span,
                            ));
                        }
                    }
                }
            }
        }

        self.asi()?;
        let span = self.end_span(span);
        Ok(self.ast.export_named_declaration(span, None, specifiers, source, export_kind))
    }

    // export Declaration
    fn parse_export_named_declaration(
        &mut self,
        span: Span,
    ) -> Result<Box<'a, ExportNamedDeclaration<'a>>> {
        let decl_span = self.start_span();
        // For tc39/proposal-decorators
        // For more information, please refer to <https://babeljs.io/docs/babel-plugin-proposal-decorators#decoratorsbeforeexport>
        self.eat_decorators()?;
        let modifiers = if self.ts_enabled() {
            self.eat_modifiers_before_declaration().1
        } else {
            Modifiers::empty()
        };

        let declaration = self.parse_declaration(decl_span, modifiers)?;
        let span = self.end_span(span);
        Ok(self.ast.export_named_declaration(
            span,
            Some(declaration),
            self.ast.new_vec(),
            None,
            ImportOrExportKind::Value,
        ))
    }

    // export default HoistableDeclaration[~Yield, +Await, +Default]
    // export default ClassDeclaration[~Yield, +Await, +Default]
    // export default AssignmentExpression[+In, ~Yield, +Await] ;
    fn parse_export_default_declaration(
        &mut self,
        span: Span,
    ) -> Result<Box<'a, ExportDefaultDeclaration<'a>>> {
        let exported = self.parse_keyword_identifier(Kind::Default);
        let decl_span = self.start_span();
        // For tc39/proposal-decorators
        // For more information, please refer to <https://babeljs.io/docs/babel-plugin-proposal-decorators#decoratorsbeforeexport>
        self.eat_decorators()?;
        let declaration = match self.cur_kind() {
            Kind::Class => self
                .parse_class_declaration(decl_span, /* modifiers */ Modifiers::empty())
                .map(ExportDefaultDeclarationKind::ClassDeclaration)?,
            _ if self.at(Kind::Abstract) && self.peek_at(Kind::Class) && self.ts_enabled() => {
                // eat the abstract modifier
                let (_, modifiers) = self.eat_modifiers_before_declaration();
                self.parse_class_declaration(decl_span, modifiers)
                    .map(ExportDefaultDeclarationKind::ClassDeclaration)?
            }
            _ if self.at(Kind::Interface)
                && !self.peek_token().is_on_new_line
                && self.ts_enabled() =>
            {
                self.parse_ts_interface_declaration(decl_span, Modifiers::empty()).map(|decl| {
                    match decl {
                        Declaration::TSInterfaceDeclaration(decl) => {
                            ExportDefaultDeclarationKind::TSInterfaceDeclaration(decl)
                        }
                        _ => unreachable!(),
                    }
                })?
            }
            _ if self.at_function_with_async() => self
                .parse_function_impl(FunctionKind::DefaultExport)
                .map(ExportDefaultDeclarationKind::FunctionDeclaration)?,
            _ => {
                let decl = self
                    .parse_assignment_expression_base()
                    .map(ExportDefaultDeclarationKind::Expression)?;
                self.asi()?;
                decl
            }
        };
        let exported = ModuleExportName::Identifier(exported);
        let span = self.end_span(span);
        Ok(self.ast.export_default_declaration(span, declaration, exported))
    }

    // export ExportFromClause FromClause ;
    // ExportFromClause :
    //   *
    //   * as ModuleExportName
    //   NamedExports
    fn parse_export_all_declaration(
        &mut self,
        span: Span,
    ) -> Result<Box<'a, ExportAllDeclaration<'a>>> {
        let export_kind = self.parse_import_or_export_kind();
        self.bump_any(); // bump `star`
        let exported = self.eat(Kind::As).then(|| self.parse_module_export_name()).transpose()?;
        self.expect(Kind::From)?;
        let source = self.parse_literal_string()?;
        let assertions = self.parse_import_attributes()?;
        self.asi()?;
        let span = self.end_span(span);
        Ok(self.ast.export_all_declaration(span, exported, source, assertions, export_kind))
    }

    // ImportSpecifier :
    //   ImportedBinding
    //   ModuleExportName as ImportedBinding
    pub(crate) fn parse_import_specifier(&mut self) -> Result<ImportSpecifier> {
        let specifier_span = self.start_span();
        let peek_kind = self.peek_kind();
        let mut import_kind = ImportOrExportKind::Value;
        if self.ts_enabled() && self.at(Kind::Type) {
            if self.peek_at(Kind::As) {
                if self.nth_at(2, Kind::As) {
                    if self.nth_kind(3).is_identifier_name() {
                        import_kind = ImportOrExportKind::Type;
                    }
                } else if !self.nth_kind(2).is_identifier_name() {
                    import_kind = ImportOrExportKind::Type;
                }
            } else if peek_kind.is_identifier_name() {
                import_kind = ImportOrExportKind::Type;
            }
        }

        if import_kind == ImportOrExportKind::Type {
            self.bump_any();
        }
        let (imported, local) = if self.peek_at(Kind::As) {
            let imported = self.parse_module_export_name()?;
            self.bump(Kind::As);
            let local = self.parse_binding_identifier()?;
            (imported, local)
        } else {
            let local = self.parse_binding_identifier()?;
            let imported = IdentifierName { span: local.span, name: local.name.clone() };
            (ModuleExportName::Identifier(imported), local)
        };
        Ok(ImportSpecifier { span: self.end_span(specifier_span), imported, local, import_kind })
    }

    // ModuleExportName :
    //   IdentifierName
    //   StringLiteral
    pub(crate) fn parse_module_export_name(&mut self) -> Result<ModuleExportName> {
        match self.cur_kind() {
            Kind::Str => {
                let literal = self.parse_literal_string()?;
                // ModuleExportName : StringLiteral
                // It is a Syntax Error if IsStringWellFormedUnicode(the SV of StringLiteral) is false.
                if !literal.is_string_well_formed_unicode() {
                    self.error(diagnostics::ExportLoneSurrogate(literal.span));
                };
                Ok(ModuleExportName::StringLiteral(literal))
            }
            _ => Ok(ModuleExportName::Identifier(self.parse_identifier_name()?)),
        }
    }

    fn parse_import_or_export_kind(&mut self) -> ImportOrExportKind {
        if !self.ts_enabled() {
            return ImportOrExportKind::Value;
        }
        // OK
        // import type { bar } from 'foo';
        // import type * as React from 'react';
        // import type ident from 'foo';
        // export type { bar } from 'foo';

        // NO
        // import type from 'foo';

        // OK
        // import type from from 'foo';
        if !self.at(Kind::Type) {
            return ImportOrExportKind::Value;
        }

        if matches!(self.peek_kind(), Kind::LCurly | Kind::Star) {
            self.bump_any();
            return ImportOrExportKind::Type;
        }

        if !self.peek_at(Kind::Ident) && !self.peek_kind().is_contextual_keyword() {
            return ImportOrExportKind::Value;
        }

        if !self.peek_at(Kind::From) || self.nth_at(2, Kind::From) {
            self.bump_any();
            return ImportOrExportKind::Type;
        }

        ImportOrExportKind::Value
    }
}
