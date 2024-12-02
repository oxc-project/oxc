//! Formatting logic
//!
//! References:
//! * <https://github.com/prettier/prettier/blob/3.3.3/src/language-js/print/estree.js>

#![allow(unused_variables)]

mod array;
mod arrow_function;
mod assignment;
mod binaryish;
mod block;
mod call_arguments;
mod call_expression;
mod class;
mod function;
mod function_parameters;
mod misc;
mod module;
mod object;
mod property;
mod statement;
mod string;
mod template_literal;
mod ternary;

use std::borrow::Cow;

use cow_utils::CowUtils;
use oxc_allocator::{Box, Vec};
use oxc_ast::{ast::*, AstKind};
use oxc_span::GetSpan;
use oxc_syntax::identifier::{is_identifier_name, is_line_terminator};

use crate::{
    format::{array::Array, object::ObjectLike, template_literal::TemplateLiteralPrinter},
    group,
    ir::{Doc, DocBuilder, Group, Separator},
    p_str, p_vec, wrap, Prettier,
};

pub trait Format<'a> {
    #[must_use]
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a>;
}

impl<'a, T> Format<'a> for Box<'a, T>
where
    T: Format<'a>,
{
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        (**self).format(p)
    }
}

impl<'a> Format<'a> for Program<'a> {
    #[allow(clippy::cast_possible_truncation)]
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, Program, {
            let mut parts = p.vec();

            if let Some(hashbang) = &self.hashbang {
                parts.push(hashbang.format(p));
            }

            if let Some(doc) = block::print_block_body(
                p,
                &self.body,
                Some(&self.directives),
                false,
                /* is_root */ true,
            ) {
                parts.push(doc);
            }

            p.array(parts)
        })
    }
}

impl<'a> Format<'a> for Hashbang<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(p.text(p_str!(p, self.span.source_text(p.source_text))));
        parts.extend(p.hardline());
        // Preserve original newline
        if let Some(c) = p.source_text[self.span.end as usize..].chars().nth(1) {
            if is_line_terminator(c) {
                parts.extend(p.hardline());
            }
        }
        p.array(parts)
    }
}

impl<'a> Format<'a> for Directive<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(p.text(p_str!(
            p,
            string::print_string(p, self.directive.as_str(), p.options.single_quote)
        )));
        if let Some(semi) = p.semi() {
            parts.push(semi);
        }
        parts.extend(p.hardline());
        p.array(parts)
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
            let mut parts = p.vec();
            parts.push(self.expression.format(p));
            if let Some(semi) = p.semi() {
                parts.push(semi);
            }
            p.array(parts)
        })
    }
}

impl<'a> Format<'a> for EmptyStatement {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("")
    }
}

impl<'a> Format<'a> for IfStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, IfStatement, {
            let mut parts = p.vec();

            let test_doc = self.test.format(p);
            let consequent = self.consequent.format(p);
            let consequent = misc::adjust_clause(p, &self.consequent, consequent, false);

            let opening = group![
                p,
                p.text("if ("),
                group!(p, p.indent(p_vec!(p, p.softline(), test_doc)), p.softline()),
                p.text(")"),
                consequent
            ];
            parts.push(opening);

            if let Some(alternate) = &self.alternate {
                let else_on_same_line = matches!(alternate, Statement::BlockStatement(_));
                if else_on_same_line {
                    parts.push(p.space());
                } else {
                    parts.extend(p.hardline());
                }
                parts.push(p.text("else"));
                let alternate_doc = alternate.format(p);
                parts.push(group!(
                    p,
                    misc::adjust_clause(
                        p,
                        alternate,
                        alternate_doc,
                        matches!(alternate, Statement::IfStatement(_))
                    )
                ));
            }

            p.array(parts)
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
            let body = self.body.format(p);
            let body = misc::adjust_clause(p, &self.body, body, false);

            if self.init.is_none() && self.test.is_none() && self.update.is_none() {
                return group![p, p.text("for (;;)"), body];
            }

            let parts_head = {
                let mut parts_head = p.vec();
                parts_head.push(p.softline());
                if let Some(init) = &self.init {
                    parts_head.push(init.format(p));
                }
                parts_head.push(p.text(";"));
                parts_head.push(p.line());
                if let Some(init) = &self.test {
                    parts_head.push(init.format(p));
                }
                parts_head.push(p.text(";"));
                parts_head.push(p.line());
                if let Some(init) = &self.update {
                    parts_head.push(init.format(p));
                }
                p.indent(parts_head)
            };

            group![p, p.text("for ("), group![p, parts_head, p.softline()], p.text(")"), body]
        })
    }
}

impl<'a> Format<'a> for ForStatementInit<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            ForStatementInit::VariableDeclaration(v) => v.format(p),
            match_expression!(ForStatementInit) => self.to_expression().format(p),
        }
    }
}

impl<'a> Format<'a> for ForInStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ForInStatement, {
            let mut parts = p.vec();
            parts.push(p.text("for ("));
            parts.push(self.left.format(p));
            parts.push(p.text(" in "));
            parts.push(self.right.format(p));
            parts.push(p.text(")"));
            let body = self.body.format(p);
            parts.push(misc::adjust_clause(p, &self.body, body, false));
            Doc::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for ForOfStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ForOfStatement, {
            let mut parts = p.vec();
            parts.push(p.text("for"));
            if self.r#await {
                parts.push(p.text(" await"));
            }
            parts.push(p.text(" ("));
            parts.push(self.left.format(p));
            parts.push(p.text(" of "));
            parts.push(self.right.format(p));
            parts.push(p.text(")"));
            let body = self.body.format(p);
            parts.push(misc::adjust_clause(p, &self.body, body, false));
            Doc::Group(Group::new(parts))
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
            let mut parts = p.vec();

            parts.push(p.text("while ("));
            let test_doc = self.test.format(p);
            parts.push(group!(p, p.indent(p_vec!(p, p.softline(), test_doc)), p.softline()));
            parts.push(p.text(")"));

            let body = self.body.format(p);
            parts.push(misc::adjust_clause(p, &self.body, body, false));

            Doc::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for DoWhileStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, DoWhileStatement, {
            let mut parts = p.vec();

            let clause = self.body.format(p);
            let clause = misc::adjust_clause(p, &self.body, clause, false);
            let do_body = group!(p, p.text("do"), clause);

            parts.push(do_body);

            if matches!(self.body, Statement::BlockStatement(_)) {
                parts.push(p.space());
            } else {
                parts.extend(p.hardline());
            }

            parts.push(p.text("while ("));
            let test_doc = self.test.format(p);
            parts.push(group!(p, p.indent(p_vec!(p, p.softline(), test_doc)), p.softline()));
            parts.push(p.text(")"));
            if let Some(semi) = p.semi() {
                parts.push(semi);
            }

            p.array(parts)
        })
    }
}

impl<'a> Format<'a> for ContinueStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(p.text("continue"));

        if let Some(label) = &self.label {
            parts.push(p.space());
            parts.push(label.format(p));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for BreakStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(p.text("break"));

        if let Some(label) = &self.label {
            parts.push(p.space());
            parts.push(label.format(p));
        }

        if p.options.semi {
            parts.push(p.text(";"));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for SwitchStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, SwitchStatement, {
            let mut parts = p.vec();

            let mut header_parts = p.vec();

            header_parts.push(p.text("switch ("));

            let discriminant_doc = self.discriminant.format(p);
            header_parts.push(p.indent(p_vec!(p, p.softline(), discriminant_doc)));

            header_parts.push(p.softline());
            header_parts.push(p.text(")"));

            parts.push(Doc::Group(Group::new(header_parts)));

            parts.push(p.text(" {"));

            let mut cases_parts = p.vec();
            let len = self.cases.len();
            for (i, case) in self.cases.iter().enumerate() {
                cases_parts.push({
                    let mut parts = p.vec();
                    parts.extend(p.hardline());
                    parts.push(case.format(p));
                    p.indent(parts)
                });
                if i != len - 1 && p.is_next_line_empty(case.span) {
                    cases_parts.extend(p.hardline());
                }
            }
            parts.extend(cases_parts);

            parts.extend(p.hardline());
            parts.push(p.text("}"));

            p.array(parts)
        })
    }
}

