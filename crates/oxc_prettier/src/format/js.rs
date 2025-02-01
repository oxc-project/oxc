use std::borrow::Cow;

use cow_utils::CowUtils;
use oxc_allocator::{Box, Vec};
use oxc_ast::{ast::*, AstKind};
use oxc_span::GetSpan;
use oxc_syntax::identifier::{is_identifier_name, is_line_terminator};

use crate::{
    array, dynamic_text,
    format::{
        print::{
            array, arrow_function, assignment, binaryish, block, call_expression, class, function,
            function_parameters, literal, member, misc, module, object, property, statement,
            template_literal, ternary,
        },
        Format,
    },
    group, hardline, indent,
    ir::{Doc, JoinSeparator},
    join, line, softline, text, utils, wrap, Prettier,
};

impl<'a> Format<'a> for Program<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, Program, {
            let mut parts = Vec::new_in(p.allocator);

            // In Prettier, this is treated as a comment
            if let Some(hashbang) = &self.hashbang {
                parts.push(hashbang.format(p));
            }

            if let Some(body_doc) = block::print_block_body(p, &self.body, Some(&self.directives)) {
                parts.push(body_doc);
                parts.push(hardline!(p));
            }

            array!(p, parts)
        })
    }
}

impl<'a> Format<'a> for Hashbang<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        parts.push(dynamic_text!(p, self.span.source_text(p.source_text)));
        parts.push(hardline!(p));
        // Preserve original newline
        if p.is_next_line_empty(self.span) {
            parts.push(hardline!(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for Directive<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        let not_quoted_raw_text = &self.directive.as_str();
        // If quote is used, don't replace enclosing quotes, keep as is
        if not_quoted_raw_text.contains('"') || not_quoted_raw_text.contains('\'') {
            parts.push(dynamic_text!(p, &self.span.source_text(p.source_text)));
        } else {
            let enclosing_quote = || text!(if p.options.single_quote { "'" } else { "\"" });
            parts.push(enclosing_quote());
            parts.push(dynamic_text!(p, &not_quoted_raw_text));
            parts.push(enclosing_quote());
        }
        if let Some(semi) = p.semi() {
            parts.push(semi);
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for Statement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::BlockStatement(stmt) => stmt.format(p),
            Self::BreakStatement(stmt) => stmt.format(p),
            Self::ContinueStatement(stmt) => stmt.format(p),
            Self::DebuggerStatement(stmt) => stmt.format(p),
            Self::DoWhileStatement(stmt) => stmt.format(p),
            Self::EmptyStatement(stmt) => stmt.format(p),
            Self::ExpressionStatement(stmt) => stmt.format(p),
            Self::ForInStatement(stmt) => stmt.format(p),
            Self::ForOfStatement(stmt) => stmt.format(p),
            Self::ForStatement(stmt) => stmt.format(p),
            Self::IfStatement(stmt) => stmt.format(p),
            Self::LabeledStatement(stmt) => stmt.format(p),
            Self::ReturnStatement(stmt) => stmt.format(p),
            Self::SwitchStatement(stmt) => stmt.format(p),
            Self::ThrowStatement(stmt) => stmt.format(p),
            Self::TryStatement(stmt) => stmt.format(p),
            Self::WhileStatement(stmt) => stmt.format(p),
            Self::WithStatement(stmt) => stmt.format(p),
            match_module_declaration!(Self) => self.to_module_declaration().format(p),
            match_declaration!(Self) => self.to_declaration().format(p),
        }
    }
}

impl<'a> Format<'a> for ExpressionStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ExpressionStatement, {
            let mut parts = Vec::new_in(p.allocator);
            parts.push(self.expression.format(p));
            if let Some(semi) = p.semi() {
                parts.push(semi);
            }
            array!(p, parts)
        })
    }
}

impl<'a> Format<'a> for EmptyStatement {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("")
    }
}

impl<'a> Format<'a> for IfStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, IfStatement, {
            let mut parts = Vec::new_in(p.allocator);

            let consequent_doc = self.consequent.format(p);
            parts.push(group!(
                p,
                [
                    text!("if ("),
                    group!(p, [indent!(p, [softline!(), self.test.format(p)]), softline!()]),
                    text!(")"),
                    misc::adjust_clause(p, &self.consequent, consequent_doc, false)
                ]
            ));

            if let Some(alternate) = &self.alternate {
                let else_on_same_line = matches!(alternate, Statement::BlockStatement(_));
                parts.push(if else_on_same_line { text!(" ") } else { hardline!(p) });

                parts.push(text!("else"));

                let alternate_doc = alternate.format(p);
                parts.push(group!(
                    p,
                    [misc::adjust_clause(
                        p,
                        alternate,
                        alternate_doc,
                        matches!(alternate, Statement::IfStatement(_)),
                    )]
                ));
            }

