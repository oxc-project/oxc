use oxc_ast::{ast::*, AstBuilder, Visit, NONE};
use oxc_ecmascript::BoundNames;
use oxc_span::{Atom, Span, SPAN};

pub struct KeepVar<'a> {
    ast: AstBuilder<'a>,
    vars: std::vec::Vec<(Atom<'a>, Span)>,
    all_hoisted: bool,
}

impl<'a> Visit<'a> for KeepVar<'a> {
    fn visit_statement(&mut self, it: &Statement<'a>) {
        // Only visit blocks where vars could be hoisted
        match it {
            Statement::BlockStatement(it) => self.visit_block_statement(it),
            Statement::BreakStatement(it) => self.visit_break_statement(it),
            Statement::ContinueStatement(it) => self.visit_continue_statement(it),
            // Statement::DebuggerStatement(it) => self.visit_debugger_statement(it),
            Statement::DoWhileStatement(it) => self.visit_do_while_statement(it),
            // Statement::EmptyStatement(it) => self.visit_empty_statement(it),
            // Statement::ExpressionStatement(it) => self.visit_expression_statement(it),
            Statement::ForInStatement(it) => self.visit_for_in_statement(it),
            Statement::ForOfStatement(it) => self.visit_for_of_statement(it),
            Statement::ForStatement(it) => self.visit_for_statement(it),
            Statement::IfStatement(it) => self.visit_if_statement(it),
            Statement::LabeledStatement(it) => self.visit_labeled_statement(it),
            // Statement::ReturnStatement(it) => self.visit_return_statement(it),
            Statement::SwitchStatement(it) => self.visit_switch_statement(it),
            // Statement::ThrowStatement(it) => self.visit_throw_statement(it),
            Statement::TryStatement(it) => self.visit_try_statement(it),
            Statement::WhileStatement(it) => self.visit_while_statement(it),
            Statement::WithStatement(it) => self.visit_with_statement(it),
            // match_declaration!(Statement) => visitor.visit_declaration(it.to_declaration()),
            // match_module_declaration!(Statement) => {
            // visitor.visit_module_declaration(it.to_module_declaration())
            // }
            Statement::VariableDeclaration(decl) => self.visit_variable_declaration(decl),
            _ => {}
        }
    }

    fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'a>) {
        if it.kind.is_var() {
            it.bound_names(&mut |ident| {
                self.vars.push((ident.name.clone(), ident.span));
            });
            if it.has_init() {
                self.all_hoisted = false;
            }
        }
    }
}

impl<'a> KeepVar<'a> {
    pub fn new(ast: AstBuilder<'a>) -> Self {
        Self { ast, vars: std::vec![], all_hoisted: true }
    }

    pub fn all_hoisted(&self) -> bool {
        self.all_hoisted
    }

    pub fn get_variable_declaration(self) -> Option<VariableDeclaration<'a>> {
        if self.vars.is_empty() {
            return None;
        }

        let kind = VariableDeclarationKind::Var;
        let decls = self.ast.vec_from_iter(self.vars.into_iter().map(|(name, span)| {
            let binding_kind = self.ast.binding_pattern_kind_binding_identifier(span, name);
            let id = self.ast.binding_pattern(binding_kind, NONE, false);
            self.ast.variable_declarator(span, kind, id, None, false)
        }));

        let decl = self.ast.variable_declaration(SPAN, kind, decls, false);
        Some(decl)
    }

    pub fn get_variable_declaration_statement(self) -> Option<Statement<'a>> {
        let stmt = self.ast.statement_declaration(
            self.ast.declaration_from_variable(self.get_variable_declaration()?),
        );
        Some(stmt)
    }
}
