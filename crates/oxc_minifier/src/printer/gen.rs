use oxc_allocator::{Box, Vec};
#[allow(clippy::wildcard_imports)]
use oxc_hir::hir::*;
use oxc_hir::precedence;
use oxc_syntax::{
    operator::{
        AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
    },
    precedence::{GetPrecedence, Precedence},
    NumberBase,
};

use super::{Context, Operator, Printer, Separator};

pub trait Gen {
    fn gen(&self, p: &mut Printer, ctx: Context) {}
}

pub trait GenExpr {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {}
}

impl<'a, T> Gen for Box<'a, T>
where
    T: Gen,
{
    fn gen(&self, p: &mut Printer, ctx: Context) {
        (**self).gen(p, ctx);
    }
}

impl<'a, T> GenExpr for Box<'a, T>
where
    T: GenExpr,
{
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        (**self).gen_expr(p, precedence, ctx);
    }
}

impl<'a> Gen for Program<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        for directive in &self.directives {
            directive.gen(p, ctx);
        }
        for stmt in &self.body {
            p.print_semicolon_if_needed();
            stmt.gen(p, ctx);
        }
    }
}

impl <'a> Gen for Hashbang<'a> {
	fn gen(&self, p: &mut Printer, ctx: Context) {
		p.print(b'#');
		p.print(b'!');
		p.print_str(self.value.as_bytes());
	}
}

impl<'a> Gen for Directive<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print(b'"');
        p.print_str(self.directive.as_bytes());
        p.print(b'"');
        p.print_semicolon();
    }
}

impl<'a> Gen for Statement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::BlockStatement(stmt) => stmt.gen(p, ctx),
            Self::BreakStatement(stmt) => stmt.gen(p, ctx),
            Self::ContinueStatement(stmt) => stmt.gen(p, ctx),
            Self::DebuggerStatement(stmt) => stmt.gen(p, ctx),
            Self::DoWhileStatement(stmt) => stmt.gen(p, ctx),
            Self::ExpressionStatement(stmt) => stmt.gen(p, ctx),
            Self::ForInStatement(stmt) => stmt.gen(p, ctx),
            Self::ForOfStatement(stmt) => stmt.gen(p, ctx),
            Self::ForStatement(stmt) => stmt.gen(p, ctx),
            Self::IfStatement(stmt) => stmt.gen(p, ctx),
            Self::LabeledStatement(stmt) => stmt.gen(p, ctx),
            Self::ModuleDeclaration(decl) => decl.gen(p, ctx),
            Self::ReturnStatement(stmt) => stmt.gen(p, ctx),
            Self::SwitchStatement(stmt) => stmt.gen(p, ctx),
            Self::ThrowStatement(stmt) => stmt.gen(p, ctx),
            Self::TryStatement(stmt) => stmt.gen(p, ctx),
            Self::WhileStatement(stmt) => stmt.gen(p, ctx),
            Self::WithStatement(stmt) => stmt.gen(p, ctx),
            Self::Declaration(decl) => decl.gen(p, ctx),
        }
    }
}

impl<'a> Gen for Option<Statement<'a>> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Some(stmt) => stmt.gen(p, ctx),
            None => p.print(b';'),
        }
    }
}

impl<'a> Gen for ExpressionStatement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.start_of_stmt = p.code_len();
        self.expression.gen_expr(p, Precedence::lowest(), Context::default());
        if let Expression::Identifier(ident) = &self.expression
        && ident.name == "let" {
            p.print_semicolon();
        } else {
            p.print_semicolon_after_statement();
        }
    }
}

impl<'a> Gen for IfStatement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        print_if(self, p, ctx);
    }
}

fn print_if(if_stmt: &IfStatement<'_>, p: &mut Printer, ctx: Context) {
    p.print_str(b"if");
    p.print(b'(');
    if_stmt.test.gen_expr(p, Precedence::lowest(), Context::default());
    p.print(b')');

    match &if_stmt.consequent {
        Some(Statement::BlockStatement(block)) => {
            p.print_block1(block, ctx);
        }
        Some(stmt) if wrap_to_avoid_ambiguous_else(stmt) => {
            p.print(b'{');
            stmt.gen(p, ctx);
            p.print(b'}');
            p.needs_semicolon = false;
        }
        Some(stmt) => {
            stmt.gen(p, ctx);
        }
        None => {
            p.print(b';');
        }
    }

    if let Some(alternate) = if_stmt.alternate.as_ref() {
        p.print_semicolon_if_needed();
        p.print(b' ');
        p.print_str(b"else");
        p.print(b' ');
        match alternate {
            Statement::BlockStatement(block) => {
                p.print_block1(block, ctx);
            }
            Statement::IfStatement(if_stmt) => {
                print_if(if_stmt, p, ctx);
            }
            _ => {
                alternate.gen(p, ctx);
            }
        }
    }
}

// <https://github.com/evanw/esbuild/blob/e6a8169c3a574f4c67d4cdd5f31a938b53eb7421/internal/js_printer/js_printer.go#L3444>
fn wrap_to_avoid_ambiguous_else(stmt: &Statement) -> bool {
    let mut current = stmt;
    loop {
        current = match current {
            Statement::IfStatement(Box(IfStatement { alternate, .. })) => {
                if let Some(stmt) = &alternate {
                    stmt
                } else {
                    return true;
                }
            }
            Statement::ForStatement(Box(ForStatement { body, .. }))
            | Statement::ForOfStatement(Box(ForOfStatement { body, .. }))
            | Statement::ForInStatement(Box(ForInStatement { body, .. }))
            | Statement::WhileStatement(Box(WhileStatement { body, .. }))
            | Statement::WithStatement(Box(WithStatement { body, .. }))
            | Statement::LabeledStatement(Box(LabeledStatement { body, .. })) => {
                if let Some(stmt) = &body {
                    stmt
                } else {
                    return false;
                }
            }
            _ => return false,
        }
    }
    false
}

impl<'a> Gen for BlockStatement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_block1(self, ctx);
    }
}

