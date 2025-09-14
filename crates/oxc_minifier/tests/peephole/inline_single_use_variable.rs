use oxc_minifier::{CompressOptions, CompressOptionsKeepNames};
use oxc_span::SourceType;

use crate::{default_options, test, test_options, test_options_source_type, test_same};

#[track_caller]
fn test_script_same(source_text: &str) {
    test_script(source_text, source_text);
}

#[track_caller]
fn test_script(source_text: &str, expected: &str) {
    test_options_source_type(source_text, expected, SourceType::cjs(), &default_options());
}

#[track_caller]
fn test_keep_names(source_text: &str, expected: &str) {
    test_options(
        source_text,
        expected,
        &CompressOptions { keep_names: CompressOptionsKeepNames::all_true(), ..default_options() },
    );
}

#[test]
fn test_inline_single_use_variable() {
    test_same("function wrapper(arg0, arg1) {using x = foo; return x}");
    test_same("async function wrapper(arg0, arg1) { await using x = foo; return x}");

    test(
        "
        class Foo {
            #foo;
            static {
                let v = this;
                let r = #foo in v;
                console.log(r);
            }
        }
        ",
        "
        class Foo {
            #foo;
            static {
                let r = #foo in this;
                console.log(r);
            }
        }
    ",
    );
    test(
        "
        class Foo {
            #foo;
            static {
                let x = foo;
                this.#foo = x;
            }
        }
        ",
        "
        class Foo {
            #foo;
            static {
                this.#foo = foo;
            }
        }
    ",
    );
    test(
        "
        class Foo {
            #foo;
            static {
                let x = this;
                let y = x.#foo;
                console.log(y);
            }
        }
        ",
        "
        class Foo {
            #foo;
            static {
                let y = this.#foo;
                console.log(y);
            }
        }
    ",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return new arg1(...x);}",
        "function wrapper(arg0, arg1) { return new arg1(...arg0);}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return new arg1(x, ...arg1);}",
        "function wrapper(arg0, arg1) { return new arg1(arg0, ...arg1);}",
    );
    test_same("function wrapper(arg0, arg1) { let x = arg0; return new arg1(...arg1, x);}");
    test(
        "function wrapper(arg0, arg1) { let x = arg0; new arg1(x);}",
        "function wrapper(arg0, arg1) { new arg1(arg0);}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; new x()}",
        "function wrapper(arg0, arg1) { new arg0();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; new (0, x)()}",
        "function wrapper(arg0, arg1) { new arg0();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0.foo; new x.bar()}",
        "function wrapper(arg0, arg1) { new arg0.foo.bar();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0.foo; new x[bar]()}",
        "function wrapper(arg0, arg1) { new arg0.foo[bar]();}",
    );
    test_same("function wrapper(arg0, arg1) { let x = arg0.foo; new x()}");
    test_same("function wrapper(arg0, arg1) { let x = arg0[foo]; new x()}");
    test_same("function wrapper(arg0, arg1) { let x = arg0?.foo; new x()}");
    test_same("function wrapper(arg0, arg1) { let x = arg0?.[foo]; new x()}");
    test(
        "function wrapper(arg0, arg1) { let x = arg0.foo; new (0, x)()}",
        "function wrapper(arg0, arg1) { let x = arg0.foo; new x();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0[foo]; new (0, x)()}",
        "function wrapper(arg0, arg1) { let x = arg0[foo]; new x();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0?.foo; new (0, x)()}",
        "function wrapper(arg0, arg1) { let x = arg0?.foo; new x();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0?.[foo]; new (0, x)()}",
        "function wrapper(arg0, arg1) { let x = arg0?.[foo]; new x();}",
    );
    test(
        "function wrapper(arg0, arg1) { let x = arg0; return (x(), 1);}",
        "function wrapper(arg0, arg1) { return (arg0(), 1);}",
    );
    test_same("function wrapper(arg0, arg1) { let x = arg0; return (foo(), x(), 1);}");
    test(
        "function wrapper() { let x = [0, 1, 2]; return foo.bar(x);}",
        "function wrapper() { return foo.bar([0, 1, 2]);}",
    );
    test(
        "function wrapper() { let x = () => { console.log() }; foo(x) }",
        "function wrapper() { foo(() => { console.log() }) }",
    );
    test(
        "function wrapper() { let x = function () { console.log() }; foo(x) }",
        "function wrapper() { foo(function() { console.log() }) }",
    );

    test(
        "function wrapper() { var x = foo; for (var i = x; i < 10; i++) console.log(i) }",
        "function wrapper() { for (var i = foo; i < 10; i++) console.log(i) }",
    );
    test(
        "function wrapper() { var i, x = foo; for (i = x; i < 10; i++) console.log(i) }",
        "function wrapper() { var i; for (i = foo; i < 10; i++) console.log(i) }",
    );
    test(
        "function wrapper() { var x = {}; for (var a in x) console.log(a) }",
        "function wrapper() { for (var a in {}) console.log(a) }",
    );
    test(
        "function wrapper() { var x = {}; for (var a = 0 in x) console.log(a) }",
        "function wrapper() { var x = {}; for (var a = 0 in x) console.log(a) }",
    );
    test(
        "function wrapper() { var x = []; for (var a of x) console.log(a) }",
        "function wrapper() { for (var a of []) console.log(a) }",
    );
}

#[test]
fn keep_exposed_variables() {
    test_same("var x = foo; x(); export { x }");
    test("var x = foo; x()", "foo()");
    test_script_same("var x = foo; x()");
    test_script("{ let x = foo; x() }", "foo()");
}

#[test]
fn keep_names() {
    test(
        "var x = function() {}; var y = x; console.log(y.name)",
        "console.log(function() {}.name)",
    );
    test_keep_names(
        "var x = function() {}; var y = x; console.log(y.name)",
        "var x = function() {}, y = x; console.log(y.name)",
    );
    test_keep_names(
        "var x = (function() {}); var y = x; console.log(y.name)",
        "var x = (function() {}), y = x; console.log(y.name)",
    );
    test_keep_names(
        "var x = function foo() {}; var y = x; console.log(y.name)",
        "console.log(function foo() {}.name)",
    );

    test(
        "var x = class {}; var y = x; console.log(y.name)",
        "var y = class {}; console.log(y.name)",
    );
    test_keep_names(
        "var x = class {}; var y = x; console.log(y.name)",
        "var x = class {}, y = x; console.log(y.name)",
    );
    test_keep_names(
        "var x = (class {}); var y = x; console.log(y.name)",
        "var x = (class {}), y = x; console.log(y.name)",
    );
    test_keep_names(
        "var x = class Foo {}; var y = x; console.log(y.name)",
        "var y = class Foo {}; console.log(y.name)",
    );
}

#[test]
fn integration() {
    test(
        "
        export function foo() {
        var args = [];
        for (var _i = 0; _i < arguments.length; _i++) {
            args[_i] = arguments[_i];
        }
        return bar(args);
        }

        function bar(args) {
        return args.concat(0)
        }
    ",
        "
        export function foo() {
                return bar([...arguments]);
        }
        function bar(args) {
                return args.concat(0);
        }
    ",
    );
    test(
        "
        var bar = foo.bar;
        if (typeof bar !== 'object' || bar === null) console.log('foo')
        ",
        "
        var bar = foo.bar;
        (typeof bar != 'object' || !bar) && console.log('foo')
        ",
    );
}
