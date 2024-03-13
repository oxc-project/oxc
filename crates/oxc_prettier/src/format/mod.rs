//! Formatting logic
//!
//! References:
//! * <https://github.com/prettier/prettier/blob/main/src/language-js/print/estree.js>

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

use oxc_allocator::{Box, Vec};
use oxc_ast::{ast::*, AstKind};
use oxc_span::GetSpan;
use oxc_syntax::identifier::is_identifier_name;

use crate::{
    array,
    doc::{Doc, DocBuilder, Group, Separator},
    format, group, hardline, indent, line, softline, space, ss, string, wrap, Prettier,
};

use self::{array::Array, object::ObjectLike, template_literal::TemplateLiteralPrinter};

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
        p.enter_node(AstKind::Program(p.alloc(self)));
        let mut parts = p.vec();
        if let Some(hashbang) = &self.hashbang {
            parts.push(hashbang.format(p));
            let c = p.source_text[..hashbang.span.end as usize].chars().last().unwrap();
            if p.is_next_line_empty_after_index(hashbang.span.end - c.len_utf8() as u32) {
                parts.extend(hardline!());
            }
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
        p.leave_node();
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for Hashbang<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str(self.span.source_text(p.source_text))
    }
}

impl<'a> Format<'a> for Directive<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(Doc::Str(string::print_string(
            p,
            self.directive.as_str(),
            p.options.single_quote,
        )));
        if let Some(semi) = p.semi() {
            parts.push(semi);
        }
        parts.extend(hardline!());
        Doc::Array(parts)
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
            Self::ModuleDeclaration(decl) => decl.format(p),
            Self::ReturnStatement(stmt) => stmt.format(p),
            Self::SwitchStatement(stmt) => stmt.format(p),
            Self::ThrowStatement(stmt) => stmt.format(p),
            Self::TryStatement(stmt) => stmt.format(p),
            Self::WhileStatement(stmt) => stmt.format(p),
            Self::WithStatement(stmt) => stmt.format(p),
            Self::Declaration(decl) => decl.format(p),
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
            Doc::Array(parts)
        })
    }
}

impl<'a> Format<'a> for EmptyStatement {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("")
    }
}

impl<'a> Format<'a> for IfStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, IfStatement, {
            let mut parts = p.vec();

            let test_doc = format!(p, self.test);
            let consequent = format!(p, self.consequent);
            let consequent = misc::adjust_clause(p, &self.consequent, consequent, false);

            let opening = group![
                p,
                ss!("if ("),
                group!(p, indent!(p, softline!(), test_doc), softline!()),
                ss!(")"),
                consequent
            ];
            parts.push(opening);

            if let Some(alternate) = &self.alternate {
                let else_on_same_line = matches!(alternate, Statement::BlockStatement(_));
                if else_on_same_line {
                    parts.push(space!());
                } else {
                    parts.extend(hardline!());
                }
                parts.push(ss!("else"));
                let alternate_doc = format!(p, alternate);
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

            Doc::Array(parts)
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
            let body = format!(p, self.body);
            let body = misc::adjust_clause(p, &self.body, body, false);

            if self.init.is_none() && self.test.is_none() && self.update.is_none() {
                return group![p, ss!("for (;;)"), body];
            }

            let parts_head = {
                let mut parts_head = p.vec();
                parts_head.push(softline!());
                if let Some(init) = &self.init {
                    parts_head.push(format!(p, init));
                }
                parts_head.push(ss!(";"));
                parts_head.push(line!());
                if let Some(init) = &self.test {
                    parts_head.push(format!(p, init));
                }
                parts_head.push(ss!(";"));
                parts_head.push(line!());
                if let Some(init) = &self.update {
                    parts_head.push(format!(p, init));
                }
                Doc::Indent(parts_head)
            };

            group![p, ss!("for ("), group![p, parts_head, softline!()], ss!(")"), body]
        })
    }
}

impl<'a> Format<'a> for ForStatementInit<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            ForStatementInit::VariableDeclaration(v) => v.format(p),
            ForStatementInit::Expression(v) => v.format(p),
            ForStatementInit::UsingDeclaration(v) => v.format(p),
        }
    }
}

