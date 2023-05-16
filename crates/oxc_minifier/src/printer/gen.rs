use std::matches;

use oxc_allocator::{Box, Vec};
#[allow(clippy::wildcard_imports)]
use oxc_hir::hir::*;
use oxc_syntax::operator::{
    AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
};

use super::{Operator, Printer, Separator};

pub trait Gen {
    fn gen(&self, p: &mut Printer);
}

impl<'a, T> Gen for Box<'a, T>
where
    T: Gen,
{
    fn gen(&self, p: &mut Printer) {
        (**self).gen(p);
    }
}

impl<'a> Gen for Program<'a> {
    fn gen(&self, p: &mut Printer) {
        for directive in &self.directives {
            directive.gen(p);
        }
        for stmt in &self.body {
            p.print_semicolon_if_needed();
            stmt.gen(p);
        }
    }
}

impl<'a> Gen for Directive<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print(b'"');
        p.print_str(self.directive.as_bytes());
        p.print(b'"');
        p.print_semicolon();
        p.print_newline();
    }
}

impl<'a> Gen for Statement<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::BlockStatement(stmt) => stmt.gen(p),
            Self::BreakStatement(stmt) => stmt.gen(p),
            Self::ContinueStatement(stmt) => stmt.gen(p),
            Self::DebuggerStatement(stmt) => stmt.gen(p),
            Self::DoWhileStatement(stmt) => stmt.gen(p),
            Self::EmptyStatement(stmt) => stmt.gen(p),
            Self::ExpressionStatement(stmt) => stmt.gen(p),
            Self::ForInStatement(stmt) => stmt.gen(p),
            Self::ForOfStatement(stmt) => stmt.gen(p),
            Self::ForStatement(stmt) => stmt.gen(p),
            Self::IfStatement(stmt) => stmt.gen(p),
            Self::LabeledStatement(stmt) => stmt.gen(p),
            Self::ModuleDeclaration(decl) => decl.gen(p),
            Self::ReturnStatement(stmt) => stmt.gen(p),
            Self::SwitchStatement(stmt) => stmt.gen(p),
            Self::ThrowStatement(stmt) => stmt.gen(p),
            Self::TryStatement(stmt) => stmt.gen(p),
            Self::WhileStatement(stmt) => stmt.gen(p),
            Self::WithStatement(stmt) => stmt.gen(p),
            Self::Declaration(decl) => decl.gen(p),
        }
    }
}

impl<'a> Gen for ExpressionStatement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        self.expression.gen(p);
        if let Expression::Identifier(ident) = &self.expression
        && ident.name == "let" {
            p.print_semicolon();
        } else {
            p.print_semicolon_after_statement();
        }
    }
}

impl Gen for EmptyStatement {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print_semicolon();
        p.print_newline();
    }
}

impl<'a> Gen for IfStatement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        print_if(self, p);
    }
}

fn print_if(if_stmt: &IfStatement<'_>, p: &mut Printer) {
    p.print_str(b"if");
    p.print_space();
    p.print(b'(');
    if_stmt.test.gen(p);
    p.print(b')');
    p.print_space();
    if_stmt.consequent.gen(p);
    if if_stmt.alternate.is_some() {
        p.print_space();
    } else {
        p.print_newline();
    }
    if let Some(alternate) = if_stmt.alternate.as_ref() {
        p.print_semicolon_if_needed();
        p.print(b' ');
        p.print_str(b"else");
        p.print(b' ');
        match alternate {
            Statement::BlockStatement(block) => {
                p.print_block1(block);
                p.print_newline();
            }
            Statement::IfStatement(if_stmt) => {
                print_if(if_stmt, p);
            }
            _ => {
                p.print_newline();
                p.indent();
                alternate.gen(p);
                p.dedent();
            }
        }
    }
}

impl<'a> Gen for BlockStatement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print_block1(self);
        p.print_newline();
    }
}

impl<'a> Gen for ForStatement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print_str(b"for");
        p.print_space();
        p.print(b'(');

        if let Some(init) = self.init.as_ref() {
            match init {
                ForStatementInit::Expression(expr) => expr.gen(p),
                ForStatementInit::VariableDeclaration(var) => var.gen(p),
            }
        }

        p.print_semicolon();
        p.print_space();

        if let Some(test) = self.test.as_ref() {
            test.gen(p);
        }

        p.print_semicolon();
        p.print_space();

        if let Some(update) = self.update.as_ref() {
            update.gen(p);
        }

        p.print(b')');
        p.print_body(&self.body);
    }
}

impl<'a> Gen for ForInStatement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print_str(b"for");
        gen_for_statement_brack_content(&self.left, &self.right, &self.body, b"in", p);
    }
}

impl<'a> Gen for ForOfStatement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print_str(b"for");
        if self.r#await {
            p.print_str(b" await");
        }
        gen_for_statement_brack_content(&self.left, &self.right, &self.body, b"of", p);
    }
}

fn gen_for_statement_brack_content<'a>(
    left: &ForStatementLeft<'a>,
    right: &Expression<'a>,
    body: &Statement,
    key: &[u8],
    p: &mut Printer,
) {
    p.print_space();
    p.print(b'(');
    left.gen(p);
    p.print(b' ');
    p.print_str(key);
    p.print(b' ');
    right.gen(p);
    p.print(b')');
    p.print_body(body);
}

