use oxc_allocator::{Allocator, Vec};
use oxc_ast::{
    ast::*, syntax_directed_operations::BoundNames, visit::walk_mut, AstBuilder, Visit, VisitMut,
};
use oxc_span::{Atom, Span, SPAN};
use oxc_syntax::scope::ScopeFlags;

use crate::{compressor::ast_util::get_boolean_value, folder::Folder};

/// Remove Dead Code from the AST.
///
/// Terser option: `dead_code: true`.
///
/// See `KeepVar` at the end of this file for `var` hoisting logic.
pub struct RemoveDeadCode<'a> {
    ast: AstBuilder<'a>,
    folder: Folder<'a>,
}

impl<'a> VisitMut<'a> for RemoveDeadCode<'a> {
    fn visit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        self.dead_code_elimintation(stmts);
        walk_mut::walk_statements(self, stmts);
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        self.fold_conditional_expression(expr);
        self.fold_logical_expression(expr);
        walk_mut::walk_expression(self, expr);
    }
}

impl<'a> RemoveDeadCode<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        let ast = AstBuilder::new(allocator);
        Self { ast, folder: Folder::new(ast) }
    }

    pub fn build(&mut self, program: &mut Program<'a>) {
        self.visit_program(program);
    }

    /// Removes dead code thats comes after `return` statements after inlining `if` statements
    fn dead_code_elimintation(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        // Fold if statements
        for stmt in stmts.iter_mut() {
            if self.fold_if_statement(stmt) {}
        }

        // Remove code after `return` and `throw` statements
        let mut index = None;
        'outer: for (i, stmt) in stmts.iter().enumerate() {
            if matches!(stmt, Statement::ReturnStatement(_) | Statement::ThrowStatement(_)) {
                index.replace(i);
                break;
            }
            // Double check block statements folded by if statements above
            if let Statement::BlockStatement(block_stmt) = stmt {
                for stmt in &block_stmt.body {
                    if matches!(stmt, Statement::ReturnStatement(_) | Statement::ThrowStatement(_))
                    {
                        index.replace(i);
                        break 'outer;
                    }
                }
            }
        }

        let Some(index) = index else { return };

        let mut keep_var = KeepVar::new(self.ast);

        for stmt in stmts.iter().skip(index) {
            keep_var.visit_statement(stmt);
        }

        stmts.drain(index + 1..);
        if let Some(stmt) = keep_var.get_variable_declaration_statement() {
            stmts.push(stmt);
        }
    }

    #[must_use]
    fn fold_if_statement(&mut self, stmt: &mut Statement<'a>) -> bool {
        let Statement::IfStatement(if_stmt) = stmt else { return false };
        match self.fold_expression_and_get_boolean_value(&mut if_stmt.test) {
            Some(true) => {
                *stmt = self.ast.move_statement(&mut if_stmt.consequent);
                true
            }
            Some(false) => {
                *stmt = if let Some(alternate) = &mut if_stmt.alternate {
                    self.ast.move_statement(alternate)
                } else {
                    // Keep hoisted `vars` from the consequent block.
                    let mut keep_var = KeepVar::new(self.ast);
                    keep_var.visit_statement(&if_stmt.consequent);
                    if let Some(stmt) = keep_var.get_variable_declaration_statement() {
                        stmt
                    } else {
                        self.ast.statement_empty(SPAN)
                    }
                };
                true
            }
            _ => false,
        }
    }

    fn fold_expression_and_get_boolean_value(&mut self, expr: &mut Expression<'a>) -> Option<bool> {
        self.folder.fold_expression(expr);
        get_boolean_value(expr)
    }

    fn fold_conditional_expression(&mut self, expr: &mut Expression<'a>) {
        let Expression::ConditionalExpression(conditional_expr) = expr else {
            return;
        };
        match self.fold_expression_and_get_boolean_value(&mut conditional_expr.test) {
            Some(true) => {
                *expr = self.ast.move_expression(&mut conditional_expr.consequent);
            }
            Some(false) => {
                *expr = self.ast.move_expression(&mut conditional_expr.alternate);
            }
            _ => {}
        }
    }

    fn fold_logical_expression(&mut self, expr: &mut Expression<'a>) {
        let Expression::LogicalExpression(logical_expr) = expr else {
            return;
        };
        if let Some(e) = self.folder.try_fold_logical_expression(logical_expr) {
            *expr = e;
        }
    }
}

struct KeepVar<'a> {
    ast: AstBuilder<'a>,
    vars: std::vec::Vec<(Atom<'a>, Span)>,
}

impl<'a> Visit<'a> for KeepVar<'a> {
    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        if decl.kind.is_var() {
            decl.id.bound_names(&mut |ident| {
                self.vars.push((ident.name.clone(), ident.span));
            });
        }
    }

    fn visit_function(&mut self, _it: &Function<'a>, _flags: ScopeFlags) {
        /* skip functions */
    }

    fn visit_class(&mut self, _it: &Class<'a>) {
        /* skip classes */
    }
}

impl<'a> KeepVar<'a> {
    fn new(ast: AstBuilder<'a>) -> Self {
        Self { ast, vars: std::vec![] }
    }

    fn get_variable_declaration_statement(self) -> Option<Statement<'a>> {
        if self.vars.is_empty() {
            return None;
        }

        let kind = VariableDeclarationKind::Var;
        let decls = self.ast.vec_from_iter(self.vars.into_iter().map(|(name, span)| {
            let binding_kind = self.ast.binding_pattern_kind_binding_identifier(span, name);
            let id =
                self.ast.binding_pattern::<Option<TSTypeAnnotation>>(binding_kind, None, false);
            self.ast.variable_declarator(span, kind, id, None, false)
        }));

        let decl = self.ast.variable_declaration(SPAN, kind, decls, false);
        let stmt = self.ast.statement_declaration(self.ast.declaration_from_variable(decl));
        Some(stmt)
    }
}