impl<'a> Format<'a> for ForInStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ForInStatement, {
            let mut parts = p.vec();
            parts.push(ss!("for ("));
            parts.push(format!(p, self.left));
            parts.push(ss!(" in "));
            parts.push(format!(p, self.right));
            parts.push(ss!(")"));
            let body = format!(p, self.body);
            parts.push(misc::adjust_clause(p, &self.body, body, false));
            Doc::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for ForOfStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ForOfStatement, {
            let mut parts = p.vec();
            parts.push(ss!("for"));
            if self.r#await {
                parts.push(ss!(" await"));
            }
            parts.push(ss!(" ("));
            parts.push(format!(p, self.left));
            parts.push(ss!(" of "));
            parts.push(format!(p, self.right));
            parts.push(ss!(")"));
            let body = format!(p, self.body);
            parts.push(misc::adjust_clause(p, &self.body, body, false));
            Doc::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for ForStatementLeft<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            ForStatementLeft::VariableDeclaration(v) => v.format(p),
            ForStatementLeft::AssignmentTarget(v) => v.format(p),
            ForStatementLeft::UsingDeclaration(v) => v.format(p),
        }
    }
}

impl<'a> Format<'a> for WhileStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, WhileStatement, {
            let mut parts = p.vec();

            parts.push(ss!("while ("));
            parts.push(group!(p, indent!(p, softline!(), format!(p, self.test)), softline!()));
            parts.push(ss!(")"));

            let body = format!(p, self.body);
            parts.push(misc::adjust_clause(p, &self.body, body, false));

            Doc::Group(Group::new(parts))
        })
    }
}

impl<'a> Format<'a> for DoWhileStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, DoWhileStatement, {
            let mut parts = p.vec();

            let clause = format!(p, self.body);
            let clause = misc::adjust_clause(p, &self.body, clause, false);
            let do_body = group!(p, ss!("do"), clause);

            parts.push(do_body);

            if matches!(self.body, Statement::BlockStatement(_)) {
                parts.push(space!());
            } else {
                parts.extend(hardline!());
            }

            parts.push(ss!("while ("));
            parts.push(group!(p, indent!(p, softline!(), format!(p, self.test)), softline!()));
            parts.push(ss!(")"));
            if let Some(semi) = p.semi() {
                parts.push(semi);
            }

            Doc::Array(parts)
        })
    }
}

impl<'a> Format<'a> for ContinueStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(ss!("continue"));

        if let Some(label) = &self.label {
            parts.push(space!());
            parts.push(format!(p, label));
        }

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for BreakStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(ss!("break"));

        if let Some(label) = &self.label {
            parts.push(space!());
            parts.push(format!(p, label));
        }

        if p.options.semi {
            parts.push(Doc::Str(";"));
        }

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for SwitchStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, SwitchStatement, {
            let mut parts = p.vec();

            let mut header_parts = p.vec();

            header_parts.push(ss!("switch ("));

            header_parts.push(indent!(p, softline!(), format!(p, self.discriminant)));

            header_parts.push(softline!());
            header_parts.push(ss!(")"));

            parts.push(Doc::Group(Group::new(header_parts)));

            parts.push(ss!(" {"));

            let mut cases_parts = p.vec();
            let len = self.cases.len();
            for (i, case) in self.cases.iter().enumerate() {
                cases_parts.push({
                    let mut parts = p.vec();
                    parts.extend(hardline!());
                    parts.push(format!(p, case));
                    Doc::Indent(parts)
                });
                if i != len - 1 && p.is_next_line_empty(case.span) {
                    cases_parts.extend(hardline!());
                }
            }
            parts.extend(cases_parts);

            parts.extend(hardline!());
            parts.push(ss!("}"));

            Doc::Array(parts)
        })
    }
}

impl<'a> Format<'a> for SwitchCase<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        if let Some(test) = &self.test {
            parts.push(ss!("case "));
            parts.push(format!(p, test));
            parts.push(ss!(":"));
        } else {
            parts.push(ss!("default:"));
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
                    consequent_parts.extend(hardline!());
                }
            }

            if is_only_one_block_statement {
                consequent_parts.push(space!());
            } else {
                consequent_parts.extend(hardline!());
            }
            consequent_parts.push(format!(p, stmt));
        }

        if !consequent_parts.is_empty() {
            if is_only_one_block_statement {
                parts.extend(consequent_parts);
            } else {
                parts.push(indent!(p, Doc::Group(Group::new(consequent_parts))));
            }
        }

        Doc::Array(parts)
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
            return array!(p, self.label.format(p), ss!(":;"));
        }

        array!(p, self.label.format(p), ss!(": "), format!(p, self.body))
    }
}

