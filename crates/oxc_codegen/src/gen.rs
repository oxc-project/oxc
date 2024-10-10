use std::{borrow::Cow, ops::Not};

use cow_utils::CowUtils;
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_syntax::{
    identifier::{LS, PS},
    operator::UnaryOperator,
    precedence::{GetPrecedence, Precedence},
};

use crate::{
    binary_expr_visitor::{BinaryExpressionVisitor, Binaryish, BinaryishOperator},
    Codegen, Context, Operator,
};

pub trait Gen: GetSpan {
    fn gen(&self, p: &mut Codegen, ctx: Context);

    fn print(&self, p: &mut Codegen, ctx: Context) {
        self.gen(p, ctx);
    }
}

pub trait GenExpr: GetSpan {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context);

    fn print_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        self.gen_expr(p, precedence, ctx);
    }
}

impl<'a> Gen for Program<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if let Some(hashbang) = &self.hashbang {
            hashbang.print(p, ctx);
        }
        for directive in &self.directives {
            directive.print(p, ctx);
        }
        for stmt in &self.body {
            stmt.print(p, ctx);
            p.print_semicolon_if_needed();
        }
    }
}

impl<'a> Gen for Hashbang<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.print_str("#!");
        p.print_str(self.value.as_str());
        p.print_hard_newline();
    }
}

impl<'a> Gen for Directive<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        // A Use Strict Directive may not contain an EscapeSequence or LineContinuation.
        // So here should print original `directive` value, the `expression` value is escaped str.
        // See https://github.com/babel/babel/blob/main/packages/babel-generator/src/generators/base.ts#L64
        p.wrap_quote(|p, _| {
            p.print_str(self.directive.as_str());
        });
        p.print_char(b';');
        p.print_soft_newline();
    }
}

impl<'a> Gen for Statement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_leading_comments(self.span().start);
        match self {
            Self::BlockStatement(stmt) => stmt.print(p, ctx),
            Self::BreakStatement(stmt) => stmt.print(p, ctx),
            Self::ContinueStatement(stmt) => stmt.print(p, ctx),
            Self::DebuggerStatement(stmt) => stmt.print(p, ctx),
            Self::DoWhileStatement(stmt) => stmt.print(p, ctx),
            Self::EmptyStatement(stmt) => stmt.print(p, ctx),
            Self::ExpressionStatement(stmt) => stmt.print(p, ctx),
            Self::ForInStatement(stmt) => stmt.print(p, ctx),
            Self::ForOfStatement(stmt) => stmt.print(p, ctx),
            Self::ForStatement(stmt) => stmt.print(p, ctx),
            Self::IfStatement(stmt) => stmt.print(p, ctx),
            Self::LabeledStatement(stmt) => stmt.print(p, ctx),
            Self::ReturnStatement(stmt) => stmt.print(p, ctx),
            Self::SwitchStatement(stmt) => stmt.print(p, ctx),
            Self::ThrowStatement(stmt) => stmt.print(p, ctx),
            Self::TryStatement(stmt) => stmt.print(p, ctx),
            Self::WhileStatement(stmt) => stmt.print(p, ctx),
            Self::WithStatement(stmt) => stmt.print(p, ctx),

            Self::ImportDeclaration(decl) => decl.print(p, ctx),
            Self::ExportAllDeclaration(decl) => decl.print(p, ctx),
            Self::ExportDefaultDeclaration(decl) => decl.print(p, ctx),
            Self::ExportNamedDeclaration(decl) => decl.print(p, ctx),
            Self::TSExportAssignment(decl) => decl.print(p, ctx),
            Self::TSNamespaceExportDeclaration(decl) => decl.print(p, ctx),

            Self::VariableDeclaration(decl) => {
                p.print_indent();
                decl.print(p, ctx);
                p.print_semicolon_after_statement();
            }
            Self::FunctionDeclaration(decl) => {
                p.print_indent();
                decl.print(p, ctx);
                p.print_soft_newline();
            }
            Self::ClassDeclaration(decl) => {
                p.print_indent();
                decl.print(p, ctx);
                p.print_soft_newline();
            }
            Self::TSModuleDeclaration(decl) => {
                p.print_indent();
                decl.print(p, ctx);
                p.print_soft_newline();
            }
            Self::TSTypeAliasDeclaration(decl) => {
                p.print_indent();
                decl.print(p, ctx);
                p.print_semicolon_after_statement();
            }
            Self::TSInterfaceDeclaration(decl) => {
                p.print_indent();
                decl.print(p, ctx);
                p.print_soft_newline();
            }
            Self::TSEnumDeclaration(decl) => {
                p.print_indent();
                decl.print(p, ctx);
                p.print_soft_newline();
            }
            Self::TSImportEqualsDeclaration(decl) => {
                p.print_indent();
                decl.print(p, ctx);
                p.print_semicolon_after_statement();
            }
        }
    }
}

impl<'a> Gen for ExpressionStatement<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
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

impl<'a> Gen for IfStatement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        print_if(self, p, ctx);
    }
}

fn print_if(if_stmt: &IfStatement<'_>, p: &mut Codegen, ctx: Context) {
    p.print_str("if");
    p.print_soft_space();
    p.print_char(b'(');
    p.print_expression(&if_stmt.test);
    p.print_char(b')');

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
            stmt.print(p, ctx);
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
        p.print_str("else");
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

impl<'a> Gen for BlockStatement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_indent();
        p.print_block_statement(self, ctx);
        p.print_soft_newline();
    }
}

impl<'a> Gen for ForStatement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str("for");
        p.print_soft_space();
        p.print_char(b'(');

        if let Some(init) = &self.init {
            init.print(p, Context::FORBID_IN);
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

        p.print_char(b')');
        p.print_body(&self.body, false, ctx);
    }
}

impl<'a> Gen for ForInStatement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str("for");
        p.print_soft_space();
        p.print_char(b'(');
        self.left.print(p, Context::empty().and_forbid_in(false));
        p.print_soft_space();
        p.print_space_before_identifier();
        p.print_str("in");
        p.print_hard_space();
        p.print_expression(&self.right);
        p.print_char(b')');
        p.print_body(&self.body, false, ctx);
    }
}

impl<'a> Gen for ForOfStatement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str("for");
        if self.r#await {
            p.print_str(" await");
        }
        p.print_soft_space();
        p.print_char(b'(');
        self.left.print(p, ctx);
        p.print_soft_space();
        p.print_space_before_identifier();
        p.print_str("of ");
        self.right.print_expr(p, Precedence::Comma, Context::empty());
        p.print_char(b')');
        p.print_body(&self.body, false, ctx);
    }
}

impl<'a> Gen for ForStatementInit<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            match_expression!(ForStatementInit) => {
                self.to_expression().print_expr(p, Precedence::Lowest, ctx);
            }
            Self::VariableDeclaration(var) => var.print(p, ctx),
        }
    }
}

impl<'a> Gen for ForStatementLeft<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            ForStatementLeft::VariableDeclaration(var) => var.print(p, ctx),
            ForStatementLeft::AssignmentTargetIdentifier(identifier) => {
                let wrap = identifier.name == "async";
                p.wrap(wrap, |p| self.to_assignment_target().print(p, ctx));
            }
            match_assignment_target!(ForStatementLeft) => {
                p.wrap(false, |p| self.to_assignment_target().print(p, ctx));
            }
        }
    }
}

impl<'a> Gen for WhileStatement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str("while");
        p.print_soft_space();
        p.print_char(b'(');
        p.print_expression(&self.test);
        p.print_char(b')');
        p.print_body(&self.body, false, ctx);
    }
}

impl<'a> Gen for DoWhileStatement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str("do ");
        if let Statement::BlockStatement(block) = &self.body {
            p.print_block_statement(block, ctx);
            p.print_soft_space();
        } else {
            p.print_soft_newline();
            p.indent();
            self.body.print(p, ctx);
            p.print_semicolon_if_needed();
            p.dedent();
            p.print_indent();
        }
        p.print_str("while");
        p.print_soft_space();
        p.print_char(b'(');
        p.print_expression(&self.test);
        p.print_char(b')');
        p.print_semicolon_after_statement();
    }
}

impl Gen for EmptyStatement {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_semicolon();
        p.print_soft_newline();
    }
}

impl<'a> Gen for ContinueStatement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str("continue");
        if let Some(label) = &self.label {
            p.print_hard_space();
            label.print(p, ctx);
        }
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for BreakStatement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str("break");
        if let Some(label) = &self.label {
            p.print_hard_space();
            label.print(p, ctx);
        }
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for SwitchStatement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str("switch");
        p.print_soft_space();
        p.print_char(b'(');
        p.print_expression(&self.discriminant);
        p.print_char(b')');
        p.print_soft_space();
        p.print_curly_braces(self.span, self.cases.is_empty(), |p| {
            for case in &self.cases {
                p.add_source_mapping(case.span.start);
                case.print(p, ctx);
            }
        });
        p.print_soft_newline();
        p.needs_semicolon = false;
    }
}

impl<'a> Gen for SwitchCase<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_semicolon_if_needed();
        p.print_indent();
        match &self.test {
            Some(test) => {
                p.print_str("case ");
                p.print_expression(test);
            }
            None => p.print_str("default"),
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
            item.print(p, ctx);
        }
        p.dedent();
    }
}

impl<'a> Gen for ReturnStatement<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_space_before_identifier();
        p.print_str("return");
        if let Some(arg) = &self.argument {
            p.print_hard_space();
            p.print_expression(arg);
        }
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for LabeledStatement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if !p.options.minify && (p.indent > 0 || p.print_next_indent_as_space) {
            p.add_source_mapping(self.span.start);
            p.print_indent();
        }
        p.print_space_before_identifier();
        self.label.print(p, ctx);
        p.print_colon();
        p.print_body(&self.body, false, ctx);
    }
}

impl<'a> Gen for TryStatement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_space_before_identifier();
        p.print_str("try");
        p.print_soft_space();
        p.print_block_statement(&self.block, ctx);
        if let Some(handler) = &self.handler {
            p.print_soft_space();
            p.print_str("catch");
            if let Some(param) = &handler.param {
                p.print_soft_space();
                p.print_str("(");
                param.pattern.print(p, ctx);
                p.print_str(")");
            }
            p.print_soft_space();
            p.print_block_statement(&handler.body, ctx);
        }
        if let Some(finalizer) = &self.finalizer {
            p.print_soft_space();
            p.print_str("finally");
            p.print_soft_space();
            p.print_block_statement(finalizer, ctx);
        }
        p.print_soft_newline();
    }
}

impl<'a> Gen for ThrowStatement<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str("throw ");
        p.print_expression(&self.argument);
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for WithStatement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str("with");
        p.print_char(b'(');
        p.print_expression(&self.object);
        p.print_char(b')');
        p.print_body(&self.body, false, ctx);
    }
}

impl Gen for DebuggerStatement {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str("debugger");
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for VariableDeclaration<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        if self.declare {
            p.print_str("declare ");
        }

        if p.preserve_annotate_comments()
            && p.start_of_annotation_comment.is_none()
            && matches!(self.kind, VariableDeclarationKind::Const)
            && matches!(self.declarations.first(), Some(VariableDeclarator { init: Some(init), .. }) if init.is_function())
            && p.has_annotation_comment(self.span.start)
        {
            p.start_of_annotation_comment = Some(self.span.start);
        }

