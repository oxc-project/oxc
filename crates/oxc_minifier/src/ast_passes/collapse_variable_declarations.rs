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

    #[test]
    #[ignore]
    fn test_collapsing() {
        // Basic collapsing
        test("var a;var b;", "var a,b;");

        // With initial values
        test("var a = 1;var b = 1;", "var a=1,b=1;");

        // Already collapsed
        test_same("var a, b;");

        // Already collapsed with values
        test_same("var a = 1, b = 1;");

        // Some already collapsed
        test("var a;var b, c;var d;", "var a,b,c,d;");

        // Some already collapsed with values
        test("var a = 1;var b = 2, c = 3;var d = 4;", "var a=1,b=2,c=3,d=4;");

        test(
            "var x = 2; foo(x); x = 3; x = 1; var y = 2; var z = 4; x = 5",
            "var x = 2; foo(x); x = 3; x = 1; var y = 2, z = 4; x = 5",
        );
    }

    #[test]
    fn test_issue820() {
        // Don't redeclare function parameters, this is incompatible with
        // strict mode.
        test_same("function f(a){ var b=1; a=2; var c; }");
    }

    #[test]
    fn test_if_else_var_declarations() {
        test_same("if (x) var a = 1; else var b = 2;");
    }

    #[test]
    fn test_aggressive_redeclaration_in_for() {
        test_same("for(var x = 1; x = 2; x = 3) {x = 4}");
        test_same("for(var x = 1; y = 2; z = 3) {var a = 4}");
        test_same("var x; for(x = 1; x = 2; z = 3) {x = 4}");
    }

    #[test]
    #[ignore]
    fn test_issue397() {
        test_same("var x; x = 5; var z = 7;");
        test("var x; var y = 3; x = 5;", "var x, y = 3; x = 5;");
        test("var a = 1; var x; var y = 3; x = 5;", "var a = 1, x, y = 3; x = 5;");
        test("var x; var y = 3; x = 5; var z = 7;", "var x, y = 3; x = 5; var z = 7;");
    }

    #[test]
    fn test_arguments_assignment() {
        test_same("function f() {arguments = 1;}");
    }

    // ES6 Tests
    #[test]
    #[ignore]
    fn test_collapsing_let_const() {
        // Basic collapsing
        test("let a;let b;", "let a,b;");

        // With initial values
        test("const a = 1;const b = 1;", "const a=1,b=1;");

        // Already collapsed
        test_same("let a, b;");

        // Already collapsed with values
        test_same("let a = 1, b = 1;");

        // Some already collapsed
        test("let a;let b, c;let d;", "let a,b,c,d;");

        // Some already collapsed with values
        test("let a = 1;let b = 2, c = 3;let d = 4;", "let a=1,b=2,c=3,d=4;");

        // Different variable types
        test_same("let a = 1; const b = 2;");
    }

    #[test]
    fn test_if_else_var_declarations_let() {
        test_same("if (x) { let a = 1; } else { let b = 2; }");
    }

    #[test]
    fn test_aggressive_redeclaration_of_let_in_for() {
        test_same("for(let x = 1; x = 2; x = 3) {x = 4}");
        test_same("for(let x = 1; y = 2; z = 3) {let a = 4}");
        test_same("let x; for(x = 1; x = 2; z = 3) {x = 4}");
    }

    #[test]
    #[ignore]
    fn test_redeclaration_let_in_function() {
        test(
            "function f() { let x = 1; let y = 2; let z = 3; x + y + z; }",
            "function f() { let x = 1, y = 2, z = 3; x + y + z; } ",
        );

        // recognize local scope version of x
        test(
            "var x = 1; function f() { let x = 1; let y = 2; x + y; }",
            "var x = 1; function f() { let x = 1, y = 2; x + y } ",
        );

        // do not redeclare function parameters
        // incompatible with strict mode
        test_same("function f(x) { let y = 3; x = 4; x + y; }");
    }

    #[test]
    #[ignore]
    fn test_arrow_function() {
        test("() => {let x = 1; let y = 2; x + y; }", "() => {let x = 1, y = 2; x + y; }");

        // do not redeclare function parameters
        // incompatible with strict mode
        test_same("(x) => {x = 4; let y = 2; x + y; }");
    }

    #[test]
    fn test_uncollapsable_declarations() {
        test_same("let x = 1; var y = 2; const z = 3");
        test_same("let x = 1; var y = 2; let z = 3;");
    }

    #[test]
    #[ignore]
    fn test_mixed_declaration_types() {
        // lets, vars, const declarations consecutive
        test("let x = 1; let z = 3; var y = 2;", "let x = 1, z = 3; var y = 2;");
        test("let x = 1; let y = 2; var z = 3; var a = 4;", "let x = 1, y = 2; var z = 3, a = 4");
    }
}