impl<'a> Format<'a> for TryStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.enter_node(AstKind::TryStatement(p.alloc(self)));
        let mut parts = p.vec();
        parts.push(ss!("try "));
        parts.push(format!(p, self.block));
        if let Some(handler) = &self.handler {
            parts.push(space!());
            parts.push(format!(p, handler));
        }
        if let Some(finalizer) = &self.finalizer {
            parts.push(ss!(" finally "));
            parts.push(format!(p, finalizer));
        }
        p.leave_node();
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for CatchClause<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, CatchClause, {
            let mut parts = p.vec();

            parts.push(ss!("catch "));
            if let Some(param) = &self.param {
                parts.push(ss!("("));
                parts.push(format!(p, param));
                parts.push(ss!(") "));
            }
            parts.push(format!(p, self.body));

            Doc::Array(parts)
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
            ss!("with ("),
            format!(p, self.object),
            ss!(")"),
            misc::adjust_clause(p, &self.body, body_doc, false)
        ]
    }
}

impl<'a> Format<'a> for DebuggerStatement {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(Doc::Str("debugger"));

        if p.options.semi {
            parts.push(Doc::Str(";"));
        }

        Doc::Array(parts)
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
            Self::UsingDeclaration(decl) => decl.format(p),
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
            parts.push(ss!(kind));
            parts.push(space!());

            let is_hardline = !p.parent_kind().is_iteration_statement()
                && self.declarations.iter().all(|decl| decl.init.is_some());
            let decls_len = self.declarations.len();
            parts.extend(self.declarations.iter().enumerate().map(|(i, decl)| {
                if decls_len > 1 {
                    let mut d_parts = p.vec();
                    if i != 0 {
                        d_parts.push(p.str(","));
                        if is_hardline {
                            d_parts.extend(hardline!());
                        } else {
                            d_parts.push(line!());
                        }
                    }
                    d_parts.push(decl.format(p));
                    Doc::Indent(d_parts)
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

impl<'a> Format<'a> for UsingDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSTypeAliasDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(ss!("type "));
        parts.push(format!(p, self.id));
        parts.push(ss!(" = "));
        parts.push(format!(p, self.type_annotation));

        if let Some(semi) = p.semi() {
            parts.push(semi);
        }

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for TSType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSType::TSAnyKeyword(v) => v.format(p),
            TSType::TSBigIntKeyword(v) => v.format(p),
            TSType::TSBooleanKeyword(v) => v.format(p),
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
            TSType::JSDocNullableType(v) => v.format(p),
            TSType::JSDocUnknownType(v) => v.format(p),
        }
    }
}

impl<'a> Format<'a> for TSAnyKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("any")
    }
}

impl<'a> Format<'a> for TSBigIntKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("bigint")
    }
}

impl<'a> Format<'a> for TSBooleanKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("boolean")
    }
}

impl<'a> Format<'a> for TSNeverKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("never")
    }
}

impl<'a> Format<'a> for TSNullKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("null")
    }
}

impl<'a> Format<'a> for TSNumberKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("number")
    }
}

impl<'a> Format<'a> for TSObjectKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("object")
    }
}

impl<'a> Format<'a> for TSStringKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("string")
    }
}

impl<'a> Format<'a> for TSSymbolKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("symbol")
    }
}

impl<'a> Format<'a> for TSThisType {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("this")
    }
}

impl<'a> Format<'a> for TSUndefinedKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("undefined")
    }
}

impl<'a> Format<'a> for TSUnknownKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("unknown")
    }
}

impl<'a> Format<'a> for TSVoidKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("void")
    }
}

impl<'a> Format<'a> for TSArrayType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSConditionalType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSConstructorType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSFunctionType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSImportType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSIndexedAccessType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(format!(p, self.object_type));
        parts.push(ss!("["));
        parts.push(format!(p, self.index_type));
        parts.push(ss!("]"));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for TSInferType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array!(p, ss!("infer "), format!(p, self.type_parameter))
    }
}

