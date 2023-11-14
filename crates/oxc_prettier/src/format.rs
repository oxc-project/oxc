//! Formatting logic
//!
//! References:
//! * <https://github.com/prettier/prettier/blob/main/src/language-js/print/estree.js>

#![allow(unused_variables)]

use oxc_allocator::{Box, Vec};
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{array, doc::Doc, format, group, hardline, indent, softline, string, Prettier};

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
        parts.extend(self.body.iter().map(|stmt| stmt.format(p)));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for Directive {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
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
        let mut parts = p.vec();
        parts.push(self.expression.format(p));
        if p.options.semi {
            parts.push(p.str(";"));
        }
        parts.push(Doc::Hardline);
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

        let opening = group![
            p,
            string!(p, "if ("),
            group!(p, indent!(p, softline!(), format!(p, self.test), softline!())),
            string!(p, ")"),
            format!(p, self.consequent)
        ];
        parts.push(opening);

        if let Some(alternate) = &self.alternate {
            parts.push(string!(p, "else"));
            parts.push(group!(p, format!(p, alternate)));
        }

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for BlockStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(p.str("{"));
        parts.push(Doc::Softline);
        parts.extend(self.body.iter().map(|stmt| stmt.format(p)));
        parts.push(Doc::Softline);
        parts.push(p.str("}"));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for ForStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ForInStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ForOfStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ForStatementLeft<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for WhileStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for DoWhileStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ContinueStatement {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(string!(p, "continue"));

        if let Some(label) = &self.label {
            parts.push(string!(p, " "));
            parts.push(format!(p, label));
        }

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for BreakStatement {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(string!(p, "break"));

        if let Some(label) = &self.label {
            parts.push(string!(p, " "));
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

        header_parts.push(string!(p, "switch ("));

        header_parts.push(indent!(p, softline!(), format!(p, self.discriminant)));

        header_parts.push(softline!());
        header_parts.push(string!(p, ")"));

        parts.push(group!(p, Doc::Array(header_parts)));

        parts.push(p.str(" {"));

        let mut cases_parts = p.vec();

        for case in &self.cases {
            cases_parts.push(hardline!());
            cases_parts.push(format!(p, case));
        }

        parts.push(indent!(p, hardline!(), group!(p, Doc::Array(cases_parts))));

        parts.push(hardline!());
        parts.push(p.str("}"));

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for SwitchCase<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        if let Some(test) = &self.test {
            parts.push(string!(p, "case "));
            parts.push(format!(p, test));
            parts.push(string!(p, ":"));
        } else {
            parts.push(string!(p, "default:"));
        }

        let mut consequent_parts = p.vec();
        for stmt in &self.consequent {
            consequent_parts.push(hardline!());
            consequent_parts.push(format!(p, stmt));
        }

        parts.push(indent!(p, hardline!(), group!(p, Doc::Array(consequent_parts))));

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for ReturnStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        string!(p, "return")
    }
}

impl<'a> Format<'a> for LabeledStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TryStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(string!(p, "try "));

        parts.push(format!(p, self.block));

        if let Some(handler) = &self.handler {
            parts.push(string!(p, " "));
            parts.push(format!(p, handler));
        }

        if let Some(finalizer) = &self.finalizer {
            parts.push(string!(p, " finally "));
            parts.push(format!(p, finalizer));
        }

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for CatchClause<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(string!(p, "catch "));
        if let Some(param) = &self.param {
            parts.push(string!(p, "("));
            parts.push(format!(p, param));
            parts.push(string!(p, ") "));
        }
        parts.push(format!(p, self.body));

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for ThrowStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(string!(p, "throw "));
        parts.push(string!(p, " "));
        parts.push(format!(p, self.argument));

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for WithStatement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
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
        Doc::Line
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
        let kind = self.kind.to_string();
        let mut decls = p.vec();
        decls.extend(self.declarations.iter().map(|decl| decl.format(p)));

        array!(p, p.str(kind), p.str(" "), Doc::Array(decls), p.str(";"), Doc::Hardline)
    }
}

