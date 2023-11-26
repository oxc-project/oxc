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
mod call_expression;
mod class;
mod function;
mod function_parameters;
mod misc;
mod module;
mod object;
mod statement;
mod string;
mod template_literal;
mod ternary;

use std::borrow::Cow;

use oxc_allocator::{Box, Vec};
use oxc_ast::{ast::*, AstKind};
use oxc_span::{GetSpan, Span};

use crate::{
    array,
    doc::{Doc, DocBuilder, Group, Separator},
    enter, format, group, hardline, indent, line, softline, ss, string, Prettier,
};

use self::{
    array::Array,
    binaryish::{BinaryishLeft, BinaryishOperator},
    object::ObjectLike,
    template_literal::TemplateLiteralPrinter,
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
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        if let Some(hashbang) = &self.hashbang {
            parts.push(enter!(p, Hashbang, hashbang));
            if p.is_next_line_empty(hashbang.span.end - 1) {
                parts.push(hardline!());
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
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for Hashbang {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str(self.span.source_text(p.source_text))
    }
}

impl<'a> Format<'a> for Directive {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(
            p.str(&string::print_string(self.expression.value.as_str(), p.options.single_quote)),
        );
        if p.options.semi {
            parts.push(ss!(";"));
        }
        parts.push(hardline!());
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for Statement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::BlockStatement(stmt) => enter!(p, BlockStatement, stmt),
            Self::BreakStatement(stmt) => enter!(p, BreakStatement, stmt),
            Self::ContinueStatement(stmt) => enter!(p, ContinueStatement, stmt),
            Self::DebuggerStatement(stmt) => enter!(p, DebuggerStatement, stmt),
            Self::DoWhileStatement(stmt) => enter!(p, DoWhileStatement, stmt),
            Self::EmptyStatement(stmt) => enter!(p, EmptyStatement, stmt),
            Self::ExpressionStatement(stmt) => enter!(p, ExpressionStatement, stmt),
            Self::ForInStatement(stmt) => enter!(p, ForInStatement, stmt),
            Self::ForOfStatement(stmt) => enter!(p, ForOfStatement, stmt),
            Self::ForStatement(stmt) => enter!(p, ForStatement, stmt),
            Self::IfStatement(stmt) => enter!(p, IfStatement, stmt),
            Self::LabeledStatement(stmt) => enter!(p, LabeledStatement, stmt),
            Self::ModuleDeclaration(decl) => enter!(p, ModuleDeclaration, decl),
            Self::ReturnStatement(stmt) => enter!(p, ReturnStatement, stmt),
            Self::SwitchStatement(stmt) => enter!(p, SwitchStatement, stmt),
            Self::ThrowStatement(stmt) => enter!(p, ThrowStatement, stmt),
            Self::TryStatement(stmt) => enter!(p, TryStatement, stmt),
            Self::WhileStatement(stmt) => enter!(p, WhileStatement, stmt),
            Self::WithStatement(stmt) => enter!(p, WithStatement, stmt),
            Self::Declaration(decl) => decl.format(p),
        }
    }
}

impl<'a> Format<'a> for ExpressionStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(self.expression.format(p));
        if p.options.semi {
            parts.push(ss!(";"));
        }
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for EmptyStatement {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("")
    }
}

impl<'a> Format<'a> for IfStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        let consequent = format!(p, self.consequent);
        let consequent = misc::adjust_clause(p, &self.consequent, consequent, false);

        let opening = group![
            p,
            ss!("if ("),
            group!(p, indent!(p, softline!(), format!(p, self.test)), softline!()),
            ss!(")"),
            consequent
        ];
        parts.push(opening);

        if let Some(alternate) = &self.alternate {
            let else_on_same_line = matches!(alternate, Statement::BlockStatement(_));
            parts.push(if else_on_same_line { ss!(" ") } else { hardline!() });
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
    }
}

impl<'a> Format<'a> for BlockStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        block::print_block(p, &self.body, None)
    }
}

impl<'a> Format<'a> for ForStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
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
    }
}

