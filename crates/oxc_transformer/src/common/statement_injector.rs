//! Utility transform to add new statements before or after the specified statement.
//!
//! `StatementInjectorStore` contains a `FxHashMap<Address, Vec<AdjacentStatement>>`. It is stored on `TransformCtx`.
//!
//! `StatementInjector` transform inserts new statements before or after a statement which is determined by the address of the statement.
//!
//! Other transforms can add statements to the store with following methods:
//!
//! ```rs
//! self.ctx.statement_injector.insert_before(address, statement);
//! self.ctx.statement_injector.insert_after(address, statement);
//! self.ctx.statement_injector.insert_many_after(address, statements);
//! ```

use std::cell::RefCell;

use oxc_allocator::{Address, GetAddress, Vec as AVec};

use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};
use rustc_hash::FxHashMap;

use crate::TransformCtx;

/// Transform that inserts any statements which have been requested insertion via `StatementInjectorStore`
pub struct StatementInjector<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> StatementInjector<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for StatementInjector<'a, 'ctx> {
    fn exit_statements(
        &mut self,
        statements: &mut AVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.ctx.statement_injector.insert_into_statements(statements, ctx);
    }
}

enum Direction {
    Before,
    After,
}

struct AdjacentStatement<'a> {
    stmt: Statement<'a>,
    direction: Direction,
}

/// Store for statements to be added to the statements.
pub struct StatementInjectorStore<'a> {
    insertions: RefCell<FxHashMap<Address, Vec<AdjacentStatement<'a>>>>,
}

// Public methods
impl<'a> StatementInjectorStore<'a> {
    /// Create new `StatementInjectorStore`.
    pub fn new() -> Self {
        Self { insertions: RefCell::new(FxHashMap::default()) }
    }

    /// Add a statement to be inserted immediately before the target statement.
    #[expect(dead_code)]
    pub fn insert_before(&self, target: Address, stmt: Statement<'a>) {
        let mut insertions = self.insertions.borrow_mut();
        let adjacent_stmts = insertions.entry(target).or_default();
        let index = adjacent_stmts
            .iter()
            .position(|s| matches!(s.direction, Direction::After))
            .unwrap_or(adjacent_stmts.len());
        adjacent_stmts.insert(index, AdjacentStatement { stmt, direction: Direction::Before });
    }

    /// Add a statement to be inserted immediately after the target statement.
    #[expect(dead_code)]
    pub fn insert_after(&self, target: Address, stmt: Statement<'a>) {
        let mut insertions = self.insertions.borrow_mut();
        let adjacent_stmts = insertions.entry(target).or_default();
        adjacent_stmts.push(AdjacentStatement { stmt, direction: Direction::After });
    }

    /// Add multiple statements to be inserted immediately after the target statement.
    pub fn insert_many_after(&self, target: Address, stmts: Vec<Statement<'a>>) {
        let mut insertions = self.insertions.borrow_mut();
        let adjacent_stmts = insertions.entry(target).or_default();
        adjacent_stmts.extend(
            stmts.into_iter().map(|stmt| AdjacentStatement { stmt, direction: Direction::After }),
        );
    }

    /// Insert statements immediately before / after the target statement.
    pub(self) fn insert_into_statements(
        &self,
        statements: &mut AVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut insertions = self.insertions.borrow_mut();
        if insertions.is_empty() {
            return;
        }

        let new_statement_count = statements
            .iter()
            .filter_map(|s| insertions.get(&s.address()).map(Vec::len))
            .sum::<usize>();
        if new_statement_count == 0 {
            return;
        }

        let mut new_statements = ctx.ast.vec_with_capacity(statements.len() + new_statement_count);

        for stmt in statements.drain(..) {
            if let Some(mut adjacent_stmts) = insertions.remove(&stmt.address()) {
                let first_after_stmt_index = adjacent_stmts
                    .iter()
                    .position(|s| matches!(s.direction, Direction::After))
                    .unwrap_or(adjacent_stmts.len());
                if first_after_stmt_index != 0 {
                    let right = adjacent_stmts.split_off(first_after_stmt_index);
                    new_statements.extend(adjacent_stmts.into_iter().map(|s| s.stmt));
                    new_statements.push(stmt);
                    new_statements.extend(right.into_iter().map(|s| s.stmt));
                } else {
                    new_statements.push(stmt);
                    new_statements.extend(adjacent_stmts.into_iter().map(|s| s.stmt));
                }
            } else {
                new_statements.push(stmt);
            }
        }

        *statements = new_statements;
    }
}