impl<'a> Format<'a> for SwitchCase<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        if let Some(test) = &self.test {
            parts.push(p.text("case "));
            parts.push(test.format(p));
            parts.push(p.text(":"));
        } else {
            parts.push(p.text("default:"));
        }

        let consequent: Vec<_> = Vec::from_iter_in(
            self.consequent.iter().filter(|c| !matches!(c, Statement::EmptyStatement(_))),
            p.allocator,
        );
        let len = consequent.len();
        let is_only_one_block_statement =
            len == 1 && matches!(self.consequent[0], Statement::BlockStatement(_));

        let mut consequent_parts = p.vec();
        for i in 0..len {
            let stmt = &consequent[i];

            if i != 0 && matches!(stmt, Statement::BreakStatement(_)) {
                let last_stmt = &consequent[i - 1];
                if p.is_next_line_empty(last_stmt.span()) {
                    consequent_parts.extend(p.hardline());
                }
            }

            if is_only_one_block_statement {
                consequent_parts.push(p.space());
            } else {
                consequent_parts.extend(p.hardline());
            }
            consequent_parts.push(stmt.format(p));
        }

        if !consequent_parts.is_empty() {
            if is_only_one_block_statement {
                parts.extend(consequent_parts);
            } else {
                parts.push(p.indent(p_vec!(p, Doc::Group(Group::new(consequent_parts)))));
            }
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for ReturnStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ReturnStatement, {
            function::print_return_or_throw_argument(p, self.argument.as_ref(), true)
        })
    }
}

impl<'a> Format<'a> for LabeledStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        if matches!(self.body, Statement::EmptyStatement(_)) {
            let label_doc = self.label.format(p);
            return p.array(p_vec!(p, label_doc, p.text(":;")));
        }

        let label_doc = self.label.format(p);
        let body_doc = self.body.format(p);
        p.array(p_vec!(p, label_doc, p.text(": "), body_doc))
    }
}

impl<'a> Format<'a> for TryStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, TryStatement, {
            let mut parts = p.vec();

            parts.push(p.text("try "));
            parts.push(self.block.format(p));
            if let Some(handler) = &self.handler {
                parts.push(p.space());
                parts.push(handler.format(p));
            }
            if let Some(finalizer) = &self.finalizer {
                parts.push(p.text(" finally "));
                parts.push(finalizer.format(p));
            }

            p.array(parts)
        })
    }
}

impl<'a> Format<'a> for CatchClause<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, CatchClause, {
            let mut parts = p.vec();

            parts.push(p.text("catch "));
            if let Some(param) = &self.param {
                parts.push(p.text("("));
                parts.push(param.pattern.format(p));
                parts.push(p.text(") "));
            }
            parts.push(self.body.format(p));

            p.array(parts)
        })
    }
}

impl<'a> Format<'a> for ThrowStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        function::print_return_or_throw_argument(p, Some(&self.argument), false)
    }
}

impl<'a> Format<'a> for WithStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let body_doc = self.body.format(p);
        group![
            p,
            p.text("with ("),
            self.object.format(p),
            p.text(")"),
            misc::adjust_clause(p, &self.body, body_doc, false)
        ]
    }
}

impl<'a> Format<'a> for DebuggerStatement {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(p.text("debugger"));

        if p.options.semi {
            parts.push(p.text(";"));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for ModuleDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ModuleDeclaration, {
            if let ModuleDeclaration::ImportDeclaration(decl) = self {
                decl.format(p)
            } else {
                module::print_export_declaration(p, self)
            }
        })
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

            let mut parts = p.vec();

            if self.declare {
                parts.push(p.text("declare "));
            }

            parts.push(p.text(kind));
            parts.push(p.space());

            let is_hardline = !p.parent_kind().is_iteration_statement()
                && self.declarations.iter().all(|decl| decl.init.is_some());
            let decls_len = self.declarations.len();
            parts.extend(self.declarations.iter().enumerate().map(|(i, decl)| {
                if decls_len > 1 {
                    let mut d_parts = p.vec();
                    if i != 0 {
                        d_parts.push(p.text(","));
                        if is_hardline {
                            d_parts.extend(p.hardline());
                        } else {
                            d_parts.push(p.line());
                        }
                    }
                    d_parts.push(decl.format(p));
                    p.indent(d_parts)
                } else {
                    decl.format(p)
                }
            }));

            if !parent_for_loop.is_some_and(|span| span != self.span) {
                if let Some(semi) = p.semi() {
                    parts.push(semi);
                }
            }

            Doc::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for VariableDeclarator<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, VariableDeclarator, { assignment::print_variable_declarator(p, self) })
    }
}

impl<'a> Format<'a> for TSTypeAliasDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        if self.declare {
            parts.push(p.text("declare "));
        }

        parts.push(p.text("type "));
        parts.push(self.id.format(p));

        if let Some(params) = &self.type_parameters {
            parts.push(params.format(p));
        }

        parts.push(p.text(" = "));
        parts.push(self.type_annotation.format(p));

        if let Some(semi) = p.semi() {
            parts.push(semi);
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSType::TSAnyKeyword(v) => v.format(p),
            TSType::TSBigIntKeyword(v) => v.format(p),
            TSType::TSBooleanKeyword(v) => v.format(p),
            TSType::TSIntrinsicKeyword(v) => v.format(p),
            TSType::TSNeverKeyword(v) => v.format(p),
            TSType::TSNullKeyword(v) => v.format(p),
            TSType::TSNumberKeyword(v) => v.format(p),
            TSType::TSObjectKeyword(v) => v.format(p),
            TSType::TSStringKeyword(v) => v.format(p),
            TSType::TSSymbolKeyword(v) => v.format(p),
            TSType::TSThisType(v) => v.format(p),
            TSType::TSUndefinedKeyword(v) => v.format(p),
            TSType::TSUnknownKeyword(v) => v.format(p),
            TSType::TSVoidKeyword(v) => v.format(p),
            TSType::TSArrayType(v) => v.format(p),
            TSType::TSConditionalType(v) => v.format(p),
            TSType::TSConstructorType(v) => v.format(p),
            TSType::TSFunctionType(v) => v.format(p),
            TSType::TSImportType(v) => v.format(p),
            TSType::TSIndexedAccessType(v) => v.format(p),
            TSType::TSInferType(v) => v.format(p),
            TSType::TSIntersectionType(v) => v.format(p),
            TSType::TSLiteralType(v) => v.format(p),
            TSType::TSMappedType(v) => v.format(p),
            TSType::TSNamedTupleMember(v) => v.format(p),
            TSType::TSQualifiedName(v) => v.format(p),
            TSType::TSTemplateLiteralType(v) => v.format(p),
            TSType::TSTupleType(v) => v.format(p),
            TSType::TSTypeLiteral(v) => v.format(p),
            TSType::TSTypeOperatorType(v) => v.format(p),
            TSType::TSTypePredicate(v) => v.format(p),
            TSType::TSTypeQuery(v) => v.format(p),
            TSType::TSTypeReference(v) => v.format(p),
            TSType::TSUnionType(v) => v.format(p),
            TSType::TSParenthesizedType(v) => v.format(p),
            TSType::JSDocNullableType(v) => v.format(p),
            TSType::JSDocNonNullableType(v) => v.format(p),
            TSType::JSDocUnknownType(v) => v.format(p),
        }
    }
}

impl<'a> Format<'a> for TSAnyKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("any")
    }
}

impl<'a> Format<'a> for TSBigIntKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("bigint")
    }
}

impl<'a> Format<'a> for TSBooleanKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("boolean")
    }
}

impl<'a> Format<'a> for TSIntrinsicKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("intrinsic")
    }
}

impl<'a> Format<'a> for TSNeverKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("never")
    }
}

impl<'a> Format<'a> for TSNullKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("null")
    }
}

impl<'a> Format<'a> for TSNumberKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("number")
    }
}

impl<'a> Format<'a> for TSObjectKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("object")
    }
}

