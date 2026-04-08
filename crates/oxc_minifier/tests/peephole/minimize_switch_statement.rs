use crate::{test, test_same};

#[expect(clippy::literal_string_with_formatting_args)]
#[test]
fn minimize_switch() {
    test("switch(a()){}", "a()");
    test("switch(a){default: }", "a;");
    test("switch(a){default: break;}", "a;");
    test("switch(a){default: var b; break;}", "a;var b");
    test("switch(a){default: b()}", "a, b();");
    test("switch(a){default: b(); return;}", "a, b(); return");

    test("switch(a){case 1: break;}", "a;");
    test("switch(a){case 1: b();}", "a === 1 && b()");
    test("switch(a){case 1: b();break; }", "a === 1 && b()");
    test("switch(a){case 1: b();return; }", "if (a === 1) { b(); return; }");

    test("switch(a){default: case 1: }", "a");
    test("switch(a){case 1: default: }", "a");
    test_same("switch(a){case 1: default: break; case 2: b()}");
    test_same("switch(a){case 1: b(); default: c()}");
    test_same("switch(a){case 1: b(); default: break; case 2: c()}");
    test_same("switch(a){case 1: b(); case 2: break; case 3: c()}");
    test(
        "switch(a){case 1: b(); break; case 2: c();break;}",
        "switch(a){case 1: b(); break; case 2: c();}",
    );
    test_same("switch(a){case 1: b(); case 2: b();}");
    test("switch(a){case 1: var c=2; break;}", "if (a === 1) var c = 2");
    test("switch(a){case 1: case 2: default: b(); break;}", "a, b()");

    test("switch(a){default: break; case 1: break;}", "a");
    test("switch(a){default: b();break;case 1: c();break;}", "a === 1 ? c() : b()");
    test("switch(a){default: {b();break;} case 1: {c();break;}}", "a === 1 ? c() : b()");

    test_same("switch(a){case b(): default:}");
    test("switch(a){case 2: case 1: break; default: break;}", "a;");
    test("switch(a){case 3: b(); break; case 2: break;}", "a === 3 && b()");
    test("switch(a){case 3: b(); case 2: break;}", "a === 3 && b()");
    test("switch(a){case 3: b(); case 2: c(); break;}", "switch(a){case 3: b(); case 2: c();}");
    test("switch(a){case 3: b(); case 2: case 1: break;}", "a === 3 && b()");
    test("switch(a){case 3: b(); case 2: case 1: }", "a === 3 && b()");
    test("switch(a){case 3: if (b) break }", "a === 3 && b");
    test("switch(a){case 3: { if (b) break } }", "a === 3 && b");
    test("switch(a){case 3: { if(b) {c()} else {break;} }}", "a === 3 && b && c()");
    test(
        "switch(a){case 3: { if(b) {c(); break;} else { d(); break;} }}",
        "a === 3 && (b ? c() : d())",
    );
    test("switch(a){case 3: { for (;;) break } }", "if(a === 3) for (;;) break;");
    test("switch(a){case 3: { for (b of c) break } }", "if (a === 3) for (b of c) break;");
    test_same("switch(a){case 3: with(b) break}");
    test("switch(a){case 3: while(!0) break}", "if (a === 3) for (;;) break;");

    test(
        "switch(a){case 1: c(); case 2: default: b();break;}",
        "switch(a){case 1: c(); default: b();}",
    );
    test("var x=1;switch(x){case 1: var y;}", "var y;");
    test("function f(){switch(a){case 1: return;}}", "function f() {a;}");
    test("switch(a()) { default: {let y;} }", "a();{let y;}");
    test(
        "function f(){switch('x'){case 'x': var x = 1;break; case 'y': break; }}",
        "function f(){var x = 1;}",
    );
    test("switch(a){default: if(a) {break;}c();}", "switch(a){default: if(a) break;c();}");
    test("switch(a){case 1: if(a) {b();}c();}", "a === 1 && (a && b(), c())");
    test("switch ('\\v') {case '\\u000B': foo();}", "foo()");

    test("x: switch(a){case 1: break x;}", "x: if (a === 1) break x;");
    test_same("x: switch(a){case 2: break x; case 1: break x;}");
    test("x: switch(2){case 2: f(); break outer; }", "x: {f(); break outer;}");
    test(
        "x: switch(x){case 2: f(); for (;;){break outer;}}",
        "x: if(x===2) for(f();;) break outer",
    );
    test("x: switch(a){case 2: if(b) { break outer; } }", "x: if(a===2 && b) break outer;");

    test(
        "switch('r'){case 'r': a();break; case 'r': var x=0;break;}",
        "switch('r'){case 'r': a();break; case 'r': var x=0;}",
    );
    test(
        "switch('r'){case 'r': a();break; case 'r': bar();break;}",
        "switch('r'){case 'r': a();break; case 'r': bar()}",
    );
    test_same("switch(2) {default: a; case 1: b()}");
    test("switch(1) {case 1: a();break; default: b();}", "a()");
    test_same("switch('e') {case 'e': case 'f': a();}");
    test(
        "switch('a') {case 'a': a();break; case 'b': b();break;}",
        "switch('a') {case 'a': a();break; case 'b': b();}",
    );
    test(
        "switch('c') {case 'a': a();break; case 'b': b();break;}",
        "switch('c') {case 'a': a();break; case 'b': b();}",
    );
    test(
        "switch(1) {case 1: a();break; case 2: bar();break;}",
        "switch(1) {case 1: a();break; case 2: bar();}",
    );
    test_same("switch('f') {case 'f': a(); case 'b': b();}");
    test_same("switch('f') {case 'f': if (a() > 0) {b();break;} c(); case 'd': f();}");
    test(
        "switch('f') {case 'b': bar();break; case x: x();break; case 'f': f();break;}",
        "switch('f') {case 'b': bar();break; case x: x();break; case 'f': f();}",
    );
    test(
        "switch(1){case 1: case 2: {break;} case 3: case 4: default: b(); break;}",
        "switch(1){case 1: case 2: break; default: b();}",
    );
    test("switch ('d') {case 'foo': foo();break; default: bar();break;}", "bar()");
    test(
        "switch(0){case NaN: foobar();break;case -0.0: foo();break; case 2: bar();break;}",
        "switch(0){case NaN: foobar();break;case -0.0: foo();break; case 2: bar();}",
    );
    test("let x = 1; switch('x') { case 'x': let x = 2; break;}", "let x = 1; { let x = 2 }");
    test("switch(1){case 2: var x=0;}", "if (0) var x;");
    test(
        "switch(b){case 2: switch(a){case 2: a();break;case 3: foo();break;}}",
        "if (b === 2) switch (a) {case 2: a(); break;	case 3: foo();}",
    );
    test("switch(b){case 2: switch(a){case 2: foo()}}", "b === 2 && a === 2 && foo();");
}
