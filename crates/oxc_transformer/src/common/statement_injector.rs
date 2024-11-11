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

use oxc_allocator::{Address, GetAddress, Vec as ArenaVec};

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
        statements: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.ctx.statement_injector.insert_into_statements(statements, ctx);
    }
}

#[derive(Debug)]
enum Direction {
    Before,
    After,
}

#[derive(Debug)]
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
}

// Insertion methods.
//
// Each of these functions is split into 2 parts:
//
// 1. Public outer function which is generic over any `GetAddress`.
// 2. Private inner function which is non-generic and takes `Address`.
//
// Outer functions are marked `#[inline]`, as `GetAddress::address` is generally only 1 or 2 instructions.
// The non-trivial inner functions are not marked `#[inline]` - compiler can decide whether to inline or not.
impl<'a> StatementInjectorStore<'a> {
    /// Add a statement to be inserted immediately before the target statement.
    #[expect(dead_code)]
    #[inline]
    pub fn insert_before<A: GetAddress>(&self, target: &A, stmt: Statement<'a>) {
        self.insert_before_address(target.address(), stmt);
    }

    fn insert_before_address(&self, target: Address, stmt: Statement<'a>) {
        let mut insertions = self.insertions.borrow_mut();
        let adjacent_stmts = insertions.entry(target).or_default();
        let index = adjacent_stmts
            .iter()
            .position(|s| matches!(s.direction, Direction::After))
            .unwrap_or(adjacent_stmts.len());
        adjacent_stmts.insert(index, AdjacentStatement { stmt, direction: Direction::Before });
    }

    /// Add a statement to be inserted immediately after the target statement.
    #[inline]
    pub fn insert_after<A: GetAddress>(&self, target: &A, stmt: Statement<'a>) {
        self.insert_after_address(target.address(), stmt);
    }

    fn insert_after_address(&self, target: Address, stmt: Statement<'a>) {
        let mut insertions = self.insertions.borrow_mut();
        let adjacent_stmts = insertions.entry(target).or_default();
        adjacent_stmts.push(AdjacentStatement { stmt, direction: Direction::After });
    }

    /// Add multiple statements to be inserted immediately before the target statement.
    #[inline]
    pub fn insert_many_before<A, S>(&self, target: &A, stmts: S)
    where
        A: GetAddress,
        S: IntoIterator<Item = Statement<'a>>,
    {
        self.insert_many_before_address(target.address(), stmts);
    }

    fn insert_many_before_address<S>(&self, target: Address, stmts: S)
    where
        S: IntoIterator<Item = Statement<'a>>,
    {
        let mut insertions = self.insertions.borrow_mut();
        let adjacent_stmts = insertions.entry(target).or_default();
        adjacent_stmts.splice(
            0..0,
            stmts.into_iter().map(|stmt| AdjacentStatement { stmt, direction: Direction::Before }),
        );
    }

    /// Add multiple statements to be inserted immediately after the target statement.
    #[inline]
    pub fn insert_many_after<A, S>(&self, target: &A, stmts: S)
    where
        A: GetAddress,
        S: IntoIterator<Item = Statement<'a>>,
    {
        self.insert_many_after_address(target.address(), stmts);
    }

    fn insert_many_after_address<S>(&self, target: Address, stmts: S)
    where
        S: IntoIterator<Item = Statement<'a>>,
    {
        let mut insertions = self.insertions.borrow_mut();
        let adjacent_stmts = insertions.entry(target).or_default();
        adjacent_stmts.extend(
            stmts.into_iter().map(|stmt| AdjacentStatement { stmt, direction: Direction::After }),
        );
    }
}

// Internal methods
impl<'a> StatementInjectorStore<'a> {
    /// Insert statements immediately before / after the target statement.
    fn insert_into_statements(
        &self,
        statements: &mut ArenaVec<'a, Statement<'a>>,
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