impl<'a> Gen for ForStatement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"for");
        p.print(b'(');

        if let Some(init) = self.init.as_ref() {
            let ctx = Context::empty();
            match init {
                ForStatementInit::Expression(expr) => {
                    expr.gen_expr(p, Precedence::lowest(), ctx);
                }
                ForStatementInit::VariableDeclaration(var) => var.gen(p, ctx),
            }
        }

        p.print_semicolon();

        if let Some(test) = self.test.as_ref() {
            test.gen_expr(p, Precedence::lowest(), Context::default());
        }

        p.print_semicolon();

        if let Some(update) = self.update.as_ref() {
            update.gen_expr(p, Precedence::lowest(), Context::default());
        }

        p.print(b')');
        self.body.gen(p, ctx);
    }
}

impl<'a> Gen for ForInStatement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"for");
        p.print(b'(');
        self.left.gen(p, ctx);
        p.print(b' ');
        p.print_str(b"in");
        p.print(b' ');
        self.right.gen_expr(p, Precedence::lowest(), Context::default());
        p.print(b')');
        self.body.gen(p, ctx);
    }
}

impl<'a> Gen for ForOfStatement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"for");
        if self.r#await {
            p.print_str(b" await");
        }
        p.print(b'(');
        self.left.gen(p, ctx);
        p.print(b' ');
        p.print_str(b"of");
        p.print(b' ');
        self.right.gen_expr(p, Precedence::Assign, Context::default());
        p.print(b')');
        self.body.gen(p, ctx);
    }
}

impl<'a> Gen for ForStatementLeft<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match &self {
            ForStatementLeft::VariableDeclaration(var) => var.gen(p, ctx),
            ForStatementLeft::AssignmentTarget(target) => target.gen(p, ctx),
        }
    }
}

impl<'a> Gen for WhileStatement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"while");
        p.print(b'(');
        self.test.gen_expr(p, Precedence::lowest(), Context::default());
        p.print(b')');
        self.body.gen(p, ctx);
    }
}

impl<'a> Gen for DoWhileStatement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"do");
        p.print(b' ');
        if let Some(Statement::BlockStatement(block)) = &self.body {
            p.print_block1(block, ctx);
        } else {
            self.body.gen(p, ctx);
            p.print_semicolon_if_needed();
        }
        p.print_str(b"while");
        p.print(b'(');
        self.test.gen_expr(p, Precedence::lowest(), Context::default());
        p.print(b')');
        p.print_semicolon_after_statement();
    }
}

impl Gen for ContinueStatement {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"continue");
        if let Some(label) = &self.label {
            p.print(b' ');
            label.gen(p, ctx);
        }
        p.print_semicolon_after_statement();
    }
}

impl Gen for BreakStatement {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"break");
        if let Some(label) = &self.label {
            p.print(b' ');
            label.gen(p, ctx);
        }
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for SwitchStatement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"switch");
        p.print(b'(');
        self.discriminant.gen_expr(p, Precedence::lowest(), Context::default());
        p.print(b')');
        p.print(b'{');
        for case in &self.cases {
            case.gen(p, ctx);
        }
        p.print(b'}');
        p.needs_semicolon = false;
    }
}

impl<'a> Gen for SwitchCase<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_semicolon_if_needed();
        match &self.test {
            Some(test) => {
                p.print_str(b"case");
                p.print(b' ');
                test.gen_expr(p, Precedence::lowest(), Context::default());
            }
            None => p.print_str(b"default"),
        }
        p.print_colon();
        for item in &self.consequent {
            p.print_semicolon_if_needed();
            item.gen(p, ctx);
        }
    }
}

impl<'a> Gen for ReturnStatement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"return");
        if let Some(arg) = &self.argument {
            p.print(b' ');
            arg.gen_expr(p, Precedence::lowest(), Context::default());
        }
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for LabeledStatement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.label.gen(p, ctx);
        p.print_colon();
        self.body.gen(p, ctx);
    }
}

impl<'a> Gen for TryStatement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"try");
        p.print_block1(&self.block, ctx);
        if let Some(handler) = &self.handler {
            p.print_str(b"catch");
            if let Some(param) = &handler.param {
                p.print_str(b"(");
                param.gen(p, ctx);
                p.print_str(b")");
            }
            p.print_block1(&handler.body, ctx);
        }
        if let Some(finalizer) = &self.finalizer {
            p.print_str(b"finally");
            p.print_block1(finalizer, ctx);
        }
    }
}

impl<'a> Gen for ThrowStatement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"throw");
        p.print(b' ');
        self.argument.gen_expr(p, Precedence::lowest(), Context::default());
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for WithStatement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"with");
        p.print(b'(');
        self.object.gen_expr(p, Precedence::lowest(), Context::default());
        p.print(b')');
        self.body.gen(p, ctx);
    }
}

impl Gen for DebuggerStatement {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"debugger");
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for ModuleDeclaration<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::ImportDeclaration(decl) => decl.gen(p, ctx),
            Self::ExportAllDeclaration(decl) => decl.gen(p, ctx),
            Self::ExportDefaultDeclaration(decl) => decl.gen(p, ctx),
            Self::ExportNamedDeclaration(decl) => decl.gen(p, ctx),
        }
    }
}

impl<'a> Gen for Declaration<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::VariableDeclaration(stmt) => {
                stmt.gen(p, ctx);
                p.print_semicolon_after_statement();
            }
            Self::FunctionDeclaration(stmt) => {
                stmt.gen(p, ctx);
            }
            Self::ClassDeclaration(declaration) => {
                declaration.gen(p, ctx);
            }
            Self::TSEnumDeclaration(_) => {}
        }
    }
}

impl<'a> Gen for VariableDeclaration<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(match self.kind {
            VariableDeclarationKind::Const => b"const",
            VariableDeclarationKind::Let => b"let",
            VariableDeclarationKind::Var => b"var",
        });
        p.print(b' ');
        p.print_list(&self.declarations, ctx);
    }
}