impl<'a> Format<'a> for TSStringKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("string")
    }
}

impl<'a> Format<'a> for TSSymbolKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("symbol")
    }
}

impl<'a> Format<'a> for TSThisType {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("this")
    }
}

impl<'a> Format<'a> for TSUndefinedKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("undefined")
    }
}

impl<'a> Format<'a> for TSUnknownKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("unknown")
    }
}

impl<'a> Format<'a> for TSVoidKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("void")
    }
}

impl<'a> Format<'a> for TSArrayType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let element_type_doc = self.element_type.format(p);
        p.array(p_vec!(p, element_type_doc, p.text("[]")))
    }
}

impl<'a> Format<'a> for TSConditionalType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(self.check_type.format(p));
        parts.push(p.text(" extends "));
        parts.push(self.extends_type.format(p));
        parts.push(p.text(" ? "));
        parts.push(self.true_type.format(p));
        parts.push(p.text(" : "));
        parts.push(self.false_type.format(p));

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSConstructorType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        if self.r#abstract {
            parts.push(p.text("abstract "));
        }
        parts.push(p.text("new "));
        parts.push(self.params.format(p));
        let type_annotation_doc = self.return_type.type_annotation.format(p);
        parts.push(p.array(p_vec!(p, p.text(" => "), type_annotation_doc)));
        p.array(parts)
    }
}

impl<'a> Format<'a> for TSFunctionType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        parts.push(self.params.format(p));

        parts.push(p.text(" => "));
        parts.push(self.return_type.type_annotation.format(p));

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSThisParameter<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(p.text("this"));

        if let Some(type_annotation) = &self.type_annotation {
            parts.push(p.text(": "));
            parts.push(type_annotation.type_annotation.format(p));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSImportType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        if self.is_type_of {
            parts.push(p.text("typeof "));
        }

        parts.push(p.text("import("));
        parts.push(self.parameter.format(p));
        // ToDo: attributes
        parts.push(p.text(")"));

        if let Some(qualifier) = &self.qualifier {
            parts.push(p.text("."));
            parts.push(qualifier.format(p));
        }

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSIndexedAccessType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(self.object_type.format(p));
        parts.push(p.text("["));
        parts.push(self.index_type.format(p));
        parts.push(p.text("]"));
        p.array(parts)
    }
}

impl<'a> Format<'a> for TSInferType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let type_parameter_doc = self.type_parameter.format(p);
        p.array(p_vec!(p, p.text("infer "), type_parameter_doc))
    }
}

impl<'a> Format<'a> for TSIntersectionType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        let mut add_symbol = false;

        for ts_type in &self.types {
            if add_symbol {
                parts.push(p.text(" & "));
            } else {
                add_symbol = true;
            }

            parts.push(ts_type.format(p));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSLiteralType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match &self.literal {
            TSLiteral::BooleanLiteral(v) => v.format(p),
            TSLiteral::NullLiteral(v) => v.format(p),
            TSLiteral::NumericLiteral(v) => v.format(p),
            TSLiteral::BigIntLiteral(v) => v.format(p),
            TSLiteral::RegExpLiteral(v) => v.format(p),
            TSLiteral::StringLiteral(v) => v.format(p),
            TSLiteral::TemplateLiteral(v) => v.format(p),
            TSLiteral::UnaryExpression(v) => v.format(p),
        }
    }
}

impl<'a> Format<'a> for TSMappedType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts: Vec<'_, Doc<'_>> = p.vec();

        match self.readonly {
            TSMappedTypeModifierOperator::Plus => parts.push(p.text("+readonly ")),
            TSMappedTypeModifierOperator::Minus => parts.push(p.text("-readonly ")),
            TSMappedTypeModifierOperator::True => parts.push(p.text("readonly ")),
            TSMappedTypeModifierOperator::None => (),
        }

        parts.push(p.text("["));
        parts.push(self.type_parameter.format(p));

        if let Some(name_type) = &self.name_type {
            parts.push(p.text(" as "));
            parts.push(name_type.format(p));
        }

        parts.push(p.text("]"));

        match self.optional {
            TSMappedTypeModifierOperator::Plus => parts.push(p.text("+?")),
            TSMappedTypeModifierOperator::Minus => parts.push(p.text("-?")),
            TSMappedTypeModifierOperator::True => parts.push(p.text("?")),
            TSMappedTypeModifierOperator::None => (),
        }

        if let Some(type_annotation) = &self.type_annotation {
            parts.push(p.text(": "));
            parts.push(type_annotation.format(p));
        }

        let mut result = p.vec();
        result.push(p.text("{ "));

        // ToDo: check ident/grouping in method/method-signature.ts
        result.push(Doc::Group(Group::new(parts)));
        result.push(p.text(" }"));

        p.array(result)
    }
}

impl<'a> Format<'a> for TSNamedTupleMember<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(self.label.format(p));

        if self.optional {
            parts.push(p.text("?"));
        }

        parts.push(p.text(": "));
        parts.push(self.element_type.format(p));

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSRestType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let type_annotation_doc = self.type_annotation.format(p);
        p.array(p_vec!(p, p.text("..."), type_annotation_doc))
    }
}

impl<'a> Format<'a> for TSOptionalType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let type_annotation_doc = self.type_annotation.format(p);
        p.array(p_vec!(p, type_annotation_doc, p.text("?")))
    }
}

impl<'a> Format<'a> for TSQualifiedName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let left_doc = self.left.format(p);
        let right_doc = self.right.format(p);
        p.array(p_vec!(p, left_doc, p.text("."), right_doc))
    }
}

impl<'a> Format<'a> for TSTemplateLiteralType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        template_literal::print_template_literal(
            p,
            &TemplateLiteralPrinter::TSTemplateLiteralType(self),
        )
    }
}

impl<'a> Format<'a> for TSTupleType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array::print_array(p, &Array::TSTupleType(self))
    }
}

impl<'a> Format<'a> for TSTypeLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        object::print_object_properties(p, ObjectLike::TSTypeLiteral(self))
    }
}

impl<'a> Format<'a> for TSTypeOperator<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(p.text(self.operator.to_str()));
        parts.push(p.space());
        parts.push(self.type_annotation.format(p));

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSTypePredicate<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        if self.asserts {
            parts.push(p.text("asserts "));
        }
        parts.push(self.parameter_name.format(p));

        if let Some(type_annotation) = &self.type_annotation {
            parts.push(p.text(" is "));
            parts.push(type_annotation.type_annotation.format(p));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSTypePredicateName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSTypePredicateName::Identifier(it) => it.format(p),
            TSTypePredicateName::This(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for TSTypeQuery<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(p.text("typeof "));

        match &self.expr_name {
            TSTypeQueryExprName::TSImportType(import_type) => parts.push(import_type.format(p)),
            TSTypeQueryExprName::IdentifierReference(identifier_reference) => {
                parts.push(identifier_reference.format(p));
            }
            TSTypeQueryExprName::QualifiedName(qualified_name) => {
                parts.push(qualified_name.format(p));
            }
        }

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSTypeReference<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(self.type_name.format(p));
        if let Some(params) = &self.type_parameters {
            parts.push(params.format(p));
        }
        p.array(parts)
    }
}

impl<'a> Format<'a> for TSParenthesizedType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, TSParenthesizedType, { self.type_annotation.format(p) })
    }
}

impl<'a> Format<'a> for TSUnionType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        let mut add_symbol = false;

        for ts_type in &self.types {
            if add_symbol {
                parts.push(p.text(" | "));
            } else {
                add_symbol = true;
            }

            parts.push(ts_type.format(p));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for JSDocNullableType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.line()
    }
}

impl<'a> Format<'a> for JSDocNonNullableType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.line()
    }
}

impl<'a> Format<'a> for JSDocUnknownType {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.line()
    }
}

