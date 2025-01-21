mod collapse_variable_declarations;
mod convert_to_dotted_properties;
mod fold_constants;
mod minimize_conditions;
mod minimize_exit_points;
mod normalize;
mod remove_dead_code;
mod replace_known_methods;
mod statement_fusion;
mod substitute_alternate_syntax;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_syntax::{es_target::ESTarget, scope::ScopeId};
use oxc_traverse::{traverse_mut_with_ctx, ReusableTraverseCtx, Traverse, TraverseCtx};

pub use normalize::{Normalize, NormalizeOptions};
use rustc_hash::FxHashSet;

pub struct PeepholeOptimizations {
    target: ESTarget,

    /// `in_fixed_loop`: Do not compress syntaxes that are hard to analyze inside the fixed loop.
    /// Opposite of `late` in Closure Compiler.
    in_fixed_loop: bool,

    /// Walk the ast in a fixed point loop until no changes are made.
    /// `prev_function_changed`, `functions_changed` and `current_function` track changes
    /// in top level and each function. No minification code are run if the function is not changed
    /// in the previous walk.
    iteration: u8,
    prev_functions_changed: FxHashSet<ScopeId>,
    functions_changed: FxHashSet<ScopeId>,
    /// Track the current function as a stack.
    current_function_stack:
        std::vec::Vec<(ScopeId, /* prev changed */ bool, /* current changed */ bool)>,
}

impl<'a> PeepholeOptimizations {
    pub fn new(target: ESTarget, in_fixed_loop: bool) -> Self {
        Self {
            target,
            in_fixed_loop,
            iteration: 0,
            prev_functions_changed: FxHashSet::default(),
            functions_changed: FxHashSet::default(),
            current_function_stack: std::vec::Vec::new(),
        }
    }

    pub fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }

    pub fn run_in_loop(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        loop {
            self.build(program, ctx);
            if self.functions_changed.is_empty() {
                break;
            }
            self.prev_functions_changed.clear();
            std::mem::swap(&mut self.prev_functions_changed, &mut self.functions_changed);
            if self.iteration > 10 {
                debug_assert!(false, "Ran loop more than 10 times.");
                break;
            }
            self.iteration += 1;
        }
    }

    fn mark_current_function_as_changed(&mut self) {
        if let Some((_scope_id, _prev_changed, current_changed)) =
            self.current_function_stack.last_mut()
        {
            *current_changed = true;
        }
    }

    pub fn is_current_function_changed(&self) -> bool {
        if let Some((_, _, current_changed)) = self.current_function_stack.last() {
            return *current_changed;
        }
        false
    }

    fn is_prev_function_changed(&self) -> bool {
        if !self.in_fixed_loop || self.iteration == 0 {
            return true;
        }
        if let Some((_, prev_changed, _)) = self.current_function_stack.last() {
            return *prev_changed;
        }
        false
    }

    fn enter_program_or_function(&mut self, scope_id: ScopeId) {
        self.current_function_stack.push((
            scope_id,
            self.prev_functions_changed.contains(&scope_id),
            false,
        ));
    }

    fn exit_program_or_function(&mut self) {
        if let Some((scope_id, _, changed)) = self.current_function_stack.pop() {
            if changed {
                self.functions_changed.insert(scope_id);
            }
        }
    }
}

impl<'a> Traverse<'a> for PeepholeOptimizations {
    fn enter_program(&mut self, program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.enter_program_or_function(program.scope_id());
    }

    fn exit_program(&mut self, _program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.exit_program_or_function();
    }

    fn enter_function(&mut self, func: &mut Function<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.enter_program_or_function(func.scope_id());
    }

    fn exit_function(&mut self, _: &mut Function<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.exit_program_or_function();
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.statement_fusion_exit_statements(stmts, ctx);
        self.collapse_variable_declarations(stmts, ctx);
        self.minimize_conditions_exit_statements(stmts, ctx);
        self.remove_dead_code_exit_statements(stmts, ctx);
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.minimize_conditions_exit_statement(stmt, ctx);
        self.remove_dead_code_exit_statement(stmt, ctx);
    }

    fn exit_return_statement(&mut self, stmt: &mut ReturnStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.substitute_return_statement(stmt, ctx);
    }

    fn exit_function_body(&mut self, body: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.minimize_exit_points(body, ctx);
    }

    fn exit_class_body(&mut self, body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.remove_dead_code_exit_class_body(body, ctx);
    }

    fn exit_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.substitute_variable_declaration(decl, ctx);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.fold_constants_exit_expression(expr, ctx);
        self.minimize_conditions_exit_expression(expr, ctx);
        self.remove_dead_code_exit_expression(expr, ctx);
        self.replace_known_methods_exit_expression(expr, ctx);
        self.substitute_exit_expression(expr, ctx);
    }

    fn exit_call_expression(&mut self, expr: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.substitute_call_expression(expr, ctx);
    }

    fn exit_member_expression(
        &mut self,
        expr: &mut MemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.convert_to_dotted_properties(expr, ctx);
    }

    fn exit_object_property(&mut self, prop: &mut ObjectProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.substitute_object_property(prop, ctx);
    }

    fn exit_assignment_target_property_property(
        &mut self,
        prop: &mut AssignmentTargetPropertyProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.substitute_assignment_target_property_property(prop, ctx);
    }

    fn exit_binding_property(&mut self, prop: &mut BindingProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.substitute_binding_property(prop, ctx);
    }

    fn exit_method_definition(
        &mut self,
        prop: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.substitute_method_definition(prop, ctx);
    }

    fn exit_property_definition(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.substitute_property_definition(prop, ctx);
    }

    fn exit_accessor_property(
        &mut self,
        prop: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.substitute_accessor_property(prop, ctx);
    }

    fn exit_catch_clause(&mut self, catch: &mut CatchClause<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        self.substitute_catch_clause(catch, ctx);
    }
}

pub struct DeadCodeElimination {
    inner: PeepholeOptimizations,
}

impl<'a> DeadCodeElimination {
    pub fn new() -> Self {
        Self { inner: PeepholeOptimizations::new(ESTarget::ESNext, false) }
    }

    pub fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for DeadCodeElimination {
    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.inner.remove_dead_code_exit_statement(stmt, ctx);
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.inner.remove_dead_code_exit_statements(stmts, ctx);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.inner.fold_constants_exit_expression(expr, ctx);
        self.inner.remove_dead_code_exit_expression(expr, ctx);
    }
}