impl<'a> Gen for VariableDeclarator<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.id.gen(p, ctx);
        if let Some(init) = &self.init {
            p.print_equal();
            init.gen_expr(p, Precedence::Assign, ctx);
        }
    }
}

impl<'a> Gen for Function<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        let n = p.code_len();
        let wrap = self.is_expression() && (p.start_of_stmt == n || p.start_of_default_export == n);
        p.wrap(wrap, |p| {
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
                id.gen(p, ctx);
            }
            p.print(b'(');
            self.params.gen(p, ctx);
            p.print(b')');
            if let Some(body) = &self.body {
                body.gen(p, ctx);
            }
        });
    }
}

impl<'a> Gen for FunctionBody<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print(b'{');
        for directive in &self.directives {
            directive.gen(p, ctx);
        }
        p.needs_semicolon = if let Some(Statement::ExpressionStatement(expr_stmt)) = self.statements.get(0)
            && matches!(expr_stmt.expression, Expression::StringLiteral(_)) {
                true
            } else {
                false
            };
        for stmt in &self.statements {
            p.print_semicolon_if_needed();
            stmt.gen(p, ctx);
        }
        p.print(b'}');
        p.needs_semicolon = false;
    }
}

impl<'a> Gen for FormalParameter<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.decorators.gen(p, ctx);
        self.pattern.gen(p, ctx);
    }
}

impl<'a> Gen for FormalParameters<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_list(&self.items, ctx);
        if let Some(rest) = &self.rest {
            if !self.items.is_empty() {
                p.print_comma();
            }
            rest.gen(p, ctx);
        }
    }
}

impl<'a> Gen for ImportDeclaration<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"import ");
        if self.specifiers.is_empty() {
            p.print(b'\'');
            p.print_str(self.source.value.as_bytes());
            p.print(b'\'');
            self.assertions.gen(p, ctx);
            p.print_semicolon_after_statement();
            return;
        }

        let mut in_block = false;
        for (index, specifier) in self.specifiers.iter().enumerate() {
            match specifier {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                    if in_block {
                        p.print_str(b"},");
                        in_block = false;
                    } else if index != 0 {
                        p.print_comma();
                    }
                    spec.local.gen(p, ctx);
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                    if in_block {
                        p.print_str(b"},");
                        in_block = false;
                    } else if index != 0 {
                        p.print_comma();
                    }
                    p.print_str(b"* as ");
                    spec.local.gen(p, ctx);
                }
                ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                    if in_block {
                        p.print_comma();
                    } else {
                        if index != 0 {
                            p.print_comma();
                        }
                        in_block = true;
                        p.print(b'{');
                    }

                    let imported_name = match &spec.imported {
                        ModuleExportName::Identifier(identifier) => {
                            identifier.gen(p, ctx);
                            identifier.name.as_bytes()
                        }
                        ModuleExportName::StringLiteral(literal) => {
                            literal.gen(p, ctx);
                            literal.value.as_bytes()
                        }
                    };

                    let local_name = spec.local.name.as_bytes();

                    if imported_name != local_name {
                        p.print_str(b" as ");
                        spec.local.gen(p, ctx);
                    }
                }
            }
        }
        if in_block {
            p.print(b'}');
        }
        p.print_str(b" from ");
        self.source.gen(p, ctx);
        self.assertions.gen(p, ctx);
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for Option<Vec<'a, ImportAttribute>> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        if let Some(assertions) = &self {
            p.print_str(b"assert");
            p.print_block(assertions, Separator::Comma, ctx);
        };
    }
}

impl Gen for ImportAttribute {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match &self.key {
            ImportAttributeKey::Identifier(identifier) => {
                p.print_str(identifier.name.as_bytes());
            }
            ImportAttributeKey::StringLiteral(literal) => literal.gen(p, ctx),
        };
        p.print_colon();
        self.value.gen(p, ctx);
    }
}

impl<'a> Gen for ExportNamedDeclaration<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"export ");
        match &self.declaration {
            Some(decl) => decl.gen(p, ctx),
            None => {
                p.print(b'{');
                if !self.specifiers.is_empty() {
                    p.print_list(&self.specifiers, ctx);
                }
                p.print(b'}');
                if let Some(source) = &self.source {
                    p.print_str(b"from");
                    source.gen(p, ctx);
                }
                p.print_semicolon_after_statement();
            }
        }
    }
}

impl Gen for ExportSpecifier {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.local.gen(p, ctx);
        if self.local.name() != self.exported.name() {
            p.print_str(b" as ");
            self.exported.gen(p, ctx);
        }
    }
}

impl Gen for ModuleExportName {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::Identifier(identifier) => {
                p.print_str(identifier.name.as_bytes());
            }
            Self::StringLiteral(literal) => literal.gen(p, ctx),
        };
    }
}

impl<'a> Gen for ExportAllDeclaration<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"export");
        p.print(b'*');

        if let Some(exported) = &self.exported {
            p.print_str(b"as ");
            exported.gen(p, ctx);
        }

        p.print_str(b" from");
        self.source.gen(p, ctx);
        self.assertions.gen(p, ctx);

        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for ExportDefaultDeclaration<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"export default ");
        self.declaration.gen(p, ctx);
    }
}
impl<'a> Gen for ExportDefaultDeclarationKind<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::Expression(expr) => {
                p.start_of_default_export = p.code_len();
                expr.gen_expr(p, Precedence::Assign, Context::default());
                p.print_semicolon_after_statement();
            }
            Self::FunctionDeclaration(fun) => fun.gen(p, ctx),
            Self::ClassDeclaration(value) => value.gen(p, ctx),
            Self::TSEnumDeclaration(_) => {}
        }
    }
}