impl<'a> Format<'a> for UsingDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TSTypeAliasDeclaration<'a> {
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

impl<'a> Format<'a> for TSTypeParameterDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for VariableDeclarator<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(self.id.format(p));
        if let Some(init) = &self.init {
            parts.push(p.str(" = "));
            parts.push(init.format(p));
        }
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for Function<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        if self.r#async {
            parts.push(string!(p, "async "));
        }
        if self.generator {
            parts.push(string!(p, "function* "));
        } else {
            parts.push(p.str("function "));
        }
        if let Some(type_params) = &self.type_parameters {
            parts.push(format!(p, type_params));
        }
        if let Some(id) = &self.id {
            parts.push(p.str(id.name.as_str()));
        }
        parts.push(self.params.format(p));
        if let Some(body) = &self.body {
            parts.push(p.str(" "));
            parts.push(body.format(p));
        }
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for FunctionBody<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(p.str("{"));
        parts.push(Doc::Softline);
        parts.extend(self.statements.iter().map(|stmt| stmt.format(p)));
        parts.push(Doc::Softline);
        parts.push(p.str("}"));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for FormalParameters<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(p.str("("));
        parts.extend(self.items.iter().map(|stmt| stmt.format(p)));
        if let Some(rest) = &self.rest {
            parts.push(rest.format(p));
        }
        parts.push(p.str(")"));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for FormalParameter<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ImportDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
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
        Doc::Line
    }
}

impl<'a> Format<'a> for ExportSpecifier {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ModuleExportName {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ExportAllDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ExportDefaultDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}
impl<'a> Format<'a> for ExportDefaultDeclarationKind<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for Expression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            Self::BooleanLiteral(lit) => lit.format(p),
            Self::NullLiteral(lit) => lit.format(p),
            Self::NumberLiteral(lit) => lit.format(p),
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
            Self::ArrowExpression(expr) => expr.format(p),
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
        let raw = p.str(self.raw.to_lowercase().as_str());
        if self.raw.starts_with('.') {
            array!(p, string!(p, "0"), raw)
        } else {
            p.str(self.raw.to_lowercase().as_str())
        }
    }
}

impl<'a> Format<'a> for BigintLiteral {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for RegExpLiteral {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(p.str("/"));
        parts.push(p.str(self.regex.pattern.as_str()));
        parts.push(p.str("/"));
        parts.push(format!(p, self.regex.flags));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for StringLiteral {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
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
            parts.push(string!(p, "?."));
        }
        parts.push(string!(p, "["));
        parts.push(format!(p, self.expression));
        parts.push(string!(p, "["));

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for StaticMemberExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(format!(p, self.object));
        if self.optional {
            parts.push(string!(p, "?"));
        }
        parts.push(string!(p, "."));
        parts.push(format!(p, self.property));

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for PrivateFieldExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(format!(p, self.object));
        if self.optional {
            parts.push(string!(p, "?."));
        }
        parts.push(string!(p, "#"));
        parts.push(format!(p, self.field));

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for CallExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
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
        array![p, string!(p, "..."), format!(p, self.argument)]
    }
}

impl<'a> Format<'a> for ArrayExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        if self.elements.len() == 0 {
            return p.str("[]");
        }

        let mut parts = p.vec();
        parts.push(p.str("["));

        let mut parts_inner = p.vec();
        parts_inner.push(Doc::Softline);

        for (i, element) in self.elements.iter().enumerate() {
            if i > 0 {
                parts_inner.push(string!(p, ","));
                parts_inner.push(Doc::Softline);
            }
            parts_inner.push(format!(p, element));
        }

        let mut if_break_comma = p.vec();
        if_break_comma.push(Doc::Str(","));

        parts_inner.push(Doc::if_break(if_break_comma, p.vec()));

        parts.push(group!(p, Doc::Indent(parts_inner)));

        parts.push(Doc::Softline);
        parts.push(p.str("]"));

        Doc::Group(parts)
    }
}

