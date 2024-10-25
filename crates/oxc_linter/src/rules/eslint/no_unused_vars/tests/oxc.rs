//! Test cases created by oxc maintainers

use serde_json::json;

use super::NoUnusedVars;
use crate::{tester::Tester, FixKind, RuleMeta as _};

// uncomment to only run a single test. useful for step-through debugging.
#[test]
fn test_debug() {
    let pass = vec![
        (
            "const [ a, _b, c ] = items;
			console.log(a+c);",
            Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
        ), // { "ecmaVersion": 6 },
    ];
    let fail = vec![];
    Tester::new(NoUnusedVars::NAME, pass, fail).intentionally_allow_no_fix_tests().test();
}

#[test]
fn test_vars_simple() {
    let pass = vec![
        ("let a = 1; console.log(a)", None),
        ("let a = 1; new Foo(a)", None),
        ("let a = 1; let b = a + 1; console.log(b)", None),
        ("let a = 1; if (true) { console.log(a) }", None),
        ("let _a = 1", Some(json!([{ "varsIgnorePattern": "^_" }]))),
        ("const { foo: _foo, baz } = obj; f(baz);", Some(json!([{ "varsIgnorePattern": "^_" }]))),
        (
            r"export const rendered = marked(markdown, {
                  renderer: new (class CustomRenderer extends Renderer {})(),
              });",
            None,
        ),
        // https://github.com/oxc-project/oxc/issues/5391
        (
            "
            import styled from 'styled-components';

            import { Prose, ProseProps } from './prose';

            interface Props extends ProseProps {
              density?: number;
            }

            export const HandMarkedPaperBallotProse = styled(Prose)<Props>`
            line-height: ${({ density }) => (density !== 0 ? '1.1' : '1.3')};
            `;
            ",
            None,
        ),
        (
            "
                const a = 0
                obj[a]++;
                obj[a] += 1;
            ",
            None,
        ),
        (
            "
                const obj = 0
                obj.a++;
                obj.a += 1;
                obj.b.c++;
            ",
            None,
        ),
        ("console.log(function a() {} ? b : c)", None),
        ("console.log(a ? function b() {} : c)", None),
        ("console.log(a ? b : function c() {})", None),
    ];
    let fail = vec![
        ("let a = 1", None),
        ("let a: number = 1", None),
        ("let a = 1; a = 2", None),
        (
            "let _a = 1; console.log(_a)",
            Some(json!([{ "varsIgnorePattern": "^_", "reportUsedIgnorePattern": true }])),
        ),
        ("const { foo: fooBar, baz } = obj; f(baz);", None),
        ("let _a = 1", Some(json!([{ "argsIgnorePattern": "^_" }]))),
    ];

    let fix = vec![
        // unused vars should be removed
        ("let a = 1;", "", None, FixKind::DangerousSuggestion),
        // FIXME: b should be deleted as well.
        ("let a = 1, b = 2;", "let b = 2;", None, FixKind::DangerousSuggestion),
        (
            "let a = 1; let b = 2; console.log(a);",
            "let a = 1;  console.log(a);",
            None,
            FixKind::DangerousSuggestion,
        ),
        (
            "let a = 1; let b = 2; console.log(b);",
            " let b = 2; console.log(b);",
            None,
            FixKind::DangerousSuggestion,
        ),
        (
            "let a = 1, b = 2; console.log(b);",
            "let b = 2; console.log(b);",
            None,
            FixKind::DangerousSuggestion,
        ),
        (
            "let a = 1, b = 2; console.log(a);",
            "let a = 1; console.log(a);",
            None,
            FixKind::DangerousSuggestion,
        ),
        (
            "let a = 1, b = 2, c = 3; console.log(a + c);",
            "let a = 1, c = 3; console.log(a + c);",
            None,
            FixKind::DangerousSuggestion,
        ),
        (
            "let a = 1, b = 2, c = 3; console.log(b + c);",
            "let b = 2, c = 3; console.log(b + c);",
            None,
            FixKind::DangerousSuggestion,
        ),
        // vars initialized to `await` are not removed
        ("const x = await foo();", "const x = await foo();", None, FixKind::DangerousSuggestion),
        (
            "const x = (await foo()) as unknown as MyType",
            "const x = (await foo()) as unknown as MyType",
            None,
            FixKind::DangerousSuggestion,
        ),
        // vars with references get renamed
        ("let x = 1; x = 2;", "let _x = 1; _x = 2;", None, FixKind::DangerousFix),
        (
            "let a = 1; a = 2; a = 3;",
            "let _a = 1; _a = 2; _a = 3;",
            Some(json!([{ "varsIgnorePattern": "^_" }])),
            FixKind::DangerousFix,
        ),
        (
            "let x = 1; x = 2;",
            "let x = 1; x = 2;",
            Some(json!( [{ "varsIgnorePattern": "^tooCompli[cated]" }] )),
            FixKind::DangerousFix,
        ),
        // type annotations do not get clobbered
        ("let x: number = 1; x = 2;", "let _x: number = 1; _x = 2;", None, FixKind::DangerousFix),
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .expect_fix(fix)
        .with_snapshot_suffix("oxc-vars-simple")
        .test_and_snapshot();
}