        p.print_str(match self.kind {
            VariableDeclarationKind::Const => "const",
            VariableDeclarationKind::Let => "let",
            VariableDeclarationKind::Var => "var",
            VariableDeclarationKind::Using => "using",
            VariableDeclarationKind::AwaitUsing => "await using",
        });
        if !self.declarations.is_empty() {
            p.print_hard_space();
        }
        p.print_list(&self.declarations, ctx);
    }
}

impl<'a> Gen for VariableDeclarator<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.id.kind.print(p, ctx);
        if self.definite {
            p.print_char(b'!');
        }
        if self.id.optional {
            p.print_str("?");
        }
        if let Some(type_annotation) = &self.id.type_annotation {
            p.print_colon();
            p.print_soft_space();
            type_annotation.print(p, ctx);
        }
        if let Some(init) = &self.init {
            p.print_soft_space();
            p.print_equal();
            p.print_soft_space();
            p.print_annotation_comments(self.span.start);
            init.print_expr(p, Precedence::Comma, ctx);
        }
    }
}

impl<'a> Gen for Function<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        let n = p.code_len();
        let wrap = self.is_expression() && (p.start_of_stmt == n || p.start_of_default_export == n);
        p.print_annotation_comments(self.span.start);
        p.wrap(wrap, |p| {
            p.print_space_before_identifier();
            p.add_source_mapping(self.span.start);
            if self.declare {
                p.print_str("declare ");
            }
            if self.r#async {
                p.print_str("async ");
            }
            p.print_str("function");
            if self.generator {
                p.print_char(b'*');
                p.print_soft_space();
            }
            if let Some(id) = &self.id {
                p.print_space_before_identifier();
                id.print(p, ctx);
            }
            if let Some(type_parameters) = &self.type_parameters {
                type_parameters.print(p, ctx);
            }
            p.print_char(b'(');
            if let Some(this_param) = &self.this_param {
                this_param.print(p, ctx);
                if !self.params.is_empty() || self.params.rest.is_some() {
                    p.print_str(",");
                }
                p.print_soft_space();
            }
            self.params.print(p, ctx);
            p.print_char(b')');
            if let Some(return_type) = &self.return_type {
                p.print_str(": ");
                return_type.print(p, ctx);
            }
            if let Some(body) = &self.body {
                p.print_soft_space();
                body.print(p, ctx);
            } else {
                p.print_semicolon();
            }
        });
    }
}

impl<'a> Gen for FunctionBody<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_curly_braces(self.span, self.is_empty(), |p| {
            for directive in &self.directives {
                directive.print(p, ctx);
            }
            for stmt in &self.statements {
                p.print_semicolon_if_needed();
                stmt.print(p, ctx);
            }
        });
        p.needs_semicolon = false;
    }
}

impl<'a> Gen for FormalParameter<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        for decorator in &self.decorators {
            decorator.print(p, ctx);
            p.print_hard_space();
        }
        if let Some(accessibility) = self.accessibility {
            p.print_str(accessibility.as_str());
            p.print_hard_space();
        }
        if self.readonly {
            p.print_str("readonly ");
        }
        self.pattern.print(p, ctx);
    }
}

impl<'a> Gen for FormalParameters<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_list(&self.items, ctx);
        if let Some(rest) = &self.rest {
            if !self.items.is_empty() {
                p.print_comma();
                p.print_soft_space();
            }
            rest.print(p, ctx);
        }
    }
}

impl<'a> Gen for ImportDeclaration<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str("import ");
        if self.import_kind.is_type() {
            p.print_str("type ");
        }
        if let Some(specifiers) = &self.specifiers {
            if specifiers.is_empty() {
                p.print_str("{}");
                p.print_soft_space();
                p.print_str("from");
                p.print_soft_space();
                p.print_char(b'"');
                p.print_str(self.source.value.as_str());
                p.print_char(b'"');
                if let Some(with_clause) = &self.with_clause {
                    p.print_hard_space();
                    with_clause.print(p, ctx);
                }
                p.print_semicolon_after_statement();
                return;
            }

            let mut in_block = false;
            for (index, specifier) in specifiers.iter().enumerate() {
                match specifier {
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                        if in_block {
                            p.print_soft_space();
                            p.print_str("},");
                            in_block = false;
                        } else if index != 0 {
                            p.print_comma();
                            p.print_soft_space();
                        }
                        spec.local.print(p, ctx);
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                        if in_block {
                            p.print_soft_space();
                            p.print_str("},");
                            in_block = false;
                        } else if index != 0 {
                            p.print_comma();
                            p.print_soft_space();
                        }
                        p.print_str("* as ");
                        spec.local.print(p, ctx);
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
                            p.print_char(b'{');
                            p.print_soft_space();
                        }

                        if spec.import_kind.is_type() {
                            p.print_str("type ");
                        }

                        spec.imported.print(p, ctx);
                        let local_name = p.get_binding_identifier_name(&spec.local);
                        let imported_name = get_module_export_name(&spec.imported, p);
                        if imported_name.is_none() || imported_name != Some(local_name) {
                            p.print_str(" as ");
                            spec.local.print(p, ctx);
                        }
                    }
                }
            }
            if in_block {
                p.print_soft_space();
                p.print_char(b'}');
            }
            p.print_str(" from ");
        }
        self.source.print(p, ctx);
        if let Some(with_clause) = &self.with_clause {
            p.print_hard_space();
            with_clause.print(p, ctx);
        }
        p.add_source_mapping(self.span.end);
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for WithClause<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        self.attributes_keyword.print(p, ctx);
        p.print_soft_space();
        p.print_block_start(self.span.start);
        p.print_sequence(&self.with_entries, ctx);
        p.print_block_end(self.span.end);
    }
}

impl<'a> Gen for ImportAttribute<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match &self.key {
            ImportAttributeKey::Identifier(identifier) => {
                p.print_str(identifier.name.as_str());
            }
            ImportAttributeKey::StringLiteral(literal) => literal.print(p, ctx),
        };
        p.print_colon();
        p.print_soft_space();
        self.value.print(p, ctx);
    }
}

impl<'a> Gen for ExportNamedDeclaration<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();

        if p.preserve_annotate_comments() {
            match &self.declaration {
                Some(Declaration::FunctionDeclaration(_)) => {
                    p.print_annotation_comments(self.span.start);
                }
                Some(Declaration::VariableDeclaration(var_decl))
                    if matches!(var_decl.kind, VariableDeclarationKind::Const) =>
                {
                    if matches!(var_decl.declarations.first(), Some(VariableDeclarator { init: Some(init), .. }) if init.is_function())
                        && p.has_annotation_comment(self.span.start)
                    {
                        p.start_of_annotation_comment = Some(self.span.start);
                    }
                }
                _ => {}
            };
        }
        p.print_str("export ");
        if self.export_kind.is_type() {
            p.print_str("type ");
        }
        match &self.declaration {
            Some(decl) => {
                match decl {
                    Declaration::VariableDeclaration(decl) => decl.print(p, ctx),
                    Declaration::FunctionDeclaration(decl) => decl.print(p, ctx),
                    Declaration::ClassDeclaration(decl) => decl.print(p, ctx),
                    Declaration::TSModuleDeclaration(decl) => decl.print(p, ctx),
                    Declaration::TSTypeAliasDeclaration(decl) => decl.print(p, ctx),
                    Declaration::TSInterfaceDeclaration(decl) => decl.print(p, ctx),
                    Declaration::TSEnumDeclaration(decl) => decl.print(p, ctx),
                    Declaration::TSImportEqualsDeclaration(decl) => decl.print(p, ctx),
                }
                if matches!(
                    decl,
                    Declaration::VariableDeclaration(_)
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
                p.print_char(b'{');
                if !self.specifiers.is_empty() {
                    p.print_soft_space();
                    p.print_list(&self.specifiers, ctx);
                    p.print_soft_space();
                }
                p.print_char(b'}');
                if let Some(source) = &self.source {
                    p.print_soft_space();
                    p.print_str("from");
                    p.print_soft_space();
                    source.print(p, ctx);
                }
                p.print_semicolon_after_statement();
            }
        }
    }
}

impl<'a> Gen for TSExportAssignment<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_indent();
        p.print_str("export = ");
        self.expression.print_expr(p, Precedence::Lowest, ctx);
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for TSNamespaceExportDeclaration<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_indent();
        p.print_str("export as namespace ");
        self.id.print(p, ctx);
        p.print_semicolon_after_statement();
    }
}

fn get_module_export_name<'a>(
    module_export_name: &ModuleExportName<'a>,
    p: &Codegen<'a>,
) -> Option<&'a str> {
    match module_export_name {
        ModuleExportName::IdentifierName(ident) => Some(ident.name.as_str()),
        ModuleExportName::IdentifierReference(ident) => {
            Some(p.get_identifier_reference_name(ident))
        }
        ModuleExportName::StringLiteral(_) => None,
    }
}

impl<'a> Gen for ExportSpecifier<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if self.export_kind.is_type() {
            p.print_str("type ");
        }
        self.local.print(p, ctx);
        let local_name = get_module_export_name(&self.local, p);
        let exported_name = get_module_export_name(&self.exported, p);
        if exported_name.is_none() || local_name != exported_name {
            p.print_str(" as ");
            self.exported.print(p, ctx);
        }
    }
}

impl<'a> Gen for ModuleExportName<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::IdentifierName(ident) => ident.print(p, ctx),
            Self::IdentifierReference(ident) => ident.print(p, ctx),
            Self::StringLiteral(literal) => literal.print(p, ctx),
        };
    }
}

impl<'a> Gen for ExportAllDeclaration<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str("export ");
        if self.export_kind.is_type() {
            p.print_str("type ");
        }
        p.print_char(b'*');

        if let Some(exported) = &self.exported {
            p.print_str(" as ");
            exported.print(p, ctx);
        }

        p.print_str(" from ");
        self.source.print(p, ctx);
        if let Some(with_clause) = &self.with_clause {
            p.print_hard_space();
            with_clause.print(p, ctx);
        }
        p.print_semicolon_after_statement();
    }
}

impl<'a> Gen for ExportDefaultDeclaration<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_indent();
        p.print_str("export default ");
        self.declaration.print(p, ctx);
    }
}
impl<'a> Gen for ExportDefaultDeclarationKind<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            match_expression!(Self) => {
                p.start_of_default_export = p.code_len();
                self.to_expression().print_expr(p, Precedence::Comma, Context::empty());
                p.print_semicolon_after_statement();
            }
            Self::FunctionDeclaration(fun) => {
                fun.print(p, ctx);
                p.print_soft_newline();
            }
            Self::ClassDeclaration(class) => {
                class.print(p, ctx);
                p.print_soft_newline();
            }
            Self::TSInterfaceDeclaration(interface) => interface.print(p, ctx),
        }
    }
}