impl<'a> Gen for ForStatementLeft<'a> {
    fn gen(&self, p: &mut Printer) {
        match &self {
            ForStatementLeft::VariableDeclaration(var) => var.gen(p),
            ForStatementLeft::AssignmentTarget(target) => target.gen(p),
        }
    }
}

impl<'a> Gen for WhileStatement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print_str(b"while");
        p.print_space();
        p.print(b'(');
        self.test.gen(p);
        p.print(b')');
        p.print_body(&self.body);
    }
}

impl<'a> Gen for DoWhileStatement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print_str(b"do");
        p.print(b' ');
        if let Statement::BlockStatement(block) = &self.body {
            p.print_space();
            p.print_block1(block);
            p.print_space();
        } else {
            p.print_newline();
            p.indent();
            self.body.gen(p);
            p.print_semicolon_if_needed();
            p.dedent();
            p.print_indent();
        }
        p.print_str(b"while");
        p.print_space();
        p.print(b'(');
        self.test.gen(p);
        p.print(b')');
        p.print_semicolon_after_statement();
    }
}

impl Gen for ContinueStatement {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print_str(b"continue");
        if let Some(label) = &self.label {
            p.print_space();
            label.gen(p);
        }
        p.print_semicolon_after_statement();
    }
}

impl Gen for BreakStatement {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print_str(b"break");
        if let Some(label) = &self.label {
            p.print_space();
            label.gen(p);
        }
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for SwitchStatement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print_str(b"switch");
        p.print_space();
        p.print(b'(');
        self.discriminant.gen(p);
        p.print(b')');
        p.print_space();
        p.print(b'{');
        p.print_newline();
        p.indent();
        for case in &self.cases {
            case.gen(p);
        }
        p.dedent();
        p.print_indent();
        p.print(b'}');
        p.print_newline();
        p.needs_semicolon = false;
    }
}

impl<'a> Gen for SwitchCase<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_semicolon_if_needed();
        p.print_indent();
        match &self.test {
            Some(test) => {
                p.print_str(b"case");
                p.print(b' ');
                test.gen(p);
            }
            None => p.print_str(b"default"),
        }
        p.print_colon();
        p.print_newline();
        p.indent();
        for item in &self.consequent {
            p.print_semicolon_if_needed();
            item.gen(p);
        }
        p.dedent();
    }
}

impl<'a> Gen for ReturnStatement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print_str(b"return");
        if let Some(arg) = &self.argument {
            p.print(b' ');
            arg.gen(p);
        }
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for LabeledStatement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        self.label.gen(p);
        p.print_colon();
        p.print_newline();
        self.body.gen(p);
    }
}

impl<'a> Gen for TryStatement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print_str(b"try");
        p.print_space();
        p.print_block1(&self.block);
        if let Some(handler) = &self.handler {
            p.print_space();
            p.print_str(b"catch");
            if let Some(param) = &handler.param {
                p.print_space();
                p.print_str(b"(");
                param.gen(p);
                p.print_str(b")");
            }
            p.print_space();
            p.print_block1(&handler.body);
        }
        if let Some(finalizer) = &self.finalizer {
            p.print_space();
            p.print_str(b"finally");
            p.print_space();
            p.print_block1(finalizer);
        }
        p.print_newline();
    }
}

impl<'a> Gen for ThrowStatement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print_str(b"throw");
        p.print(b' ');
        self.argument.gen(p);
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for WithStatement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print_str(b"with");
        p.print_space();
        p.print(b'(');
        self.object.gen(p);
        p.print(b')');
        p.print_space();
        self.body.gen(p);
    }
}

impl Gen for DebuggerStatement {
    fn gen(&self, p: &mut Printer) {
        p.print_indent();
        p.print_str(b"debugger");
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for ModuleDeclaration<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::ImportDeclaration(decl) => decl.gen(p),
            Self::ExportAllDeclaration(decl) => decl.gen(p),
            Self::ExportDefaultDeclaration(decl) => decl.gen(p),
            Self::ExportNamedDeclaration(decl) => decl.gen(p),
        }
    }
}

impl<'a> Gen for Declaration<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::VariableDeclaration(stmt) => {
                p.print_indent();
                stmt.gen(p);
                p.print_semicolon_after_statement();
            }
            Self::FunctionDeclaration(stmt) => {
                p.print_indent();
                stmt.gen(p);
                p.print_newline();
            }
            Self::ClassDeclaration(declaration) => {
                declaration.gen(p);
                p.print_newline();
            }
            Self::TSEnumDeclaration(_) => {}
        }
    }
}

impl<'a> Gen for VariableDeclaration<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_str(match self.kind {
            VariableDeclarationKind::Const => b"const",
            VariableDeclarationKind::Let => b"let",
            VariableDeclarationKind::Var => b"var",
        });
        p.print(b' ');
        p.print_list(&self.declarations);
    }
}

impl<'a> Gen for VariableDeclarator<'a> {
    fn gen(&self, p: &mut Printer) {
        self.id.gen(p);
        if let Some(init) = &self.init {
            p.print_space();
            p.print_equal();
            p.print_space();
            init.gen(p);
        }
    }
}