#[test]
fn test_vars_self_use() {
    let pass = vec![
        "
        function foo() {
            let bar = 0;
            return bar++;
        }
        foo();
        ",
    ];
    let fail = vec![
        "
        function foo() {
            return foo
        }
        ",
        "
        const foo = () => {
            return foo
        }
        ",
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("oxc-vars-self-use")
        .test_and_snapshot();
}

#[test]
fn test_vars_discarded_reads() {
    let pass = vec![
        // https://github.com/oxc-project/oxc/pull/4445#issuecomment-2254122889
        "
        (() => {
            const t = import.meta.url,
                s = {};
            return '' !== t && (s.resourcesUrl = new URL('.', t).href), e(s);
        })();
        ",
        "var a; b !== '' && (x = a, f(c))",
        "
        class Test {
            async updateContextGroup(t, i, s = !0) {
                s ? await this.leave(i) : await this.join(t, i), false;
            }
        }

        new Test();
        ",
    ];

    let fail = vec![
        "
        function foo(a) { return (a, 0); }
        foo(1);
        ",
        "
        const I = (e) => (l) => {
            e.push(l), n || false;
        };
        ",
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("oxc-vars-discarded-read")
        .test_and_snapshot();
}

#[test]
fn test_vars_reassignment() {
    let pass = vec![
        "let i = 0; someFunction(i++);",
        "
        const thunk = () => 3;
        let result = undefined;
        result &&= thunk();
        console.log(result);
        ",
        r"
        const thunk = () => 3;
        {
            let a = thunk();
            console.log(a);
        }
        ",
        "let a = 0; let b = a++; f(b);",
        "let a = 0, b = 1; let c = b = a = 1; f(c+b);",
        // implicit returns
        "
		let i = 0;
        const func = () => 'value: ' + i++;
        func();
        ",
        // parenthesis are transparent
        "let a = 0; let b = ((a++)); f(b);",
        // type casting is transparent
        "let a = 0; let b = a as any; f(b);",
        "let a = 0; let b = a as unknown as string as unknown as number; f(b);",
        "let a = 0; let b = a++ as string | number; f(b);",
        // pathological sequence assignments
        "let a = 0; let b = (0, a++); f(b);",
        "let a = 0; let b = (0, (a++)); f(b);",
        "let a = 0; let b = (0, (a++) as string | number); f(b);",
        "let a = 0; let b = (0, (0, a++)); f(b);",
        "let a = 0; let b = (0, (((0, a++)))); f(b);",
        "let a = 0; let b = (0, a) + 1; f(b);",
        // reassignment in conditions
        "
        function foo() {
            if (i++ === 0) {
                return 'zero';
            } else {
                return 'not zero';
            }
            var i = 0;
        }
        foo();
        ",
        "
        let i = 10;
        while (i-- > 0) {
            console.log('countdown');
        };
        ",
        "
        let i = 10;
        do {
            console.log('countdown');
        } while(i-- > 0);
        ",
        "let i = 0; i > 0 ? 'positive' : 'negative';",
        "let i = 0; i > 0 && console.log('positive');",
    ];

    let fail = vec![
        "let a = 1; a ||= 2;",
        "let a = 0; a = a + 1;",
        // type casting is transparent
        "let a = 0; a = a++ as any;",
        "let a = 0; a = a as unknown as string as unknown as number;",
        // pathological sequence assignments
        "let a = 0; a = ++a;",
        "let a = 0; a = (0, ++a);",
        "let a = 0; a = (a++, 0);",
        "let a = 0; let b = (a++, 0); f(b);",
        "let a = 0; let b = (0, (a++, 0)); f(b);",
        "let a = 0; let b = ((0, a++), 0); f(b);",
        "let a = 0; let b = (a, 0) + 1; f(b);",
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("oxc-vars-reassignment")
        .test_and_snapshot();
}

#[test]
fn test_vars_destructure() {
    let pass = vec![
        (
            "const { a, ...rest } = obj; console.log(rest)",
            Some(json![[{ "ignoreRestSiblings": true }]]),
        ),
        (
            "const { a, ...rest } = obj; console.log(rest)",
            Some(json!( [{ "ignoreRestSiblings": true, "vars": "all" }] )),
        ),
        (
            "const { a, ...rest } = obj; console.log(rest)",
            Some(json!( [{ "ignoreRestSiblings": true, "vars": "all" }] )),
        ),
        // https://github.com/oxc-project/oxc/issues/4888
        (
            "const { text, ...dbEntry } = entry; return doSomething({ ...dbEntry, someOtherProp });",
            Some(json!([{
                "args": "none",
                "caughtErrors": "none",
                "ignoreRestSiblings": true,
                "vars": "all"
            }])),
        ),
    ];
    let fail = vec![
        ("const { a, ...rest } = obj", Some(json!( [{ "ignoreRestSiblings": true }] ))),
        ("const [a, ...rest] = arr", Some(json!( [{ "ignoreRestSiblings": true }] ))),
        (
            "const { a: { b }, ...rest } = obj; console.log(a)",
            Some(json!( [{ "ignoreRestSiblings": true }] )),
        ),
        (
            "const { a: { b }, ...rest } = obj; console.log(rest)",
            Some(json!( [{ "ignoreRestSiblings": true }] )),
        ),
        // https://github.com/oxc-project/oxc/issues/4839
        (r#"const l="",{e}=r"#, None),
    ];

    let fix = vec![
        // single destructure
        ("const { a } = obj;", "", None, FixKind::DangerousSuggestion),
        ("const [a] = arr;", "", None, FixKind::DangerousSuggestion),
        // multi destructure
        (
            "const { a, b } = obj; f(b)",
            "const { b } = obj; f(b)",
            None,
            FixKind::DangerousSuggestion,
        ),
        ("const [a, b] = arr; f(b)", "const [,b] = arr; f(b)", None, FixKind::DangerousSuggestion),
        ("const [a, b] = arr; f(a)", "const [a] = arr; f(a)", None, FixKind::DangerousSuggestion),
        (
            "const [a, b, c] = arr; f(a, c)",
            "const [a, ,c] = arr; f(a, c)",
            None,
            FixKind::DangerousSuggestion,
        ),
        ("let [f,\u{a0}a]=p", "let [,a]=p", None, FixKind::DangerousSuggestion),
        (
            "const [a, b, c, d, e] = arr; f(a, e)",
            "const [a, ,,,e] = arr; f(a, e)",
            None,
            FixKind::DangerousSuggestion,
        ),
        (
            "const [a, b, c, d, e, f] = arr; fn(a, e)",
            "const [a, ,,,e] = arr; fn(a, e)",
            None,
            FixKind::DangerousSuggestion,
        ),
        (
            "const { foo: fooBar, baz } = obj; f(baz);",
            "const { baz } = obj; f(baz);",
            None,
            FixKind::DangerousSuggestion,
        ),
        // multi destructure with rename
        (
            "const { a: foo, b: bar } = obj; f(bar)",
            "const { b: bar } = obj; f(bar)",
            None,
            FixKind::DangerousSuggestion,
        ),
        // TODO: destructures in VariableDeclarations with more than one declarator
        (r#"const l="",{e}=r"#, r"const {e}=r", None, FixKind::All),
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .expect_fix(fix)
        .with_snapshot_suffix("oxc-vars-destructure")
        .test_and_snapshot();
}

#[test]
fn test_vars_catch() {
    let pass = vec![
        ("try {} catch (e) { throw e }", None),
        ("try {} catch (e) { }", Some(json!([{ "caughtErrors": "none" }]))),
        ("try {} catch { }", None),
        ("try {} catch(_) { }", Some(json!([{ "caughtErrorsIgnorePattern": "^_" }]))),
        (
            "try {} catch(_) { }",
            Some(json!([{ "caughtErrors": "all", "caughtErrorsIgnorePattern": "^_" }])),
        ),
        (
            "try {} catch(_e) { }",
            Some(json!([{ "caughtErrors": "all", "caughtErrorsIgnorePattern": "^_" }])),
        ),
    ];

    let fail = vec![
        ("try {} catch (e) { }", Some(json!([{ "caughtErrors": "all" }]))),
        ("try {} catch(_) { }", None),
        (
            "try {} catch(_) { }",
            Some(json!([{ "caughtErrors": "all", "varsIgnorePattern": "^_" }])),
        ),
        (
            "try {} catch(foo) { }",
            Some(json!([{ "caughtErrors": "all", "caughtErrorsIgnorePattern": "^ignored" }])),
        ),
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("oxc-vars-catch")
        .test_and_snapshot();
}

#[test]
fn test_vars_using() {
    let pass = vec![("using a = 1; console.log(a)", None)];

    let fail = vec![("using a = 1;", None)];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("oxc-vars-using")
        .test_and_snapshot();
}
#[test]
fn test_functions() {
    let pass = vec![
        "function foo() {}\nfoo()",
        "const a = () => {}; a();",
        "var foo = function foo() {}\n foo();",
        "var foo = function bar() {}\n foo();",
        "var foo; foo = function bar() {}; foo();",
        "
        const obj = {
            foo: function foo () {}
        }
        f(obj)
        ",
        "
        function foo() {}
        function bar() { foo() }
        bar()
        ",
        "
        function foo() {}
        if (true) {
            foo()
        }
        ",
        "
        function main() {
            function foo() {}
            if (true) { foo() }
        }
        main()
        ",
        "
        function foo() {
        return function bar() {}
        }
        foo()()
        ",
        "
        import debounce from 'debounce';

        const debouncedFoo = debounce(function foo() {
            console.log('do a thing');
        }, 100);

        debouncedFoo();
        ",
        // FIXME
        "
            const createIdFactory = ((): (() => string) => {
                let count = 0;
                return () => `${count++}`
        })();

        const getId = createIdFactory();
        console.log(getId());
            ",
        // calls on optional chains should be valid
        "
        let foo = () => {};
        foo?.();
        ",
        "
        function foo(a: number): number;
        function foo(a: number | string): number {
            return Number(a)
        }
        foo();
        ",
        "export const Component = () => <button onClick={function onClick(e) { console.log(e) }} />",
        // https://github.com/oxc-project/oxc/pull/4445#issuecomment-2254122889
        "
        Promise.withResolvers ||
            (Promise.withResolvers = function withResolvers<T>() {
                let resolve!: (value: T | PromiseLike<T>) => void;
                let reject!: (reason: unknown) => void;

                const promise = new this<T>((promiseResolve, promiseReject) => {
                    resolve = promiseResolve;
                    reject = promiseReject;
                });

                return {
                    resolve,
                    reject,
                    promise,
                };
            });
        ",
        "const foo = () => function bar() { }\nfoo()",
        "module.exports.foo = () => function bar() { }",
        // https://github.com/oxc-project/oxc/issues/5406
        "
        export function log(message: string, ...interpolations: unknown[]): void;
        export function log(message: string, ...interpolations: unknown[]): void {
            console.log(message, interpolations);
        }
        ",
        "declare function func(strings: any, ...values: any[]): object"
    ];

    let fail = vec![
        "function foo() {}",
        "function foo() { foo() }",
        "const foo = () => { function bar() { } }\nfoo()",
        "
        export function log(message: string, ...interpolations: unknown[]): void;
        export function log(message: string, ...interpolations: unknown[]): void {
            console.log(message);
        }
        ",
        "
        export function log(...messages: unknown[]): void {
            return;
        }
        ",
    ];

    let fix = vec![
        // function declarations are never removed
        ("function foo() {}", "function foo() {}", None, FixKind::DangerousSuggestion),
        (
            "function foo() { function bar() {} }",
            "function foo() { function bar() {} }",
            None,
            FixKind::DangerousSuggestion,
        ),
        (
            "function foo() { function bar() {} }\nfoo()",
            "function foo() { function bar() {} }\nfoo()",
            None,
            FixKind::DangerousSuggestion,
        ),
        // function expressions + arrow functions are not removed if declared in
        // the root scope
        (
            "const foo = function foo() {}",
            "const foo = function foo() {}",
            None,
            FixKind::DangerousSuggestion,
        ),
        (r"const foo = () => {}", r"const foo = () => {}", None, FixKind::DangerousSuggestion),
        // function expressions + arrow functions are removed if not declared in
        // root scope
        (
            "
                function foo() { const bar = function bar() {} }
                foo();
            ",
            "
                function foo() {  }
                foo();
            ",
            None,
            FixKind::DangerousSuggestion,
        ),
        (
            "
                function foo() { const bar = x => x }
                foo();
            ",
            "
                function foo() {  }
                foo();
            ",
            None,
            FixKind::DangerousSuggestion,
        ),
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .with_snapshot_suffix("oxc-functions")
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test_self_call() {
    let pass = vec![
        "const _thunk = (function createThunk(count) {
            if (count === 0) return () => count
            return () => createThunk(count - 1)()
        })()",
    ];

    let fail = vec![
        // Functions that call themselves are considered unused, even if that
        // call happens within an inner function.
        "function foo() { return function bar() { return foo() } }",
        // Classes that construct themselves are considered unused
        "class Foo {
            static createFoo() {
                return new Foo();
            }
        }",
        "class Foo {
            static createFoo(): Foo {
                return new Foo();
            }
        }",
        "class Point {
            public x: number;
            public y: number;
            public add(other): Point {
                const p = new Point();
                p.x = this.x + (other as Point).x;
                p.y = this.y + (other as Point).y;
                return p;
            }
        }
        ",
        // FIXME
        // "class Foo {
        //     inner: any
        //     public foo(): Foo {
        //         if(this.inner?.constructor.name === Foo.name) {
        //             return this.inner;
        //         } else {
        //             return new Foo();
        //         }
        //     }
        // }",
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("oxc-self-call")
        .test_and_snapshot();
}

#[test]
fn test_imports() {
    let pass = vec![
        ("import { a } from 'b'; console.log(a)", None),
        ("import * as a from 'a'; console.log(a)", None),
        ("import a from 'a'; console.log(a)", None),
        ("import { default as a } from 'a'; console.log(a)", None),
        (
            "import { createElement } from 'preact/compat';",
            Some(json!([{ "varsIgnorePattern": "^(h|React|createElement)$" }])),
        ),
        (
            "import { createElement } from 'preact/compat';",
            Some(json!([{ "args": "none", "varsIgnorePattern": "^(h|React|createElement)$" }])),
        ),
    ];
    let fail = vec![
        ("import { a } from 'a'", None),
        ("import * as a from 'a'", None),
        ("import { a as b } from 'a'; console.log(a)", None),
    ];

    let fix = vec![
        // None used
        ("import foo from './foo';", "", None, FixKind::DangerousSuggestion),
        ("import * as foo from './foo';", "", None, FixKind::DangerousSuggestion),
        ("import { Foo } from './foo';", "", None, FixKind::DangerousSuggestion),
        ("import { Foo as Bar } from './foo';", "", None, FixKind::DangerousSuggestion),
        // Some used
        (
            "import foo, { bar } from './foo'; bar();",
            "import { bar } from './foo'; bar();",
            None,
            FixKind::DangerousSuggestion,
        ),
        (
            "import foo, { bar } from './foo'; foo();",
            "import foo, { } from './foo'; foo();",
            None,
            FixKind::DangerousSuggestion,
        ),
        (
            "import { foo, bar, baz } from './foo'; foo(bar);",
            "import { foo, bar, } from './foo'; foo(bar);",
            None,
            FixKind::DangerousSuggestion,
        ),
        (
            "import { foo, bar, baz } from './foo'; foo(baz);",
            "import { foo, baz } from './foo'; foo(baz);",
            None,
            FixKind::DangerousSuggestion,
        ),
        (
            "import { foo, bar, baz } from './foo'; bar(baz);",
            "import { bar, baz } from './foo'; bar(baz);",
            None,
            FixKind::DangerousSuggestion,
        ),
        // type imports
        (
            "import { type foo, bar } from './foo'; bar();",
            "import { bar } from './foo'; bar();",
            None,
            FixKind::DangerousSuggestion,
        ),
        (
            "import { foo, type bar, baz } from './foo'; foo(baz);",
            "import { foo, baz } from './foo'; foo(baz);",
            None,
            FixKind::DangerousSuggestion,
        ),
        (
            "import foo, { type bar } from './foo'; foo();",
            "import foo, { } from './foo'; foo();",
            None,
            FixKind::DangerousSuggestion,
        ),
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .expect_fix(fix)
        .with_snapshot_suffix("oxc-imports")
        .test_and_snapshot();
}

#[test]
fn test_used_declarations() {
    let pass = vec![
        // function declarations passed as arguments, used in assignments, etc. are used, even if they are
        // first put into an intermediate (e.g. an object or array)
        "arr.reduce(function reducer (acc, el) { return acc + el }, 0)",
        "console.log({ foo: function foo() {} })",
        "console.log({ foo: function foo() {} as unknown as Function })",
        "test.each([ function foo() {} ])('test some function', (fn) => { expect(fn(1)).toBe(1) })",
        "export default { foo() {}  }",
        "const arr = [function foo() {}, function bar() {}]; console.log(arr[0]())",
        "const foo = function foo() {}; console.log(foo())",
        "const foo = function bar() {}; console.log(foo())",
        // Class expressions behave similarly
        "console.log([class Foo {}])",
        "export default { foo: class Foo {} }",
        "export const Foo = class Foo {}",
        "export const Foo = class Bar {}",
        "export const Foo = @SomeDecorator() class Foo {}",
    ];
    let fail = vec![
        // array is not used, so the function is not used
        ";[function foo() {}]",
        ";[class Foo {}]",
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("oxc-used-declarations")
        .test_and_snapshot();
}

#[test]
fn test_exports() {
    let pass = vec![
        "export const a = 1; console.log(a)",
        "export function foo() {}",
        "export default function foo() {}",
        "export class A {}",
        "export interface A {}",
        "export type A = string",
        "export enum E { }",
        // default exports
        "export default class Foo {}",
        "export default [ class Foo {} ];",
        "export default function foo() {}",
        "export default { foo() {} };",
        "export default { foo: function foo() {} };",
        "export default { get foo() {} };",
        "export default [ function foo() {} ];",
        "export default (function foo() { return 1 })();",
        // "export enum E { A, B }",
        "const a = 1; export { a }",
        "const a = 1; export default a",
        // re-exports
        "import { a } from 'a'; export { a }",
        "import { a as b } from 'a'; export { b }",
        "export * as a from 'a'",
        "export { a, b } from 'a'",
    ];
    let fail = vec!["import { a as b } from 'a'; export { a }"];

    // these are mostly pass[] cases, so do not snapshot
    Tester::new(NoUnusedVars::NAME, pass, fail).intentionally_allow_no_fix_tests().test();
}

#[test]
fn test_react() {
    let pass = vec![
        "
        import React from 'react';

        export const Foo = () => <div />;
        ",
        "
        // React 17 API
        import React from 'react';
        import ReactDOM from 'react-dom';

        interface Props {}
        const Component = React.forwardRef<HTMLElement, Props>(
            function Component(props, ref) {
                return <div ref={ref} {...props} />
            }
        );

        ReactDOM.render(<Component />, document.getElementById('root'));
        ",
        "
        import React from 'react';
        export class Foo extends React.Component<{}, { i: number }> {
            constructor(props) {
                super(props);
            }

            getId = () => {
            }
        }
        ",
    ];

    let fail = vec![
        "
        const React = {};

        export const Foo = () => <div />
        ",
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail).intentionally_allow_no_fix_tests().test();
}

#[test]
fn test_arguments() {
    let pass = vec![
        ("function foo(a) { return a } foo()", None),
        ("function foo(a, b) { return b } foo()", Some(json!([{ "args": "after-used" }]))),
        ("let ids = arr.map(el => el.id); f(ids)", None),
        (
            "let targetId = '1234'; let user = users.find(user => user.id === targetId); f(user)",
            None,
        ),
        (
            "
        const unboxed = arr.map(el => {
            if (typeof el === 'object') return el['value']
            else return el
        })
        f(unboxed)
        ",
            None,
        ),
    ];
    let fail = vec![
        ("function foo(a) {} foo()", None),
        ("function foo(a: number) {} foo()", None),
        ("function foo({ a }, b) { return b } foo()", Some(json!([{ "args": "after-used" }]))),
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("oxc-arguments")
        .test_and_snapshot();
}

#[test]
fn test_enums() {
    let pass = vec![
        "export enum Foo { A, B }",
        "enum Foo { A }\nconsole.log(Foo.A)",
        "enum Foo { A, B }\n export { Foo }",
    ];

    let fail = vec!["enum Foo { A }"];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("oxc-enums")
        .test_and_snapshot();
}

#[test]
fn test_classes() {
    let pass = vec![
        "
        export class Foo {
            public a = 4;
            private b;
        }
        ",
        // TS constructor property definitions
        "export class Foo { constructor(public a) {} }",
        "export class Foo { constructor(private a) {} }",
        "export class Foo { constructor(protected a) {} }",
        "export class Foo { constructor(readonly a) {} }",
        "export class Foo extends Bar { constructor(override a) {} }",
        "export class Foo { constructor(public readonly a) {} }",
        // note: abstract doesn't count, but that's a parse error
        // setters can have unused methods
        "export class Foo { set foo(value) { } }",
        "export class Foo { public set foo(value) { } }",
        "
        class Foo { }
        class Bar extends Foo {}
        console.log(new Bar());
        ",
        "
        export abstract class Foo {
            public abstract bar(a: number): string;
        }
        ",
        "var Foo = class Foo {}; new Foo();",
        // override methods must have the same signature as their parent and so
        // any unused parameters in them are allowed
        "
        class Foo {
            public method(a: number, b: number): number {
                return a + b;
            }
        }
        class Bar extends Foo {
            public override method(a: number, b: number): number {
                return a;
            }
        }
        new Bar();
        ",
    ];

    let fail = vec![
        // no modifier = no property
        "export class Foo { constructor(a: number) {} }",
        // not a setter
        "export class Foo { set(value) { } }",
        "
        export abstract class Foo {
            public bar(a: number): string {}
        }
        ",
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("oxc-classes")
        .test_and_snapshot();
}

#[test]
fn test_namespaces() {
    let pass = vec![
        "export namespace N {}",
        "namespace N { export function foo() {} }\nconsole.log(N.foo());",
        "export namespace N { export function foo() {} }",
        "export namespace N { export const foo = 1 }",
        "
        export namespace N {
            export function foo() {
                bar()
            }
            function bar() {}
        }
        ",
        "declare global {}",
        "declare global { interface Window {} }",
        "
        declare global {
            namespace jest {
                interface Matcher {
                    someCustomMatcher(): void
                }
            }
        }
        ",
        "
        declare global {
            const x: number;
        }
        ",
        "
        interface Foo {}
        namespace Foo {
            export const a = {};
        }
        const foo: Foo = Foo.a
        console.log(foo)
        ",
    ];

    let fail = vec![
        "namespace N {}",
        // FIXME
        // "export namespace N { function foo() }",
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("oxc-namespaces")
        .test_and_snapshot();
}

#[test]
fn test_type_aliases() {
    let pass = vec![];

    let fail = vec![
        // usages within own declaration do not count
        "type Foo = Foo",
        "type Foo = Array<Foo>",
        "type Unbox<B> = B extends Box<infer R> ? Unbox<R> : B",
        "export type F<T> = T extends infer R ? /* R not used */ string : never",
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("oxc-type-aliases")
        .test_and_snapshot();
}

#[test]
fn test_type_references() {
    let pass = vec![
        "type A = number; export type B = Array<A>;",
        "
        type A = number;
        type B<T> = T;
        export type C = B<A>;
        ",
        "
        type A<T> = T;
        type B<T> = T;
        export type C = B<A<number>>;
        ",
        "const x: number = 1; function foo(): typeof x { return x }; foo()",
        // not handled by typescript-eslint. Maybe we'll add this one day
        "function foo(): typeof foo { }",
        "function foo(): typeof foo { return foo }",
        // ---
        "type T = number; console.log(3 as T);",
        "type T = number; console.log(((3) as T));",
        "type T = Record<string, any>; console.log({} as Readonly<T>)",
        // https://github.com/oxc-project/oxc/issues/4494
        "
        import type { mySchema } from './my-schema';
        function test(arg: ReturnType<typeof mySchema>) {
            arg;
        }
        test('');
        ",
        // https://github.com/oxc-project/oxc/pull/4445#issuecomment-2254122889
        "
        type PermissionValues<T> = {
            [K in keyof T]: T[K] extends object ? PermissionValues<T[K]> : T[K];
        }[keyof T];

        export type ApiPermission = PermissionValues<typeof API_PERMISSIONS>;

        export const API_PERMISSIONS = {} as const;
        ",
        "
        type Foo = 'foo' | 'bar';
        export class Bar {
            accessor x: Foo
            accessor y!: Foo
        }
        ",
    ];

    let fail = vec![
        // Type aliases
        "type T = number; function foo<T>(a: T): T { return a as T }; foo(1)",
        "type A = number; type B<A> = A; console.log(3 as B<3>)",
        "type T = { foo: T }",
        "type T = { foo?: T | undefined }",
        "type A<T> = { foo: T extends Array<infer R> ? A<R> : T }",
        "type T = { foo(): T }",
        // Type references on class symbols within that classes' definition is
        // not considered used
        "class Foo {
            private _inner: Foo | undefined;
        }",
        "class Foo {
            _inner: any;
            constructor(other: Foo);
            constructor(somethingElse: any) {
                this._inner = somethingElse;
            }
        }",
        "class LinkedList<T> {
            #next?: LinkedList<T>;
            public append(other: LinkedList<T>) {
                this.#next = other;
            }
        }",
        "class LinkedList<T> {
            #next?: LinkedList<T>;
            public nextUnchecked(): LinkedList<T> {
                return <LinkedList<T>>this.#next!;
            }
        }",
        // FIXME: ambient classes declared using `declare` are not bound by
        // semantic's binder.
        // https://github.com/oxc-project/oxc/blob/a9260cf6d1b83917c7a61b25cabd2d40858b0fff/crates/oxc_semantic/src/binder.rs#L105
        // "declare class LinkedList<T> {
        //     next(): LinkedList<T> | undefined;
        // }"

        // Same is true for interfaces
        "interface LinkedList<T> { next: LinkedList<T> | undefined }",
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("oxc-type-references")
        .change_rule_path_extension("ts")
        .test_and_snapshot();
}

// #[test]
// fn test_template() {
//     let pass = vec![];

//     let fail = vec![];

//     Tester::new(NoUnusedVars::NAME, pass, fail)
//         .with_snapshot_suffix("<replace>")
//         .test_and_snapshot();
// }
