use crate::{test, test_same};

#[test]
fn test_for_variable_declaration() {
    test(
        "function _() { var x; for (var i = 0; i < 10; i++) console.log(i) }",
        "function _() { for (var x, i = 0; i < 10; i++) console.log(i) }",
    );
    test(
        "function _() { var x = 1; for (var i = 0; i < 10; i++) console.log(i) }",
        "function _() { for (var x = 1, i = 0; i < 10; i++) console.log(i) }",
    );
    // this is fine because `let j` inside the block cannot be referenced from `var i = j`
    test(
        "function _() { var x = function () { return console.log(j), 1 }; for (var i = 0; i < 10; i++) { let j = k; console.log(i, j, j) } }",
        "function _() { for (var x = function () { return console.log(j), 1 }, i = 0; i < 10; i++) { let j = k; console.log(i, j, j) } }",
    );
    // this is fine because `let j` inside the block cannot be referenced from `var i = j`
    test(
        "function _() { var x = j; for (var i = 0; i < 10; i++) { let j = k; console.log(i, j, j) } }",
        "function _() { for (var x = j, i = 0; i < 10; i++) { let j = k; console.log(i, j, j) } }",
    );
}

#[test]
fn test_for_continue_in_for() {
    test("for( a of b ){ if(c) { continue; } d() }", "for ( a of b ) c || d();");
    test("for( a in b ){ if(c) { continue; } d() }", "for ( a in b ) c || d();");
    test("for( ; ; ){ if(c) { continue; } d() }", "for ( ; ; ) c || d();");

    test("for( a of b ){ c(); continue; }", "for ( a of b ) c();");
    test("for( a in b ){ c(); continue; }", "for ( a in b ) c();");
    test("for( ; ; ){ c(); continue; }", "for ( ; ; ) c();");
}

#[test]
fn test_array_variable_destruction() {
    test_same("let [] = []");
    test("let [a] = [1]", "let a=1");
    test("let [a, b, c, d] = [1, 2, 3, 4]", "let a = 1, b = 2, c = 3, d = 4");
    test("let [a, b, c, d] = [1, 2, 3]", "let a = 1, b = 2, c = 3, d");
    test("let [a, b, c = 2, d] = [1]", "let a = 1, b, c = 2, d");
    test("let [a, b, c] = [1, 2, 3, 4]", "let a = 1, b = 2, c = 3, [] = [4]");
    test("let [a, b, c = 2] = [1, 2, 3, 4]", "let a = 1, b = 2, c = 3, [] = [4]");
    test("let [a, b, c = 3] = [1, 2]", "let a = 1, b = 2, c = 3");
    test("let [a, b] = [1, 2, 3]", "let a = 1, b = 2, [] = [3]");
    test("let [a] = [123, 2222, 2222]", "let a = 123, [] = [2222, 2222]");
    test_same("let [a = 1] = [void foo()]");
    // spread
    test("let [...a] = [...b]", "let a = [...b]");
    test("let [a, a, ...d] = []", "let a, a, d = []");
    test("let [a, ...d] = []", "let a, d = []");
    test("let [a, ...d] = [1, ...f]", "let a = 1, d = [...f]");
    test("let [a, ...d] = [1, foo]", "let a = 1, d = [foo] ");
    test("let [a, b, c, ...d] = [1, 2, ...foo]", "let a = 1, b = 2, [c, ...d] = [...foo]");
    test("let [a, b, ...c] = [1, 2, 3, ...foo]", "let a = 1, b = 2, c = [3, ...foo]");
    test("let [a, b] = [...c, ...d]", "let [a, b] = [...c, ...d]");
    test("let [a, b] = [...c, c, d]", "let [a,b] = [...c, c, d]");
    // defaults
    test("let [a = 1] = []", "let a = 1");
    test("let [a = 1] = [void 0]", "let a = 1");
    test("let [a = 1] = [null]", "let a = null");
    test_same("let [a = 1] = [foo]");
    test("let [a = foo] = [2]", "let a = 2");
    test("let [a = foo] = [,]", "let a = foo");
    // holes
    test("let [, , , ] = [, , , ]", "");
    test("let [, , ] = [1, 2]", "");
    test("let [a, , c, d] = [, 3, , 4]", "let a, c, d = 4");
    test("let [a, , c, d] = [void 0, e, null, f]", "let a, [] = [e], c = null, d = f");
    test("let [a, , c, d] = [1, 2, 3, 4]", "let a = 1, c = 3, d = 4");
    test("let [ , , a] = [1, 2, 3, 4]", "let a = 3, [] = [4]");
    test("let [ , , ...t] = [1, 2, 3, 4]", "let t = [3, 4]");
    test("let [ , , ...t] = [1, ...a, 2, , 4]", "let [, ...t] = [...a, 2, , 4]");
    test("let [a, , b] = [, , , ]", "let a, b");
    test("const [a, , b] = [, , , ]", "const a = void 0, b = void 0;");
    // nested
    test("let [a, [b, c]] = [1, [2, 3]]", "let a = 1, b = 2, c = 3");
    test("let [a, [b, [c, d]]] = [1, ...[2, 3]]", "let a = 1, [[b, [c, d]]] = [...[2, 3]]");
    test("let [a, [b, [c, ]]] = [1, [...2, 3]]", "let a = 1, [b, [c]] = [...2, 3]");
    test("let [a, [b, [c, ]]] = [1, [2, [...3]]]", "let a = 1, b = 2, [c] = [...3];");
    // self reference
    test("let [a] = [a]", "let a = a");
    test("let [a, b] = [b, a]", "let b = b");
    // can't access lexical declaration 'b' before initialization
    test("let [a, b] = [b, a]", "let b = b");
    test("let [a, ...b] = [b, a]", "let b = [b]");
    test_same("let [a, ...b] = [...b, a]");
    // SyntaxError: redeclaration of let a
    test("let [a, b] = [1, 2], [a, b] = [b, a]", "let a = 1, b = 2, a = 2, b = 2");
    test("let [a, b] = [b, a], [a, b] = [b, a]", "let a = b, b = a, a = b, b = a");
    // const
    test("const [[x, y, z] = [4, 5, 6]] = []", "const x = 4, y = 5, z = 6;");
    test("const [a, ...d] = []", "const a = void 0, d = [];");
    test("const [a] = []", "const a = void 0");
    // vars
    test("var [a] = [a]", "var a = a");
    test("var [...a] = [b, c]", "var a = [b, c]");
    test_same("var [a, b] = [1, ...[2, 3]]");
    test_same("var [a, b] = [c, ...[d, e]]");
    test_same("var [ , , ...t] = [1, ...a, 2, , 4]");
    test("var [a, ...b] = [3, 4, 5]", "var a = 3, b = [4, 5]");
    test("var [c, ...d] = [6]", "var c = 6, d = []");
    test("var [c, d] = [6]", "var c = 6, d = void 0");
    test("var [a, b] = [1, 2]", "var a = 1, b = 2");
    test("var [a, b] = [d, c]", "var a = d, b = c");
    test_same("var [a, b] = [!d, !a]");
    test("var [a, ...b] = [1, 2]", "var a = 1, b = [2]");
    test("var [a, b] = [1, 2], [a, b] = [b, a]", "var a = 2, b = 1");
    test_same("var [a, b] = [b, a]");
    test_same("var [a, b] = [b, a], [a, b] = [b, a]");
}
