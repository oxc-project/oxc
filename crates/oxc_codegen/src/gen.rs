use oxc_allocator::{Box, Vec};
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_syntax::{
    identifier::{LS, PS},
    keyword::is_reserved_keyword_or_global_object,
    number::NumberBase,
    operator::{BinaryOperator, UnaryOperator},
    precedence::{GetPrecedence, Precedence},
};

use crate::{
    annotation_comment::{gen_comment, get_leading_annotate_comment},
    Codegen, Context, Operator,
};

pub trait Gen<const MINIFY: bool> {
    fn gen(&self, _p: &mut Codegen<{ MINIFY }>, _ctx: Context) {}
}

pub trait GenExpr<const MINIFY: bool> {
    fn gen_expr(&self, _p: &mut Codegen<{ MINIFY }>, _precedence: Precedence, _ctx: Context) {}
}

impl<'a, const MINIFY: bool, T> Gen<MINIFY> for Box<'a, T>
where
    T: Gen<MINIFY>,
{
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        (**self).gen(p, ctx);
    }
}

/// the [GenComment] trait only generate annotate comments like `/* @__PURE__ */` and `/* @__NO_SIDE_EFFECTS__ */`.
pub trait GenComment<const MINIFY: bool> {
    fn gen_comment(&self, _p: &mut Codegen<{ MINIFY }>, _ctx: Context) {}
}

