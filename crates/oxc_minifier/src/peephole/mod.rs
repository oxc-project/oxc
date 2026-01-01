mod convert_to_dotted_properties;
mod fold_constants;
mod inline;
mod minimize_conditional_expression;
mod minimize_conditions;
mod minimize_expression_in_boolean_context;
mod minimize_for_statement;
mod minimize_if_statement;
mod minimize_logical_expression;
mod minimize_not_expression;
mod minimize_statements;
mod normalize;
mod remove_dead_code;
mod remove_unused_declaration;
mod remove_unused_expression;
mod remove_unused_private_members;
mod replace_known_methods;
mod substitute_alternate_syntax;

use oxc_ast_visit::Visit;
use oxc_semantic::ReferenceId;
use oxc_syntax::scope::ScopeId;
use rustc_hash::FxHashSet;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::{ReusableTraverseCtx, Traverse, traverse_mut_with_ctx};

use crate::{
    ctx::{Ctx, TraverseCtx},
    state::MinifierState,
};

pub use self::normalize::{Normalize, NormalizeOptions};

pub struct PeepholeOptimizations {
    max_iterations: Option<u8>,
    /// Walk the ast in a fixed point loop until no changes are made.
    /// `prev_function_changed`, `functions_changed` and `current_function` track changes
    /// in top level and each function. No minification code are run if the function is not changed
    /// in the previous walk.
    iteration: u8,
    changed: bool,
}