impl<'a> Format<'a> for TSInterfaceDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        if self.declare {
            parts.push(p.text("declare "));
        }

        parts.push(p.text("interface "));
        parts.push(self.id.format(p));

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        parts.push(p.space());

        if let Some(extends) = &self.extends {
            if extends.len() > 0 {
                let mut extends_parts = p.vec();
                let mut display_comma = false;

                extends_parts.push(p.text("extends "));

                for extend in extends {
                    if display_comma {
                        extends_parts.push(p.text(", "));
                    } else {
                        display_comma = true;
                    }

                    extends_parts.push(extend.expression.format(p));
                    if let Some(type_parameters) = &extend.type_parameters {
                        extends_parts.push(type_parameters.format(p));
                    }
                }

                parts.extend(extends_parts);
                parts.push(p.space());
            }
        }

        parts.push(p.text("{"));
        if self.body.body.len() > 0 {
            let mut indent_parts = p.vec();
            for sig in &self.body.body {
                indent_parts.extend(p.hardline());
                indent_parts.push(sig.format(p));

                if let Some(semi) = p.semi() {
                    indent_parts.push(semi);
                }
            }
            parts.push(p.indent(indent_parts));
            parts.extend(p.hardline());
        }
        parts.push(p.text("}"));
        p.array(parts)
    }
}

impl<'a> Format<'a> for TSEnumDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        if self.declare {
            parts.push(p.text("declare "));
        }
        if self.r#const {
            parts.push(p.text("const "));
        }
        parts.push(p.text("enum "));
        parts.push(self.id.format(p));
        parts.push(p.text(" {"));
        if self.members.len() > 0 {
            let mut indent_parts = p.vec();
            for member in &self.members {
                indent_parts.extend(p.hardline());
                indent_parts.push(member.format(p));
            }
            parts.push(p.indent(indent_parts));
            parts.extend(p.hardline());
        }
        parts.push(p.text("}"));

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSEnumMember<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(self.id.format(p));

        if let Some(initializer) = &self.initializer {
            parts.push(p.text(" = "));
            parts.push(initializer.format(p));
        }

        parts.push(p.text(","));

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSEnumMemberName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSEnumMemberName::Identifier(identifier) => identifier.format(p),
            TSEnumMemberName::String(string_literal) => string_literal.format(p),
        }
    }
}

impl<'a> Format<'a> for TSModuleDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        if self.declare {
            parts.push(p.text("declare "));
        }

        parts.push(p.text(self.kind.as_str()));
        parts.push(p.space());
        parts.push(self.id.format(p));
        parts.push(p.text(" {"));

        if let Some(body) = &self.body {
            if !body.is_empty() {
                let mut indent_parts = p.vec();

                indent_parts.extend(p.hardline());
                indent_parts.push(body.format(p));

                parts.push(p.indent(indent_parts));
                parts.extend(p.hardline());
            }
        }

        parts.push(p.text("}"));

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSModuleDeclarationName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSModuleDeclarationName::Identifier(identifier) => identifier.format(p),
            TSModuleDeclarationName::StringLiteral(string_literal) => string_literal.format(p),
        }
    }
}

impl<'a> Format<'a> for TSModuleDeclarationBody<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSModuleDeclarationBody::TSModuleBlock(module_block) => module_block.format(p),
            TSModuleDeclarationBody::TSModuleDeclaration(module_declaration) => {
                module_declaration.format(p)
            }
        }
    }
}

impl<'a> Format<'a> for TSModuleBlock<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        let mut add_line = false;

        for body_part in &self.body {
            if add_line {
                parts.push(p.line());
            } else {
                add_line = true;
            }

            parts.push(body_part.format(p));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSImportEqualsDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(p.text("import "));

        if self.import_kind == ImportOrExportKind::Type {
            parts.push(p.text("type "));
        }

        parts.push(self.id.format(p));
        parts.push(p.text(" = "));
        parts.push(self.module_reference.format(p));

        if let Some(semi) = p.semi() {
            parts.push(semi);
        }
        p.array(parts)
    }
}

impl<'a> Format<'a> for TSModuleReference<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSModuleReference::IdentifierReference(it) => it.format(p),
            TSModuleReference::QualifiedName(it) => it.format(p),
            TSModuleReference::ExternalModuleReference(v) => v.format(p),
        }
    }
}

impl<'a> Format<'a> for TSTypeName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSTypeName::IdentifierReference(it) => it.format(p),
            TSTypeName::QualifiedName(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for TSExternalModuleReference<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let expression_doc = self.expression.format(p);
        p.array(p_vec!(p, p.text("require("), expression_doc, p.text(")")))
    }
}

impl<'a> Format<'a> for TSTypeParameter<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        if self.r#in {
            parts.push(p.text("in "));
        }

        if self.out {
            parts.push(p.text("out "));
        }

        parts.push(self.name.format(p));

        if let Some(constraint) = &self.constraint {
            parts.push(p.text(" extends "));
            parts.push(constraint.format(p));
        }

        if let Some(default) = &self.default {
            parts.push(p.text(" = "));
            parts.push(default.format(p));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSTypeParameterDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        let mut print_comma = false;

        parts.push(p.text("<"));

        for param in &self.params {
            if print_comma {
                parts.push(p.text(", "));
            } else {
                print_comma = true;
            }

            parts.push(param.format(p));
        }

        parts.push(p.text(">"));

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSTypeParameterInstantiation<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        let mut print_comma = false;

        parts.push(p.text("<"));

        for param in &self.params {
            if print_comma {
                parts.push(p.text(", "));
            } else {
                print_comma = true;
            }

            parts.push(param.format(p));
        }

        parts.push(p.text(">"));

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSTupleElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSTupleElement::TSOptionalType(it) => it.format(p),
            TSTupleElement::TSRestType(it) => it.format(p),
            TSTupleElement::TSAnyKeyword(it) => it.format(p),
            TSTupleElement::TSBigIntKeyword(it) => it.format(p),
            TSTupleElement::TSBooleanKeyword(it) => it.format(p),
            TSTupleElement::TSIntrinsicKeyword(it) => it.format(p),
            TSTupleElement::TSNeverKeyword(it) => it.format(p),
            TSTupleElement::TSNullKeyword(it) => it.format(p),
            TSTupleElement::TSNumberKeyword(it) => it.format(p),
            TSTupleElement::TSObjectKeyword(it) => it.format(p),
            TSTupleElement::TSStringKeyword(it) => it.format(p),
            TSTupleElement::TSSymbolKeyword(it) => it.format(p),
            TSTupleElement::TSUndefinedKeyword(it) => it.format(p),
            TSTupleElement::TSUnknownKeyword(it) => it.format(p),
            TSTupleElement::TSVoidKeyword(it) => it.format(p),
            TSTupleElement::TSArrayType(it) => it.format(p),
            TSTupleElement::TSConditionalType(it) => it.format(p),
            TSTupleElement::TSConstructorType(it) => it.format(p),
            TSTupleElement::TSFunctionType(it) => it.format(p),
            TSTupleElement::TSImportType(it) => it.format(p),
            TSTupleElement::TSIndexedAccessType(it) => it.format(p),
            TSTupleElement::TSInferType(it) => it.format(p),
            TSTupleElement::TSIntersectionType(it) => it.format(p),
            TSTupleElement::TSLiteralType(it) => it.format(p),
            TSTupleElement::TSMappedType(it) => it.format(p),
            TSTupleElement::TSNamedTupleMember(it) => it.format(p),
            TSTupleElement::TSQualifiedName(it) => it.format(p),
            TSTupleElement::TSTemplateLiteralType(it) => it.format(p),
            TSTupleElement::TSThisType(it) => it.format(p),
            TSTupleElement::TSTupleType(it) => it.format(p),
            TSTupleElement::TSTypeLiteral(it) => it.format(p),
            TSTupleElement::TSTypeOperatorType(it) => it.format(p),
            TSTupleElement::TSTypePredicate(it) => it.format(p),
            TSTupleElement::TSTypeQuery(it) => it.format(p),
            TSTupleElement::TSTypeReference(it) => it.format(p),
            TSTupleElement::TSUnionType(it) => it.format(p),
            TSTupleElement::TSParenthesizedType(it) => it.format(p),
            TSTupleElement::JSDocNullableType(it) => it.format(p),
            TSTupleElement::JSDocNonNullableType(it) => it.format(p),
            TSTupleElement::JSDocUnknownType(it) => it.format(p),
        }
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
        let mut parts = p.vec();
        parts.push(p.text("import"));
        if self.import_kind.is_type() {
            parts.push(p.text(" type"));
        }

        if let Some(specifiers) = &self.specifiers {
            let is_default = specifiers.first().is_some_and(|x| {
                matches!(x, ImportDeclarationSpecifier::ImportDefaultSpecifier(_))
            });

            let validate_namespace = |x: &ImportDeclarationSpecifier| {
                matches!(x, ImportDeclarationSpecifier::ImportNamespaceSpecifier(_))
            };

            let is_namespace = specifiers.first().is_some_and(validate_namespace)
                || specifiers.get(1).is_some_and(validate_namespace);

            parts.push(module::print_module_specifiers(p, specifiers, is_default, is_namespace));
            parts.push(p.text(" from"));
        }
        parts.push(p.space());
        parts.push(self.source.format(p));

        if let Some(with_clause) = &self.with_clause {
            parts.push(p.space());
            parts.push(with_clause.format(p));
        }

        if let Some(semi) = p.semi() {
            parts.push(semi);
        }
        p.array(parts)
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
        let typed = if self.import_kind.is_type() { p.text("type ") } else { p.text("") };

        if self.imported.span() == self.local.span {
            let local_doc = self.local.format(p);
            p.array(p_vec!(p, typed, local_doc))
        } else {
            let imported_doc = self.imported.format(p);
            let local_doc = self.local.format(p);
            p.array(p_vec!(p, typed, imported_doc, p.text(" as "), local_doc))
        }
    }
}