impl<'a> GenExpr for Expression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        match self {
            Self::BooleanLiteral(lit) => lit.gen(p, ctx),
            Self::NullLiteral(lit) => lit.gen(p, ctx),
            Self::NumberLiteral(lit) => lit.gen(p, ctx),
            Self::BigintLiteral(lit) => lit.gen(p, ctx),
            Self::RegExpLiteral(lit) => lit.gen(p, ctx),
            Self::StringLiteral(lit) => lit.gen(p, ctx),
            Self::Identifier(ident) => ident.gen(p, ctx),
            Self::ThisExpression(expr) => expr.gen(p, ctx),
            Self::MemberExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::CallExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::ArrayExpression(expr) => expr.gen(p, ctx),
            Self::ObjectExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::FunctionExpression(expr) => expr.gen(p, ctx),
            Self::ArrowExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::YieldExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::UpdateExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::UnaryExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::BinaryExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::PrivateInExpression(expr) => expr.gen(p, ctx),
            Self::LogicalExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::ConditionalExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::AssignmentExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::SequenceExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::ImportExpression(expr) => expr.gen(p, ctx),
            Self::TemplateLiteral(literal) => literal.gen(p, ctx),
            Self::TaggedTemplateExpression(expr) => expr.gen(p, ctx),
            Self::Super(sup) => sup.gen(p, ctx),
            Self::AwaitExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::ChainExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::NewExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::MetaProperty(expr) => expr.gen(p, ctx),
            Self::ClassExpression(expr) => expr.gen(p, ctx),
            Self::JSXElement(el) => el.gen(p, ctx),
            Self::JSXFragment(fragment) => fragment.gen(p, ctx),
        }
    }
}

impl Gen for IdentifierReference {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        if p.mangle && let Some(symbol_id) = p.symbol_table.get_reference(self.reference_id).symbol_id {
            p.print_symbol(symbol_id, &self.name);
        } else {
            p.print_str(self.name.as_bytes());
        }
    }
}

impl Gen for IdentifierName {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(self.name.as_bytes());
    }
}

impl Gen for BindingIdentifier {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_symbol(self.symbol_id, &self.name);
    }
}

impl Gen for LabelIdentifier {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(self.name.as_bytes());
    }
}

impl Gen for BooleanLiteral {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(self.as_str().as_bytes());
    }
}

impl Gen for NullLiteral {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_space_before_identifier();
        p.print_str(b"null");
    }
}

impl<'a> Gen for NumberLiteral<'a> {
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_space_before_identifier();

        let result = if self.base == NumberBase::Float {
            print_non_negative_float(self.value, p)
        } else {
            let value = self.value as u64;
            // If integers less than 1000, we know that exponential notation will always be longer than
            // the integer representation. This is not the case for 1000 which is "1e3".
            if value < 1000 {
                format!("{value}")
            } else if (1_000_000_000_000..=0xFFFF_FFFF_FFFF_F800).contains(&value) {
                let hex = format!("{value:#x}");
                let result = print_non_negative_float(self.value, p);
                if hex.len() < result.len() { hex } else { result }
            } else {
                print_non_negative_float(self.value, p)
            }
        };
        let bytes = result.as_bytes();
        p.print_str(bytes);

        // We'll need a space before "." if it could be parsed as a decimal point
        if !bytes.iter().any(|&b| matches!(b, b'.' | b'e' | b'x')) {
            p.need_space_before_dot = p.code_len();
        }
    }
}

// TODO: refactor this with less allocations
fn print_non_negative_float(value: f64, p: &mut Printer) -> String {
    let mut result = value.to_string();
    let chars = result.as_bytes();
    let len = chars.len();
    let dot = chars.iter().position(|&c| c == b'.');
    let u8_to_string = |num: &[u8]| unsafe { String::from_utf8_unchecked(num.to_vec()) };

    if dot == Some(1) && chars[0] == b'0' {
        // Strip off the leading zero when minifying
        // "0.5" => ".5"
        let stripped_result = &chars[1..];
        // after stripping the leading zero, the after dot position will be start from 1
        let after_dot = 1;

        // Try using an exponent
        // "0.001" => "1e-3"
        if stripped_result[after_dot] == b'0' {
            let mut i = after_dot + 1;
            while stripped_result[i] == b'0' {
                i += 1;
            }
            let remaining = &stripped_result[i..];
            let exponent = format!("-{}", remaining.len() - after_dot + i);

            // Only switch if it's actually shorter
            if stripped_result.len() > remaining.len() + 1 + exponent.len() {
                result = format!("{}e{}", u8_to_string(remaining), exponent);
            } else {
                result = u8_to_string(stripped_result);
            }
        } else {
            result = u8_to_string(stripped_result);
        }
    } else if chars[len - 1] == b'0' {
        // Simplify numbers ending with "0" by trying to use an exponent
        // "1000" => "1e3"
        let mut i = len - 1;
        while i > 0 && chars[i - 1] == b'0' {
            i -= 1;
        }
        let remaining = &chars[0..i];
        let exponent = format!("{}", chars.len() - i);

        // Only switch if it's actually shorter
        if chars.len() > remaining.len() + 1 + exponent.len() {
            result = format!("{}e{}", u8_to_string(remaining), exponent);
        } else {
            result = u8_to_string(chars);
        }
    }

    result
}

impl Gen for BigintLiteral {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(self.value.to_string().as_bytes());
        p.print(b'n');
    }
}

impl Gen for RegExpLiteral {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        let last = p.peek_nth(0);
        // Avoid forming a single-line comment or "</script" sequence
        if Some('/') == last
            || (Some('<') == last
                && self.regex.pattern.as_str().to_lowercase().starts_with("script"))
        {
            p.print(b' ');
        }
        p.print(b'/');
        p.print_str(self.regex.pattern.as_bytes());
        p.print(b'/');
        p.print_str(self.regex.flags.to_string().as_bytes());
        p.prev_reg_exp_end = p.code().len();
    }
}

impl Gen for StringLiteral {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print(b'\'');
        for c in self.value.chars() {
            p.print_str(c.escape_default().to_string().as_bytes());
        }
        p.print(b'\'');
    }
}

impl Gen for ThisExpression {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_space_before_identifier();
        p.print_str(b"this");
    }
}