impl<'a> GenExpr for Expression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        match self {
            Self::BooleanLiteral(lit) => lit.print(p, ctx),
            Self::NullLiteral(lit) => lit.print(p, ctx),
            Self::NumericLiteral(lit) => lit.print(p, ctx),
            Self::BigIntLiteral(lit) => lit.print(p, ctx),
            Self::RegExpLiteral(lit) => lit.print(p, ctx),
            Self::StringLiteral(lit) => lit.print(p, ctx),
            Self::Identifier(ident) => ident.print(p, ctx),
            Self::ThisExpression(expr) => expr.print(p, ctx),
            match_member_expression!(Self) => {
                self.to_member_expression().print_expr(p, precedence, ctx);
            }
            Self::CallExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::ArrayExpression(expr) => expr.print(p, ctx),
            Self::ObjectExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::FunctionExpression(expr) => expr.print(p, ctx),
            Self::ArrowFunctionExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::YieldExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::UpdateExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::UnaryExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::BinaryExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::PrivateInExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::LogicalExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::ConditionalExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::AssignmentExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::SequenceExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::ImportExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::TemplateLiteral(literal) => literal.print(p, ctx),
            Self::TaggedTemplateExpression(expr) => expr.print(p, ctx),
            Self::Super(sup) => sup.print(p, ctx),
            Self::AwaitExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::ChainExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::NewExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::MetaProperty(expr) => expr.print(p, ctx),
            Self::ClassExpression(expr) => expr.print(p, ctx),
            Self::JSXElement(el) => el.print(p, ctx),
            Self::JSXFragment(fragment) => fragment.print(p, ctx),
            Self::ParenthesizedExpression(e) => e.print_expr(p, precedence, ctx),
            Self::TSAsExpression(e) => e.print_expr(p, precedence, ctx),
            Self::TSSatisfiesExpression(e) => e.print_expr(p, precedence, ctx),
            Self::TSTypeAssertion(e) => e.print_expr(p, precedence, ctx),
            Self::TSNonNullExpression(e) => e.print_expr(p, precedence, ctx),
            Self::TSInstantiationExpression(e) => e.print_expr(p, precedence, ctx),
        }
    }
}

impl<'a> GenExpr for ParenthesizedExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        self.expression.print_expr(p, precedence, ctx);
    }
}

impl<'a> Gen for IdentifierReference<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        let name = p.get_identifier_reference_name(self);
        p.print_space_before_identifier();
        p.add_source_mapping_for_name(self.span, name);
        p.print_str(name);
    }
}

impl<'a> Gen for IdentifierName<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str(self.name.as_str());
    }
}

impl<'a> Gen for BindingIdentifier<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        let name = p.get_binding_identifier_name(self);
        p.add_source_mapping_for_name(self.span, name);
        p.print_str(name);
    }
}

impl<'a> Gen for LabelIdentifier<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping_for_name(self.span, &self.name);
        p.print_str(self.name.as_str());
    }
}

impl Gen for BooleanLiteral {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_space_before_identifier();
        p.print_str(self.as_str());
    }
}

impl Gen for NullLiteral {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.print_space_before_identifier();
        p.add_source_mapping(self.span.start);
        p.print_str("null");
    }
}

// Need a space before "." if it could be parsed as a decimal point.
fn need_space_before_dot(s: &str, p: &mut Codegen) {
    if !s.bytes().any(|b| matches!(b, b'.' | b'e' | b'x')) {
        p.need_space_before_dot = p.code_len();
    }
}

impl<'a> Gen for NumericLiteral<'a> {
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        if !p.options.minify && !self.raw.is_empty() {
            p.print_str(self.raw);
            need_space_before_dot(self.raw, p);
        } else if self.value != f64::INFINITY {
            p.print_space_before_identifier();
            let abs_value = self.value.abs();
            if self.value.is_sign_negative() {
                p.print_space_before_operator(Operator::Unary(UnaryOperator::UnaryNegation));
                p.print_str("-");
            }
            let result = get_minified_number(abs_value);
            let bytes = result.as_str();
            p.print_str(bytes);
            need_space_before_dot(bytes, p);
        } else if self.value == f64::INFINITY && self.raw.is_empty() {
            p.print_str("Infinity");
            need_space_before_dot("Infinity", p);
        } else {
            p.print_str(self.raw);
            need_space_before_dot(self.raw, p);
        };
    }
}

// https://github.com/terser/terser/blob/c5315c3fd6321d6b2e076af35a70ef532f498505/lib/output.js#L2418
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_possible_wrap)]
fn get_minified_number(num: f64) -> String {
    use oxc_syntax::number::ToJsString;
    if num < 1000.0 && num.fract() == 0.0 {
        return num.to_js_string();
    }

    let mut s = num.to_js_string();

    if s.starts_with("0.") {
        s = s[1..].to_string();
    }

    s = s.cow_replacen("e+", "e", 1).to_string();

    let mut candidates = vec![s.clone()];

    if num.fract() == 0.0 {
        candidates.push(format!("0x{:x}", num as u128));
    }

    if s.starts_with(".0") {
        // create `1e-2`
        if let Some((i, _)) = s[1..].bytes().enumerate().find(|(_, c)| *c != b'0') {
            let len = i + 1; // `+1` to include the dot.
            let digits = &s[len..];
            candidates.push(format!("{digits}e-{}", digits.len() + len - 1));
        }
    } else if s.ends_with('0') {
        // create 1e2
        if let Some((len, _)) = s.bytes().rev().enumerate().find(|(_, c)| *c != b'0') {
            candidates.push(format!("{}e{len}", &s[0..s.len() - len]));
        }
    } else if let Some((integer, point, exponent)) =
        s.split_once('.').and_then(|(a, b)| b.split_once('e').map(|e| (a, e.0, e.1)))
    {
        // `1.2e101` -> ("1", "2", "101")
        candidates.push(format!(
            "{integer}{point}e{}",
            exponent.parse::<isize>().unwrap() - point.len() as isize
        ));
    }

    candidates.into_iter().min_by_key(String::len).unwrap()
}

impl<'a> Gen for BigIntLiteral<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str(self.raw.as_str());
    }
}

impl<'a> Gen for RegExpLiteral<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        let last = p.peek_nth(0);
        let pattern_text = p.source_text.map_or_else(
            || Cow::Owned(self.regex.pattern.to_string()),
            |src| self.regex.pattern.source_text(src),
        );
        // Avoid forming a single-line comment or "</script" sequence
        if Some('/') == last
            || (Some('<') == last && pattern_text.cow_to_lowercase().starts_with("script"))
        {
            p.print_hard_space();
        }
        p.print_char(b'/');
        p.print_str(pattern_text.as_ref());
        p.print_char(b'/');
        p.print_str(self.regex.flags.to_string().as_str());
        p.prev_reg_exp_end = p.code().len();
    }
}

fn print_unquoted_str(s: &str, quote: u8, p: &mut Codegen) {
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\x00' => {
                if chars.peek().is_some_and(|&next| next.is_ascii_digit()) {
                    p.print_str("\\x00");
                } else {
                    p.print_str("\\0");
                }
            }
            '\x07' => {
                p.print_str("\\x07");
            }
            // \b
            '\u{8}' => {
                p.print_str("\\b");
            }
            // \v
            '\u{b}' => {
                p.print_str("\\v");
            }
            // \f
            '\u{c}' => {
                p.print_str("\\f");
            }
            '\n' => {
                p.print_str("\\n");
            }
            '\r' => {
                p.print_str("\\r");
            }
            '\x1B' => {
                p.print_str("\\x1B");
            }
            '\\' => {
                p.print_str("\\\\");
            }
            '\'' => {
                if quote == b'\'' {
                    p.print_str("\\'");
                } else {
                    p.print_str("'");
                }
            }
            '\"' => {
                if quote == b'"' {
                    p.print_str("\\\"");
                } else {
                    p.print_str("\"");
                }
            }
            '`' => {
                if quote == b'`' {
                    p.print_str("\\`");
                } else {
                    p.print_str("`");
                }
            }
            '$' => {
                if chars.peek().is_some_and(|&next| next == '{') {
                    p.print_str("\\$");
                } else {
                    p.print_str("$");
                }
            }
            // Allow `U+2028` and `U+2029` in string literals
            // <https://tc39.es/proposal-json-superset>
            // <https://github.com/tc39/proposal-json-superset>
            LS => p.print_str("\\u2028"),
            PS => p.print_str("\\u2029"),
            '\u{a0}' => {
                p.print_str("\\xA0");
            }
            _ => {
                p.print_str(c.encode_utf8([0; 4].as_mut()));
            }
        }
    }
}

impl<'a> Gen for StringLiteral<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        let s = self.value.as_str();
        p.wrap_quote(|p, quote| {
            print_unquoted_str(s, quote, p);
        });
    }
}

impl Gen for ThisExpression {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_space_before_identifier();
        p.print_str("this");
    }
}

impl<'a> GenExpr for MemberExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        match self {
            Self::ComputedMemberExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::StaticMemberExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::PrivateFieldExpression(expr) => expr.print_expr(p, precedence, ctx),
        }
    }
}

impl<'a> GenExpr for ComputedMemberExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, _precedence: Precedence, ctx: Context) {
        // `(let[0] = 100);` -> `(let)[0] = 100`;
        let wrap = self.object.get_identifier_reference().is_some_and(|r| r.name == "let");
        p.wrap(wrap, |p| {
            self.object.print_expr(p, Precedence::Postfix, ctx.intersection(Context::FORBID_CALL));
        });
        if self.optional {
            p.print_str("?.");
        }
        p.print_char(b'[');
        self.expression.print_expr(p, Precedence::Lowest, Context::empty());
        p.print_char(b']');
    }
}

impl<'a> GenExpr for StaticMemberExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, _precedence: Precedence, ctx: Context) {
        self.object.print_expr(p, Precedence::Postfix, ctx.intersection(Context::FORBID_CALL));
        if self.optional {
            p.print_char(b'?');
        } else if p.need_space_before_dot == p.code_len() {
            // `0.toExponential()` is invalid, add a space before the dot, `0 .toExponential()` is valid
            p.print_hard_space();
        }
        p.print_char(b'.');
        self.property.print(p, ctx);
    }
}

impl<'a> GenExpr for PrivateFieldExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, _precedence: Precedence, ctx: Context) {
        self.object.print_expr(p, Precedence::Prefix, ctx.intersection(Context::FORBID_CALL));
        if self.optional {
            p.print_str("?");
        }
        p.print_char(b'.');
        self.field.print(p, ctx);
    }
}

impl<'a> GenExpr for CallExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        let is_export_default = p.start_of_default_export == p.code_len();
        let mut wrap = precedence >= Precedence::New || ctx.intersects(Context::FORBID_CALL);
        if p.has_annotation_comment(self.span.start) && precedence >= Precedence::Postfix {
            wrap = true;
        }

        p.wrap(wrap, |p| {
            p.print_annotation_comments(self.span.start);
            if is_export_default {
                p.start_of_default_export = p.code_len();
            }
            p.add_source_mapping(self.span.start);
            self.callee.print_expr(p, Precedence::Postfix, Context::empty());
            if self.optional {
                p.print_str("?.");
            }
            if let Some(type_parameters) = &self.type_parameters {
                type_parameters.print(p, ctx);
            }
            p.print_char(b'(');
            let has_comment = (self.span.end > 0 && p.has_comment(self.span.end - 1))
                || self.arguments.iter().any(|item| p.has_comment(item.span().start));
            if has_comment {
                p.indent();
                p.print_list_with_comments(&self.arguments, ctx);
                // Handle `/* comment */);`
                if !p.print_expr_comments(self.span.end - 1) {
                    p.print_soft_newline();
                }
                p.dedent();
            } else {
                p.print_list(&self.arguments, ctx);
            }
            p.print_char(b')');
            p.add_source_mapping(self.span.end);
        });
    }
}

impl<'a> Gen for Argument<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::SpreadElement(elem) => elem.print(p, ctx),
            match_expression!(Self) => {
                self.to_expression().print_expr(p, Precedence::Comma, Context::empty());
            }
        }
    }
}

impl<'a> Gen for ArrayExpressionElement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            match_expression!(Self) => {
                self.to_expression().print_expr(p, Precedence::Comma, Context::empty());
            }
            Self::SpreadElement(elem) => elem.print(p, ctx),
            Self::Elision(_span) => {}
        }
    }
}