impl<'a> Format<'a> for ImportDefaultSpecifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        self.local.format(p)
    }
}

impl<'a> Format<'a> for ImportNamespaceSpecifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let local_doc = self.local.format(p);
        p.array(p_vec!(p, p.text("* as "), local_doc))
    }
}

impl<'a> Format<'a> for WithClause<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let attribute_keyword_doc = self.attributes_keyword.format(p);
        let with_clause_doc = object::print_object_properties(p, ObjectLike::WithClause(self));
        p.array(p_vec!(p, attribute_keyword_doc, p.space(), with_clause_doc))
    }
}

impl<'a> Format<'a> for ImportAttribute<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let key_doc = self.key.format(p);
        let value_doc = self.value.format(p);
        p.array(p_vec!(p, key_doc, p.text(": "), value_doc))
    }
}

impl<'a> Format<'a> for ImportAttributeKey<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::Identifier(ident) => ident.format(p),
            Self::StringLiteral(literal) => literal.format(p),
        }
    }
}

impl<'a> Format<'a> for ExportNamedDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        if let Some(decl) = &self.declaration {
            parts.push(p.space());
            parts.push(decl.format(p));
        } else {
            parts.push(module::print_module_specifiers(
                p,
                &self.specifiers,
                /* include_default */ false,
                /* include_namespace */ false,
            ));
        }
        p.array(parts)
    }
}

impl<'a> Format<'a> for TSExportAssignment<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let expression_doc = self.expression.format(p);
        p.array(p_vec!(p, p.text(" = "), expression_doc))
    }
}

impl<'a> Format<'a> for TSNamespaceExportDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let id_doc = self.id.format(p);
        p.array(p_vec!(p, p.text(" as namespace "), id_doc, p.text(";")))
    }
}

impl<'a> Format<'a> for ExportSpecifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        if self.exported.span() == self.local.span() {
            self.local.format(p)
        } else {
            let local_doc = self.local.format(p);
            let exported_doc = self.exported.format(p);
            p.array(p_vec!(p, local_doc, p.text(" as "), exported_doc))
        }
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
        let mut parts = p.vec();
        parts.push(p.text(" *"));
        if let Some(exported) = &self.exported {
            parts.push(p.text(" as "));
            parts.push(exported.format(p));
        }
        p.array(parts)
    }
}

impl<'a> Format<'a> for ExportDefaultDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        self.declaration.format(p)
    }
}
impl<'a> Format<'a> for ExportDefaultDeclarationKind<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            match_expression!(Self) => self.to_expression().format(p),
            Self::FunctionDeclaration(decl) => decl.format(p),
            Self::ClassDeclaration(decl) => decl.format(p),
            Self::TSInterfaceDeclaration(decl) => decl.format(p),
        }
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
        wrap!(p, self, IdentifierReference, { p.text(p_str!(p, self.name.as_str())) })
    }
}

impl<'a> Format<'a> for IdentifierName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text(p_str!(p, self.name.as_str()))
    }
}

impl<'a> Format<'a> for BindingIdentifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, BindingIdentifier, { p.text(p_str!(p, self.name.as_str())) })
    }
}

impl<'a> Format<'a> for LabelIdentifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text(p_str!(p, self.name.as_str()))
    }
}

impl<'a> Format<'a> for BooleanLiteral {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text(if self.value { "true" } else { "false" })
    }
}

impl<'a> Format<'a> for NullLiteral {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("null")
    }
}

impl<'a> Format<'a> for NumericLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, NumericLiteral, {
            // See https://github.com/prettier/prettier/blob/3.3.3/src/utils/print-number.js
            // Perf: the regexes from prettier code above are ported to manual search for performance reasons.
            let mut string = self.span.source_text(p.source_text).cow_to_ascii_lowercase();

            // Remove unnecessary plus and zeroes from scientific notation.
            if let Some((head, tail)) = string.split_once('e') {
                let negative = if tail.starts_with('-') { "-" } else { "" };
                let trimmed = tail.trim_start_matches(['+', '-']).trim_start_matches('0');
                if trimmed.starts_with(|c: char| c.is_ascii_digit()) {
                    string = Cow::Owned(std::format!("{head}e{negative}{trimmed}"));
                }
            }

            // Remove unnecessary scientific notation (1e0).
            if let Some((head, tail)) = string.split_once('e') {
                if tail.trim_start_matches(['+', '-']).trim_start_matches('0').is_empty() {
                    string = Cow::Owned(head.to_string());
                }
            }

            // Make sure numbers always start with a digit.
            if string.starts_with('.') {
                string = Cow::Owned(std::format!("0{string}"));
            }

            // Remove extraneous trailing decimal zeroes.
            if let Some((head, tail)) = string.split_once('.') {
                if let Some((head_e, tail_e)) = tail.split_once('e') {
                    if !head_e.is_empty() {
                        let trimmed = head_e.trim_end_matches('0');
                        if trimmed.is_empty() {
                            string = Cow::Owned(std::format!("{head}.0e{tail_e}"));
                        } else {
                            string = Cow::Owned(std::format!("{head}.{trimmed}e{tail_e}"));
                        }
                    }
                } else if !tail.is_empty() {
                    let trimmed = tail.trim_end_matches('0');
                    if trimmed.is_empty() {
                        string = Cow::Owned(std::format!("{head}.0"));
                    } else {
                        string = Cow::Owned(std::format!("{head}.{trimmed}"));
                    }
                }
            }

            // Remove trailing dot.
            if let Some((head, tail)) = string.split_once('.') {
                if tail.is_empty() {
                    string = Cow::Owned(head.to_string());
                } else if tail.starts_with('e') {
                    string = Cow::Owned(std::format!("{head}{tail}"));
                }
            }

            p.text(p_str!(p, &string))
        })
    }
}

impl<'a> Format<'a> for BigIntLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self.span.source_text(p.source_text).cow_to_ascii_lowercase() {
            Cow::Borrowed(s) => p.text(p_str!(p, s)),
            Cow::Owned(s) => p.text(p_str!(p, &s)),
        }
    }
}

impl<'a> Format<'a> for RegExpLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(p.text("/"));
        parts.push(p.text(p_str!(p, self.regex.pattern.source_text(p.source_text).as_ref())));
        parts.push(p.text("/"));
        parts.push(self.regex.flags.format(p));
        p.array(parts)
    }
}

