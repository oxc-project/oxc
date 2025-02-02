mod dead_code_elimination;
mod esbuild;

use oxc_minifier::CompressOptions;

fn test(source_text: &str, expected: &str) {
    let options = CompressOptions::all_false();
    crate::test(source_text, expected, options);
}

fn test_same(source_text: &str) {
    test(source_text, source_text);
}

// Oxc Integration Tests

#[test]
fn integration() {
    test(
        "require('./index.js')(function (e, os) {
    if (e) return console.log(e)
    return console.log(JSON.stringify(os))
    })",
        r#"require("./index.js")(function(e, os) {
    return console.log(e || JSON.stringify(os));
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
        return value === null || Array.isArray(value) || isTimeDisabled == null ? void 0 : isTimeDisabled(value);
    }");

    test_same("a && (b && (c && (d && (e && (f && (g && (h && i && j && k && l && m && n && o && p && q && r && s && t && u && v && w && x && y && z)))))))");

    test(
        "if (((() => console.log('effect'))(), true)) {
         } else {
           var c = 1;
           for (var c; unknownGlobal && true; unknownGlobal && true) var d;
         }
         console.log(c, d);
        ",
        "if ((() => console.log('effect'))(), !1) for (var c = 1, c; unknownGlobal; unknownGlobal && !0) var d;
        console.log(c, d);
        ",
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
    test_same("(1, o.f)()");
    test_same("(1, o.f)``");
    test_same("(!0 && o.f)()");
    test_same("(!0 && o.f)``");
    test("(!0 ? o.f : !1)()", "(0 ? !1: o.f)()");
    test("(!0 ? o.f : !1)``", "(0 ? !1: o.f)``");

    test("foo(true && o.f)", "foo(o.f)");
    test("foo(true ? o.f : false)", "foo(o.f)");
}