            array!(p, parts)
        })
    }
}

impl<'a> Format<'a> for BlockStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, BlockStatement, { block::print_block(p, &self.body, None) })
    }
}

impl<'a> Format<'a> for ForStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ForStatement, {
            let body_doc = self.body.format(p);
            let body_doc = misc::adjust_clause(p, &self.body, body_doc, false);

            if self.init.is_none() && self.test.is_none() && self.update.is_none() {
                return group!(p, [text!("for (;;)"), body_doc]);
            }

            let mut init_test_update_parts = Vec::new_in(p.allocator);
            init_test_update_parts.push(softline!());
            if let Some(init) = &self.init {
                init_test_update_parts.push(match init {
                    ForStatementInit::VariableDeclaration(v) => v.format(p),
                    match_expression!(ForStatementInit) => init.to_expression().format(p),
                });
            }
            init_test_update_parts.push(text!(";"));
            init_test_update_parts.push(line!());
            if let Some(init) = &self.test {
                init_test_update_parts.push(init.format(p));
            }
            init_test_update_parts.push(text!(";"));
            init_test_update_parts.push(line!());
            if let Some(init) = &self.update {
                init_test_update_parts.push(init.format(p));
            }

            group!(
                p,
                [
                    text!("for ("),
                    group!(p, [indent!(p, init_test_update_parts), softline!()]),
                    text!(")"),
                    body_doc
                ]
            )
        })
    }
}

impl<'a> Format<'a> for ForInStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ForInStatement, {
            let mut parts = Vec::new_in(p.allocator);
            parts.push(text!("for ("));
            parts.push(self.left.format(p));
            parts.push(text!(" in "));
            parts.push(self.right.format(p));
            parts.push(text!(")"));

            let body_doc = self.body.format(p);
            parts.push(misc::adjust_clause(p, &self.body, body_doc, false));

            group!(p, parts)
        })
    }
}

impl<'a> Format<'a> for ForOfStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ForOfStatement, {
            let mut parts = Vec::new_in(p.allocator);
            parts.push(text!("for"));
            if self.r#await {
                parts.push(text!(" await"));
            }
            parts.push(text!(" ("));
            parts.push(self.left.format(p));
            parts.push(text!(" of "));
            parts.push(self.right.format(p));
            parts.push(text!(")"));

            let body_doc = self.body.format(p);
            parts.push(misc::adjust_clause(p, &self.body, body_doc, false));

            group!(p, parts)
        })
    }
}

impl<'a> Format<'a> for ForStatementLeft<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            ForStatementLeft::VariableDeclaration(v) => v.format(p),
            match_assignment_target!(ForStatementLeft) => self.to_assignment_target().format(p),
        }
    }
}

impl<'a> Format<'a> for WhileStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, WhileStatement, {
            let mut parts = Vec::new_in(p.allocator);
            parts.push(text!("while ("));
            parts.push(group!(p, [indent!(p, [softline!(), self.test.format(p)]), softline!()]));
            parts.push(text!(")"));

            let body_doc = self.body.format(p);
            parts.push(misc::adjust_clause(p, &self.body, body_doc, false));

            group!(p, parts)
        })
    }
}

impl<'a> Format<'a> for DoWhileStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, DoWhileStatement, {
            let mut parts = Vec::new_in(p.allocator);

            let clause_doc = self.body.format(p);
            let clause = misc::adjust_clause(p, &self.body, clause_doc, false);
            let do_body = group!(p, [text!("do"), clause]);
            parts.push(do_body);

            if matches!(self.body, Statement::BlockStatement(_)) {
                parts.push(text!(" "));
            } else {
                parts.push(hardline!(p));
            }

            parts.push(text!("while ("));
            parts.push(group!(p, [indent!(p, [softline!(), self.test.format(p)]), softline!()]));
            parts.push(text!(")"));

            if let Some(semi) = p.semi() {
                parts.push(semi);
            }

            array!(p, parts)
        })
    }
}

impl<'a> Format<'a> for ContinueStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        parts.push(text!("continue"));

        if let Some(label) = &self.label {
            parts.push(text!(" "));
            parts.push(label.format(p));
        }

        if p.options.semi {
            parts.push(text!(";"));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for BreakStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        parts.push(text!("break"));

        if let Some(label) = &self.label {
            parts.push(text!(" "));
            parts.push(label.format(p));
        }

        if p.options.semi {
            parts.push(text!(";"));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for SwitchStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, SwitchStatement, {
            let mut parts = Vec::new_in(p.allocator);

            parts.push(group!(
                p,
                [
                    text!("switch ("),
                    indent!(p, [softline!(), self.discriminant.format(p)]),
                    softline!(),
                    text!(")"),
                ]
            ));

            parts.push(text!(" {"));

            let mut cases_parts = Vec::new_in(p.allocator);
            let len = self.cases.len();
            for (i, case) in self.cases.iter().enumerate() {
                cases_parts.push(indent!(p, [hardline!(p), case.format(p)]));
                if i != len - 1 && p.is_next_line_empty(case.span) {
                    cases_parts.push(hardline!(p));
                }
            }
            parts.extend(cases_parts);

            parts.push(hardline!(p));
            parts.push(text!("}"));

            array!(p, parts)
        })
    }
}