impl<'a> Format<'a> for ForStatementInit<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            ForStatementInit::VariableDeclaration(v) => {
                enter!(p, VariableDeclaration, v)
            }
            ForStatementInit::Expression(v) => v.format(p),
            ForStatementInit::UsingDeclaration(v) => {
                enter!(p, UsingDeclaration, v)
            }
        }
    }
}

impl<'a> Format<'a> for ForInStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(ss!("for ("));
        parts.push(format!(p, self.left));
        parts.push(ss!(" in "));
        parts.push(format!(p, self.right));
        parts.push(ss!(")"));
        let body = format!(p, self.body);
        parts.push(misc::adjust_clause(p, &self.body, body, false));
        Doc::Group(Group::new(parts, false))
    }
}

impl<'a> Format<'a> for ForOfStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(ss!("for"));
        if self.r#await {
            parts.push(ss!(" await"));
        }
        parts.push(ss!(" ("));

        // for ((async) of []);
        // for ((let) of []);
        // for ((let.a) of []);
        let mut should_hug = false;
        if let ForStatementLeft::AssignmentTarget(AssignmentTarget::SimpleAssignmentTarget(
            target,
        )) = &self.left
        {
            if let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = target {
                if ident.name == "let" {
                    should_hug = true;
                }
                if ident.name == "async" && !self.r#await {
                    should_hug = true;
                }
            }
            if let SimpleAssignmentTarget::MemberAssignmentTarget(member_expr) = target {
                let object = match &**member_expr {
                    MemberExpression::ComputedMemberExpression(e) => Some(&e.object),
                    MemberExpression::StaticMemberExpression(e) => Some(&e.object),
                    MemberExpression::PrivateFieldExpression(e) => None,
                };
                if let Some(Expression::Identifier(ident)) = object {
                    if ident.name == "let" {
                        should_hug = true;
                    }
                }
            }
        }

        if should_hug {
            parts.push(ss!("("));
            parts.push(format!(p, self.left));
            parts.push(ss!(")"));
        } else {
            parts.push(format!(p, self.left));
        }

        parts.push(ss!(" of "));
        parts.push(format!(p, self.right));
        parts.push(ss!(")"));
        let body = format!(p, self.body);
        parts.push(misc::adjust_clause(p, &self.body, body, false));
        Doc::Group(Group::new(parts, false))
    }
}

impl<'a> Format<'a> for ForStatementLeft<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            ForStatementLeft::VariableDeclaration(v) => enter!(p, VariableDeclaration, v),
            ForStatementLeft::AssignmentTarget(v) => enter!(p, AssignmentTarget, v),
            ForStatementLeft::UsingDeclaration(v) => enter!(p, UsingDeclaration, v),
        }
    }
}

impl<'a> Format<'a> for WhileStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(ss!("while ("));
        parts.push(group!(p, indent!(p, softline!(), format!(p, self.test)), softline!()));
        parts.push(ss!(")"));

        let body = format!(p, self.body);
        parts.push(misc::adjust_clause(p, &self.body, body, false));

        Doc::Group(Group::new(parts, false))
    }
}

impl<'a> Format<'a> for DoWhileStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        let clause = format!(p, self.body);
        let clause = misc::adjust_clause(p, &self.body, clause, false);
        let do_body = group!(p, ss!("do"), clause);

        parts.push(do_body);

        if matches!(self.body, Statement::BlockStatement(_)) {
            parts.push(ss!(" "));
        } else {
            parts.push(hardline!());
        }

        parts.push(ss!("while ("));
        parts.push(group!(p, indent!(p, softline!(), format!(p, self.test)), softline!()));
        parts.push(ss!(")"));
        if p.options.semi {
            parts.push(ss!(";"));
        }

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for ContinueStatement {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(ss!("continue"));

        if let Some(label) = &self.label {
            parts.push(ss!(" "));
            parts.push(format!(p, label));
        }

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for BreakStatement {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(ss!("break"));

        if let Some(label) = &self.label {
            parts.push(ss!(" "));
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
        let mut parts = p.vec();

        let mut header_parts = p.vec();

        header_parts.push(ss!("switch ("));

        header_parts.push(indent!(p, softline!(), format!(p, self.discriminant)));

        header_parts.push(softline!());
        header_parts.push(ss!(")"));

        parts.push(Doc::Group(Group::new(header_parts, false)));

        parts.push(ss!(" {"));

        let mut cases_parts = p.vec();
        let len = self.cases.len();
        for (i, case) in self.cases.iter().enumerate() {
            cases_parts.push(indent!(p, hardline!(), format!(p, case)));
            if i != len - 1 && p.is_next_line_empty(case.span.end) {
                cases_parts.push(hardline!());
            }
        }
        parts.extend(cases_parts);

        parts.push(hardline!());
        parts.push(ss!("}"));

        Doc::Array(parts)
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
                if p.is_next_line_empty(last_stmt.span().end) {
                    consequent_parts.push(hardline!());
                }
            }

            consequent_parts.push(if is_only_one_block_statement { ss!(" ") } else { hardline!() });
            consequent_parts.push(format!(p, stmt));
        }

        if !consequent_parts.is_empty() {
            if is_only_one_block_statement {
                parts.extend(consequent_parts);
            } else {
                parts.push(indent!(
                    p,
                    Doc::Group(Group { contents: consequent_parts, should_break: false })
                ));
            }
        }

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for ReturnStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        function::print_return_or_throw_argument(p, self.argument.as_ref(), true)
    }
}

