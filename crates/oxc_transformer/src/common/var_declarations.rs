//! Utility transform to add `var` or `let` declarations to top of statement blocks.
//!
//! `VarDeclarationsStore` contains a stack of `Declarators`s, each comprising
//! 2 x `Vec<Declarator<'a>>` (1 for `var`s, 1 for `let`s).
//! `VarDeclarationsStore` is stored on `TransformCtx`.
//!
//! `VarDeclarations` transform pushes an empty entry onto this stack when entering a statement block,
//! and when exiting the block, writes `var` / `let` statements to top of block.
//!
//! Other transforms can add declarators to the store by calling methods of `VarDeclarationsStore`:
//!
//! ```rs
//! self.ctx.var_declarations.insert_var(name, binding, None, ctx);
//! self.ctx.var_declarations.insert_let(name2, binding2, None, ctx);
//! ```

use std::cell::RefCell;

use oxc_allocator::Vec as ArenaVec;
use oxc_ast::ast::*;
use oxc_data_structures::stack::SparseStack;
use oxc_span::SPAN;
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse, TraverseCtx};

use crate::TransformCtx;

/// Transform that maintains the stack of `Vec<VariableDeclarator>`s, and adds a `var` statement
/// to top of a statement block if another transform has requested that.
///
/// Must run after all other transforms except `TopLevelStatements`.
pub struct VarDeclarations<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> VarDeclarations<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for VarDeclarations<'a, 'ctx> {
    fn enter_statements(
        &mut self,
        _stmts: &mut ArenaVec<'a, Statement<'a>>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.ctx.var_declarations.record_entering_statements();
    }

    fn exit_statements(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.ctx.var_declarations.insert_into_statements(stmts, ctx);
    }

    fn exit_program(&mut self, _program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.ctx.var_declarations.insert_into_program(self.ctx, ctx);
    }
}

/// Store for `VariableDeclarator`s to be added to enclosing statement block.
pub struct VarDeclarationsStore<'a> {
    stack: RefCell<SparseStack<Declarators<'a>>>,
}

/// Declarators to be inserted in a statement block.
struct Declarators<'a> {
    var_declarators: ArenaVec<'a, VariableDeclarator<'a>>,
    let_declarators: ArenaVec<'a, VariableDeclarator<'a>>,
}

impl<'a> Declarators<'a> {
    fn new(ctx: &TraverseCtx<'a>) -> Self {
        Self { var_declarators: ctx.ast.vec(), let_declarators: ctx.ast.vec() }
    }
}

// Public methods
impl<'a> VarDeclarationsStore<'a> {
    /// Create new `VarDeclarationsStore`.
    pub fn new() -> Self {
        Self { stack: RefCell::new(SparseStack::new()) }
    }

    /// Add a `var` declaration to be inserted at top of current enclosing statement block,
    /// given a `BoundIdentifier`.
    pub fn insert_var(
        &self,
        binding: &BoundIdentifier<'a>,
        init: Option<Expression<'a>>,
        ctx: &TraverseCtx<'a>,
    ) {
        let pattern = binding.create_binding_pattern(ctx);
        self.insert_var_binding_pattern(pattern, init, ctx);
    }

    /// Add a `let` declaration to be inserted at top of current enclosing statement block,
    /// given a `BoundIdentifier`.
    #[expect(dead_code)]
    pub fn insert_let(
        &self,
        binding: &BoundIdentifier<'a>,
        init: Option<Expression<'a>>,
        ctx: &TraverseCtx<'a>,
    ) {
        let pattern = binding.create_binding_pattern(ctx);
        self.insert_let_binding_pattern(pattern, init, ctx);
    }

    /// Add a `var` declaration to be inserted at top of current enclosing statement block,
    /// given a `BindingPattern`.
    pub fn insert_var_binding_pattern(
        &self,
        ident: BindingPattern<'a>,
        init: Option<Expression<'a>>,
        ctx: &TraverseCtx<'a>,
    ) {
        let declarator =
            ctx.ast.variable_declarator(SPAN, VariableDeclarationKind::Var, ident, init, false);
        self.insert_var_declarator(declarator, ctx);
    }

    /// Add a `let` declaration to be inserted at top of current enclosing statement block,
    /// given a `BindingPattern`.
    pub fn insert_let_binding_pattern(
        &self,
        ident: BindingPattern<'a>,
        init: Option<Expression<'a>>,
        ctx: &TraverseCtx<'a>,
    ) {
        let declarator =
            ctx.ast.variable_declarator(SPAN, VariableDeclarationKind::Let, ident, init, false);
        self.insert_let_declarator(declarator, ctx);
    }

    /// Add a `var` declaration to be inserted at top of current enclosing statement block.
    pub fn insert_var_declarator(&self, declarator: VariableDeclarator<'a>, ctx: &TraverseCtx<'a>) {
        let mut stack = self.stack.borrow_mut();
        let declarators = stack.last_mut_or_init(|| Declarators::new(ctx));
        declarators.var_declarators.push(declarator);
    }

    /// Add a `let` declaration to be inserted at top of current enclosing statement block.
    pub fn insert_let_declarator(&self, declarator: VariableDeclarator<'a>, ctx: &TraverseCtx<'a>) {
        let mut stack = self.stack.borrow_mut();
        let declarators = stack.last_mut_or_init(|| Declarators::new(ctx));
        declarators.let_declarators.push(declarator);
    }
}

// Internal methods
impl<'a> VarDeclarationsStore<'a> {
    fn record_entering_statements(&self) {
        let mut stack = self.stack.borrow_mut();
        stack.push(None);
    }

    fn insert_into_statements(
        &self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if matches!(ctx.parent(), Ancestor::ProgramBody(_)) {
            // Handle in `insert_into_program` instead
            return;
        }

        if let Some(insert_stmts) = self.get_var_statement(ctx) {
            stmts.splice(0..0, insert_stmts);
        }
    }

    fn insert_into_program(&self, transform_ctx: &TransformCtx<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(insert_stmts) = self.get_var_statement(ctx) {
            // Delegate to `TopLevelStatements`
            transform_ctx.top_level_statements.insert_statements(insert_stmts);
        }

        // Check stack is emptied
        let stack = self.stack.borrow();
        debug_assert!(stack.len() == 1);
        debug_assert!(stack.last().is_none());
    }

    fn get_var_statement(&self, ctx: &mut TraverseCtx<'a>) -> Option<Vec<Statement<'a>>> {
        let mut stack = self.stack.borrow_mut();
        let Declarators { var_declarators, let_declarators } = stack.pop()?;

        let mut stmts = Vec::with_capacity(2);
        if !var_declarators.is_empty() {
            stmts.push(Self::create_declaration(
                VariableDeclarationKind::Var,
                var_declarators,
                ctx,
            ));
        }
        if !let_declarators.is_empty() {
            stmts.push(Self::create_declaration(
                VariableDeclarationKind::Let,
                let_declarators,
                ctx,
            ));
        }
        Some(stmts)
    }

    fn create_declaration(
        kind: VariableDeclarationKind,
        declarators: ArenaVec<'a, VariableDeclarator<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        Statement::VariableDeclaration(ctx.ast.alloc_variable_declaration(
            SPAN,
            kind,
            declarators,
            false,
        ))
    }
}