impl<'a> GenExpr for MemberExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        p.wrap(precedence > self.precedence(), |p| match self {
            Self::ComputedMemberExpression(expr) => {
                expr.gen_expr(p, self.precedence(), ctx.and_in(true));
            }
            Self::StaticMemberExpression(expr) => expr.gen_expr(p, self.precedence(), ctx),
            Self::PrivateFieldExpression(expr) => expr.gen_expr(p, self.precedence(), ctx),
        });
    }
}

impl<'a> GenExpr for ComputedMemberExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        self.object.gen_expr(p, Precedence::Postfix, ctx);
        if self.optional {
            p.print_str(b"?.");
        }
        p.print(b'[');
        self.expression.gen_expr(p, Precedence::lowest(), ctx);
        p.print(b']');
    }
}

impl<'a> GenExpr for StaticMemberExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        self.object.gen_expr(p, Precedence::Postfix, ctx);
        if self.optional {
            p.print(b'?');
        } else if p.need_space_before_dot == p.code_len() {
            // `0.toExponential()` is invalid, add a space before the dot, `0 .toExponential()` is valid
            p.print(b' ');
        }
        p.print(b'.');
        self.property.gen(p, ctx);
    }
}

impl<'a> GenExpr for PrivateFieldExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        self.object.gen_expr(p, Precedence::Postfix, ctx);
        if self.optional {
            p.print_str(b"?");
        }
        p.print(b'.');
        self.field.gen(p, ctx);
    }
}

impl<'a> GenExpr for CallExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        p.wrap(precedence > self.precedence(), |p| {
            self.callee.gen_expr(p, self.precedence(), ctx);
            if self.optional {
                p.print_str(b"?.");
            }
            p.print(b'(');
            p.print_list(&self.arguments, ctx);
            p.print(b')');
        });
    }
}

impl<'a> Gen for Argument<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::SpreadElement(elem) => elem.gen(p, ctx),
            Self::Expression(elem) => elem.gen_expr(p, Precedence::Assign, Context::default()),
        }
    }
}

impl<'a> Gen for ArrayExpressionElement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::Expression(expr) => expr.gen_expr(p, Precedence::Assign, Context::default()),
            Self::SpreadElement(elem) => elem.gen(p, ctx),
            Self::Elision(_span) => {}
        }
    }
}

impl<'a> Gen for SpreadElement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_ellipsis();
        self.argument.gen_expr(p, Precedence::Assign, Context::default());
    }
}

impl<'a> Gen for ArrayExpression<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print(b'[');
        p.print_list(&self.elements, ctx);
        if self.trailing_comma.is_some() {
            p.print_comma();
        }
        p.print(b']');
    }
}

impl<'a> GenExpr for ObjectExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        let n = p.code_len();
        p.wrap(p.start_of_stmt == n || p.start_of_arrow_expr == n, |p| {
            p.print(b'{');
            for (i, item) in self.properties.iter().enumerate() {
                if i != 0 {
                    p.print_comma();
                }
                item.gen(p, ctx);
            }
            p.print(b'}');
        });
    }
}

impl<'a> Gen for ObjectPropertyKind<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::ObjectProperty(prop) => prop.gen(p, ctx),
            Self::SpreadProperty(elem) => elem.gen(p, ctx),
        }
    }
}

impl<'a> Gen for ObjectProperty<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
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
                self.key.gen(p, ctx);
                if self.computed {
                    p.print(b']');
                }
                p.print(b'(');
                func.params.gen(p, ctx);
                p.print(b')');
                if let Some(body) = &func.body {
                    body.gen(p, ctx);
                }
                return;
            }
        }
        if self.computed {
            p.print(b'[');
        }
        self.key.gen(p, ctx);
        if self.computed {
            p.print(b']');
        }
        p.print_colon();
        self.value.gen_expr(p, Precedence::Assign, Context::default());
    }
}

impl<'a> Gen for PropertyKey<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::Identifier(ident) => ident.gen(p, ctx),
            Self::PrivateIdentifier(ident) => ident.gen(p, ctx),
            Self::Expression(expr) => expr.gen_expr(p, Precedence::Assign, Context::default()),
        }
    }
}

impl<'a> Gen for PropertyValue<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::Pattern(pattern) => pattern.gen(p, ctx),
            Self::Expression(expr) => expr.gen_expr(p, Precedence::Assign, Context::default()),
        }
    }
}

impl<'a> GenExpr for ArrowExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        p.wrap(precedence > Precedence::Assign, |p| {
            if self.r#async {
                p.print_str(b"async");
            }
            // No wrap for `a => {}`
            let nowrap = self.params.rest.is_none()
                && self.params.items.len() == 1
                && self.params.items[0].pattern.is_binding_identifier();
            if nowrap && self.r#async {
                p.print(b' ');
            }
            p.wrap(!nowrap, |p| {
                self.params.gen(p, ctx);
            });
            p.print_str(b"=>");
            if self.expression {
                if let Statement::ExpressionStatement(stmt) = &self.body.statements[0] {
                    p.start_of_arrow_expr = p.code_len();
                    stmt.expression.gen_expr(p, Precedence::Assign, ctx);
                }
            } else {
                self.body.gen(p, ctx);
            }
        });
    }
}

impl<'a> GenExpr for YieldExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        p.wrap(precedence >= self.precedence(), |p| {
            p.print_str(b"yield");
            if self.delegate {
                p.print(b'*');
            }

            if let Some(argument) = self.argument.as_ref() {
                if !self.delegate {
                    p.print(b' ');
                }
                argument.gen_expr(p, Precedence::Assign, ctx);
            }
        });
    }
}

impl<'a> GenExpr for UpdateExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        let operator = self.operator.as_str().as_bytes();
        p.wrap(precedence > self.precedence(), |p| {
            if self.prefix {
                p.print_space_before_operator(self.operator.into());
                p.print_str(operator);
                p.prev_op = Some(self.operator.into());
                p.prev_op_end = p.code().len();
                self.argument.gen_expr(p, Precedence::Prefix, ctx);
            } else {
                self.argument.gen_expr(p, Precedence::Postfix, ctx);
                p.print_str(operator);
                p.prev_op = Some(self.operator.into());
                p.prev_op_end = p.code().len();
            }
        });
    }
}