impl<'a> Format<'a> for LabeledStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        if matches!(self.body, Statement::EmptyStatement(_)) {
            return array!(p, enter!(p, LabelIdentifier, &self.label), ss!(":;"));
        }

        array!(p, enter!(p, LabelIdentifier, &self.label), ss!(": "), format!(p, self.body))
    }
}

impl<'a> Format<'a> for TryStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(ss!("try "));
        parts.push(enter!(p, BlockStatement, &self.block));
        if let Some(handler) = &self.handler {
            parts.push(ss!(" "));
            parts.push(enter!(p, CatchClause, handler));
        }
        if let Some(finalizer) = &self.finalizer {
            parts.push(ss!(" finally "));
            parts.push(enter!(p, BlockStatement, finalizer));
        }
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for CatchClause<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(ss!("catch "));
        if let Some(param) = &self.param {
            parts.push(ss!("("));
            parts.push(format!(p, param));
            parts.push(ss!(") "));
        }
        parts.push(enter!(p, BlockStatement, &self.body));

        Doc::Array(parts)
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
        if let ModuleDeclaration::ImportDeclaration(decl) = self {
            decl.format(p)
        } else {
            module::print_export_declaration(p, self)
        }
    }
}

impl<'a> Format<'a> for Declaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Declaration::ClassDeclaration(v) => enter!(p, Class, v),
            Declaration::FunctionDeclaration(v) => enter!(p, Function, v),
            Declaration::VariableDeclaration(v) => enter!(p, VariableDeclaration, v),
            Declaration::UsingDeclaration(v) => enter!(p, UsingDeclaration, v),
            Declaration::TSTypeAliasDeclaration(v) => enter!(p, TSTypeAliasDeclaration, v),
            Declaration::TSInterfaceDeclaration(v) => enter!(p, TSInterfaceDeclaration, v),
            Declaration::TSEnumDeclaration(v) => enter!(p, TSEnumDeclaration, v),
            Declaration::TSModuleDeclaration(v) => enter!(p, TSModuleDeclaration, v),
            Declaration::TSImportEqualsDeclaration(v) => {
                enter!(p, TSImportEqualsDeclaration, v)
            }
        }
    }
}

impl<'a> Format<'a> for VariableDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
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
        parts.push(ss!(" "));

        let is_hardline = !p.parent_kind().is_iteration_statement()
            && self.declarations.iter().all(|decl| decl.init.is_some());
        let decls_len = self.declarations.len();
        parts.extend(self.declarations.iter().enumerate().map(|(i, decl)| {
            if decls_len > 1 {
                let mut d_parts = p.vec();
                if i != 0 {
                    d_parts.push(p.str(","));
                    d_parts.push(if is_hardline { hardline!() } else { line!() });
                }
                d_parts.push(decl.format(p));
                Doc::Indent(d_parts)
            } else {
                decl.format(p)
            }
        }));

        if !parent_for_loop.is_some_and(|span| span != self.span) {
            parts.push(ss!(";"));
        }

        Doc::Group(Group::new(parts, false))
    }
}