impl<'a> Format<'a> for SwitchCase<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        if let Some(test) = &self.test {
            parts.push(text!("case "));
            parts.push(test.format(p));
            parts.push(text!(":"));
        } else {
            parts.push(text!("default:"));
        }

        let len =
            self.consequent.iter().filter(|c| !matches!(c, Statement::EmptyStatement(_))).count();
        if len != 0 {
            let consequent_parts =
                statement::print_statement_sequence(p, self.consequent.as_slice());

            if len == 1 && matches!(self.consequent[0], Statement::BlockStatement(_)) {
                parts.push(array!(p, [text!(" "), array!(p, consequent_parts)]));
            } else {
                parts.push(indent!(p, [hardline!(p), array!(p, consequent_parts)]));
            }
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for ReturnStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ReturnStatement, {
            array!(
                p,
                [
                    text!("return"),
                    function::print_return_or_throw_argument(p, self.argument.as_ref())
                ]
            )
        })
    }
}

impl<'a> Format<'a> for LabeledStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        if matches!(self.body, Statement::EmptyStatement(_)) {
            return array!(p, [self.label.format(p), text!(":;")]);
        }

        array!(p, [self.label.format(p), text!(": "), self.body.format(p)])
    }
}

impl<'a> Format<'a> for TryStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, TryStatement, {
            let mut parts = Vec::new_in(p.allocator);

            parts.push(text!("try "));
            parts.push(self.block.format(p));
            if let Some(handler) = &self.handler {
                parts.push(text!(" "));
                parts.push(handler.format(p));
            }
            if let Some(finalizer) = &self.finalizer {
                parts.push(text!(" finally "));
                parts.push(finalizer.format(p));
            }

            array!(p, parts)
        })
    }
}

impl<'a> Format<'a> for CatchClause<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, CatchClause, {
            if let Some(param) = &self.param {
                return array!(
                    p,
                    [text!("catch ("), param.pattern.format(p), text!(") "), self.body.format(p)]
                );
            }

            array!(p, [text!("catch "), self.body.format(p)])
        })
    }
}

impl<'a> Format<'a> for ThrowStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ThrowStatement, {
            array!(
                p,
                [text!("throw"), function::print_return_or_throw_argument(p, Some(&self.argument))]
            )
        })
    }
}

impl<'a> Format<'a> for WithStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let body_doc = self.body.format(p);
        group!(
            p,
            [
                text!("with ("),
                self.object.format(p),
                text!(")"),
                misc::adjust_clause(p, &self.body, body_doc, false)
            ]
        )
    }
}

impl<'a> Format<'a> for DebuggerStatement {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        parts.push(text!("debugger"));

        if p.options.semi {
            parts.push(text!(";"));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for ModuleDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            ModuleDeclaration::ImportDeclaration(import) => import.format(p),
            ModuleDeclaration::ExportDefaultDeclaration(export) => export.format(p),
            ModuleDeclaration::ExportNamedDeclaration(export) => export.format(p),
            ModuleDeclaration::ExportAllDeclaration(export) => export.format(p),
            ModuleDeclaration::TSExportAssignment(export) => export.format(p),
            ModuleDeclaration::TSNamespaceExportDeclaration(export) => export.format(p),
        }
    }
}

impl<'a> Format<'a> for Declaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::VariableDeclaration(stmt) => stmt.format(p),
            Self::FunctionDeclaration(stmt) => stmt.format(p),
            Self::ClassDeclaration(decl) => decl.format(p),
            Self::TSTypeAliasDeclaration(decl) => decl.format(p),
            Self::TSInterfaceDeclaration(decl) => decl.format(p),
            Self::TSEnumDeclaration(decl) => decl.format(p),
            Self::TSModuleDeclaration(decl) => decl.format(p),
            Self::TSImportEqualsDeclaration(decl) => decl.format(p),
        }
    }
}

impl<'a> Format<'a> for VariableDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, VariableDeclaration, {
            // We generally want to terminate all variable declarations with a
            // semicolon, except when they in the () part of for loops.
            let parent_for_loop = match p.parent_kind() {
                AstKind::ForStatement(stmt) => Some(stmt.body.span()),
                AstKind::ForInStatement(stmt) => Some(stmt.body.span()),
                AstKind::ForOfStatement(stmt) => Some(stmt.body.span()),
                _ => None,
            };

            let kind = self.kind.as_str();

