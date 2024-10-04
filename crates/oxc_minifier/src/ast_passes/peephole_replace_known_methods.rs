use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::CompressorPass;

/// Minimize With Known Methods
/// <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeReplaceKnownMethods.java>
pub struct PeepholeReplaceKnownMethods {
    changed: bool,
}

impl<'a> CompressorPass<'a> for PeepholeReplaceKnownMethods {
    fn changed(&self) -> bool {
        self.changed
    }

    fn build(&mut self, _program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.changed = false;
        // oxc_traverse::walk_program(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for PeepholeReplaceKnownMethods {}

impl PeepholeReplaceKnownMethods {
    pub fn new() -> Self {
        Self { changed: false }
    }
}

/// <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeReplaceKnownMethodsTest.java>
#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::tester;

    fn test(source_text: &str, positive: &str) {
        let allocator = Allocator::default();
        let mut pass = super::PeepholeReplaceKnownMethods::new();
        tester::test(&allocator, source_text, positive, &mut pass);
    }

    fn test_same(source_text: &str) {
        test(source_text, source_text);
    }

    fn fold_same(js: &str) {
        test_same(js);
    }

    fn fold(js: &str, expected: &str) {
        test(js, expected);
    }

    #[test]
    #[ignore]
    fn test_string_index_of() {
        fold("x = 'abcdef'.indexOf('g')", "x = -1");
        fold("x = 'abcdef'.indexOf('b')", "x = 1");
        fold("x = 'abcdefbe'.indexOf('b', 2)", "x = 6");
        fold("x = 'abcdef'.indexOf('bcd')", "x = 1");
        fold("x = 'abcdefsdfasdfbcdassd'.indexOf('bcd', 4)", "x = 13");

        fold("x = 'abcdef'.lastIndexOf('b')", "x = 1");
        fold("x = 'abcdefbe'.lastIndexOf('b')", "x = 6");
        fold("x = 'abcdefbe'.lastIndexOf('b', 5)", "x = 1");

        // Both elements must be strings. Don't do anything if either one is not
        // string.
        fold("x = 'abc1def'.indexOf(1)", "x = 3");
        fold("x = 'abcNaNdef'.indexOf(NaN)", "x = 3");
        fold("x = 'abcundefineddef'.indexOf(undefined)", "x = 3");
        fold("x = 'abcnulldef'.indexOf(null)", "x = 3");
        fold("x = 'abctruedef'.indexOf(true)", "x = 3");

        // The following test case fails with JSC_PARSE_ERROR. Hence omitted.
        // fold_same("x = 1.indexOf('bcd');");
        fold_same("x = NaN.indexOf('bcd')");
        fold_same("x = undefined.indexOf('bcd')");
        fold_same("x = null.indexOf('bcd')");
        fold_same("x = true.indexOf('bcd')");
        fold_same("x = false.indexOf('bcd')");

        // Avoid dealing with regex or other types.
        fold_same("x = 'abcdef'.indexOf(/b./)");
        fold_same("x = 'abcdef'.indexOf({a:2})");
        fold_same("x = 'abcdef'.indexOf([1,2])");

        // Template Strings
        fold_same("x = `abcdef`.indexOf('b')");
        fold_same("x = `Hello ${name}`.indexOf('a')");
        fold_same("x = tag `Hello ${name}`.indexOf('a')");
    }
}