impl<'a> Gen for SpreadElement<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_ellipsis();
        self.argument.print_expr(p, Precedence::Comma, Context::empty());
    }
}

impl<'a> Gen for ArrayExpression<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        let is_multi_line = self.elements.len() > 2;
        p.add_source_mapping(self.span.start);
        p.print_char(b'[');
        if is_multi_line {
            p.indent();
        }
        for (i, item) in self.elements.iter().enumerate() {
            if i != 0 {
                p.print_comma();
            }
            if is_multi_line {
                p.print_soft_newline();
                p.print_indent();
            } else if i != 0 {
                p.print_soft_space();
            }
            item.print(p, ctx);
            if i == self.elements.len() - 1 && matches!(item, ArrayExpressionElement::Elision(_)) {
                p.print_comma();
            }
        }
        if is_multi_line {
            p.print_soft_newline();
            p.dedent();
            p.print_indent();
        }
        p.add_source_mapping(self.span.end);
        p.print_char(b']');
    }
}

impl<'a> GenExpr for ObjectExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, _precedence: Precedence, ctx: Context) {
        let n = p.code_len();
        let len = self.properties.len();
        let is_multi_line = len > 1;
        let wrap = p.start_of_stmt == n || p.start_of_arrow_expr == n;
        p.wrap(wrap, |p| {
            p.add_source_mapping(self.span.start);
            p.print_char(b'{');
            if is_multi_line {
                p.indent();
            }
            for (i, item) in self.properties.iter().enumerate() {
                if i != 0 {
                    p.print_comma();
                }
                if is_multi_line {
                    p.print_soft_newline();
                    p.print_indent();
                } else {
                    p.print_soft_space();
                }
                item.print(p, ctx);
            }
            if is_multi_line {
                p.print_soft_newline();
                p.dedent();
                p.print_indent();
            } else if len > 0 {
                p.print_soft_space();
            }
            p.add_source_mapping(self.span.end);
            p.print_char(b'}');
        });
    }
}

impl<'a> Gen for ObjectPropertyKind<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::ObjectProperty(prop) => prop.print(p, ctx),
            Self::SpreadProperty(elem) => elem.print(p, ctx),
        }
    }
}

impl<'a> Gen for ObjectProperty<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if let Expression::FunctionExpression(func) = &self.value {
            p.add_source_mapping(self.span.start);
            let is_accessor = match &self.kind {
                PropertyKind::Init => false,
                PropertyKind::Get => {
                    p.add_source_mapping(self.span.start);
                    p.print_str("get ");
                    true
                }
                PropertyKind::Set => {
                    p.add_source_mapping(self.span.start);
                    p.print_str("set ");
                    true
                }
            };
            if self.method || is_accessor {
                if func.r#async {
                    p.print_str("async ");
                }
                if func.generator {
                    p.print_str("*");
                }
                if self.computed {
                    p.print_char(b'[');
                }
                self.key.print(p, ctx);
                if self.computed {
                    p.print_char(b']');
                }
                if let Some(type_parameters) = &func.type_parameters {
                    type_parameters.print(p, ctx);
                }
                p.print_char(b'(');
                func.params.print(p, ctx);
                p.print_char(b')');
                if let Some(body) = &func.body {
                    p.print_soft_space();
                    body.print(p, ctx);
                }
                return;
            }
        }

        let mut shorthand = false;
        if let PropertyKey::StaticIdentifier(key) = &self.key {
            if let Expression::Identifier(ident) = self.value.without_parentheses() {
                if key.name == p.get_identifier_reference_name(ident) && key.name != "__proto__" {
                    shorthand = true;
                }
            }
        }

        if self.computed {
            p.print_char(b'[');
        }
        if !shorthand {
            self.key.print(p, ctx);
        }
        if self.computed {
            p.print_char(b']');
        }
        if !shorthand {
            p.print_colon();
            p.print_soft_space();
        }
        self.value.print_expr(p, Precedence::Comma, Context::empty());
    }
}

impl<'a> Gen for PropertyKey<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::StaticIdentifier(ident) => ident.print(p, ctx),
            Self::PrivateIdentifier(ident) => ident.print(p, ctx),
            match_expression!(Self) => {
                self.to_expression().print_expr(p, Precedence::Comma, Context::empty());
            }
        }
    }
}

impl<'a> GenExpr for ArrowFunctionExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        p.wrap(precedence >= Precedence::Assign, |p| {
            p.print_annotation_comments(self.span.start);
            if self.r#async {
                p.add_source_mapping(self.span.start);
                p.print_str("async");
            }

            if self.r#async {
                p.print_hard_space();
            }

            if let Some(type_parameters) = &self.type_parameters {
                type_parameters.print(p, ctx);
            }
            p.add_source_mapping(self.span.start);
            p.print_char(b'(');
            self.params.print(p, ctx);
            p.print_char(b')');
            if let Some(return_type) = &self.return_type {
                p.print_str(":");
                p.print_soft_space();
                return_type.print(p, ctx);
            }
            p.print_soft_space();
            p.print_str("=>");
            p.print_soft_space();
            if self.expression {
                if let Some(Statement::ExpressionStatement(stmt)) = &self.body.statements.first() {
                    p.start_of_arrow_expr = p.code_len();
                    stmt.expression.print_expr(p, Precedence::Comma, ctx.and_forbid_in(true));
                }
            } else {
                self.body.print(p, ctx);
            }
        });
    }
}

impl<'a> GenExpr for YieldExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, _ctx: Context) {
        p.wrap(precedence >= Precedence::Assign, |p| {
            p.add_source_mapping(self.span.start);
            p.print_space_before_identifier();
            p.print_str("yield");
            if self.delegate {
                p.print_char(b'*');
                p.print_soft_space();
            }
            if let Some(argument) = self.argument.as_ref() {
                if !self.delegate {
                    p.print_hard_space();
                }
                argument.print_expr(p, Precedence::Yield, Context::empty());
            }
        });
    }
}

impl<'a> GenExpr for UpdateExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        let operator = self.operator.as_str();
        p.wrap(precedence >= self.precedence(), |p| {
            if self.prefix {
                p.add_source_mapping(self.span.start);
                p.print_space_before_operator(self.operator.into());
                p.print_str(operator);
                p.prev_op = Some(self.operator.into());
                p.prev_op_end = p.code().len();
                self.argument.print_expr(p, Precedence::Prefix, ctx);
            } else {
                p.print_space_before_operator(self.operator.into());
                self.argument.print_expr(p, Precedence::Postfix, ctx);
                p.print_str(operator);
                p.prev_op = Some(self.operator.into());
                p.prev_op_end = p.code().len();
            }
        });
    }
}

impl<'a> GenExpr for UnaryExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        p.wrap(precedence >= self.precedence(), |p| {
            let operator = self.operator.as_str();
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
            self.argument.print_expr(p, Precedence::Exponentiation, ctx);
        });
    }
}

impl<'a> GenExpr for BinaryExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        let v = BinaryExpressionVisitor {
            // SAFETY:
            // The pointer is stored on the heap and all will be consumed in the binary expression visitor.
            e: Binaryish::Binary(unsafe {
                std::mem::transmute::<&BinaryExpression<'_>, &BinaryExpression<'_>>(self)
            }),
            precedence,
            ctx,
            left_precedence: Precedence::Lowest,
            left_ctx: Context::empty(),
            operator: BinaryishOperator::Binary(self.operator),
            wrap: false,
            right_precedence: Precedence::Lowest,
        };
        BinaryExpressionVisitor::gen_expr(v, p);
    }
}

impl<'a> GenExpr for PrivateInExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        p.wrap(precedence >= Precedence::Compare, |p| {
            self.left.print(p, ctx);
            p.print_str(" in ");
            self.right.print_expr(p, Precedence::Equals, Context::empty());
        });
    }
}

impl<'a> GenExpr for LogicalExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        let v = BinaryExpressionVisitor {
            // SAFETY:
            // The pointer is stored on the heap and all will be consumed in the binary expression visitor.
            e: Binaryish::Logical(unsafe {
                std::mem::transmute::<&LogicalExpression<'_>, &LogicalExpression<'_>>(self)
            }),
            precedence,
            ctx,
            left_precedence: Precedence::Lowest,
            left_ctx: Context::empty(),
            operator: BinaryishOperator::Logical(self.operator),
            wrap: false,
            right_precedence: Precedence::Lowest,
        };
        BinaryExpressionVisitor::gen_expr(v, p);
    }
}

impl<'a> GenExpr for ConditionalExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        let mut ctx = ctx;
        let wrap = precedence >= self.precedence();
        if wrap {
            ctx &= Context::FORBID_IN.not();
        }
        p.wrap(wrap, |p| {
            self.test.print_expr(p, Precedence::Conditional, ctx & Context::FORBID_IN);
            p.print_soft_space();
            p.print_char(b'?');
            p.print_soft_space();
            self.consequent.print_expr(p, Precedence::Yield, Context::empty());
            p.print_soft_space();
            p.print_colon();
            p.print_soft_space();
            self.alternate.print_expr(p, Precedence::Yield, ctx & Context::FORBID_IN);
        });
    }
}

impl<'a> GenExpr for AssignmentExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        let n = p.code_len();
        // Destructuring assignments must be parenthesized
        let wrap = (p.start_of_stmt == n || p.start_of_arrow_expr == n)
            && matches!(self.left, AssignmentTarget::ObjectAssignmentTarget(_));
        p.wrap(wrap || precedence >= self.precedence(), |p| {
            self.left.print(p, ctx);
            p.print_soft_space();
            p.print_str(self.operator.as_str());
            p.print_soft_space();
            self.right.print_expr(p, Precedence::Comma, ctx);
        });
    }
}

impl<'a> Gen for AssignmentTarget<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            match_simple_assignment_target!(Self) => {
                self.to_simple_assignment_target().print_expr(
                    p,
                    Precedence::Comma,
                    Context::empty(),
                );
            }
            match_assignment_target_pattern!(Self) => {
                self.to_assignment_target_pattern().print(p, ctx);
            }
        }
    }
}

impl<'a> GenExpr for SimpleAssignmentTarget<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.print(p, ctx),
            match_member_expression!(Self) => {
                self.to_member_expression().print_expr(p, precedence, ctx);
            }
            Self::TSAsExpression(e) => e.print_expr(p, precedence, ctx),
            Self::TSSatisfiesExpression(e) => e.print_expr(p, precedence, ctx),
            Self::TSNonNullExpression(e) => e.print_expr(p, precedence, ctx),
            Self::TSTypeAssertion(e) => e.print_expr(p, precedence, ctx),
            Self::TSInstantiationExpression(e) => e.print_expr(p, precedence, ctx),
        }
    }
}

impl<'a> Gen for AssignmentTargetPattern<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::ArrayAssignmentTarget(target) => target.print(p, ctx),
            Self::ObjectAssignmentTarget(target) => target.print(p, ctx),
        }
    }
}

impl<'a> Gen for ArrayAssignmentTarget<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_char(b'[');
        for (index, item) in self.elements.iter().enumerate() {
            if index != 0 {
                p.print_comma();
                p.print_soft_space();
            }
            if let Some(item) = item {
                item.print(p, ctx);
            }
        }
        if let Some(target) = &self.rest {
            if !self.elements.is_empty() {
                p.print_comma();
            }
            p.add_source_mapping(self.span.start);
            target.print(p, ctx);
        }
        if self.trailing_comma.is_some() {
            p.print_comma();
        }
        p.print_char(b']');
        p.add_source_mapping(self.span.end);
    }
}