            let mut parts = Vec::new_in(p.allocator);

            if self.declare {
                parts.push(text!("declare "));
            }

            parts.push(text!(kind));
            parts.push(text!(" "));

            let is_hardline = !p.parent_kind().is_iteration_statement()
                && self.declarations.iter().all(|decl| decl.init.is_some());
            let decls_len = self.declarations.len();
            parts.extend(self.declarations.iter().enumerate().map(|(i, decl)| {
                if decls_len > 1 {
                    let mut d_parts = Vec::new_in(p.allocator);
                    if i != 0 {
                        d_parts.push(text!(","));
                        d_parts.push(if is_hardline { hardline!(p) } else { line!() });
                    }
                    d_parts.push(decl.format(p));
                    indent!(p, d_parts)
                } else {
                    decl.format(p)
                }
            }));

            if !parent_for_loop.is_some_and(|span| span != self.span) {
                if let Some(semi) = p.semi() {
                    parts.push(semi);
                }
            }

            group!(p, parts)
        })
    }
}

impl<'a> Format<'a> for VariableDeclarator<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, VariableDeclarator, {
            let left_doc = self.id.format(p);
            assignment::print_assignment(
                p,
                assignment::AssignmentLike::VariableDeclarator(self),
                left_doc,
                text!(" ="),
                self.init.as_ref(),
            )
        })
    }
}

impl<'a> Format<'a> for Function<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, Function, { function::print_function(p, self, None) })
    }
}

impl<'a> Format<'a> for FunctionBody<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, FunctionBody, {
            block::print_block(p, &self.statements, Some(&self.directives))
        })
    }
}

impl<'a> Format<'a> for FormalParameters<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, FormalParameters, {
            function_parameters::print_function_parameters(p, self)
        })
    }
}

impl<'a> Format<'a> for FormalParameter<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, FormalParameter, { self.pattern.format(p) })
    }
}

impl<'a> Format<'a> for ImportDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ImportDeclaration, { module::print_import_declaration(p, self) })
    }
}

impl<'a> Format<'a> for ImportDeclarationSpecifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::ImportSpecifier(specifier) => specifier.format(p),
            Self::ImportDefaultSpecifier(specifier) => specifier.format(p),
            Self::ImportNamespaceSpecifier(specifier) => specifier.format(p),
        }
    }
}

impl<'a> Format<'a> for ImportSpecifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        if self.import_kind.is_type() {
            parts.push(text!("type "));
        }

        // If both imported and local are the same name
        if self.imported.span() == self.local.span {
            parts.push(self.local.format(p));
            return array!(p, parts);
        }

        parts.push(self.imported.format(p));
        parts.push(text!(" as "));
        parts.push(self.local.format(p));
        array!(p, parts)
    }
}

impl<'a> Format<'a> for ImportDefaultSpecifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        self.local.format(p)
    }
}

impl<'a> Format<'a> for ImportNamespaceSpecifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array!(p, [text!("* as "), self.local.format(p)])
    }
}

impl<'a> Format<'a> for ImportAttribute<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let left_doc = property::print_property_key(
            p,
            &property::PropertyKeyLike::ImportAttributeKey(&self.key),
            false, // Can not be computed
        );

        assignment::print_assignment(
            p,
            assignment::AssignmentLike::ImportAttribute(self),
            left_doc,
            text!(":"),
            // PERF: Can be better without clone...?
            Some(&Expression::StringLiteral(Box::new_in(self.value.clone(), p.allocator))),
        )
    }
}

impl<'a> Format<'a> for ExportSpecifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        // If both exported and local are the same name
        if self.exported.span() == self.local.span() {
            return self.local.format(p);
        }

        array!(p, [self.local.format(p), text!(" as "), self.exported.format(p)])
    }
}

impl<'a> Format<'a> for ModuleExportName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::IdentifierName(ident) => ident.format(p),
            Self::IdentifierReference(ident) => ident.format(p),
            Self::StringLiteral(literal) => literal.format(p),
        }
    }
}

impl<'a> Format<'a> for ExportAllDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ExportAllDeclaration, {
            module::print_export_declaration(
                p,
                &module::ExportDeclarationLike::ExportAllDeclaration(self),
            )
        })
    }
}

impl<'a> Format<'a> for ExportDefaultDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ExportDefaultDeclaration, {
            module::print_export_declaration(
                p,
                &module::ExportDeclarationLike::ExportDefaultDeclaration(self),
            )
        })
    }
}

impl<'a> Format<'a> for ExportNamedDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ExportNamedDeclaration, {
            module::print_export_declaration(
                p,
                &module::ExportDeclarationLike::ExportNamedDeclaration(self),
            )
        })
    }
}