impl<'a> Format<'a> for UsingDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSTypeAliasDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(ss!("type "));
        parts.push(format!(p, self.id));
        parts.push(ss!(" = "));
        parts.push(format!(p, self.type_annotation));

        if p.options.semi {
            parts.push(ss!(";"));
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
            TSType::TSThisKeyword(v) => v.format(p),
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

impl<'a> Format<'a> for TSThisKeyword {
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
        Doc::Line
    }
}

impl<'a> Format<'a> for TSConditionalType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSConstructorType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSFunctionType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSImportType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
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
        Doc::Line
    }
}

impl<'a> Format<'a> for TSLiteralType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match &self.literal {
            TSLiteral::BooleanLiteral(v) => enter!(p, BooleanLiteral, v),
            TSLiteral::NullLiteral(v) => enter!(p, NullLiteral, v),
            TSLiteral::NumberLiteral(v) => enter!(p, NumberLiteral, v),
            TSLiteral::BigintLiteral(v) => enter!(p, BigintLiteral, v),
            TSLiteral::RegExpLiteral(v) => enter!(p, RegExpLiteral, v),
            TSLiteral::StringLiteral(v) => enter!(p, StringLiteral, v),
            TSLiteral::TemplateLiteral(v) => enter!(p, TemplateLiteral, v),
            TSLiteral::UnaryExpression(v) => enter!(p, UnaryExpression, v),
        }
    }
}

impl<'a> Format<'a> for TSMappedType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSQualifiedName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
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
        Doc::Line
    }
}

impl<'a> Format<'a> for TSTypeOperatorType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSTypePredicate<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSTypeQuery<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSTypeReference<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSUnionType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSDocNullableType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSDocUnknownType {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSInterfaceDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSEnumDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSModuleDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSImportEqualsDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSTypeParameter<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSTypeParameterDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSTypeParameterInstantiation<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSTupleElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for VariableDeclarator<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        assignment::print_variable_declarator(p, self)
    }
}

impl<'a> Format<'a> for Function<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        function::print_function(p, self, None)
    }
}

impl<'a> Format<'a> for FunctionBody<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        block::print_block(p, &self.statements, Some(&self.directives))
    }
}

impl<'a> Format<'a> for FormalParameters<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        function_parameters::print_function_parameters(p, self)
    }
}

impl<'a> Format<'a> for FormalParameter<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        self.pattern.format(p)
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
            let is_default = specifiers.get(0).is_some_and(|x| {
                matches!(x, ImportDeclarationSpecifier::ImportDefaultSpecifier(_))
            });

            let validate_namespace = |x: &ImportDeclarationSpecifier| {
                matches!(x, ImportDeclarationSpecifier::ImportNamespaceSpecifier(_))
            };

            let is_namespace = specifiers.get(0).is_some_and(validate_namespace)
                || specifiers.get(1).is_some_and(validate_namespace);

            parts.push(module::print_module_specifiers(p, specifiers, is_default, is_namespace));
        }
        parts.push(ss!(" from "));
        parts.push(self.source.format(p));
        if p.options.semi {
            parts.push(ss!(";"));
        }
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for ImportDeclarationSpecifier {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::ImportSpecifier(specifier) => specifier.format(p),
            Self::ImportDefaultSpecifier(specifier) => specifier.format(p),
            Self::ImportNamespaceSpecifier(specifier) => specifier.format(p),
        }
    }
}

impl<'a> Format<'a> for ImportSpecifier {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        if self.imported.span() == self.local.span {
            self.local.format(p)
        } else {
            array![p, self.imported.format(p), ss!(" as "), self.local.format(p)]
        }
    }
}

impl<'a> Format<'a> for ImportDefaultSpecifier {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        self.local.format(p)
    }
}

impl<'a> Format<'a> for ImportNamespaceSpecifier {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array!(p, ss!("* as "), self.local.format(p))
    }
}