impl<'a> Gen for Function<'a> {
    fn gen(&self, p: &mut Printer) {
        if self.r#async {
            p.print_str(b"async");
            p.print(b' ');
        }
        p.print_str(b"function");
        if self.generator {
            p.print(b'*');
        }
        if let Some(id) = &self.id {
            if !self.generator {
                p.print(b' ');
            }
            id.gen(p);
            p.print_space();
        }
        p.print(b'(');
        self.params.gen(p);
        p.print(b')');
        p.print_space();
        if let Some(body) = &self.body {
            body.gen(p);
        }
    }
}

impl<'a> Gen for FunctionBody<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print(b'{');
        p.indent();
        p.print_newline();
        for directive in &self.directives {
            directive.gen(p);
        }
        p.needs_semicolon = if let Some(Statement::ExpressionStatement(expr_stmt)) = self.statements.get(0)
            && matches!(expr_stmt.expression, Expression::StringLiteral(_)) {
                true
            } else {
                false
            };
        for stmt in &self.statements {
            p.print_semicolon_if_needed();
            stmt.gen(p);
        }
        p.dedent();
        p.print_indent();
        p.print(b'}');
        p.needs_semicolon = false;
    }
}

impl<'a> Gen for FormalParameter<'a> {
    fn gen(&self, p: &mut Printer) {
        self.decorators.gen(p);
        self.pattern.gen(p);
    }
}

impl<'a> Gen for FormalParameters<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_list(&self.items);
        if let Some(rest) = &self.rest {
            if !self.items.is_empty() {
                p.print_comma();
            }
            rest.gen(p);
        }
    }
}

impl<'a> Gen for ImportDeclaration<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"import ");
        if self.specifiers.is_empty() {
            p.print(b'\'');
            p.print_str(self.source.value.as_bytes());
            p.print(b'\'');
            self.assertions.gen(p);
            p.print_semicolon_after_statement();
            return;
        }

        let mut in_block = false;
        for (index, specifier) in self.specifiers.iter().enumerate() {
            match specifier {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                    if in_block {
                        p.print_space();
                        p.print_str(b"},");
                        p.print_space();
                        in_block = false;
                    } else if index != 0 {
                        p.print_comma();
                        p.print_space();
                    }
                    spec.local.gen(p);
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                    if in_block {
                        p.print_space();
                        p.print_str(b"},");
                        p.print_space();
                        in_block = false;
                    } else if index != 0 {
                        p.print_comma();
                        p.print_space();
                    }
                    p.print_str(b"* as ");
                    spec.local.gen(p);
                }
                ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                    if in_block {
                        p.print_comma();
                    } else {
                        if index != 0 {
                            p.print_comma();
                            p.print_space();
                        }
                        in_block = true;
                        p.print(b'{');
                    }
                    p.print_space();

                    let imported_name = match &spec.imported {
                        ModuleExportName::Identifier(identifier) => {
                            identifier.gen(p);
                            identifier.name.as_bytes()
                        }
                        ModuleExportName::StringLiteral(literal) => {
                            literal.gen(p);
                            literal.value.as_bytes()
                        }
                    };

                    let local_name = spec.local.name.as_bytes();

                    if imported_name != local_name {
                        p.print_str(b" as ");
                        spec.local.gen(p);
                    }
                }
            }
        }
        if in_block {
            p.print_space();
            p.print(b'}');
        }
        p.print_str(b" from ");
        self.source.gen(p);
        self.assertions.gen(p);
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for Option<Vec<'a, ImportAttribute>> {
    fn gen(&self, p: &mut Printer) {
        if let Some(assertions) = &self {
            p.print_space();
            p.print_str(b"assert");
            p.print_space();
            p.print_block(assertions, Separator::Comma);
        };
    }
}

impl Gen for ImportAttribute {
    fn gen(&self, p: &mut Printer) {
        match &self.key {
            ImportAttributeKey::Identifier(identifier) => {
                p.print_str(identifier.name.as_bytes());
            }
            ImportAttributeKey::StringLiteral(literal) => literal.gen(p),
        };
        p.print_colon();
        p.print_space();
        self.value.gen(p);
    }
}

impl<'a> Gen for ExportNamedDeclaration<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"export ");
        match &self.declaration {
            Some(decl) => decl.gen(p),
            None => {
                p.print(b'{');
                if !self.specifiers.is_empty() {
                    p.print_space();
                    p.print_list(&self.specifiers);
                    p.print_space();
                }
                p.print(b'}');
                if let Some(source) = &self.source {
                    p.print_space();
                    p.print_str(b"from");
                    p.print_space();
                    source.gen(p);
                }
                p.print_semicolon_after_statement();
            }
        }
    }
}

impl Gen for ExportSpecifier {
    fn gen(&self, p: &mut Printer) {
        self.local.gen(p);
        if self.local.name() != self.exported.name() {
            p.print_str(b" as ");
            self.exported.gen(p);
        }
    }
}

impl Gen for ModuleExportName {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::Identifier(identifier) => {
                p.print_str(identifier.name.as_bytes());
            }
            Self::StringLiteral(literal) => literal.gen(p),
        };
    }
}