impl<'a> Format<'a> for Expression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::BooleanLiteral(lit) => lit.format(p),
            Self::NullLiteral(lit) => lit.format(p),
            Self::NumericLiteral(lit) => lit.format(p),
            Self::BigIntLiteral(lit) => lit.format(p),
            Self::RegExpLiteral(lit) => lit.format(p),
            Self::StringLiteral(lit) => lit.format(p),
            Self::Identifier(ident) => ident.format(p),
            Self::ThisExpression(expr) => expr.format(p),
            match_member_expression!(Self) => self.to_member_expression().format(p),
            Self::CallExpression(expr) => expr.format(p),
            Self::ArrayExpression(expr) => expr.format(p),
            Self::ObjectExpression(expr) => expr.format(p),
            Self::FunctionExpression(expr) => expr.format(p),
            Self::ArrowFunctionExpression(expr) => expr.format(p),
            Self::YieldExpression(expr) => expr.format(p),
            Self::UpdateExpression(expr) => expr.format(p),
            Self::UnaryExpression(expr) => expr.format(p),
            Self::BinaryExpression(expr) => expr.format(p),
            Self::PrivateInExpression(expr) => expr.format(p),
            Self::LogicalExpression(expr) => expr.format(p),
            Self::ConditionalExpression(expr) => expr.format(p),
            Self::AssignmentExpression(expr) => expr.format(p),
            Self::SequenceExpression(expr) => expr.format(p),
            Self::ParenthesizedExpression(expr) => expr.format(p),
            Self::ImportExpression(expr) => expr.format(p),
            Self::TemplateLiteral(literal) => literal.format(p),
            Self::TaggedTemplateExpression(expr) => expr.format(p),
            Self::Super(sup) => sup.format(p),
            Self::AwaitExpression(expr) => expr.format(p),
            Self::ChainExpression(expr) => expr.format(p),
            Self::NewExpression(expr) => expr.format(p),
            Self::MetaProperty(expr) => expr.format(p),
            Self::ClassExpression(expr) => expr.format(p),
            Self::JSXElement(el) => el.format(p),
            Self::JSXFragment(fragment) => fragment.format(p),
            Self::TSAsExpression(expr) => expr.format(p),
            Self::TSSatisfiesExpression(expr) => expr.format(p),
            Self::TSTypeAssertion(expr) => expr.format(p),
            Self::TSNonNullExpression(expr) => expr.format(p),
            Self::TSInstantiationExpression(expr) => expr.format(p),
        }
    }
}

impl<'a> Format<'a> for IdentifierReference<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, IdentifierReference, { dynamic_text!(p, self.name.as_str()) })
    }
}

impl<'a> Format<'a> for IdentifierName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        dynamic_text!(p, self.name.as_str())
    }
}

impl<'a> Format<'a> for BindingIdentifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, BindingIdentifier, { dynamic_text!(p, self.name.as_str()) })
    }
}

impl<'a> Format<'a> for LabelIdentifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        dynamic_text!(p, self.name.as_str())
    }
}

impl<'a> Format<'a> for BooleanLiteral {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!(if self.value { "true" } else { "false" })
    }
}

impl<'a> Format<'a> for NullLiteral {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("null")
    }
}

impl<'a> Format<'a> for NumericLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        literal::print_number(p, self.span.source_text(p.source_text))
    }
}

impl<'a> Format<'a> for BigIntLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self.span.source_text(p.source_text).cow_to_ascii_lowercase() {
            Cow::Borrowed(s) => dynamic_text!(p, s),
            Cow::Owned(s) => dynamic_text!(p, &s),
        }
    }
}

impl<'a> Format<'a> for RegExpLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        dynamic_text!(p, &self.regex.to_string())
    }
}

impl<'a> Format<'a> for StringLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        utils::replace_end_of_line(
            p,
            literal::print_string(p, self.span.source_text(p.source_text), p.options.single_quote),
            JoinSeparator::Literalline,
        )
    }
}

impl<'a> Format<'a> for ThisExpression {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("this")
    }
}

impl<'a> Format<'a> for MemberExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        // This `wrap!` should be used for each type, but they are not listed in the `AstKind`
        wrap!(p, self, MemberExpression, {
            match self {
                Self::ComputedMemberExpression(expr) => expr.format(p),
                Self::StaticMemberExpression(expr) => expr.format(p),
                Self::PrivateFieldExpression(expr) => expr.format(p),
            }
        })
    }
}

impl<'a> Format<'a> for ComputedMemberExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        member::print_member_expression(
            p,
            &member::MemberExpressionLike::ComputedMemberExpression(self),
        )
    }
}

impl<'a> Format<'a> for StaticMemberExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        member::print_member_expression(
            p,
            &member::MemberExpressionLike::StaticMemberExpression(self),
        )
    }
}

impl<'a> Format<'a> for PrivateFieldExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        member::print_member_expression(
            p,
            &member::MemberExpressionLike::PrivateFieldExpression(self),
        )
    }
}

