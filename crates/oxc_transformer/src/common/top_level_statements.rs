//! Utility transform to add statements to top of program.
//!
//! `TopLevelStatementsStore` contains a `Vec<Statement>`. It is stored on `TransformCtx`.
//!
//! `TopLevelStatements` transform inserts those statements at top of program.
//!
//! Other transforms can add statements to the store with `TopLevelStatementsStore::insert_statement`:
//!
//! ```rs
//! self.ctx.top_level_statements.insert_statement(stmt);
//! ```

use std::cell::UnsafeCell;

use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

/// Transform that inserts any statements which have been requested insertion via `TopLevelStatementsStore`
/// to top of the program.
///
/// Insertions are made after any existing `import` statements.
///
/// Must run after all other transforms.
pub struct TopLevelStatements<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> TopLevelStatements<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for TopLevelStatements<'a, 'ctx> {
    fn exit_program(&mut self, program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.ctx.top_level_statements.insert_into_program(program);
    }
}

/// Store for statements to be added at top of program
pub struct TopLevelStatementsStore<'a>(UnsafeCell<TopLevelStatementsState<'a>>);

// Public methods
impl<'a> TopLevelStatementsStore<'a> {
    /// Create new `TopLevelStatementsStore`.
    pub fn new() -> Self {
        Self(UnsafeCell::new(TopLevelStatementsState::new()))
    }

    /// Add a statement to be inserted at top of program.
    pub fn insert_statement(&self, stmt: Statement<'a>) {
        // SAFETY: We only borrow state once during this function and borrow expires before exiting it
        let state = unsafe { &mut *self.0.get() };
        state.insert_statement(stmt);
    }

    /// Add statements to be inserted at top of program.
    pub fn insert_statements<I: IntoIterator<Item = Statement<'a>>>(&self, stmts: I) {
        // SAFETY: We only borrow state once during this function and borrow expires before exiting it
        let state = unsafe { &mut *self.0.get() };
        state.insert_statements(stmts);
    }
}

// Internal methods
impl<'a> TopLevelStatementsStore<'a> {
    /// Insert statements at top of program.
    fn insert_into_program(&self, program: &mut Program<'a>) {
        // SAFETY: We only borrow state once during this function and borrow expires before exiting it
        let state = unsafe { &mut *self.0.get() };
        state.insert_into_program(program);
    }
}

/// Store for statements to be added at top of program
struct TopLevelStatementsState<'a> {
    stmts: Vec<Statement<'a>>,
}

// Public methods
impl<'a> TopLevelStatementsState<'a> {
    /// Create new `TopLevelStatementsState`.
    pub fn new() -> Self {
        Self { stmts: vec![] }
    }

    /// Add a statement to be inserted at top of program.
    pub fn insert_statement(&mut self, stmt: Statement<'a>) {
        self.stmts.push(stmt);
    }

    /// Add statements to be inserted at top of program.
    pub fn insert_statements<I: IntoIterator<Item = Statement<'a>>>(&mut self, stmts: I) {
        self.stmts.extend(stmts);
    }
}

// Internal methods
impl<'a> TopLevelStatementsState<'a> {
    /// Insert statements at top of program.
    fn insert_into_program(&mut self, program: &mut Program<'a>) {
        if self.stmts.is_empty() {
            return;
        }

        // Insert statements after any existing `import` statements
        let index = program
            .body
            .iter()
            .rposition(|stmt| matches!(stmt, Statement::ImportDeclaration(_)))
            .map_or(0, |i| i + 1);

        program.body.splice(index..index, self.stmts.drain(..));
    }
}