impl<'a> PeepholeOptimizations {
    pub fn new(max_iterations: Option<u8>) -> Self {
        Self { max_iterations, iteration: 0, changed: false }
    }

    fn run_once(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut ReusableTraverseCtx<'a, MinifierState<'a>>,
    ) {
        traverse_mut_with_ctx(self, program, ctx);
    }

    pub fn run_in_loop(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut ReusableTraverseCtx<'a, MinifierState<'a>>,
    ) -> u8 {
        loop {
            self.changed = false;
            self.run_once(program, ctx);
            if !self.changed {
                break;
            }
            if let Some(max_iterations) = self.max_iterations {
                if self.iteration >= max_iterations {
                    break;
                }
            } else if self.iteration > 10 {
                debug_assert!(false, "Ran loop more than 10 times.");
                break;
            }
            self.iteration += 1;
        }
        self.iteration
    }

    pub fn commutative_pair<'x, A, F, G, RetF: 'x, RetG: 'x>(
        pair: (&'x A, &'x A),
        check_a: F,
        check_b: G,
    ) -> Option<(RetF, RetG)>
    where
        F: Fn(&'x A) -> Option<RetF>,
        G: Fn(&'x A) -> Option<RetG>,
    {
        match check_a(pair.0) {
            Some(a) => {
                if let Some(b) = check_b(pair.1) {
                    return Some((a, b));
                }
            }
            _ => {
                if let Some(a) = check_a(pair.1)
                    && let Some(b) = check_b(pair.0)
                {
                    return Some((a, b));
                }
            }
        }
        None
    }

    /// Checks if a member expression's base object may be mutated.
    ///
    /// This is used to prevent incorrect transformations like:
    /// `x.y || (x = {}, x.y = 3)` → `x.y ||= (x = {}, 3)`
    ///
    /// The `||=` operator evaluates `x.y` (capturing `x`) before the RHS reassigns `x`,
    /// which would change the semantics.
    pub fn member_object_may_be_mutated(
        assignment_target: &AssignmentTarget<'a>,
        ctx: &Ctx<'a, '_>,
    ) -> bool {
        let object = match assignment_target {
            AssignmentTarget::ComputedMemberExpression(member_expr) => &member_expr.object,
            AssignmentTarget::PrivateFieldExpression(member_expr) => &member_expr.object,
            AssignmentTarget::StaticMemberExpression(member_expr) => &member_expr.object,
            _ => return false,
        };

        Self::is_expression_that_reference_may_change(object, ctx)
    }

    /// Checks if an expression's reference may change due to mutation.
    ///
    /// Returns `true` if the expression references a symbol that may be mutated,
    /// or if the expression is not a simple identifier/this reference.
    pub fn is_expression_that_reference_may_change(
        expr: &Expression<'a>,
        ctx: &Ctx<'a, '_>,
    ) -> bool {
        match expr {
            Expression::Identifier(id) => {
                if let Some(symbol_id) = ctx.scoping().get_reference(id.reference_id()).symbol_id()
                {
                    ctx.scoping().symbol_is_mutated(symbol_id)
                } else {
                    true
                }
            }
            Expression::ThisExpression(_) => false,
            _ => true,
        }
    }
}

impl<'a> Traverse<'a, MinifierState<'a>> for PeepholeOptimizations {
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        ctx.state.symbol_values.clear();
        ctx.state.changed = false;

        if self.iteration == 0 {
            ctx.state.reference_owning_symbol.clear();
            ctx.state.named_function_expr_symbols.clear();
            ctx.state.unused_symbol_cache.borrow_mut().clear();
            let scoping = ctx.scoping.scoping();
            let root_scope_id = scoping.root_scope_id();

            let mut collector = ReferenceCollector {
                refs: FxHashSet::default(),
                reference_owning_symbol: &mut ctx.state.reference_owning_symbol,
                named_function_expr_symbols: &mut ctx.state.named_function_expr_symbols,
                scoping,
                current_scope_id: root_scope_id,
                scope_stack: vec![root_scope_id],
                owning_symbol_stack: vec![],
            };
            collector.visit_program(program);
        }
    }

    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.changed = ctx.state.changed;

        // Visit the AST to:
        // 1. Collect current references (to diff and delete stale ones)
        // 2. Rebuild maps for circular dependency DCE in next iteration
        let refs_before =
            ctx.scoping().resolved_references().flatten().copied().collect::<FxHashSet<_>>();

        ctx.state.reference_owning_symbol.clear();
        ctx.state.named_function_expr_symbols.clear();
        ctx.state.unused_symbol_cache.borrow_mut().clear();
        let scoping = ctx.scoping.scoping();
        let root_scope_id = scoping.root_scope_id();

        let mut collector = ReferenceCollector {
            refs: FxHashSet::default(),
            reference_owning_symbol: &mut ctx.state.reference_owning_symbol,
            named_function_expr_symbols: &mut ctx.state.named_function_expr_symbols,
            scoping,
            current_scope_id: root_scope_id,
            scope_stack: vec![root_scope_id],
            owning_symbol_stack: vec![],
        };
        collector.visit_program(program);

        // Delete references that no longer exist in the AST
        for reference_id_to_remove in refs_before.difference(&collector.refs) {
            ctx.scoping_mut().delete_reference(*reference_id_to_remove);
        }

        debug_assert!(ctx.state.class_symbols_stack.is_exhausted());
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        let ctx = &mut Ctx::new(ctx);
        Self::minimize_statements(stmts, ctx);
    }

    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = &mut Ctx::new(ctx);
        Self::keep_track_of_pure_functions(stmt, ctx);
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = &mut Ctx::new(ctx);
        match stmt {
            Statement::BlockStatement(_) => Self::try_optimize_block(stmt, ctx),
            Statement::IfStatement(s) => {
                Self::minimize_expression_in_boolean_context(&mut s.test, ctx);
                Self::try_fold_if(stmt, ctx);
                if let Statement::IfStatement(if_stmt) = stmt
                    && let Some(folded_stmt) = Self::try_minimize_if(if_stmt, ctx)
                {
                    *stmt = folded_stmt;
                    ctx.state.changed = true;
                }
            }
            Statement::WhileStatement(s) => {
                Self::minimize_expression_in_boolean_context(&mut s.test, ctx);
            }
            Statement::ForStatement(s) => {
                if let Some(test) = &mut s.test {
                    Self::minimize_expression_in_boolean_context(test, ctx);
                }
                Self::try_fold_for(stmt, ctx);
            }
            Statement::DoWhileStatement(s) => {
                Self::minimize_expression_in_boolean_context(&mut s.test, ctx);
            }
            Statement::TryStatement(_) => Self::try_fold_try(stmt, ctx),
            Statement::LabeledStatement(_) => Self::try_fold_labeled(stmt, ctx),
            Statement::FunctionDeclaration(_) => {
                Self::remove_unused_function_declaration(stmt, ctx);
            }
            Statement::ClassDeclaration(_) => Self::remove_unused_class_declaration(stmt, ctx),
            Statement::ImportDeclaration(_) => Self::remove_unused_import_specifiers(stmt, ctx),
            _ => {}
        }
        Self::try_fold_expression_stmt(stmt, ctx);
    }

    fn exit_for_statement(&mut self, stmt: &mut ForStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = &mut Ctx::new(ctx);
        Self::substitute_for_statement(stmt, ctx);
        Self::minimize_for_statement(stmt, ctx);
    }

    fn exit_return_statement(&mut self, stmt: &mut ReturnStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = &mut Ctx::new(ctx);
        Self::substitute_return_statement(stmt, ctx);
    }

    fn exit_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ctx = &mut Ctx::new(ctx);
        Self::substitute_variable_declaration(decl, ctx);
    }

    fn exit_variable_declarator(
        &mut self,
        decl: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ctx = &mut Ctx::new(ctx);
        Self::init_symbol_value(decl, ctx);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = &mut Ctx::new(ctx);
        match expr {
            Expression::TemplateLiteral(t) => {
                Self::inline_template_literal(t, ctx);
                Self::substitute_template_literal(expr, ctx);
            }
            Expression::ObjectExpression(e) => Self::fold_object_exp(e, ctx),
            Expression::BinaryExpression(e) => {
                Self::substitute_swap_binary_expressions(e);
                Self::fold_binary_expr(expr, ctx);
                Self::fold_binary_typeof_comparison(expr, ctx);
                Self::minimize_loose_boolean(expr, ctx);
                Self::minimize_binary(expr, ctx);
                Self::substitute_loose_equals_undefined(expr, ctx);
                Self::substitute_typeof_undefined(expr, ctx);
                Self::substitute_rotate_binary_expression(expr, ctx);
            }
            Expression::UnaryExpression(_) => {
                Self::fold_unary_expr(expr, ctx);
                Self::minimize_unary(expr, ctx);
                Self::substitute_unary_plus(expr, ctx);
            }
            Expression::StaticMemberExpression(_) => {
                Self::fold_static_member_expr(expr, ctx);
                Self::replace_known_property_access(expr, ctx);
            }
            Expression::ComputedMemberExpression(_) => {
                Self::fold_computed_member_expr(expr, ctx);
                Self::replace_known_property_access(expr, ctx);
            }
            Expression::LogicalExpression(_) => {
                Self::fold_logical_expr(expr, ctx);
                Self::minimize_logical_expression(expr, ctx);
                Self::substitute_is_object_and_not_null(expr, ctx);
                Self::substitute_rotate_logical_expression(expr, ctx);
            }
            Expression::ChainExpression(_) => {
                Self::fold_chain_expr(expr, ctx);
                Self::substitute_chain_expression(expr, ctx);
            }
            Expression::CallExpression(_) => {
                Self::fold_call_expression(expr, ctx);
                Self::substitute_iife_call(expr, ctx);
                Self::remove_dead_code_call_expression(expr, ctx);
                Self::replace_concat_chain(expr, ctx);
                Self::replace_known_global_methods(expr, ctx);
                Self::substitute_simple_function_call(expr, ctx);
                Self::substitute_object_or_array_constructor(expr, ctx);
            }
            Expression::ConditionalExpression(logical_expr) => {
                Self::minimize_expression_in_boolean_context(&mut logical_expr.test, ctx);
                if let Some(changed) = Self::minimize_conditional_expression(logical_expr, ctx) {
                    *expr = changed;
                    ctx.state.changed = true;
                }
                Self::try_fold_conditional_expression(expr, ctx);
            }
            Expression::AssignmentExpression(e) => {
                Self::minimize_normal_assignment_to_combined_logical_assignment(e, ctx);
                Self::minimize_normal_assignment_to_combined_assignment(e, ctx);
                Self::minimize_assignment_to_update_expression(expr, ctx);
                Self::remove_unused_assignment_expr(expr, ctx);
            }
            Expression::SequenceExpression(_) => Self::remove_sequence_expression(expr, ctx),
            Expression::ArrowFunctionExpression(e) => Self::substitute_arrow_expression(e, ctx),
            Expression::FunctionExpression(e) => Self::try_remove_name_from_functions(e, ctx),
            Expression::ClassExpression(e) => Self::try_remove_name_from_classes(e, ctx),
            Expression::NewExpression(e) => {
                Self::substitute_typed_array_constructor(e, ctx);
                Self::substitute_global_new_expression(expr, ctx);
                Self::substitute_object_or_array_constructor(expr, ctx);
            }
            Expression::BooleanLiteral(_) => Self::substitute_boolean(expr, ctx),
            Expression::ArrayExpression(_) => Self::substitute_array_expression(expr, ctx),
            Expression::Identifier(_) => Self::inline_identifier_reference(expr, ctx),
            _ => {}
        }
    }

    fn exit_unary_expression(&mut self, expr: &mut UnaryExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if expr.operator.is_not() {
            let ctx = &mut Ctx::new(ctx);
            Self::minimize_expression_in_boolean_context(&mut expr.argument, ctx);
        }
    }

    fn exit_call_expression(&mut self, e: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = &mut Ctx::new(ctx);
        Self::substitute_call_expression(e, ctx);
        Self::remove_empty_spread_arguments(&mut e.arguments);
    }

    fn exit_new_expression(&mut self, e: &mut NewExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = &mut Ctx::new(ctx);
        Self::substitute_new_expression(e, ctx);
        Self::remove_empty_spread_arguments(&mut e.arguments);
    }

    fn exit_object_property(&mut self, prop: &mut ObjectProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = &mut Ctx::new(ctx);
        Self::substitute_object_property(prop, ctx);
    }

    fn exit_assignment_target_property(
        &mut self,
        node: &mut AssignmentTargetProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ctx = &mut Ctx::new(ctx);
        Self::substitute_assignment_target_property(node, ctx);
    }

    fn exit_assignment_target_property_property(
        &mut self,
        prop: &mut AssignmentTargetPropertyProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ctx = &mut Ctx::new(ctx);
        Self::substitute_assignment_target_property_property(prop, ctx);
    }

    fn exit_binding_property(&mut self, prop: &mut BindingProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = &mut Ctx::new(ctx);
        Self::substitute_binding_property(prop, ctx);
    }

    fn exit_method_definition(
        &mut self,
        prop: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ctx = &mut Ctx::new(ctx);
        Self::substitute_method_definition(prop, ctx);
    }

    fn exit_property_definition(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ctx = &mut Ctx::new(ctx);
        Self::substitute_property_definition(prop, ctx);
    }

    fn exit_accessor_property(
        &mut self,
        prop: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ctx = &mut Ctx::new(ctx);
        Self::substitute_accessor_property(prop, ctx);
    }

    fn exit_member_expression(
        &mut self,
        expr: &mut MemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ctx = Ctx::new(ctx);
        Self::convert_to_dotted_properties(expr, &ctx);
    }

    fn enter_class_body(&mut self, _body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        ctx.state.class_symbols_stack.push_class_scope();
    }

    fn exit_class_body(&mut self, body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = &mut Ctx::new(ctx);
        Self::remove_dead_code_exit_class_body(body, ctx);
        Self::remove_unused_private_members(body, ctx);
        ctx.state.class_symbols_stack.pop_class_scope(Self::get_declared_private_symbols(body));
    }

    fn exit_catch_clause(&mut self, catch: &mut CatchClause<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = Ctx::new(ctx);
        Self::substitute_catch_clause(catch, &ctx);
    }

    fn exit_private_field_expression(
        &mut self,
        node: &mut PrivateFieldExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        ctx.state.class_symbols_stack.push_private_member_to_current_class(node.field.name);
    }

    fn exit_private_in_expression(
        &mut self,
        node: &mut PrivateInExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        ctx.state.class_symbols_stack.push_private_member_to_current_class(node.left.name);
    }
}