impl<'a> Format<'a> for CallExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, CallExpression, {
            call_expression::print_call_expression(
                p,
                &call_expression::CallExpressionLike::CallExpression(self),
            )
        })
    }
}

impl<'a> Format<'a> for Argument<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            match_expression!(Self) => self.to_expression().format(p),
            Self::SpreadElement(expr) => expr.format(p),
        }
    }
}

impl<'a> Format<'a> for ArrayExpressionElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::SpreadElement(expr) => expr.format(p),
            match_expression!(Self) => self.to_expression().format(p),
            Self::Elision(elision) => text!(""),
        }
    }
}

impl<'a> Format<'a> for SpreadElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, SpreadElement, { array!(p, [text!("..."), self.argument.format(p)]) })
    }
}

impl<'a> Format<'a> for ArrayExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ArrayExpression, {
            array::print_array(p, &array::ArrayLike::ArrayExpression(self))
        })
    }
}

impl<'a> Format<'a> for ObjectExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ObjectExpression, {
            object::print_object(p, object::ObjectLike::Expression(self))
        })
    }
}

impl<'a> Format<'a> for ObjectPropertyKind<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            ObjectPropertyKind::ObjectProperty(prop) => prop.format(p),
            ObjectPropertyKind::SpreadProperty(prop) => prop.format(p),
        }
    }
}

impl<'a> Format<'a> for ObjectProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ObjectProperty, {
            if self.method || self.kind == PropertyKind::Get || self.kind == PropertyKind::Set {
                let mut parts = Vec::new_in(p.allocator);
                match self.kind {
                    PropertyKind::Get => {
                        parts.push(text!("get "));
                    }
                    PropertyKind::Set => {
                        parts.push(text!("set "));
                    }
                    PropertyKind::Init => (),
                }
                if let Expression::FunctionExpression(func_expr) = &self.value {
                    parts.push(wrap!(p, func_expr, Function, {
                        function::print_function(
                            p,
                            func_expr,
                            Some(self.key.span().source_text(p.source_text)),
                        )
                    }));
                }
                return group!(p, parts);
            }

            if self.shorthand {
                return self.value.format(p);
            }

            let left_doc = property::print_property_key(
                p,
                &property::PropertyKeyLike::PropertyKey(&self.key),
                self.computed,
            );

            assignment::print_assignment(
                p,
                assignment::AssignmentLike::ObjectProperty(self),
                left_doc,
                text!(":"),
                Some(&self.value),
            )
        })
    }
}

impl<'a> Format<'a> for PropertyKey<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            PropertyKey::StaticIdentifier(ident) => ident.format(p),
            PropertyKey::PrivateIdentifier(ident) => ident.format(p),
            match_expression!(PropertyKey) => self.to_expression().format(p),
        }
    }
}

impl<'a> Format<'a> for ArrowFunctionExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ArrowFunctionExpression, { arrow_function::print_arrow_function(p, self) })
    }
}

impl<'a> Format<'a> for YieldExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, YieldExpression, {
            let mut parts = Vec::new_in(p.allocator);
            parts.push(text!("yield"));
            if self.delegate {
                parts.push(text!("*"));
            }
            if let Some(argument) = &self.argument {
                parts.push(text!(" "));
                parts.push(argument.format(p));
            }
            array!(p, parts)
        })
    }
}

impl<'a> Format<'a> for UpdateExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, UpdateExpression, {
            let argument_doc = self.argument.format(p);
            if self.prefix {
                array!(p, [text!(self.operator.as_str()), argument_doc])
            } else {
                array!(p, [argument_doc, text!(self.operator.as_str())])
            }
        })
    }
}

impl<'a> Format<'a> for UnaryExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, UnaryExpression, {
            let mut parts = Vec::new_in(p.allocator);
            parts.push(dynamic_text!(p, self.operator.as_str()));
            if self.operator.is_keyword() {
                parts.push(text!(" "));
            }
            parts.push(self.argument.format(p));
            array!(p, parts)
        })
    }
}

impl<'a> Format<'a> for BinaryExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, BinaryExpression, {
            let doc = binaryish::print_binaryish_expression(
                p,
                &self.left,
                self.operator.into(),
                &self.right,
            );
            if misc::in_parentheses(p.parent_kind(), p.source_text, self.span) {
                group!(p, [indent!(p, [softline!(), doc]), softline!()])
            } else {
                doc
            }
        })
    }
}

impl<'a> Format<'a> for PrivateInExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, PrivateInExpression, {
            let left_doc = self.left.format(p);
            let right_doc = self.right.format(p);
            array!(p, [left_doc, text!(" "), text!(self.operator.as_str()), text!(" "), right_doc])
        })
    }
}

