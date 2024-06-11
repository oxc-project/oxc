use std::cell::Cell;

use serde::Deserialize;

use oxc_allocator::Vec as ArenaVec;
use oxc_ast::ast::*;
use oxc_span::{Atom, Span, SPAN};
use oxc_syntax::{
    reference::ReferenceFlag,
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};
use oxc_traverse::{FinderRet, TraverseCtx};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ArrowFunctionsOptions {
    /// This option enables the following:
    /// * Wrap the generated function in .bind(this) and keeps uses of this inside the function as-is, instead of using a renamed this.
    /// * Add a runtime check to ensure the functions are not instantiated.
    /// * Add names to arrow functions.
    #[serde(default)]
    pub spec: bool,
}

/// [plugin-transform-arrow-functions](https://babel.dev/docs/babel-plugin-transform-arrow-functions)
///
/// This plugin transforms arrow functions to function expressions.
///
/// This plugin is included in `preset-env`
///
/// References:
///
/// * <https://babeljs.io/docs/babel-plugin-transform-arrow-functions>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-arrow-functions>
//
// TODO: The `spec` option is not currently supported. Add support for it.
// TODO: Does not currently handle correctly:
// * Arrow function in a function's params. `function foo(f = () => this) {}`
// * Arrow function in a class field initializer. `class C { foo = () => this }`
pub struct ArrowFunctions<'a> {
    _options: ArrowFunctionsOptions,
    // `true` if currently in an arrow function
    is_in_arrow: bool,
    // `ScopeId` of current vars block (i.e. program, function, class static block, TS module block)
    current_var_scope_id: ScopeId,
    // Stack of blocks which need a `var _this = this;` statement inserted in them
    var_scopes: Vec<VarScope<'a>>,
}

#[derive(Clone)]
struct VarScope<'a> {
    // `ScopeId` of block which `this` is bound in
    scope_id: ScopeId,
    // `SymbolId` of the binding for `_this`
    symbol_id: SymbolId,
    // `_this` var name
    name: Atom<'a>,
}

impl<'a> ArrowFunctions<'a> {
    pub fn new(options: ArrowFunctionsOptions) -> Self {
        Self {
            _options: options,
            is_in_arrow: false,
            current_var_scope_id: ScopeId::new(0), // Dummy value, overwritten in `enter_program`
            var_scopes: vec![],
        }
    }

    // Visitors for AST nodes which are vars blocks (i.e. `this` is bound in them).
    // These visitors:
    // 1. Update `self.current_var_scope_id` on entry / exit.
    // 2. Insert `var _this = this;` statement if required.

    pub fn transform_program(&mut self, _program: &Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.current_var_scope_id = ctx.current_scope_id();
    }

    pub fn transform_program_on_exit(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.add_var_statement_to_block(&mut program.body, ctx);
        // No point updating `current_var_scope_id`
    }

    pub fn transform_function(&mut self, _func: &Function<'a>, ctx: &mut TraverseCtx<'a>) {
        self.current_var_scope_id = ctx.current_scope_id();
    }

    pub fn transform_function_on_exit(
        &mut self,
        func: &mut Function<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(body) = func.body.as_mut() {
            self.add_var_statement_to_block(&mut body.statements, ctx);
        }
        self.update_current_var_scope(ctx);
    }

    pub fn transform_static_block(&mut self, _block: &StaticBlock<'a>, ctx: &mut TraverseCtx<'a>) {
        self.current_var_scope_id = ctx.current_scope_id();
    }

    pub fn transform_static_block_on_exit(
        &mut self,
        block: &mut StaticBlock<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.add_var_statement_to_block(&mut block.body, ctx);
        self.update_current_var_scope(ctx);
    }

    pub fn transform_ts_module_block(
        &mut self,
        _block: &mut TSModuleBlock<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.current_var_scope_id = ctx.current_scope_id();
    }

    pub fn transform_ts_module_block_on_exit(
        &mut self,
        block: &mut TSModuleBlock<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.add_var_statement_to_block(&mut block.body, ctx);
        self.update_current_var_scope(ctx);
    }