impl<'a> GenExpr for UnaryExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        p.wrap(precedence > self.precedence() || precedence == Precedence::Exponential, |p| {
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
            self.argument.gen_expr(p, Precedence::Prefix, ctx);
        });
    }
}

impl<'a> GenExpr for BinaryExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        let wrap_in = self.operator == BinaryOperator::In && !ctx.has_in();
        let wrap = precedence > self.precedence() || wrap_in;
        p.wrap(wrap, |p| {
            self.left.gen_expr(p, self.precedence(), ctx);
            if self.operator.is_keyword() {
                p.print_space_before_identifier();
            }
            self.operator.gen(p, ctx);
            self.right.gen_expr(p, self.precedence(), ctx.union_in_if(wrap));
        });
    }
}

impl Gen for BinaryOperator {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        let operator = self.as_str().as_bytes();
        if self.is_keyword() {
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
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.left.gen(p, ctx);
        p.print(b' ');
        p.print_str(b"in");
        p.print(b' ');
        self.right.gen_expr(p, Precedence::Shift, Context::default());
    }
}

impl<'a> GenExpr for LogicalExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        // Logical expressions and coalesce expressions cannot be mixed (Syntax Error).
        let mixed = matches!(
            (precedence, self.precedence()),
            (Precedence::Coalesce, Precedence::LogicalAnd | Precedence::LogicalOr)
        );
        p.wrap(mixed || (precedence > self.precedence()), |p| {
            self.left.gen_expr(p, self.precedence(), ctx);
            p.print_str(self.operator.as_str().as_bytes());
            let precedence = match self.operator {
                LogicalOperator::And | LogicalOperator::Coalesce => Precedence::BitwiseOr,
                LogicalOperator::Or => Precedence::LogicalAnd,
            };
            self.right.gen_expr(p, self.precedence(), ctx);
        });
    }
}

impl<'a> GenExpr for ConditionalExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        let wrap = precedence > self.precedence();
        p.wrap(wrap, |p| {
            self.test.gen_expr(p, self.precedence(), ctx);
            p.print(b'?');
            self.consequent.gen_expr(p, Precedence::Assign, ctx.and_in(true));
            p.print(b':');
            self.alternate.gen_expr(p, Precedence::Assign, ctx.union_in_if(wrap));
        });
    }
}

impl<'a> GenExpr for AssignmentExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        // Destructuring assignment
        let n = p.code_len();
        let wrap = (p.start_of_stmt == n || p.start_of_arrow_expr == n)
            && matches!(
                self.left,
                AssignmentTarget::AssignmentTargetPattern(
                    AssignmentTargetPattern::ObjectAssignmentTarget(_)
                )
            );
        p.wrap(wrap || precedence > self.precedence(), |p| {
            self.left.gen(p, ctx);
            p.print_str(self.operator.as_str().as_bytes());
            self.right.gen_expr(p, Precedence::Assign, ctx);
        });
    }
}

impl<'a> Gen for AssignmentTarget<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::SimpleAssignmentTarget(target) => {
                target.gen_expr(p, Precedence::Assign, Context::default());
            }
            Self::AssignmentTargetPattern(pat) => pat.gen(p, ctx),
        }
    }
}

impl<'a> GenExpr for SimpleAssignmentTarget<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.gen(p, ctx),
            Self::MemberAssignmentTarget(member_expr) => {
                member_expr.gen_expr(p, precedence, ctx);
            }
        }
    }
}

impl<'a> Gen for AssignmentTargetPattern<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::ArrayAssignmentTarget(target) => target.gen(p, ctx),
            Self::ObjectAssignmentTarget(target) => target.gen(p, ctx),
        }
    }
}

impl<'a> Gen for ArrayAssignmentTarget<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print(b'[');
        p.print_list(&self.elements, ctx);
        if let Some(target) = &self.rest {
            p.print_comma();
            p.print_ellipsis();
            target.gen(p, ctx);
        }
        if self.trailing_comma.is_some() {
            p.print_comma();
        }
        p.print(b']');
    }
}

impl<'a> Gen for Option<AssignmentTargetMaybeDefault<'a>> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        if let Some(arg) = self {
            arg.gen(p, ctx);
        }
    }
}

impl<'a> Gen for ObjectAssignmentTarget<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print(b'{');
        p.print_list(&self.properties, ctx);
        if let Some(target) = &self.rest {
            if !self.properties.is_empty() {
                p.print_comma();
            }
            p.print_ellipsis();
            target.gen(p, ctx);
        }
        p.print(b'}');
    }
}

impl<'a> Gen for AssignmentTargetMaybeDefault<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::AssignmentTarget(target) => target.gen(p, ctx),
            Self::AssignmentTargetWithDefault(target) => target.gen(p, ctx),
        }
    }
}

impl<'a> Gen for AssignmentTargetWithDefault<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.binding.gen(p, ctx);
        p.print_equal();
        self.init.gen_expr(p, Precedence::Assign, Context::default());
    }
}

impl<'a> Gen for AssignmentTargetProperty<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::AssignmentTargetPropertyIdentifier(ident) => ident.gen(p, ctx),
            Self::AssignmentTargetPropertyProperty(prop) => prop.gen(p, ctx),
        }
    }
}

impl<'a> Gen for AssignmentTargetPropertyIdentifier<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.binding.gen(p, ctx);
        if let Some(expr) = &self.init {
            p.print_equal();
            expr.gen_expr(p, Precedence::Assign, Context::default());
        }
    }
}

