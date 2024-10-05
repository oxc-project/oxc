use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::{CompressOptions, CompressorPass};

/// Collapse variable declarations.
///
/// `var a; var b = 1; var c = 2` => `var a, b = 1; c = 2`
/// <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/CollapseVariableDeclarations.java>
pub struct CollapseVariableDeclarations {
    options: CompressOptions,

    changed: bool,
}

impl<'a> CompressorPass<'a> for CollapseVariableDeclarations {
    fn changed(&self) -> bool {
        self.changed
    }

    fn build(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.changed = false;
        oxc_traverse::walk_program(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for CollapseVariableDeclarations {
    fn enter_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.join_vars(stmts, ctx);
    }
}

impl<'a> CollapseVariableDeclarations {
    pub fn new(options: CompressOptions) -> Self {
        Self { options, changed: false }
    }

    /// Join consecutive var statements
    fn join_vars(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        if self.options.join_vars {
            return;
        }
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
                // Do not join `require` calls for cjs-module-lexer.
                if cur_decl
                    .declarations
                    .first()
                    .and_then(|d| d.init.as_ref())
                    .is_some_and(Expression::is_require_call)
                {
                    break;
                }
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
        let mut new_stmts = ctx.ast.vec_with_capacity(stmts.len() - capacity);
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
        self.changed = true;
    }
}

/// <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/CollapseVariableDeclarationsTest.java>
#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::{tester, CompressOptions};

    fn test(source_text: &str, expected: &str) {
        let allocator = Allocator::default();
        let mut pass = super::CollapseVariableDeclarations::new(CompressOptions::default());
        tester::test(&allocator, source_text, expected, &mut pass);
    }

    fn test_same(source_text: &str) {
        test(source_text, source_text);
    }

    #[test]
    fn cjs() {
        // Do not join `require` calls for cjs-module-lexer.
        test_same(
            "
    Object.defineProperty(exports, '__esModule', { value: true });
    var compilerDom = require('@vue/compiler-dom');
    var runtimeDom = require('@vue/runtime-dom');
    var shared = require('@vue/shared');
    ",
        );
    }
}
