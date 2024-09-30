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
        let index = if self.ctx.source_type.is_module() {
            find_insertion_index(&program.body)
        } else {
            // Scripts can't have `import` statements, so no need to search
            0
        };

        program.body.splice(index..index, stmts.drain(..));
    }
}

/// Find index to insert statements at.
///
/// We want to insert after any `import` statements.
///
/// We could search from the end of the file backwards until we hit an `import` statement, but in
/// a large file, that's a lot of statements to search through. So instead, search from the *start*
/// for first statement which is *not* an `import`.
/// Usually before that statement is the correct insertion point.
///
/// But there is one annoying Babel test that has a non-`import` statement followed by `import`s,
/// and it expects new statements to be inserted after the `import`.
/// `babel-plugin-transform-react-jsx/test/fixtures/autoImport/after-polyfills-2`
/// To pass that test, we search again if the first statement is not an `import`.
///
/// TODO(improve-on-babel): Insertion position is not important. We only do this to pass Babel's tests.
/// Remove this once we don't have to match Babel's output exactly, and just insert at the start.
fn find_insertion_index(stmts: &[Statement]) -> usize {
    let mut iter = stmts
        .iter()
        .enumerate()
        .filter(|(_, stmt)| !matches!(stmt, Statement::ImportDeclaration(_)))
        .map(|(index, _)| index);
    let Some(first_non_import) = iter.next() else {
        // All statements are `import`s, or empty file. Insert after them all.
        return stmts.len();
    };

    if first_non_import != 0 {
        // File starts with import statements. Insert after them.
        return first_non_import;
    }

    if stmts.len() == 1 {
        // Only 1 statement, and it's not an `import`. Insert before it.
        return 0;
    }

    // First statement is not `import`. Check if there are imports after it.
    // This is purely to pass this Babel test:
    // babel-plugin-transform-react-jsx/test/fixtures/autoImport/after-polyfills-2
    if let Some(second_non_import) = iter.next() {
        if second_non_import == 1 {
            // First 2 statements are not `import`s
            return 0;
        }
        second_non_import
    } else {
        // All statements after the first one are `import`s. Insert after them.
        stmts.len()
    }
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
