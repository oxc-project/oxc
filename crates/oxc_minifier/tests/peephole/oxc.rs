use oxc_minifier::CompressOptions;

/// Oxc Integration Tests
use crate::{test, test_options, test_same};

#[track_caller]
fn test_unused(source_text: &str, expected: &str) {
    test_options(source_text, expected, &CompressOptions::default());
}

#[test]
fn integration() {
    test(
        "require('./index.js')(function (e, os) {
    if (e) return console.log(e)
    return console.log(JSON.stringify(os))
    })",
        r#"require("./index.js")(function(e, os) {
    return e ? console.log(e) : console.log(JSON.stringify(os));
    });"#,
    );

    test(
        "if (!(foo instanceof Var) || open) {
          arg0 = null;
        } else if (que || !(foo && bar)) {
          if (baz()) arg0 = null;
        }",
        "(!(foo instanceof Var) || open || (que || !(foo && bar)) && baz()) && (arg0 = null);",
    );

    test(
        "function foo() {
          if (value === null || Array.isArray(value))
            return undefined;
          return isTimeDisabled === null || isTimeDisabled === void 0 ? void 0 : isTimeDisabled(value);
    }",
    "function foo() {
        if (!(value === null || Array.isArray(value))) return isTimeDisabled == null ? void 0 : isTimeDisabled(value);
    }");

    test_same(
        "a && (b && (c && (d && (e && (f && (g && (h && i && j && k && l && m && n && o && p && q && r && s && t && u && v && w && x && y && z)))))))",
    );

    test(
        "if (((() => console.log('effect'))(), true)) {
         } else {
           var c = 1;
           for (var c; unknownGlobal && true; unknownGlobal && true) var d;
         }
         console.log(c, d);
        ",
        "if (console.log('effect'), !1) var c, c, d;
        console.log(c, d);
        ",
    );

    test(
        "v = KEY === 'delete' ? function () {
          return 1;
        } : KEY === 'has' ? function has () {
          return 1;
        } : function set () {
          return 2;
        };
        ",
        "v = KEY === 'delete' || KEY === 'has' ? function () { return 1 } : function () { return 2 }",
    );
}

#[test]
fn fold() {
    test("var x = (-0).toString()", "var x = '0'");
    test("var x = (-0).toString(36)", "var x = '0'");
    test("var x = (-123).toString()", "var x = '-123'");
    test("var x = (-Infinity).toString()", "var x = '-Infinity'");
    test("var x = (-1000000).toString(36)", "var x = (-1e6).toString(36)");
}

#[test] // https://github.com/oxc-project/oxc/issues/4341
fn tagged_template() {
    test("(1, o.f)()", "(0, o.f)()");
    test("(1, o.f)``", "(0, o.f)``");
    test("(!0 && o.f)()", "(0, o.f)()");
    test("(!0 && o.f)``", "(0, o.f)``");
    test("(!0 ? o.f : !1)()", "(0, o.f)()");
    test("(!0 ? o.f : !1)``", "(0, o.f)``");

    test("foo(true && o.f)", "foo(o.f)");
    test("foo(true ? o.f : false)", "foo(o.f)");
}

#[test]
fn eval() {
    // Keep indirect-eval syntaxes
    test("(!0 && eval)(x)", "(0, eval)(x)");
    test("(1 ? eval : 2)(x)", "(0, eval)(x)");
    test("(1 ? eval : 2)?.(x)", "(0, eval)?.(x)");
    test("(1, eval)(x)", "(0, eval)(x)");
    test("(1, eval)?.(x)", "(0, eval)?.(x)");
    test("(3, eval)(x)", "(0, eval)(x)");
    test("(4, eval)?.(x)", "(0, eval)?.(x)");
    test_same("(eval)(x)");
    test_same("(eval)?.(x)");
    test_same("eval(x)");
    test_same("eval(x, y)");
    test_same("eval(x,y)");
    test_same("eval?.(x)");
    test_same("eval?.(x)");
    test_same("eval?.(x, y)");
    test_same("eval?.(x,y)");
}

#[test]
fn unused() {
    test_unused(
        "(function() {
          let v;
          window.foo = function() {
            return v ?? (v = bar());
          }
        })()
        ",
        "(function() {
          let v;
          window.foo = function() {
            return v ??= bar();
          }
        })()
        ",
    );
    test_unused(
        "(function() {
          let v;
          window.foo = function() {
            return v ?? (console.log(), v = bar());
          }
        })()
        ",
        "(function() {
          let v;
          window.foo = function() {
            return v ??= (console.log(), bar());
          }
        })()
        ",
    );
    test_unused(
        "(function() {
          let v;
          window.foo = function() {
            return v = v || bar();
          }
        })()
        ",
        "(function() {
          let v;
          window.foo = function() {
            return v ||= bar();
          }
        })()
        ",
    );
    test_unused(
        "(function() {
          let v;
          window.foo = function() {
            return v = v + bar();
          }
        })()
        ",
        "(function() {
          let v;
          window.foo = function() {
            return v += bar();
          }
        })()
        ",
    );
}