impl<'a> Gen for ObjectAssignmentTarget<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_char(b'{');
        p.print_list(&self.properties, ctx);
        if let Some(target) = &self.rest {
            if !self.properties.is_empty() {
                p.print_comma();
            }
            p.add_source_mapping(self.span.start);
            target.print(p, ctx);
        }
        p.print_char(b'}');
        p.add_source_mapping(self.span.end);
    }
}

impl<'a> Gen for AssignmentTargetMaybeDefault<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            match_assignment_target!(Self) => self.to_assignment_target().print(p, ctx),
            Self::AssignmentTargetWithDefault(target) => target.print(p, ctx),
        }
    }
}

impl<'a> Gen for AssignmentTargetWithDefault<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.binding.print(p, ctx);
        p.print_soft_space();
        p.print_equal();
        p.print_soft_space();
        self.init.print_expr(p, Precedence::Comma, Context::empty());
    }
}

impl<'a> Gen for AssignmentTargetProperty<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::AssignmentTargetPropertyIdentifier(ident) => ident.print(p, ctx),
            Self::AssignmentTargetPropertyProperty(prop) => prop.print(p, ctx),
        }
    }
}

impl<'a> Gen for AssignmentTargetPropertyIdentifier<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        let ident_name = p.get_identifier_reference_name(&self.binding).to_owned();
        if ident_name == self.binding.name.as_str() {
            self.binding.print(p, ctx);
        } else {
            // `({x: a} = y);`
            p.print_str(self.binding.name.as_str());
            p.print_colon();
            p.print_soft_space();
            p.print_str(&ident_name);
        }
        if let Some(expr) = &self.init {
            p.print_soft_space();
            p.print_equal();
            p.print_soft_space();
            expr.print_expr(p, Precedence::Comma, Context::empty());
        }
    }
}

impl<'a> Gen for AssignmentTargetPropertyProperty<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match &self.name {
            PropertyKey::StaticIdentifier(ident) => {
                ident.print(p, ctx);
            }
            PropertyKey::PrivateIdentifier(ident) => {
                ident.print(p, ctx);
            }
            key @ match_expression!(PropertyKey) => {
                p.print_char(b'[');
                key.to_expression().print_expr(p, Precedence::Comma, Context::empty());
                p.print_char(b']');
            }
        }
        p.print_colon();
        p.print_soft_space();
        self.binding.print(p, ctx);
    }
}

impl<'a> Gen for AssignmentTargetRest<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_ellipsis();
        self.target.print(p, ctx);
    }
}

impl<'a> GenExpr for SequenceExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, _ctx: Context) {
        p.wrap(precedence >= self.precedence(), |p| {
            p.print_expressions(&self.expressions, Precedence::Lowest, Context::empty());
        });
    }
}

impl<'a> GenExpr for ImportExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        let wrap = precedence >= Precedence::New || ctx.intersects(Context::FORBID_CALL);
        let has_comment = (self.span.end > 0 && p.has_comment(self.span.end - 1))
            || p.has_comment(self.source.span().start)
            || self.arguments.first().is_some_and(|argument| p.has_comment(argument.span().start));

        p.wrap(wrap, |p| {
            p.add_source_mapping(self.span.start);
            p.print_str("import(");
            if has_comment {
                p.indent();
            }
            if p.print_expr_comments(self.source.span().start) {
                p.print_indent();
            } else if has_comment {
                p.print_soft_newline();
                p.print_indent();
            }
            self.source.print_expr(p, Precedence::Comma, Context::empty());
            if !self.arguments.is_empty() {
                p.print_comma();
                if has_comment {
                    p.print_soft_newline();
                    p.print_indent();
                } else {
                    p.print_soft_space();
                }
                p.print_expressions(&self.arguments, Precedence::Comma, Context::empty());
            }
            if has_comment {
                // Handle `/* comment */);`
                if !p.print_expr_comments(self.span.end - 1) {
                    p.print_soft_newline();
                }
                p.dedent();
            }
            p.print_char(b')');
        });
    }
}

impl<'a> Gen for TemplateLiteral<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.print_char(b'`');
        let mut expressions = self.expressions.iter();

        for quasi in &self.quasis {
            p.add_source_mapping(quasi.span.start);
            p.print_str(quasi.value.raw.as_str());

            if let Some(expr) = expressions.next() {
                p.print_str("${");
                p.print_expression(expr);
                p.print_char(b'}');
            }
        }

        p.print_char(b'`');
    }
}

impl<'a> Gen for TaggedTemplateExpression<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        self.tag.print_expr(p, Precedence::Postfix, Context::empty());
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.print(p, ctx);
        }
        self.quasi.print(p, ctx);
    }
}

impl Gen for Super {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str("super");
    }
}

impl<'a> GenExpr for AwaitExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        p.wrap(precedence >= self.precedence(), |p| {
            p.add_source_mapping(self.span.start);
            p.print_str("await ");
            self.argument.print_expr(p, Precedence::Exponentiation, ctx);
        });
    }
}

impl<'a> GenExpr for ChainExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        match &self.expression {
            ChainElement::CallExpression(expr) => expr.print_expr(p, precedence, ctx),
            match_member_expression!(ChainElement) => {
                self.expression.to_member_expression().print_expr(p, precedence, ctx);
            }
        }
    }
}

impl<'a> GenExpr for NewExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        let mut wrap = precedence >= self.precedence();
        if p.has_annotation_comment(self.span.start) && precedence >= Precedence::Postfix {
            wrap = true;
        }
        p.wrap(wrap, |p| {
            p.print_annotation_comments(self.span.start);
            p.print_space_before_identifier();
            p.add_source_mapping(self.span.start);
            p.print_str("new ");
            self.callee.print_expr(p, Precedence::New, Context::FORBID_CALL);
            p.print_char(b'(');
            let has_comment = p.has_comment(self.span.end - 1)
                || self.arguments.iter().any(|item| p.has_comment(item.span().start));
            if has_comment {
                p.indent();
                p.print_list_with_comments(&self.arguments, ctx);
                // Handle `/* comment */);`
                if !p.print_expr_comments(self.span.end - 1) {
                    p.print_soft_newline();
                }
                p.dedent();
            } else {
                p.print_list(&self.arguments, ctx);
            }
            p.print_char(b')');
        });
    }
}

impl<'a> GenExpr for TSAsExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        let wrap = precedence >= Precedence::Shift;

        p.wrap(wrap, |p| {
            self.expression.print_expr(p, Precedence::Exponentiation, ctx);
            p.print_str(" as ");
            self.type_annotation.print(p, ctx);
        });
    }
}

impl<'a> GenExpr for TSSatisfiesExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        p.print_char(b'(');
        p.print_char(b'(');
        self.expression.print_expr(p, precedence, Context::default());
        p.print_char(b')');
        p.print_str(" satisfies ");
        self.type_annotation.print(p, ctx);
        p.print_char(b')');
    }
}

impl<'a> GenExpr for TSNonNullExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        p.wrap(matches!(self.expression, Expression::ParenthesizedExpression(_)), |p| {
            self.expression.print_expr(p, precedence, ctx);
        });
        p.print_char(b'!');
        if p.options.minify {
            p.print_hard_space();
        }
    }
}

impl<'a> GenExpr for TSInstantiationExpression<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        self.expression.print_expr(p, precedence, ctx);
        self.type_parameters.print(p, ctx);
        if p.options.minify {
            p.print_hard_space();
        }
    }
}

impl<'a> GenExpr for TSTypeAssertion<'a> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        p.wrap(precedence >= self.precedence(), |p| {
            p.print_str("<");
            // var r = < <T>(x: T) => T > ((x) => { return null; });
            //          ^ make sure space is printed here.
            if matches!(self.type_annotation, TSType::TSFunctionType(_)) {
                p.print_hard_space();
            }
            self.type_annotation.print(p, ctx);
            p.print_str(">");
            self.expression.print_expr(p, Precedence::Member, ctx);
        });
    }
}

impl<'a> Gen for MetaProperty<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        self.meta.print(p, ctx);
        p.print_char(b'.');
        self.property.print(p, ctx);
    }
}

impl<'a> Gen for Class<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        let n = p.code_len();
        let wrap = self.is_expression() && (p.start_of_stmt == n || p.start_of_default_export == n);
        p.wrap(wrap, |p| {
            for decorator in &self.decorators {
                decorator.print(p, ctx);
                p.print_hard_space();
            }
            if self.declare {
                p.print_str("declare ");
            }
            if self.r#abstract {
                p.print_str("abstract ");
            }
            p.print_str("class");
            if let Some(id) = &self.id {
                p.print_hard_space();
                id.print(p, ctx);
                if let Some(type_parameters) = self.type_parameters.as_ref() {
                    type_parameters.print(p, ctx);
                }
            }
            if let Some(super_class) = self.super_class.as_ref() {
                p.print_str(" extends ");
                super_class.print_expr(p, Precedence::Call, Context::empty());
                if let Some(super_type_parameters) = &self.super_type_parameters {
                    super_type_parameters.print(p, ctx);
                }
            }
            if let Some(implements) = self.implements.as_ref() {
                p.print_str(" implements ");
                p.print_list(implements, ctx);
            }
            p.print_soft_space();
            self.body.print(p, ctx);
            p.needs_semicolon = false;
        });
    }
}

impl<'a> Gen for ClassBody<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_curly_braces(self.span, self.body.is_empty(), |p| {
            for item in &self.body {
                p.print_semicolon_if_needed();
                p.print_indent();
                item.print(p, ctx);
            }
        });
    }
}

impl<'a> Gen for ClassElement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::StaticBlock(elem) => {
                p.print_leading_comments(elem.span.start);
                elem.print(p, ctx);
                p.print_soft_newline();
            }
            Self::MethodDefinition(elem) => {
                p.print_leading_comments(elem.span.start);
                elem.print(p, ctx);
                p.print_soft_newline();
            }
            Self::PropertyDefinition(elem) => {
                p.print_leading_comments(elem.span.start);
                elem.print(p, ctx);
                p.print_semicolon_after_statement();
            }
            Self::AccessorProperty(elem) => {
                p.print_leading_comments(elem.span.start);
                elem.print(p, ctx);
                p.print_semicolon_after_statement();
            }
            Self::TSIndexSignature(elem) => {
                p.print_leading_comments(elem.span.start);
                elem.print(p, ctx);
                p.print_semicolon_after_statement();
            }
        }
    }
}

impl<'a> Gen for JSXIdentifier<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping_for_name(self.span, &self.name);
        p.print_str(self.name.as_str());
    }
}

impl<'a> Gen for JSXMemberExpressionObject<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::IdentifierReference(ident) => ident.print(p, ctx),
            Self::MemberExpression(member_expr) => member_expr.print(p, ctx),
            Self::ThisExpression(expr) => expr.print(p, ctx),
        }
    }
}

impl<'a> Gen for JSXMemberExpression<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.object.print(p, ctx);
        p.print_char(b'.');
        self.property.print(p, ctx);
    }
}

impl<'a> Gen for JSXElementName<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::Identifier(identifier) => identifier.print(p, ctx),
            Self::IdentifierReference(identifier) => identifier.print(p, ctx),
            Self::NamespacedName(namespaced_name) => namespaced_name.print(p, ctx),
            Self::MemberExpression(member_expr) => member_expr.print(p, ctx),
            Self::ThisExpression(expr) => expr.print(p, ctx),
        }
    }
}

