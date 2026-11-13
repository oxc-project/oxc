use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::{ast::*, builder::GetAstBuilder};
use oxc_ast_visit::VisitJs;
use oxc_ecmascript::BoundNames;
use oxc_span::{SPAN, Span};
use oxc_str::Ident;
use oxc_syntax::{
    node::NodeId,
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};

pub struct KeepVar<'a> {
    vars: Vec<(Ident<'a>, Span, Option<SymbolId>)>,
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
        Self { vars: std::vec![] }
    }

    /// Declare the collected `var`s in the current scope.
    pub fn get_variable_declaration(
        self,
        ctx: &mut crate::TraverseCtx<'a>,
    ) -> Option<ArenaBox<'a, VariableDeclaration<'a>>> {
        let scope_id = ctx.current_scope_id();
        self.get_variable_declaration_in(scope_id, ctx)
    }

    /// Declare the collected `var`s in `scope_id`, where the declaration will be placed.
    fn get_variable_declaration_in(
        mut self,
        scope_id: ScopeId,
        ctx: &mut crate::TraverseCtx<'a>,
    ) -> Option<ArenaBox<'a, VariableDeclaration<'a>>> {
        if self.vars.is_empty() {
            return None;
        }

        for (name, span, symbol_id) in &mut self.vars {
            if let Some(symbol_id) = symbol_id {
                *symbol_id = Self::declared_symbol(*name, *span, *symbol_id, scope_id, ctx);
                // Tell the pass flush this declaration may still be live.
                if !ctx.scoping().symbol_redeclarations(*symbol_id).is_empty() {
                    ctx.state.pass_changes.reemitted_declarations.insert((*symbol_id, *span));
                }
            }
        }

        let builder = ctx.builder();

        let kind = VariableDeclarationKind::Var;
        let decls = ArenaVec::from_iter_in(
            self.vars.into_iter().map(|(name, span, symbol_id)| {
                let id = symbol_id.map_or_else(
                    || BindingPattern::new_binding_identifier(span, name, builder),
                    |symbol_id| {
                        BindingPattern::new_binding_identifier_with_symbol_id(
                            span, name, symbol_id, builder,
                        )
                    },
                );
                VariableDeclarator::new(span, id, None, None, false, builder)
            }),
            builder,
        );

        Some(VariableDeclaration::boxed(SPAN, kind, decls, false, builder))
    }

    /// The symbol for a regenerated `var` identifier placed in `scope_id`: the binding semantic
    /// analysis would give it there, so the output minifies to a fixed point. A `var` that
    /// redeclared a catch parameter now redeclares whichever catch parameter it resolves to,
    /// or else only declares the enclosing var binding. An unreferenced catch parameter can
    /// be removed later, so it is not linked.
    fn declared_symbol(
        name: Ident<'a>,
        span: Span,
        symbol_id: SymbolId,
        scope_id: ScopeId,
        ctx: &mut crate::TraverseCtx<'a>,
    ) -> SymbolId {
        let scoping = ctx.scoping();
        let Some(var_binding) = scoping.catch_var_binding(symbol_id) else { return symbol_id };
        match scoping.find_binding(scope_id, name) {
            Some(catch)
                if scoping.symbol_flags(catch).is_catch_variable()
                    && !scoping.symbol_is_unused(catch) =>
            {
                // The regenerated `var` redeclares the surviving catch parameter, which links
                // that catch to `var_binding` (see `Scoping::catch_var_binding`).
                if scoping.catch_var_binding(catch).is_none() {
                    ctx.scoping_mut().add_symbol_redeclaration(
                        catch,
                        SymbolFlags::FunctionScopedVariable,
                        NodeId::DUMMY,
                        span,
                    );
                }
                debug_assert_eq!(ctx.scoping().catch_var_binding(catch), Some(var_binding));
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

    /// Like [`Self::get_variable_declaration_statement`], for a declaration placed in
    /// `scope_id` instead of the current scope (e.g. inside a catch body).
    pub fn get_variable_declaration_statement_in(
        self,
        scope_id: ScopeId,
        ctx: &mut crate::TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        self.get_variable_declaration_in(scope_id, ctx).map(Statement::VariableDeclaration)
    }
}
