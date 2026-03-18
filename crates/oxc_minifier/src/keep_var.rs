use oxc_allocator::Box as ArenaBox;
use oxc_ast::{AstBuilder, NONE, ast::*};
use oxc_ast::ast::StatementKind;
use oxc_ast_visit::Visit;
use oxc_ecmascript::BoundNames;
use oxc_span::{Atom, SPAN, Span};
use oxc_syntax::symbol::SymbolId;

pub struct KeepVar<'a> {
    ast: AstBuilder<'a>,
    vars: std::vec::Vec<(Atom<'a>, Span, Option<SymbolId>)>,
    all_hoisted: bool,
}

impl<'a> Visit<'a> for KeepVar<'a> {
    fn visit_statement(&mut self, it: &Statement<'a>) {
        // Only visit blocks where vars could be hoisted
        match it.kind() {
            StatementKind::BlockStatement(it) => self.visit_block_statement(it),
            StatementKind::BreakStatement(it) => self.visit_break_statement(it),
            StatementKind::ContinueStatement(it) => self.visit_continue_statement(it),
            StatementKind::DoWhileStatement(it) => self.visit_do_while_statement(it),
            StatementKind::ForInStatement(it) => self.visit_for_in_statement(it),
            StatementKind::ForOfStatement(it) => self.visit_for_of_statement(it),
            StatementKind::ForStatement(it) => self.visit_for_statement(it),
            StatementKind::IfStatement(it) => self.visit_if_statement(it),
            StatementKind::LabeledStatement(it) => self.visit_labeled_statement(it),
            StatementKind::SwitchStatement(it) => self.visit_switch_statement(it),
            StatementKind::TryStatement(it) => self.visit_try_statement(it),
            StatementKind::WhileStatement(it) => self.visit_while_statement(it),
            StatementKind::WithStatement(it) => self.visit_with_statement(it),
            StatementKind::VariableDeclaration(decl) => self.visit_variable_declaration(decl),
            _ => {}
        }
    }

    fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'a>) {
        if it.kind.is_var() {
            it.bound_names(&mut |ident| {
                self.vars.push((ident.name.into(), ident.span, ident.symbol_id.get()));
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

    pub fn get_variable_declaration(self) -> Option<ArenaBox<'a, VariableDeclaration<'a>>> {
        if self.vars.is_empty() {
            return None;
        }

        let kind = VariableDeclarationKind::Var;
        let decls = self.ast.vec_from_iter(self.vars.into_iter().map(|(name, span, symbol_id)| {
            let id = symbol_id.map_or_else(
                || self.ast.binding_pattern_binding_identifier(span, name),
                |symbol_id| {
                    self.ast
                        .binding_pattern_binding_identifier_with_symbol_id(span, name, symbol_id)
                },
            );
            self.ast.variable_declarator(span, kind, id, NONE, None, false)
        }));

        Some(self.ast.alloc_variable_declaration(SPAN, kind, decls, false))
    }

    pub fn get_variable_declaration_statement(self) -> Option<Statement<'a>> {
        self.get_variable_declaration().map(Statement::variable_declaration)
    }
}