impl<'a, const MINIFY: bool, T> GenExpr<MINIFY> for Box<'a, T>
where
    T: GenExpr<MINIFY>,
{
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        (**self).gen_expr(p, precedence, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for Program<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if let Some(hashbang) = &self.hashbang {
            hashbang.gen(p, ctx);
        }
        p.print_directives_and_statements(Some(&self.directives), &self.body, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for Hashbang<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.print_str(b"#!");
        p.print_str(self.value.as_bytes());
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for Directive<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        // A Use Strict Directive may not contain an EscapeSequence or LineContinuation.
        // So here should print original `directive` value, the `expression` value is escaped str.
        // See https://github.com/babel/babel/blob/main/packages/babel-generator/src/generators/base.ts#L64
        p.wrap_quote(self.directive.as_str(), |p, _| {
            p.print_str(self.directive.as_bytes());
        });
        p.print_semicolon_after_statement();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for Statement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::BlockStatement(stmt) => stmt.gen(p, ctx),
            Self::BreakStatement(stmt) => stmt.gen(p, ctx),
            Self::ContinueStatement(stmt) => stmt.gen(p, ctx),
            Self::DebuggerStatement(stmt) => stmt.gen(p, ctx),
            Self::DoWhileStatement(stmt) => stmt.gen(p, ctx),
            Self::EmptyStatement(stmt) => stmt.gen(p, ctx),
            Self::ExpressionStatement(stmt) => stmt.gen(p, ctx),
            Self::ForInStatement(stmt) => stmt.gen(p, ctx),
            Self::ForOfStatement(stmt) => stmt.gen(p, ctx),
            Self::ForStatement(stmt) => stmt.gen(p, ctx),
            Self::IfStatement(stmt) => stmt.gen(p, ctx),
            Self::LabeledStatement(stmt) => stmt.gen(p, ctx),
            Self::ReturnStatement(stmt) => stmt.gen(p, ctx),
            Self::SwitchStatement(stmt) => stmt.gen(p, ctx),
            Self::ThrowStatement(stmt) => stmt.gen(p, ctx),
            Self::TryStatement(stmt) => stmt.gen(p, ctx),
            Self::WhileStatement(stmt) => stmt.gen(p, ctx),
            Self::WithStatement(stmt) => stmt.gen(p, ctx),

            Self::ImportDeclaration(decl) => decl.gen(p, ctx),
            Self::ExportAllDeclaration(decl) => decl.gen(p, ctx),
            Self::ExportDefaultDeclaration(decl) => decl.gen(p, ctx),
            Self::ExportNamedDeclaration(decl) => decl.gen(p, ctx),
            Self::TSExportAssignment(decl) => decl.gen(p, ctx),
            Self::TSNamespaceExportDeclaration(decl) => decl.gen(p, ctx),

            Self::VariableDeclaration(decl) => {
                p.print_indent();
                decl.gen(p, ctx);
                p.print_semicolon_after_statement();
            }
            Self::FunctionDeclaration(decl) => {
                p.print_indent();
                decl.gen(p, ctx);
                p.print_soft_newline();
            }
            Self::ClassDeclaration(decl) => {
                p.print_indent();
                decl.gen(p, ctx);
                p.print_soft_newline();
            }
            Self::UsingDeclaration(declaration) => {
                p.print_indent();
                declaration.gen(p, ctx);
                p.print_semicolon_after_statement();
            }
            Self::TSModuleDeclaration(decl) => {
                p.print_indent();
                decl.gen(p, ctx);
                p.print_soft_newline();
            }
            Self::TSTypeAliasDeclaration(decl) => {
                p.print_indent();
                decl.gen(p, ctx);
                p.print_semicolon_after_statement();
            }
            Self::TSInterfaceDeclaration(decl) => {
                p.print_indent();
                decl.gen(p, ctx);
                p.print_soft_newline();
            }
            Self::TSEnumDeclaration(decl) => {
                p.print_indent();
                decl.gen(p, ctx);
                p.print_soft_newline();
            }
            Self::TSImportEqualsDeclaration(decl) => {
                p.print_indent();
                decl.gen(p, ctx);
                p.print_semicolon_after_statement();
            }
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ExpressionStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.start_of_stmt = p.code_len();
        p.print_expression(&self.expression);
        if self.expression.is_specific_id("let") {
            p.print_semicolon();
        } else {
            p.print_semicolon_after_statement();
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for IfStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        print_if(self, p, ctx);
    }
}

fn print_if<const MINIFY: bool>(
    if_stmt: &IfStatement<'_>,
    p: &mut Codegen<{ MINIFY }>,
    ctx: Context,
) {
    p.print_str(b"if");
    p.print_soft_space();
    p.print(b'(');
    p.print_expression(&if_stmt.test);
    p.print(b')');

    match &if_stmt.consequent {
        Statement::BlockStatement(block) => {
            p.print_soft_space();
            p.print_block_statement(block, ctx);
            if if_stmt.alternate.is_some() {
                p.print_soft_space();
            } else {
                p.print_soft_newline();
            }
        }
        stmt if wrap_to_avoid_ambiguous_else(stmt) => {
            p.print_soft_space();
            p.print_block_start(stmt.span().start);
            stmt.gen(p, ctx);
            p.needs_semicolon = false;
            p.print_block_end(stmt.span().end);
            if if_stmt.alternate.is_some() {
                p.print_soft_space();
            } else {
                p.print_soft_newline();
            }
        }
        stmt => p.print_body(stmt, false, ctx),
    }
    if let Some(alternate) = if_stmt.alternate.as_ref() {
        p.print_semicolon_if_needed();
        p.print_space_before_identifier();
        p.print_str(b"else");
        match alternate {
            Statement::BlockStatement(block) => {
                p.print_soft_space();
                p.print_block_statement(block, ctx);
                p.print_soft_newline();
            }
            Statement::IfStatement(if_stmt) => {
                p.print_hard_space();
                print_if(if_stmt, p, ctx);
            }
            stmt => p.print_body(stmt, true, ctx),
        }
    }
}

// <https://github.com/evanw/esbuild/blob/e6a8169c3a574f4c67d4cdd5f31a938b53eb7421/internal/js_printer/js_printer.go#L3444>
fn wrap_to_avoid_ambiguous_else(stmt: &Statement) -> bool {
    let mut current = stmt;
    loop {
        current = match current {
            Statement::IfStatement(if_stmt) => {
                if let Some(stmt) = &if_stmt.alternate {
                    stmt
                } else {
                    return true;
                }
            }
            Statement::ForStatement(for_stmt) => &for_stmt.body,
            Statement::ForOfStatement(for_of_stmt) => &for_of_stmt.body,
            Statement::ForInStatement(for_in_stmt) => &for_in_stmt.body,
            Statement::WhileStatement(while_stmt) => &while_stmt.body,
            Statement::WithStatement(with_stmt) => &with_stmt.body,
            Statement::LabeledStatement(labeled_stmt) => &labeled_stmt.body,
            _ => return false,
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for BlockStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_indent();
        p.print_block_statement(self, ctx);
        p.print_soft_newline();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ForStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str(b"for");
        p.print_soft_space();
        p.print(b'(');

        if let Some(init) = self.init.as_ref() {
            let ctx = Context::empty();
            match init {
                ForStatementInit::UsingDeclaration(decl) => decl.gen(p, ctx),
                match_expression!(ForStatementInit) => {
                    init.to_expression().gen_expr(p, Precedence::lowest(), ctx);
                }
                ForStatementInit::VariableDeclaration(var) => var.gen(p, ctx),
            }
        }

        p.print_semicolon();

        if let Some(test) = self.test.as_ref() {
            p.print_soft_space();
            p.print_expression(test);
        }

        p.print_semicolon();

        if let Some(update) = self.update.as_ref() {
            p.print_soft_space();
            p.print_expression(update);
        }

        p.print(b')');
        p.print_body(&self.body, false, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ForInStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str(b"for");
        p.print_soft_space();
        p.print(b'(');
        self.left.gen(p, ctx);
        p.print_soft_space();
        p.print_space_before_identifier();
        p.print_str(b"in");
        p.print_hard_space();
        p.print_expression(&self.right);
        p.print(b')');
        p.print_body(&self.body, false, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ForOfStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str(b"for");
        p.print_soft_space();
        if self.r#await {
            p.print_str(b" await");
        }
        p.print(b'(');
        self.left.gen(p, ctx);
        p.print_soft_space();
        p.print_space_before_identifier();
        p.print_str(b"of ");
        self.right.gen_expr(p, Precedence::Assign, Context::default());
        p.print(b')');
        p.print_body(&self.body, false, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ForStatementLeft<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            ForStatementLeft::UsingDeclaration(var) => var.gen(p, ctx),
            ForStatementLeft::VariableDeclaration(var) => var.gen(p, ctx),
            ForStatementLeft::AssignmentTargetIdentifier(identifier) => {
                let wrap = identifier.name == "async";
                p.wrap(wrap, |p| self.to_assignment_target().gen(p, ctx));
            }
            match_assignment_target!(ForStatementLeft) => {
                p.wrap(false, |p| self.to_assignment_target().gen(p, ctx));
            }
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for WhileStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str(b"while");
        p.print_soft_space();
        p.print(b'(');
        p.print_expression(&self.test);
        p.print(b')');
        p.print_body(&self.body, false, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for DoWhileStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str(b"do ");
        if let Statement::BlockStatement(block) = &self.body {
            p.print_block_statement(block, ctx);
            p.print_soft_space();
        } else {
            p.print_soft_newline();
            p.indent();
            self.body.gen(p, ctx);
            p.print_semicolon_if_needed();
            p.dedent();
            p.print_indent();
        }
        p.print_str(b"while");
        p.print_soft_space();
        p.print(b'(');
        p.print_expression(&self.test);
        p.print(b')');
        p.print_semicolon_after_statement();
    }
}

impl<const MINIFY: bool> Gen<MINIFY> for EmptyStatement {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_semicolon();
        p.print_soft_newline();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ContinueStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str(b"continue");
        if let Some(label) = &self.label {
            p.print_hard_space();
            label.gen(p, ctx);
        }
        p.print_semicolon_after_statement();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for BreakStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str(b"break");
        if let Some(label) = &self.label {
            p.print_hard_space();
            label.gen(p, ctx);
        }
        p.print_semicolon_after_statement();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for SwitchStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str(b"switch");
        p.print_soft_space();
        p.print(b'(');
        p.print_expression(&self.discriminant);
        p.print(b')');
        p.print_soft_space();
        p.print_curly_braces(self.span, self.cases.is_empty(), |p| {
            for case in &self.cases {
                p.add_source_mapping(case.span.start);
                case.gen(p, ctx);
            }
        });
        p.print_soft_newline();
        p.needs_semicolon = false;
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for SwitchCase<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_semicolon_if_needed();
        p.print_indent();
        match &self.test {
            Some(test) => {
                p.print_str(b"case ");
                p.print_expression(test);
            }
            None => p.print_str(b"default"),
        }
        p.print_colon();

        if self.consequent.len() == 1 {
            p.print_body(&self.consequent[0], false, ctx);
            return;
        }

        p.print_soft_newline();
        p.indent();
        for item in &self.consequent {
            p.print_semicolon_if_needed();
            item.gen(p, ctx);
        }
        p.dedent();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ReturnStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str(b"return");
        if let Some(arg) = &self.argument {
            p.print_hard_space();
            p.print_expression(arg);
        }
        p.print_semicolon_after_statement();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for LabeledStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if !MINIFY && (p.indent > 0 || p.print_next_indent_as_space) {
            p.add_source_mapping(self.span.start);
            p.print_indent();
        }
        p.print_space_before_identifier();
        self.label.gen(p, ctx);
        p.print_colon();
        p.print_body(&self.body, false, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TryStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_space_before_identifier();
        p.print_str(b"try");
        p.print_soft_space();
        p.print_block_statement(&self.block, ctx);
        if let Some(handler) = &self.handler {
            p.print_soft_space();
            p.print_str(b"catch");
            if let Some(param) = &handler.param {
                p.print_soft_space();
                p.print_str(b"(");
                param.pattern.gen(p, ctx);
                p.print_str(b")");
            }
            p.print_soft_space();
            p.print_block_statement(&handler.body, ctx);
            if self.finalizer.is_some() {
                p.print_soft_newline();
            }
        }
        if let Some(finalizer) = &self.finalizer {
            p.print_soft_space();
            p.print_str(b"finally");
            p.print_soft_space();
            p.print_block_statement(finalizer, ctx);
        }
        p.print_soft_newline();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ThrowStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str(b"throw ");
        p.print_expression(&self.argument);
        p.print_semicolon_after_statement();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for WithStatement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str(b"with");
        p.print(b'(');
        p.print_expression(&self.object);
        p.print(b')');
        p.print_body(&self.body, false, ctx);
    }
}

impl<const MINIFY: bool> Gen<MINIFY> for DebuggerStatement {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str(b"debugger");
        p.print_semicolon_after_statement();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for UsingDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if self.is_await {
            p.print_str(b"await");
            p.print_soft_space();
        }
        p.print_str(b"using");
        p.print_soft_space();
        p.print_list(&self.declarations, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for VariableDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        if self.modifiers.is_contains_declare() {
            p.print_str(b"declare ");
        }

        if p.comment_options.preserve_annotate_comments
            && matches!(self.kind, VariableDeclarationKind::Const)
        {
            if let Some(declarator) = self.declarations.first() {
                if let Some(ref init) = declarator.init {
                    if let Some(leading_annotate_comment) =
                        get_leading_annotate_comment(self.span.start, p)
                    {
                        p.move_comment(init.span().start, leading_annotate_comment);
                    }
                }
            }
        }
        p.print_str(match self.kind {
            VariableDeclarationKind::Const => "const",
            VariableDeclarationKind::Let => "let",
            VariableDeclarationKind::Var => "var",
        });
        if !self.declarations.is_empty() {
            p.print_hard_space();
        }
        p.print_list(&self.declarations, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for VariableDeclarator<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.id.gen(p, ctx);
        if let Some(init) = &self.init {
            p.print_soft_space();
            p.print_equal();
            p.print_soft_space();
            init.gen_expr(p, Precedence::Assign, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for Function<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        self.gen_comment(p, ctx);
        let n = p.code_len();
        let wrap = self.is_expression() && (p.start_of_stmt == n || p.start_of_default_export == n);
        p.wrap(wrap, |p| {
            if self.modifiers.contains(ModifierKind::Declare) {
                p.print_str(b"declare ");
            }
            if self.r#async {
                p.print_str(b"async ");
            }
            p.print_str(b"function");
            if self.generator {
                p.print(b'*');
                p.print_soft_space();
            }
            if let Some(id) = &self.id {
                p.print_space_before_identifier();
                id.gen(p, ctx);
            }
            if let Some(type_parameters) = &self.type_parameters {
                type_parameters.gen(p, ctx);
            }
            p.print(b'(');
            if let Some(this_param) = &self.this_param {
                this_param.gen(p, ctx);
                if !self.params.is_empty() || self.params.rest.is_some() {
                    p.print_str(b",");
                }
                p.print_soft_space();
            }
            self.params.gen(p, ctx);
            p.print(b')');
            if let Some(return_type) = &self.return_type {
                p.print_str(b": ");
                return_type.gen(p, ctx);
            }
            if let Some(body) = &self.body {
                p.print_soft_space();
                body.gen(p, ctx);
            } else {
                p.print_semicolon();
            }
        });
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for FunctionBody<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_curly_braces(self.span, self.is_empty(), |p| {
            p.print_directives_and_statements(Some(&self.directives), &self.statements, ctx);
        });
        p.needs_semicolon = false;
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for FormalParameter<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.decorators.gen(p, ctx);
        if self.readonly {
            p.print_str(b"readonly ");
        }
        if let Some(accessibility) = self.accessibility {
            accessibility.gen(p, ctx);
        }
        self.pattern.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for FormalParameters<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_list(&self.items, ctx);
        if let Some(rest) = &self.rest {
            if !self.items.is_empty() {
                p.print_comma();
                p.print_soft_space();
            }
            rest.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ImportDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str(b"import ");
        if self.import_kind.is_type() {
            p.print_str(b"type ");
        }
        if let Some(specifiers) = &self.specifiers {
            if specifiers.is_empty() {
                p.print_str(b"{}");
                p.print_soft_space();
                p.print_str(b"from");
                p.print_soft_space();
                p.print(b'\'');
                p.print_str(self.source.value.as_bytes());
                p.print(b'\'');
                if self.with_clause.is_some() {
                    p.print_hard_space();
                }
                self.with_clause.gen(p, ctx);
                p.print_semicolon_after_statement();
                return;
            }

            let mut in_block = false;
            for (index, specifier) in specifiers.iter().enumerate() {
                match specifier {
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                        if in_block {
                            p.print_soft_space();
                            p.print_str(b"},");
                            in_block = false;
                        } else if index != 0 {
                            p.print_comma();
                            p.print_soft_space();
                        }
                        spec.local.gen(p, ctx);
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                        if in_block {
                            p.print_soft_space();
                            p.print_str(b"},");
                            in_block = false;
                        } else if index != 0 {
                            p.print_comma();
                            p.print_soft_space();
                        }
                        p.print_str(b"* as ");
                        spec.local.gen(p, ctx);
                    }
                    ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                        if in_block {
                            p.print_comma();
                            p.print_soft_space();
                        } else {
                            if index != 0 {
                                p.print_comma();
                                p.print_soft_space();
                            }
                            in_block = true;
                            p.print(b'{');
                            p.print_soft_space();
                        }

                        if spec.import_kind.is_type() {
                            p.print_str(b"type ");
                        }

                        let imported_name = match &spec.imported {
                            ModuleExportName::IdentifierName(identifier) => {
                                identifier.gen(p, ctx);
                                identifier.name.as_bytes()
                            }
                            ModuleExportName::IdentifierReference(identifier) => {
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
                p.print_soft_space();
                p.print(b'}');
            }
            p.print_str(b" from ");
        }
        self.source.gen(p, ctx);
        if self.with_clause.is_some() {
            p.print_hard_space();
        }
        self.with_clause.gen(p, ctx);
        p.add_source_mapping(self.span.end);
        p.print_semicolon_after_statement();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for Option<WithClause<'a>> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if let Some(with_clause) = self {
            with_clause.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for WithClause<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        self.attributes_keyword.gen(p, ctx);
        p.print_soft_space();
        p.print_block_start(self.span.start);
        p.print_sequence(&self.with_entries, ctx);
        p.print_block_end(self.span.end);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ImportAttribute<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match &self.key {
            ImportAttributeKey::Identifier(identifier) => {
                p.print_str(identifier.name.as_bytes());
            }
            ImportAttributeKey::StringLiteral(literal) => literal.gen(p, ctx),
        };
        p.print_colon();
        p.print_soft_space();
        self.value.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ExportNamedDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        if p.comment_options.preserve_annotate_comments {
            match &self.declaration {
                Some(Declaration::FunctionDeclaration(_)) => {
                    gen_comment(self.span.start, p);
                }
                Some(Declaration::VariableDeclaration(var_decl))
                    if matches!(var_decl.kind, VariableDeclarationKind::Const) =>
                {
                    if let Some(declarator) = var_decl.declarations.first() {
                        if let Some(ref init) = declarator.init {
                            if let Some(leading_annotate_comment) =
                                get_leading_annotate_comment(self.span.start, p)
                            {
                                p.move_comment(init.span().start, leading_annotate_comment);
                            }
                        }
                    }
                }
                _ => {}
            };
        }
        p.print_str(b"export ");
        if self.export_kind.is_type() {
            p.print_str(b"type ");
        }
        match &self.declaration {
            Some(decl) => {
                match decl {
                    Declaration::VariableDeclaration(decl) => decl.gen(p, ctx),
                    Declaration::FunctionDeclaration(decl) => decl.gen(p, ctx),
                    Declaration::ClassDeclaration(decl) => decl.gen(p, ctx),
                    Declaration::UsingDeclaration(declaration) => declaration.gen(p, ctx),
                    Declaration::TSModuleDeclaration(decl) => decl.gen(p, ctx),
                    Declaration::TSTypeAliasDeclaration(decl) => decl.gen(p, ctx),
                    Declaration::TSInterfaceDeclaration(decl) => decl.gen(p, ctx),
                    Declaration::TSEnumDeclaration(decl) => decl.gen(p, ctx),
                    Declaration::TSImportEqualsDeclaration(decl) => decl.gen(p, ctx),
                }
                if matches!(
                    decl,
                    Declaration::VariableDeclaration(_)
                        | Declaration::UsingDeclaration(_)
                        | Declaration::TSTypeAliasDeclaration(_)
                        | Declaration::TSImportEqualsDeclaration(_)
                ) {
                    p.print_semicolon_after_statement();
                } else {
                    p.print_soft_newline();
                    p.needs_semicolon = false;
                }
            }
            None => {
                p.print(b'{');
                if !self.specifiers.is_empty() {
                    p.print_soft_space();
                    p.print_list(&self.specifiers, ctx);
                    p.print_soft_space();
                }
                p.print(b'}');
                if let Some(source) = &self.source {
                    p.print_soft_space();
                    p.print_str(b"from");
                    p.print_soft_space();
                    source.gen(p, ctx);
                }
                p.print_semicolon_after_statement();
            }
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSExportAssignment<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_indent();
        p.print_str(b"export = ");
        self.expression.gen_expr(p, Precedence::lowest(), ctx);
        p.print_semicolon_after_statement();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSNamespaceExportDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_indent();
        p.print_str(b"export as namespace ");
        self.id.gen(p, ctx);
        p.print_semicolon_after_statement();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ExportSpecifier<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if self.export_kind.is_type() {
            p.print_str(b"type ");
        }
        self.local.gen(p, ctx);
        if self.local.name() != self.exported.name() {
            p.print_str(b" as ");
            self.exported.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ModuleExportName<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::IdentifierName(identifier) => p.print_str(identifier.name.as_bytes()),
            Self::IdentifierReference(identifier) => p.print_str(identifier.name.as_bytes()),
            Self::StringLiteral(literal) => literal.gen(p, ctx),
        };
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ExportAllDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str(b"export ");
        if self.export_kind.is_type() {
            p.print_str(b"type ");
        }
        p.print(b'*');

        if let Some(exported) = &self.exported {
            p.print_str(b" as ");
            exported.gen(p, ctx);
        }

        p.print_str(b" from ");
        self.source.gen(p, ctx);
        if self.with_clause.is_some() {
            p.print_hard_space();
        }
        self.with_clause.gen(p, ctx);
        p.print_semicolon_after_statement();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ExportDefaultDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str(b"export default ");
        self.declaration.gen(p, ctx);
    }
}
impl<'a, const MINIFY: bool> Gen<MINIFY> for ExportDefaultDeclarationKind<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            match_expression!(Self) => {
                p.start_of_default_export = p.code_len();
                self.to_expression().gen_expr(p, Precedence::Assign, Context::default());
                p.print_semicolon_after_statement();
            }
            Self::FunctionDeclaration(fun) => fun.gen(p, ctx),
            Self::ClassDeclaration(class) => {
                class.gen(p, ctx);
                p.print_soft_newline();
            }
            Self::TSInterfaceDeclaration(interface) => interface.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for Expression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        match self {
            Self::BooleanLiteral(lit) => lit.gen(p, ctx),
            Self::NullLiteral(lit) => lit.gen(p, ctx),
            Self::NumericLiteral(lit) => lit.gen(p, ctx),
            Self::BigintLiteral(lit) => lit.gen(p, ctx),
            Self::RegExpLiteral(lit) => lit.gen(p, ctx),
            Self::StringLiteral(lit) => lit.gen(p, ctx),
            Self::Identifier(ident) => ident.gen(p, ctx),
            Self::ThisExpression(expr) => expr.gen(p, ctx),
            match_member_expression!(Self) => {
                self.to_member_expression().gen_expr(p, precedence, ctx);
            }
            Self::CallExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::ArrayExpression(expr) => expr.gen(p, ctx),
            Self::ObjectExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::FunctionExpression(expr) => expr.gen(p, ctx),
            Self::ArrowFunctionExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::YieldExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::UpdateExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::UnaryExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::BinaryExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::PrivateInExpression(expr) => expr.gen(p, ctx),
            Self::LogicalExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::ConditionalExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::AssignmentExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::SequenceExpression(expr) => expr.gen_expr(p, precedence, ctx),
            Self::ImportExpression(expr) => expr.gen_expr(p, precedence, ctx),
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
            Self::ParenthesizedExpression(e) => e.expression.gen_expr(p, precedence, ctx),
            Self::TSAsExpression(e) => e.gen_expr(p, precedence, ctx),
            Self::TSSatisfiesExpression(e) => {
                e.expression.gen_expr(p, precedence, ctx);
                p.print_str(b" satisfies ");
                e.type_annotation.gen(p, ctx);
            }
            Self::TSTypeAssertion(e) => e.gen_expr(p, precedence, ctx),
            Self::TSNonNullExpression(e) => e.expression.gen_expr(p, precedence, ctx),
            Self::TSInstantiationExpression(e) => e.expression.gen_expr(p, precedence, ctx),
        }
    }
}

impl<const MINIFY: bool> GenComment<MINIFY> for ArrowFunctionExpression<'_> {
    fn gen_comment(&self, codegen: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        gen_comment(self.span.start, codegen);
    }
}

impl<const MINIFY: bool> GenComment<MINIFY> for Function<'_> {
    fn gen_comment(&self, codegen: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        gen_comment(self.span.start, codegen);
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for TSAsExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        p.print_str(b"(");
        self.expression.gen_expr(p, precedence, ctx);
        p.print_str(b" as ");
        self.type_annotation.gen(p, ctx);
        p.print_str(b")");
    }
}
impl<'a, const MINIFY: bool> Gen<MINIFY> for IdentifierReference<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        // if let Some(mangler) = &p.mangler {
        // if let Some(reference_id) = self.reference_id.get() {
        // if let Some(name) = mangler.get_reference_name(reference_id) {
        // p.print_str(name.clone().as_bytes());
        // return;
        // }
        // }
        // }
        p.add_source_mapping_for_name(self.span, &self.name);
        p.print_str(self.name.as_bytes());
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for IdentifierName<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str(self.name.as_bytes());
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for BindingIdentifier<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.print_symbol(self.span, self.symbol_id.get(), self.name.as_str());
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for LabelIdentifier<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping_for_name(self.span, &self.name);
        p.print_str(self.name.as_bytes());
    }
}

impl<const MINIFY: bool> Gen<MINIFY> for BooleanLiteral {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str(self.as_str().as_bytes());
    }
}

impl<const MINIFY: bool> Gen<MINIFY> for NullLiteral {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.print_space_before_identifier();
        p.add_source_mapping(self.span.start);
        p.print_str(b"null");
    }
}

// Need a space before "." if it could be parsed as a decimal point.
fn need_space_before_dot<const MINIFY: bool>(bytes: &[u8], p: &mut Codegen<{ MINIFY }>) {
    if !bytes.iter().any(|&b| matches!(b, b'.' | b'e' | b'x')) {
        p.need_space_before_dot = p.code_len();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for NumericLiteral<'a> {
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        if self.value != f64::INFINITY && (MINIFY || self.raw.is_empty()) {
            p.print_space_before_identifier();
            let abs_value = self.value.abs();

            if self.value.is_sign_negative() {
                p.print_space_before_operator(Operator::Unary(UnaryOperator::UnaryNegation));
                p.print_str(b"-");
            }

            let result = if self.base == NumberBase::Float {
                print_non_negative_float(abs_value, p)
            } else {
                let value = abs_value as u64;
                // If integers less than 1000, we know that exponential notation will always be longer than
                // the integer representation. This is not the case for 1000 which is "1e3".
                if value < 1000 {
                    format!("{value}")
                } else if (1_000_000_000_000..=0xFFFF_FFFF_FFFF_F800).contains(&value) {
                    let hex = format!("{value:#x}");
                    let result = print_non_negative_float(abs_value, p);
                    if hex.len() < result.len() {
                        hex
                    } else {
                        result
                    }
                } else {
                    print_non_negative_float(abs_value, p)
                }
            };
            let bytes = result.as_bytes();
            p.print_str(bytes);
            need_space_before_dot(bytes, p);
        } else {
            let bytes = self.raw.as_bytes();
            p.print_str(bytes);
            need_space_before_dot(bytes, p);
        };
    }
}

// TODO: refactor this with less allocations
fn print_non_negative_float<const MINIFY: bool>(value: f64, _p: &Codegen<{ MINIFY }>) -> String {
    let mut result = value.to_string();
    let chars = result.as_bytes();
    let len = chars.len();
    let dot = chars.iter().position(|&c| c == b'.');
    let u8_to_string = |num: &[u8]| {
        // SAFETY: criteria of `from_utf8_unchecked`.are met.
        #[allow(unsafe_code)]
        unsafe {
            String::from_utf8_unchecked(num.to_vec())
        }
    };

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

impl<'a, const MINIFY: bool> Gen<MINIFY> for BigIntLiteral<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str(self.raw.as_bytes());
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for RegExpLiteral<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        let last = p.peek_nth(0);
        // Avoid forming a single-line comment or "</script" sequence
        if Some('/') == last
            || (Some('<') == last
                && self.regex.pattern.as_str().to_lowercase().starts_with("script"))
        {
            p.print_hard_space();
        }
        p.print(b'/');
        p.print_str(self.regex.pattern.as_bytes());
        p.print(b'/');
        p.print_str(self.regex.flags.to_string().as_bytes());
        p.prev_reg_exp_end = p.code().len();
    }
}

fn print_unquoted_str<const MINIFY: bool>(s: &str, quote: char, p: &mut Codegen<{ MINIFY }>) {
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\x00' => {
                if chars.peek().is_some_and(|&next| next.is_ascii_digit()) {
                    p.print_str(b"\\x00");
                } else {
                    p.print_str(b"\\0");
                }
            }
            '\x07' => {
                p.print_str(b"\\x07");
            }
            // \b
            '\u{8}' => {
                p.print_str(b"\\b");
            }
            // \v
            '\u{b}' => {
                p.print_str(b"\\v");
            }
            // \f
            '\u{c}' => {
                p.print_str(b"\\f");
            }
            '\n' => {
                p.print_str(b"\\n");
            }
            '\r' => {
                p.print_str(b"\\r");
            }
            '\x1B' => {
                p.print_str(b"\\x1B");
            }
            '\\' => {
                p.print_str(b"\\\\");
            }
            '\'' => {
                if quote == '\'' {
                    p.print_str(b"\\'");
                } else {
                    p.print_str(b"'");
                }
            }
            '\"' => {
                if quote == '"' {
                    p.print_str(b"\\\"");
                } else {
                    p.print_str(b"\"");
                }
            }
            '`' => {
                if quote == '`' {
                    p.print_str(b"\\`");
                } else {
                    p.print_str(b"`");
                }
            }
            '$' => {
                if chars.peek().is_some_and(|&next| next == '{') {
                    p.print_str(b"\\$");
                } else {
                    p.print_str(b"$");
                }
            }
            // Allow `U+2028` and `U+2029` in string literals
            // <https://tc39.es/proposal-json-superset>
            // <https://github.com/tc39/proposal-json-superset>
            LS => p.print_str(b"\\u2028"),
            PS => p.print_str(b"\\u2029"),
            '\u{a0}' => {
                p.print_str(b"\\xA0");
            }
            _ => {
                p.print_str(c.encode_utf8([0; 4].as_mut()).as_bytes());
            }
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for StringLiteral<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        let s = self.value.as_str();
        p.wrap_quote(s, |p, quote| {
            print_unquoted_str(s, quote, p);
        });
    }
}

