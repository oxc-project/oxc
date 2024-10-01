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

use std::cell::RefCell;

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
        let mut stmts = self.ctx.top_level_statements.stmts.borrow_mut();
        if stmts.is_empty() {
            return;
        }

        // Insert statements after any existing `import` statements
        let insert_index = if self.ctx.source_type.is_module() {
            find_insertion_index(&program.body)
        } else {
            // Scripts can't have `import` statements, so no need to search
            0
        };

        program.body.splice(insert_index..insert_index, stmts.drain(..));
    }
}

/// Find index to insert statements at.
///
/// We want to insert after any `import` statements.
///
/// We could search from the end of the file backwards until we hit an `import` statement, but in
/// a large file, that's a lot of statements to search through. So instead, search from the *start*
/// for first statement which is *not* an `import`.
/// Usually the correct insertion point is before that statement.
///
/// But there is one annoying Babel test that has a non-`import` statement followed by `import`s,
/// and it expects new statements to be inserted after the last of those `import`s.
/// `babel-plugin-transform-react-jsx/test/fixtures/autoImport/after-polyfills-2`
/// To pass that test, we search again if the first statement is not an `import`.
///
/// TODO(improve-on-babel): Insertion position is not important. We only do this to pass Babel's tests.
/// Remove this once we don't have to match Babel's output exactly, and just insert at the start.
fn find_insertion_index(stmts: &[Statement]) -> usize {
    let Some(first_stmt) = stmts.first() else {
        // No statements. Insert at start.
        return 0;
    };

    let search_start_index = if matches!(first_stmt, Statement::ImportDeclaration(_)) {
        // First statement is `import`. Search for more `import`s after this.
        1
    } else if !matches!(stmts.get(1), Some(Statement::ImportDeclaration(_))) {
        // Either there's only 1 statement (a non-`import`), or first 2 statements are both not `import`.
        // Insert at the start.
        return 0;
    } else {
        // Non-`import`, followed by `import`. Search for more `import`s after this.
        2
    };

    // Find first non-`import` after this
    return stmts[search_start_index..]
        .iter()
        .position(|stmt| !matches!(stmt, Statement::ImportDeclaration(_)))
        .map_or_else(|| stmts.len(), |index| search_start_index + index);
}

/// Store for statements to be added at top of program
pub struct TopLevelStatementsStore<'a> {
    stmts: RefCell<Vec<Statement<'a>>>,
}

impl<'a> TopLevelStatementsStore<'a> {
    pub fn new() -> Self {
        Self { stmts: RefCell::new(vec![]) }
    }
}

impl<'a> TopLevelStatementsStore<'a> {
    /// Add a statement to be inserted at top of program.
    pub fn insert_statement(&self, stmt: Statement<'a>) {
        self.stmts.borrow_mut().push(stmt);
    }

    /// Add statements to be inserted at top of program.
    pub fn insert_statements<I: IntoIterator<Item = Statement<'a>>>(&self, stmts: I) {
        self.stmts.borrow_mut().extend(stmts);
    }
}