impl<'a> Format<'a> for StringLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, StringLiteral, {
            let raw = &p.source_text[(self.span.start + 1) as usize..(self.span.end - 1) as usize];
            // TODO: implement `makeString` from prettier/src/utils/print-string.js
            p.text(p_str!(p, string::print_string(p, raw, p.options.single_quote)))
        })
    }
}

impl<'a> Format<'a> for ThisExpression {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("this")
    }
}

impl<'a> Format<'a> for MemberExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
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
        let mut parts = p.vec();
        parts.push(self.object.format(p));
        if self.optional {
            parts.push(p.text("?."));
        }
        parts.push(p.text("["));
        parts.push(self.expression.format(p));
        parts.push(p.text("]"));
        p.array(parts)
    }
}

impl<'a> Format<'a> for StaticMemberExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(self.object.format(p));
        if self.optional {
            parts.push(p.text("?"));
        }
        parts.push(p.text("."));
        parts.push(self.property.format(p));
        p.array(parts)
    }
}

impl<'a> Format<'a> for PrivateFieldExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(self.object.format(p));
        parts.push(if self.optional { p.text("?.") } else { p.text(".") });
        parts.push(self.field.format(p));
        p.array(parts)
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
            Self::Elision(elision) => p.text(""),
        }
    }
}

impl<'a> Format<'a> for SpreadElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, SpreadElement, {
            let argument_doc = self.argument.format(p);
            p.array(p_vec!(p, p.text("..."), argument_doc))
        })
    }
}

impl<'a> Format<'a> for ArrayExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ArrayExpression, { array::print_array(p, &Array::ArrayExpression(self)) })
    }
}

impl<'a> Format<'a> for ObjectExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ObjectExpression, {
            object::print_object_properties(p, ObjectLike::Expression(self))
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
                let mut parts = p.vec();
                let mut method = self.method;
                match self.kind {
                    PropertyKind::Get => {
                        parts.push(p.text("get "));
                        method = true;
                    }
                    PropertyKind::Set => {
                        parts.push(p.text("set "));
                        method = true;
                    }
                    PropertyKind::Init => (),
                }
                if method {
                    if let Expression::FunctionExpression(func_expr) = &self.value {
                        parts.push(wrap!(p, func_expr, Function, {
                            function::print_function(
                                p,
                                func_expr,
                                Some(self.key.span().source_text(p.source_text)),
                            )
                        }));
                    }
                } else {
                    parts.push(self.key.format(p));
                    parts.push(p.text(": "));
                    parts.push(self.value.format(p));
                }
                return Doc::Group(Group::new(parts));
            }

            if self.shorthand {
                return self.value.format(p);
            }

            let left_doc = if self.computed {
                let key_doc = self.key.format(p);
                p.array(p_vec!(p, p.text("["), key_doc, p.text("]")))
            } else {
                self.key.format(p)
            };

            assignment::print_assignment(
                p,
                assignment::AssignmentLikeNode::ObjectProperty(self),
                left_doc,
                p.text(":"),
                Some(&self.value),
            )
        })
    }
}

impl<'a> Format<'a> for PropertyKey<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let is_parent_computed = match p.current_kind() {
            AstKind::MethodDefinition(node) => node.computed,
            AstKind::PropertyDefinition(node) => node.computed,
            _ => false,
        };
        if is_parent_computed {
            let mut parts = p.vec();
            parts.push(p.text("["));
            let doc = match self {
                PropertyKey::StaticIdentifier(ident) => ident.format(p),
                PropertyKey::PrivateIdentifier(ident) => ident.format(p),
                match_expression!(PropertyKey) => self.to_expression().format(p),
            };
            parts.push(doc);
            parts.push(p.text("]"));
            return p.array(parts);
        }

        wrap!(p, self, PropertyKey, {
            // Perf: Cache the result of `need_quote` to avoid checking it in each PropertyKey
            let need_quote = p.options.quote_props.consistent()
                && match p.parent_parent_kind() {
                    Some(AstKind::ObjectExpression(a)) => a.properties.iter().any(|x| match x {
                        ObjectPropertyKind::ObjectProperty(p) => {
                            property::is_property_key_has_quote(&p.key)
                        }
                        ObjectPropertyKind::SpreadProperty(_) => false,
                    }),
                    Some(AstKind::ClassBody(a)) => a.body.iter().any(|x| match x {
                        ClassElement::PropertyDefinition(p) => {
                            property::is_property_key_has_quote(&p.key)
                        }
                        _ => false,
                    }),
                    _ => false,
                };

            match self {
                PropertyKey::StaticIdentifier(ident) => {
                    if need_quote {
                        p.text(p_str!(
                            p,
                            string::print_string(p, &ident.name, p.options.single_quote)
                        ))
                    } else {
                        ident.format(p)
                    }
                }
                PropertyKey::PrivateIdentifier(ident) => ident.format(p),
                PropertyKey::StringLiteral(literal) => {
                    // This does not pass quotes/objects.js
                    // because prettier uses the function `isEs5IdentifierName` based on unicode version 3,
                    // but `is_identifier_name` uses the latest unicode version.
                    if is_identifier_name(literal.value.as_str())
                        && (p.options.quote_props.as_needed()
                            || (p.options.quote_props.consistent()/* && !needsQuoteProps.get(parent) */))
                    {
                        p.text(p_str!(p, literal.value.as_str()))
                    } else {
                        p.text(p_str!(
                            p,
                            string::print_string(p, literal.value.as_str(), p.options.single_quote)
                        ))
                    }
                }
                PropertyKey::NumericLiteral(literal) => {
                    if need_quote {
                        p.text(p_str!(
                            p,
                            string::print_string(p, literal.raw, p.options.single_quote)
                        ))
                    } else {
                        literal.format(p)
                    }
                }
                PropertyKey::Identifier(ident) => {
                    let ident_doc = ident.format(p);
                    p.array(p_vec!(p, p.text("["), ident_doc, p.text("]")))
                }
                match_expression!(PropertyKey) => self.to_expression().format(p),
            }
        })
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
            let mut parts = p.vec();
            parts.push(p.text("yield"));
            if self.delegate {
                parts.push(p.text("*"));
            }
            if let Some(argument) = &self.argument {
                parts.push(p.space());
                parts.push(argument.format(p));
            }
            p.array(parts)
        })
    }
}

impl<'a> Format<'a> for UpdateExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, UpdateExpression, {
            if self.prefix {
                let argument_doc = self.argument.format(p);
                p.array(p_vec!(p, p.text(self.operator.as_str()), argument_doc))
            } else {
                let argument_doc = self.argument.format(p);
                p.array(p_vec!(p, argument_doc, p.text(self.operator.as_str())))
            }
        })
    }
}

impl<'a> Format<'a> for UnaryExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, UnaryExpression, {
            let mut parts = p.vec();
            parts.push(p.text(p_str!(p, self.operator.as_str())));
            if self.operator.is_keyword() {
                parts.push(p.space());
            }
            parts.push(self.argument.format(p));
            p.array(parts)
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
                group!(p, p.indent(p_vec!(p, p.softline(), doc)), p.softline())
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
            p.array(p_vec!(
                p,
                left_doc,
                p.space(),
                p.text(self.operator.as_str()),
                p.space(),
                right_doc
            ))
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
                group!(p, p.indent(p_vec!(p, p.softline(), doc)), p.softline())
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
        wrap!(p, self, AssignmentExpression, { assignment::print_assignment_expression(p, self) })
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
        array::print_array(p, &Array::ArrayAssignmentTarget(self))
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
        object::print_object_properties(p, ObjectLike::AssignmentTarget(self))
    }
}

impl<'a> Format<'a> for AssignmentTargetWithDefault<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let binding_doc = self.binding.format(p);
        let init_doc = self.init.format(p);
        p.array(p_vec!(p, binding_doc, p.text(" = "), init_doc))
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
        let mut parts = p.vec();
        parts.push(self.binding.format(p));
        if let Some(init) = &self.init {
            parts.push(p.text(" = "));
            parts.push(init.format(p));
        }
        p.array(parts)
    }
}