pub struct DeadCodeElimination {
    max_iterations: Option<u8>,
    iteration: u8,
    changed: bool,
}

impl<'a> DeadCodeElimination {
    pub fn new(max_iterations: Option<u8>) -> Self {
        Self { max_iterations, iteration: 0, changed: false }
    }

    fn run_once(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut ReusableTraverseCtx<'a, MinifierState<'a>>,
    ) {
        traverse_mut_with_ctx(self, program, ctx);
    }

    pub fn run_in_loop(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut ReusableTraverseCtx<'a, MinifierState<'a>>,
    ) -> u8 {
        loop {
            self.changed = false;
            self.run_once(program, ctx);
            if !self.changed {
                break;
            }
            if let Some(max_iterations) = self.max_iterations {
                if self.iteration >= max_iterations {
                    break;
                }
            } else if self.iteration > 10 {
                debug_assert!(false, "Ran loop more than 10 times.");
                break;
            }
            self.iteration += 1;
        }
        self.iteration
    }
}

impl<'a> Traverse<'a, MinifierState<'a>> for DeadCodeElimination {
    fn enter_program(&mut self, _program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        ctx.state.symbol_values.clear();
        ctx.state.changed = false;
    }

    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.changed = ctx.state.changed;

        // Visit the AST to:
        // 1. Collect current references (to diff and delete stale ones)
        // 2. Rebuild maps for circular dependency DCE in next iteration
        let refs_before =
            ctx.scoping().resolved_references().flatten().copied().collect::<FxHashSet<_>>();

        ctx.state.reference_owning_symbol.clear();
        ctx.state.named_function_expr_symbols.clear();
        ctx.state.unused_symbol_cache.borrow_mut().clear();
        let scoping = ctx.scoping.scoping();
        let root_scope_id = scoping.root_scope_id();

        let mut collector = ReferenceCollector {
            refs: FxHashSet::default(),
            reference_owning_symbol: &mut ctx.state.reference_owning_symbol,
            named_function_expr_symbols: &mut ctx.state.named_function_expr_symbols,
            scoping,
            current_scope_id: root_scope_id,
            scope_stack: vec![root_scope_id],
            owning_symbol_stack: vec![],
        };
        collector.visit_program(program);

        // Delete references that no longer exist in the AST
        for reference_id_to_remove in refs_before.difference(&collector.refs) {
            ctx.scoping_mut().delete_reference(*reference_id_to_remove);
        }
    }

    fn exit_variable_declarator(
        &mut self,
        decl: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ctx = &mut Ctx::new(ctx);
        PeepholeOptimizations::init_symbol_value(decl, ctx);
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = &mut Ctx::new(ctx);
        match stmt {
            Statement::BlockStatement(_) => PeepholeOptimizations::try_optimize_block(stmt, ctx),
            Statement::IfStatement(_) => PeepholeOptimizations::try_fold_if(stmt, ctx),
            Statement::ForStatement(_) => PeepholeOptimizations::try_fold_for(stmt, ctx),
            Statement::TryStatement(_) => PeepholeOptimizations::try_fold_try(stmt, ctx),
            Statement::LabeledStatement(_) => PeepholeOptimizations::try_fold_labeled(stmt, ctx),
            Statement::FunctionDeclaration(_) => {
                PeepholeOptimizations::remove_unused_function_declaration(stmt, ctx);
            }
            Statement::ClassDeclaration(_) => {
                PeepholeOptimizations::remove_unused_class_declaration(stmt, ctx);
            }
            Statement::ExpressionStatement(_) => {
                PeepholeOptimizations::try_fold_expression_stmt(stmt, ctx);
            }
            Statement::ImportDeclaration(_) => {
                PeepholeOptimizations::remove_unused_import_specifiers(stmt, ctx);
            }
            _ => {}
        }
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        let ctx = &mut Ctx::new(ctx);
        PeepholeOptimizations::minimize_statements(stmts, ctx);
    }

    fn exit_expression(&mut self, e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = &mut Ctx::new(ctx);
        match e {
            Expression::TemplateLiteral(t) => {
                PeepholeOptimizations::inline_template_literal(t, ctx);
            }
            Expression::ObjectExpression(e) => PeepholeOptimizations::fold_object_exp(e, ctx),
            Expression::BinaryExpression(_) => {
                PeepholeOptimizations::fold_binary_expr(e, ctx);
                PeepholeOptimizations::fold_binary_typeof_comparison(e, ctx);
            }
            Expression::UnaryExpression(_) => PeepholeOptimizations::fold_unary_expr(e, ctx),
            Expression::StaticMemberExpression(_) => {
                PeepholeOptimizations::fold_static_member_expr(e, ctx);
            }
            Expression::ComputedMemberExpression(_) => {
                PeepholeOptimizations::fold_computed_member_expr(e, ctx);
            }
            Expression::LogicalExpression(_) => PeepholeOptimizations::fold_logical_expr(e, ctx),
            Expression::ChainExpression(_) => PeepholeOptimizations::fold_chain_expr(e, ctx),
            Expression::CallExpression(_) => {
                PeepholeOptimizations::fold_call_expression(e, ctx);
                PeepholeOptimizations::substitute_iife_call(e, ctx);
                PeepholeOptimizations::remove_dead_code_call_expression(e, ctx);
            }
            Expression::ConditionalExpression(_) => {
                PeepholeOptimizations::try_fold_conditional_expression(e, ctx);
            }
            Expression::SequenceExpression(_) => {
                PeepholeOptimizations::remove_sequence_expression(e, ctx);
            }
            Expression::AssignmentExpression(_) => {
                PeepholeOptimizations::remove_unused_assignment_expr(e, ctx);
            }
            _ => {}
        }
    }
}