impl<'a> Format<'a> for ObjectExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ObjectPropertyKind<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ObjectProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for PropertyKey<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ArrowExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for YieldExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(string!(p, "yield"));
        if self.delegate {
            parts.push(string!(p, "*"));
        }
        if let Some(argument) = &self.argument {
            parts.push(string!(p, " "));
            parts.push(format!(p, argument));
        }
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for UpdateExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(p.str(self.operator.as_str()));

        parts.push(format!(p, self.argument));

        if self.prefix {
            parts.reverse();
        }

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for UnaryExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();
        parts.push(string!(p, self.operator.as_str()));

        if matches!(
            self.operator,
            UnaryOperator::Typeof | UnaryOperator::Void | UnaryOperator::Delete
        ) {
            parts.push(string!(p, " "));
        }

        parts.push(format!(p, self.argument));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for BinaryExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = p.vec();

        parts.push(format!(p, self.left));

        let mut parts_inner = p.vec();

        parts_inner.push(Doc::Str(" "));

        let mut parts_inner_inner = p.vec();

        parts_inner_inner.push(string!(p, self.operator.as_str()));

        parts_inner_inner.push(Doc::Line);

        parts_inner_inner.push(format!(p, self.right));

        let indent = Doc::Indent(parts_inner_inner);

        parts_inner.push(group!(p, indent));

        parts.push(Doc::Indent(parts_inner));

        Doc::Group(parts)
    }
}

impl<'a> Format<'a> for BinaryOperator {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for PrivateInExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for LogicalExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ConditionalExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for AssignmentExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array![
            p,
            format!(p, self.left),
            string!(p, " "),
            string!(p, self.operator.as_str()),
            string!(p, " "),
            format!(p, self.right)
        ]
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
        Doc::Line
    }
}

impl<'a> Format<'a> for Option<AssignmentTargetMaybeDefault<'a>> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ObjectAssignmentTarget<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for AssignmentTargetMaybeDefault<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for AssignmentTargetWithDefault<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for AssignmentTargetProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for AssignmentTargetPropertyIdentifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for AssignmentTargetPropertyProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for SequenceExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ParenthesizedExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ImportExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TemplateLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for TaggedTemplateExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
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
        parts.push(string!(p, "await "));
        parts.push(format!(p, self.argument));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for ChainExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for NewExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for MetaProperty {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array![p, format!(p, self.meta), string!(p, ","), format!(p, self.property)]
    }
}

impl<'a> Format<'a> for Class<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ClassElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            ClassElement::StaticBlock(c) => c.format(p),
            ClassElement::MethodDefinition(c) => c.format(p),
            ClassElement::PropertyDefinition(c) => c.format(p),
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
        let mut parts = p.vec();
        parts.push(p.str("static {"));
        parts.push(Doc::Softline);
        parts.extend(self.body.iter().map(|stmt| stmt.format(p)));
        parts.push(Doc::Softline);
        parts.push(p.str("}"));
        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for MethodDefinition<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
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

        parts.push(p.str("#"));
        parts.push(p.str(self.name.as_str()));

        Doc::Array(parts)
    }
}

impl<'a> Format<'a> for BindingPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ObjectPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for BindingProperty<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for RestElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for ArrayPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        Doc::Line
    }
}

impl<'a> Format<'a> for AssignmentPattern<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array![p, format!(p, self.left), string!(p, " = "), format!(p, self.right)]
    }
}

impl<'a> Format<'a> for RegExpFlags {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut str = vec![];

        if self.contains(Self::D) {
            str.push('d');
        }
        if self.contains(Self::G) {
            str.push('g');
        }
        if self.contains(Self::I) {
            str.push('i');
        }
        if self.contains(Self::M) {
            str.push('m');
        }
        if self.contains(Self::S) {
            str.push('s');
        }
        if self.contains(Self::V) {
            str.push('v');
        }
        if self.contains(Self::U) {
            str.push('u');
        }
        if self.contains(Self::Y) {
            str.push('y');
        }
        p.str(str.iter().collect::<String>().as_str())
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
