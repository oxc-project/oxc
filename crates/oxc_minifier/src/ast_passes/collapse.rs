use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
use oxc_traverse::{Traverse, TraverseCtx};

use crate::{CompressOptions, CompressorPass};

/// Collapse variable declarations (TODO: and assignments).
///
/// `var a; var b = 1; var c = 2` => `var a, b = 1; c = 2`
/// TODO: `a = null; b = null;` => `a = b = null`
pub struct Collapse<'a> {
    ast: AstBuilder<'a>,
    options: CompressOptions,
}

impl<'a> CompressorPass<'a> for Collapse<'a> {}

impl<'a> Traverse<'a> for Collapse<'a> {
    fn enter_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, _ctx: &mut TraverseCtx<'a>) {
        if self.options.join_vars {
            self.join_vars(stmts);
        }
    }
}

impl<'a> Collapse<'a> {
    pub fn new(ast: AstBuilder<'a>, options: CompressOptions) -> Self {
        Self { ast, options }
    }

    /// Join consecutive var statements
    fn join_vars(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        // Collect all the consecutive ranges that contain joinable vars.
        // This is required because Rust prevents in-place vec mutation.
        let mut ranges = vec![];
        let mut range = 0..0;
        let mut i = 1usize;
        let mut capacity = 0usize;
        for window in stmts.windows(2) {
            let [prev, cur] = window else { unreachable!() };
            if let (
                Statement::VariableDeclaration(cur_decl),
                Statement::VariableDeclaration(prev_decl),
            ) = (cur, prev)
            {
                if cur_decl.kind == prev_decl.kind {
                    if i - 1 != range.end {
                        range.start = i - 1;
                    }
                    range.end = i + 1;
                }
            }
            if (range.end != i || i == stmts.len() - 1) && range.start < range.end {
                capacity += range.end - range.start - 1;
                ranges.push(range.clone());
                range = 0..0;
            }
            i += 1;
        }

        if ranges.is_empty() {
            return;
        }

        // Reconstruct the stmts array by joining consecutive ranges
        let mut new_stmts = self.ast.vec_with_capacity(stmts.len() - capacity);
        for (i, stmt) in stmts.drain(..).enumerate() {
            if i > 0 && ranges.iter().any(|range| range.contains(&(i - 1)) && range.contains(&i)) {
                if let Statement::VariableDeclaration(prev_decl) = new_stmts.last_mut().unwrap() {
                    if let Statement::VariableDeclaration(mut cur_decl) = stmt {
                        prev_decl.declarations.append(&mut cur_decl.declarations);
                    }
                }
            } else {
                new_stmts.push(stmt);
            }
        }
        *stmts = new_stmts;
    }
}