impl<'a> Gen for AssignmentTargetPropertyProperty<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match &self.name {
            PropertyKey::Identifier(ident) => {
                ident.gen(p, ctx);
            }
            PropertyKey::PrivateIdentifier(ident) => {
                ident.gen(p, ctx);
            }
            PropertyKey::Expression(expr) => {
                p.print(b'[');
                expr.gen_expr(p, Precedence::Assign, Context::default());
                p.print(b']');
            }
        }
        p.print_colon();
        self.binding.gen(p, ctx);
    }
}

impl<'a> GenExpr for SequenceExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        p.wrap(precedence > self.precedence(), |p| {
            p.print_expressions(&self.expressions, Precedence::Assign, Context::default());
        });
    }
}

impl<'a> Gen for ImportExpression<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"import(");
        self.source.gen_expr(p, Precedence::Assign, Context::default());
        if !self.arguments.is_empty() {
            p.print_comma();
            p.print_expressions(&self.arguments, Precedence::Assign, Context::default());
        }
        p.print(b')');
    }
}

impl<'a> Gen for TemplateLiteral<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print(b'`');
        let mut expressions = self.expressions.iter();

        for quasi in &self.quasis {
            p.print_str(quasi.value.raw.as_bytes());

            if let Some(expr) = expressions.next() {
                p.print_str(b"${");
                expr.gen_expr(p, Precedence::lowest(), Context::default());
                p.print(b'}');
            }
        }

        p.print(b'`');
    }
}

impl<'a> Gen for TaggedTemplateExpression<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.tag.gen_expr(p, Precedence::Call, Context::default());
        self.quasi.gen(p, ctx);
    }
}

impl Gen for Super {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"super");
    }
}

impl<'a> GenExpr for AwaitExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        p.wrap(precedence > self.precedence(), |p| {
            p.print_str(b"await ");
            self.argument.gen_expr(p, self.precedence(), ctx);
        });
    }
}

impl<'a> GenExpr for ChainExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        match &self.expression {
            ChainElement::CallExpression(expr) => expr.gen_expr(p, precedence, ctx),
            ChainElement::MemberExpression(expr) => expr.gen_expr(p, precedence, ctx),
        }
    }
}

impl<'a> GenExpr for NewExpression<'a> {
    fn gen_expr(&self, p: &mut Printer, precedence: Precedence, ctx: Context) {
        p.wrap(precedence > self.precedence(), |p| {
            p.print_str(b"new ");
            self.callee.gen_expr(p, self.precedence(), ctx);
            p.wrap(true, |p| {
                p.print_list(&self.arguments, ctx);
            });
        });
    }
}

impl Gen for MetaProperty {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.meta.gen(p, ctx);
        p.print(b'.');
        self.property.gen(p, ctx);
    }
}

impl<'a> Gen for Class<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        let n = p.code_len();
        let wrap = self.is_expression() && (p.start_of_stmt == n || p.start_of_default_export == n);
        p.wrap(wrap, |p| {
            self.decorators.gen(p, ctx);
            p.print_str(b"class");
            if let Some(id) = &self.id {
                p.print(b' ');
                id.gen(p, ctx);
            }
            if let Some(super_class) = self.super_class.as_ref() {
                p.print_str(b" extends ");
                super_class.gen_expr(p, Precedence::Call, Context::default());
            }
            p.print(b'{');
            for item in &self.body.body {
                p.print_semicolon_if_needed();
                item.gen(p, ctx);
                if matches!(
                    item,
                    ClassElement::PropertyDefinition(_) | ClassElement::AccessorProperty(_)
                ) {
                    p.print_semicolon_after_statement();
                } else {
                }
            }
            p.needs_semicolon = false;
            p.print(b'}');
        });
    }
}

impl<'a> Gen for ClassElement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::StaticBlock(elem) => elem.gen(p, ctx),
            Self::MethodDefinition(elem) => elem.gen(p, ctx),
            Self::PropertyDefinition(elem) => elem.gen(p, ctx),
            Self::AccessorProperty(elem) => elem.gen(p, ctx),
        }
    }
}

impl Gen for JSXIdentifier {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(self.name.as_bytes());
    }
}

impl<'a> Gen for JSXMemberExpressionObject<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::Identifier(ident) => ident.gen(p, ctx),
            Self::MemberExpression(member_expr) => member_expr.gen(p, ctx),
        }
    }
}

impl<'a> Gen for JSXMemberExpression<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.object.gen(p, ctx);
        p.print(b'.');
        self.property.gen(p, ctx);
    }
}

impl<'a> Gen for JSXElementName<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::Identifier(identifier) => identifier.gen(p, ctx),
            Self::NamespacedName(namespaced_name) => namespaced_name.gen(p, ctx),
            Self::MemberExpression(member_expr) => member_expr.gen(p, ctx),
        }
    }
}

impl Gen for JSXNamespacedName {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.namespace.gen(p, ctx);
        p.print(b'.');
        self.property.gen(p, ctx);
    }
}

impl<'a> Gen for JSXAttributeName<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::Identifier(ident) => ident.gen(p, ctx),
            Self::NamespacedName(namespaced_name) => namespaced_name.gen(p, ctx),
        }
    }
}

impl<'a> Gen for JSXAttribute<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.name.gen(p, ctx);
        p.print(b'=');
        if let Some(value) = &self.value {
            value.gen(p, ctx);
        }
    }
}

impl Gen for JSXEmptyExpression {
    fn gen(&self, _: &mut Printer, ctx: Context) {}
}

impl<'a> Gen for JSXExpression<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::Expression(expr) => expr.gen_expr(p, Precedence::lowest(), Context::default()),
            Self::EmptyExpression(expr) => expr.gen(p, ctx),
        }
    }
}

impl<'a> Gen for JSXExpressionContainer<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print(b'{');
        self.expression.gen(p, ctx);
        p.print(b'}');
    }
}

impl<'a> Gen for JSXAttributeValue<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::Fragment(fragment) => fragment.gen(p, ctx),
            Self::Element(el) => el.gen(p, ctx),
            Self::StringLiteral(lit) => lit.gen(p, ctx),
            Self::ExpressionContainer(expr_container) => expr_container.gen(p, ctx),
        }
    }
}