impl<'a> Format<'a> for Option<Vec<'a, ImportAttribute>> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ImportAttribute {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ExportNamedDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(module::print_module_specifiers(
            p,
            &self.specifiers,
            /* include_default */ false,
            /* include_namespace */ false,
        ));
        if let Some(decl) = &self.declaration {
            parts.push(decl.format(p));
        }
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for TSExportAssignment<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSNamespaceExportDeclaration {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ExportSpecifier {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        if self.exported.span() == self.local.span() {
            self.local.format(p)
        } else {
            array![p, self.local.format(p), ss!(" as "), self.exported.format(p)]
        }
    }
}

impl<'a> Format<'a> for ModuleExportName {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::Identifier(ident) => enter!(p, IdentifierName, ident),
            Self::StringLiteral(literal) => enter!(p, StringLiteral, literal),
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
            Self::FunctionDeclaration(decl) => {
                enter!(p, Function, decl)
            }
            Self::ClassDeclaration(decl) => {
                enter!(p, Class, decl)
            }
            Self::TSInterfaceDeclaration(decl) => {
                enter!(p, TSInterfaceDeclaration, decl)
            }
            Self::TSEnumDeclaration(decl) => enter!(p, TSEnumDeclaration, decl),
        }
    }
}

impl<'a> Format<'a> for Expression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::BooleanLiteral(lit) => enter!(p, BooleanLiteral, lit),
            Self::NullLiteral(lit) => enter!(p, NullLiteral, lit),
            Self::NumberLiteral(lit) => enter!(p, NumberLiteral, lit),
            Self::BigintLiteral(lit) => enter!(p, BigintLiteral, lit),
            Self::RegExpLiteral(lit) => enter!(p, RegExpLiteral, lit),
            Self::StringLiteral(lit) => enter!(p, StringLiteral, lit),
            Self::Identifier(ident) => enter!(p, IdentifierReference, ident),
            Self::ThisExpression(expr) => enter!(p, ThisExpression, expr),
            Self::MemberExpression(expr) => enter!(p, MemberExpression, expr),
            Self::CallExpression(expr) => enter!(p, CallExpression, expr),
            Self::ArrayExpression(expr) => enter!(p, ArrayExpression, expr),
            Self::ObjectExpression(expr) => enter!(p, ObjectExpression, expr),
            Self::FunctionExpression(expr) => enter!(p, Function, expr),
            Self::ArrowExpression(expr) => enter!(p, ArrowExpression, expr),
            Self::YieldExpression(expr) => enter!(p, YieldExpression, expr),
            Self::UpdateExpression(expr) => enter!(p, UpdateExpression, expr),
            Self::UnaryExpression(expr) => enter!(p, UnaryExpression, expr),
            Self::BinaryExpression(expr) => enter!(p, BinaryExpression, expr),
            Self::PrivateInExpression(expr) => enter!(p, PrivateInExpression, expr),
            Self::LogicalExpression(expr) => enter!(p, LogicalExpression, expr),
            Self::ConditionalExpression(expr) => enter!(p, ConditionalExpression, expr),
            Self::AssignmentExpression(expr) => enter!(p, AssignmentExpression, expr),
            Self::SequenceExpression(expr) => enter!(p, SequenceExpression, expr),
            Self::ParenthesizedExpression(expr) => enter!(p, ParenthesizedExpression, expr),
            Self::ImportExpression(expr) => enter!(p, ImportExpression, expr),
            Self::TemplateLiteral(literal) => enter!(p, TemplateLiteral, literal),
            Self::TaggedTemplateExpression(expr) => enter!(p, TaggedTemplateExpression, expr),
            Self::Super(sup) => enter!(p, Super, sup),
            Self::AwaitExpression(expr) => enter!(p, AwaitExpression, expr),
            Self::ChainExpression(expr) => enter!(p, ChainExpression, expr),
            Self::NewExpression(expr) => enter!(p, NewExpression, expr),
            Self::MetaProperty(expr) => enter!(p, MetaProperty, expr),
            Self::ClassExpression(expr) => enter!(p, Class, expr),
            Self::JSXElement(el) => enter!(p, JSXElement, el),
            Self::JSXFragment(fragment) => enter!(p, JSXFragment, fragment),
            Self::TSAsExpression(expr) => expr.expression.format(p),
            Self::TSSatisfiesExpression(expr) => expr.expression.format(p),
            Self::TSTypeAssertion(expr) => expr.expression.format(p),
            Self::TSNonNullExpression(expr) => expr.expression.format(p),
            Self::TSInstantiationExpression(expr) => expr.expression.format(p),
        }
    }
}

