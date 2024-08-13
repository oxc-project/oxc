//! <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/CollapseVariableDeclarations.java>

use crate::CompressOptions;

fn test(source_text: &str, expected: &str) {
    let options = CompressOptions::all_true();
    crate::test(source_text, expected, options);
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
