//! Remove `var` declarations that became redundant when the mangler let variables with disjoint
//! lifetimes share a name.
//!
//! A redundant symbol shares its name with a parameter or `var` of the same function that keeps
//! its declaration, so its own declarations can become assignments:
//!
//! * `var a = 1, b = 2;` → `a = 1, b = 2;` when every declarator is redundant.
//! * `var a, b = 1;` → `var b = 1;` when only `a` is redundant.
//! * `for (var a in o)` → `for (a in o)`.

use oxc_allocator::{Allocator, ArenaVec, BitSet};
use oxc_ast::{ast::*, builder::AstBuilder};
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_mangler::MergedVars;
use oxc_semantic::{Scoping, Semantic};
use oxc_span::GetSpan;
use oxc_syntax::{
    node::NodeId,
    reference::{Reference, ReferenceFlags},
    scope::{ScopeFlags, ScopeId},
};

/// The redundant symbols of a program, and the AST nodes that contain their declarations.
pub struct RedundantVars<'a> {
    symbols: BitSet<'a>,
    nodes: BitSet<'a>,
}

impl<'a> RedundantVars<'a> {
    /// Collect from the [`Semantic`] the mangler ran on. Returns `None` if no var is redundant.
    pub fn collect(
        allocator: &'a Allocator,
        semantic: &Semantic<'_>,
        merged_vars: &MergedVars,
    ) -> Option<Self> {
        if merged_vars.is_empty() {
            return None;
        }
        let scoping = semantic.scoping();
        let ast_nodes = semantic.nodes();
        let mut symbols = BitSet::new_in(scoping.symbols_len(), allocator);
        let mut nodes = BitSet::new_in(ast_nodes.len(), allocator);
        for &symbol_id in merged_vars.redundant_symbols() {
            symbols.set_bit(symbol_id.index());
            let redeclarations = scoping.symbol_redeclarations(symbol_id);
            let declarations = std::iter::once(scoping.symbol_declaration(symbol_id))
                .chain(redeclarations.iter().map(|redeclaration| redeclaration.declaration));
            for mut node_id in declarations {
                while !nodes.has_bit(node_id.index()) {
                    nodes.set_bit(node_id.index());
                    let parent_id = ast_nodes.parent_id(node_id);
                    if parent_id == node_id {
                        break;
                    }
                    node_id = parent_id;
                }
            }
        }
        Some(Self { symbols, nodes })
    }

    /// Remove the redundant declarations from `program`, which must be unchanged since
    /// [`RedundantVars::collect`].
    pub fn remove(
        self,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
        scoping: &mut Scoping,
    ) {
        let root_scope_id = scoping.root_scope_id();
        let mut rewriter = Rewriter {
            ast: AstBuilder::new(allocator),
            scoping,
            symbols: self.symbols,
            nodes: self.nodes,
            scope_stack: ArenaVec::new_in(&allocator),
            root_scope_id,
        };
        rewriter.visit_program(program);
    }
}

enum Rewrite<'a> {
    Keep,
    Remove,
    Replace(Expression<'a>),
}

struct Rewriter<'a, 's> {
    ast: AstBuilder<'a>,
    scoping: &'s mut Scoping,
    symbols: BitSet<'a>,
    /// Declarations of redundant symbols, and their ancestors.
    nodes: BitSet<'a>,
    scope_stack: ArenaVec<'a, ScopeId>,
    root_scope_id: ScopeId,
}