    // Visitors for arrow functions.
    // Update `self.is_in_arrow` on entry/exit + transform arrow function to full function

    pub fn transform_arrow_function_expression(
        &mut self,
        _arrow_function_expr: &ArrowFunctionExpression<'a>,
    ) {
        self.is_in_arrow = true;
    }

    pub fn transform_expression_on_exit(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Expression::ArrowFunctionExpression(arrow_function_expr) = expr {
            *expr = self.transform_arrow_function(arrow_function_expr, ctx);
        }
    }

    /// Replace `this` in arrow function with `_this`
    pub fn transform_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let span = if let Expression::ThisExpression(this_expr) = expr {
            this_expr.span
        } else {
            return;
        };

        if !self.is_in_arrow {
            return;
        }

        let ident = self.create_this_ident(span, ctx);
        *expr = ctx.ast.identifier_reference_expression(ident);
    }

    /// Change `this` in `<this>` or `</this>` to `_this`.
    pub fn transform_jsx_element_name(
        &mut self,
        name: &mut JSXElementName<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_in_arrow {
            return;
        }

        let ident = match name {
            JSXElementName::Identifier(ident) => ident,
            JSXElementName::MemberExpression(member_expr) => {
                member_expr.get_object_identifier_mut()
            }
            JSXElementName::NamespacedName(_) => return,
        };
        if ident.name != "this" {
            return;
        }

        // We can't produce a proper identifier with a `ReferenceId` because `JSXIdentifier`
        // lacks that field. https://github.com/oxc-project/oxc/issues/3528
        // So generate a reference and just use its name.
        // If JSX transform is enabled, that transform runs before this and will have converted
        // this to a proper `ThisExpression`, and this visitor won't run.
        // So only a problem if JSX transform is disabled.
        let replacement = self.create_this_ident(SPAN, ctx);
        ident.name = replacement.name;
    }

    /// Update current var scope to parent var scope when exiting a function or other var block.
    ///
    /// # Panics
    /// Panics if called in program scope. Don't do that!
    fn update_current_var_scope(&mut self, ctx: &mut TraverseCtx<'a>) {
        let current_scope_id = ctx.current_scope_id();
        // TODO: This panics for fixture
        // `coverage/babel/packages/babel-parser/test/fixtures/typescript/enum/members-reserved-words`
        // ```ts
        // enum E {
        //   const,
        //   default
        // }
        // ```
        // Presumably problem is that TS transform has converted enum to a function
        // but `current_scope_id` is out of sync, or TS transform hasn't set scope ID for function
        let parent_scope_id = ctx.scopes().get_parent_id(current_scope_id).unwrap();
        let parent_var_scope_id = ctx
            .find_scope_starting_with(parent_scope_id, |scope_id| {
                match ctx.scopes().get_flags(scope_id) {
                    flags if flags.is_var() => FinderRet::Found(scope_id),
                    _ => FinderRet::Continue,
                }
            })
            .unwrap();
        self.current_var_scope_id = parent_var_scope_id;
    }

    /// Create `IdentifierReference` for `_this`.
    ///
    /// If this is first `this` in an arrow function found in AST, generate UID for `_this` var.
    /// If this is first `this` in an arrow function found for this particular `this`, create a Symbol
    /// for it, and record that in `var_scopes`. This signals to `exit_function` etc to
    /// insert a `var _this = this;` statement using that `SymbolId`.
    fn create_this_ident(
        &mut self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        // Get or create symbol for `_this` in this context
        let (symbol_id, name) = self.get_this_symbol_for_current_scope().unwrap_or_else(|| {
            // No existing symbol for `_this` in this context.
            // Create one + record it in `var_scopes`.
            // TODO(improve-on-babel): By definition, it's impossible for there to be more than 1 `this`
            // binding accessible from any position in AST. So we could use the same var name for every
            // `_this` in the AST, rather than generating a separate UID for each one.
            let scope_id = self.current_var_scope_id;
            let symbol_id = ctx.generate_uid("this", scope_id, SymbolFlags::FunctionScopedVariable);
            let name = ctx.ast.new_atom(&ctx.symbols().names[symbol_id]);
            self.var_scopes.push(VarScope { scope_id, symbol_id, name: name.clone() });
            (symbol_id, name)
        });

        // Reference is always read-only because `this` cannot be assigned to
        let reference_id =
            ctx.create_bound_reference(name.to_compact_str(), symbol_id, ReferenceFlag::Read);
        IdentifierReference::new_read(span, name, Some(reference_id))
    }

    /// Add `var _this = this;` statement at top of statements block if required.
    ///
    /// ```js
    /// function a() {
    ///   return () => console.log(this);
    /// }
    /// // to
    /// function a() {
    ///   var _this = this;
    ///   return function() { return console.log(_this); };
    /// }
    /// ```
    fn add_var_statement_to_block(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Check if need to add to block
        let Some((symbol_id, name)) = self.get_this_symbol_for_current_scope() else {
            return;
        };

        // Remove from stack
        self.var_scopes.pop();

        // Insert `var _this = this;` statement at top of block
        let binding_ident =
            BindingIdentifier { span: SPAN, name, symbol_id: Cell::new(Some(symbol_id)) };
        let binding_pattern =
            ctx.ast.binding_pattern(ctx.ast.binding_pattern_identifier(binding_ident), None, false);
        let variable_declarator = ctx.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Var,
            binding_pattern,
            Some(ctx.ast.this_expression(SPAN)),
            false,
        );
        let stmt = ctx.ast.variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            ctx.ast.new_vec_single(variable_declarator),
            Modifiers::empty(),
        );
        let stmt = Statement::VariableDeclaration(stmt);
        stmts.insert(0, stmt);
    }

    /// Get `SymbolId` for `_this` var in current vars block (if there is one)
    fn get_this_symbol_for_current_scope(&self) -> Option<(SymbolId, Atom<'a>)> {
        let var_scope = self.var_scopes.last()?;
        if var_scope.scope_id == self.current_var_scope_id {
            Some((var_scope.symbol_id, var_scope.name.clone()))
        } else {
            None
        }
    }

    fn transform_arrow_function(
        &mut self,
        arrow_function_expr: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        // Convert arrow function to full function
        let mut body = ctx.ast.copy(&arrow_function_expr.body);

        if arrow_function_expr.expression {
            let first_stmt = body.statements.remove(0);
            if let Statement::ExpressionStatement(stmt) = first_stmt {
                let return_statement =
                    ctx.ast.return_statement(stmt.span, Some(ctx.ast.copy(&stmt.expression)));
                body.statements.push(return_statement);
            }
        }

        let new_function = Function {
            r#type: FunctionType::FunctionExpression,
            span: arrow_function_expr.span,
            id: None,
            generator: false,
            r#async: arrow_function_expr.r#async,
            this_param: None,
            params: ctx.ast.copy(&arrow_function_expr.params),
            body: Some(body),
            type_parameters: ctx.ast.copy(&arrow_function_expr.type_parameters),
            return_type: ctx.ast.copy(&arrow_function_expr.return_type),
            modifiers: Modifiers::empty(),
            scope_id: Cell::new(arrow_function_expr.scope_id.get()),
        };

        let func = Expression::FunctionExpression(ctx.alloc(new_function));

        // Check if another arrow function above this, or if we're no longer in an arrow function.
        // NB: Arrow function's scope has already been exited, as we're in `exit_expression`.
        let is_not_in_arrow = ctx
            .find_scope(|scope_id| match ctx.scopes().get_flags(scope_id) {
                flags if flags.is_arrow() => FinderRet::Stop,
                flags if flags.is_var() => FinderRet::Found(()),
                _ => FinderRet::Continue,
            })
            .is_some();
        if is_not_in_arrow {
            self.is_in_arrow = false;
        }

        func
    }
}