impl<'a> Gen for JSXSpreadAttribute<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.argument.gen_expr(p, Precedence::lowest(), Context::default());
    }
}

impl<'a> Gen for JSXAttributeItem<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::Attribute(attr) => attr.gen(p, ctx),
            Self::SpreadAttribute(spread_attr) => spread_attr.gen(p, ctx),
        }
    }
}

impl<'a> Gen for JSXOpeningElement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"<");
        self.name.gen(p, ctx);
        for attr in &self.attributes {
            attr.gen(p, ctx);
        }
        if self.self_closing {
            p.print_str(b"/>");
        } else {
            p.print(b'>');
        }
    }
}

impl<'a> Gen for JSXClosingElement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"</");
        self.name.gen(p, ctx);
        p.print(b'>');
    }
}

impl<'a> Gen for JSXElement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.opening_element.gen(p, ctx);
        for child in &self.children {
            child.gen(p, ctx);
        }
        if let Some(closing_element) = &self.closing_element {
            closing_element.gen(p, ctx);
        }
    }
}

impl Gen for JSXOpeningFragment {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"<>");
    }
}

impl Gen for JSXClosingFragment {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"</>");
    }
}

impl Gen for JSXText {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(self.value.as_bytes());
    }
}

impl<'a> Gen for JSXSpreadChild<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"...");
        self.expression.gen_expr(p, Precedence::lowest(), Context::default());
    }
}

impl<'a> Gen for JSXChild<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match self {
            Self::Fragment(fragment) => fragment.gen(p, ctx),
            Self::Element(el) => el.gen(p, ctx),
            Self::Spread(spread) => {
                spread.expression.gen_expr(p, Precedence::lowest(), Context::default());
            }
            Self::ExpressionContainer(expr_container) => expr_container.gen(p, ctx),
            Self::Text(text) => text.gen(p, ctx),
        }
    }
}

impl<'a> Gen for JSXFragment<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.opening_fragment.gen(p, ctx);
        for child in &self.children {
            child.gen(p, ctx);
        }
        self.closing_fragment.gen(p, ctx);
    }
}

impl<'a> Gen for StaticBlock<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_str(b"static");
        p.print(b'{');
        for stmt in &self.body {
            p.print_semicolon_if_needed();
            stmt.gen(p, ctx);
        }
        p.needs_semicolon = false;
        p.print(b'}');
    }
}

impl<'a> Gen for MethodDefinition<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.decorators.gen(p, ctx);

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
        self.key.gen(p, ctx);
        if self.computed {
            p.print(b']');
        }
        p.print(b'(');
        self.value.params.gen(p, ctx);
        p.print(b')');
        if let Some(body) = &self.value.body {
            body.gen(p, ctx);
        }
    }
}

impl<'a> Gen for PropertyDefinition<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.decorators.gen(p, ctx);
        if self.r#static {
            p.print_str(b"static ");
        }
        if self.computed {
            p.print(b'[');
        }
        self.key.gen(p, ctx);
        if self.computed {
            p.print(b']');
        }
        if let Some(value) = &self.value {
            p.print_equal();
            value.gen_expr(p, Precedence::Assign, Context::default());
        }
    }
}

impl<'a> Gen for AccessorProperty<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        if self.r#static {
            p.print_str(b"static ");
        }
        p.print_str(b"accessor ");
        if self.computed {
            p.print(b'[');
        }
        self.key.gen(p, ctx);
        if self.computed {
            p.print(b']');
        }
        if let Some(value) = &self.value {
            p.print_equal();
            value.gen_expr(p, Precedence::Assign, Context::default());
        }
    }
}

impl Gen for PrivateIdentifier {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print(b'#');
        p.print_str(self.name.as_bytes());
    }
}

impl<'a> Gen for BindingPattern<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        match &self {
            BindingPattern::BindingIdentifier(ident) => ident.gen(p, ctx),
            BindingPattern::ObjectPattern(pattern) => pattern.gen(p, ctx),
            BindingPattern::RestElement(elem) => elem.gen(p, ctx),
            BindingPattern::ArrayPattern(pattern) => pattern.gen(p, ctx),
            BindingPattern::AssignmentPattern(pattern) => pattern.gen(p, ctx),
        }
    }
}

impl<'a> Gen for ObjectPattern<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print(b'{');
        p.print_list(&self.properties, ctx);
        if let Some(rest) = &self.rest {
            if !self.properties.is_empty() {
                p.print_comma();
            }
            rest.gen(p, ctx);
        }
        p.print(b'}');
    }
}

impl<'a> Gen for BindingProperty<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        if self.computed {
            p.print(b'[');
        }
        self.key.gen(p, ctx);
        if self.computed {
            p.print(b']');
        }
        p.print(b':');
        self.value.gen(p, ctx);
    }
}

impl<'a> Gen for RestElement<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print_ellipsis();
        self.argument.gen(p, ctx);
    }
}

impl<'a> Gen for ArrayPattern<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print(b'[');
        for (index, item) in self.elements.iter().enumerate() {
            if index != 0 {
                p.print_comma();
            }
            if let Some(item) = item {
                item.gen(p, ctx);
            }
            if index == self.elements.len() - 1 && (item.is_none() || self.rest.is_some()) {
                p.print_comma();
            }
        }
        if let Some(rest) = &self.rest {
            rest.gen(p, ctx);
        }
        p.print(b']');
    }
}

impl<'a> Gen for AssignmentPattern<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        self.left.gen(p, ctx);
        p.print_equal();
        self.right.gen_expr(p, Precedence::Assign, Context::default());
    }
}

impl<'a> Gen for Vec<'a, Decorator<'a>> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        for decorator in self {
            decorator.gen(p, ctx);
            p.print(b' ');
        }
    }
}

impl<'a> Gen for Decorator<'a> {
    fn gen(&self, p: &mut Printer, ctx: Context) {
        p.print(b'@');
        self.expression.gen_expr(p, Precedence::Assign, Context::default());
    }
}