impl<'a> Gen for ExportAllDeclaration<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"export");
        p.print_space();
        p.print(b'*');

        if let Some(exported) = &self.exported {
            p.print_space();
            p.print_str(b"as ");
            exported.gen(p);
        }

        p.print_str(b" from");
        p.print_space();
        self.source.gen(p);
        self.assertions.gen(p);

        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for ExportDefaultDeclaration<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"export default ");
        self.declaration.gen(p);
    }
}
impl<'a> Gen for ExportDefaultDeclarationKind<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::Expression(expr) => {
                expr.gen(p);
                p.print_semicolon_after_statement();
            }
            Self::FunctionDeclaration(fun) => fun.gen(p),
            Self::ClassDeclaration(value) => value.gen(p),
            Self::TSEnumDeclaration(_) => {}
        }
    }
}

impl<'a> Gen for Expression<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::BooleanLiteral(lit) => lit.gen(p),
            Self::NullLiteral(lit) => lit.gen(p),
            Self::NumberLiteral(lit) => lit.gen(p),
            Self::BigintLiteral(lit) => lit.gen(p),
            Self::RegExpLiteral(lit) => lit.gen(p),
            Self::StringLiteral(lit) => lit.gen(p),
            Self::Identifier(ident) => ident.gen(p),
            Self::ThisExpression(expr) => expr.gen(p),
            Self::MemberExpression(expr) => expr.gen(p),
            Self::CallExpression(expr) => expr.gen(p),
            Self::ArrayExpression(expr) => expr.gen(p),
            Self::ObjectExpression(expr) => expr.gen(p),
            Self::ParenthesizedExpression(expr) => expr.gen(p),
            Self::FunctionExpression(expr) => expr.gen(p),
            Self::ArrowFunctionExpression(expr) => expr.gen(p),
            Self::YieldExpression(expr) => expr.gen(p),
            Self::UpdateExpression(expr) => expr.gen(p),
            Self::UnaryExpression(expr) => expr.gen(p),
            Self::BinaryExpression(expr) => expr.gen(p),
            Self::PrivateInExpression(expr) => expr.gen(p),
            Self::LogicalExpression(expr) => expr.gen(p),
            Self::ConditionalExpression(expr) => expr.gen(p),
            Self::AssignmentExpression(expr) => expr.gen(p),
            Self::SequenceExpression(expr) => expr.gen(p),
            Self::ImportExpression(expr) => expr.gen(p),
            Self::TemplateLiteral(literal) => literal.gen(p),
            Self::TaggedTemplateExpression(expr) => expr.gen(p),
            Self::Super(sup) => sup.gen(p),
            Self::AwaitExpression(expr) => expr.gen(p),
            Self::ChainExpression(expr) => expr.gen(p),
            Self::NewExpression(expr) => expr.gen(p),
            Self::MetaProperty(expr) => expr.gen(p),
            Self::ClassExpression(expr) => expr.gen(p),
            Self::JSXElement(el) => el.gen(p),
            Self::JSXFragment(fragment) => fragment.gen(p),
        }
    }
}

impl Gen for IdentifierReference {
    fn gen(&self, p: &mut Printer) {
        if let Some(symbol_id) = self.symbol_id {
            p.print_symbol(symbol_id, &self.name);
        } else {
            p.print_str(self.name.as_bytes());
        }
    }
}

impl Gen for IdentifierName {
    fn gen(&self, p: &mut Printer) {
        p.print_str(self.name.as_bytes());
    }
}

impl Gen for BindingIdentifier {
    fn gen(&self, p: &mut Printer) {
        p.print_symbol(self.symbol_id, &self.name);
    }
}

impl Gen for LabelIdentifier {
    fn gen(&self, p: &mut Printer) {
        p.print_str(self.name.as_bytes());
    }
}

impl Gen for BooleanLiteral {
    fn gen(&self, p: &mut Printer) {
        p.print_str(self.as_str().as_bytes());
    }
}

impl Gen for NullLiteral {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"null");
    }
}

impl<'a> Gen for NumberLiteral<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_str(self.raw.as_bytes());
    }
}

impl Gen for BigintLiteral {
    fn gen(&self, p: &mut Printer) {
        p.print_str(self.value.to_string().as_bytes());
        p.print(b'n');
    }
}

impl Gen for RegExpLiteral {
    fn gen(&self, p: &mut Printer) {
        p.print(b'/');
        p.print_str(self.regex.pattern.as_bytes());
        p.print(b'/');
        p.print_str(self.regex.flags.to_string().as_bytes());
    }
}

impl Gen for StringLiteral {
    fn gen(&self, p: &mut Printer) {
        p.print(b'\'');
        for c in self.value.chars() {
            p.print_str(c.escape_default().to_string().as_bytes());
        }
        p.print(b'\'');
    }
}

impl Gen for ThisExpression {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"this");
    }
}

impl<'a> Gen for MemberExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::ComputedMemberExpression(expr) => expr.gen(p),
            Self::StaticMemberExpression(expr) => expr.gen(p),
            Self::PrivateFieldExpression(expr) => expr.gen(p),
        }
    }
}

impl<'a> Gen for ComputedMemberExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        self.object.gen(p);
        if self.optional {
            p.print_str(b"?.");
        }
        p.print(b'[');
        self.expression.gen(p);
        p.print(b']');
    }
}

impl<'a> Gen for StaticMemberExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        let is_unary_expr = matches!(self.object, Expression::UnaryExpression(_));
        if is_unary_expr {
            p.print(b'(');
        }
        self.object.gen(p);
        if is_unary_expr {
            p.print(b')');
        }
        if self.optional {
            p.print(b'?');
        }
        p.print(b'.');
        self.property.gen(p);
    }
}