impl<'a> Format<'a> for TSIntersectionType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSLiteralType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match &self.literal {
            TSLiteral::BooleanLiteral(v) => v.format(p),
            TSLiteral::NullLiteral(v) => v.format(p),
            TSLiteral::NumericLiteral(v) => v.format(p),
            TSLiteral::BigintLiteral(v) => v.format(p),
            TSLiteral::RegExpLiteral(v) => v.format(p),
            TSLiteral::StringLiteral(v) => v.format(p),
            TSLiteral::TemplateLiteral(v) => v.format(p),
            TSLiteral::UnaryExpression(v) => v.format(p),
        }
    }
}

impl<'a> Format<'a> for TSMappedType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSNamedTupleMember<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSQualifiedName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
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
        line!()
    }
}

impl<'a> Format<'a> for TSTypeOperator<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSTypePredicate<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSTypeQuery<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSTypeReference<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSUnionType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSDocNullableType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSDocUnknownType {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSInterfaceDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSEnumDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSModuleDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSImportEqualsDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(ss!("import "));

        if self.import_kind == ImportOrExportKind::Type {
            parts.push(ss!("type "));
        }

        parts.push(format!(p, self.id));
        parts.push(ss!(" = "));
        parts.push(format!(p, self.module_reference));

        if let Some(semi) = p.semi() {
            parts.push(semi);
        }
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for TSModuleReference<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSModuleReference::TypeName(v) => v.format(p),
            TSModuleReference::ExternalModuleReference(v) => v.format(p),
        }
    }
}

impl<'a> Format<'a> for TSTypeName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSExternalModuleReference<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array!(p, ss!("require("), format!(p, self.expression), ss!(")"))
    }
}

impl<'a> Format<'a> for TSTypeParameter<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSTypeParameterDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSTypeParameterInstantiation<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSTupleElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
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
        parts.push(ss!("import"));
        if self.import_kind.is_type() {
            parts.push(ss!(" type"));
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
            parts.push(ss!(" from"));
        }
        parts.push(space!());
        parts.push(self.source.format(p));

        if let Some(with_clause) = &self.with_clause {
            parts.push(space!());
            parts.push(with_clause.format(p));
        }

        if let Some(semi) = p.semi() {
            parts.push(semi);
        }
        Doc::Array(parts)
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
        if self.imported.span() == self.local.span {
            self.local.format(p)
        } else {
            array![p, self.imported.format(p), ss!(" as "), self.local.format(p)]
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
        array!(p, ss!("* as "), self.local.format(p))
    }
}

impl<'a> Format<'a> for WithClause<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array!(
            p,
            self.attributes_keyword.format(p),
            space!(),
            object::print_object_properties(p, ObjectLike::WithClause(self))
        )
    }
}

impl<'a> Format<'a> for ImportAttribute<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array!(p, self.key.format(p), ss!(": "), self.value.format(p))
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
            parts.push(space!());
            parts.push(decl.format(p));
        } else {
            parts.push(module::print_module_specifiers(
                p,
                &self.specifiers,
                /* include_default */ false,
                /* include_namespace */ false,
            ));
        }
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for TSExportAssignment<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array!(p, ss!(" = "), self.expression.format(p))
    }
}

impl<'a> Format<'a> for TSNamespaceExportDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for ExportSpecifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        if self.exported.span() == self.local.span() {
            self.local.format(p)
        } else {
            array![p, self.local.format(p), ss!(" as "), self.exported.format(p)]
        }
    }
}

impl<'a> Format<'a> for ModuleExportName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::Identifier(ident) => ident.format(p),
            Self::StringLiteral(literal) => literal.format(p),
        }
    }
}

impl<'a> Format<'a> for ExportAllDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(ss!(" *"));
        if let Some(exported) = &self.exported {
            parts.push(ss!(" as "));
            parts.push(exported.format(p));
        }
        Doc::Array(parts)
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
            Self::Expression(expr) => expr.format(p),
            Self::FunctionDeclaration(decl) => decl.format(p),
            Self::ClassDeclaration(decl) => decl.format(p),
            Self::TSInterfaceDeclaration(decl) => decl.format(p),
            Self::TSEnumDeclaration(decl) => decl.format(p),
        }
    }
}