impl<'a> Format<'a> for AssignmentTargetPropertyProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let name_doc = self.name.format(p);
        let binding_doc = self.binding.format(p);
        p.array(p_vec!(p, name_doc, p.text(": "), binding_doc))
    }
}

impl<'a> Format<'a> for AssignmentTargetRest<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let target_doc = self.target.format(p);
        p.array(p_vec!(p, p.text("..."), target_doc))
    }
}

impl<'a> Format<'a> for SequenceExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, SequenceExpression, {
            let docs =
                self.expressions.iter().map(|expr| expr.format(p)).collect::<std::vec::Vec<_>>();
            group![p, p.array(p.join(Separator::CommaLine, docs))]
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
            let mut parts = p.vec();
            parts.push(p.text("import"));
            parts.push(p.text("("));
            let mut indent_parts = p.vec();
            indent_parts.push(p.softline());
            indent_parts.push(self.source.format(p));
            if !self.arguments.is_empty() {
                for arg in &self.arguments {
                    indent_parts.push(p.text(","));
                    indent_parts.push(p.line());
                    indent_parts.push(arg.format(p));
                }
            }
            parts.push(group!(p, p.indent(indent_parts)));
            parts.push(p.softline());
            parts.push(p.text(")"));

            Doc::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for TemplateLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        template_literal::print_template_literal(p, &TemplateLiteralPrinter::TemplateLiteral(self))
    }
}

impl<'a> Format<'a> for TemplateElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        // TODO: `replaceEndOfLine`
        p.text(p_str!(p, self.value.raw.as_str()))
    }
}

impl<'a> Format<'a> for TaggedTemplateExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, TaggedTemplateExpression, {
            let mut parts = p.vec();

            parts.push(self.tag.format(p));

            if let Some(type_parameters) = &self.type_parameters {
                parts.push(p.text("<"));
                parts.push(type_parameters.format(p));
                parts.push(p.text(">"));
            }

            parts.push(self.quasi.format(p));

            p.array(parts)
        })
    }
}

impl<'a> Format<'a> for Super {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("super")
    }
}

impl<'a> Format<'a> for AwaitExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, AwaitExpression, {
            let mut parts = p.vec();
            parts.push(p.text("await "));
            parts.push(self.argument.format(p));
            p.array(parts)
        })
    }
}

impl<'a> Format<'a> for ChainExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ChainExpression, { self.expression.format(p) })
    }
}

impl<'a> Format<'a> for ChainElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::CallExpression(expr) => expr.format(p),
            Self::TSNonNullExpression(expr) => expr.format(p),
            match_member_expression!(Self) => self.to_member_expression().format(p),
        }
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
        let meta_doc = self.meta.format(p);
        let property_doc = self.property.format(p);
        p.array(p_vec!(p, meta_doc, p.text("."), property_doc))
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

impl<'a> Format<'a> for TSClassImplements<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(self.expression.format(p));

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSTypeAssertion<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let type_annotation_doc = self.type_annotation.format(p);
        let expression_doc = self.expression.format(p);
        p.array(p_vec!(p, p.text("<"), type_annotation_doc, p.text(">"), expression_doc))
    }
}

impl<'a> Format<'a> for TSSatisfiesExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let expression_doc = self.expression.format(p);
        let type_annotation_doc = self.type_annotation.format(p);
        p.array(p_vec!(p, expression_doc, p.text(" satisfies "), type_annotation_doc))
    }
}

impl<'a> Format<'a> for TSInstantiationExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let expression_doc = self.expression.format(p);
        let type_parameters_doc = self.type_parameters.format(p);
        p.array(p_vec!(p, expression_doc, type_parameters_doc))
    }
}

impl<'a> Format<'a> for TSNonNullExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let expression_doc = self.expression.format(p);
        p.array(p_vec!(p, expression_doc, p.text("!")))
    }
}

impl<'a> Format<'a> for JSXIdentifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text(p_str!(p, self.name.as_str()))
    }
}

impl<'a> Format<'a> for JSXMemberExpressionObject<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            JSXMemberExpressionObject::IdentifierReference(it) => it.format(p),
            JSXMemberExpressionObject::MemberExpression(it) => it.format(p),
            JSXMemberExpressionObject::ThisExpression(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for JSXMemberExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let object_doc = self.object.format(p);
        let property_doc = self.property.format(p);
        p.array(p_vec!(p, object_doc, p.text("."), property_doc))
    }
}

impl<'a> Format<'a> for JSXElementName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            JSXElementName::Identifier(it) => it.format(p),
            JSXElementName::IdentifierReference(it) => it.format(p),
            JSXElementName::MemberExpression(it) => it.format(p),
            JSXElementName::NamespacedName(it) => it.format(p),
            JSXElementName::ThisExpression(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for JSXNamespacedName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let namespace_doc = self.namespace.format(p);
        let property_doc = self.property.format(p);
        p.array(p_vec!(p, namespace_doc, p.text(":"), property_doc))
    }
}

impl<'a> Format<'a> for JSXAttributeName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            JSXAttributeName::Identifier(it) => it.format(p),
            JSXAttributeName::NamespacedName(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for JSXAttribute<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(self.name.format(p));

        if let Some(value) = &self.value {
            parts.push(p.text("="));
            parts.push(value.format(p));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for JSXEmptyExpression {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("")
    }
}

impl<'a> Format<'a> for JSXExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            JSXExpression::EmptyExpression(it) => it.format(p),
            match_member_expression!(Self) => self.to_member_expression().format(p),
            JSXExpression::BooleanLiteral(it) => it.format(p),
            JSXExpression::NullLiteral(it) => it.format(p),
            JSXExpression::NumericLiteral(it) => it.format(p),
            JSXExpression::BigIntLiteral(it) => it.format(p),
            JSXExpression::RegExpLiteral(it) => it.format(p),
            JSXExpression::StringLiteral(it) => it.format(p),
            JSXExpression::TemplateLiteral(it) => it.format(p),
            JSXExpression::Identifier(it) => it.format(p),
            JSXExpression::MetaProperty(it) => it.format(p),
            JSXExpression::Super(it) => it.format(p),
            JSXExpression::ArrayExpression(it) => it.format(p),
            JSXExpression::ArrowFunctionExpression(it) => it.format(p),
            JSXExpression::AssignmentExpression(it) => it.format(p),
            JSXExpression::AwaitExpression(it) => it.format(p),
            JSXExpression::BinaryExpression(it) => it.format(p),
            JSXExpression::CallExpression(it) => it.format(p),
            JSXExpression::ChainExpression(it) => it.format(p),
            JSXExpression::ClassExpression(it) => it.format(p),
            JSXExpression::ConditionalExpression(it) => it.format(p),
            JSXExpression::FunctionExpression(it) => it.format(p),
            JSXExpression::ImportExpression(it) => it.format(p),
            JSXExpression::LogicalExpression(it) => it.format(p),
            JSXExpression::NewExpression(it) => it.format(p),
            JSXExpression::ObjectExpression(it) => it.format(p),
            JSXExpression::ParenthesizedExpression(it) => it.format(p),
            JSXExpression::SequenceExpression(it) => it.format(p),
            JSXExpression::TaggedTemplateExpression(it) => it.format(p),
            JSXExpression::ThisExpression(it) => it.format(p),
            JSXExpression::UnaryExpression(it) => it.format(p),
            JSXExpression::UpdateExpression(it) => it.format(p),
            JSXExpression::YieldExpression(it) => it.format(p),
            JSXExpression::PrivateInExpression(it) => it.format(p),
            JSXExpression::JSXElement(it) => it.format(p),
            JSXExpression::JSXFragment(it) => it.format(p),
            JSXExpression::TSAsExpression(it) => it.format(p),
            JSXExpression::TSSatisfiesExpression(it) => it.format(p),
            JSXExpression::TSTypeAssertion(it) => it.format(p),
            JSXExpression::TSNonNullExpression(it) => it.format(p),
            JSXExpression::TSInstantiationExpression(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for JSXExpressionContainer<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let expression_doc = self.expression.format(p);
        p.array(p_vec!(p, p.text("{"), expression_doc, p.text("}")))
    }
}