impl<'a> Format<'a> for LogicalExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, LogicalExpression, {
            let doc = binaryish::print_binaryish_expression(
                p,
                &self.left,
                self.operator.into(),
                &self.right,
            );

            if misc::in_parentheses(p.parent_kind(), p.source_text, self.span) {
                group!(p, [indent!(p, [softline!(), doc]), softline!()])
            } else {
                doc
            }
        })
    }
}

impl<'a> Format<'a> for ConditionalExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ConditionalExpression, { ternary::print_ternary(p, self) })
    }
}

impl<'a> Format<'a> for AssignmentExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, AssignmentExpression, {
            let left_doc = self.left.format(p);
            assignment::print_assignment(
                p,
                assignment::AssignmentLike::AssignmentExpression(self),
                left_doc,
                array!(p, [text!(" "), text!(self.operator.as_str())]),
                Some(&self.right),
            )
        })
    }
}

impl<'a> Format<'a> for AssignmentTarget<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            match_simple_assignment_target!(Self) => self.to_simple_assignment_target().format(p),
            match_assignment_target_pattern!(Self) => self.to_assignment_target_pattern().format(p),
        }
    }
}

impl<'a> Format<'a> for SimpleAssignmentTarget<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.format(p),
            match_member_expression!(Self) => self.to_member_expression().format(p),
            Self::TSAsExpression(expr) => expr.format(p),
            Self::TSSatisfiesExpression(expr) => expr.format(p),
            Self::TSNonNullExpression(expr) => expr.format(p),
            Self::TSTypeAssertion(expr) => expr.format(p),
            Self::TSInstantiationExpression(expr) => expr.format(p),
        }
    }
}

impl<'a> Format<'a> for AssignmentTargetPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::ArrayAssignmentTarget(target) => target.format(p),
            Self::ObjectAssignmentTarget(target) => target.format(p),
        }
    }
}

impl<'a> Format<'a> for ArrayAssignmentTarget<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array::print_array(p, &array::ArrayLike::ArrayAssignmentTarget(self))
    }
}

impl<'a> Format<'a> for AssignmentTargetMaybeDefault<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            match_assignment_target!(AssignmentTargetMaybeDefault) => {
                self.to_assignment_target().format(p)
            }
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(v) => v.format(p),
        }
    }
}

impl<'a> Format<'a> for ObjectAssignmentTarget<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        object::print_object(p, object::ObjectLike::AssignmentTarget(self))
    }
}

impl<'a> Format<'a> for AssignmentTargetWithDefault<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, AssignmentTargetWithDefault, {
            array!(p, [self.binding.format(p), text!(" = "), self.init.format(p)])
        })
    }
}

impl<'a> Format<'a> for AssignmentTargetProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::AssignmentTargetPropertyIdentifier(ident) => ident.format(p),
            Self::AssignmentTargetPropertyProperty(prop) => prop.format(p),
        }
    }
}

impl<'a> Format<'a> for AssignmentTargetPropertyIdentifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        parts.push(self.binding.format(p));

        if let Some(init) = &self.init {
            parts.push(text!(" = "));
            parts.push(init.format(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for AssignmentTargetPropertyProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let left_doc = self.name.format(p);

        // TODO: How to convert `AssignmentTargetMaybeDefault` to `Expression`?
        // Or `print_assignment` is not needed?
        // assignment::print_assignment(
        //     p,
        //     assignment::AssignmentLike::AssignmentTargetPropertyProperty(self),
        //     left_doc,
        //     text!(":"),
        //     // self.binding
        // )
        group!(p, [left_doc, text!(": "), self.binding.format(p)])
    }
}

impl<'a> Format<'a> for AssignmentTargetRest<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array!(p, [text!("..."), self.target.format(p)])
    }
}

impl<'a> Format<'a> for SequenceExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, SequenceExpression, {
            let docs =
                self.expressions.iter().map(|expr| expr.format(p)).collect::<std::vec::Vec<_>>();
            group!(p, [join!(p, JoinSeparator::CommaLine, docs)])
        })
    }
}

impl<'a> Format<'a> for ParenthesizedExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        unreachable!("Parser preserve_parens option need to be set to false.");
    }
}

impl<'a> Format<'a> for ImportExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ImportExpression, {
            // TODO: Use `print_call_expression`?
            let mut parts = Vec::new_in(p.allocator);
            parts.push(text!("import"));
            parts.push(text!("("));
            let mut indent_parts = Vec::new_in(p.allocator);
            indent_parts.push(softline!());
            indent_parts.push(self.source.format(p));
            if !self.arguments.is_empty() {
                for arg in &self.arguments {
                    indent_parts.push(text!(","));
                    indent_parts.push(line!());
                    indent_parts.push(arg.format(p));
                }
            }
            parts.push(group!(p, [indent!(p, indent_parts)]));
            parts.push(softline!());
            parts.push(text!(")"));

