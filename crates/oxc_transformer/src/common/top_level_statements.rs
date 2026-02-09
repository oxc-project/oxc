//! Utility transform to add statements to top of program.
//!
//! `TopLevelStatementsStore` contains a `Vec<Statement>`. It is stored on `TransformState`.
//!
//! Statements are inserted at top of program by `Common` transform's `exit_program`.
//!
//! Other transforms can add statements to the store with `TopLevelStatementsStore::insert_statement`:
//!
//! ```rs
//! ctx.state.top_level_statements.insert_statement(stmt);
//! ```

use oxc_ast::ast::*;

/// Store for statements to be added at top of program
pub struct TopLevelStatementsStore<'a> {
    stmts: Vec<Statement<'a>>,
}

// Public methods
impl<'a> TopLevelStatementsStore<'a> {
    /// Create new `TopLevelStatementsStore`.
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

// Internal methods - called by `Common` transform
impl<'a> TopLevelStatementsStore<'a> {
    /// Insert statements at top of program.
    pub(crate) fn insert_into_program(&mut self, program: &mut Program<'a>) {
        if self.stmts.is_empty() {
            return;
        }

        // Insert statements before the first non-import statement.
        let index = program
            .body
            .iter()
            .position(|stmt| !matches!(stmt, Statement::ImportDeclaration(_)))
            .unwrap_or(program.body.len());

        program.body.splice(index..index, self.stmts.drain(..));
    }
}