impl<'a> Format<'a> for JSXAttributeValue<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            JSXAttributeValue::Element(it) => it.format(p),
            JSXAttributeValue::ExpressionContainer(it) => it.format(p),
            JSXAttributeValue::Fragment(it) => it.format(p),
            JSXAttributeValue::StringLiteral(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for JSXSpreadAttribute<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let argument_doc = self.argument.format(p);
        p.array(p_vec!(p, p.text("..."), argument_doc))
    }
}

impl<'a> Format<'a> for JSXAttributeItem<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            JSXAttributeItem::Attribute(it) => it.format(p),
            JSXAttributeItem::SpreadAttribute(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for JSXOpeningElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(p.text("<"));
        parts.push(self.name.format(p));

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        for attribute in &self.attributes {
            parts.push(p.space());
            parts.push(attribute.format(p));
        }

        if self.self_closing {
            parts.push(p.space());
            parts.push(p.text("/"));
        }

        parts.push(p.text(">"));

        p.array(parts)
    }
}

impl<'a> Format<'a> for JSXClosingElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let name_doc = self.name.format(p);
        p.array(p_vec!(p, p.text("</"), name_doc, p.text(">")))
    }
}

impl<'a> Format<'a> for JSXElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(self.opening_element.format(p));

        for child in &self.children {
            parts.push(child.format(p));
        }

        if let Some(closing_element) = &self.closing_element {
            parts.push(closing_element.format(p));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for JSXOpeningFragment {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("<>")
    }
}

impl<'a> Format<'a> for JSXClosingFragment {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text("</>")
    }
}

impl<'a> Format<'a> for JSXText<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.text(p_str!(p, self.value.as_str()))
    }
}

impl<'a> Format<'a> for JSXSpreadChild<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let expression_doc = self.expression.format(p);
        p.array(p_vec!(p, p.text("..."), expression_doc))
    }
}

impl<'a> Format<'a> for JSXChild<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            JSXChild::Element(it) => it.format(p),
            JSXChild::ExpressionContainer(it) => it.format(p),
            JSXChild::Fragment(it) => it.format(p),
            JSXChild::Spread(it) => it.format(p),
            JSXChild::Text(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for JSXFragment<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(self.opening_fragment.format(p));

        for child in &self.children {
            parts.push(child.format(p));
        }

        parts.push(self.closing_fragment.format(p));

        p.array(parts)
    }
}

impl<'a> Format<'a> for StaticBlock<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, StaticBlock, {
            let block_doc = block::print_block(p, &self.body, None);
            p.array(p_vec!(p, p.text("static "), block_doc))
        })
    }
}

impl<'a> Format<'a> for MethodDefinition<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, MethodDefinition, { function::print_method(p, self) })
    }
}

impl<'a> Format<'a> for PropertyDefinition<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, PropertyDefinition, {
            class::print_class_property(p, &class::ClassMemberish::PropertyDefinition(self))
        })
    }
}

impl<'a> Format<'a> for AccessorProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        class::print_class_property(p, &class::ClassMemberish::AccessorProperty(self))
    }
}

impl<'a> Format<'a> for PrivateIdentifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(p.text("#"));
        parts.push(p.text(p_str!(p, self.name.as_str())));
        p.array(parts)
    }
}

impl<'a> Format<'a> for BindingPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(match self.kind {
            BindingPatternKind::BindingIdentifier(ref ident) => ident.format(p),
            BindingPatternKind::ObjectPattern(ref pattern) => pattern.format(p),
            BindingPatternKind::ArrayPattern(ref pattern) => pattern.format(p),
            BindingPatternKind::AssignmentPattern(ref pattern) => pattern.format(p),
        });

        if self.optional {
            parts.push(p.text("?"));
        }

        if let Some(typ) = &self.type_annotation {
            let type_annotation_doc = typ.type_annotation.format(p);
            parts.push(p.array(p_vec!(p, p.text(": "), type_annotation_doc)));
        }
        p.array(parts)
    }
}

impl<'a> Format<'a> for ObjectPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ObjectPattern, {
            object::print_object_properties(p, ObjectLike::Pattern(self))
        })
    }
}

impl<'a> Format<'a> for BindingProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        if self.shorthand {
            self.value.format(p)
        } else {
            group!(p, self.key.format(p), p.text(": "), self.value.format(p))
        }
    }
}

impl<'a> Format<'a> for BindingRestElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let argument_doc = self.argument.format(p);
        p.array(p_vec!(p, p.text("..."), argument_doc))
    }
}

impl<'a> Format<'a> for ArrayPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ArrayPattern, { array::print_array(p, &Array::ArrayPattern(self)) })
    }
}

impl<'a> Format<'a> for AssignmentPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, AssignmentPattern, {
            let left_doc = self.left.format(p);
            let right_doc = self.right.format(p);
            p.array(p_vec!(p, left_doc, p.text(" = "), right_doc))
        })
    }
}

impl<'a> Format<'a> for RegExpFlags {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut string = std::vec::Vec::with_capacity(self.iter().count());
        if self.contains(Self::D) {
            string.push('d');
        }
        if self.contains(Self::G) {
            string.push('g');
        }
        if self.contains(Self::I) {
            string.push('i');
        }
        if self.contains(Self::M) {
            string.push('m');
        }
        if self.contains(Self::S) {
            string.push('s');
        }
        if self.contains(Self::U) {
            string.push('u');
        }
        if self.contains(Self::V) {
            string.push('v');
        }
        if self.contains(Self::Y) {
            string.push('y');
        }
        let sorted = &string.iter().collect::<String>();
        p.text(p_str!(p, &sorted))
    }
}

impl<'a> Format<'a> for TSIndexSignature<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        if self.readonly {
            parts.push(p.text("readonly "));
        }

        parts.push(p.text("["));
        for param in &self.parameters {
            parts.push(param.format(p));
        }
        parts.push(p.text("]: "));
        parts.push(self.type_annotation.type_annotation.format(p));

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSIndexSignatureName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let type_annotation_doc = self.type_annotation.type_annotation.format(p);
        p.array(p_vec!(p, p.text(p_str!(p, self.name.as_str())), p.text(": "), type_annotation_doc))
    }
}

impl<'a> Format<'a> for TSPropertySignature<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        if self.readonly {
            parts.push(p.text("readonly "));
        }
        parts.push(self.key.format(p));
        if let Some(ty) = &self.type_annotation {
            if self.optional {
                parts.push(p.text("?"));
            }
            parts.push(p.text(":"));
            parts.push(p.space());
            parts.push(ty.type_annotation.format(p));
        }
        p.array(parts)
    }
}

impl<'a> Format<'a> for TSCallSignatureDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        parts.push(self.params.format(p));

        if let Some(return_type) = &self.return_type {
            parts.push(p.text(": "));
            parts.push(return_type.type_annotation.format(p));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSConstructSignatureDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(p.text("new "));

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        parts.push(self.params.format(p));

        if let Some(return_type) = &self.return_type {
            parts.push(p.text(": "));
            parts.push(return_type.type_annotation.format(p));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSMethodSignature<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        if self.computed {
            parts.push(p.text("["));
        }

        parts.push(self.key.format(p));

        if self.computed {
            parts.push(p.text("]"));
        }

        if self.optional {
            parts.push(p.text("?"));
        }

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        parts.push(self.params.format(p));

        if let Some(return_type) = &self.return_type {
            parts.push(p.text(": "));
            parts.push(return_type.type_annotation.format(p));
        }

        p.array(parts)
    }
}

impl<'a> Format<'a> for TSSignature<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSSignature::TSIndexSignature(it) => it.format(p),
            TSSignature::TSPropertySignature(it) => it.format(p),
            TSSignature::TSCallSignatureDeclaration(it) => it.format(p),
            TSSignature::TSConstructSignatureDeclaration(it) => it.format(p),
            TSSignature::TSMethodSignature(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for TSAsExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let expression_doc = self.expression.format(p);
        let type_annotation_doc = self.type_annotation.format(p);
        p.array(p_vec!(p, expression_doc, p.text(" as "), type_annotation_doc))
    }
}
