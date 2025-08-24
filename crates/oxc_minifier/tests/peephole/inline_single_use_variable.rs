use super::{test, test_same};

#[test]
fn test_inline_single_use_variable() {
    test_same("function wrapper(arg0, arg1) {using x = foo; return x}");
    test_same("async function wrapper(arg0, arg1) { await using x = foo; return x}");
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
}
