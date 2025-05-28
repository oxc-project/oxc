use oxc_allocator::{Box, Vec};
use oxc_ast::{NONE, ast::*};
use oxc_span::GetSpan;
use rustc_hash::FxHashMap;

use super::FunctionKind;
use crate::{Context, ParserImpl, diagnostics, lexer::Kind, modifiers::Modifiers};

impl<'a> ParserImpl<'a> {
    /// [Import Call](https://tc39.es/ecma262/#sec-import-calls)
    /// `ImportCall` : import ( `AssignmentExpression` )
    pub(crate) fn parse_import_expression(
        &mut self,
        span: u32,
        phase: Option<ImportPhase>,
    ) -> Expression<'a> {
        self.expect(Kind::LParen);
        if self.eat(Kind::RParen) {
            let error = diagnostics::import_requires_a_specifier(self.end_span(span));
            return self.fatal_error(error);
        }
        let has_in = self.ctx.has_in();
        self.ctx = self.ctx.and_in(true);
        let expression = self.parse_assignment_expression_or_higher();
        let arguments = if self.eat(Kind::Comma) && !self.at(Kind::RParen) {
            Some(self.parse_assignment_expression_or_higher())
        } else {
            None
        };
        // Allow trailing comma
        self.bump(Kind::Comma);
        if !self.eat(Kind::RParen) {
            let error = diagnostics::import_arguments(self.end_span(span));
            return self.fatal_error(error);
        }
        self.ctx = self.ctx.and_in(has_in);
        let expr =
            self.ast.alloc_import_expression(self.end_span(span), expression, arguments, phase);
        self.module_record_builder.visit_import_expression(&expr);
        Expression::ImportExpression(expr)
    }

    /// Section 16.2.2 Import Declaration
    pub(crate) fn parse_import_declaration(&mut self) -> Statement<'a> {
        let span = self.start_span();

        self.expect(Kind::Import);

        // `import something = ...`
        // `import type something = ...`
        if self.is_ts
            && ((self.cur_kind().is_binding_identifier()
                && self.lookahead(Self::is_next_token_equals))
                || (self.at(Kind::Type)
                    && self.lookahead(|p| {
                        p.bump_any();
                        if !p.cur_kind().is_binding_identifier() {
                            return false;
                        }
                        p.bump_any();
                        p.at(Kind::Eq)
                    })))
        {
            let decl = self.parse_ts_import_equals_declaration(span);
            return Statement::from(decl);
        }

        // `import type ...`
        // `import source ...`
        // `import defer ...`
        let mut import_kind = ImportOrExportKind::Value;
        let mut phase = None;
        match self.cur_kind() {
            Kind::Source => {
                // `import source something from ...`
                if self.lookahead(|p| {
                    p.bump_any();
                    if !p.cur_kind().is_binding_identifier() {
                        return false;
                    }
                    p.bump_any();
                    p.at(Kind::From)
                }) {
                    self.bump_any();
                    phase = Some(ImportPhase::Source);
                }
            }
            Kind::Defer
                if self.lookahead(|p| {
                    p.bump_any();
                    p.at(Kind::Star)
                }) =>
            {
                // `import defer * ...`
                self.bump_any();
                phase = Some(ImportPhase::Defer);
            }
            Kind::Type if self.is_ts => import_kind = self.parse_import_or_export_kind(),
            _ => {}
        }

        let specifiers = if self.at(Kind::Str) {
            // import "source"
            None
        } else {
            Some(self.parse_import_declaration_specifiers(import_kind))
        };

        let source = self.parse_literal_string();
        let with_clause = self.parse_import_attributes();
        self.asi();
        let span = self.end_span(span);
        self.ast
            .module_declaration_import_declaration(
                span,
                specifiers,
                source,
                phase,
                with_clause,
                import_kind,
            )
            .into()
    }

    // Full Syntax: <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/import#syntax>
    fn parse_import_declaration_specifiers(
        &mut self,
        import_kind: ImportOrExportKind,
    ) -> Vec<'a, ImportDeclarationSpecifier<'a>> {
        let mut specifiers = self.ast.vec();
        // import defaultExport from "module-name";
        if self.cur_kind().is_binding_identifier() {
            specifiers.push(self.parse_import_default_specifier());
            if self.eat(Kind::Comma) {
                match self.cur_kind() {
                    // import defaultExport, * as name from "module-name";
                    Kind::Star => specifiers.push(self.parse_import_namespace_specifier()),
                    // import defaultExport, { export1 [ , [...] ] } from "module-name";
                    Kind::LCurly => {
                        let mut import_specifiers = self.parse_import_specifiers(import_kind);
                        specifiers.append(&mut import_specifiers);
                    }
                    _ => return self.unexpected(),
                }
            }
        // import * as name from "module-name";
        } else if self.at(Kind::Star) {
            specifiers.push(self.parse_import_namespace_specifier());
        // import { export1 , export2 as alias2 , [...] } from "module-name";
        } else if self.at(Kind::LCurly) {
            let mut import_specifiers = self.parse_import_specifiers(import_kind);
            specifiers.append(&mut import_specifiers);
        }

        self.expect(Kind::From);
        specifiers
    }

    // import default from "module-name"
    fn parse_import_default_specifier(&mut self) -> ImportDeclarationSpecifier<'a> {
        let span = self.start_span();
        let local = self.parse_binding_identifier();
        let span = self.end_span(span);
        self.ast.import_declaration_specifier_import_default_specifier(span, local)
    }

    // import * as name from "module-name"
    fn parse_import_namespace_specifier(&mut self) -> ImportDeclarationSpecifier<'a> {
        let span = self.start_span();
        self.bump_any(); // advance `*`
        self.expect(Kind::As);
        let local = self.parse_binding_identifier();
        let span = self.end_span(span);
        self.ast.import_declaration_specifier_import_namespace_specifier(span, local)
    }

    // import { export1 , export2 as alias2 , [...] } from "module-name";
    fn parse_import_specifiers(
        &mut self,
        import_kind: ImportOrExportKind,
    ) -> Vec<'a, ImportDeclarationSpecifier<'a>> {
        self.expect(Kind::LCurly);
        let list = self.context(Context::empty(), self.ctx, |p| {
            p.parse_delimited_list(
                Kind::RCurly,
                Kind::Comma,
                /* trailing_separator */ true,
                |parser| parser.parse_import_specifier(import_kind),
            )
        });
        self.expect(Kind::RCurly);
        list
    }

    /// [Import Attributes](https://tc39.es/proposal-import-attributes)
    fn parse_import_attributes(&mut self) -> Option<WithClause<'a>> {
        let attributes_keyword = match self.cur_kind() {
            Kind::Assert if !self.cur_token().is_on_new_line() => self.parse_identifier_name(),
            Kind::With => self.parse_identifier_name(),
            _ => {
                return None;
            }
        };
        let span = self.start_span();
        self.expect(Kind::LCurly);
        let with_entries = self.context(Context::empty(), self.ctx, |p| {
            p.parse_delimited_list(
                Kind::RCurly,
                Kind::Comma,
                /*trailing_separator*/ true,
                Self::parse_import_attribute,
            )
        });
        self.expect(Kind::RCurly);

        let mut keys = FxHashMap::default();
        for e in &with_entries {
            let key = e.key.as_atom().as_str();
            let span = e.key.span();
            if let Some(old_span) = keys.insert(key, span) {
                self.error(diagnostics::redeclaration(key, old_span, span));
            }
        }

        Some(self.ast.with_clause(self.end_span(span), attributes_keyword, with_entries))
    }

    fn parse_import_attribute(&mut self) -> ImportAttribute<'a> {
        let span = self.start_span();
        let key = match self.cur_kind() {
            Kind::Str => ImportAttributeKey::StringLiteral(self.parse_literal_string()),
            _ => ImportAttributeKey::Identifier(self.parse_identifier_name()),
        };
        self.expect(Kind::Colon);
        let value = self.parse_literal_string();
        self.ast.import_attribute(self.end_span(span), key, value)
    }

    pub(crate) fn parse_ts_export_assignment_declaration(
        &mut self,
        start_span: u32,
    ) -> Box<'a, TSExportAssignment<'a>> {
        self.expect(Kind::Eq);
        let expression = self.parse_assignment_expression_or_higher();
        self.asi();
        self.ast.alloc_ts_export_assignment(self.end_span(start_span), expression)
    }

    pub(crate) fn parse_ts_export_namespace(
        &mut self,
        start_span: u32,
    ) -> Box<'a, TSNamespaceExportDeclaration<'a>> {
        self.expect(Kind::As);
        self.expect(Kind::Namespace);
        let id = self.parse_identifier_name();
        self.asi();
        self.ast.alloc_ts_namespace_export_declaration(self.end_span(start_span), id)
    }

    /// [Exports](https://tc39.es/ecma262/#sec-exports)
    pub(crate) fn parse_export_declaration(&mut self) -> Statement<'a> {
        let span = self.start_span();
        self.expect(Kind::Export);

        let decl = match self.cur_kind() {
            Kind::Eq if self.is_ts => ModuleDeclaration::TSExportAssignment(
                self.parse_ts_export_assignment_declaration(span),
            ),
            Kind::As
                if self.is_ts
                    && self.lookahead(|p| {
                        p.bump_any();
                        p.at(Kind::Namespace)
                    }) =>
            {
                // `export as namespace ...`
                ModuleDeclaration::TSNamespaceExportDeclaration(
                    self.parse_ts_export_namespace(span),
                )
            }
            Kind::Default => ModuleDeclaration::ExportDefaultDeclaration(
                self.parse_export_default_declaration(span),
            ),
            Kind::Star => {
                ModuleDeclaration::ExportAllDeclaration(self.parse_export_all_declaration(span))
            }
            Kind::LCurly => {
                ModuleDeclaration::ExportNamedDeclaration(self.parse_export_named_specifiers(span))
            }
            Kind::Type
                if self.is_ts
                    && self.lookahead(|p| {
                        p.bump_any();
                        p.at(Kind::LCurly)
                    }) =>
            {
                // `export type { ...`
                ModuleDeclaration::ExportNamedDeclaration(self.parse_export_named_specifiers(span))
            }
            Kind::Type
                if self.lookahead(|p| {
                    p.bump_any();
                    p.at(Kind::Star)
                }) =>
            {
                // `export type * as ...`
                ModuleDeclaration::ExportAllDeclaration(self.parse_export_all_declaration(span))
            }
            _ => {
                ModuleDeclaration::ExportNamedDeclaration(self.parse_export_named_declaration(span))
            }
        };
        Statement::from(decl)
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
    fn parse_export_named_specifiers(&mut self, span: u32) -> Box<'a, ExportNamedDeclaration<'a>> {
        let export_kind = self.parse_import_or_export_kind();
        self.expect(Kind::LCurly);
        let mut specifiers = self.context(Context::empty(), self.ctx, |p| {
            p.parse_delimited_list(
                Kind::RCurly,
                Kind::Comma,
                /* trailing_separator */ true,
                |parser| parser.parse_export_named_specifier(export_kind),
            )
        });
        self.expect(Kind::RCurly);
        let (source, with_clause) = if self.eat(Kind::From) && self.cur_kind().is_literal() {
            let source = self.parse_literal_string();
            (Some(source), self.parse_import_attributes())
        } else {
            (None, None)
        };

        // ExportDeclaration : export NamedExports ;
        if source.is_none() {
            for specifier in &mut specifiers {
                match &specifier.local {
                    // It is a Syntax Error if ReferencedBindings of NamedExports contains any StringLiterals.
                    ModuleExportName::StringLiteral(literal) => {
                        self.error(diagnostics::export_named_string(
                            &specifier.local.to_string(),
                            &specifier.exported.to_string(),
                            literal.span,
                        ));
                    }
                    // For each IdentifierName n in ReferencedBindings of NamedExports:
                    // It is a Syntax Error if StringValue of n is a ReservedWord or the StringValue of n
                    // is one of "implements", "interface", "let", "package", "private", "protected", "public", or "static".
                    ModuleExportName::IdentifierName(ident) => {
                        let match_result = Kind::match_keyword(&ident.name);
                        if match_result.is_reserved_keyword()
                            || match_result.is_future_reserved_keyword()
                        {
                            self.error(diagnostics::export_reserved_word(
                                &specifier.local.to_string(),
                                &specifier.exported.to_string(),
                                ident.span,
                            ));
                        }

                        // `local` becomes a reference for `export { local }`.
                        specifier.local = ModuleExportName::IdentifierReference(
                            self.ast.identifier_reference(ident.span, ident.name.as_str()),
                        );
                    }
                    // No prior code path should lead to parsing `ModuleExportName` as `IdentifierReference`.
                    ModuleExportName::IdentifierReference(_) => unreachable!(),
                }
            }
        }

        self.asi();
        let span = self.end_span(span);
        self.ast.alloc_export_named_declaration(
            span,
            None,
            specifiers,
            source,
            export_kind,
            with_clause,
        )
    }

    // export Declaration
    fn parse_export_named_declaration(&mut self, span: u32) -> Box<'a, ExportNamedDeclaration<'a>> {
        let decl_span = self.start_span();
        // For tc39/proposal-decorators
        // For more information, please refer to <https://babeljs.io/docs/babel-plugin-proposal-decorators#decoratorsbeforeexport>
        self.eat_decorators();
        let reserved_ctx = self.ctx;
        let modifiers =
            if self.is_ts { self.eat_modifiers_before_declaration() } else { Modifiers::empty() };
        self.ctx = self.ctx.union_ambient_if(modifiers.contains_declare());

        let declaration = self.parse_declaration(decl_span, &modifiers);
        let export_kind = if declaration.declare() || declaration.is_type() {
            ImportOrExportKind::Type
        } else {
            ImportOrExportKind::Value
        };
        self.ctx = reserved_ctx;
        self.ast.alloc_export_named_declaration(
            self.end_span(span),
            Some(declaration),
            self.ast.vec(),
            None,
            export_kind,
            NONE,
        )
    }

    // export default HoistableDeclaration[~Yield, +Await, +Default]
    // export default ClassDeclaration[~Yield, +Await, +Default]
    // export default AssignmentExpression[+In, ~Yield, +Await] ;
    fn parse_export_default_declaration(
        &mut self,
        span: u32,
    ) -> Box<'a, ExportDefaultDeclaration<'a>> {
        let exported = self.parse_keyword_identifier(Kind::Default);
        let decl_span = self.start_span();
        let has_no_side_effects_comment =
            self.lexer.trivia_builder.previous_token_has_no_side_effects_comment();
        // For tc39/proposal-decorators
        // For more information, please refer to <https://babeljs.io/docs/babel-plugin-proposal-decorators#decoratorsbeforeexport>
        self.eat_decorators();
        let declaration = match self.cur_kind() {
            Kind::Class => ExportDefaultDeclarationKind::ClassDeclaration(
                self.parse_class_declaration(decl_span, /* modifiers */ &Modifiers::empty()),
            ),
            _ if self.is_ts
                && self.at(Kind::Abstract)
                && self.lookahead(|p| {
                    p.bump_any();
                    p.at(Kind::Class)
                }) =>
            {
                // `export default abstract class ...`
                // eat the abstract modifier
                let modifiers = self.eat_modifiers_before_declaration();
                ExportDefaultDeclarationKind::ClassDeclaration(
                    self.parse_class_declaration(decl_span, &modifiers),
                )
            }
            _ if self.is_ts
                && self.at(Kind::Interface)
                && self.lookahead(|p| {
                    p.bump_any();
                    !p.cur_token().is_on_new_line()
                }) =>
            {
                // `export default interface [no line break here] ...`
                let decl = self.parse_ts_interface_declaration(decl_span, &Modifiers::empty());
                match decl {
                    Declaration::TSInterfaceDeclaration(decl) => {
                        ExportDefaultDeclarationKind::TSInterfaceDeclaration(decl)
                    }
                    _ => unreachable!(),
                }
            }
            _ if self.at_function_with_async() => {
                let mut func = self.parse_function_impl(FunctionKind::DefaultExport);
                if has_no_side_effects_comment {
                    func.pure = true;
                }
                ExportDefaultDeclarationKind::FunctionDeclaration(func)
            }
            _ => {
                let decl = ExportDefaultDeclarationKind::from(
                    self.parse_assignment_expression_or_higher(),
                );
                self.asi();
                decl
            }
        };
        let exported = ModuleExportName::IdentifierName(exported);
        let span = self.end_span(span);
        self.ast.alloc_export_default_declaration(span, exported, declaration)
    }

    // export ExportFromClause FromClause ;
    // ExportFromClause :
    //   *
    //   * as ModuleExportName
    //   NamedExports
    fn parse_export_all_declaration(&mut self, span: u32) -> Box<'a, ExportAllDeclaration<'a>> {
        let export_kind = self.parse_import_or_export_kind();
        self.bump_any(); // bump `star`
        let exported = self.eat(Kind::As).then(|| self.parse_module_export_name());
        self.expect(Kind::From);
        let source = self.parse_literal_string();
        let with_clause = self.parse_import_attributes();
        self.asi();
        let span = self.end_span(span);
        self.ast.alloc_export_all_declaration(span, exported, source, with_clause, export_kind)
    }

    // ImportSpecifier :
    //   ImportedBinding
    //   ModuleExportName as ImportedBinding
    pub(crate) fn parse_import_specifier(
        &mut self,
        parent_import_kind: ImportOrExportKind,
    ) -> ImportDeclarationSpecifier<'a> {
        let specifier_span = self.start_span();
        let mut import_kind = ImportOrExportKind::Value;
        let checkpoint = self.checkpoint();
        if self.is_ts && self.at(Kind::Type) {
            self.bump_any();
            if self.at(Kind::As) {
                // { type as ...? }
                self.bump_any();
                if self.at(Kind::As) {
                    // { type as as ...? }
                    self.bump_any();
                    if self.can_parse_module_export_name() {
                        // { type as as something }
                        // { type as as "something" }
                        import_kind = ImportOrExportKind::Type;
                    }
                } else if !self.can_parse_module_export_name() {
                    // { type as }
                    import_kind = ImportOrExportKind::Type;
                }
            } else if self.can_parse_module_export_name() {
                // { type something }
                import_kind = ImportOrExportKind::Type;
            }
        }
        self.rewind(checkpoint);

        // `import type { type bar } from 'foo';`
        if parent_import_kind == ImportOrExportKind::Type && import_kind == ImportOrExportKind::Type
        {
            self.error(diagnostics::type_modifier_on_named_type_import(self.cur_token().span()));
        }
        if import_kind == ImportOrExportKind::Type {
            self.bump_any();
        }
        let is_next_token_as = self.lookahead(|p| {
            p.bump_any();
            p.at(Kind::As)
        });
        let (imported, local) = if is_next_token_as {
            let imported = self.parse_module_export_name();
            self.bump(Kind::As);
            let local = self.parse_binding_identifier();
            (imported, local)
        } else {
            let local = self.parse_binding_identifier();
            (self.ast.module_export_name_identifier_name(local.span, local.name), local)
        };
        self.ast.import_declaration_specifier_import_specifier(
            self.end_span(specifier_span),
            imported,
            local,
            import_kind,
        )
    }

    // ModuleExportName :
    //   IdentifierName
    //   StringLiteral
    pub(crate) fn parse_module_export_name(&mut self) -> ModuleExportName<'a> {
        match self.cur_kind() {
            Kind::Str => {
                let literal = self.parse_literal_string();
                // ModuleExportName : StringLiteral
                // It is a Syntax Error if IsStringWellFormedUnicode(the SV of StringLiteral) is false.
                if literal.lone_surrogates || !literal.is_string_well_formed_unicode() {
                    self.error(diagnostics::export_lone_surrogate(literal.span));
                }
                ModuleExportName::StringLiteral(literal)
            }
            _ => ModuleExportName::IdentifierName(self.parse_identifier_name()),
        }
    }

    fn parse_import_or_export_kind(&mut self) -> ImportOrExportKind {
        if !self.is_ts {
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

        let next_token = {
            let checkpoint = self.checkpoint();
            self.bump_any();
            let next_token = self.cur_token();
            self.rewind(checkpoint);
            next_token
        };

        if matches!(next_token.kind(), Kind::LCurly | Kind::Star) {
            self.bump_any();
            return ImportOrExportKind::Type;
        }

        if !(next_token.kind() == Kind::Ident) && !next_token.kind().is_contextual_keyword() {
            return ImportOrExportKind::Value;
        }

        if next_token.kind() != Kind::From
            || self.lookahead(|p| {
                p.bump_any();
                p.bump_any();
                p.at(Kind::From)
            })
        {
            self.bump_any();
            return ImportOrExportKind::Type;
        }

        ImportOrExportKind::Value
    }

    fn parse_export_named_specifier(
        &mut self,
        parent_export_kind: ImportOrExportKind,
    ) -> ExportSpecifier<'a> {
        let specifier_span = self.start_span();
        let checkpoint = self.checkpoint();
        // export { type}              // name: `type`
        // export { type type }        // name: `type`    type-export: `true`
        // export { type as }          // name: `as`      type-export: `true`
        // export { type as as }       // name: `type`    type-export: `false` (aliased to `as`)
        // export { type as as as }    // name: `as`      type-export: `true`, aliased to `as`
        let mut export_kind = ImportOrExportKind::Value;
        if self.is_ts && self.at(Kind::Type) {
            self.bump_any();
            if self.at(Kind::As) {
                // { type as ...? }
                self.bump_any();
                if self.at(Kind::As) {
                    // { type as as ...? }
                    self.bump_any();
                    if self.can_parse_module_export_name() {
                        // { type as as something }
                        // { type as as "something" }
                        export_kind = ImportOrExportKind::Type;
                    }
                } else if !self.can_parse_module_export_name() {
                    export_kind = ImportOrExportKind::Type;
                }
            } else if self.can_parse_module_export_name() {
                // { type something }
                export_kind = ImportOrExportKind::Type;
            }
        }
        self.rewind(checkpoint);

        // `export type { type bar } from 'foo';`
        if parent_export_kind == ImportOrExportKind::Type && export_kind == ImportOrExportKind::Type
        {
            self.error(diagnostics::type_modifier_on_named_type_export(self.cur_token().span()));
        }

        if export_kind == ImportOrExportKind::Type {
            self.bump_any();
        }

        let local = self.parse_module_export_name();
        let exported =
            if self.eat(Kind::As) { self.parse_module_export_name() } else { local.clone() };
        self.ast.export_specifier(self.end_span(specifier_span), local, exported, export_kind)
    }

    fn can_parse_module_export_name(&self) -> bool {
        self.cur_kind().is_identifier_name() || self.at(Kind::Str)
    }
}
