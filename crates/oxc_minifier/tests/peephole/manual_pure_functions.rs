use oxc_minifier::{CompressOptions, CompressOptionsUnused, TreeShakeOptions};

use crate::{default_options, test_options};

#[track_caller]
fn test(source_text: &str, expected: &str, pure_functions: &[&str]) {
    let options = CompressOptions {
        treeshake: TreeShakeOptions {
            manual_pure_functions: pure_functions.iter().map(ToString::to_string).collect(),
            ..TreeShakeOptions::default()
        },
        unused: CompressOptionsUnused::Remove,
        ..default_options()
    };
    test_options(source_text, expected, &options);
}

// Tests from Terser pure_funcs.js
// https://github.com/terser/terser/blob/v5.44.1/test/compress/pure_funcs.js
mod terser_tests {
    use super::test;

    #[test]
    #[ignore = "FIXME"]
    fn array() {
        test(
            "
                var a;
                export function f(b) {
                    Math.floor(a / b);
                    Math.floor(c / b);
                }
            ",
            "export function f(b) {}",
            &["Math.floor"],
        );
    }

    #[test]
    #[ignore = "FIXME"]
    fn side_effects() {
        test(
            "
                export function f(a, b) {
                    console.log(a());
                    console.log(b);
                }
            ",
            "
                export function f(a, b) {
                    a();
                }
            ",
            &["console.log"],
        );
    }

    #[test]
    #[ignore = "FIXME"]
    fn unused() {
        test(
            "
                export function foo() {
                    var u = pure(1);
                    var x = pure(2);
                    var y = pure(x);
                    var z = pure(pure(side_effects()));
                    return pure(3);
                }
            ",
            "
                export function foo() {
                    side_effects();
                    return pure(3);
                }
            ",
            &["pure"],
        );
    }

    #[test]
    fn babel() {
        test(
            r#"
                function _classCallCheck(instance, Constructor) {
                    if (!(instance instanceof Constructor))
                        throw new TypeError("Cannot call a class as a function");
                }
                export var Foo = function Foo() {
                    _classCallCheck(this, Foo);
                };
            "#,
            r"
                export var Foo = function() {};
            ",
            &["_classCallCheck"],
        );
    }

    #[test]
    #[ignore = "FIXME"]
    fn conditional() {
        test(
            "
                pure(1 | a() ? 2 & b() : 7 ^ c());
                pure(1 | a() ? 2 & b() : 5);
                pure(1 | a() ? 4 : 7 ^ c());
                pure(1 | a() ? 4 : 5);
                pure(3 ? 2 & b() : 7 ^ c());
                pure(3 ? 2 & b() : 5);
                pure(3 ? 4 : 7 ^ c());
                pure(3 ? 4 : 5);
            ",
            "
                1 | a() ? b() : c(),
                1 | a() && b(),
                1 | a() || c(),
                a(),
                3 ? b() : c(),
                3 && b(),
                3 || c()
            ",
            &["pure"],
        );
    }

    #[test]
    #[ignore = "FIXME"]
    fn relational() {
        test(
            r#"
                foo() in foo();
                foo() instanceof bar();
                foo() < "bar";
                bar() > foo();
                bar() != bar();
                bar() !== "bar";
                "bar" == foo();
                "bar" === bar();
                "bar" >= "bar";
            "#,
            "
                bar(),
                bar(),
                bar(), bar(),
                bar(),
                bar()
            ",
            &["foo"],
        );
    }

    #[test]
    #[ignore = "FIXME"]
    fn arithmetic() {
        test(
            r#"
                foo() + foo();
                foo() - bar();
                foo() * "bar";
                bar() / foo();
                bar() & bar();
                bar() | "bar";
                "bar" >> foo();
                "bar" << bar();
                "bar" >>> "bar";
            "#,
            "
                bar(),
                bar(),
                bar(), bar(),
                bar(),
                bar()
            ",
            &["foo"],
        );
    }

    #[test]
    #[ignore = "FIXME"]
    fn boolean_and() {
        // Test logical AND with pure function calls
        test(
            r#"
                foo() && foo();
                foo() && bar();
                foo() && "bar";
                bar() && foo();
                bar() && bar();
                bar() && "bar";
                "bar" && foo();
                "bar" && bar();
                "bar" && "bar";
            "#,
            r#"
                foo() && bar(),
                bar(),
                bar() && bar(),
                bar(),
                "bar" && bar()
            "#,
            &["foo"],
        );
    }

    #[test]
    fn boolean_or() {
        // Test logical OR with pure function calls
        test(
            r#"
                foo() || foo();
                foo() || bar();
                foo() || "bar";
                bar() || foo();
                bar() || bar();
                bar() || "bar";
                "bar" || foo();
                "bar" || bar();
                "bar" || "bar";
            "#,
            r"
                foo() || bar(),
                bar(),
                bar() || bar(),
                bar()
            ",
            &["foo"],
        );
    }

    #[test]
    fn assign() {
        test(
            "
                var a;
                export function f(b) {
                    a = foo();
                    b *= 4 + foo();
                    c >>= 0 | foo();
                }
            ",
            "
                export function f(b) {
                    b *= 4 + foo(), c >>= 0 | foo();
                }
            ",
            &["foo"],
        );
    }

    #[test]
    #[ignore = "FIXME"]
    fn unary() {
        test(
            r#"
                typeof foo();
                typeof bar();
                typeof "bar";
                void foo();
                void bar();
                void "bar";
                delete a[foo()];
                delete a[bar()];
                delete a["bar"];
                a[foo()]++;
                a[bar()]++;
                a["bar"]++;
                --a[foo()];
                --a[bar()];
                --a["bar"];
                ~foo();
                ~bar();
                ~"bar";
            "#,
            "
                bar(),
                bar(),
                delete a[foo()],
                delete a[bar()],
                delete a.bar,
                a[foo()]++,
                a[bar()]++,
                a.bar++,
                --a[foo()],
                --a[bar()],
                --a.bar,
                bar()
            ",
            &["foo"],
        );
    }

    #[test]
    fn issue_3065_1() {
        test(
            "
                function modifyWrapper(a, f, wrapper) {
                    wrapper.a = a;
                    wrapper.f = f;
                    return wrapper;
                }
                function pureFunc(fun) {
                    return modifyWrapper(1, fun, function(a) {
                        return fun(a);
                    });
                }
                var unused = pureFunc(function(x) {
                    return x;
                });
            ",
            "",
            &["pureFunc"],
        );
    }

    #[test]
    #[ignore = "FIXME"]
    fn issue_3065_3() {
        test(
            r#"
                function debug(msg) {
                    console.log(msg);
                }
                debug(function() {
                    console.log("PASS");
                    return "FAIL";
                }());
            "#,
            r#"
                (function() {
                    console.log("PASS");
                })();
            "#,
            &["debug"],
        );
    }

    #[test]
    #[ignore = "FIXME"]
    fn issue_3065_4() {
        test(
            r#"
                var debug = function(msg) {
                    console.log(msg);
                };
                debug(function() {
                    console.log("PASS");
                    return "FAIL";
                }());
            "#,
            r#"
                (function() {
                    console.log("PASS");
                })();
            "#,
            &["debug"],
        );
    }
}