impl<'a> Gen for JSXNamespacedName<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.namespace.print(p, ctx);
        p.print_colon();
        self.property.print(p, ctx);
    }
}

impl<'a> Gen for JSXAttributeName<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::Identifier(ident) => ident.print(p, ctx),
            Self::NamespacedName(namespaced_name) => namespaced_name.print(p, ctx),
        }
    }
}

impl<'a> Gen for JSXAttribute<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.name.print(p, ctx);
        if let Some(value) = &self.value {
            p.print_equal();
            value.print(p, ctx);
        }
    }
}

impl Gen for JSXEmptyExpression {
    fn gen(&self, _: &mut Codegen, _ctx: Context) {}
}

impl<'a> Gen for JSXExpression<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            match_expression!(Self) => p.print_expression(self.to_expression()),
            Self::EmptyExpression(expr) => expr.print(p, ctx),
        }
    }
}

impl<'a> Gen for JSXExpressionContainer<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_char(b'{');
        self.expression.print(p, ctx);
        p.print_char(b'}');
    }
}

impl<'a> Gen for JSXAttributeValue<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::Fragment(fragment) => fragment.print(p, ctx),
            Self::Element(el) => el.print(p, ctx),
            Self::StringLiteral(lit) => {
                let quote = if lit.value.contains('"') { b'\'' } else { b'"' };
                p.print_char(quote);
                p.print_str(&lit.value);
                p.print_char(quote);
            }
            Self::ExpressionContainer(expr_container) => expr_container.print(p, ctx),
        }
    }
}

impl<'a> Gen for JSXSpreadAttribute<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.print_str("{...");
        self.argument.print_expr(p, Precedence::Comma, Context::empty());
        p.print_char(b'}');
    }
}

impl<'a> Gen for JSXAttributeItem<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::Attribute(attr) => attr.print(p, ctx),
            Self::SpreadAttribute(spread_attr) => spread_attr.print(p, ctx),
        }
    }
}

impl<'a> Gen for JSXOpeningElement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_char(b'<');
        self.name.print(p, ctx);
        for attr in &self.attributes {
            match attr {
                JSXAttributeItem::Attribute(_) => {
                    p.print_hard_space();
                }
                JSXAttributeItem::SpreadAttribute(_) => {
                    p.print_soft_space();
                }
            }
            attr.print(p, ctx);
        }
        if self.self_closing {
            p.print_soft_space();
            p.print_str("/");
        }
        p.print_char(b'>');
    }
}

impl<'a> Gen for JSXClosingElement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str("</");
        self.name.print(p, ctx);
        p.print_char(b'>');
    }
}

impl<'a> Gen for JSXElement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.opening_element.print(p, ctx);
        for child in &self.children {
            child.print(p, ctx);
        }
        if let Some(closing_element) = &self.closing_element {
            closing_element.print(p, ctx);
        }
    }
}

impl Gen for JSXOpeningFragment {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str("<>");
    }
}

impl Gen for JSXClosingFragment {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str("</>");
    }
}

impl<'a> Gen for JSXText<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str(self.value.as_str());
    }
}

impl<'a> Gen for JSXSpreadChild<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.print_str("...");
        p.print_expression(&self.expression);
    }
}

impl<'a> Gen for JSXChild<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::Fragment(fragment) => fragment.print(p, ctx),
            Self::Element(el) => el.print(p, ctx),
            Self::Spread(spread) => p.print_expression(&spread.expression),
            Self::ExpressionContainer(expr_container) => expr_container.print(p, ctx),
            Self::Text(text) => text.print(p, ctx),
        }
    }
}

impl<'a> Gen for JSXFragment<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.opening_fragment.print(p, ctx);
        for child in &self.children {
            child.print(p, ctx);
        }
        self.closing_fragment.print(p, ctx);
    }
}

impl<'a> Gen for StaticBlock<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_str("static");
        p.print_soft_space();
        p.print_curly_braces(self.span, self.body.is_empty(), |p| {
            for stmt in &self.body {
                p.print_semicolon_if_needed();
                stmt.print(p, ctx);
            }
        });
        p.needs_semicolon = false;
    }
}

impl<'a> Gen for MethodDefinition<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        for decorator in &self.decorators {
            decorator.print(p, ctx);
            p.print_hard_space();
        }

        if let Some(accessibility) = &self.accessibility {
            p.print_str(accessibility.as_str());
            p.print_hard_space();
        }
        if self.r#type == MethodDefinitionType::TSAbstractMethodDefinition {
            p.print_str("abstract ");
        }
        if self.r#static {
            p.print_str("static ");
        }

        match &self.kind {
            MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => {}
            MethodDefinitionKind::Get => {
                p.print_str("get ");
            }
            MethodDefinitionKind::Set => {
                p.print_str("set ");
            }
        }

        if self.value.r#async {
            p.print_str("async ");
        }

        if self.value.generator {
            p.print_str("*");
        }

        if self.computed {
            p.print_char(b'[');
        }
        self.key.print(p, ctx);
        if self.computed {
            p.print_char(b']');
        }
        if self.optional {
            p.print_char(b'?');
        }
        if let Some(type_parameters) = self.value.type_parameters.as_ref() {
            type_parameters.print(p, ctx);
        }
        p.print_char(b'(');
        self.value.params.print(p, ctx);
        p.print_char(b')');
        if let Some(return_type) = &self.value.return_type {
            p.print_colon();
            p.print_soft_space();
            return_type.print(p, ctx);
        }
        if let Some(body) = &self.value.body {
            p.print_soft_space();
            body.print(p, ctx);
        } else {
            p.print_semicolon();
        }
    }
}

impl<'a> Gen for PropertyDefinition<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        for decorator in &self.decorators {
            decorator.print(p, ctx);
            p.print_hard_space();
        }
        if self.declare {
            p.print_str("declare ");
        }
        if let Some(accessibility) = self.accessibility {
            p.print_str(accessibility.as_str());
            p.print_hard_space();
        }
        if self.r#type == PropertyDefinitionType::TSAbstractPropertyDefinition {
            p.print_str("abstract ");
        }
        if self.r#static {
            p.print_str("static ");
        }
        if self.readonly {
            p.print_str("readonly ");
        }
        if self.computed {
            p.print_char(b'[');
        }
        self.key.print(p, ctx);
        if self.computed {
            p.print_char(b']');
        }
        if self.optional {
            p.print_str("?");
        }
        if let Some(type_annotation) = &self.type_annotation {
            p.print_colon();
            p.print_soft_space();
            type_annotation.print(p, ctx);
        }
        if let Some(value) = &self.value {
            p.print_soft_space();
            p.print_equal();
            p.print_soft_space();
            value.print_expr(p, Precedence::Comma, Context::empty());
        }
    }
}

impl<'a> Gen for AccessorProperty<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        for decorator in &self.decorators {
            decorator.print(p, ctx);
            p.print_hard_space();
        }
        if self.r#type.is_abstract() {
            p.print_str("abstract ");
        }
        if let Some(accessibility) = self.accessibility {
            p.print_str(accessibility.as_str());
            p.print_hard_space();
        }
        if self.r#static {
            p.print_str("static ");
        }
        p.print_str("accessor");
        if self.computed {
            p.print_soft_space();
            p.print_char(b'[');
        } else {
            p.print_hard_space();
        }
        self.key.print(p, ctx);
        if self.computed {
            p.print_char(b']');
        }
        if let Some(type_annotation) = &self.type_annotation {
            p.print_colon();
            p.print_soft_space();
            type_annotation.print(p, ctx);
        }
        if let Some(value) = &self.value {
            p.print_soft_space();
            p.print_equal();
            p.print_soft_space();
            value.print_expr(p, Precedence::Comma, Context::empty());
        }
    }
}

impl<'a> Gen for PrivateIdentifier<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
        p.add_source_mapping_for_name(self.span, &self.name);
        p.print_char(b'#');
        p.print_str(self.name.as_str());
    }
}

impl<'a> Gen for BindingPattern<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.kind.print(p, ctx);
        if self.optional {
            p.print_str("?");
        }
        if let Some(type_annotation) = &self.type_annotation {
            p.print_colon();
            p.print_soft_space();
            type_annotation.print(p, ctx);
        }
    }
}

impl<'a> Gen for BindingPatternKind<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            BindingPatternKind::BindingIdentifier(ident) => ident.print(p, ctx),
            BindingPatternKind::ObjectPattern(pattern) => pattern.print(p, ctx),
            BindingPatternKind::ArrayPattern(pattern) => pattern.print(p, ctx),
            BindingPatternKind::AssignmentPattern(pattern) => pattern.print(p, ctx),
        }
    }
}

impl<'a> Gen for ObjectPattern<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_char(b'{');
        if !self.is_empty() {
            p.print_soft_space();
        }
        p.print_list(&self.properties, ctx);
        if let Some(rest) = &self.rest {
            if !self.properties.is_empty() {
                p.print_comma();
            }
            rest.print(p, ctx);
        }
        if !self.is_empty() {
            p.print_soft_space();
        }
        p.print_char(b'}');
        p.add_source_mapping(self.span.end);
    }
}

impl<'a> Gen for BindingProperty<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        if self.computed {
            p.print_char(b'[');
        }

        let mut shorthand = false;
        if let PropertyKey::StaticIdentifier(key) = &self.key {
            match &self.value.kind {
                BindingPatternKind::BindingIdentifier(ident)
                    if key.name == p.get_binding_identifier_name(ident) =>
                {
                    shorthand = true;
                }
                BindingPatternKind::AssignmentPattern(assignment_pattern) => {
                    if let BindingPatternKind::BindingIdentifier(ident) =
                        &assignment_pattern.left.kind
                    {
                        if key.name == p.get_binding_identifier_name(ident) {
                            shorthand = true;
                        }
                    }
                }
                _ => {}
            }
        }

        if !shorthand {
            self.key.print(p, ctx);
        }
        if self.computed {
            p.print_char(b']');
        }
        if !shorthand {
            p.print_colon();
            p.print_soft_space();
        }
        self.value.print(p, ctx);
    }
}

impl<'a> Gen for BindingRestElement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_ellipsis();
        self.argument.print(p, ctx);
    }
}

impl<'a> Gen for ArrayPattern<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span.start);
        p.print_char(b'[');
        for (index, item) in self.elements.iter().enumerate() {
            if index != 0 {
                p.print_comma();
                p.print_soft_space();
            }
            if let Some(item) = item {
                item.print(p, ctx);
            }
            if index == self.elements.len() - 1 && (item.is_none() || self.rest.is_some()) {
                p.print_comma();
            }
        }
        if let Some(rest) = &self.rest {
            p.print_soft_space();
            rest.print(p, ctx);
        }
        p.print_char(b']');
        p.add_source_mapping(self.span.end);
    }
}

impl<'a> Gen for AssignmentPattern<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.left.print(p, ctx);
        p.print_soft_space();
        p.print_equal();
        p.print_soft_space();
        self.right.print_expr(p, Precedence::Comma, Context::empty());
    }
}

impl<'a> Gen for Decorator<'a> {
    fn gen(&self, p: &mut Codegen, _ctx: Context) {
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
        p.print_char(b'@');
        let wrap = need_wrap(&self.expression);
        p.wrap(wrap, |p| {
            self.expression.print_expr(p, Precedence::Lowest, Context::empty());
        });
    }
}

impl<'a> Gen for TSClassImplements<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.expression.print(p, ctx);
        if let Some(type_parameters) = self.type_parameters.as_ref() {
            type_parameters.print(p, ctx);
        }
    }
}