impl<'a> Format<'a> for IdentifierReference {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.str(self.name.as_str())
    }
}

impl<'a> Format<'a> for IdentifierName {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.str(self.name.as_str())
    }
}

impl<'a> Format<'a> for BindingIdentifier {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.str(self.name.as_str())
    }
}

impl<'a> Format<'a> for LabelIdentifier {
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

impl<'a> Format<'a> for NumberLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
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
            let trimmed = tail.trim_start_matches(|c| c == '+' || c == '-').trim_start_matches('0');
            if trimmed.starts_with(|c: char| c.is_ascii_digit()) {
                string = Cow::Owned(std::format!("{head}e{negative}{trimmed}"));
            }
        }

        // Remove unnecessary scientific notation (1e0).
        if let Some((head, tail)) = string.split_once('e') {
            if tail.trim_start_matches(|c| c == '+' || c == '-').trim_start_matches('0').is_empty()
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
    }
}

impl<'a> Format<'a> for BigintLiteral {
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

impl<'a> Format<'a> for RegExpLiteral {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(ss!("/"));
        parts.push(p.str(self.regex.pattern.as_str()));
        parts.push(ss!("/"));
        parts.push(format!(p, self.regex.flags));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for StringLiteral {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        p.str(&string::print_string(self.value.as_str(), p.options.single_quote))
    }
}

impl<'a> Format<'a> for ThisExpression {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("this")
    }
}

impl<'a> Format<'a> for MemberExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::ComputedMemberExpression(expr) => expr.format(p),
            Self::StaticMemberExpression(expr) => expr.format(p),
            Self::PrivateFieldExpression(expr) => expr.format(p),
        }
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
        call_expression::print_call_expression(
            p,
            &call_expression::CallExpressionLike::CallExpression(self),
        )
    }
}

impl<'a> Format<'a> for Argument<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::Expression(expr) => expr.format(p),
            Self::SpreadElement(expr) => enter!(p, SpreadElement, expr),
        }
    }
}

impl<'a> Format<'a> for ArrayExpressionElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::SpreadElement(expr) => enter!(p, SpreadElement, expr),
            Self::Expression(expr) => expr.format(p),
            Self::Elision(elision) => Doc::Str(""),
        }
    }
}

impl<'a> Format<'a> for SpreadElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array![p, ss!("..."), format!(p, self.argument)]
    }
}

impl<'a> Format<'a> for ArrayExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array::print_array(p, &Array::ArrayExpression(self))
    }
}

impl<'a> Format<'a> for ObjectExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        object::print_object_properties(p, &ObjectLike::ObjectExpression(self), &self.properties)
    }
}

impl<'a> Format<'a> for ObjectPropertyKind<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            ObjectPropertyKind::ObjectProperty(prop) => {
                enter!(p, ObjectProperty, prop)
            }
            ObjectPropertyKind::SpreadProperty(prop) => {
                enter!(p, SpreadElement, prop)
            }
        }
    }
}

impl<'a> Format<'a> for ObjectProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        // Perf: Use same print function with BindingProperty
        if self.shorthand {
            self.key.format(p)
        } else {
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
                    parts.push(function::print_function(
                        p,
                        func_expr,
                        Some(self.key.span().source_text(p.source_text)),
                    ));
                }
            } else {
                parts.push(format!(p, self.key));
                let comments =
                    p.print_inner_comment(Span::new(self.key.span().end, self.value.span().start));
                if !comments.is_empty() {
                    parts.push(ss!(" "));
                    parts.extend(comments);
                }
                parts.push(ss!(": "));
                parts.push(format!(p, self.value));
            }
            Doc::Group(Group::new(parts, false))
        }
    }
}