impl<'a> Format<'a> for Expression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::BooleanLiteral(lit) => lit.format(p),
            Self::NullLiteral(lit) => lit.format(p),
            Self::NumericLiteral(lit) => lit.format(p),
            Self::BigintLiteral(lit) => lit.format(p),
            Self::RegExpLiteral(lit) => lit.format(p),
            Self::StringLiteral(lit) => lit.format(p),
            Self::Identifier(ident) => ident.format(p),
            Self::ThisExpression(expr) => expr.format(p),
            Self::MemberExpression(expr) => expr.format(p),
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
            Self::TSAsExpression(expr) => expr.expression.format(p),
            Self::TSSatisfiesExpression(expr) => expr.expression.format(p),
            Self::TSTypeAssertion(expr) => expr.expression.format(p),
            Self::TSNonNullExpression(expr) => expr.expression.format(p),
            Self::TSInstantiationExpression(expr) => expr.expression.format(p),
        }
    }
}

impl<'a> Format<'a> for IdentifierReference<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, IdentifierReference, { p.str(self.name.as_str()) })
    }
}

impl<'a> Format<'a> for IdentifierName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.str(self.name.as_str())
    }
}

impl<'a> Format<'a> for BindingIdentifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, BindingIdentifier, { p.str(self.name.as_str()) })
    }
}

impl<'a> Format<'a> for LabelIdentifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.str(self.name.as_str())
    }
}

impl<'a> Format<'a> for BooleanLiteral {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str(if self.value { "true" } else { "false" })
    }
}

impl<'a> Format<'a> for NullLiteral {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("null")
    }
}

impl<'a> Format<'a> for NumericLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, NumericLiteral, {
            // See https://github.com/prettier/prettier/blob/main/src/utils/print-number.js
            // Perf: the regexes from prettier code above are ported to manual search for performance reasons.
            let raw = self.span.source_text(p.source_text);
            let mut string = Cow::Borrowed(raw);

            if string.contains(|c: char| c.is_ascii_uppercase()) {
                string = Cow::Owned(string.to_ascii_lowercase());
            }

            // Remove unnecessary plus and zeroes from scientific notation.
            if let Some((head, tail)) = string.split_once('e') {
                let negative = if tail.starts_with('-') { "-" } else { "" };
                let trimmed =
                    tail.trim_start_matches(|c| c == '+' || c == '-').trim_start_matches('0');
                if trimmed.starts_with(|c: char| c.is_ascii_digit()) {
                    string = Cow::Owned(std::format!("{head}e{negative}{trimmed}"));
                }
            }

            // Remove unnecessary scientific notation (1e0).
            if let Some((head, tail)) = string.split_once('e') {
                if tail
                    .trim_start_matches(|c| c == '+' || c == '-')
                    .trim_start_matches('0')
                    .is_empty()
                {
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

            p.str(&string)
        })
    }
}

impl<'a> Format<'a> for BigIntLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let text = self.span.source_text(p.source_text);
        // Perf: avoid a memory allocation from `to_ascii_lowercase`.
        if text.contains(|c: char| c.is_lowercase()) {
            p.str(&text.to_ascii_lowercase())
        } else {
            Doc::Str(text)
        }
    }
}

impl<'a> Format<'a> for RegExpLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(ss!("/"));
        parts.push(p.str(self.regex.pattern.as_str()));
        parts.push(ss!("/"));
        parts.push(format!(p, self.regex.flags));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for StringLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, StringLiteral, {
            let raw = &p.source_text[(self.span.start + 1) as usize..(self.span.end - 1) as usize];
            // TODO: implement `makeString` from prettier/src/utils/print-string.js
            Doc::Str(string::print_string(p, raw, p.options.single_quote))
        })
    }
}

impl<'a> Format<'a> for ThisExpression {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("this")
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
        parts.push(format!(p, self.object));
        if self.optional {
            parts.push(ss!("?."));
        }
        parts.push(ss!("["));
        parts.push(format!(p, self.expression));
        parts.push(ss!("]"));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for StaticMemberExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(format!(p, self.object));
        if self.optional {
            parts.push(ss!("?"));
        }
        parts.push(ss!("."));
        parts.push(format!(p, self.property));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for PrivateFieldExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(format!(p, self.object));
        parts.push(if self.optional { ss!("?.") } else { ss!(".") });
        parts.push(format!(p, self.field));
        Doc::Array(parts)
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
            Self::Expression(expr) => expr.format(p),
            Self::SpreadElement(expr) => expr.format(p),
        }
    }
}

