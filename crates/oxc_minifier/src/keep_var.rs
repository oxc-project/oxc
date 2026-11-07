use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::{ast::*, builder::GetAstBuilder};
use oxc_ast_visit::VisitJs;
use oxc_ecmascript::BoundNames;
use oxc_span::{SPAN, Span};
use oxc_str::Ident;
use oxc_syntax::symbol::SymbolId;

pub struct KeepVar<'a> {
    vars: Vec<(Ident<'a>, Span, Option<SymbolId>)>,
    /// Simple catch parameters whose catch clauses are inside the visited statement.
    removed_catch_params: Vec<SymbolId>,
}

impl<'a> VisitJs<'a> for KeepVar<'a> {
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

    fn visit_catch_clause(&mut self, it: &CatchClause<'a>) {
        if let Some(param) = &it.param
            && let BindingPattern::BindingIdentifier(id) = &param.pattern
            && let Some(symbol_id) = id.symbol_id.get()
        {
            self.removed_catch_params.push(symbol_id);
        }
        self.visit_block_statement(&it.body);
    }

    fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'a>) {
        if it.kind.is_var() {
            it.bound_names(&mut |ident| {
                self.vars.push((ident.name, ident.span, ident.symbol_id.get()));
            });
        }
    }
}

impl<'a> KeepVar<'a> {
    pub fn new() -> Self {
        Self { vars: std::vec![], removed_catch_params: std::vec![] }
    }

    pub fn get_variable_declaration(
        self,
        ctx: &mut crate::TraverseCtx<'a>,
    ) -> Option<ArenaBox<'a, VariableDeclaration<'a>>> {
        if self.vars.is_empty() {
            return None;
        }

        let kind = VariableDeclarationKind::Var;
        let mut decls = ArenaVec::with_capacity_in(self.vars.len(), ctx.builder());
        for &(name, span, symbol_id) in &self.vars {
            let symbol_id = symbol_id.map(|symbol_id| self.declared_symbol(name, symbol_id, ctx));
            let builder = ctx.builder();
            let id = symbol_id.map_or_else(
                || BindingPattern::new_binding_identifier(span, name, builder),
                |symbol_id| {
                    BindingPattern::new_binding_identifier_with_symbol_id(
                        span, name, symbol_id, builder,
                    )
                },
            );
            decls.push(VariableDeclarator::new(span, id, None, None, false, builder));
        }

        Some(VariableDeclaration::boxed(SPAN, kind, decls, false, ctx.builder()))
    }

    /// The symbol for a regenerated `var` identifier at the replacement site.
    /// Keep the nearest surviving catch parameter when later code can access it, including
    /// when an inner catch was removed. Unreferenced catch parameters can be removed too;
    /// in that case, only the enclosing var binding needs to be preserved.
    fn declared_symbol(
        &self,
        name: Ident<'a>,
        symbol_id: SymbolId,
        ctx: &mut crate::TraverseCtx<'a>,
    ) -> SymbolId {
        let scoping = ctx.scoping();
        let Some(var_binding) = scoping.catch_var_binding(symbol_id) else { return symbol_id };
        let var_scope_id = scoping.symbol_scope_id(var_binding);
        let mut scope_id = Some(scoping.symbol_scope_id(symbol_id));
        let mut enclosing_catch = None;
        while let Some(id) = scope_id
            && id != var_scope_id
        {
            if let Some(binding) = scoping.get_binding(id, name)
                && scoping.symbol_flags(binding).is_catch_variable()
                && !self.removed_catch_params.contains(&binding)
            {
                enclosing_catch = Some(binding);
                break;
            }
            scope_id = scoping.scope_parent_id(id);
        }
        match enclosing_catch {
            Some(catch) if !scoping.symbol_is_unused(catch) => {
                if catch != symbol_id {
                    ctx.scoping_mut().add_catch_var_binding(catch, var_binding);
                }
                catch
            }
            _ => var_binding,
        }
    }

    pub fn get_variable_declaration_statement(
        self,
        ctx: &mut crate::TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        self.get_variable_declaration(ctx).map(Statement::VariableDeclaration)
    }
}
