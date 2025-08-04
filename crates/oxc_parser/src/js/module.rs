use oxc_allocator::{Box, Vec};
use oxc_ast::{NONE, ast::*};
use oxc_span::GetSpan;
use rustc_hash::FxHashMap;

use super::FunctionKind;
use crate::{
    Context, ParserImpl, diagnostics,
    lexer::Kind,
    modifiers::{Modifier, ModifierFlags, ModifierKind, Modifiers},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportOrExport {
    /// Some kind of `import` statement/declaration
    Import,
    /// Some kind of `export` statement/declaration
    Export,
}

#[derive(Debug)]
enum ImportOrExportSpecifier<'a> {
    /// An import specifier, such as `import { a } from 'b'`
    Import(ImportSpecifier<'a>),
    /// An export specifier, such as `export { a } from 'b'`
    Export(ExportSpecifier<'a>),
}

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
    pub(crate) fn parse_import_declaration(&mut self, span: u32) -> Statement<'a> {
        let token_after_import = self.cur_token();
        let mut identifier_after_import: Option<BindingIdentifier<'_>> =
            if self.cur_kind().is_binding_identifier() {
                // `import something ...`
                Some(self.parse_binding_identifier())
            } else {
                // `import ...`
                None
            };
        let mut has_default_specifier = identifier_after_import.is_some();
        let mut should_parse_specifiers = true;

        let mut phase = None;
        let mut import_kind = ImportOrExportKind::Value;

        if self.at(Kind::Eq) && identifier_after_import.is_some() {
            // `import something = ...`
            let decl = self.parse_ts_import_equals_declaration(
                ImportOrExportKind::Value,
                identifier_after_import.unwrap(),
                span,
            );
            return Statement::from(decl);
        } else if self.is_ts && token_after_import.kind() == Kind::Type {
            // `import type ...`

            if token_after_import.escaped() {
                self.error(diagnostics::escaped_keyword(token_after_import.span()));
            }

            if self.at(Kind::LCurly) || self.at(Kind::Star) {
                // `import type { ...`
                // `import type * ...`
                import_kind = ImportOrExportKind::Type;
                has_default_specifier = false;
            } else if self.cur_kind().is_binding_identifier() {
                // `import type something ...`
                let token = self.cur_token();
                let identifier_after_type = self.parse_binding_identifier();
                if token.kind() == Kind::From && self.at(Kind::Str) {
                    // `import type from 'source'`
                    has_default_specifier = true;
                    import_kind = ImportOrExportKind::Value;
                    should_parse_specifiers = false;
                } else {
                    identifier_after_import = Some(identifier_after_type);
                    import_kind = ImportOrExportKind::Type;

                    if self.at(Kind::Eq) {
                        // `import type something = ...`
                        let decl = self.parse_ts_import_equals_declaration(
                            ImportOrExportKind::Type,
                            identifier_after_import.unwrap(),
                            span,
                        );
                        return Statement::from(decl);
                    } else if self.at(Kind::From) {
                        // `import type something from ...`
                        has_default_specifier = true;
                        should_parse_specifiers = false;
                    }
                }
            }
        } else if token_after_import.kind() == Kind::Defer && self.at(Kind::Star) {
            // `import defer * ...`
            phase = Some(ImportPhase::Defer);
            has_default_specifier = false;
        } else if token_after_import.kind() == Kind::Source
            && self.cur_kind().is_binding_identifier()
        {
            // `import source something ...`
            let kind = self.cur_kind();
            let identifier_after_source = self.parse_binding_identifier();
            if kind == Kind::From {
                // `import source from ...`
                if self.at(Kind::From) {
                    // `import source from from ...`
                    identifier_after_import = Some(identifier_after_source);
                    phase = Some(ImportPhase::Source);
                    has_default_specifier = true;
                } else if self.at(Kind::Str) {
                    // `import source from 'source'`
                    has_default_specifier = true;
                    should_parse_specifiers = false;
                }
            } else if self.at(Kind::From) {
                // `import source something from ...`
                identifier_after_import = Some(identifier_after_source);
                phase = Some(ImportPhase::Source);
                has_default_specifier = true;
            } else {
                return self.unexpected();
            }
        }

        let specifiers = if self.at(Kind::Str) {
            if has_default_specifier && !should_parse_specifiers {
                match identifier_after_import {
                    Some(identifier_after_import) => {
                        // Special case: `import type from 'source'` where we already consumed `type` and `from`
                        Some(self.ast.vec1(
                            self.ast.import_declaration_specifier_import_default_specifier(
                                identifier_after_import.span,
                                identifier_after_import,
                            ),
                        ))
                    }
                    None => unreachable!(),
                }
            } else {
                None
            }
        } else {
            let default_specifier =
                if has_default_specifier { identifier_after_import } else { None };

            Some(self.parse_import_declaration_specifiers(default_specifier, import_kind))
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
        // A default specifier, if we already saw any identifier after `import`
        default_specifier: Option<BindingIdentifier<'a>>,
        import_kind: ImportOrExportKind,
    ) -> Vec<'a, ImportDeclarationSpecifier<'a>> {
        // If there is a default specifier, create a Vec with the default specifier in it,
        // otherwise, create an empty Vec.
        let mut specifiers = if default_specifier.is_some() {
            self.ast.vec_with_capacity(1)
        } else {
            self.ast.vec()
        };

        if let Some(default_specifier) = default_specifier {
            specifiers.push(self.ast.import_declaration_specifier_import_default_specifier(
                default_specifier.span,
                default_specifier,
            ));
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

            // If the default specifiers name was `type`, and it was not a type import
            // then skip the `from` specifier expect. This is specifically to support an import
            // like: `import type from 'source'`
            if self.is_ts
                && import_kind == ImportOrExportKind::Value
                && specifiers.len() == 1
                && specifiers[0].name() == "type"
            {
                return specifiers;
            }
        } else if self.at(Kind::Star) {
            // import * as name from "module-name";
            specifiers.push(self.parse_import_namespace_specifier());
        } else if self.at(Kind::LCurly) {
            // import { export1 , export2 as alias2 , [...] } from "module-name";
            let mut import_specifiers = self.parse_import_specifiers(import_kind);
            specifiers.append(&mut import_specifiers);
        }

        self.expect(Kind::From);
        specifiers
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
        let (list, _) = self.context(Context::empty(), self.ctx, |p| {
            p.parse_delimited_list(Kind::RCurly, Kind::Comma, |parser| {
                parser.parse_import_specifier(import_kind)
            })
        });
        self.expect(Kind::RCurly);
        list
    }

    /// [Import Attributes](https://tc39.es/proposal-import-attributes)
    fn parse_import_attributes(&mut self) -> Option<WithClause<'a>> {
        let keyword_kind = self.cur_kind();
        let keyword = match keyword_kind {
            Kind::With => WithClauseKeyword::With,
            Kind::Assert if !self.cur_token().is_on_new_line() => WithClauseKeyword::Assert,
            _ => return None,
        };
        self.bump_remap(keyword_kind);

        let span = self.start_span();
        self.expect(Kind::LCurly);
        let (with_entries, _) = self.context(Context::empty(), self.ctx, |p| {
            p.parse_delimited_list(Kind::RCurly, Kind::Comma, Self::parse_import_attribute)
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

        Some(self.ast.with_clause(self.end_span(span), keyword, with_entries))
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
    pub(crate) fn parse_export_declaration(
        &mut self,
        span: u32,
        mut decorators: Vec<'a, Decorator<'a>>,
    ) -> Statement<'a> {
        self.bump_any(); // bump `export`
        let decl = match self.cur_kind() {
            // `export import A = B`
            Kind::Import => {
                let import_span = self.start_span();
                self.bump_any();
                let stmt = self.parse_import_declaration(import_span);
                if stmt.is_declaration() {
                    self.ast.module_declaration_export_named_declaration(
                        self.end_span(span),
                        Some(stmt.into_declaration()),
                        self.ast.vec(),
                        None,
                        ImportOrExportKind::Value,
                        NONE,
                    )
                } else {
                    return self.fatal_error(diagnostics::unexpected_export(stmt.span()));
                }
            }
            Kind::At => {
                let class_span = self.start_span();
                let after_export_decorators = self.parse_decorators();
                if !decorators.is_empty() {
                    for decorator in &after_export_decorators {
                        self.error(diagnostics::decorators_in_export_and_class(decorator.span));
                    }
                }
                decorators.extend(after_export_decorators);
                let modifiers = self.parse_modifiers(false, false);
                let class_decl = self.parse_class_declaration(class_span, &modifiers, decorators);
                let decl = Declaration::ClassDeclaration(class_decl);
                self.ast.module_declaration_export_named_declaration(
                    self.end_span(span),
                    Some(decl),
                    self.ast.vec(),
                    None,
                    ImportOrExportKind::Value,
                    NONE,
                )
            }
            Kind::Eq if self.is_ts => ModuleDeclaration::TSExportAssignment(
                self.parse_ts_export_assignment_declaration(span),
            ),
            Kind::As if self.is_ts && self.lexer.peek_token().kind() == Kind::Namespace => {
                // `export as namespace ...`
                ModuleDeclaration::TSNamespaceExportDeclaration(
                    self.parse_ts_export_namespace(span),
                )
            }
            Kind::Default => ModuleDeclaration::ExportDefaultDeclaration(
                self.parse_export_default_declaration(span, decorators),
            ),
            Kind::Star => {
                ModuleDeclaration::ExportAllDeclaration(self.parse_export_all_declaration(span))
            }
            Kind::LCurly => {
                ModuleDeclaration::ExportNamedDeclaration(self.parse_export_named_specifiers(span))
            }
            Kind::Type if self.is_ts => {
                let next_kind = self.lexer.peek_token().kind();

                match next_kind {
                    // `export type { ...`
                    Kind::LCurly => ModuleDeclaration::ExportNamedDeclaration(
                        self.parse_export_named_specifiers(span),
                    ),
                    // `export type * as ...`
                    Kind::Star => ModuleDeclaration::ExportAllDeclaration(
                        self.parse_export_all_declaration(span),
                    ),
                    _ => ModuleDeclaration::ExportNamedDeclaration(
                        self.parse_export_named_declaration(span, decorators),
                    ),
                }
            }
            _ => ModuleDeclaration::ExportNamedDeclaration(
                self.parse_export_named_declaration(span, decorators),
            ),
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
        let (mut specifiers, _) = self.context(Context::empty(), self.ctx, |p| {
            p.parse_delimited_list(Kind::RCurly, Kind::Comma, |parser| {
                parser.parse_export_specifier(export_kind)
            })
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
    fn parse_export_named_declaration(
        &mut self,
        span: u32,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Box<'a, ExportNamedDeclaration<'a>> {
        let decl_span = self.start_span();
        let reserved_ctx = self.ctx;
        let modifiers =
            if self.is_ts { self.eat_modifiers_before_declaration() } else { Modifiers::empty() };
        self.ctx = self.ctx.union_ambient_if(modifiers.contains_declare());

        let declaration = self.parse_declaration(decl_span, &modifiers, decorators);
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
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Box<'a, ExportDefaultDeclaration<'a>> {
        let exported = self.parse_keyword_identifier(Kind::Default);
        let declaration = self.parse_export_default_declaration_kind(decorators);
        let exported = ModuleExportName::IdentifierName(exported);
        let span = self.end_span(span);
        self.ast.alloc_export_default_declaration(span, exported, declaration)
    }

    fn parse_export_default_declaration_kind(
        &mut self,
        mut decorators: Vec<'a, Decorator<'a>>,
    ) -> ExportDefaultDeclarationKind<'a> {
        let decl_span = self.start_span();

        // export default /* @__NO_SIDE_EFFECTS__ */ ...
        let has_no_side_effects_comment =
            self.lexer.trivia_builder.previous_token_has_no_side_effects_comment();

        // export default @decorator ...
        if self.at(Kind::At) {
            let after_export_decorators = self.parse_decorators();
            // @decorator export default @decorator ...
            if !decorators.is_empty() {
                for decorator in &after_export_decorators {
                    self.error(diagnostics::decorators_in_export_and_class(decorator.span));
                }
            }
            decorators.extend(after_export_decorators);
        }

        let function_span = self.start_span();

        let checkpoint = self.checkpoint();
        let mut is_abstract = false;
        let mut is_async = false;
        let mut is_interface = false;

        match self.cur_kind() {
            Kind::Abstract => is_abstract = true,
            Kind::Async => is_async = true,
            Kind::Interface => is_interface = true,
            _ => {}
        }

        if is_abstract || is_async || is_interface {
            let modifier_span = self.start_span();
            self.bump_any();
            let cur_token = self.cur_token();
            let kind = cur_token.kind();
            if !cur_token.is_on_new_line() {
                // export default abstract class ...
                if is_abstract && kind == Kind::Class {
                    let modifiers = self
                        .ast
                        .vec1(Modifier::new(self.end_span(modifier_span), ModifierKind::Abstract));
                    let modifiers = Modifiers::new(Some(modifiers), ModifierFlags::ABSTRACT);
                    return ExportDefaultDeclarationKind::ClassDeclaration(
                        self.parse_class_declaration(decl_span, &modifiers, decorators),
                    );
                }

                // export default async function ...
                if is_async && kind == Kind::Function {
                    for decorator in &decorators {
                        self.error(diagnostics::decorators_are_not_valid_here(decorator.span));
                    }
                    let mut func = self.parse_function_impl(
                        function_span,
                        /* r#async */ true,
                        FunctionKind::DefaultExport,
                    );
                    if has_no_side_effects_comment {
                        func.pure = true;
                    }
                    return ExportDefaultDeclarationKind::FunctionDeclaration(func);
                }

                // export default interface ...
                if is_interface {
                    for decorator in &decorators {
                        self.error(diagnostics::decorators_are_not_valid_here(decorator.span));
                    }
                    if let Declaration::TSInterfaceDeclaration(decl) =
                        self.parse_ts_interface_declaration(modifier_span, &Modifiers::empty())
                    {
                        return ExportDefaultDeclarationKind::TSInterfaceDeclaration(decl);
                    }
                }
            }
            self.rewind(checkpoint);
        }

        let kind = self.cur_kind();
        // export default class ...
        if kind == Kind::Class {
            return ExportDefaultDeclarationKind::ClassDeclaration(self.parse_class_declaration(
                decl_span,
                &Modifiers::empty(),
                decorators,
            ));
        }

        for decorator in &decorators {
            self.error(diagnostics::decorators_are_not_valid_here(decorator.span));
        }

        // export default function ...
        if kind == Kind::Function {
            let mut func = self.parse_function_impl(
                function_span,
                /* r#async */ false,
                FunctionKind::DefaultExport,
            );
            if has_no_side_effects_comment {
                func.pure = true;
            }
            return ExportDefaultDeclarationKind::FunctionDeclaration(func);
        }

        // export default expr
        let decl = ExportDefaultDeclarationKind::from(self.parse_assignment_expression_or_higher());
        self.asi();
        decl
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
        match self.parse_import_or_export_specifier(ImportOrExport::Import, parent_import_kind) {
            ImportOrExportSpecifier::Import(specifier) => {
                self.ast.import_declaration_specifier_import_specifier(
                    specifier.span,
                    specifier.imported,
                    specifier.local,
                    specifier.import_kind,
                )
            }
            ImportOrExportSpecifier::Export(_) => unreachable!(),
        }
    }

    fn parse_import_or_export_specifier(
        &mut self,
        specifier_type: ImportOrExport,
        parent_kind: ImportOrExportKind,
    ) -> ImportOrExportSpecifier<'a> {
        let specifier_span = self.start_span();
        let type_or_name_token = self.cur_token();
        let type_or_name_token_kind = type_or_name_token.kind();
        let mut check_identifier_token = self.cur_token();

        let mut kind = ImportOrExportKind::Value;
        let mut can_parse_as_keyword = true;
        let mut property_name: Option<ModuleExportName<'a>> = None;
        let mut name = self.parse_module_export_name();

        if self.is_ts && name.is_identifier() && type_or_name_token_kind == Kind::Type {
            // If the first token of an import/export specifier is 'type', there are a lot of possibilities,
            // especially if we see 'as' afterwards:
            //
            // import { type } from "mod";          - isTypeOnly: false,   name: type
            // import { type as } from "mod";       - isTypeOnly: true,    name: as
            // import { type as as } from "mod";    - isTypeOnly: false,   name: as,    propertyName: type
            // import { type as as as } from "mod"; - isTypeOnly: true,    name: as,    propertyName: as
            if self.at(Kind::As) {
                // { type as ...? }
                let first_as = self.parse_identifier_name();
                if self.at(Kind::As) {
                    // { type as as ...? }
                    let second_as = self.parse_identifier_name();
                    if self.can_parse_module_export_name() {
                        // { type as as something }
                        // { type as as "something" }
                        kind = ImportOrExportKind::Type;
                        property_name = Some(
                            self.ast
                                .module_export_name_identifier_name(second_as.span, second_as.name),
                        );
                        check_identifier_token = self.cur_token();
                        name = self.parse_module_export_name();
                        can_parse_as_keyword = false;
                    } else {
                        // { type as as }
                        property_name = Some(self.ast.module_export_name_identifier_name(
                            type_or_name_token.span(),
                            self.token_source(&type_or_name_token),
                        ));
                        name = self
                            .ast
                            .module_export_name_identifier_name(second_as.span, second_as.name);
                        can_parse_as_keyword = false;
                    }
                } else if self.can_parse_module_export_name() {
                    // { type as something }
                    // { type as "something" }
                    property_name = Some(name);
                    can_parse_as_keyword = false;
                    check_identifier_token = self.cur_token();
                    name = self.parse_module_export_name();
                } else {
                    // { type as }
                    kind = ImportOrExportKind::Type;
                    name =
                        self.ast.module_export_name_identifier_name(first_as.span, first_as.name);
                }
            } else if self.can_parse_module_export_name() {
                // { type something ...? }
                // { type "something" ...? }
                kind = ImportOrExportKind::Type;
                check_identifier_token = self.cur_token();
                name = self.parse_module_export_name();
            }
        }

        if can_parse_as_keyword && self.eat(Kind::As) {
            property_name = Some(name);
            check_identifier_token = self.cur_token();
            name = self.parse_module_export_name();
        }

        if self.is_ts && type_or_name_token_kind == Kind::Type && type_or_name_token.escaped() {
            self.error(diagnostics::escaped_keyword(type_or_name_token.span()));
        }

        match specifier_type {
            ImportOrExport::Import => {
                // `import type { type } from 'mod';`
                if parent_kind == ImportOrExportKind::Type && kind == ImportOrExportKind::Type {
                    self.error(diagnostics::type_modifier_on_named_type_import(
                        type_or_name_token.span(),
                    ));
                }

                if !name.is_identifier() {
                    self.error(diagnostics::identifier_expected(name.span()));
                } else if check_identifier_token.kind().is_reserved_keyword() {
                    self.error(diagnostics::identifier_reserved_word(
                        check_identifier_token.span(),
                        check_identifier_token.kind().to_str(),
                    ));
                }

                ImportOrExportSpecifier::Import(self.ast.import_specifier(
                    self.end_span(specifier_span),
                    property_name.unwrap_or_else(|| name.clone()),
                    self.ast.binding_identifier(name.span(), name.name()),
                    kind,
                ))
            }
            ImportOrExport::Export => {
                // `export type { type } from 'mod';`
                if parent_kind == ImportOrExportKind::Type && kind == ImportOrExportKind::Type {
                    self.error(diagnostics::type_modifier_on_named_type_export(
                        type_or_name_token.span(),
                    ));
                }

                let exported = match property_name {
                    Some(property_name) => property_name,
                    None => name.clone(),
                };
                ImportOrExportSpecifier::Export(self.ast.export_specifier(
                    self.end_span(specifier_span),
                    exported,
                    name,
                    kind,
                ))
            }
        }
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

        let next_kind = self.lexer.peek_token().kind();

        if matches!(next_kind, Kind::LCurly | Kind::Star) {
            self.bump_any();
            return ImportOrExportKind::Type;
        }

        if !(next_kind == Kind::Ident) && !next_kind.is_contextual_keyword() {
            return ImportOrExportKind::Value;
        }

        if next_kind != Kind::From
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

    fn parse_export_specifier(
        &mut self,
        parent_export_kind: ImportOrExportKind,
    ) -> ExportSpecifier<'a> {
        match self.parse_import_or_export_specifier(ImportOrExport::Export, parent_export_kind) {
            ImportOrExportSpecifier::Export(specifier) => specifier,
            ImportOrExportSpecifier::Import(_) => unreachable!(),
        }
    }

    fn can_parse_module_export_name(&self) -> bool {
        self.cur_kind().is_identifier_name() || self.at(Kind::Str)
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, ImportPhase, Statement};
    use oxc_span::SourceType;

    use crate::Parser;
    #[test]
    fn test_parse_import_declaration() {
        let src = "import foo from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "foo");
        });

        let src = "import type foo from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Type);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "foo");
        });

        let src = "import type from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "type");
        });

        let src = "import type from from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Type);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "from");
            assert!(matches!(specifiers[0], ImportDeclarationSpecifier::ImportDefaultSpecifier(_)));
        });

        let src = "import type type from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Type);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "type");
            assert!(matches!(specifiers[0], ImportDeclarationSpecifier::ImportDefaultSpecifier(_)));
        });

        let src = "import type { type } from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Type);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "type");
        });

        let src = "import './a'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert!(decl.specifiers.is_none());
        });

        let src = "import type, { bar } from 'foo';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 2);
            assert_eq!(specifiers[0].name(), "type");
            assert_eq!(specifiers[1].name(), "bar");
        });

        let src = "import { foo, bar } from 'baz';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert!(decl.specifiers.is_some());
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 2);
            assert_eq!(specifiers[0].name(), "foo");
            assert_eq!(specifiers[1].name(), "bar");
        });

        let src = "import type { foo, bar } from 'baz';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Type);
            assert!(decl.specifiers.is_some());
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 2);
            assert_eq!(specifiers[0].name(), "foo");
            assert_eq!(specifiers[1].name(), "bar");
        });

        let src = "import type { from, type } from 'baz';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Type);
            assert!(decl.specifiers.is_some());
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 2);
            assert_eq!(specifiers[0].name(), "from");
            assert_eq!(specifiers[1].name(), "type");
        });

        let src = "import defaultItem, { type from, type } from 'baz';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert!(decl.specifiers.is_some());
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 3);
            assert_eq!(specifiers[0].name(), "defaultItem");
            assert_eq!(specifiers[1].name(), "from");
            assert_eq!(specifiers[2].name(), "type");
        });

        let src = "import { type as as } from 'baz';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert!(decl.specifiers.is_some());
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = &specifiers[0] {
                assert_eq!(specifier.local.name, "as");
                assert_eq!(specifier.imported.name(), "type");
            } else {
                panic!("Expected ImportSpecifier, found: {:?}", specifiers[0]);
            }
        });

        let src = "import * as foo from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "foo");
        });

        let src = "import type * as foo from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Type);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "foo");
        });

        let src = "import defer * as ns from 'x'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert_eq!(decl.phase, Some(ImportPhase::Defer));
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "ns");
        });

        let src = "import source x from 'source'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert_eq!(decl.phase, Some(ImportPhase::Source));
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "x");
        });

        let src = "import source from 'source'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert_eq!(decl.phase, None);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "source");
        });

        let src = "import source from from 'source'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert_eq!(decl.phase, Some(ImportPhase::Source));
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "from");
        });

        let src = "import source as from 'source'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert_eq!(decl.phase, Some(ImportPhase::Source));
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "as");
        });

        let src = "import source source from 'source'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert_eq!(decl.phase, Some(ImportPhase::Source));
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "source");
        });

        let src = "import defer from 'source'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert_eq!(decl.phase, None);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "defer");
        });

        let src = "import type foo, { bar } from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Type);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 2);
            assert_eq!(specifiers[0].name(), "foo");
            assert_eq!(specifiers[1].name(), "bar");
        });

        let src = "import foo = bar";
        parse_and_assert_statements(src, |statements| {
            if let Statement::TSImportEqualsDeclaration(decl) = statements[0] {
                assert_eq!(decl.import_kind, ImportOrExportKind::Value);
                assert_eq!(decl.id.name, "foo");
            } else {
                panic!("Expected TSImportEqualsDeclaration, found: {:?}", statements[0]);
            }
        });

        let src = "import type foo = bar";
        parse_and_assert_statements(src, |statements| {
            if let Statement::TSImportEqualsDeclaration(decl) = statements[0] {
                assert_eq!(decl.import_kind, ImportOrExportKind::Type);
                assert_eq!(decl.id.name, "foo");
            } else {
                panic!("Expected TSImportEqualsDeclaration, found: {:?}", statements[0]);
            }
        });

        let src = "import type from = require('./a')";
        parse_and_assert_statements(src, |statements| {
            if let Statement::TSImportEqualsDeclaration(decl) = statements[0] {
                assert_eq!(decl.import_kind, ImportOrExportKind::Type);
                assert_eq!(decl.id.name, "from");
            } else {
                panic!("Expected TSImportEqualsDeclaration, found: {:?}", statements[0]);
            }
        });

        let src = "import from = b";
        parse_and_assert_statements(src, |statements| {
            if let Statement::TSImportEqualsDeclaration(decl) = statements[0] {
                assert_eq!(decl.import_kind, ImportOrExportKind::Value);
                assert_eq!(decl.id.name, "from");
            } else {
                panic!("Expected TSImportEqualsDeclaration, found: {:?}", statements[0]);
            }
        });
    }

    // https://github.com/oxc-project/oxc/issues/11505
    #[test]
    fn test_type_from_js_file() {
        let src = "import type from '../type.js'";
        let source_type = SourceType::default().with_typescript(false);
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, src, source_type).parse();
        assert!(ret.errors.is_empty(), "Failed to parse source: {src:?}, error: {:?}", ret.errors);
        let declarations = ret.program.body.iter().collect::<Vec<_>>();
        assert_eq!(declarations.len(), 1);
        let Statement::ImportDeclaration(decl) = declarations[0] else {
            panic!("Expected ImportDeclaration, found: {:?}", declarations[0]);
        };
        assert_eq!(decl.import_kind, ImportOrExportKind::Value);
        assert!(decl.specifiers.is_some());
        let specifiers = decl.specifiers.as_ref().unwrap();
        assert_eq!(specifiers[0].name(), "type");
    }

    fn parse_and_assert_statements(
        src: &'static str,
        // takes a function which accepts the list of statements
        f: fn(Vec<&oxc_ast::ast::Statement<'_>>) -> (),
    ) {
        let source_type = SourceType::default().with_typescript(true);
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, src, source_type).parse();
        assert!(ret.errors.is_empty(), "Failed to parse source: {src:?}, error: {:?}", ret.errors);
        f(ret.program.body.iter().collect::<Vec<_>>());
    }

    fn parse_and_assert_import_declarations(
        src: &'static str,
        // takes a function which accepts the list of statements
        f: fn(Vec<&oxc_allocator::Box<'_, oxc_ast::ast::ImportDeclaration<'_>>>) -> (),
    ) {
        let source_type = SourceType::default().with_typescript(true);
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, src, source_type).parse();
        assert!(ret.errors.is_empty(), "Failed to parse source: {src:?}, error: {:?}", ret.errors);
        let statements =
            ret.program
                .body
                .iter()
                .filter_map(|s| {
                    if let Statement::ImportDeclaration(decl) = s { Some(decl) } else { None }
                })
                .collect::<Vec<_>>();
        f(statements);
    }
}