impl<'a> Format<'a> for PropertyKey<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            PropertyKey::Identifier(ident) => enter!(p, IdentifierName, ident),
            PropertyKey::PrivateIdentifier(ident) => {
                enter!(p, PrivateIdentifier, ident)
            }
            PropertyKey::Expression(expr) => match expr {
                Expression::StringLiteral(literal) => {
                    let value = literal.value.as_bytes();
                    if !&value[0].is_ascii_digit() && !value.contains(&b'_') {
                        p.str(&literal.value)
                    } else {
                        enter!(p, StringLiteral, literal)
                    }
                }
                Expression::Identifier(ident) => {
                    array!(p, ss!("["), enter!(p, IdentifierReference, ident), ss!("]"))
                }
                _ => expr.format(p),
            },
        }
    }
}

impl<'a> Format<'a> for ArrowExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        arrow_function::print_arrow_function(p, self)
    }
}

impl<'a> Format<'a> for YieldExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(ss!("yield"));
        if self.delegate {
            parts.push(ss!("*"));
        }
        if let Some(argument) = &self.argument {
            parts.push(ss!(" "));
            parts.push(format!(p, argument));
        }
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for UpdateExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        if self.prefix {
            array![p, ss!(self.operator.as_str()), format!(p, self.argument)]
        } else {
            array![p, format!(p, self.argument), ss!(self.operator.as_str())]
        }
    }
}

impl<'a> Format<'a> for UnaryExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(string!(p, self.operator.as_str()));
        if self.operator.is_keyword() {
            parts.push(ss!(" "));
        }
        parts.push(format!(p, self.argument));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for BinaryExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let doc = binaryish::print_binaryish_expression(
            p,
            &BinaryishLeft::Expression(&self.left),
            BinaryishOperator::BinaryOperator(self.operator),
            &self.right,
        );
        if misc::in_parentheses(p.parent_kind(), p.source_text, self.span) {
            group!(p, indent!(p, softline!(), doc), softline!())
        } else {
            doc
        }
    }
}

impl<'a> Format<'a> for PrivateInExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        binaryish::print_binaryish_expression(
            p,
            &BinaryishLeft::PrivateIdentifier(&self.left),
            BinaryishOperator::BinaryOperator(self.operator),
            &self.right,
        )
    }
}

impl<'a> Format<'a> for LogicalExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let doc = binaryish::print_binaryish_expression(
            p,
            &BinaryishLeft::Expression(&self.left),
            BinaryishOperator::LogicalOperator(self.operator),
            &self.right,
        );
        if misc::in_parentheses(p.parent_kind(), p.source_text, self.span) {
            group!(p, indent!(p, softline!(), doc), softline!())
        } else {
            doc
        }
    }
}

impl<'a> Format<'a> for ConditionalExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        ternary::print_ternary(p, self)
    }
}

impl<'a> Format<'a> for AssignmentExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        assignment::print_assignment_expression(p, self)
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
            Self::AssignmentTargetIdentifier(ident) => enter!(p, IdentifierReference, ident),
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
        object::print_object_properties(
            p,
            &ObjectLike::ObjectAssignmentTarget(self),
            &self.properties,
        )
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
        parts.push(enter!(p, IdentifierReference, &self.binding));
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
        parts.push(self.binding.format(p));
        parts.push(ss!(": "));
        parts.push(self.name.format(p));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for SequenceExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let docs = self.expressions.iter().map(|expr| expr.format(p)).collect::<std::vec::Vec<_>>();
        group![p, Doc::Array(p.join(Separator::CommaLine, docs))]
    }
}

impl<'a> Format<'a> for ParenthesizedExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        unreachable!("Parser preserve_parens option need to be set to false.");
    }
}

impl<'a> Format<'a> for ImportExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(ss!("import"));
        parts.push(ss!("("));
        let mut indent_parts = p.vec();
        indent_parts.push(softline!());
        indent_parts.push(format!(p, self.source));
        if !self.arguments.is_empty() {
            for arg in &self.arguments {
                indent_parts.push(ss!(","));
                indent_parts.push(Doc::Line);
                indent_parts.push(format!(p, arg));
            }
        }
        parts.push(group!(p, Doc::Indent(indent_parts)));
        parts.push(softline!());
        parts.push(ss!(")"));

        Doc::Group(Group::new(parts, false))
    }
}

impl<'a> Format<'a> for TemplateLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        template_literal::print_template_literal(p, &TemplateLiteralPrinter::TemplateLiteral(self))
    }
}

