//! Utility transform to add `var` declarations to top of statement blocks.
//!
//! `VarDeclarationsStore` contains a stack of `Vec<VariableDeclarator>`s.
//! It is stored on `TransformCtx`.
//!
//! `VarDeclarations` transform pushes an empty entry onto this stack when entering a statement block,
//! and when exiting the block, writes a `var` statement to top of block containing the declarators.
//!
//! Other transforms can add declarators to the store by calling methods of `VarDeclarationsStore`:
//!
//! ```rs
//! self.ctx.var_declarations.insert_declarator(name, symbol_id, None, ctx);
//! ```

use std::cell::RefCell;

use oxc_allocator::Vec;
use oxc_ast::{ast::*, NONE};
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
        _stmts: &mut Vec<'a, Statement<'a>>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.ctx.var_declarations.record_entering_statements();
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.ctx.var_declarations.insert_into_statements(stmts, ctx);
    }

    fn exit_program(&mut self, _program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.ctx.var_declarations.insert_into_program(self.ctx, ctx);
    }
}

/// Store for `VariableDeclarator`s to be added to enclosing statement block.
pub struct VarDeclarationsStore<'a> {
    stack: RefCell<SparseStack<Vec<'a, VariableDeclarator<'a>>>>,
}

// Public methods
impl<'a> VarDeclarationsStore<'a> {
    /// Create new `VarDeclarationsStore`.
    pub fn new() -> Self {
        Self { stack: RefCell::new(SparseStack::new()) }
    }

    /// Add a `VariableDeclarator` to be inserted at top of current enclosing statement block,
    /// given a `BoundIdentifier`.
    pub fn insert(
        &self,
        binding: &BoundIdentifier<'a>,
        init: Option<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ident = binding.create_binding_identifier();
        let ident = ctx.ast.binding_pattern_kind_from_binding_identifier(ident);
        let ident = ctx.ast.binding_pattern(ident, NONE, false);
        self.insert_binding_pattern(ident, init, ctx);
    }

    /// Add a `VariableDeclarator` to be inserted at top of current enclosing statement block,
    /// given a `BindingPattern`.
    pub fn insert_binding_pattern(
        &self,
        ident: BindingPattern<'a>,
        init: Option<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let declarator =
            ctx.ast.variable_declarator(SPAN, VariableDeclarationKind::Var, ident, init, false);
        self.insert_declarator(declarator, ctx);
    }

    /// Add a `VariableDeclarator` to be inserted at top of current enclosing statement block.
    pub fn insert_declarator(&self, declarator: VariableDeclarator<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut stack = self.stack.borrow_mut();
        stack.last_mut_or_init(|| ctx.ast.vec()).push(declarator);
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
        stmts: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if matches!(ctx.parent(), Ancestor::ProgramBody(_)) {
            // Handle in `insert_into_program` instead
            return;
        }

        if let Some(stmt) = self.get_var_statement(ctx) {
            stmts.insert(0, stmt);
        }
    }

    fn insert_into_program(&self, transform_ctx: &TransformCtx<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(stmt) = self.get_var_statement(ctx) {
            // Delegate to `TopLevelStatements`
            transform_ctx.top_level_statements.insert_statement(stmt);
        }

        // Check stack is emptied
        let stack = self.stack.borrow();
        debug_assert!(stack.len() == 1);
        debug_assert!(stack.last().is_none());
    }

    fn get_var_statement(&self, ctx: &mut TraverseCtx<'a>) -> Option<Statement<'a>> {
        let mut stack = self.stack.borrow_mut();
        let declarators = stack.pop()?;
        debug_assert!(!declarators.is_empty());

        let stmt = Statement::VariableDeclaration(ctx.ast.alloc_variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            declarators,
            false,
        ));
        Some(stmt)
    }
}