impl<'a> Format<'a> for ArrayExpressionElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::SpreadElement(expr) => expr.format(p),
            Self::Expression(expr) => expr.format(p),
            Self::Elision(elision) => Doc::Str(""),
        }
    }
}

impl<'a> Format<'a> for SpreadElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, SpreadElement, { array![p, ss!("..."), format!(p, self.argument)] })
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
                        parts.push(ss!("get "));
                        method = true;
                    }
                    PropertyKind::Set => {
                        parts.push(ss!("set "));
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
                    parts.push(format!(p, self.key));
                    parts.push(ss!(": "));
                    parts.push(format!(p, self.value));
                }
                return Doc::Group(Group::new(parts));
            }

            if self.shorthand {
                return self.value.format(p);
            }

            let left_doc = if self.computed {
                array!(p, ss!("["), format!(p, self.key), ss!("]"))
            } else {
                format!(p, self.key)
            };

            assignment::print_assignment(
                p,
                assignment::AssignmentLikeNode::ObjectProperty(self),
                left_doc,
                ss!(":"),
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
            parts.push(ss!("["));
            let doc = match &self {
                PropertyKey::Identifier(ident) => ident.format(p),
                PropertyKey::PrivateIdentifier(ident) => ident.format(p),
                PropertyKey::Expression(expr) => expr.format(p),
            };
            parts.push(doc);
            parts.push(ss!("]"));
            return Doc::Array(parts);
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
                PropertyKey::Identifier(ident) => {
                    if need_quote {
                        Doc::Str(string::print_string(p, &ident.name, p.options.single_quote))
                    } else {
                        ident.format(p)
                    }
                }
                PropertyKey::PrivateIdentifier(ident) => ident.format(p),
                PropertyKey::Expression(expr) => match expr {
                    Expression::StringLiteral(literal) => {
                        // This does not pass quotes/objects.js
                        // because prettier uses the function `isEs5IdentifierName` based on unicode version 3,
                        // but `is_identifier_name` uses the latest unicode version.
                        if is_identifier_name(literal.value.as_str())
                            && (p.options.quote_props.as_needed()
                                || (p.options.quote_props.consistent()/* && !needsQuoteProps.get(parent) */))
                        {
                            string!(p, literal.value.as_str())
                        } else {
                            Doc::Str(string::print_string(
                                p,
                                literal.value.as_str(),
                                p.options.single_quote,
                            ))
                        }
                    }
                    Expression::NumericLiteral(literal) => {
                        if need_quote {
                            Doc::Str(string::print_string(p, literal.raw, p.options.single_quote))
                        } else {
                            literal.format(p)
                        }
                    }
                    Expression::Identifier(ident) => {
                        array!(p, ss!("["), ident.format(p), ss!("]"))
                    }
                    _ => expr.format(p),
                },
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
            parts.push(ss!("yield"));
            if self.delegate {
                parts.push(ss!("*"));
            }
            if let Some(argument) = &self.argument {
                parts.push(space!());
                parts.push(format!(p, argument));
            }
            Doc::Array(parts)
        })
    }
}

impl<'a> Format<'a> for UpdateExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, UpdateExpression, {
            if self.prefix {
                array![p, ss!(self.operator.as_str()), format!(p, self.argument)]
            } else {
                array![p, format!(p, self.argument), ss!(self.operator.as_str())]
            }
        })
    }
}

impl<'a> Format<'a> for UnaryExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, UnaryExpression, {
            let mut parts = p.vec();
            parts.push(string!(p, self.operator.as_str()));
            if self.operator.is_keyword() {
                parts.push(space!());
            }
            parts.push(format!(p, self.argument));
            Doc::Array(parts)
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
                group!(p, indent!(p, softline!(), doc), softline!())
            } else {
                doc
            }
        })
    }
}

impl<'a> Format<'a> for PrivateInExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, PrivateInExpression, {
            array![
                p,
                format!(p, self.left),
                space!(),
                ss!(self.operator.as_str()),
                space!(),
                format!(p, self.right)
            ]
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
                group!(p, indent!(p, softline!(), doc), softline!())
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
            Self::SimpleAssignmentTarget(target) => target.format(p),
            Self::AssignmentTargetPattern(pat) => pat.format(p),
        }
    }
}