impl<'a> Rewriter<'a, '_> {
    fn current_scope(&self) -> ScopeId {
        self.scope_stack.last().copied().unwrap_or(self.root_scope_id)
    }

    fn contains_redundant(&self, node_id: NodeId) -> bool {
        self.nodes.has_bit(node_id.index())
    }

    fn is_redundant(&self, declarator: &VariableDeclarator<'a>) -> bool {
        matches!(
            &declarator.id,
            BindingPattern::BindingIdentifier(id)
                if id.symbol_id.get().is_some_and(|symbol_id| self.symbols.has_bit(symbol_id.index()))
        )
    }

    fn assignment_target(
        &mut self,
        id: &BindingIdentifier<'a>,
        scope_id: ScopeId,
    ) -> AssignmentTarget<'a> {
        let symbol_id = id.symbol_id();
        let reference = Reference::new_with_symbol_id(
            NodeId::DUMMY,
            symbol_id,
            scope_id,
            ReferenceFlags::Write,
        );
        let reference_id = self.scoping.create_reference(reference);
        self.scoping.add_resolved_reference(symbol_id, reference_id);
        AssignmentTarget::new_assignment_target_identifier_with_reference_id(
            id.span,
            id.name,
            reference_id,
            &self.ast,
        )
    }

    fn rewrite_declaration(
        &mut self,
        declaration: &mut VariableDeclaration<'a>,
        scope_id: ScopeId,
    ) -> Rewrite<'a> {
        if !declaration.kind.is_var() {
            return Rewrite::Keep;
        }
        let mut redundant = 0;
        let mut initialized = 0;
        for declarator in &declaration.declarations {
            if self.is_redundant(declarator) {
                redundant += 1;
                initialized += usize::from(declarator.init.is_some());
            }
        }
        if redundant == 0 {
            return Rewrite::Keep;
        }
        if redundant < declaration.declarations.len() {
            declaration
                .declarations
                .retain(|declarator| declarator.init.is_some() || !self.is_redundant(declarator));
            return Rewrite::Keep;
        }
        if initialized == 0 {
            return Rewrite::Remove;
        }
        let mut expressions = ArenaVec::with_capacity_in(initialized, &self.ast);
        for declarator in &mut declaration.declarations {
            let Some(init) = declarator.init.take() else { continue };
            let BindingPattern::BindingIdentifier(id) = &declarator.id else { continue };
            let target = self.assignment_target(id, scope_id);
            expressions.push(Expression::new_assignment_expression(
                declarator.span,
                AssignmentOperator::Assign,
                target,
                init,
                &self.ast,
            ));
        }
        if expressions.len() == 1 {
            return Rewrite::Replace(expressions.pop().unwrap());
        }
        Rewrite::Replace(Expression::new_sequence_expression(
            declaration.span,
            expressions,
            &self.ast,
        ))
    }

    /// Returns `false` if the statement should be removed.
    fn rewrite_statement(&mut self, statement: &mut Statement<'a>) -> bool {
        if !self.contains_redundant(statement.node_id()) {
            return true;
        }
        let Statement::VariableDeclaration(declaration) = statement else {
            walk_mut::walk_statement(self, statement);
            return true;
        };
        walk_mut::walk_variable_declaration(self, declaration);
        let span = declaration.span;
        match self.rewrite_declaration(declaration, self.current_scope()) {
            Rewrite::Keep => true,
            Rewrite::Remove => false,
            Rewrite::Replace(expression) => {
                *statement = Statement::new_expression_statement(span, expression, &self.ast);
                true
            }
        }
    }

    fn rewrite_for_left(&mut self, left: &mut ForStatementLeft<'a>, scope_id: ScopeId) {
        let ForStatementLeft::VariableDeclaration(declaration) = left else { return };
        if !declaration.kind.is_var() {
            return;
        }
        let [declarator] = declaration.declarations.as_slice() else { return };
        if declarator.init.is_some() || !self.is_redundant(declarator) {
            return;
        }
        let BindingPattern::BindingIdentifier(id) = &declarator.id else { return };
        let target = self.assignment_target(id, scope_id);
        *left = ForStatementLeft::from(target);
    }
}

impl<'a> VisitMut<'a> for Rewriter<'a, '_> {
    fn enter_scope(&mut self, _flags: ScopeFlags, scope_id: &std::cell::Cell<Option<ScopeId>>) {
        let scope_id = scope_id.get().unwrap_or_else(|| self.current_scope());
        self.scope_stack.push(scope_id);
    }

    fn leave_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn visit_function(&mut self, it: &mut Function<'a>, flags: ScopeFlags) {
        if self.contains_redundant(it.node_id()) {
            walk_mut::walk_function(self, it, flags);
        }
    }

    fn visit_arrow_function_expression(&mut self, it: &mut ArrowFunctionExpression<'a>) {
        if self.contains_redundant(it.node_id()) {
            walk_mut::walk_arrow_function_expression(self, it);
        }
    }

    fn visit_static_block(&mut self, it: &mut StaticBlock<'a>) {
        if self.contains_redundant(it.node_id()) {
            walk_mut::walk_static_block(self, it);
        }
    }

    fn visit_expression(&mut self, it: &mut Expression<'a>) {
        if self.contains_redundant(it.node_id()) {
            walk_mut::walk_expression(self, it);
        }
    }

    fn visit_statements(&mut self, it: &mut ArenaVec<'a, Statement<'a>>) {
        it.retain_mut(|statement| self.rewrite_statement(statement));
    }

    fn visit_statement(&mut self, it: &mut Statement<'a>) {
        if !self.rewrite_statement(it) {
            *it = Statement::new_empty_statement(it.span(), &self.ast);
        }
    }

    fn visit_if_statement(&mut self, it: &mut IfStatement<'a>) {
        walk_mut::walk_if_statement(self, it);
        if matches!(it.alternate, Some(Statement::EmptyStatement(_))) {
            it.alternate = None;
        }
    }

    fn visit_for_statement(&mut self, it: &mut ForStatement<'a>) {
        walk_mut::walk_for_statement(self, it);
        let scope_id = it.scope_id();
        if let Some(ForStatementInit::VariableDeclaration(declaration)) = &mut it.init {
            match self.rewrite_declaration(declaration, scope_id) {
                Rewrite::Keep => {}
                Rewrite::Remove => it.init = None,
                Rewrite::Replace(expression) => it.init = Some(ForStatementInit::from(expression)),
            }
        }
    }

    fn visit_for_in_statement(&mut self, it: &mut ForInStatement<'a>) {
        walk_mut::walk_for_in_statement(self, it);
        let scope_id = it.scope_id();
        self.rewrite_for_left(&mut it.left, scope_id);
    }

    fn visit_for_of_statement(&mut self, it: &mut ForOfStatement<'a>) {
        walk_mut::walk_for_of_statement(self, it);
        let scope_id = it.scope_id();
        self.rewrite_for_left(&mut it.left, scope_id);
    }
}