/// Collects references and their scope information.
/// Used in `exit_program` to:
/// 1. Track which references still exist in the AST (for cleanup)
/// 2. Build maps for circular dependency DCE in the next iteration
struct ReferenceCollector<'b> {
    /// Set of reference IDs that still exist in the AST
    refs: FxHashSet<ReferenceId>,
    /// Maps each reference directly to its innermost containing function/class symbol
    reference_owning_symbol:
        &'b mut rustc_hash::FxHashMap<ReferenceId, Option<oxc_syntax::symbol::SymbolId>>,
    /// Set of symbols that are named function expression inner bindings
    named_function_expr_symbols: &'b mut rustc_hash::FxHashSet<oxc_syntax::symbol::SymbolId>,
    /// Reference to scoping for looking up symbol scope IDs
    scoping: &'b oxc_semantic::Scoping,
    current_scope_id: ScopeId,
    scope_stack: std::vec::Vec<ScopeId>,
    /// Stack of owning symbols. Top of stack is the innermost function/class we're inside.
    /// Empty stack means we're at the top level.
    owning_symbol_stack: std::vec::Vec<oxc_syntax::symbol::SymbolId>,
}

impl ReferenceCollector<'_> {
    fn enter_scope(&mut self, scope_id: ScopeId) {
        self.scope_stack.push(scope_id);
        self.current_scope_id = scope_id;
    }

    fn leave_scope(&mut self) {
        self.scope_stack.pop();
        if let Some(&scope_id) = self.scope_stack.last() {
            self.current_scope_id = scope_id;
        }
    }
}