impl<'a> Format<'a> for SimpleAssignmentTarget<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.format(p),
            Self::MemberAssignmentTarget(member_expr) => member_expr.format(p),
            Self::TSAsExpression(expr) => expr.expression.format(p),
            Self::TSSatisfiesExpression(expr) => expr.expression.format(p),
            Self::TSNonNullExpression(expr) => expr.expression.format(p),
            Self::TSTypeAssertion(expr) => expr.expression.format(p),
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
            AssignmentTargetMaybeDefault::AssignmentTarget(v) => v.format(p),
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
        array!(p, format!(p, self.binding), ss!(" = "), format!(p, self.init))
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
            parts.push(ss!(" = "));
            parts.push(init.format(p));
        }
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for AssignmentTargetPropertyProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(self.name.format(p));
        parts.push(ss!(": "));
        parts.push(self.binding.format(p));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for AssignmentTargetRest<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array![p, ss!("..."), self.target.format(p)]
    }
}

impl<'a> Format<'a> for SequenceExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, SequenceExpression, {
            let docs =
                self.expressions.iter().map(|expr| expr.format(p)).collect::<std::vec::Vec<_>>();
            group![p, Doc::Array(p.join(Separator::CommaLine, docs))]
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
            parts.push(ss!("import"));
            parts.push(ss!("("));
            let mut indent_parts = p.vec();
            indent_parts.push(softline!());
            indent_parts.push(format!(p, self.source));
            if !self.arguments.is_empty() {
                for arg in &self.arguments {
                    indent_parts.push(ss!(","));
                    indent_parts.push(line!());
                    indent_parts.push(format!(p, arg));
                }
            }
            parts.push(group!(p, Doc::Indent(indent_parts)));
            parts.push(softline!());
            parts.push(ss!(")"));

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
        p.str(self.value.raw.as_str())
    }
}

impl<'a> Format<'a> for TaggedTemplateExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, TaggedTemplateExpression, {
            let mut parts = p.vec();

            parts.push(format!(p, self.tag));

            if let Some(type_parameters) = &self.type_parameters {
                parts.push(string!(p, "<"));
                parts.push(format!(p, type_parameters));
                parts.push(string!(p, ">"));
            }

            parts.push(format!(p, self.quasi));

            Doc::Array(parts)
        })
    }
}

impl<'a> Format<'a> for Super {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("super")
    }
}

impl<'a> Format<'a> for AwaitExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, AwaitExpression, {
            let mut parts = p.vec();
            parts.push(ss!("await "));
            parts.push(format!(p, self.argument));
            Doc::Array(parts)
        })
    }
}

impl<'a> Format<'a> for ChainExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, ChainExpression, { format!(p, self.expression) })
    }
}

impl<'a> Format<'a> for ChainElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::CallExpression(expr) => expr.format(p),
            Self::MemberExpression(expr) => expr.format(p),
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
        array![p, format!(p, self.meta), ss!("."), format!(p, self.property)]
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

impl<'a> Format<'a> for JSXIdentifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXMemberExpressionObject<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXMemberExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXElementName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXNamespacedName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXAttributeName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXAttribute<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXEmptyExpression {
    fn format(&self, _: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXExpressionContainer<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXAttributeValue<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXSpreadAttribute<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXAttributeItem<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXOpeningElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXClosingElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXOpeningFragment {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXClosingFragment {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXText<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXSpreadChild<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXChild<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSXFragment<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for StaticBlock<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, StaticBlock, {
            array![p, ss!("static "), block::print_block(p, &self.body, None)]
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
        parts.push(ss!("#"));
        parts.push(p.str(self.name.as_str()));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for BindingPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self.kind {
            BindingPatternKind::BindingIdentifier(ref ident) => ident.format(p),
            BindingPatternKind::ObjectPattern(ref pattern) => pattern.format(p),
            BindingPatternKind::ArrayPattern(ref pattern) => pattern.format(p),
            BindingPatternKind::AssignmentPattern(ref pattern) => pattern.format(p),
        }
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
            group!(p, format!(p, self.key), ss!(": "), format!(p, self.value))
        }
    }
}

impl<'a> Format<'a> for BindingRestElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array!(p, ss!("..."), format!(p, self.argument))
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
            array![p, format!(p, self.left), ss!(" = "), format!(p, self.right)]
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
        p.str(&string.iter().collect::<String>())
    }
}

impl<'a> Format<'a> for TSIndexSignature<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}
