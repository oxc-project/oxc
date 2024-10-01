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
use oxc_syntax::symbol::SymbolId;
use oxc_traverse::{Traverse, TraverseCtx};

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
    fn exit_program(&mut self, _program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(stmt) = self.get_var_statement(ctx) {
            // Delegate to `TopLevelStatements`
            self.ctx.top_level_statements.insert_statement(stmt);
        }

        let declarators = self.ctx.var_declarations.declarators.borrow();
        debug_assert!(declarators.len() == 1);
        debug_assert!(declarators.last().is_none());
    }

    fn enter_statements(
        &mut self,
        _stmts: &mut Vec<'a, Statement<'a>>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        let mut declarators = self.ctx.var_declarations.declarators.borrow_mut();
        declarators.push(None);
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        if ctx.ancestors_depth() == 2 {
            // Top level. Handle in `exit_program` instead.
            // (depth 1 = None, depth 2 = Program)
            return;
        }

        if let Some(stmt) = self.get_var_statement(ctx) {
            stmts.insert(0, stmt);
        }
    }
}

impl<'a, 'ctx> VarDeclarations<'a, 'ctx> {
    fn get_var_statement(&mut self, ctx: &mut TraverseCtx<'a>) -> Option<Statement<'a>> {
        let mut declarators = self.ctx.var_declarations.declarators.borrow_mut();
        let declarators = declarators.pop()?;
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

/// Store for `VariableDeclarator`s to be added to enclosing statement block.
pub struct VarDeclarationsStore<'a> {
    declarators: RefCell<SparseStack<Vec<'a, VariableDeclarator<'a>>>>,
}

impl<'a> VarDeclarationsStore<'a> {
    pub fn new() -> Self {
        Self { declarators: RefCell::new(SparseStack::new()) }
    }
}

impl<'a> VarDeclarationsStore<'a> {
    /// Add a `VariableDeclarator` to be inserted at top of current enclosing statement block,
    /// given `name` and `symbol_id`.
    pub fn insert(
        &self,
        name: Atom<'a>,
        symbol_id: SymbolId,
        init: Option<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ident = BindingIdentifier::new_with_symbol_id(SPAN, name, symbol_id);
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
        let mut declarators = self.declarators.borrow_mut();
        declarators.last_mut_or_init(|| ctx.ast.vec()).push(declarator);
    }
}
