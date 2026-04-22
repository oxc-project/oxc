//! Utility transform to add `var` or `let` declarations to top of statement blocks.
//!
//! `VarDeclarationsStore` contains a stack of `Declarators`s, each comprising
//! 2 x `Vec<Declarator<'a>>` (1 for `var`s, 1 for `let`s).
//! `VarDeclarationsStore` is stored on `TransformState`.
//!
//! `Common` transform pushes an empty entry onto this stack when entering a statement block,
//! and when exiting the block, writes `var` / `let` statements to top of block.
//!
//! Other transforms can add declarators to the store by calling methods of `VarDeclarationsStore`:
//!
//! ```rs
//! ctx.state.var_declarations.insert_var(name, binding, None, ctx.ast);
//! ctx.state.var_declarations.insert_let(name2, binding2, None, ctx.ast);
//! ```

use oxc_allocator::Vec as ArenaVec;
use oxc_ast::{AstBuilder, NONE, ast::*};
use oxc_data_structures::stack::SparseStack;
use oxc_span::SPAN;
use oxc_traverse::{BoundIdentifier, ast_operations::GatherNodeParts};

use crate::context::TraverseCtx;

/// Store for `VariableDeclarator`s to be added to enclosing statement block.
pub struct VarDeclarationsStore<'a> {
    stack: SparseStack<Declarators<'a>>,
}

/// Declarators to be inserted in a statement block.
struct Declarators<'a> {
    var_declarators: ArenaVec<'a, VariableDeclarator<'a>>,
    let_declarators: ArenaVec<'a, VariableDeclarator<'a>>,
}

impl<'a> Declarators<'a> {
    fn new(ast: AstBuilder<'a>) -> Self {
        Self { var_declarators: ast.vec(), let_declarators: ast.vec() }
    }
}

// Public methods
impl<'a> VarDeclarationsStore<'a> {
    /// Create new `VarDeclarationsStore`.
    pub fn new() -> Self {
        Self { stack: SparseStack::new() }
    }

    /// Add a `var` declaration to be inserted at top of current enclosing statement block,
    /// given a `BoundIdentifier`.
    #[inline]
    pub fn insert_var(&mut self, binding: &BoundIdentifier<'a>, ast: AstBuilder<'a>) {
        let ident = ast.binding_identifier_with_symbol_id(SPAN, binding.name, binding.symbol_id);
        let pattern = BindingPattern::BindingIdentifier(ast.alloc(ident));
        self.insert_var_binding_pattern(pattern, None, ast);
    }

    /// Add a `var` declaration with the given init expression to be inserted at top of
    /// current enclosing statement block, given a `BoundIdentifier`.
    #[inline]
    pub fn insert_var_with_init(
        &mut self,
        binding: &BoundIdentifier<'a>,
        init: Expression<'a>,
        ast: AstBuilder<'a>,
    ) {
        let ident = ast.binding_identifier_with_symbol_id(SPAN, binding.name, binding.symbol_id);
        let pattern = BindingPattern::BindingIdentifier(ast.alloc(ident));
        self.insert_var_binding_pattern(pattern, Some(init), ast);
    }

    /// Create a new UID based on `name`, add a `var` declaration to be inserted at the top of
    /// the current enclosing statement block, and return the [`BoundIdentifier`].
    ///
    /// This is a static method to avoid borrow conflicts when accessing the store through
    /// `ctx.state.var_declarations` while also passing `ctx`.
    #[inline]
    pub fn create_uid_var(name: &str, ctx: &mut TraverseCtx<'a>) -> BoundIdentifier<'a> {
        let binding = ctx.generate_uid_in_current_hoist_scope(name);
        ctx.state.var_declarations.insert_var(&binding, ctx.ast);
        binding
    }

    /// Create a new UID based on `name`, add a `var` declaration with the given init expression
    /// to be inserted at the top of the current enclosing statement block, and return the
    /// [`BoundIdentifier`].
    ///
    /// This is a static method to avoid borrow conflicts when accessing the store through
    /// `ctx.state.var_declarations` while also passing `ctx`.
    #[inline]
    pub fn create_uid_var_with_init(
        name: &str,
        expression: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        let binding = ctx.generate_uid_in_current_hoist_scope(name);
        ctx.state.var_declarations.insert_var_with_init(&binding, expression, ctx.ast);
        binding
    }

    /// Create a new UID with name based on `node`, add a `var` declaration to be inserted
    /// at the top of the current enclosing statement block, and return the [`BoundIdentifier`].
    ///
    /// This is a static method to avoid borrow conflicts when accessing the store through
    /// `ctx.state.var_declarations` while also passing `ctx`.
    #[inline]
    pub fn create_uid_var_based_on_node<N: GatherNodeParts<'a>>(
        node: &N,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        let binding = ctx.generate_uid_in_current_hoist_scope_based_on_node(node);
        ctx.state.var_declarations.insert_var(&binding, ctx.ast);
        binding
    }