impl<'a> Gen for PrivateFieldExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        self.object.gen(p);
        if self.optional {
            p.print_str(b"?");
        }
        p.print(b'.');
        self.field.gen(p);
    }
}

impl<'a> Gen for CallExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        self.callee.gen(p);

        if self.optional {
            p.print_str(b"?.");
        }
        p.print(b'(');
        p.print_list(&self.arguments);
        p.print(b')');
    }
}

impl<'a> Gen for Argument<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::SpreadElement(elem) => elem.gen(p),
            Self::Expression(elem) => elem.gen(p),
        }
    }
}

impl<'a> Gen for ArrayExpressionElement<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::Expression(expr) => expr.gen(p),
            Self::SpreadElement(elem) => elem.gen(p),
            Self::Elision(_span) => {}
        }
    }
}

impl<'a> Gen for SpreadElement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_ellipsis();
        self.argument.gen(p);
    }
}

impl<'a> Gen for ArrayExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print(b'[');
        p.print_list(&self.elements);
        if self.trailing_comma.is_some() {
            p.print_comma();
        }
        p.print(b']');
    }
}

impl<'a> Gen for ObjectExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print(b'{');
        p.indent();
        for (i, item) in self.properties.iter().enumerate() {
            if i != 0 {
                p.print_comma();
            }
            if p.options.minify_whitespace {
                p.print_space();
            } else {
                p.print_newline();
                p.print_indent();
            }
            item.gen(p);
        }
        p.print_newline();
        p.dedent();
        p.print_indent();
        p.print(b'}');
    }
}

impl<'a> Gen for ObjectPropertyKind<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::ObjectProperty(prop) => prop.gen(p),
            Self::SpreadProperty(elem) => elem.gen(p),
        }
    }
}

impl<'a> Gen for ObjectProperty<'a> {
    fn gen(&self, p: &mut Printer) {
        if let Expression::FunctionExpression(func) = &self.value {
            let is_accessor = match &self.kind {
                PropertyKind::Init => false,
                PropertyKind::Get => {
                    p.print_str(b"get ");
                    true
                }
                PropertyKind::Set => {
                    p.print_str(b"set ");
                    true
                }
            };
            if self.method || is_accessor {
                if func.r#async {
                    p.print_str(b"async ");
                }
                if func.generator {
                    p.print_str(b"*");
                }
                if self.computed {
                    p.print(b'[');
                }
                self.key.gen(p);
                if self.computed {
                    p.print(b']');
                }
                p.print(b'(');
                func.params.gen(p);
                p.print(b')');
                p.print_space();
                if let Some(body) = &func.body {
                    body.gen(p);
                }
                return;
            }
        }
        if self.computed {
            p.print(b'[');
        }
        self.key.gen(p);
        if self.computed {
            p.print(b']');
        }
        p.print_colon();
        p.print_space();
        self.value.gen(p);
    }
}

impl<'a> Gen for PropertyKey<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::Identifier(ident) => ident.gen(p),
            Self::PrivateIdentifier(ident) => ident.gen(p),
            Self::Expression(expr) => expr.gen(p),
        }
    }
}

impl<'a> Gen for PropertyValue<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::Pattern(pattern) => pattern.gen(p),
            Self::Expression(expr) => expr.gen(p),
        }
    }
}

impl<'a> Gen for ParenthesizedExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print(b'(');
        self.expression.gen(p);
        p.print(b')');
    }
}

impl<'a> Gen for ArrowExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        if self.r#async {
            p.print_str(b"async");
            p.print_space();
        }
        p.print(b'(');
        self.params.gen(p);
        p.print(b')');
        p.print_space();
        p.print_str(b"=>");
        p.print_space();
        if self.expression {
            if let Statement::ExpressionStatement(stmt) = &self.body.statements[0] {
                stmt.expression.gen(p);
            }
        } else {
            self.body.gen(p);
        }
    }
}

impl<'a> Gen for YieldExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"yield");
        if self.delegate {
            p.print_space();
            p.print(b'*');
        }

        if let Some(argument) = self.argument.as_ref() {
            p.print(b' ');
            argument.gen(p);
        }
    }
}

impl<'a> Gen for UpdateExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        let operator = self.operator.as_str().as_bytes();
        if self.prefix {
            p.print_space_before_operator(self.operator.into());
            p.print_str(operator);
            p.prev_op = Some(self.operator.into());
            p.prev_op_end = p.code().len();
            self.argument.gen(p);
        } else {
            self.argument.gen(p);
            p.print_str(operator);
            p.prev_op = Some(self.operator.into());
            p.prev_op_end = p.code().len();
        }
    }
}

impl<'a> Gen for UnaryExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        let operator = self.operator.as_str().as_bytes();
        if self.operator.is_keyword() {
            p.print_str(operator);
            p.print(b' ');
        } else {
            p.print_space_before_operator(self.operator.into());
            p.print_str(operator);
            p.prev_op = Some(self.operator.into());
            p.prev_op_end = p.code().len();
        }
        self.argument.gen(p);
    }
}

impl<'a> Gen for BinaryExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        self.left.gen(p);
        self.operator.gen(p);
        self.right.gen(p);
    }
}