            group!(p, parts)
        })
    }
}

impl<'a> Format<'a> for TemplateLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        template_literal::print_template_literal(
            p,
            &template_literal::TemplateLiteralLike::TemplateLiteral(self),
        )
    }
}

impl<'a> Format<'a> for TemplateElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        utils::replace_end_of_line(
            p,
            dynamic_text!(p, self.value.raw.as_str()),
            JoinSeparator::Literalline,
        )
    }
}

impl<'a> Format<'a> for TaggedTemplateExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, TaggedTemplateExpression, {
            template_literal::print_tagged_template_literal(p, self)
        })
    }
}

impl<'a> Format<'a> for Super {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("super")
    }
}

impl<'a> Format<'a> for AwaitExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, AwaitExpression, {
            let mut parts = Vec::new_in(p.allocator);
            parts.push(text!("await "));
            parts.push(self.argument.format(p));
            array!(p, parts)
        })
    }
}

impl<'a> Format<'a> for ChainExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ChainExpression, {
            match &self.expression {
                ChainElement::CallExpression(expr) => expr.format(p),
                ChainElement::TSNonNullExpression(expr) => expr.format(p),
                match_member_expression!(ChainElement) => {
                    self.expression.to_member_expression().format(p)
                }
            }
        })
    }
}

impl<'a> Format<'a> for NewExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, NewExpression, {
            call_expression::print_call_expression(
                p,
                &call_expression::CallExpressionLike::NewExpression(self),
            )
        })
    }
}

impl<'a> Format<'a> for MetaProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array!(p, [self.meta.format(p), text!("."), self.property.format(p)])
    }
}

impl<'a> Format<'a> for Class<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, Class, { class::print_class(p, self) })
    }
}

impl<'a> Format<'a> for ClassBody<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ClassBody, { class::print_class_body(p, self) })
    }
}

impl<'a> Format<'a> for ClassElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            ClassElement::StaticBlock(c) => c.format(p),
            ClassElement::MethodDefinition(c) => c.format(p),
            ClassElement::PropertyDefinition(c) => c.format(p),
            ClassElement::AccessorProperty(c) => c.format(p),
            ClassElement::TSIndexSignature(c) => c.format(p),
        }
    }
}

impl<'a> Format<'a> for StaticBlock<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, StaticBlock, {
            array!(p, [text!("static "), block::print_block(p, &self.body, None)])
        })
    }
}

impl<'a> Format<'a> for MethodDefinition<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, MethodDefinition, { class::print_class_method(p, self) })
    }
}

impl<'a> Format<'a> for PropertyDefinition<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, PropertyDefinition, {
            class::print_class_property(p, &class::ClassPropertyLike::PropertyDefinition(self))
        })
    }
}

impl<'a> Format<'a> for AccessorProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        class::print_class_property(p, &class::ClassPropertyLike::AccessorProperty(self))
    }
}

impl<'a> Format<'a> for PrivateIdentifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array!(p, [text!("#"), dynamic_text!(p, self.name.as_str())])
    }
}

impl<'a> Format<'a> for BindingPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        parts.push(match &self.kind {
            BindingPatternKind::BindingIdentifier(ident) => ident.format(p),
            BindingPatternKind::ObjectPattern(pattern) => pattern.format(p),
            BindingPatternKind::ArrayPattern(pattern) => pattern.format(p),
            BindingPatternKind::AssignmentPattern(pattern) => pattern.format(p),
        });

        if self.optional {
            parts.push(text!("?"));
        }

        if let Some(typ) = &self.type_annotation {
            parts.push(array!(p, [text!(": "), typ.type_annotation.format(p)]));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for ObjectPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ObjectPattern, {
            object::print_object(p, object::ObjectLike::Pattern(self))
        })
    }
}

impl<'a> Format<'a> for BindingProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        if self.shorthand {
            return self.value.format(p);
        }

        let left_doc = self.key.format(p);

        // TODO: How to convert `BindingPattern` to `Expression`...?
        // Or `print_assignment` is not needed?
        // assignment::print_assignment(
        //     p,
        //     assignment::AssignmentLike::BindingProperty(self),
        //     left_doc,
        //     text!(":"),
        //     Some(&self.value),
        // )
        group!(p, [left_doc, text!(": "), self.value.format(p)])
    }
}

impl<'a> Format<'a> for BindingRestElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, BindingRestElement, { array!(p, [text!("..."), self.argument.format(p)]) })
    }
}

impl<'a> Format<'a> for ArrayPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ArrayPattern, {
            array::print_array(p, &array::ArrayLike::ArrayPattern(self))
        })
    }
}

impl<'a> Format<'a> for AssignmentPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, AssignmentPattern, {
            array!(p, [self.left.format(p), text!(" = "), self.right.format(p)])
        })
    }
}
