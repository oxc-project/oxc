use crate::{test, test_same};

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