impl Gen for BinaryOperator {
    fn gen(&self, p: &mut Printer) {
        let operator = self.as_str().as_bytes();
        if self.is_keyword() {
            p.print(b' ');
            p.print_str(operator);
            p.print(b' ');
        } else {
            let op: Operator = (*self).into();
            p.print_space_before_operator(op);
            p.print_str(operator);
            p.prev_op = Some(op);
            p.prev_op_end = p.code().len();
        }
    }
}

impl<'a> Gen for PrivateInExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        self.left.gen(p);
        p.print(b' ');
        p.print_str(b"in");
        p.print(b' ');
        self.right.gen(p);
    }
}

impl<'a> Gen for LogicalExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        self.left.gen(p);
        p.print_space();
        p.print_str(self.operator.as_str().as_bytes());
        p.print_space();
        self.right.gen(p);
    }
}

impl<'a> Gen for ConditionalExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        self.test.gen(p);
        p.print_space();
        p.print(b'?');
        p.print_space();
        self.consequent.gen(p);
        p.print_space();
        p.print(b':');
        p.print_space();
        self.alternate.gen(p);
    }
}

impl<'a> Gen for AssignmentExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        self.left.gen(p);
        p.print_space();
        p.print_str(self.operator.as_str().as_bytes());
        p.print_space();
        self.right.gen(p);
    }
}

impl<'a> Gen for AssignmentTarget<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::SimpleAssignmentTarget(target) => target.gen(p),
            Self::AssignmentTargetPattern(pat) => pat.gen(p),
        }
    }
}

impl<'a> Gen for SimpleAssignmentTarget<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.gen(p),
            Self::MemberAssignmentTarget(member_expr) => member_expr.gen(p),
        }
    }
}

impl<'a> Gen for AssignmentTargetPattern<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::ArrayAssignmentTarget(target) => target.gen(p),
            Self::ObjectAssignmentTarget(target) => target.gen(p),
        }
    }
}

impl<'a> Gen for ArrayAssignmentTarget<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print(b'[');
        p.print_list(&self.elements);
        if let Some(target) = &self.rest {
            p.print_comma();
            p.print_ellipsis();
            target.gen(p);
        }
        if self.trailing_comma.is_some() {
            p.print_comma();
        }
        p.print(b']');
    }
}

impl<'a> Gen for Option<AssignmentTargetMaybeDefault<'a>> {
    fn gen(&self, p: &mut Printer) {
        if let Some(arg) = self {
            arg.gen(p);
        }
    }
}

impl<'a> Gen for ObjectAssignmentTarget<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print(b'{');
        p.print_list(&self.properties);
        if let Some(target) = &self.rest {
            if !self.properties.is_empty() {
                p.print_comma();
            }
            p.print_ellipsis();
            target.gen(p);
        }
        p.print(b'}');
    }
}

impl<'a> Gen for AssignmentTargetMaybeDefault<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::AssignmentTarget(target) => target.gen(p),
            Self::AssignmentTargetWithDefault(target) => target.gen(p),
        }
    }
}

impl<'a> Gen for AssignmentTargetWithDefault<'a> {
    fn gen(&self, p: &mut Printer) {
        self.binding.gen(p);
        p.print_equal();
        self.init.gen(p);
    }
}

impl<'a> Gen for AssignmentTargetProperty<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::AssignmentTargetPropertyIdentifier(ident) => ident.gen(p),
            Self::AssignmentTargetPropertyProperty(prop) => prop.gen(p),
        }
    }
}

impl<'a> Gen for AssignmentTargetPropertyIdentifier<'a> {
    fn gen(&self, p: &mut Printer) {
        self.binding.gen(p);
        if let Some(expr) = &self.init {
            p.print_space();
            p.print_equal();
            p.print_space();
            expr.gen(p);
        }
    }
}

impl<'a> Gen for AssignmentTargetPropertyProperty<'a> {
    fn gen(&self, p: &mut Printer) {
        match &self.name {
            PropertyKey::Identifier(ident) => {
                ident.gen(p);
            }
            PropertyKey::PrivateIdentifier(ident) => {
                ident.gen(p);
            }
            PropertyKey::Expression(expr) => {
                p.print(b'[');
                expr.gen(p);
                p.print(b']');
            }
        }
        p.print_colon();
        p.print_space();
        self.binding.gen(p);
    }
}

impl<'a> Gen for SequenceExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_list(&self.expressions);
    }
}

impl<'a> Gen for ImportExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"import(");
        self.source.gen(p);
        if !self.arguments.is_empty() {
            p.print_comma();
            p.print_space();
            p.print_list(&self.arguments);
        }
        p.print(b')');
    }
}

impl<'a> Gen for TemplateLiteral<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print(b'`');
        let mut expressions = self.expressions.iter();

        for quasi in &self.quasis {
            p.print_str(quasi.value.raw.as_bytes());

            if let Some(expr) = expressions.next() {
                p.print_str(b"${");
                expr.gen(p);
                p.print(b'}');
            }
        }

        p.print(b'`');
    }
}

impl<'a> Gen for TaggedTemplateExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        self.tag.gen(p);
        self.quasi.gen(p);
    }
}

impl Gen for Super {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"super");
    }
}

impl<'a> Gen for AwaitExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"await ");
        self.argument.gen(p);
    }
}