impl<const MINIFY: bool> Gen<MINIFY> for ThisExpression {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_space_before_identifier();
        p.print_str(b"this");
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for MemberExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        p.wrap(precedence > self.precedence(), |p| match self {
            Self::ComputedMemberExpression(expr) => {
                expr.gen_expr(p, self.precedence(), ctx.and_in(true));
            }
            Self::StaticMemberExpression(expr) => expr.gen_expr(p, self.precedence(), ctx),
            Self::PrivateFieldExpression(expr) => expr.gen_expr(p, self.precedence(), ctx),
        });
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for ComputedMemberExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, _precedence: Precedence, ctx: Context) {
        self.object.gen_expr(p, Precedence::Postfix, ctx);
        if self.optional {
            p.print_str(b"?.");
        }
        p.print(b'[');
        self.expression.gen_expr(p, Precedence::lowest(), ctx);
        p.print(b']');
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for StaticMemberExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, _precedence: Precedence, ctx: Context) {
        self.object.gen_expr(p, Precedence::Postfix, ctx);
        if self.optional {
            p.print(b'?');
        } else if p.need_space_before_dot == p.code_len() {
            // `0.toExponential()` is invalid, add a space before the dot, `0 .toExponential()` is valid
            p.print_hard_space();
        }
        p.print(b'.');
        self.property.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for PrivateFieldExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, _precedence: Precedence, ctx: Context) {
        self.object.gen_expr(p, Precedence::Postfix, ctx);
        if self.optional {
            p.print_str(b"?");
        }
        p.print(b'.');
        self.field.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for CallExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        let wrap = precedence > self.precedence() || ctx.has_forbid_call();
        let ctx = ctx.and_forbid_call(false);
        p.wrap(wrap, |p| {
            p.add_source_mapping(self.span.start);
            self.callee.gen_expr(p, self.precedence(), ctx);
            if self.optional {
                p.print_str(b"?.");
            }
            if let Some(type_parameters) = &self.type_parameters {
                type_parameters.gen(p, ctx);
            }
            p.print(b'(');
            p.print_list(&self.arguments, ctx);
            p.print(b')');
            p.add_source_mapping(self.span.end);
        });
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for Argument<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::SpreadElement(elem) => elem.gen(p, ctx),
            match_expression!(Self) => {
                self.to_expression().gen_expr(p, Precedence::Assign, Context::default());
            }
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ArrayExpressionElement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            match_expression!(Self) => {
                self.to_expression().gen_expr(p, Precedence::Assign, Context::default());
            }
            Self::SpreadElement(elem) => elem.gen(p, ctx),
            Self::Elision(_span) => {}
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for SpreadElement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_ellipsis();
        self.argument.gen_expr(p, Precedence::Assign, Context::default());
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ArrayExpression<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print(b'[');
        p.print_list(&self.elements, ctx);
        if self.trailing_comma.is_some() {
            p.print_comma();
        }
        p.print(b']');
        p.add_source_mapping(self.span.end);
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for ObjectExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, _precedence: Precedence, ctx: Context) {
        let n = p.code_len();
        p.wrap(p.start_of_stmt == n || p.start_of_arrow_expr == n, |p| {
            let single_line = self.properties.len() <= 1;
            p.print_curly_braces(self.span, single_line, |p| {
                for (index, item) in self.properties.iter().enumerate() {
                    if index != 0 {
                        p.print_comma();
                        p.print_soft_newline();
                    }
                    if !single_line {
                        p.print_indent();
                    }
                    item.gen(p, ctx);
                }
                if !single_line {
                    p.print_soft_newline();
                }
            });
        });
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ObjectPropertyKind<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::ObjectProperty(prop) => prop.gen(p, ctx),
            Self::SpreadProperty(elem) => elem.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ObjectProperty<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if let Expression::FunctionExpression(func) = &self.value {
            p.add_source_mapping(self.span.start);
            let is_accessor = match &self.kind {
                PropertyKind::Init => false,
                PropertyKind::Get => {
                    p.add_source_mapping(self.span.start);
                    p.print_str(b"get ");
                    true
                }
                PropertyKind::Set => {
                    p.add_source_mapping(self.span.start);
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
                if let Some(type_parameters) = &func.type_parameters {
                    type_parameters.gen(p, ctx);
                }
                p.print(b'(');
                func.params.gen(p, ctx);
                p.print(b')');
                if let Some(body) = &func.body {
                    p.print_soft_space();
                    body.gen(p, ctx);
                }
                return;
            }
        }
        if self.computed {
            p.print(b'[');
        }
        if !self.shorthand {
            self.key.gen(p, ctx);
        }
        if self.computed {
            p.print(b']');
        }
        if !self.shorthand {
            p.print_colon();
            p.print_soft_space();
        }
        self.value.gen_expr(p, Precedence::Assign, Context::default());
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for PropertyKey<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::StaticIdentifier(ident) => ident.gen(p, ctx),
            Self::PrivateIdentifier(ident) => ident.gen(p, ctx),
            match_expression!(Self) => {
                self.to_expression().gen_expr(p, Precedence::Assign, Context::default());
            }
        }
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for ArrowFunctionExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        p.wrap(precedence > Precedence::Assign, |p| {
            self.gen_comment(p, ctx);
            if self.r#async {
                p.add_source_mapping(self.span.start);
                p.print_str(b"async");
            }

            if self.r#async {
                p.print_hard_space();
            }

            if let Some(type_parameters) = &self.type_parameters {
                type_parameters.gen(p, ctx);
            }
            p.add_source_mapping(self.span.start);
            p.print(b'(');
            self.params.gen(p, ctx);
            p.print(b')');
            if let Some(return_type) = &self.return_type {
                p.print_str(b":");
                p.print_soft_space();
                return_type.gen(p, ctx);
            }
            p.print_soft_space();
            p.print_str(b"=>");
            p.print_soft_space();
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

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for YieldExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        p.wrap(precedence >= self.precedence(), |p| {
            p.add_source_mapping(self.span.start);
            p.print_space_before_identifier();
            p.print_str(b"yield");
            if self.delegate {
                p.print(b'*');
                p.print_soft_space();
            }
            if let Some(argument) = self.argument.as_ref() {
                if !self.delegate {
                    p.print_hard_space();
                }
                argument.gen_expr(p, Precedence::Assign, ctx);
            }
        });
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for UpdateExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        let operator = self.operator.as_str().as_bytes();
        p.wrap(precedence > self.precedence(), |p| {
            if self.prefix {
                p.add_source_mapping(self.span.start);
                p.print_space_before_operator(self.operator.into());
                p.print_str(operator);
                p.prev_op = Some(self.operator.into());
                p.prev_op_end = p.code().len();
                self.argument.gen_expr(p, Precedence::Prefix, ctx);
            } else {
                p.print_space_before_operator(self.operator.into());
                self.argument.gen_expr(p, Precedence::Postfix, ctx);
                p.print_str(operator);
                p.prev_op = Some(self.operator.into());
                p.prev_op_end = p.code().len();
            }
        });
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for UnaryExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        p.wrap(precedence > self.precedence() || precedence == Precedence::Exponential, |p| {
            let operator = self.operator.as_str().as_bytes();
            if self.operator.is_keyword() {
                p.print_space_before_identifier();
                p.print_str(operator);
                p.print_hard_space();
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

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for BinaryExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        let wrap_in = self.operator == BinaryOperator::In && !ctx.has_in();
        let wrap = precedence >= self.precedence() || wrap_in;
        p.wrap(wrap, |p| {
            let left_precedence = if self.precedence().is_right_associative() {
                self.precedence()
            } else {
                self.operator.lower_precedence()
            };
            self.left.gen_expr(p, left_precedence, ctx);
            if self.operator.is_keyword() {
                p.print_space_before_identifier();
            } else {
                p.print_soft_space();
            }
            self.operator.gen(p, ctx);
            let right_precedence = if self.precedence().is_left_associative() {
                self.precedence()
            } else {
                self.operator.lower_precedence()
            };
            self.right.gen_expr(p, right_precedence, ctx.union_in_if(wrap));
        });
    }
}

impl<const MINIFY: bool> Gen<MINIFY> for BinaryOperator {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        let operator = self.as_str().as_bytes();
        if self.is_keyword() {
            p.print_str(operator);
            p.print_hard_space();
        } else {
            let op: Operator = (*self).into();
            p.print_space_before_operator(op);
            p.print_str(operator);
            p.print_soft_space();
            p.prev_op = Some(op);
            p.prev_op_end = p.code().len();
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for PrivateInExpression<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.left.gen(p, ctx);
        p.print_str(b" in ");
        self.right.gen_expr(p, Precedence::Shift, Context::default());
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for LogicalExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        // Logical expressions and coalesce expressions cannot be mixed (Syntax Error).
        let mixed = matches!(
            (precedence, self.precedence()),
            (Precedence::Coalesce, Precedence::LogicalAnd | Precedence::LogicalOr)
        );
        p.wrap(mixed || (precedence > self.precedence()), |p| {
            self.left.gen_expr(p, self.precedence(), ctx);
            p.print_soft_space();
            p.print_str(self.operator.as_str().as_bytes());
            p.print_soft_space();
            self.right.gen_expr(p, self.precedence(), ctx);
        });
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for ConditionalExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        let wrap = precedence > self.precedence();
        p.wrap(wrap, |p| {
            self.test.gen_expr(p, self.precedence(), ctx);
            p.print_soft_space();
            p.print(b'?');
            p.print_soft_space();
            self.consequent.gen_expr(p, Precedence::Assign, ctx.and_in(true));
            p.print_soft_space();
            p.print_colon();
            p.print_soft_space();
            self.alternate.gen_expr(p, Precedence::Assign, ctx.union_in_if(wrap));
        });
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for AssignmentExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        // Destructuring assignment
        let n = p.code_len();

        let identifier_is_keyword = match &self.left {
            AssignmentTarget::AssignmentTargetIdentifier(target) => {
                is_reserved_keyword_or_global_object(target.name.as_str())
            }
            AssignmentTarget::ComputedMemberExpression(expression) => match &expression.object {
                Expression::Identifier(ident) => {
                    is_reserved_keyword_or_global_object(ident.name.as_str())
                }
                _ => false,
            },
            AssignmentTarget::StaticMemberExpression(expression) => {
                is_reserved_keyword_or_global_object(expression.property.name.as_str())
            }
            AssignmentTarget::PrivateFieldExpression(expression) => {
                is_reserved_keyword_or_global_object(expression.field.name.as_str())
            }
            _ => false,
        };

        let wrap = ((p.start_of_stmt == n || p.start_of_arrow_expr == n)
            && matches!(self.left, AssignmentTarget::ObjectAssignmentTarget(_)))
            || identifier_is_keyword;
        p.wrap(wrap || precedence > self.precedence(), |p| {
            self.left.gen(p, ctx);
            p.print_soft_space();
            p.print_str(self.operator.as_str().as_bytes());
            p.print_soft_space();
            self.right.gen_expr(p, Precedence::Assign, ctx);
        });
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for AssignmentTarget<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            match_simple_assignment_target!(Self) => {
                self.to_simple_assignment_target().gen_expr(
                    p,
                    Precedence::Assign,
                    Context::default(),
                );
            }
            match_assignment_target_pattern!(Self) => {
                self.to_assignment_target_pattern().gen(p, ctx);
            }
        }
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for SimpleAssignmentTarget<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.gen(p, ctx),
            match_member_expression!(Self) => {
                self.to_member_expression().gen_expr(p, precedence, ctx);
            }
            Self::TSAsExpression(e) => e.gen_expr(p, precedence, ctx),
            Self::TSSatisfiesExpression(e) => e.expression.gen_expr(p, precedence, ctx),
            Self::TSNonNullExpression(e) => e.expression.gen_expr(p, precedence, ctx),
            Self::TSTypeAssertion(e) => e.gen_expr(p, precedence, ctx),
            Self::TSInstantiationExpression(e) => e.expression.gen_expr(p, precedence, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for AssignmentTargetPattern<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::ArrayAssignmentTarget(target) => target.gen(p, ctx),
            Self::ObjectAssignmentTarget(target) => target.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ArrayAssignmentTarget<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print(b'[');
        p.print_list(&self.elements, ctx);
        if let Some(target) = &self.rest {
            if !self.elements.is_empty() {
                p.print_comma();
            }
            p.add_source_mapping(self.span.start);
            target.gen(p, ctx);
        }
        if self.trailing_comma.is_some() {
            p.print_comma();
        }
        p.print(b']');
        p.add_source_mapping(self.span.end);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for Option<AssignmentTargetMaybeDefault<'a>> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if let Some(arg) = self {
            arg.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ObjectAssignmentTarget<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print(b'{');
        p.print_list(&self.properties, ctx);
        if let Some(target) = &self.rest {
            if !self.properties.is_empty() {
                p.print_comma();
            }
            p.add_source_mapping(self.span.start);
            target.gen(p, ctx);
        }
        p.print(b'}');
        p.add_source_mapping(self.span.end);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for AssignmentTargetMaybeDefault<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            match_assignment_target!(Self) => self.to_assignment_target().gen(p, ctx),
            Self::AssignmentTargetWithDefault(target) => target.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for AssignmentTargetWithDefault<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.binding.gen(p, ctx);
        p.print_soft_space();
        p.print_equal();
        p.print_soft_space();
        self.init.gen_expr(p, Precedence::Assign, Context::default());
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for AssignmentTargetProperty<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::AssignmentTargetPropertyIdentifier(ident) => ident.gen(p, ctx),
            Self::AssignmentTargetPropertyProperty(prop) => prop.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for AssignmentTargetPropertyIdentifier<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.binding.gen(p, ctx);
        if let Some(expr) = &self.init {
            p.print_soft_space();
            p.print_equal();
            p.print_soft_space();
            expr.gen_expr(p, Precedence::Assign, Context::default());
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for AssignmentTargetPropertyProperty<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match &self.name {
            PropertyKey::StaticIdentifier(ident) => {
                ident.gen(p, ctx);
            }
            PropertyKey::PrivateIdentifier(ident) => {
                ident.gen(p, ctx);
            }
            key @ match_expression!(PropertyKey) => {
                p.print(b'[');
                key.to_expression().gen_expr(p, Precedence::Assign, Context::default());
                p.print(b']');
            }
        }
        p.print_colon();
        p.print_soft_space();
        self.binding.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for AssignmentTargetRest<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_ellipsis();
        self.target.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for SequenceExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, _ctx: Context) {
        p.wrap(precedence > self.precedence(), |p| {
            p.print_expressions(&self.expressions, Precedence::Assign, Context::default());
        });
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for ImportExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        let wrap = precedence > self.precedence() || ctx.has_forbid_call();
        let ctx = ctx.and_forbid_call(false);
        p.wrap(wrap, |p| {
            p.add_source_mapping(self.span.start);
            p.print_str(b"import(");
            self.source.gen_expr(p, Precedence::Assign, ctx);
            if !self.arguments.is_empty() {
                p.print_comma();
                p.print_expressions(&self.arguments, Precedence::Assign, ctx);
            }
            p.print(b')');
        });
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TemplateLiteral<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.print(b'`');
        let mut expressions = self.expressions.iter();

        for quasi in &self.quasis {
            p.add_source_mapping(quasi.span.start);
            p.print_str(quasi.value.raw.as_bytes());

            if let Some(expr) = expressions.next() {
                p.print_str(b"${");
                p.print_expression(expr);
                p.print(b'}');
            }
        }

        p.print(b'`');
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TaggedTemplateExpression<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        self.tag.gen_expr(p, Precedence::Postfix, Context::default());
        self.quasi.gen(p, ctx);
    }
}

impl<const MINIFY: bool> Gen<MINIFY> for Super {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str(b"super");
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for AwaitExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        p.wrap(precedence > self.precedence(), |p| {
            p.add_source_mapping(self.span.start);
            p.print_str(b"await ");
            self.argument.gen_expr(p, self.precedence(), ctx);
        });
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for ChainExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        match &self.expression {
            ChainElement::CallExpression(expr) => expr.gen_expr(p, precedence, ctx),
            match_member_expression!(ChainElement) => {
                self.expression.to_member_expression().gen_expr(p, precedence, ctx);
            }
        }
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for NewExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        p.wrap(precedence > self.precedence(), |p| {
            p.add_source_mapping(self.span.start);
            p.print_str(b"new ");
            self.callee.gen_expr(p, Precedence::NewWithoutArgs, ctx.and_forbid_call(true));
            p.wrap(true, |p| {
                p.print_list(&self.arguments, ctx);
            });
        });
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for MetaProperty<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        self.meta.gen(p, ctx);
        p.print(b'.');
        self.property.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for Class<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        if self.modifiers.is_contains_declare() {
            p.print_str(b"declare ");
        }
        if self.modifiers.is_contains_abstract() {
            p.print_str(b"abstract ");
        }
        let n = p.code_len();
        let wrap = self.is_expression() && (p.start_of_stmt == n || p.start_of_default_export == n);
        p.wrap(wrap, |p| {
            self.decorators.gen(p, ctx);
            p.print_str(b"class");
            if let Some(id) = &self.id {
                p.print_hard_space();
                id.gen(p, ctx);
            }
            if let Some(super_class) = self.super_class.as_ref() {
                p.print_str(b" extends ");
                super_class.gen_expr(p, Precedence::Call, Context::default());
                if let Some(super_type_parameters) = &self.super_type_parameters {
                    super_type_parameters.gen(p, ctx);
                }
            }
            if let Some(implements) = self.implements.as_ref() {
                p.print_str(b" implements ");
                p.print_list(implements, ctx);
            }
            p.print_soft_space();
            self.body.gen(p, ctx);
            p.needs_semicolon = false;
        });
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ClassBody<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_curly_braces(self.span, self.body.is_empty(), |p| {
            for item in &self.body {
                p.print_semicolon_if_needed();
                p.print_indent();
                item.gen(p, ctx);
            }
        });
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ClassElement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::StaticBlock(elem) => {
                elem.gen(p, ctx);
                p.print_soft_newline();
            }
            Self::MethodDefinition(elem) => {
                elem.gen(p, ctx);
                p.print_soft_newline();
            }
            Self::PropertyDefinition(elem) => {
                elem.gen(p, ctx);
                p.print_semicolon_after_statement();
            }
            Self::AccessorProperty(elem) => {
                elem.gen(p, ctx);
                p.print_semicolon_after_statement();
            }
            Self::TSIndexSignature(elem) => {
                elem.gen(p, ctx);
                p.print_semicolon_after_statement();
            }
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXIdentifier<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping_for_name(self.span, &self.name);
        p.print_str(self.name.as_bytes());
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXMemberExpressionObject<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::Identifier(ident) => ident.gen(p, ctx),
            Self::MemberExpression(member_expr) => member_expr.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXMemberExpression<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.object.gen(p, ctx);
        p.print(b'.');
        self.property.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXElementName<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::Identifier(identifier) => identifier.gen(p, ctx),
            Self::NamespacedName(namespaced_name) => namespaced_name.gen(p, ctx),
            Self::MemberExpression(member_expr) => member_expr.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXNamespacedName<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.namespace.gen(p, ctx);
        p.print_colon();
        self.property.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXAttributeName<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::Identifier(ident) => ident.gen(p, ctx),
            Self::NamespacedName(namespaced_name) => namespaced_name.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXAttribute<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.name.gen(p, ctx);
        if let Some(value) = &self.value {
            p.print_equal();
            value.gen(p, ctx);
        }
    }
}

impl<const MINIFY: bool> Gen<MINIFY> for JSXEmptyExpression {
    fn gen(&self, _: &mut Codegen<{ MINIFY }>, _ctx: Context) {}
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXExpression<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            match_expression!(Self) => p.print_expression(self.to_expression()),
            Self::EmptyExpression(expr) => expr.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXExpressionContainer<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print(b'{');
        self.expression.gen(p, ctx);
        p.print(b'}');
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXAttributeValue<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::Fragment(fragment) => fragment.gen(p, ctx),
            Self::Element(el) => el.gen(p, ctx),
            Self::StringLiteral(lit) => {
                p.print(b'"');
                print_unquoted_str(&lit.value, '"', p);
                p.print(b'"');
            }
            Self::ExpressionContainer(expr_container) => expr_container.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXSpreadAttribute<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.print_str(b"{...");
        self.argument.gen_expr(p, Precedence::Assign, Context::default());
        p.print(b'}');
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXAttributeItem<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::Attribute(attr) => attr.gen(p, ctx),
            Self::SpreadAttribute(spread_attr) => spread_attr.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXOpeningElement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str(b"<");
        self.name.gen(p, ctx);
        for attr in &self.attributes {
            p.print_hard_space();
            attr.gen(p, ctx);
        }
        if self.self_closing {
            p.print_str(b"/>");
        } else {
            p.print(b'>');
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXClosingElement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str(b"</");
        self.name.gen(p, ctx);
        p.print(b'>');
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXElement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.opening_element.gen(p, ctx);
        for child in &self.children {
            child.gen(p, ctx);
        }
        if let Some(closing_element) = &self.closing_element {
            closing_element.gen(p, ctx);
        }
    }
}

impl<const MINIFY: bool> Gen<MINIFY> for JSXOpeningFragment {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str(b"<>");
    }
}

impl<const MINIFY: bool> Gen<MINIFY> for JSXClosingFragment {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str(b"</>");
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXText<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str(self.value.as_bytes());
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXSpreadChild<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.print_str(b"...");
        p.print_expression(&self.expression);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXChild<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::Fragment(fragment) => fragment.gen(p, ctx),
            Self::Element(el) => el.gen(p, ctx),
            Self::Spread(spread) => p.print_expression(&spread.expression),
            Self::ExpressionContainer(expr_container) => expr_container.gen(p, ctx),
            Self::Text(text) => text.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSXFragment<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.opening_fragment.gen(p, ctx);
        for child in &self.children {
            child.gen(p, ctx);
        }
        self.closing_fragment.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for StaticBlock<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str(b"static");
        p.print_soft_space();
        p.print_curly_braces(self.span, self.body.is_empty(), |p| {
            for stmt in &self.body {
                p.print_semicolon_if_needed();
                stmt.gen(p, ctx);
            }
        });
        p.needs_semicolon = false;
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for MethodDefinition<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        self.decorators.gen(p, ctx);

        if self.r#type == MethodDefinitionType::TSAbstractMethodDefinition {
            p.print_str(b"abstract ");
        }
        if let Some(accessibility) = &self.accessibility {
            accessibility.gen(p, ctx);
        }
        if self.r#static {
            p.print_str(b"static ");
        }

        match &self.kind {
            MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => {}
            MethodDefinitionKind::Get => {
                p.print_str(b"get ");
            }
            MethodDefinitionKind::Set => {
                p.print_str(b"set ");
            }
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
        if let Some(return_type) = &self.value.return_type {
            p.print_colon();
            p.print_soft_space();
            return_type.gen(p, ctx);
        }
        if let Some(body) = &self.value.body {
            p.print_soft_space();
            body.gen(p, ctx);
        } else {
            p.print_semicolon();
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for PropertyDefinition<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        self.decorators.gen(p, ctx);
        if self.r#type == PropertyDefinitionType::TSAbstractPropertyDefinition {
            p.print_str(b"abstract ");
        }
        if let Some(accessibility) = &self.accessibility {
            accessibility.gen(p, ctx);
        }

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
        if self.optional {
            p.print_str(b"?");
        }
        if let Some(type_annotation) = &self.type_annotation {
            p.print_colon();
            p.print_soft_space();
            type_annotation.gen(p, ctx);
        }
        if let Some(value) = &self.value {
            p.print_soft_space();
            p.print_equal();
            p.print_soft_space();
            value.gen_expr(p, Precedence::Assign, Context::default());
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for AccessorProperty<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        if self.r#type.is_abstract() {
            p.print_str(b"abstract ");
        }
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
        p.print_semicolon();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for PrivateIdentifier<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        p.add_source_mapping_for_name(self.span, &self.name);
        p.print(b'#');
        p.print_str(self.name.as_bytes());
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for BindingPattern<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match &self.kind {
            BindingPatternKind::BindingIdentifier(ident) => ident.gen(p, ctx),
            BindingPatternKind::ObjectPattern(pattern) => pattern.gen(p, ctx),
            BindingPatternKind::ArrayPattern(pattern) => pattern.gen(p, ctx),
            BindingPatternKind::AssignmentPattern(pattern) => pattern.gen(p, ctx),
        }
        if self.optional {
            p.print_str(b"?");
        }
        if let Some(type_annotation) = &self.type_annotation {
            p.print_colon();
            p.print_soft_space();
            type_annotation.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ObjectPattern<'a> {
    fn gen(&self, p: &mut Codegen<MINIFY>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print(b'{');
        p.print_soft_space();
        p.print_list(&self.properties, ctx);
        if let Some(rest) = &self.rest {
            if !self.properties.is_empty() {
                p.print_comma();
            }
            rest.gen(p, ctx);
        }
        p.print_soft_space();
        p.print(b'}');
        p.add_source_mapping(self.span.end);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for BindingProperty<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        if self.computed {
            p.print(b'[');
        }
        if !self.shorthand {
            self.key.gen(p, ctx);
        }
        if self.computed {
            p.print(b']');
        }
        if !self.shorthand {
            p.print_colon();
            p.print_soft_space();
        }
        self.value.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for BindingRestElement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_ellipsis();
        self.argument.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for ArrayPattern<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print(b'[');
        for (index, item) in self.elements.iter().enumerate() {
            if index != 0 {
                p.print_comma();
                if item.is_some() {
                    p.print_soft_space();
                }
            }
            if let Some(item) = item {
                item.gen(p, ctx);
            }
            if index == self.elements.len() - 1 && (item.is_none() || self.rest.is_some()) {
                p.print_comma();
            }
        }
        if let Some(rest) = &self.rest {
            p.print_soft_space();
            rest.gen(p, ctx);
        }
        p.print(b']');
        p.add_source_mapping(self.span.end);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for AssignmentPattern<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.left.gen(p, ctx);
        p.print_soft_space();
        p.print_equal();
        p.print_soft_space();
        self.right.gen_expr(p, Precedence::Assign, Context::default());
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for Vec<'a, Decorator<'a>> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        for decorator in self {
            decorator.gen(p, ctx);
            p.print_hard_space();
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for Decorator<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        fn need_wrap(expr: &Expression) -> bool {
            match expr {
                // "@foo"
                // "@foo.bar"
                // "@foo.#bar"
                Expression::Identifier(_)
                | Expression::StaticMemberExpression(_)
                | Expression::PrivateFieldExpression(_) => false,
                Expression::CallExpression(call_expr) => need_wrap(&call_expr.callee),
                // "@(foo + bar)"
                // "@(() => {})"
                // "@(foo['bar'])"
                _ => true,
            }
        }

        p.add_source_mapping(self.span.start);
        p.print(b'@');
        let wrap = need_wrap(&self.expression);
        p.wrap(wrap, |p| {
            self.expression.gen_expr(p, Precedence::Assign, Context::default());
        });
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSClassImplements<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.expression.gen(p, ctx);
        if let Some(type_parameters) = self.type_parameters.as_ref() {
            type_parameters.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeParameterDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"<");
        p.print_list(&self.params, ctx);
        p.print_str(b">");
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeAnnotation<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.type_annotation.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::TSFunctionType(ty) => ty.gen(p, ctx),
            Self::TSConstructorType(ty) => ty.gen(p, ctx),
            Self::TSArrayType(ty) => ty.gen(p, ctx),
            Self::TSTupleType(ty) => ty.gen(p, ctx),
            Self::TSUnionType(ty) => ty.gen(p, ctx),
            Self::TSIntersectionType(ty) => ty.gen(p, ctx),
            Self::TSConditionalType(ty) => ty.gen(p, ctx),
            Self::TSInferType(ty) => ty.gen(p, ctx),
            Self::TSIndexedAccessType(ty) => ty.gen(p, ctx),
            Self::TSMappedType(ty) => ty.gen(p, ctx),
            Self::TSNamedTupleMember(ty) => ty.gen(p, ctx),
            Self::TSLiteralType(ty) => ty.literal.gen(p, ctx),
            Self::TSImportType(ty) => ty.gen(p, ctx),
            Self::TSQualifiedName(ty) => ty.gen(p, ctx),
            Self::TSAnyKeyword(_) => p.print_str(b"any"),
            Self::TSBigIntKeyword(_) => p.print_str(b"bigint"),
            Self::TSBooleanKeyword(_) => p.print_str(b"boolean"),
            Self::TSIntrinsicKeyword(_) => p.print_str(b"intrinsic"),
            Self::TSNeverKeyword(_) => p.print_str(b"never"),
            Self::TSNullKeyword(_) => p.print_str(b"null"),
            Self::TSNumberKeyword(_) => p.print_str(b"number"),
            Self::TSObjectKeyword(_) => p.print_str(b"object"),
            Self::TSStringKeyword(_) => p.print_str(b"string"),
            Self::TSSymbolKeyword(_) => p.print_str(b"symbol"),
            Self::TSThisType(_) => p.print_str(b"this"),
            Self::TSUndefinedKeyword(_) => p.print_str(b"undefined"),
            Self::TSUnknownKeyword(_) => p.print_str(b"unknown"),
            Self::TSVoidKeyword(_) => p.print_str(b"void"),
            Self::TSTemplateLiteralType(ty) => ty.gen(p, ctx),
            Self::TSTypeLiteral(ty) => ty.gen(p, ctx),
            Self::TSTypeOperatorType(ty) => ty.gen(p, ctx),
            Self::TSTypePredicate(ty) => ty.gen(p, ctx),
            Self::TSTypeQuery(ty) => ty.gen(p, ctx),
            Self::TSTypeReference(ty) => ty.gen(p, ctx),
            Self::JSDocNullableType(ty) => ty.gen(p, ctx),
            Self::JSDocUnknownType(_ty) => p.print_str(b"unknown"),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSArrayType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"(");
        self.element_type.gen(p, ctx);
        p.print_str(b")[]");
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTupleType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"[");
        p.print_list(&self.element_types, ctx);
        p.print_str(b"]");
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSUnionType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if self.types.len() == 1 {
            self.types[0].gen(p, ctx);
            return;
        }
        p.print_str(b"(");
        for (index, item) in self.types.iter().enumerate() {
            if index != 0 {
                p.print_soft_space();
                p.print_str(b"|");
                p.print_soft_space();
            }
            p.print_str(b"(");
            item.gen(p, ctx);
            p.print_str(b")");
        }
        p.print_str(b")");
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSIntersectionType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if self.types.len() == 1 {
            self.types[0].gen(p, ctx);
            return;
        }

        p.print_str(b"(");
        for (index, item) in self.types.iter().enumerate() {
            if index != 0 {
                p.print_soft_space();
                p.print_str(b"&");
                p.print_soft_space();
            }
            p.print_str(b"(");
            item.gen(p, ctx);
            p.print_str(b")");
        }
        p.print_str(b")");
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSConditionalType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.check_type.gen(p, ctx);
        p.print_str(b" extends (");
        self.extends_type.gen(p, ctx);
        p.print_str(b") ? ");
        self.true_type.gen(p, ctx);
        p.print_str(b" : ");
        self.false_type.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSInferType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"infer ");
        self.type_parameter.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSIndexedAccessType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.object_type.gen(p, ctx);
        p.print_str(b"[");
        self.index_type.gen(p, ctx);
        p.print_str(b"]");
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSMappedType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"{");
        match self.readonly {
            TSMappedTypeModifierOperator::True => {
                p.print_str(b"readonly");
            }
            TSMappedTypeModifierOperator::Plus => {
                p.print_str(b"+readonly");
            }
            TSMappedTypeModifierOperator::Minus => {
                p.print_str(b"-readonly");
            }
            TSMappedTypeModifierOperator::None => {}
        }
        p.print_hard_space();
        p.print_str(b"[");
        self.type_parameter.name.gen(p, ctx);
        if let Some(constraint) = &self.type_parameter.constraint {
            p.print_str(b" in ");
            constraint.gen(p, ctx);
        }
        if let Some(default) = &self.type_parameter.default {
            p.print_str(b" = ");
            default.gen(p, ctx);
        }
        p.print_str(b"]");
        match self.optional {
            TSMappedTypeModifierOperator::True => {
                p.print_str(b"?");
            }
            TSMappedTypeModifierOperator::Plus => {
                p.print_str(b"+?");
            }
            TSMappedTypeModifierOperator::Minus => {
                p.print_str(b"-?");
            }
            TSMappedTypeModifierOperator::None => {}
        }
        p.print_soft_space();
        if let Some(type_annotation) = &self.type_annotation {
            p.print_str(b":");
            p.print_soft_space();
            type_annotation.gen(p, ctx);
        }
        p.print_str(b"}");
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSQualifiedName<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.left.gen(p, ctx);
        p.print_str(b".");
        self.right.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeOperator<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self.operator {
            TSTypeOperatorOperator::Keyof => {
                p.print_str(b"keyof ");
            }
            TSTypeOperatorOperator::Unique => {
                p.print_str(b"unique ");
            }
            TSTypeOperatorOperator::Readonly => {
                p.print_str(b"readonly ");
            }
        }
        self.type_annotation.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypePredicate<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if self.asserts {
            p.print_str(b"asserts ");
        }
        match &self.parameter_name {
            TSTypePredicateName::Identifier(ident) => {
                ident.gen(p, ctx);
            }
            TSTypePredicateName::This(_ident) => {
                p.print_str(b"this");
            }
        }
        if let Some(type_annotation) = &self.type_annotation {
            p.print_str(b" is ");
            type_annotation.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeReference<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.type_name.gen(p, ctx);
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for JSDocNullableType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if self.postfix {
            self.type_annotation.gen(p, ctx);
            p.print_str(b"?");
        } else {
            p.print_str(b"?");
            self.type_annotation.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTemplateLiteralType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"`");
        for (index, item) in self.quasis.iter().enumerate() {
            if index != 0 {
                if let Some(types) = self.types.get(index - 1) {
                    p.print_str(b"${");
                    types.gen(p, ctx);
                    p.print_str(b"}");
                }
            }
            p.print_str(item.value.raw.as_bytes());
        }
        p.print_str(b"`");
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeLiteral<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        let single_line = self.members.len() <= 1;
        p.print_curly_braces(self.span, single_line, |p| {
            for item in &self.members {
                p.print_indent();
                item.gen(p, ctx);
                if !single_line {
                    p.print_semicolon();
                    p.print_soft_newline();
                }
            }
        });
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeName<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::IdentifierReference(decl) => {
                p.print_str(decl.name.as_bytes());
            }
            Self::QualifiedName(decl) => {
                decl.left.gen(p, ctx);
                p.print_str(b".");
                decl.right.gen(p, ctx);
            }
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSLiteral<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::BooleanLiteral(decl) => decl.gen(p, ctx),
            Self::NullLiteral(decl) => decl.gen(p, ctx),
            Self::NumericLiteral(decl) => decl.gen(p, ctx),
            Self::BigintLiteral(decl) => decl.gen(p, ctx),
            Self::RegExpLiteral(decl) => decl.gen(p, ctx),
            Self::StringLiteral(decl) => decl.gen(p, ctx),
            Self::TemplateLiteral(decl) => decl.gen(p, ctx),
            Self::UnaryExpression(decl) => decl.gen_expr(p, Precedence::Assign, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeParameter<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.name.gen(p, ctx);
        if let Some(constraint) = &self.constraint {
            p.print_str(b" extends ");
            constraint.gen(p, ctx);
        }
        if let Some(default) = &self.default {
            p.print_str(b" = ");
            default.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSFunctionType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.gen(p, ctx);
        }
        p.print_str(b"(");
        if let Some(this_param) = &self.this_param {
            this_param.gen(p, ctx);
            if !self.params.is_empty() || self.params.rest.is_some() {
                p.print_str(b",");
            }
            p.print_soft_space();
        }
        self.params.gen(p, ctx);
        p.print_str(b")");
        p.print_soft_space();
        p.print_str(b"=>");
        p.print_soft_space();
        self.return_type.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSThisParameter<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.this.gen(p, ctx);
        if let Some(type_annotation) = &self.type_annotation {
            p.print_str(b": ");
            type_annotation.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSSignature<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::TSIndexSignature(signature) => signature.gen(p, ctx),
            Self::TSPropertySignature(signature) => {
                if signature.readonly {
                    p.print_str(b"readonly ");
                }
                if signature.computed {
                    p.print(b'[');
                    signature.key.gen(p, ctx);
                    p.print(b']');
                } else {
                    match &signature.key {
                        PropertyKey::StaticIdentifier(key) => {
                            key.gen(p, ctx);
                        }
                        PropertyKey::PrivateIdentifier(key) => {
                            p.print_str(key.name.as_bytes());
                        }
                        key @ match_expression!(PropertyKey) => {
                            key.to_expression().gen_expr(p, Precedence::Assign, ctx);
                        }
                    }
                }
                if signature.optional {
                    p.print_str(b"?");
                }
                if let Some(type_annotation) = &signature.type_annotation {
                    p.print_colon();
                    p.print_soft_space();
                    type_annotation.gen(p, ctx);
                }
            }
            Self::TSCallSignatureDeclaration(signature) => {
                p.print_str(b"(");
                if let Some(this_param) = &signature.this_param {
                    this_param.gen(p, ctx);
                    if !signature.params.is_empty() || signature.params.rest.is_some() {
                        p.print_str(b",");
                    }
                    p.print_soft_space();
                }
                signature.params.gen(p, ctx);
                p.print_str(b")");
                if let Some(return_type) = &signature.return_type {
                    p.print_colon();
                    p.print_soft_space();
                    return_type.gen(p, ctx);
                }
            }
            Self::TSConstructSignatureDeclaration(signature) => {
                p.print_str(b"new ");
                p.print_str(b"(");
                signature.params.gen(p, ctx);
                p.print_str(b")");
                if let Some(return_type) = &signature.return_type {
                    p.print_colon();
                    p.print_soft_space();
                    return_type.gen(p, ctx);
                }
            }
            Self::TSMethodSignature(signature) => {
                match signature.kind {
                    TSMethodSignatureKind::Method => {}
                    TSMethodSignatureKind::Get => p.print_str(b"get "),
                    TSMethodSignatureKind::Set => p.print_str(b"set "),
                }
                if signature.computed {
                    p.print(b'[');
                    signature.key.gen(p, ctx);
                    p.print(b']');
                } else {
                    match &signature.key {
                        PropertyKey::StaticIdentifier(key) => {
                            key.gen(p, ctx);
                        }
                        PropertyKey::PrivateIdentifier(key) => {
                            p.print_str(key.name.as_bytes());
                        }
                        key @ match_expression!(PropertyKey) => {
                            key.to_expression().gen_expr(p, Precedence::Assign, ctx);
                        }
                    }
                }
                if signature.optional {
                    p.print_str(b"?");
                }
                if let Some(type_parameters) = &signature.type_parameters {
                    type_parameters.gen(p, ctx);
                }
                p.print_str(b"(");
                if let Some(this_param) = &signature.this_param {
                    this_param.gen(p, ctx);
                    if !signature.params.is_empty() || signature.params.rest.is_some() {
                        p.print_str(b",");
                    }
                    p.print_soft_space();
                }
                signature.params.gen(p, ctx);
                p.print_str(b")");
                if let Some(return_type) = &signature.return_type {
                    p.print_colon();
                    p.print_soft_space();
                    return_type.gen(p, ctx);
                }
            }
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeQuery<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"typeof ");
        self.expr_name.gen(p, ctx);
        if let Some(type_params) = &self.type_parameters {
            type_params.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeQueryExprName<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            match_ts_type_name!(Self) => self.to_ts_type_name().gen(p, ctx),
            Self::TSImportType(decl) => decl.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSImportType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"import(");
        self.argument.gen(p, ctx);
        if let Some(attributes) = &self.attributes {
            p.print_str(", ");
            attributes.gen(p, ctx);
        }
        p.print_str(b")");
        if let Some(qualifier) = &self.qualifier {
            p.print(b'.');
            qualifier.gen(p, ctx);
        }
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSImportAttributes<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        // { with: { ... } }
        p.print_str(b"{ with: { ");
        p.print_list(&self.elements, ctx);
        p.print_str(b" }}");
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSImportAttribute<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.name.gen(p, ctx);
        p.print_str(": ");
        self.value.gen_expr(p, Precedence::Member, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSImportAttributeName<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            TSImportAttributeName::Identifier(ident) => ident.gen(p, ctx),
            TSImportAttributeName::StringLiteral(literal) => literal.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeParameterInstantiation<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"<");
        p.print_list(&self.params, ctx);
        p.print_str(b">");
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSIndexSignature<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if self.readonly {
            p.print_str(b"readonly ");
        }
        p.print_str(b"[");
        for (index, parameter) in self.parameters.iter().enumerate() {
            if index != 0 {
                p.print_str(b" | ");
            }
            p.print_str(parameter.name.as_bytes());
            p.print_colon();
            p.print_soft_space();
            parameter.type_annotation.gen(p, ctx);
        }
        p.print_str(b"]");
        p.print_colon();
        p.print_soft_space();
        self.type_annotation.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTupleElement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            match_ts_type!(TSTupleElement) => self.to_ts_type().gen(p, ctx),
            TSTupleElement::TSOptionalType(ts_type) => {
                ts_type.type_annotation.gen(p, ctx);
                p.print_str(b"?");
            }
            TSTupleElement::TSRestType(ts_type) => {
                p.print_str(b"...");
                ts_type.type_annotation.gen(p, ctx);
            }
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSNamedTupleMember<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.label.gen(p, ctx);
        p.print_str(b":");
        p.print_soft_space();
        self.element_type.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSModuleDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if self.modifiers.contains(ModifierKind::Export) {
            p.print_str(b"export ");
        }
        if self.modifiers.contains(ModifierKind::Declare) {
            p.print_str(b"declare ");
        }
        p.print_str(b"module");
        p.print_space_before_identifier();
        self.id.gen(p, ctx);

        if let Some(body) = &self.body {
            let mut body = body;
            loop {
                match body {
                    TSModuleDeclarationBody::TSModuleDeclaration(b) => {
                        p.print(b'.');
                        b.id.gen(p, ctx);
                        if let Some(b) = &b.body {
                            body = b;
                        } else {
                            break;
                        }
                    }
                    TSModuleDeclarationBody::TSModuleBlock(body) => {
                        p.print_soft_space();
                        body.gen(p, ctx);
                        break;
                    }
                }
            }
        }
        p.needs_semicolon = false;
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSModuleDeclarationName<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::Identifier(ident) => ident.gen(p, ctx),
            Self::StringLiteral(s) => s.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSModuleBlock<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        let is_empty = self.directives.is_empty() && self.body.is_empty();
        p.print_curly_braces(self.span, is_empty, |p| {
            p.print_directives_and_statements(Some(&self.directives), &self.body, ctx);
        });
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeAliasDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if self.modifiers.contains(ModifierKind::Export) {
            p.print_str(b"export ");
        }
        if self.modifiers.contains(ModifierKind::Declare) {
            p.print_str(b"declare ");
        }
        p.print_str(b"type");
        p.print_space_before_identifier();
        self.id.gen(p, ctx);
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.gen(p, ctx);
        }
        p.print_soft_space();
        p.print_str(b"=");
        p.print_soft_space();
        self.type_annotation.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSInterfaceDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"interface");
        p.print_hard_space();
        self.id.gen(p, ctx);
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.gen(p, ctx);
        }
        if let Some(extends) = &self.extends {
            if !extends.is_empty() {
                p.print_str(b" extends ");
                p.print_list(extends, ctx);
            }
        }
        p.print_soft_space();
        p.print_curly_braces(self.body.span, self.body.body.is_empty(), |p| {
            for item in &self.body.body {
                p.print_indent();
                item.gen(p, ctx);
                p.print_semicolon();
                p.print_soft_newline();
            }
        });
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSInterfaceHeritage<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.expression.gen_expr(p, Precedence::Call, ctx);
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSEnumDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_indent();
        if self.modifiers.contains(ModifierKind::Export) {
            p.print_str(b"export ");
        }
        if self.modifiers.contains(ModifierKind::Declare) {
            p.print_str(b"declare ");
        }
        if self.modifiers.contains(ModifierKind::Const) {
            p.print_str(b"const ");
        }
        p.print_space_before_identifier();
        p.print_str(b"enum ");
        self.id.gen(p, ctx);
        p.print_space_before_identifier();
        p.print_curly_braces(self.span, self.members.is_empty(), |p| {
            for member in &self.members {
                p.print_indent();
                member.gen(p, ctx);
                p.print_comma();
                p.print_soft_newline();
            }
        });
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSEnumMember<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match &self.id {
            TSEnumMemberName::StaticIdentifier(decl) => decl.gen(p, ctx),
            TSEnumMemberName::StaticStringLiteral(decl) => decl.gen(p, ctx),
            TSEnumMemberName::StaticNumericLiteral(decl) => decl.gen(p, ctx),
            decl @ match_expression!(TSEnumMemberName) => {
                p.print_str(b"[");
                decl.to_expression().gen_expr(p, Precedence::lowest(), ctx);
                p.print_str(b"]");
            }
        }
        if let Some(init) = &self.initializer {
            p.print_soft_space();
            p.print_equal();
            p.print_soft_space();
            init.gen_expr(p, Precedence::lowest(), ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSConstructorType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if self.r#abstract {
            p.print_str(b"abstract ");
        }
        p.print_str(b"new ");
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.gen(p, ctx);
        }
        p.print_str(b"(");
        self.params.gen(p, ctx);
        p.print_str(b")");
        p.print_soft_space();
        p.print_str(b"=>");
        p.print_soft_space();
        self.return_type.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSImportEqualsDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"import ");
        self.id.gen(p, ctx);
        p.print_str(b" = ");
        self.module_reference.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSModuleReference<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::ExternalModuleReference(decl) => {
                p.print_str(b"require(");
                decl.expression.gen(p, ctx);
                p.print_str(b")");
            }
            match_ts_type_name!(Self) => self.to_ts_type_name().gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for TSTypeAssertion<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        p.wrap(precedence > self.precedence(), |p| {
            p.print_str(b"<");
            self.type_annotation.gen(p, ctx);
            p.print_str(b">");
            self.expression.gen_expr(p, Precedence::Grouping, ctx);
        });
    }
}

impl<const MINIFY: bool> Gen<MINIFY> for TSAccessibility {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        match self {
            Self::Public => p.print_str(b"public "),
            Self::Private => p.print_str(b"private "),
            Self::Protected => p.print_str(b"protected "),
        }
    }
}