impl<'a> Visit<'a> for ReferenceCollector<'_> {
    fn visit_program(&mut self, program: &Program<'a>) {
        self.enter_scope(program.scope_id());
        oxc_ast_visit::walk::walk_program(self, program);
        self.leave_scope();
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        self.enter_scope(func.scope_id());

        // Track which function symbol we're inside for reference ownership
        let has_symbol = if let Some(id) = &func.id {
            let symbol_id = id.symbol_id();

            // Check if this is a named function expression (symbol declared in own scope)
            // For `var outer = function inner() { ... }`, inner's symbol_scope_id == func.scope_id()
            let symbol_scope = self.scoping.symbol_scope_id(symbol_id);
            if symbol_scope == self.current_scope_id {
                self.named_function_expr_symbols.insert(symbol_id);
            }

            self.owning_symbol_stack.push(symbol_id);
            true
        } else {
            false
        };

        oxc_ast_visit::walk::walk_function(self, func, flags);

        if has_symbol {
            self.owning_symbol_stack.pop();
        }
        self.leave_scope();
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.enter_scope(arrow.scope_id());
        oxc_ast_visit::walk::walk_arrow_function_expression(self, arrow);
        self.leave_scope();
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        self.enter_scope(class.scope_id());

        // Track which class symbol we're inside for reference ownership
        let has_symbol = if let Some(id) = &class.id {
            let symbol_id = id.symbol_id();
            self.owning_symbol_stack.push(symbol_id);
            true
        } else {
            false
        };

        oxc_ast_visit::walk::walk_class(self, class);

        if has_symbol {
            self.owning_symbol_stack.pop();
        }
        self.leave_scope();
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.enter_scope(block.scope_id());
        oxc_ast_visit::walk::walk_block_statement(self, block);
        self.leave_scope();
    }

    fn visit_for_statement(&mut self, stmt: &ForStatement<'a>) {
        self.enter_scope(stmt.scope_id());
        oxc_ast_visit::walk::walk_for_statement(self, stmt);
        self.leave_scope();
    }

    fn visit_for_in_statement(&mut self, stmt: &ForInStatement<'a>) {
        self.enter_scope(stmt.scope_id());
        oxc_ast_visit::walk::walk_for_in_statement(self, stmt);
        self.leave_scope();
    }

    fn visit_for_of_statement(&mut self, stmt: &ForOfStatement<'a>) {
        self.enter_scope(stmt.scope_id());
        oxc_ast_visit::walk::walk_for_of_statement(self, stmt);
        self.leave_scope();
    }

    fn visit_switch_statement(&mut self, stmt: &SwitchStatement<'a>) {
        self.enter_scope(stmt.scope_id());
        oxc_ast_visit::walk::walk_switch_statement(self, stmt);
        self.leave_scope();
    }

    fn visit_catch_clause(&mut self, clause: &CatchClause<'a>) {
        self.enter_scope(clause.scope_id());
        oxc_ast_visit::walk::walk_catch_clause(self, clause);
        self.leave_scope();
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        self.enter_scope(block.scope_id());
        oxc_ast_visit::walk::walk_static_block(self, block);
        self.leave_scope();
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        let reference_id = ident.reference_id();
        // Track that this reference exists in the AST
        self.refs.insert(reference_id);
        // Track the innermost owning function/class symbol (None if at top level)
        self.reference_owning_symbol.insert(reference_id, self.owning_symbol_stack.last().copied());
    }
}