impl<'a> Gen for ChainExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        match &self.expression {
            ChainElement::CallExpression(expr) => expr.gen(p),
            ChainElement::MemberExpression(expr) => expr.gen(p),
        }
    }
}

impl<'a> Gen for NewExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"new ");
        self.callee.gen(p);
        p.print(b'(');
        p.print_list(&self.arguments);
        p.print(b')');
    }
}

impl Gen for MetaProperty {
    fn gen(&self, p: &mut Printer) {
        self.meta.gen(p);
        p.print(b'.');
        self.property.gen(p);
    }
}

impl<'a> Gen for Class<'a> {
    fn gen(&self, p: &mut Printer) {
        self.decorators.gen(p);
        p.print_str(b"class");
        if let Some(id) = &self.id {
            p.print(b' ');
            id.gen(p);
        }
        if let Some(super_class) = self.super_class.as_ref() {
            p.print_str(b" extends ");
            super_class.gen(p);
        }
        p.print_space();
        p.print(b'{');
        p.print_newline();
        p.indent();
        for item in &self.body.body {
            p.print_semicolon_if_needed();
            p.print_indent();
            item.gen(p);
            if matches!(
                item,
                ClassElement::PropertyDefinition(_) | ClassElement::AccessorProperty(_)
            ) {
                p.print_semicolon_after_statement();
            } else {
                p.print_newline();
            }
        }
        p.needs_semicolon = false;
        p.dedent();
        p.print_indent();
        p.print(b'}');
    }
}

impl<'a> Gen for ClassElement<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::StaticBlock(elem) => elem.gen(p),
            Self::MethodDefinition(elem) => elem.gen(p),
            Self::PropertyDefinition(elem) => elem.gen(p),
            Self::AccessorProperty(elem) => elem.gen(p),
        }
    }
}

impl Gen for JSXIdentifier {
    fn gen(&self, p: &mut Printer) {
        p.print_str(self.name.as_bytes());
    }
}

impl<'a> Gen for JSXMemberExpressionObject<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::Identifier(ident) => ident.gen(p),
            Self::MemberExpression(member_expr) => member_expr.gen(p),
        }
    }
}

impl<'a> Gen for JSXMemberExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        self.object.gen(p);
        p.print(b'.');
        self.property.gen(p);
    }
}

impl<'a> Gen for JSXElementName<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::Identifier(identifier) => identifier.gen(p),
            Self::NamespacedName(namespaced_name) => namespaced_name.gen(p),
            Self::MemberExpression(member_expr) => member_expr.gen(p),
        }
    }
}

impl Gen for JSXNamespacedName {
    fn gen(&self, p: &mut Printer) {
        self.namespace.gen(p);
        p.print(b'.');
        self.property.gen(p);
    }
}

impl<'a> Gen for JSXAttributeName<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::Identifier(ident) => ident.gen(p),
            Self::NamespacedName(namespaced_name) => namespaced_name.gen(p),
        }
    }
}

impl<'a> Gen for JSXAttribute<'a> {
    fn gen(&self, p: &mut Printer) {
        self.name.gen(p);
        p.print(b'=');
        if let Some(value) = &self.value {
            value.gen(p);
        }
    }
}

impl Gen for JSXEmptyExpression {
    fn gen(&self, _: &mut Printer) {}
}

impl<'a> Gen for JSXExpression<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::Expression(expr) => expr.gen(p),
            Self::EmptyExpression(expr) => expr.gen(p),
        }
    }
}

impl<'a> Gen for JSXExpressionContainer<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print(b'{');
        self.expression.gen(p);
        p.print(b'}');
    }
}

impl<'a> Gen for JSXAttributeValue<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::Fragment(fragment) => fragment.gen(p),
            Self::Element(el) => el.gen(p),
            Self::StringLiteral(lit) => lit.gen(p),
            Self::ExpressionContainer(expr_container) => expr_container.gen(p),
        }
    }
}

impl<'a> Gen for JSXSpreadAttribute<'a> {
    fn gen(&self, p: &mut Printer) {
        self.argument.gen(p);
    }
}

impl<'a> Gen for JSXAttributeItem<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::Attribute(attr) => attr.gen(p),
            Self::SpreadAttribute(spread_attr) => spread_attr.gen(p),
        }
    }
}

impl<'a> Gen for JSXOpeningElement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"<");
        self.name.gen(p);
        for attr in &self.attributes {
            attr.gen(p);
        }
        if self.self_closing {
            p.print_space();
            p.print_str(b"/>");
        } else {
            p.print(b'>');
        }
    }
}

impl<'a> Gen for JSXClosingElement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"</");
        self.name.gen(p);
        p.print(b'>');
    }
}

impl<'a> Gen for JSXElement<'a> {
    fn gen(&self, p: &mut Printer) {
        self.opening_element.gen(p);
        for child in &self.children {
            child.gen(p);
        }
        if let Some(closing_element) = &self.closing_element {
            closing_element.gen(p);
        }
    }
}

impl Gen for JSXOpeningFragment {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"<>");
    }
}

impl Gen for JSXClosingFragment {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"</>");
    }
}

impl Gen for JSXText {
    fn gen(&self, p: &mut Printer) {
        p.print_str(self.value.as_bytes());
    }
}

impl<'a> Gen for JSXSpreadChild<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"...");
        self.expression.gen(p);
    }
}