impl<'a> Format<'a> for TemplateElement {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        // TODO: `replaceEndOfLine`
        p.str(self.value.raw.as_str())
    }
}

impl<'a> Format<'a> for TaggedTemplateExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(format!(p, self.tag));

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(string!(p, "<"));
            parts.push(format!(p, type_parameters));
            parts.push(string!(p, ">"));
        }

        parts.push(format!(p, self.quasi));

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for Super {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Str("super")
    }
}

impl<'a> Format<'a> for AwaitExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(ss!("await "));
        parts.push(format!(p, self.argument));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for ChainExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        format!(p, self.expression)
    }
}

impl<'a> Format<'a> for ChainElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::CallExpression(expr) => enter!(p, CallExpression, expr),
            Self::MemberExpression(expr) => enter!(p, MemberExpression, expr),
        }
    }
}

impl<'a> Format<'a> for NewExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        call_expression::print_call_expression(
            p,
            &call_expression::CallExpressionLike::NewExpression(self),
        )
    }
}

impl<'a> Format<'a> for MetaProperty {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array![p, format!(p, self.meta), ss!("."), format!(p, self.property)]
    }
}

impl<'a> Format<'a> for Class<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        class::print_class(p, self)
    }
}

impl<'a> Format<'a> for ClassBody<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        class::print_class_body(p, self)
    }
}

impl<'a> Format<'a> for ClassElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            ClassElement::StaticBlock(c) => enter!(p, StaticBlock, c),
            ClassElement::MethodDefinition(c) => enter!(p, MethodDefinition, c),
            ClassElement::PropertyDefinition(c) => enter!(p, PropertyDefinition, c),
            ClassElement::AccessorProperty(c) => c.format(p),
            ClassElement::TSAbstractMethodDefinition(c) => c.format(p),
            ClassElement::TSAbstractPropertyDefinition(c) => c.format(p),
            ClassElement::TSIndexSignature(c) => c.format(p),
        }
    }
}

impl<'a> Format<'a> for JSXIdentifier {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXMemberExpressionObject<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXMemberExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXElementName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXNamespacedName {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXAttributeName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXAttribute<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXEmptyExpression {
    fn format(&self, _: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXExpressionContainer<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXAttributeValue<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXSpreadAttribute<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXAttributeItem<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXOpeningElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXClosingElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXOpeningFragment {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXClosingFragment {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXText {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXSpreadChild<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXChild<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for JSXFragment<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for StaticBlock<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array![p, ss!("static "), block::print_block(p, &self.body, None)]
    }
}

impl<'a> Format<'a> for MethodDefinition<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        function::print_method(p, self)
    }
}

impl<'a> Format<'a> for PropertyDefinition<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for AccessorProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for PrivateIdentifier {
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
            BindingPatternKind::BindingIdentifier(ref ident) => {
                enter!(p, BindingIdentifier, ident)
            }
            BindingPatternKind::ObjectPattern(ref pattern) => enter!(p, ObjectPattern, pattern),
            BindingPatternKind::ArrayPattern(ref pattern) => enter!(p, ArrayPattern, pattern),
            BindingPatternKind::AssignmentPattern(ref pattern) => {
                enter!(p, AssignmentPattern, pattern)
            }
        }
    }
}

impl<'a> Format<'a> for ObjectPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        object::print_object_properties(p, &ObjectLike::ObjectPattern(self), &self.properties)
    }
}

impl<'a> Format<'a> for BindingProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        if self.shorthand {
            self.key.format(p)
        } else {
            group!(p, format!(p, self.key), ss!(": "), format!(p, self.value))
        }
    }
}

impl<'a> Format<'a> for RestElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array!(p, ss!("..."), format!(p, self.argument))
    }
}

impl<'a> Format<'a> for ArrayPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array::print_array(p, &Array::ArrayPattern(self))
    }
}

impl<'a> Format<'a> for AssignmentPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array![p, format!(p, self.left), ss!(" = "), format!(p, self.right)]
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

impl<'a> Format<'a> for TSAbstractMethodDefinition<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSAbstractPropertyDefinition<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSIndexSignature<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}