impl<'a> Gen for TSTypeParameterDeclaration<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        let is_multi_line = self.params.len() >= 2;
        p.print_char(b'<');
        if is_multi_line {
            p.indent();
        }
        for (index, item) in self.params.iter().enumerate() {
            if index != 0 {
                p.print_comma();
            }
            if is_multi_line {
                p.print_soft_newline();
                p.print_indent();
            } else if index != 0 {
                p.print_soft_space();
            }
            item.print(p, ctx);
        }
        if is_multi_line {
            p.print_soft_newline();
            p.dedent();
            p.print_indent();
        }
        p.print_char(b'>');
    }
}

impl<'a> Gen for TSTypeAnnotation<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.type_annotation.print(p, ctx);
    }
}

impl<'a> Gen for TSType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::TSFunctionType(ty) => ty.print(p, ctx),
            Self::TSConstructorType(ty) => ty.print(p, ctx),
            Self::TSArrayType(ty) => ty.print(p, ctx),
            Self::TSTupleType(ty) => ty.print(p, ctx),
            Self::TSUnionType(ty) => ty.print(p, ctx),
            Self::TSParenthesizedType(ty) => ty.print(p, ctx),
            Self::TSIntersectionType(ty) => ty.print(p, ctx),
            Self::TSConditionalType(ty) => ty.print(p, ctx),
            Self::TSInferType(ty) => ty.print(p, ctx),
            Self::TSIndexedAccessType(ty) => ty.print(p, ctx),
            Self::TSMappedType(ty) => ty.print(p, ctx),
            Self::TSNamedTupleMember(ty) => ty.print(p, ctx),
            Self::TSLiteralType(ty) => ty.literal.print(p, ctx),
            Self::TSImportType(ty) => ty.print(p, ctx),
            Self::TSQualifiedName(ty) => ty.print(p, ctx),
            Self::TSAnyKeyword(_) => p.print_str("any"),
            Self::TSBigIntKeyword(_) => p.print_str("bigint"),
            Self::TSBooleanKeyword(_) => p.print_str("boolean"),
            Self::TSIntrinsicKeyword(_) => p.print_str("intrinsic"),
            Self::TSNeverKeyword(_) => p.print_str("never"),
            Self::TSNullKeyword(_) => p.print_str("null"),
            Self::TSNumberKeyword(_) => p.print_str("number"),
            Self::TSObjectKeyword(_) => p.print_str("object"),
            Self::TSStringKeyword(_) => p.print_str("string"),
            Self::TSSymbolKeyword(_) => p.print_str("symbol"),
            Self::TSThisType(_) => p.print_str("this"),
            Self::TSUndefinedKeyword(_) => p.print_str("undefined"),
            Self::TSUnknownKeyword(_) => p.print_str("unknown"),
            Self::TSVoidKeyword(_) => p.print_str("void"),
            Self::TSTemplateLiteralType(ty) => ty.print(p, ctx),
            Self::TSTypeLiteral(ty) => ty.print(p, ctx),
            Self::TSTypeOperatorType(ty) => ty.print(p, ctx),
            Self::TSTypePredicate(ty) => ty.print(p, ctx),
            Self::TSTypeQuery(ty) => ty.print(p, ctx),
            Self::TSTypeReference(ty) => ty.print(p, ctx),
            Self::JSDocNullableType(ty) => ty.print(p, ctx),
            Self::JSDocNonNullableType(ty) => ty.print(p, ctx),
            Self::JSDocUnknownType(_ty) => p.print_str("unknown"),
        }
    }
}

impl<'a> Gen for TSArrayType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.element_type.print(p, ctx);
        p.print_str("[]");
    }
}

impl<'a> Gen for TSTupleType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_str("[");
        p.print_list(&self.element_types, ctx);
        p.print_str("]");
    }
}

impl<'a> Gen for TSUnionType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if self.types.len() == 1 {
            self.types[0].print(p, ctx);
            return;
        }
        for (index, item) in self.types.iter().enumerate() {
            if index != 0 {
                p.print_soft_space();
                p.print_str("|");
                p.print_soft_space();
            }
            item.print(p, ctx);
        }
    }
}

impl<'a> Gen for TSParenthesizedType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_char(b'(');
        self.type_annotation.print(p, ctx);
        p.print_char(b')');
    }
}

impl<'a> Gen for TSIntersectionType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if self.types.len() == 1 {
            self.types[0].print(p, ctx);
            return;
        }
        for (index, item) in self.types.iter().enumerate() {
            if index != 0 {
                p.print_soft_space();
                p.print_str("&");
                p.print_soft_space();
            }
            item.print(p, ctx);
        }
    }
}

impl<'a> Gen for TSConditionalType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.check_type.print(p, ctx);
        p.print_str(" extends ");
        self.extends_type.print(p, ctx);
        p.print_str(" ? ");
        self.true_type.print(p, ctx);
        p.print_str(" : ");
        self.false_type.print(p, ctx);
    }
}

impl<'a> Gen for TSInferType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_str("infer ");
        self.type_parameter.print(p, ctx);
    }
}

impl<'a> Gen for TSIndexedAccessType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.object_type.print(p, ctx);
        p.print_str("[");
        self.index_type.print(p, ctx);
        p.print_str("]");
    }
}

impl<'a> Gen for TSMappedType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_str("{");
        p.print_soft_space();
        match self.readonly {
            TSMappedTypeModifierOperator::True => {
                p.print_str("readonly ");
            }
            TSMappedTypeModifierOperator::Plus => {
                p.print_str("+readonly ");
            }
            TSMappedTypeModifierOperator::Minus => {
                p.print_str("-readonly ");
            }
            TSMappedTypeModifierOperator::None => {}
        }
        p.print_str("[");
        self.type_parameter.name.print(p, ctx);
        if let Some(constraint) = &self.type_parameter.constraint {
            p.print_str(" in ");
            constraint.print(p, ctx);
        }
        if let Some(default) = &self.type_parameter.default {
            p.print_str(" = ");
            default.print(p, ctx);
        }
        if let Some(name_type) = &self.name_type {
            p.print_str(" as ");
            name_type.print(p, ctx);
        }
        p.print_str("]");
        match self.optional {
            TSMappedTypeModifierOperator::True => {
                p.print_str("?");
            }
            TSMappedTypeModifierOperator::Plus => {
                p.print_str("+?");
            }
            TSMappedTypeModifierOperator::Minus => {
                p.print_str("-?");
            }
            TSMappedTypeModifierOperator::None => {}
        }
        p.print_soft_space();
        if let Some(type_annotation) = &self.type_annotation {
            p.print_str(":");
            p.print_soft_space();
            type_annotation.print(p, ctx);
        }
        p.print_soft_space();
        p.print_str("}");
    }
}

impl<'a> Gen for TSQualifiedName<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.left.print(p, ctx);
        p.print_str(".");
        self.right.print(p, ctx);
    }
}

impl<'a> Gen for TSTypeOperator<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self.operator {
            TSTypeOperatorOperator::Keyof => {
                p.print_str("keyof ");
            }
            TSTypeOperatorOperator::Unique => {
                p.print_str("unique ");
            }
            TSTypeOperatorOperator::Readonly => {
                p.print_str("readonly ");
            }
        }
        self.type_annotation.print(p, ctx);
    }
}

impl<'a> Gen for TSTypePredicate<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if self.asserts {
            p.print_str("asserts ");
        }
        match &self.parameter_name {
            TSTypePredicateName::Identifier(ident) => {
                ident.print(p, ctx);
            }
            TSTypePredicateName::This(_ident) => {
                p.print_str("this");
            }
        }
        if let Some(type_annotation) = &self.type_annotation {
            p.print_str(" is ");
            type_annotation.print(p, ctx);
        }
    }
}

impl<'a> Gen for TSTypeReference<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.type_name.print(p, ctx);
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.print(p, ctx);
        }
    }
}

impl<'a> Gen for JSDocNullableType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if self.postfix {
            self.type_annotation.print(p, ctx);
            p.print_str("?");
        } else {
            p.print_str("?");
            self.type_annotation.print(p, ctx);
        }
    }
}

impl<'a> Gen for JSDocNonNullableType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if self.postfix {
            self.type_annotation.print(p, ctx);
            p.print_str("!");
        } else {
            p.print_str("!");
            self.type_annotation.print(p, ctx);
        }
    }
}

impl<'a> Gen for TSTemplateLiteralType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_str("`");
        for (index, item) in self.quasis.iter().enumerate() {
            if index != 0 {
                if let Some(types) = self.types.get(index - 1) {
                    p.print_str("${");
                    types.print(p, ctx);
                    p.print_str("}");
                }
            }
            p.print_str(item.value.raw.as_str());
        }
        p.print_str("`");
    }
}

impl<'a> Gen for TSTypeLiteral<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        let single_line = self.members.len() <= 1;
        p.print_curly_braces(self.span, single_line, |p| {
            for item in &self.members {
                if single_line {
                    p.print_soft_space();
                } else {
                    p.print_indent();
                }
                item.print(p, ctx);
                if single_line {
                    p.print_soft_space();
                } else {
                    p.print_semicolon();
                    p.print_soft_newline();
                }
            }
        });
    }
}

impl<'a> Gen for TSTypeName<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::IdentifierReference(ident) => {
                ident.print(p, ctx);
            }
            Self::QualifiedName(decl) => {
                decl.left.print(p, ctx);
                p.print_str(".");
                decl.right.print(p, ctx);
            }
        }
    }
}

impl<'a> Gen for TSLiteral<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::BooleanLiteral(decl) => decl.print(p, ctx),
            Self::NullLiteral(decl) => decl.print(p, ctx),
            Self::NumericLiteral(decl) => decl.print(p, ctx),
            Self::BigIntLiteral(decl) => decl.print(p, ctx),
            Self::RegExpLiteral(decl) => decl.print(p, ctx),
            Self::StringLiteral(decl) => decl.print(p, ctx),
            Self::TemplateLiteral(decl) => decl.print(p, ctx),
            Self::UnaryExpression(decl) => decl.print_expr(p, Precedence::Comma, ctx),
        }
    }
}

impl<'a> Gen for TSTypeParameter<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if self.r#const {
            p.print_str("const ");
        }
        self.name.print(p, ctx);
        if let Some(constraint) = &self.constraint {
            p.print_str(" extends ");
            constraint.print(p, ctx);
        }
        if let Some(default) = &self.default {
            p.print_str(" = ");
            default.print(p, ctx);
        }
    }
}

impl<'a> Gen for TSFunctionType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.print(p, ctx);
        }
        p.print_str("(");
        if let Some(this_param) = &self.this_param {
            this_param.print(p, ctx);
            if !self.params.is_empty() || self.params.rest.is_some() {
                p.print_str(",");
            }
            p.print_soft_space();
        }
        self.params.print(p, ctx);
        p.print_str(")");
        p.print_soft_space();
        p.print_str("=>");
        p.print_soft_space();
        self.return_type.print(p, ctx);
    }
}

impl<'a> Gen for TSThisParameter<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_str("this");
        if let Some(type_annotation) = &self.type_annotation {
            p.print_str(": ");
            type_annotation.print(p, ctx);
        }
    }
}

