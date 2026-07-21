mod convert_to_dotted_properties;
mod find_nested_break;
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

use oxc_syntax::{scope::ScopeId, symbol::SymbolId};

use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::IsLiteralValue;

use crate::{Traverse, TraverseCtx};

pub use self::normalize::{Normalize, NormalizeOptions};

/// Stateless peephole optimizer. Configuration and the per-pass
/// `PassChanges` accumulator live on `MinifierState`.
pub struct PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
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

    /// A body-level statement is "declarative" if executing it cannot run user
    /// code that observes a subsequent hoisted `var x = <literal>;` as
    /// `undefined`. Module loaders (`import`, `export * from`, `export … from`)
    /// can evaluate foreign modules but only observe our bindings on an actual
    /// cycle — handled at program scope by starting the root prelude unsafe when
    /// the module has loaders (see `enter_program`).
    /// Type-only declarations (`type`, `interface`) are erased and never run.
    fn is_declarative_body_statement(stmt: &Statement<'a>) -> bool {
        match stmt {
            Statement::EmptyStatement(_)
            | Statement::ImportDeclaration(_)
            | Statement::ExportAllDeclaration(_) => true,
            // `export { foo }`, `export { foo } from './x'`, `export type T = …` —
            // no executable code at the statement itself. The cyclic-eval hazard
            // from a `from` source is gated separately at program scope (see
            // `enter_program`).
            Statement::ExportNamedDeclaration(e) => {
                e.declaration.as_ref().is_none_or(Self::is_declarative_declaration)
            }
            // `export default function() {}` is hoisted; `export default <expr>`
            // or `export default class C extends … {}` runs user code.
            Statement::ExportDefaultDeclaration(e) => {
                matches!(&e.declaration, ExportDefaultDeclarationKind::FunctionDeclaration(_))
            }
            // Bare declarations route through the shared classifier; anything else
            // (blocks, expressions, control flow) can run user code.
            _ => stmt.as_declaration().is_some_and(Self::is_declarative_declaration),
        }
    }

    /// A `Declaration` runs no user code at evaluation: function/type/interface
    /// declarations are inert, and a `var`/`let`/`const` is declarative only when
    /// every declarator is a simple binding with a literal (or no) initializer.
    /// Classes, enums, and TS modules run user code, so they are not declarative.
    fn is_declarative_declaration(decl: &Declaration<'a>) -> bool {
        match decl {
            Declaration::FunctionDeclaration(_)
            | Declaration::TSTypeAliasDeclaration(_)
            | Declaration::TSInterfaceDeclaration(_) => true,
            Declaration::VariableDeclaration(decl) => {
                Self::is_declarative_variable_declaration(decl)
            }
            _ => false,
        }
    }

    /// A `VariableDeclaration` is declarative when every declarator is a simple
    /// `BindingIdentifier` (no destructuring / defaults / computed keys, all of
    /// which can run user code) with either no initializer or a primitive
    /// literal initializer.
    fn is_declarative_variable_declaration(decl: &VariableDeclaration<'a>) -> bool {
        decl.declarations.iter().all(Self::is_declarative_variable_declarator)
    }

    /// Note: only AST `Literal`s qualify. Constant-but-non-literal initializers
    /// (`-1`, `void 0`, `1 + 2`) run no user code either, but conservatively end
    /// the prelude here — a missed optimization, never a correctness risk.
    fn is_declarative_variable_declarator(decl: &VariableDeclarator<'a>) -> bool {
        matches!(decl.id, BindingPattern::BindingIdentifier(_))
            && decl.init.as_ref().is_none_or(Expression::is_literal)
    }

    /// Mark the current function/program body as no longer in its declarative
    /// prelude. No-op if the flag is already set, or if `current_scope_id` is
    /// some inner scope (a block/for/etc.) — those don't end the prelude.
    fn mark_current_body_unsafe(ctx: &mut TraverseCtx<'a>) {
        let &(body_scope, body_unsafe) = ctx.state.body_unsafe_stack.last();
        if !body_unsafe && body_scope == ctx.current_scope_id() {
            ctx.state.body_unsafe_stack.last_mut().1 = true;
        }
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
        ctx: &TraverseCtx<'a>,
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
    /// is externally mutable through an ESM import or Script global, or is not a
    /// simple identifier/this reference.
    pub fn is_expression_that_reference_may_change(
        expr: &Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        match expr {
            Expression::Identifier(id) => {
                let symbol_id = ctx.scoping().get_reference(id.reference_id()).symbol_id();
                symbol_id.is_none_or(|symbol_id| Self::symbol_value_may_change(symbol_id, ctx))
            }
            Expression::ThisExpression(_) => false,
            _ => true,
        }
    }

    /// Whether reading a resolved symbol again could produce a different value.
    /// Imported bindings and Script-root globals can change externally.
    ///
    /// Uses the O(1) cached reference counts from `SymbolValue` when available,
    /// falling back to the O(num_refs) scan in `Scoping::symbol_is_mutated` for
    /// symbols without cached values.
    ///
    /// Only variable declarators have cached values (populated during
    /// `exit_variable_declarator` → `init_symbol_value`); function declarations
    /// and other binding kinds still take the fallback path.
    fn symbol_value_may_change(symbol_id: SymbolId, ctx: &TraverseCtx<'a>) -> bool {
        let scoping = ctx.scoping();
        if scoping.symbol_flags(symbol_id).is_import()
            || (ctx.source_type().is_script()
                && scoping.symbol_scope_id(symbol_id) == scoping.root_scope_id())
        {
            return true;
        }

        if let Some(sv) = ctx.state.symbols.value(symbol_id) {
            sv.references.has_writes()
        } else {
            scoping.symbol_is_mutated(symbol_id)
        }
    }

    /// Whether the current read closes over a block-scoped binding.
    ///
    /// This is a structural test, not proof that the binding is currently in its
    /// Temporal Dead Zone. Moving such a read before an `await`/`yield` can expose
    /// the TDZ while outer code is still initializing the binding. A same-function
    /// binding cannot be initialized mid-suspension, so it stays inlinable.
    ///
    /// <https://github.com/rolldown/rolldown/issues/9959>
    fn is_closed_over_block_scoped_read(symbol_id: SymbolId, ctx: &TraverseCtx<'a>) -> bool {
        let scoping = ctx.scoping();
        if !scoping.symbol_flags(symbol_id).is_block_scoped() {
            return false;
        }

        let binding_scope = scoping.symbol_scope_id(symbol_id);
        Self::read_crosses_function_boundary(ctx.current_scope_id(), binding_scope, ctx)
    }

    /// Whether moving this identifier read earlier could observe a different
    /// value or enter a closed-over lexical's TDZ.
    fn identifier_read_blocks_reorder(id: &IdentifierReference<'a>, ctx: &TraverseCtx<'a>) -> bool {
        let symbol_id = ctx.scoping().get_reference(id.reference_id()).symbol_id();
        symbol_id.is_none_or(|symbol_id| {
            Self::symbol_value_may_change(symbol_id, ctx)
                || Self::is_closed_over_block_scoped_read(symbol_id, ctx)
        })
    }

    /// Whether evaluating a member assignment-target part earlier could observe
    /// a different binding value. This includes ordinary mutation and closed-over
    /// lexicals that could still be in their TDZ (e.g. `v.x = await f()` or
    /// `obj[v] = await f()`).
    fn member_part_blocks_reorder(expr: &Expression<'a>, ctx: &TraverseCtx<'a>) -> bool {
        match expr {
            Expression::Identifier(id) => Self::identifier_read_blocks_reorder(id, ctx),
            Expression::ThisExpression(_) => false,
            _ => true,
        }
    }

    /// Whether evaluating a computed member key before a side-effecting
    /// replacement could observe a different value.
    ///
    /// The key expression and `GetValue` move before the assignment RHS, but
    /// `ToPropertyKey` still happens afterward. A scope-independent literal or
    /// a stable simple reference is therefore safe regardless of its value type.
    /// <https://tc39.es/ecma262/#sec-evaluate-property-access-with-expression-key>
    fn computed_key_blocks_reorder(key: &Expression<'a>, ctx: &TraverseCtx<'a>) -> bool {
        !key.is_literal_value(false, ctx) && Self::member_part_blocks_reorder(key, ctx)
    }

    /// True if the scope chain from `read_scope` up to (excluding) `stop_scope`
    /// crosses a function boundary — i.e. the read is in a closure relative to
    /// `stop_scope`. Async/generator/arrow scopes are all `Function`.
    fn read_crosses_function_boundary(
        read_scope: ScopeId,
        stop_scope: ScopeId,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        let scoping = ctx.scoping();
        scoping
            .scope_ancestors(read_scope)
            .take_while(|&scope_id| scope_id != stop_scope)
            .any(|scope_id| scoping.scope_flags(scope_id).is_function())
    }
}