    /// Add a `let` declaration to be inserted at top of current enclosing statement block,
    /// given a `BoundIdentifier`.
    pub fn insert_let(
        &mut self,
        binding: &BoundIdentifier<'a>,
        init: Option<Expression<'a>>,
        ast: AstBuilder<'a>,
    ) {
        let ident = ast.binding_identifier_with_symbol_id(SPAN, binding.name, binding.symbol_id);
        let pattern = BindingPattern::BindingIdentifier(ast.alloc(ident));
        self.insert_let_binding_pattern(pattern, init, ast);
    }

    /// Add a `var` declaration to be inserted at top of current enclosing statement block,
    /// given a `BindingPattern`.
    pub fn insert_var_binding_pattern(
        &mut self,
        ident: BindingPattern<'a>,
        init: Option<Expression<'a>>,
        ast: AstBuilder<'a>,
    ) {
        let declarator =
            ast.variable_declarator(SPAN, VariableDeclarationKind::Var, ident, NONE, init, false);
        self.insert_var_declarator(declarator, ast);
    }

    /// Add a `let` declaration to be inserted at top of current enclosing statement block,
    /// given a `BindingPattern`.
    pub fn insert_let_binding_pattern(
        &mut self,
        ident: BindingPattern<'a>,
        init: Option<Expression<'a>>,
        ast: AstBuilder<'a>,
    ) {
        let declarator =
            ast.variable_declarator(SPAN, VariableDeclarationKind::Let, ident, NONE, init, false);
        self.insert_let_declarator(declarator, ast);
    }

    /// Add a `var` declaration to be inserted at top of current enclosing statement block.
    pub fn insert_var_declarator(
        &mut self,
        declarator: VariableDeclarator<'a>,
        ast: AstBuilder<'a>,
    ) {
        let declarators = self.stack.last_mut_or_init(|| Declarators::new(ast));
        declarators.var_declarators.push(declarator);
    }

    /// Add a `let` declaration to be inserted at top of current enclosing statement block.
    pub fn insert_let_declarator(
        &mut self,
        declarator: VariableDeclarator<'a>,
        ast: AstBuilder<'a>,
    ) {
        let declarators = self.stack.last_mut_or_init(|| Declarators::new(ast));
        declarators.let_declarators.push(declarator);
    }
}

// Internal methods - called by `Common` transform
impl<'a> VarDeclarationsStore<'a> {
    pub(crate) fn record_entering_statements(&mut self) {
        self.stack.push(None);
    }

    pub(crate) fn insert_into_statements(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        is_program_body: bool,
        ast: AstBuilder<'a>,
    ) {
        if is_program_body {
            // Handle in `insert_into_program` instead
            return;
        }

        if let Some((var_statement, let_statement)) = self.get_var_statement(ast) {
            let mut new_stmts = ast.vec_with_capacity(stmts.len() + 2);
            match (var_statement, let_statement) {
                (Some(var_statement), Some(let_statement)) => {
                    // Insert `var` and `let` statements
                    new_stmts.extend([var_statement, let_statement]);
                }
                (Some(statement), None) | (None, Some(statement)) => {
                    // Insert `var` or `let` statement
                    new_stmts.push(statement);
                }
                (None, None) => return,
            }
            new_stmts.append(stmts);
            *stmts = new_stmts;
        }
    }

    /// Pop the var/let declarations from the stack and return as statements.
    pub(crate) fn get_var_statement(
        &mut self,
        ast: AstBuilder<'a>,
    ) -> Option<(Option<Statement<'a>>, Option<Statement<'a>>)> {
        let Declarators { var_declarators, let_declarators } = self.stack.pop()?;

        let var_statement = (!var_declarators.is_empty())
            .then(|| Self::create_declaration(VariableDeclarationKind::Var, var_declarators, ast));
        let let_statement = (!let_declarators.is_empty())
            .then(|| Self::create_declaration(VariableDeclarationKind::Let, let_declarators, ast));

        Some((var_statement, let_statement))
    }

    /// Assert that the stack is exhausted (debug-only).
    // `#[inline(always)]` because this is a no-op in release mode
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(crate) fn assert_stack_exhausted(&self) {
        debug_assert!(self.stack.is_exhausted());
        debug_assert!(self.stack.last().is_none());
    }

    fn create_declaration(
        kind: VariableDeclarationKind,
        declarators: ArenaVec<'a, VariableDeclarator<'a>>,
        ast: AstBuilder<'a>,
    ) -> Statement<'a> {
        Statement::VariableDeclaration(ast.alloc_variable_declaration(
            SPAN,
            kind,
            declarators,
            false,
        ))
    }
}