impl<'a> Gen for TSSignature<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::TSIndexSignature(signature) => signature.print(p, ctx),
            Self::TSPropertySignature(signature) => {
                if signature.readonly {
                    p.print_str("readonly ");
                }
                if signature.computed {
                    p.print_char(b'[');
                    signature.key.print(p, ctx);
                    p.print_char(b']');
                } else {
                    match &signature.key {
                        PropertyKey::StaticIdentifier(key) => {
                            key.print(p, ctx);
                        }
                        PropertyKey::PrivateIdentifier(key) => {
                            p.print_str(key.name.as_str());
                        }
                        key @ match_expression!(PropertyKey) => {
                            key.to_expression().print_expr(p, Precedence::Comma, ctx);
                        }
                    }
                }
                if signature.optional {
                    p.print_str("?");
                }
                if let Some(type_annotation) = &signature.type_annotation {
                    p.print_colon();
                    p.print_soft_space();
                    type_annotation.print(p, ctx);
                }
            }
            Self::TSCallSignatureDeclaration(signature) => {
                if let Some(type_parameters) = signature.type_parameters.as_ref() {
                    type_parameters.print(p, ctx);
                }
                p.print_str("(");
                if let Some(this_param) = &signature.this_param {
                    this_param.print(p, ctx);
                    if !signature.params.is_empty() || signature.params.rest.is_some() {
                        p.print_str(",");
                    }
                    p.print_soft_space();
                }
                signature.params.print(p, ctx);
                p.print_str(")");
                if let Some(return_type) = &signature.return_type {
                    p.print_colon();
                    p.print_soft_space();
                    return_type.print(p, ctx);
                }
            }
            Self::TSConstructSignatureDeclaration(signature) => {
                p.print_str("new ");
                if let Some(type_parameters) = signature.type_parameters.as_ref() {
                    type_parameters.print(p, ctx);
                }
                p.print_str("(");
                signature.params.print(p, ctx);
                p.print_str(")");
                if let Some(return_type) = &signature.return_type {
                    p.print_colon();
                    p.print_soft_space();
                    return_type.print(p, ctx);
                }
            }
            Self::TSMethodSignature(signature) => {
                match signature.kind {
                    TSMethodSignatureKind::Method => {}
                    TSMethodSignatureKind::Get => p.print_str("get "),
                    TSMethodSignatureKind::Set => p.print_str("set "),
                }
                if signature.computed {
                    p.print_char(b'[');
                    signature.key.print(p, ctx);
                    p.print_char(b']');
                } else {
                    match &signature.key {
                        PropertyKey::StaticIdentifier(key) => {
                            key.print(p, ctx);
                        }
                        PropertyKey::PrivateIdentifier(key) => {
                            p.print_str(key.name.as_str());
                        }
                        key @ match_expression!(PropertyKey) => {
                            key.to_expression().print_expr(p, Precedence::Comma, ctx);
                        }
                    }
                }
                if signature.optional {
                    p.print_str("?");
                }
                if let Some(type_parameters) = &signature.type_parameters {
                    type_parameters.print(p, ctx);
                }
                p.print_str("(");
                if let Some(this_param) = &signature.this_param {
                    this_param.print(p, ctx);
                    if !signature.params.is_empty() || signature.params.rest.is_some() {
                        p.print_str(",");
                    }
                    p.print_soft_space();
                }
                signature.params.print(p, ctx);
                p.print_str(")");
                if let Some(return_type) = &signature.return_type {
                    p.print_colon();
                    p.print_soft_space();
                    return_type.print(p, ctx);
                }
            }
        }
    }
}

impl<'a> Gen for TSTypeQuery<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_str("typeof ");
        self.expr_name.print(p, ctx);
        if let Some(type_params) = &self.type_parameters {
            type_params.print(p, ctx);
        }
    }
}

impl<'a> Gen for TSTypeQueryExprName<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            match_ts_type_name!(Self) => self.to_ts_type_name().print(p, ctx),
            Self::TSImportType(decl) => decl.print(p, ctx),
        }
    }
}

impl<'a> Gen for TSImportType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if self.is_type_of {
            p.print_str("typeof ");
        }
        p.print_str("import(");
        self.parameter.print(p, ctx);
        if let Some(attributes) = &self.attributes {
            p.print_str(", ");
            attributes.print(p, ctx);
        }
        p.print_str(")");
        if let Some(qualifier) = &self.qualifier {
            p.print_char(b'.');
            qualifier.print(p, ctx);
        }
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.print(p, ctx);
        }
    }
}

impl<'a> Gen for TSImportAttributes<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_char(b'{');
        p.print_soft_space();
        self.attributes_keyword.print(p, ctx);
        p.print_str(":");
        p.print_soft_space();
        p.print_char(b'{');
        p.print_soft_space();
        p.print_list(&self.elements, ctx);
        p.print_soft_space();
        p.print_char(b'}');
        p.print_soft_space();
        p.print_char(b'}');
    }
}

impl<'a> Gen for TSImportAttribute<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.name.print(p, ctx);
        p.print_str(": ");
        self.value.print_expr(p, Precedence::Member, ctx);
    }
}

impl<'a> Gen for TSImportAttributeName<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            TSImportAttributeName::Identifier(ident) => ident.print(p, ctx),
            TSImportAttributeName::StringLiteral(literal) => literal.print(p, ctx),
        }
    }
}

impl<'a> Gen for TSTypeParameterInstantiation<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_str("<");
        p.print_list(&self.params, ctx);
        p.print_str(">");
    }
}

impl<'a> Gen for TSIndexSignature<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if self.readonly {
            p.print_str("readonly ");
        }
        p.print_str("[");
        for (index, parameter) in self.parameters.iter().enumerate() {
            if index != 0 {
                p.print_str(" | ");
            }
            p.print_str(parameter.name.as_str());
            p.print_colon();
            p.print_soft_space();
            parameter.type_annotation.print(p, ctx);
        }
        p.print_str("]");
        p.print_colon();
        p.print_soft_space();
        self.type_annotation.print(p, ctx);
    }
}

impl<'a> Gen for TSTupleElement<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            match_ts_type!(TSTupleElement) => self.to_ts_type().print(p, ctx),
            TSTupleElement::TSOptionalType(ts_type) => {
                ts_type.type_annotation.print(p, ctx);
                p.print_str("?");
            }
            TSTupleElement::TSRestType(ts_type) => {
                p.print_str("...");
                ts_type.type_annotation.print(p, ctx);
            }
        }
    }
}

impl<'a> Gen for TSNamedTupleMember<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.label.print(p, ctx);
        if self.optional {
            p.print_str("?");
        }
        p.print_str(":");
        p.print_soft_space();
        self.element_type.print(p, ctx);
    }
}

impl<'a> Gen for TSModuleDeclaration<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if self.declare {
            p.print_str("declare ");
        }
        p.print_str(self.kind.as_str());
        // If the kind is global, then the id is also `global`, so we don't need to print it
        if !self.kind.is_global() {
            p.print_space_before_identifier();
            self.id.print(p, ctx);
        }

        if let Some(body) = &self.body {
            let mut body = body;
            loop {
                match body {
                    TSModuleDeclarationBody::TSModuleDeclaration(b) => {
                        p.print_char(b'.');
                        b.id.print(p, ctx);
                        if let Some(b) = &b.body {
                            body = b;
                        } else {
                            break;
                        }
                    }
                    TSModuleDeclarationBody::TSModuleBlock(body) => {
                        p.print_soft_space();
                        body.print(p, ctx);
                        break;
                    }
                }
            }
        }
        p.needs_semicolon = false;
    }
}

impl<'a> Gen for TSModuleDeclarationName<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::Identifier(ident) => ident.print(p, ctx),
            Self::StringLiteral(s) => s.print(p, ctx),
        }
    }
}

impl<'a> Gen for TSModuleBlock<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        let is_empty = self.directives.is_empty() && self.body.is_empty();
        p.print_curly_braces(self.span, is_empty, |p| {
            for directive in &self.directives {
                directive.print(p, ctx);
            }
            for stmt in &self.body {
                p.print_semicolon_if_needed();
                stmt.print(p, ctx);
            }
        });
        p.needs_semicolon = false;
    }
}

impl<'a> Gen for TSTypeAliasDeclaration<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if self.declare {
            p.print_str("declare ");
        }
        p.print_str("type");
        p.print_space_before_identifier();
        self.id.print(p, ctx);
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.print(p, ctx);
        }
        p.print_soft_space();
        p.print_str("=");
        p.print_soft_space();
        self.type_annotation.print(p, ctx);
    }
}

impl<'a> Gen for TSInterfaceDeclaration<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_str("interface");
        p.print_hard_space();
        self.id.print(p, ctx);
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.print(p, ctx);
        }
        if let Some(extends) = &self.extends {
            if !extends.is_empty() {
                p.print_str(" extends ");
                p.print_list(extends, ctx);
            }
        }
        p.print_soft_space();
        p.print_curly_braces(self.body.span, self.body.body.is_empty(), |p| {
            for item in &self.body.body {
                p.print_indent();
                p.print_leading_comments(item.span().start);
                item.print(p, ctx);
                p.print_semicolon();
                p.print_soft_newline();
            }
        });
    }
}

impl<'a> Gen for TSInterfaceHeritage<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        self.expression.print_expr(p, Precedence::Call, ctx);
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.print(p, ctx);
        }
    }
}

impl<'a> Gen for TSEnumDeclaration<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_indent();
        if self.declare {
            p.print_str("declare ");
        }
        if self.r#const {
            p.print_str("const ");
        }
        p.print_space_before_identifier();
        p.print_str("enum ");
        self.id.print(p, ctx);
        p.print_space_before_identifier();
        p.print_curly_braces(self.span, self.members.is_empty(), |p| {
            for member in &self.members {
                p.print_indent();
                member.print(p, ctx);
                p.print_comma();
                p.print_soft_newline();
            }
        });
    }
}

impl<'a> Gen for TSEnumMember<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_leading_comments(self.span.start);
        match &self.id {
            TSEnumMemberName::StaticIdentifier(decl) => decl.print(p, ctx),
            TSEnumMemberName::StaticStringLiteral(decl) => decl.print(p, ctx),
            TSEnumMemberName::StaticTemplateLiteral(decl) => decl.print(p, ctx),
            TSEnumMemberName::StaticNumericLiteral(decl) => decl.print(p, ctx),
            decl @ match_expression!(TSEnumMemberName) => {
                p.print_str("[");
                decl.to_expression().print_expr(p, Precedence::Lowest, ctx);
                p.print_str("]");
            }
        }
        if let Some(init) = &self.initializer {
            p.print_soft_space();
            p.print_equal();
            p.print_soft_space();
            init.print_expr(p, Precedence::Lowest, ctx);
        }
    }
}

impl<'a> Gen for TSConstructorType<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        if self.r#abstract {
            p.print_str("abstract ");
        }
        p.print_str("new ");
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.print(p, ctx);
        }
        p.print_str("(");
        self.params.print(p, ctx);
        p.print_str(")");
        p.print_soft_space();
        p.print_str("=>");
        p.print_soft_space();
        self.return_type.print(p, ctx);
    }
}

impl<'a> Gen for TSImportEqualsDeclaration<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        p.print_str("import ");
        self.id.print(p, ctx);
        p.print_str(" = ");
        self.module_reference.print(p, ctx);
    }
}

impl<'a> Gen for TSModuleReference<'a> {
    fn gen(&self, p: &mut Codegen, ctx: Context) {
        match self {
            Self::ExternalModuleReference(decl) => {
                p.print_str("require(");
                decl.expression.print(p, ctx);
                p.print_str(")");
            }
            match_ts_type_name!(Self) => self.to_ts_type_name().print(p, ctx),
        }
    }
}