impl<'a> Traverse<'a> for PeepholeOptimizations {
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        // Any module loader (`import`, `export * from`, `export … from`) can, on a
        // cycle, evaluate a foreign module that observes a not-yet-assigned binding
        // our exports close over. So the program root starts its prelude "unsafe"
        // when the body has any loader — bailing every program-scope var inline.
        // Loaders are hoisted, so scan the whole current body each pass (an
        // import may follow a leading var, or an earlier pass may remove one).
        let module_has_loaders = program
            .body
            .iter()
            .any(|s| s.as_module_declaration().is_some_and(|m| m.source().is_some()));
        // `enter`/`exit_function_body` are balanced, so the stack is back to its
        // single program-root entry by the next pass; reset it in place rather
        // than reallocating (matching the `reset`/`clear` above).
        *ctx.state.body_unsafe_stack.last_mut() =
            (ctx.scoping().root_scope_id(), module_has_loaders);
        // `PassChanges` is managed by pass completion, not reset per
        // traversal.
    }

    fn enter_function_body(&mut self, _body: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        ctx.state.body_unsafe_stack.push((ctx.current_scope_id(), false));
    }

    fn exit_function_body(&mut self, _body: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        ctx.state.body_unsafe_stack.pop();
    }

    fn exit_program(&mut self, _program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        // Private member usage is collected only in full optimization mode.
        debug_assert!(ctx.is_tree_shake_only() || ctx.state.private_member_usage.is_at_root());
    }

    fn exit_statements(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        Self::minimize_statements(stmts, ctx);
    }

    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::keep_track_of_pure_functions(stmt, ctx);
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.is_tree_shake_only() {
            match stmt {
                Statement::BlockStatement(_) => Self::try_optimize_block(stmt, ctx),
                Statement::IfStatement(_) => Self::try_fold_if(stmt, ctx),
                Statement::ForStatement(_) => Self::try_fold_for(stmt, ctx),
                Statement::TryStatement(_) => Self::try_fold_try(stmt, ctx),
                Statement::LabeledStatement(_) => Self::try_fold_labeled(stmt, ctx),
                Statement::FunctionDeclaration(_) => {
                    Self::remove_unused_function_declaration(stmt, ctx);
                }
                Statement::ClassDeclaration(_) => {
                    Self::remove_unused_class_declaration(stmt, ctx);
                }
                Statement::ExpressionStatement(_) => {
                    Self::try_fold_expression_stmt(stmt, ctx);
                }
                Statement::ImportDeclaration(_) => {
                    Self::remove_unused_import_specifiers(stmt, ctx);
                }
                _ => {}
            }
        } else {
            match stmt {
                Statement::BlockStatement(_) => Self::try_optimize_block(stmt, ctx),
                Statement::IfStatement(s) => {
                    Self::minimize_expression_in_boolean_context(&mut s.test, ctx);
                    Self::try_fold_if(stmt, ctx);
                    if let Statement::IfStatement(if_stmt) = stmt
                        && let Some(folded_stmt) = Self::try_minimize_if(if_stmt, ctx)
                    {
                        ctx.replace_statement(stmt, folded_stmt);
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
                Statement::FunctionDeclaration(f) => {
                    Self::init_function_declaration_symbol_value(f.id.as_ref(), ctx);
                    Self::remove_unused_function_declaration(stmt, ctx);
                }
                Statement::ClassDeclaration(c) => {
                    Self::init_class_declaration_symbol_value(c, ctx);
                    Self::remove_unused_class_declaration(stmt, ctx);
                }
                Statement::ImportDeclaration(_) => Self::remove_unused_import_specifiers(stmt, ctx),
                _ => {}
            }
            Self::try_fold_expression_stmt(stmt, ctx);
        }

        // Maintain the per-body declarative-prelude flag used by
        // `is_hoisted_var_inlineable`.
        if !Self::is_declarative_body_statement(stmt) {
            Self::mark_current_body_unsafe(ctx);
        }
    }

    fn exit_for_statement(&mut self, stmt: &mut ForStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.is_tree_shake_only() {
            return;
        }
        Self::substitute_for_statement(stmt, ctx);
        Self::minimize_for_statement(stmt, ctx);
    }

    fn exit_return_statement(&mut self, stmt: &mut ReturnStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.is_tree_shake_only() {
            return;
        }
        Self::substitute_return_statement(stmt, ctx);
    }

    fn exit_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.is_tree_shake_only() {
            return;
        }
        Self::substitute_variable_declaration(decl, ctx);
    }

    fn exit_variable_declarator(
        &mut self,
        decl: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        Self::init_symbol_value(decl, ctx);
        // Per-declarator update of the body-unsafe flag. Catches multi-declarator
        // statements (`var [x=call()] = '', flag = true;`, possibly produced by
        // join-vars) where an earlier declarator runs user code via a
        // destructuring default or non-literal init — the per-statement check
        // would fire too late for subsequent declarators' `init_symbol_value`.
        if !Self::is_declarative_variable_declarator(decl) {
            Self::mark_current_body_unsafe(ctx);
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        // Tree-shaking mode: fewer passes than full minify below. Only the ones
        // that remove code, plus the constant folds those removals need. The
        // folds stay on because the removal passes don't evaluate compound
        // conditions themselves: `if ('production' === 'production')` must fold
        // to `true` before the dead branch can be dropped. Passes that only
        // shrink code (`substitute_*`, `minimize_*`) are left out.
        if ctx.is_tree_shake_only() {
            match expr {
                Expression::TemplateLiteral(t) => {
                    Self::inline_template_literal(t, ctx);
                }
                Expression::ObjectExpression(e) => Self::fold_object_exp(e, ctx),
                Expression::BinaryExpression(_) => {
                    Self::fold_binary_expr(expr, ctx);
                    Self::fold_binary_typeof_comparison(expr, ctx);
                }
                Expression::UnaryExpression(_) => Self::fold_unary_expr(expr, ctx),
                Expression::StaticMemberExpression(_) => {
                    Self::fold_static_member_expr(expr, ctx);
                }
                Expression::ComputedMemberExpression(_) => {
                    Self::fold_computed_member_expr(expr, ctx);
                }
                Expression::LogicalExpression(_) => Self::fold_logical_expr(expr, ctx),
                Expression::ChainExpression(_) => Self::fold_chain_expr(expr, ctx),
                Expression::CallExpression(_) => {
                    Self::fold_call_expression(expr, ctx);
                    Self::substitute_iife_call(expr, ctx);
                    Self::remove_dead_code_call_expression(expr, ctx);
                }
                Expression::ConditionalExpression(_) => {
                    Self::try_fold_conditional_expression(expr, ctx);
                }
                Expression::SequenceExpression(_) => {
                    Self::remove_sequence_expression(expr, ctx);
                }
                Expression::AssignmentExpression(_) => {
                    Self::remove_unused_assignment_expr(expr, ctx);
                }
                _ => {}
            }
        } else {
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
                    Self::fold_sequence_expression(expr, ctx);
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
                    Self::fold_sequence_expression(expr, ctx);
                }
                Expression::YieldExpression(_) | Expression::AwaitExpression(_) => {
                    Self::fold_sequence_expression(expr, ctx);
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
                    Self::fold_sequence_expression(expr, ctx);
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
                    if let Some(changed) = Self::minimize_conditional_expression(logical_expr, ctx)
                    {
                        ctx.replace_expression(expr, changed);
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
                Expression::ArrayExpression(_) => {
                    Self::try_flatten_array_expression_elements(expr, ctx);
                    Self::substitute_array_expression(expr, ctx);
                }
                Expression::Identifier(_) => Self::inline_identifier_reference(expr, ctx),
                _ => {}
            }
        }
    }

    fn exit_unary_expression(&mut self, expr: &mut UnaryExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.is_tree_shake_only() {
            return;
        }
        if expr.operator.is_not() {
            Self::minimize_expression_in_boolean_context(&mut expr.argument, ctx);
        }
    }

    fn exit_call_expression(&mut self, e: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !ctx.is_tree_shake_only() {
            Self::substitute_call_expression(e, ctx);
            Self::remove_empty_spread_arguments(&mut e.arguments);
        }
        // Re-evaluate each iteration: peephole folding/inlining may expose a
        // pure-eligible arg shape that `Normalize`'s one-shot pass missed.
        Normalize::set_no_side_effects_to_call_expr(e, ctx);
    }

    fn exit_new_expression(&mut self, e: &mut NewExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !ctx.is_tree_shake_only() {
            Self::substitute_new_expression(e, ctx);
            Self::remove_empty_spread_arguments(&mut e.arguments);
        }
        Normalize::set_pure_or_no_side_effects_to_new_expr(e, ctx);
    }

    fn exit_object_property(&mut self, prop: &mut ObjectProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.is_tree_shake_only() {
            return;
        }
        Self::substitute_object_property(prop, ctx);
    }

    fn exit_assignment_target_property(
        &mut self,
        node: &mut AssignmentTargetProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.is_tree_shake_only() {
            return;
        }
        Self::substitute_assignment_target_property(node, ctx);
    }

    fn exit_assignment_target_property_property(
        &mut self,
        prop: &mut AssignmentTargetPropertyProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.is_tree_shake_only() {
            return;
        }
        Self::substitute_assignment_target_property_property(prop, ctx);
    }

    fn exit_binding_property(&mut self, prop: &mut BindingProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.is_tree_shake_only() {
            return;
        }
        Self::substitute_binding_property(prop, ctx);
    }

    fn exit_method_definition(
        &mut self,
        prop: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.is_tree_shake_only() {
            return;
        }
        Self::substitute_method_definition(prop, ctx);
    }

    fn exit_property_definition(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.is_tree_shake_only() {
            return;
        }
        Self::substitute_property_definition(prop, ctx);
    }

    fn exit_accessor_property(
        &mut self,
        prop: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.is_tree_shake_only() {
            return;
        }
        Self::substitute_accessor_property(prop, ctx);
    }

    fn exit_member_expression(
        &mut self,
        expr: &mut MemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.is_tree_shake_only() {
            return;
        }
        Self::convert_to_dotted_properties(expr, ctx);
    }

    fn enter_class_body(&mut self, _body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.is_tree_shake_only() {
            return;
        }
        ctx.state.private_member_usage.enter_class();
    }

    fn exit_class_body(&mut self, body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.is_tree_shake_only() {
            return;
        }
        Self::remove_dead_code_exit_class_body(body, ctx);
        Self::remove_unused_private_members(body, ctx);
        ctx.state.private_member_usage.exit_class(Self::declared_private_member_names(body));
    }

    fn exit_catch_clause(&mut self, catch: &mut CatchClause<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.is_tree_shake_only() {
            return;
        }
        Self::substitute_catch_clause(catch, ctx);
    }

    fn exit_private_field_expression(
        &mut self,
        node: &mut PrivateFieldExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.is_tree_shake_only() {
            return;
        }
        ctx.state.private_member_usage.record_use(node.field.name.into());
    }

    fn exit_private_in_expression(
        &mut self,
        node: &mut PrivateInExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.is_tree_shake_only() {
            return;
        }
        ctx.state.private_member_usage.record_use(node.left.name.into());
    }
}