impl<'a> Gen for JSXChild<'a> {
    fn gen(&self, p: &mut Printer) {
        match self {
            Self::Fragment(fragment) => fragment.gen(p),
            Self::Element(el) => el.gen(p),
            Self::Spread(spread) => spread.expression.gen(p),
            Self::ExpressionContainer(expr_container) => expr_container.gen(p),
            Self::Text(text) => text.gen(p),
        }
    }
}

impl<'a> Gen for JSXFragment<'a> {
    fn gen(&self, p: &mut Printer) {
        self.opening_fragment.gen(p);
        for child in &self.children {
            child.gen(p);
        }
        self.closing_fragment.gen(p);
    }
}

impl<'a> Gen for StaticBlock<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_str(b"static");
        p.print_space();
        p.print(b'{');
        p.print_newline();
        p.indent();
        for stmt in &self.body {
            p.print_semicolon_if_needed();
            stmt.gen(p);
        }
        p.dedent();
        p.needs_semicolon = false;
        p.print_indent();
        p.print(b'}');
    }
}

impl<'a> Gen for MethodDefinition<'a> {
    fn gen(&self, p: &mut Printer) {
        self.decorators.gen(p);

        if self.r#static {
            p.print_str(b"static ");
        }

        match &self.kind {
            MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => {}
            MethodDefinitionKind::Get => p.print_str(b"get "),
            MethodDefinitionKind::Set => p.print_str(b"set "),
        }

        if self.value.r#async {
            p.print_str(b"async ");
        }

        if self.value.generator {
            p.print_str(b"*");
        }

        if self.computed {
            p.print(b'[');
        }
        self.key.gen(p);
        if self.computed {
            p.print(b']');
        }
        p.print(b'(');
        self.value.params.gen(p);
        p.print(b')');
        p.print_space();
        if let Some(body) = &self.value.body {
            body.gen(p);
        }
    }
}

impl<'a> Gen for PropertyDefinition<'a> {
    fn gen(&self, p: &mut Printer) {
        self.decorators.gen(p);
        if self.r#static {
            p.print_str(b"static ");
        }
        if self.computed {
            p.print(b'[');
        }
        self.key.gen(p);
        if self.computed {
            p.print(b']');
        }
        if let Some(value) = &self.value {
            p.print_space();
            p.print_equal();
            p.print_space();
            value.gen(p);
        }
    }
}

impl<'a> Gen for AccessorProperty<'a> {
    fn gen(&self, p: &mut Printer) {
        if self.r#static {
            p.print_str(b"static ");
        }
        p.print_str(b"accessor ");
        if self.computed {
            p.print(b'[');
        }
        self.key.gen(p);
        if self.computed {
            p.print(b']');
        }
        if let Some(value) = &self.value {
            p.print_space();
            p.print_equal();
            p.print_space();
            value.gen(p);
        }
    }
}

impl Gen for PrivateIdentifier {
    fn gen(&self, p: &mut Printer) {
        p.print(b'#');
        p.print_str(self.name.as_bytes());
    }
}

impl<'a> Gen for BindingPattern<'a> {
    fn gen(&self, p: &mut Printer) {
        match &self {
            BindingPattern::BindingIdentifier(ident) => ident.gen(p),
            BindingPattern::ObjectPattern(pattern) => pattern.gen(p),
            BindingPattern::RestElement(elem) => elem.gen(p),
            BindingPattern::ArrayPattern(pattern) => pattern.gen(p),
            BindingPattern::AssignmentPattern(pattern) => pattern.gen(p),
        }
    }
}

impl<'a> Gen for ObjectPattern<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print(b'{');
        p.print_space();
        p.print_list(&self.properties);
        if let Some(rest) = &self.rest {
            if !self.properties.is_empty() {
                p.print_comma();
            }
            rest.gen(p);
        }
        p.print_space();
        p.print(b'}');
    }
}

impl<'a> Gen for BindingProperty<'a> {
    fn gen(&self, p: &mut Printer) {
        if self.computed {
            p.print(b'[');
        }
        self.key.gen(p);
        if self.computed {
            p.print(b']');
        }
        p.print(b':');
        p.print_space();
        self.value.gen(p);
    }
}

impl<'a> Gen for RestElement<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print_ellipsis();
        self.argument.gen(p);
    }
}

impl<'a> Gen for ArrayPattern<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print(b'[');
        for (index, item) in self.elements.iter().enumerate() {
            if index != 0 {
                p.print_comma();
            }
            if let Some(item) = item {
                item.gen(p);
            }
            if index == self.elements.len() - 1 && (item.is_none() || self.rest.is_some()) {
                p.print_comma();
            }
        }
        if let Some(rest) = &self.rest {
            rest.gen(p);
        }
        p.print(b']');
    }
}

impl<'a> Gen for AssignmentPattern<'a> {
    fn gen(&self, p: &mut Printer) {
        self.left.gen(p);
        p.print_space();
        p.print_equal();
        p.print_space();
        self.right.gen(p);
    }
}

impl<'a> Gen for Vec<'a, Decorator<'a>> {
    fn gen(&self, p: &mut Printer) {
        for decorator in self {
            decorator.gen(p);
            p.print(b' ');
        }
    }
}

impl<'a> Gen for Decorator<'a> {
    fn gen(&self, p: &mut Printer) {
        p.print(b'@');
        self.expression.gen(p);
    }
}
