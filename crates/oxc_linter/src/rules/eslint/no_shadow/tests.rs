use super::NoShadow;
use crate::rule::RuleMeta;

#[test]
fn test_eslint() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "var a=3; function b(x) { a++; return x + a; }; setTimeout(function() { b(a); }, 0);",
            None,
            None,
            None,
        ),
        (
            "(function() { var doSomething = function doSomething() {}; doSomething() }())",
            None,
            None,
            None,
        ),
        (
            "(function() { var doSomething = foo || function doSomething() {}; doSomething() }())",
            None,
            None,
            None,
        ),
        (
            "(function() { var doSomething = function doSomething() {} || foo; doSomething() }())",
            None,
            None,
            None,
        ),
        (
            "(function() { var doSomething = foo && function doSomething() {}; doSomething() }())",
            None,
            None,
            None,
        ),
        (
            "(function() { var doSomething = foo ?? function doSomething() {}; doSomething() }())",
            None,
            None,
            None,
        ), // { "ecmaVersion": 2020 },
        (
            "(function() { var doSomething = foo || (bar || function doSomething() {}); doSomething() }())",
            None,
            None,
            None,
        ),
        (
            "(function() { var doSomething = foo || (bar && function doSomething() {}); doSomething() }())",
            None,
            None,
            None,
        ),
        (
            "(function() { var doSomething = foo ? function doSomething() {} : bar; doSomething() }())",
            None,
            None,
            None,
        ),
        (
            "(function() { var doSomething = foo ? bar: function doSomething() {}; doSomething() }())",
            None,
            None,
            None,
        ),
        (
            "(function() { var doSomething = foo ? bar: (baz || function doSomething() {}); doSomething() }())",
            None,
            None,
            None,
        ),
        (
            "(function() { var doSomething = (foo ? bar: function doSomething() {}) || baz; doSomething() }())",
            None,
            None,
            None,
        ),
        (
            "(function() { var { doSomething = function doSomething() {} } = obj; doSomething() }())",
            None,
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "(function() { var { doSomething = function doSomething() {} || foo } = obj; doSomething() }())",
            None,
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "(function() { var { doSomething = foo ? function doSomething() {} : bar } = obj; doSomething() }())",
            None,
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "(function() { var { doSomething = foo ? bar : function doSomething() {} } = obj; doSomething() }())",
            None,
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "(function() { var { doSomething = foo || (bar ? baz : (qux || function doSomething() {})) || quux } = obj; doSomething() }())",
            None,
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo(doSomething = function doSomething() {}) { doSomething(); }",
            None,
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo(doSomething = function doSomething() {} || foo) { doSomething(); }",
            None,
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo(doSomething = foo ? function doSomething() {} : bar) { doSomething(); }",
            None,
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo(doSomething = foo ? bar : function doSomething() {}) { doSomething(); }",
            None,
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo(doSomething = foo || (bar ? baz : (qux || function doSomething() {})) || quux) { doSomething(); }",
            None,
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "var arguments;
            function bar() { }",
            None,
            None,
            None,
        ),
        (
            "var a=3; var b = (x) => { a++; return x + a; }; setTimeout(() => { b(a); }, 0);",
            None,
            None,
            None,
        ), // { "ecmaVersion": 6 },
        ("class A {}", None, None, None), // { "ecmaVersion": 6 },
        ("class A { constructor() { var a; } }", None, None, None), // { "ecmaVersion": 6 },
        ("(function() { var A = class A {}; })()", None, None, None), // { "ecmaVersion": 6 },
        ("(function() { var A = foo || class A {}; })()", None, None, None), // { "ecmaVersion": 6 },
        ("(function() { var A = class A {} || foo; })()", None, None, None), // { "ecmaVersion": 6 },
        ("(function() { var A = foo && class A {} || foo; })()", None, None, None), // { "ecmaVersion": 6 },
        ("(function() { var A = foo ?? class A {}; })()", None, None, None), // { "ecmaVersion": 2020 },
        ("(function() { var A = foo || (bar || class A {}); })()", None, None, None), // { "ecmaVersion": 2020 },
        ("(function() { var A = foo || (bar && class A {}); })()", None, None, None), // { "ecmaVersion": 2020 },
        ("(function() { var A = foo ? class A {} : bar; })()", None, None, None), // { "ecmaVersion": 6 },
        ("(function() { var A = foo ? bar : class A {}; })()", None, None, None), // { "ecmaVersion": 6 },
        ("(function() { var A = foo ? bar: (baz || class A {}); })()", None, None, None), // { "ecmaVersion": 6 },
        ("(function() { var A = (foo ? bar: class A {}) || baz; })()", None, None, None), // { "ecmaVersion": 6 },
        ("(function() { var { A = class A {} } = obj; }())", None, None, None), // { "ecmaVersion": 6 },
        ("(function() { var { A = class A {} || foo } = obj; }())", None, None, None), // { "ecmaVersion": 6 },
        ("(function() { var { A = foo ? class A {} : bar } = obj; }())", None, None, None), // { "ecmaVersion": 6 },
        ("(function() { var { A = foo ? bar : class A {} } = obj; }())", None, None, None), // { "ecmaVersion": 6 },
        (
            "(function() { var { A = foo || (bar ? baz : (qux || class A {})) || quux } = obj; }())",
            None,
            None,
            None,
        ), // { "ecmaVersion": 6 },
        ("function foo(A = class A {}) { doSomething(); }", None, None, None), // { "ecmaVersion": 6 },
        ("function foo(A = class A {} || foo) { doSomething(); }", None, None, None), // { "ecmaVersion": 6 },
        ("function foo(A = foo ? class A {} : bar) { doSomething(); }", None, None, None), // { "ecmaVersion": 6 },
        ("function foo(A = foo ? bar : class A {}) { doSomething(); }", None, None, None), // { "ecmaVersion": 6 },
        (
            "function foo(A = foo || (bar ? baz : (qux || class A {})) || quux) { doSomething(); }",
            None,
            None,
            None,
        ), // { "ecmaVersion": 6 },
        ("{ var a; } var a;", None, None, None), // { "ecmaVersion": 6 },
        ("{ let a; } let a;", Some(serde_json::json!([{ "hoist": "never" }])), None, None), // { "ecmaVersion": 6 },
        ("{ let a; } var a;", Some(serde_json::json!([{ "hoist": "never" }])), None, None), // { "ecmaVersion": 6 },
        ("{ let a; } function a() {}", Some(serde_json::json!([{ "hoist": "never" }])), None, None), // { "ecmaVersion": 6 },
        (
            "{ const a = 0; } const a = 1;",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        ("{ const a = 0; } var a;", Some(serde_json::json!([{ "hoist": "never" }])), None, None), // { "ecmaVersion": 6 },
        (
            "{ const a = 0; } function a() {}",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { let a; } let a;",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { let a; } var a;",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { let a; } function a() {}",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { var a; } let a;",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { var a; } var a;",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { var a; } function a() {}",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        ("function foo(a) { } let a;", Some(serde_json::json!([{ "hoist": "never" }])), None, None), // { "ecmaVersion": 6 },
        ("function foo(a) { } var a;", Some(serde_json::json!([{ "hoist": "never" }])), None, None), // { "ecmaVersion": 6 },
        (
            "function foo(a) { } function a() {}",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        ("{ let a; } let a;", None, None, None), // { "ecmaVersion": 6 },
        ("{ let a; } var a;", None, None, None), // { "ecmaVersion": 6 },
        ("{ const a = 0; } const a = 1;", None, None, None), // { "ecmaVersion": 6 },
        ("{ const a = 0; } var a;", None, None, None), // { "ecmaVersion": 6 },
        ("function foo() { let a; } let a;", None, None, None), // { "ecmaVersion": 6 },
        ("function foo() { let a; } var a;", None, None, None), // { "ecmaVersion": 6 },
        ("function foo() { var a; } let a;", None, None, None), // { "ecmaVersion": 6 },
        ("function foo() { var a; } var a;", None, None, None), // { "ecmaVersion": 6 },
        ("function foo(a) { } let a;", None, None, None), // { "ecmaVersion": 6 },
        ("function foo(a) { } var a;", None, None, None), // { "ecmaVersion": 6 },
        ("function foo() { var Object = 0; }", None, None, None),
        ("function foo() { var top = 0; }", None, None, None), // { "globals": globals.browser },
        (
            "function foo(cb) { (function (cb) { cb(42); })(cb); }",
            Some(serde_json::json!([{ "allow": ["cb"] }])),
            None,
            None,
        ),
        ("class C { foo; foo() { let foo; } }", None, None, None), // { "ecmaVersion": 2022 },
        ("class C { static { var x; } static { var x; } }", None, None, None), // { "ecmaVersion": 2022 },
        ("class C { static { let x; } static { let x; } }", None, None, None), // { "ecmaVersion": 2022 },
        ("class C { static { var x; { var x; /* redeclaration */ } } }", None, None, None), // { "ecmaVersion": 2022 },
        ("class C { static { { var x; } { var x; /* redeclaration */ } } }", None, None, None), // { "ecmaVersion": 2022 },
        ("class C { static { { let x; } { let x; } } }", None, None, None), // { "ecmaVersion": 2022 },
        (
            "const a = [].find(a => a)",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const a = [].find(function(a) { return a; })",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const [a = [].find(a => true)] = dummy",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const { a = [].find(a => true) } = dummy",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function func(a = [].find(a => true)) {}",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "for (const a in [].find(a => true)) {}",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "for (const a of [].find(a => true)) {}",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const a = [].map(a => true).filter(a => a === 'b')",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const a = [].map(a => true).filter(a => a === 'b').find(a => a === 'c')",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const { a } = (({ a }) => ({ a }))();",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const person = people.find(item => {const person = item.name; return person === 'foo'})",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "var y = bar || foo(y => y);",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "var y = bar && foo(y => y);",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "var z = bar(foo(z => z));",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "var z = boo(bar(foo(z => z)));",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "var match = function (person) { return person.name === 'foo'; };
            const person = [].find(match);",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const a = foo(x || (a => {}))",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const { a = 1 } = foo(a => {})",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const person = {...people.find((person) => person.firstName.startsWith('s'))}",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 2021 },
        (
            "const person = { firstName: people.filter((person) => person.firstName.startsWith('s')).map((person) => person.firstName)[0]}",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 2021 },
        (
            "() => { const y = foo(y => y); }",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const x = (x => x)()",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "var y = bar || (y => y)();",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "var y = bar && (y => y)();",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "var x = (x => x)((y => y)());",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const { a = 1 } = (a => {})()",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "() => { const y = (y => y)(); }",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        ("const [x = y => y] = [].map(y => y)", None, None, None),          // { "ecmaVersion": 6 },
        ("function foo<T = (arg: any) => any>(arg: T) {}", None, None, None),
        ("function foo<T = ([arg]: [any]) => any>(arg: T) {}", None, None, None),
        ("function foo<T = ({ args }: { args: any }) => any>(arg: T) {}", None, None, None),
        ("function foo<T = (...args: any[]) => any>(fn: T, args: any[]) {}", None, None, None),
        (
            "function foo<T extends (...args: any[]) => any>(fn: T, args: any[]) {}",
            None,
            None,
            None,
        ),
        (
            "function foo<T extends (...args: any[]) => any>(fn: T, ...args: any[]) {}",
            None,
            None,
            None,
        ),
        ("function foo<T extends ([args]: any[]) => any>(fn: T, args: any[]) {}", None, None, None),
        (
            "function foo<T extends ([...args]: any[]) => any>(fn: T, args: any[]) {}",
            None,
            None,
            None,
        ),
        (
            "function foo<T extends ({ args }: { args: any }) => any>(fn: T, args: any) {}",
            None,
            None,
            None,
        ),
        (
            "
              function foo<T extends (id: string, ...args: any[]) => any>(
                fn: T,
                ...args: any[]
              ) {}
                  ",
            None,
            None,
            None,
        ),
        (
            "
              type Args = 1;
              function foo<T extends (Args: any) => void>(arg: T) {}
                  ",
            None,
            None,
            None,
        ),
        (
            "
              export type ArrayInput<Func> = Func extends (arg0: Array<infer T>) => any
                ? T[]
                : Func extends (...args: infer T) => any
                  ? T
                  : never;
                  ",
            None,
            None,
            None,
        ),
        (
            "
              function foo() {
                var Object = 0;
              }
                  ",
            None,
            None,
            None,
        ),
        (
            "
              function test(this: Foo) {
                function test2(this: Bar) {}
              }
                  ",
            None,
            None,
            None,
        ),
        (
            "
              class Foo {
                prop = 1;
              }
              namespace Foo {
                export const v = 2;
              }
                  ",
            None,
            None,
            None,
        ),
        (
            "
              function Foo() {}
              namespace Foo {
                export const v = 2;
              }
                  ",
            None,
            None,
            None,
        ),
        (
            "
              class Foo {
                prop = 1;
              }
              interface Foo {
                prop2: string;
              }
                  ",
            None,
            None,
            None,
        ),
        (
            "
              import type { Foo } from 'bar';

              declare module 'bar' {
                export interface Foo {
                  x: string;
                }
              }
                  ",
            None,
            None,
            None,
        ),
        (
            "
              const x = 1;
              type x = string;
                  ",
            None,
            None,
            None,
        ),
        (
            "
              const x = 1;
              {
                type x = string;
              }
                  ",
            None,
            None,
            None,
        ),
        (
            "
              type Foo = 1;
                    ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": true }])),
            None,
            None,
        ), // { "globals": { "Foo": "writable", }, },
        (
            "
              type Foo = 1;
                    ",
            Some(
                serde_json::json!([ { "builtinGlobals": false, "ignoreTypeValueShadow": false, }, ]),
            ),
            None,
            None,
        ), // { "globals": { "Foo": "writable", }, },
        (
            "
              enum Direction {
                left = 'left',
                right = 'right',
              }
                  ",
            None,
            None,
            None,
        ),
        (
            "
              const test = 1;
              type Fn = (test: string) => typeof test;
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
              type Fn = (Foo: string) => typeof Foo;
                    ",
            Some(
                serde_json::json!([ { "builtinGlobals": false, "ignoreFunctionTypeParameterNameValueShadow": true, }, ]),
            ),
            None,
            None,
        ), // { "globals": { "Foo": "writable", }, },
        (
            "
              const arg = 0;

              interface Test {
                (arg: string): typeof arg;
              }
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
              const arg = 0;

              interface Test {
                p1(arg: string): typeof arg;
              }
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
              const arg = 0;

              declare function test(arg: string): typeof arg;
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
              const arg = 0;

              declare const test: (arg: string) => typeof arg;
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
              const arg = 0;

              declare class Test {
                p1(arg: string): typeof arg;
              }
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
              const arg = 0;

              declare const Test: {
                new (arg: string): typeof arg;
              };
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
              const arg = 0;

              type Bar = new (arg: number) => typeof arg;
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
              const arg = 0;

              declare namespace Lib {
                function test(arg: string): typeof arg;
              }
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
                      declare global {
                        interface ArrayConstructor {}
                      }
                      export {};
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            None,
        ),
        (
            "
                    declare global {
                      const a: string;

                      namespace Foo {
                        const a: number;
                      }
                    }
                    export {};
                  ",
            None,
            None,
            None,
        ),
        (
            "
                      declare global {
                        type A = 'foo';

                        namespace Foo {
                          type A = 'bar';
                        }
                      }
                      export {};
                    ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": false }])),
            None,
            None,
        ),
        (
            "
                      declare global {
                        const foo: string;
                        type Fn = (foo: number) => void;
                      }
                      export {};
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
              export class Wrapper<Wrapped> {
                private constructor(private readonly wrapped: Wrapped) {}

                unwrap(): Wrapped {
                  return this.wrapped;
                }

                static create<Wrapped>(wrapped: Wrapped) {
                  return new Wrapper<Wrapped>(wrapped);
                }
              }
                  ",
            None,
            None,
            None,
        ),
        (
            "
              function makeA() {
                return class A<T> {
                  constructor(public value: T) {}

                  static make<T>(value: T) {
                    return new A<T>(value);
                  }
                };
              }
                  ",
            None,
            None,
            None,
        ),
        (
            "
              import type { foo } from './foo';
              type bar = number;

              // 'foo' is already declared in the upper scope
              // 'bar' is fine
              function doThing(foo: number, bar: number) {}
                    ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": true }])),
            None,
            None,
        ),
        (
            "
              import { type foo } from './foo';

              // 'foo' is already declared in the upper scope
              function doThing(foo: number) {}
                    ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": true }])),
            None,
            None,
        ),
        (
            "const a = [].find(a => a);",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "
              const a = [].find(function (a) {
                return a;
              });
                    ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const [a = [].find(a => true)] = dummy;",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const { a = [].find(a => true) } = dummy;",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "function func(a = [].find(a => true)) {}",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "
              for (const a in [].find(a => true)) {
              }
                    ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "
              for (const a of [].find(a => true)) {
              }
                    ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const a = [].map(a => true).filter(a => a === 'b');",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "
              const a = []
                .map(a => true)
                .filter(a => a === 'b')
                .find(a => a === 'c');
                    ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const { a } = (({ a }) => ({ a }))();",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "
              const person = people.find(item => {
                const person = item.name;
                return person === 'foo';
              });
                    ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "var y = bar || foo(y => y);",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "var y = bar && foo(y => y);",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "var z = bar(foo(z => z));",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "var z = boo(bar(foo(z => z)));",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "
              var match = function (person) {
                return person.name === 'foo';
              };
              const person = [].find(match);
                    ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const a = foo(x || (a => {}));",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const { a = 1 } = foo(a => {});",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const person = { ...people.find(person => person.firstName.startsWith('s')) };",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "parserOptions": { "ecmaVersion": 2021 } },
        (
            "
              const person = {
                firstName: people
                  .filter(person => person.firstName.startsWith('s'))
                  .map(person => person.firstName)[0],
              };
                    ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "parserOptions": { "ecmaVersion": 2021 } },
        (
            "
              () => {
                const y = foo(y => y);
              };
                    ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const x = (x => x)();",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "var y = bar || (y => y)();",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "var y = bar && (y => y)();",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "var x = (x => x)((y => y)());",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const { a = 1 } = (a => {})();",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "
              () => {
                const y = (y => y)();
              };
                    ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        ("const [x = y => y] = [].map(y => y);", None, None, None),
        (
            "
              type Foo<A> = 1;
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ),
        (
            "
              interface Foo<A> {}
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ),
        (
            "
              interface Foo<A> {}
              interface A {}
                    ",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ),
        (
            "
              type Foo<A> = 1;
              interface A {}
                    ",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ),
        (
            "
              {
                type A = 1;
              }
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ),
        (
            "
              {
                interface Foo<A> {}
              }
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ),
        (
            "
              type Foo<A> = 1;
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "functions" }])),
            None,
            None,
        ),
        (
            "
              interface Foo<A> {}
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "functions" }])),
            None,
            None,
        ),
        (
            "
              interface Foo<A> {}
              interface A {}
                    ",
            Some(serde_json::json!([{ "hoist": "functions" }])),
            None,
            None,
        ),
        (
            "
              type Foo<A> = 1;
              interface A {}
                    ",
            Some(serde_json::json!([{ "hoist": "functions" }])),
            None,
            None,
        ),
        (
            "
              {
                type A = 1;
              }
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "functions" }])),
            None,
            None,
        ),
        (
            "
              {
                interface Foo<A> {}
              }
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "functions" }])),
            None,
            None,
        ),
        (
            "
              import type { Foo } from 'bar';

              declare module 'bar' {
                export type Foo = string;
              }
                    ",
            None,
            None,
            None,
        ),
        (
            "
              import type { Foo } from 'bar';

              declare module 'bar' {
                interface Foo {
                  x: string;
                }
              }
                    ",
            None,
            None,
            None,
        ),
        (
            "
              import { type Foo } from 'bar';

              declare module 'bar' {
                export type Foo = string;
              }
                    ",
            None,
            None,
            None,
        ),
        (
            "
              import { type Foo } from 'bar';

              declare module 'bar' {
                export interface Foo {
                  x: string;
                }
              }
                    ",
            None,
            None,
            None,
        ),
        (
            "
              import { type Foo } from 'bar';

              declare module 'bar' {
                type Foo = string;
              }
                    ",
            None,
            None,
            None,
        ),
        (
            "
              import { type Foo } from 'bar';

              declare module 'bar' {
                interface Foo {
                  x: string;
                }
              }
                    ",
            None,
            None,
            None,
        ),
        (
            "
              declare const foo1: boolean;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "foo1": false, }, },
        (
            "
              declare let foo2: boolean;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "foo2": false, }, },
        (
            "
              declare var foo3: boolean;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "foo3": false, }, },
        (
            "
              function foo4(name: string): void;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "foo4": false, }, },
        (
            "
              declare class Foopy1 {
                name: string;
              }
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "Foopy1": false, }, },
        (
            "
              declare interface Foopy2 {
                name: string;
              }
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "Foopy2": false, }, },
        (
            "
              declare type Foopy3 = {
                x: number;
              };
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "Foopy3": false, }, },
        (
            "
              declare enum Foopy4 {
                x,
              }
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "Foopy4": false, }, },
        (
            "
              declare namespace Foopy5 {}
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "Foopy5": false, }, },
        (
            "
              declare;
              foo5: boolean;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "foo5": false, }, },
        (
            "
              declare const foo1: boolean;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.cts")),
        ), // { "globals": { "foo1": false, }, },
        (
            "
              declare let foo2: boolean;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.cts")),
        ), // { "globals": { "foo2": false, }, },
        (
            "
              declare var foo3: boolean;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.cts")),
        ), // { "globals": { "foo3": false, }, },
        (
            "
              function foo4(name: string): void;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.cts")),
        ), // { "globals": { "foo4": false, }, },
        (
            "
              declare class Foopy1 {
                name: string;
              }
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.cts")),
        ), // { "globals": { "Foopy1": false, }, },
        (
            "
              declare interface Foopy2 {
                name: string;
              }
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.cts")),
        ), // { "globals": { "Foopy2": false, }, },
        (
            "
              declare type Foopy3 = {
                x: number;
              };
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.cts")),
        ), // { "globals": { "Foopy3": false, }, },
        (
            "
              declare enum Foopy4 {
                x,
              }
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.cts")),
        ), // { "globals": { "Foopy4": false, }, },
        (
            "
              declare namespace Foopy5 {}
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.cts")),
        ), // { "globals": { "Foopy5": false, }, },
        (
            "
              declare;
              foo5: boolean;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.cts")),
        ), // { "globals": { "foo5": false, }, },
        (
            "
              declare const foo1: boolean;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.mts")),
        ), // { "globals": { "foo1": false, }, },
        (
            "
              declare let foo2: boolean;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.mts")),
        ), // { "globals": { "foo2": false, }, },
        (
            "
              declare var foo3: boolean;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.mts")),
        ), // { "globals": { "foo3": false, }, },
        (
            "
              function foo4(name: string): void;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.mts")),
        ), // { "globals": { "foo4": false, }, },
        (
            "
              declare class Foopy1 {
                name: string;
              }
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.mts")),
        ), // { "globals": { "Foopy1": false, }, },
        (
            "
              declare interface Foopy2 {
                name: string;
              }
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.mts")),
        ), // { "globals": { "Foopy2": false, }, },
        (
            "
              declare type Foopy3 = {
                x: number;
              };
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.mts")),
        ), // { "globals": { "Foopy3": false, }, },
        (
            "
              declare enum Foopy4 {
                x,
              }
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.mts")),
        ), // { "globals": { "Foopy4": false, }, },
        (
            "
              declare namespace Foopy5 {}
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.mts")),
        ), // { "globals": { "Foopy5": false, }, },
        (
            "
              declare;
              foo5: boolean;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.mts")),
        ), // { "globals": { "foo5": false, }, }
    ];

    let fail = vec![
        ("function a(x) { var b = function c() { var x = 'foo'; }; }", None, None, None),
        ("var a = (x) => { var b = () => { var x = 'foo'; }; }", None, None, None), // { "ecmaVersion": 6 },
        ("function a(x) { var b = function () { var x = 'foo'; }; }", None, None, None),
        ("var x = 1; function a(x) { return ++x; }", None, None, None),
        ("var a=3; function b() { var a=10; }", None, None, None),
        (
            "var a=3; function b() { var a=10; }; setTimeout(function() { b(); }, 0);",
            None,
            None,
            None,
        ),
        (
            "var a=3; function b() { var a=10; var b=0; }; setTimeout(function() { b(); }, 0);",
            None,
            None,
            None,
        ),
        ("var x = 1; { let x = 2; }", None, None, None), // { "ecmaVersion": 6 },
        ("let x = 1; { const x = 2; }", None, None, None), // { "ecmaVersion": 6 },
        ("{ let a; } function a() {}", None, None, None), // { "ecmaVersion": 6 },
        ("{ const a = 0; } function a() {}", None, None, None), // { "ecmaVersion": 6 },
        ("function foo() { let a; } function a() {}", None, None, None), // { "ecmaVersion": 6 },
        ("function foo() { var a; } function a() {}", None, None, None), // { "ecmaVersion": 6 },
        ("function foo(a) { } function a() {}", None, None, None), // { "ecmaVersion": 6 },
        ("{ let a; } let a;", Some(serde_json::json!([{ "hoist": "all" }])), None, None), // { "ecmaVersion": 6 },
        ("{ let a; } var a;", Some(serde_json::json!([{ "hoist": "all" }])), None, None), // { "ecmaVersion": 6 },
        ("{ let a; } function a() {}", Some(serde_json::json!([{ "hoist": "all" }])), None, None), // { "ecmaVersion": 6 },
        (
            "{ const a = 0; } const a = 1;",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        ("{ const a = 0; } var a;", Some(serde_json::json!([{ "hoist": "all" }])), None, None), // { "ecmaVersion": 6 },
        (
            "{ const a = 0; } function a() {}",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { let a; } let a;",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { let a; } var a;",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { let a; } function a() {}",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { var a; } let a;",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { var a; } var a;",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { var a; } function a() {}",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        ("function foo(a) { } let a;", Some(serde_json::json!([{ "hoist": "all" }])), None, None), // { "ecmaVersion": 6 },
        ("function foo(a) { } var a;", Some(serde_json::json!([{ "hoist": "all" }])), None, None), // { "ecmaVersion": 6 },
        (
            "function foo(a) { } function a() {}",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        ("(function a() { function a(){} })()", None, None, None),
        ("(function a() { class a{} })()", None, None, None), // { "ecmaVersion": 6 },
        ("(function a() { (function a(){}); })()", None, None, None),
        ("(function a() { (class a{}); })()", None, None, None), // { "ecmaVersion": 6 },
        ("(function() { var a = function(a) {}; })()", None, None, None),
        ("(function() { var a = function() { function a() {} }; })()", None, None, None),
        ("(function() { var a = function() { class a{} }; })()", None, None, None), // { "ecmaVersion": 6 },
        ("(function() { var a = function() { (function a() {}); }; })()", None, None, None),
        ("(function() { var a = function() { (class a{}); }; })()", None, None, None), // { "ecmaVersion": 6 },
        ("(function() { var a = class { constructor() { class a {} } }; })()", None, None, None), // { "ecmaVersion": 6 },
        ("class A { constructor() { var A; } }", None, None, None), // { "ecmaVersion": 6 },
        ("(function a() { function a(){ function a(){} } })()", None, None, None),
        (
            "function foo() { var Object = 0; }",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            None,
        ),
        (
            "function foo() { var top = 0; }",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            None,
        ), // { "globals": globals.browser },
        ("var Object = 0;", Some(serde_json::json!([{ "builtinGlobals": true }])), None, None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var top = 0;", Some(serde_json::json!([{ "builtinGlobals": true }])), None, None), // { "ecmaVersion": 6, "sourceType": "module", "globals": globals.browser, },
        ("var Object = 0;", Some(serde_json::json!([{ "builtinGlobals": true }])), None, None), // { "parserOptions": { "ecmaFeatures": { "globalReturn": true } }, },
        ("var top = 0;", Some(serde_json::json!([{ "builtinGlobals": true }])), None, None), // { "parserOptions": { "ecmaFeatures": { "globalReturn": true } }, "globals": globals.browser, },
        ("function foo(cb) { (function (cb) { cb(42); })(cb); }", None, None, None),
        ("class C { static { let a; { let a; } } }", None, None, None), // { "ecmaVersion": 2022 },
        ("class C { static { var C; } }", None, None, None),            // { "ecmaVersion": 2022 },
        ("class C { static { let C; } }", None, None, None),            // { "ecmaVersion": 2022 },
        ("var a; class C { static { var a; } }", None, None, None),     // { "ecmaVersion": 2022 },
        (
            "class C { static { var a; } } var a;",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { let a; } } let a;",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { var a; } } let a;",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "ecmaVersion": 2022 },
        ("class C { static { var a; class D { static { var a; } } } }", None, None, None), // { "ecmaVersion": 2022 },
        ("class C { static { let a; class D { static { let a; } } } }", None, None, None), // { "ecmaVersion": 2022 },
        (
            "let x = foo((x,y) => {});
            let y;",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const a = fn(()=>{ class C { fn () { const a = 42; return a } } return new C() })",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function a() {}
            foo(a => {});",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const a = fn(()=>{ function C() { this.fn=function() { const a = 42; return a } } return new C() });",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const x = foo(() => { const bar = () => { return x => {}; }; return bar; });",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const x = foo(() => { return { bar(x) {} }; });",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const x = () => { foo(x => x); }",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const foo = () => { let x; bar(x => x); }",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "foo(() => { const x = x => x; });",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const foo = (x) => { bar(x => {}) }",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "let x = ((x,y) => {})();
            let y;",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const a = (()=>{ class C { fn () { const a = 42; return a } } return new C() })()",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        (
            "const x = () => { (x => x)(); }",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6 },
        // The following tests are disabled as the behaviour of eslint vs typescript-eslint
        // differs. We are following typescript-eslint's behaviour, but these tests are left
        // here for clarity.
        // (
        //     "let x = false; export const a = wrap(function a() { if (!x) { x = true; a(); } });",
        //     Some(serde_json::json!([{ "hoist": "all" }])),
        //     None,
        //     None,
        // ), // { "ecmaVersion": 6, "sourceType": "module" },
        // ("const a = wrap(function a() {});", None, None, None), // { "ecmaVersion": 6 },
        // ("const a = foo || wrap(function a() {});", None, None, None), // { "ecmaVersion": 6 },
        // ("const { a = wrap(function a() {}) } = obj;", None, None, None), // { "ecmaVersion": 6 },
        // ("const { a = foo || wrap(function a() {}) } = obj;", None, None, None), // { "ecmaVersion": 6 },
        ("const { a = foo, b = function a() {} } = {}", None, None, None), // { "ecmaVersion": 6 },
        ("const { A = Foo, B = class A {} } = {}", None, None, None),      // { "ecmaVersion": 6 },
        ("function foo(a = wrap(function a() {})) {}", None, None, None),  // { "ecmaVersion": 6 },
        ("function foo(a = foo || wrap(function a() {})) {}", None, None, None), // { "ecmaVersion": 6 },
        // ("const A = wrap(class A {});", None, None, None), // { "ecmaVersion": 6 },
        // ("const A = foo || wrap(class A {});", None, None, None), // { "ecmaVersion": 6 },
        // ("const { A = wrap(class A {}) } = obj;", None, None, None), // { "ecmaVersion": 6 },
        // ("const { A = foo || wrap(class A {}) } = obj;", None, None, None), // { "ecmaVersion": 6 },
        ("function foo(A = wrap(class A {})) {}", None, None, None), // { "ecmaVersion": 6 },
        ("function foo(A = foo || wrap(class A {})) {}", None, None, None), // { "ecmaVersion": 6 },
        // ("var a = function a() {} ? foo : bar", None, None, None),
        // ("var A = class A {} ? foo : bar", None, None, None), // { "ecmaVersion": 6, },
        (
            "(function Array() {})",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            None,
        ), // { "ecmaVersion": 6, "sourceType": "module", },
        ("let a; { let b = (function a() {}) }", None, None, None), // { "ecmaVersion": 6, },
        ("let a = foo; { let b = (function a() {}) }", None, None, None), // { "ecmaVersion": 6, },
        (
            "
              type T = 1;
              {
                type T = 2;
              }
                    ",
            None,
            None,
            None,
        ),
        (
            "
              type T = 1;
              function foo<T>(arg: T) {}
                    ",
            None,
            None,
            None,
        ),
        (
            "
              function foo<T>() {
                return function <T>() {};
              }
                    ",
            None,
            None,
            None,
        ),
        (
            "
              type T = string;
              function foo<T extends (arg: any) => void>(arg: T) {}
                    ",
            None,
            None,
            None,
        ),
        (
            "
              const x = 1;
              {
                type x = string;
              }
                    ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": false }])),
            None,
            None,
        ),
        (
            "
              type Foo = 1;
                    ",
            Some(
                serde_json::json!([ { "builtinGlobals": true, "ignoreTypeValueShadow": false, }, ]),
            ),
            Some(serde_json::json!({ "globals": { "Foo": "writable" } })),
            None,
        ), // { "globals": { "Foo": "writable", }, },
        (
            "
              const test = 1;
              type Fn = (test: string) => typeof test;
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
              type Fn = (Foo: string) => typeof Foo;
                    ",
            Some(
                serde_json::json!([ { "builtinGlobals": true, "ignoreFunctionTypeParameterNameValueShadow": false, }, ]),
            ),
            Some(serde_json::json!({ "globals": { "Foo": "writable" } })),
            None,
        ), // { "globals": { "Foo": "writable", }, },
        (
            "
              const arg = 0;

              interface Test {
                (arg: string): typeof arg;
              }
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
              const arg = 0;

              interface Test {
                p1(arg: string): typeof arg;
              }
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
              const arg = 0;

              declare function test(arg: string): typeof arg;
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
              const arg = 0;

              declare const test: (arg: string) => typeof arg;
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
              const arg = 0;

              declare class Test {
                p1(arg: string): typeof arg;
              }
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
              const arg = 0;

              declare const Test: {
                new (arg: string): typeof arg;
              };
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
              const arg = 0;

              type Bar = new (arg: number) => typeof arg;
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
              const arg = 0;

              declare namespace Lib {
                function test(arg: string): typeof arg;
              }
                    ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
              import type { foo } from './foo';
              function doThing(foo: number) {}
                    ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": false }])),
            None,
            None,
        ),
        (
            "
              import { type foo } from './foo';
              function doThing(foo: number) {}
                    ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": false }])),
            None,
            None,
        ),
        (
            "
              import { foo } from './foo';
              function doThing(foo: number, bar: number) {}
                    ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": true }])),
            None,
            None,
        ),
        (
            "
              interface Foo {}

              declare module 'bar' {
                export interface Foo {
                  x: string;
                }
              }
                    ",
            None,
            None,
            None,
        ),
        (
            "
              import type { Foo } from 'bar';

              declare module 'baz' {
                export interface Foo {
                  x: string;
                }
              }
                    ",
            None,
            None,
            None,
        ),
        (
            "
              import { type Foo } from 'bar';

              declare module 'baz' {
                export interface Foo {
                  x: string;
                }
              }
                    ",
            None,
            None,
            None,
        ),
        (
            "
              let x = foo((x, y) => {});
              let y;
                    ",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "parserOptions": { "ecmaVersion": 6 } },
        (
            "
              let x = foo((x, y) => {});
              let y;
                    ",
            Some(serde_json::json!([{ "hoist": "functions" }])),
            None,
            None,
        ), // { "parserOptions": { "ecmaVersion": 6 } },
        (
            "
              type Foo<A> = 1;
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "types" }])),
            None,
            None,
        ),
        (
            "
              interface Foo<A> {}
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "types" }])),
            None,
            None,
        ),
        (
            "
              interface Foo<A> {}
              interface A {}
                    ",
            Some(serde_json::json!([{ "hoist": "types" }])),
            None,
            None,
        ),
        (
            "
              type Foo<A> = 1;
              interface A {}
                    ",
            Some(serde_json::json!([{ "hoist": "types" }])),
            None,
            None,
        ),
        (
            "
              {
                type A = 1;
              }
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "types" }])),
            None,
            None,
        ),
        (
            "
              {
                interface A {}
              }
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "types" }])),
            None,
            None,
        ),
        (
            "
              type Foo<A> = 1;
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ),
        (
            "
              interface Foo<A> {}
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ),
        (
            "
              interface Foo<A> {}
              interface A {}
                    ",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ),
        (
            "
              type Foo<A> = 1;
              interface A {}
                    ",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ),
        (
            "
              {
                type A = 1;
              }
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ),
        (
            "
              {
                interface A {}
              }
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ),
        (
            "
              type Foo<A> = 1;
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "functions-and-types" }])),
            None,
            None,
        ),
        (
            "
                if (true) {
                    const foo = 6;
                }

                function foo() { }
                    ",
            Some(serde_json::json!([{ "hoist": "functions-and-types" }])),
            None,
            None,
        ),
        (
            "
                // types
                type Bar<Foo> = 1;
                type Foo = 1;

                // functions
                if (true) {
                    const b = 6;
                }

                function b() { }
                    ",
            Some(serde_json::json!([{ "hoist": "functions-and-types" }])),
            None,
            None,
        ),
        (
            "
                // types
                type Bar<Foo> = 1;
                type Foo = 1;

                // functions
                if (true) {
                    const b = 6;
                }

                function b() { }
                    ",
            Some(serde_json::json!([{ "hoist": "types" }])),
            None,
            None,
        ),
        (
            "
              interface Foo<A> {}
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "functions-and-types" }])),
            None,
            None,
        ),
        (
            "
              interface Foo<A> {}
              interface A {}
                    ",
            Some(serde_json::json!([{ "hoist": "functions-and-types" }])),
            None,
            None,
        ),
        (
            "
              type Foo<A> = 1;
              interface A {}
                    ",
            Some(serde_json::json!([{ "hoist": "functions-and-types" }])),
            None,
            None,
        ),
        (
            "
              {
                type A = 1;
              }
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "functions-and-types" }])),
            None,
            None,
        ),
        (
            "
              {
                interface A {}
              }
              type A = 1;
                    ",
            Some(serde_json::json!([{ "hoist": "functions-and-types" }])),
            None,
            None,
        ),
        (
            "
              function foo<T extends (...args: any[]) => any>(fn: T, args: any[]) {}
                    ",
            Some(
                serde_json::json!([ { "builtinGlobals": true, "ignoreTypeValueShadow": false, }, ]),
            ),
            Some(serde_json::json!({ "globals": { "args": "writable" } })),
            None,
        ), // { "globals": { "args": "writable", }, },
        (
            "
              declare const has = (environment: 'dev' | 'prod' | 'test') => boolean;
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            Some(serde_json::json!({ "globals": { "has": false } })),
            None,
        ), // { "globals": { "has": false, }, },
        (
            "
              declare const has: (environment: 'dev' | 'prod' | 'test') => boolean;
              const fn = (has: string) => {};
                    ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("foo.d.ts")),
        ), // { "globals": { "has": false, }, },
        (
            "
                        const A = 2;
                        enum Test {
                            A = 1,
                            B = A,
                        }
                    ",
            None,
            None,
            None,
        ),
    ];

    Tester::new(NoShadow::NAME, NoShadow::PLUGIN, pass, fail).test_and_snapshot();
}

#[test]
fn test_typescript_eslint() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        ("function foo<T = (arg: any) => any>(arg: T) {}", None, None, None),
        ("function foo<T = ([arg]: [any]) => any>(arg: T) {}", None, None, None),
        ("function foo<T = ({ args }: { args: any }) => any>(arg: T) {}", None, None, None),
        ("function foo<T = (...args: any[]) => any>(fn: T, args: any[]) {}", None, None, None),
        (
            "function foo<T extends (...args: any[]) => any>(fn: T, args: any[]) {}",
            None,
            None,
            None,
        ),
        (
            "function foo<T extends (...args: any[]) => any>(fn: T, ...args: any[]) {}",
            None,
            None,
            None,
        ),
        ("function foo<T extends ([args]: any[]) => any>(fn: T, args: any[]) {}", None, None, None),
        (
            "function foo<T extends ([...args]: any[]) => any>(fn: T, args: any[]) {}",
            None,
            None,
            None,
        ),
        (
            "function foo<T extends ({ args }: { args: any }) => any>(fn: T, args: any) {}",
            None,
            None,
            None,
        ),
        (
            "
            import { memo } from 'react';

            const FooBarComponent = memo(function FooBarComponent() {
              return <div>Foo</div>;
            });
                ",
            None,
            None,
            Some(PathBuf::from("test.tsx")),
        ),
        (
            "
            function foo<T extends (id: string, ...args: any[]) => any>(
              fn: T,
              ...args: any[]
            ) {}
                ",
            None,
            None,
            None,
        ),
        (
            "
            type Args = 1;
            function foo<T extends (Args: any) => void>(arg: T) {}
                ",
            None,
            None,
            None,
        ),
        (
            "
            export type ArrayInput<Func> = Func extends (arg0: Array<infer T>) => any
              ? T[]
              : Func extends (...args: infer T) => any
                ? T
                : never;
                ",
            None,
            None,
            None,
        ),
        (
            "
            function foo() {
              var Object = 0;
            }
                ",
            None,
            None,
            None,
        ),
        (
            "
            function test(this: Foo) {
              function test2(this: Bar) {}
            }
                ",
            None,
            None,
            None,
        ),
        (
            "
            class Foo {
              prop = 1;
            }
            namespace Foo {
              export const v = 2;
            }
                ",
            None,
            None,
            None,
        ),
        (
            "
            function Foo() {}
            namespace Foo {
              export const v = 2;
            }
                ",
            None,
            None,
            None,
        ),
        (
            "
            class Foo {
              prop = 1;
            }
            interface Foo {
              prop2: string;
            }
                ",
            None,
            None,
            None,
        ),
        (
            "
            import type { Foo } from 'bar';

            declare module 'bar' {
              export interface Foo {
                x: string;
              }
            }
                ",
            None,
            None,
            None,
        ),
        (
            "
            const x = 1;
            type x = string;
                ",
            None,
            None,
            None,
        ),
        (
            "
            const x = 1;
            {
              type x = string;
            }
                ",
            None,
            None,
            None,
        ),
        (
            "
            type Foo = 1;
                  ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": true }])),
            None,
            None,
        ), // { "globals": { "Foo": "writable", }, },
        (
            "
            type Foo = 1;
                  ",
            Some(
                serde_json::json!([ { "builtinGlobals": false, "ignoreTypeValueShadow": false, }, ]),
            ),
            None,
            None,
        ), // { "globals": { "Foo": "writable", }, },
        (
            "
            enum Direction {
              left = 'left',
              right = 'right',
            }
                ",
            None,
            None,
            None,
        ),
        (
            "
            const test = 1;
            type Fn = (test: string) => typeof test;
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
            type Fn = (Foo: string) => typeof Foo;
                  ",
            Some(
                serde_json::json!([ { "builtinGlobals": false, "ignoreFunctionTypeParameterNameValueShadow": true, }, ]),
            ),
            None,
            None,
        ), // { "globals": { "Foo": "writable", }, },
        (
            "
            const arg = 0;

            interface Test {
              (arg: string): typeof arg;
            }
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
            const arg = 0;

            interface Test {
              p1(arg: string): typeof arg;
            }
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
            const arg = 0;

            declare function test(arg: string): typeof arg;
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
            const arg = 0;

            declare const test: (arg: string) => typeof arg;
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
            const arg = 0;

            declare class Test {
              p1(arg: string): typeof arg;
            }
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
            const arg = 0;

            declare const Test: {
              new (arg: string): typeof arg;
            };
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
            const arg = 0;

            type Bar = new (arg: number) => typeof arg;
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
            const arg = 0;

            declare namespace Lib {
              function test(arg: string): typeof arg;
            }
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": true }])),
            None,
            None,
        ),
        (
            "
                    declare global {
                      interface ArrayConstructor {}
                    }
                    export {};
                  ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            None,
        ),
        (
            "
                  declare global {
                    const a: string;

                    namespace Foo {
                      const a: number;
                    }
                  }
                  export {};
                ",
            None,
            None,
            None,
        ),
        (
            "
                    declare global {
                      type A = 'foo';

                      namespace Foo {
                        type A = 'bar';
                      }
                    }
                    export {};
                  ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": false }])),
            None,
            None,
        ),
        (
            "
                    declare global {
                      const foo: string;
                      type Fn = (foo: number) => void;
                    }
                    export {};
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
            export class Wrapper<Wrapped> {
              private constructor(private readonly wrapped: Wrapped) {}

              unwrap(): Wrapped {
                return this.wrapped;
              }

              static create<Wrapped>(wrapped: Wrapped) {
                return new Wrapper<Wrapped>(wrapped);
              }
            }
                ",
            None,
            None,
            None,
        ),
        (
            "
            function makeA() {
              return class A<T> {
                constructor(public value: T) {}

                static make<T>(value: T) {
                  return new A<T>(value);
                }
              };
            }
                ",
            None,
            None,
            None,
        ),
        (
            "
            import type { foo } from './foo';
            type bar = number;

            // 'foo' is already declared in the upper scope
            // 'bar' is fine
            function doThing(foo: number, bar: number) {}
                  ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": true }])),
            None,
            None,
        ),
        (
            "
            import { type foo } from './foo';

            // 'foo' is already declared in the upper scope
            function doThing(foo: number) {}
                  ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": true }])),
            None,
            None,
        ),
        (
            "const a = [].find(a => a);",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "
            const a = [].find(function (a) {
              return a;
            });
                  ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const [a = [].find(a => true)] = dummy;",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const { a = [].find(a => true) } = dummy;",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "function func(a = [].find(a => true)) {}",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "
            for (const a in [].find(a => true)) {
            }
                  ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "
            for (const a of [].find(a => true)) {
            }
                  ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const a = [].map(a => true).filter(a => a === 'b');",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "
            const a = []
              .map(a => true)
              .filter(a => a === 'b')
              .find(a => a === 'c');
                  ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const { a } = (({ a }) => ({ a }))();",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "
            const person = people.find(item => {
              const person = item.name;
              return person === 'foo';
            });
                  ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "var y = bar || foo(y => y);",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "var y = bar && foo(y => y);",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "var z = bar(foo(z => z));",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "var z = boo(bar(foo(z => z)));",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "
            var match = function (person) {
              return person.name === 'foo';
            };
            const person = [].find(match);
                  ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const a = foo(x || (a => {}));",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const { a = 1 } = foo(a => {});",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const person = { ...people.find(person => person.firstName.startsWith('s')) };",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "parserOptions": { "ecmaVersion": 2021 } },
        (
            "
            const person = {
              firstName: people
                .filter(person => person.firstName.startsWith('s'))
                .map(person => person.firstName)[0],
            };
                  ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ), // { "parserOptions": { "ecmaVersion": 2021 } },
        (
            "
            () => {
              const y = foo(y => y);
            };
                  ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const x = (x => x)();",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "var y = bar || (y => y)();",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "var y = bar && (y => y)();",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "var x = (x => x)((y => y)());",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "const { a = 1 } = (a => {})();",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        (
            "
            () => {
              const y = (y => y)();
            };
                  ",
            Some(serde_json::json!([{ "ignoreOnInitialization": true }])),
            None,
            None,
        ),
        ("const [x = y => y] = [].map(y => y);", None, None, None),
        (
            "
            type Foo<A> = 1;
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ),
        (
            "
            interface Foo<A> {}
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ),
        (
            "
            interface Foo<A> {}
            interface A {}
                  ",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ),
        (
            "
            type Foo<A> = 1;
            interface A {}
                  ",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ),
        (
            "
            {
              type A = 1;
            }
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ),
        (
            "
            {
              interface Foo<A> {}
            }
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "never" }])),
            None,
            None,
        ),
        (
            "
            type Foo<A> = 1;
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "functions" }])),
            None,
            None,
        ),
        (
            "
            interface Foo<A> {}
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "functions" }])),
            None,
            None,
        ),
        (
            "
            interface Foo<A> {}
            interface A {}
                  ",
            Some(serde_json::json!([{ "hoist": "functions" }])),
            None,
            None,
        ),
        (
            "
            type Foo<A> = 1;
            interface A {}
                  ",
            Some(serde_json::json!([{ "hoist": "functions" }])),
            None,
            None,
        ),
        (
            "
            {
              type A = 1;
            }
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "functions" }])),
            None,
            None,
        ),
        (
            "
            {
              interface Foo<A> {}
            }
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "functions" }])),
            None,
            None,
        ),
        (
            "
            import type { Foo } from 'bar';

            declare module 'bar' {
              export type Foo = string;
            }
                  ",
            None,
            None,
            None,
        ),
        (
            "
            import type { Foo } from 'bar';

            declare module 'bar' {
              interface Foo {
                x: string;
              }
            }
                  ",
            None,
            None,
            None,
        ),
        (
            "
            import { type Foo } from 'bar';

            declare module 'bar' {
              export type Foo = string;
            }
                  ",
            None,
            None,
            None,
        ),
        (
            "
            import { type Foo } from 'bar';

            declare module 'bar' {
              export interface Foo {
                x: string;
              }
            }
                  ",
            None,
            None,
            None,
        ),
        (
            "
            import { type Foo } from 'bar';

            declare module 'bar' {
              type Foo = string;
            }
                  ",
            None,
            None,
            None,
        ),
        (
            "
            import { type Foo } from 'bar';

            declare module 'bar' {
              interface Foo {
                x: string;
              }
            }
                  ",
            None,
            None,
            None,
        ),
        (
            "
            declare const foo1: boolean;
                  ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "foo1": false, }, },
        (
            "
            declare let foo2: boolean;
                  ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "foo2": false, }, },
        (
            "
            declare var foo3: boolean;
                  ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "foo3": false, }, },
        (
            "
            function foo4(name: string): void;
                  ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "foo4": false, }, },
        (
            "
            declare class Foopy1 {
              name: string;
            }
                  ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "Foopy1": false, }, },
        (
            "
            declare interface Foopy2 {
              name: string;
            }
                  ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "Foopy2": false, }, },
        (
            "
            declare type Foopy3 = {
              x: number;
            };
                  ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "Foopy3": false, }, },
        (
            "
            declare enum Foopy4 {
              x,
            }
                  ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "Foopy4": false, }, },
        (
            "
            declare namespace Foopy5 {}
                  ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "Foopy5": false, }, },
        (
            "
            declare;
            foo5: boolean;
                  ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("baz.d.ts")),
        ), // { "globals": { "foo5": false, }, }
    ];

    let fail = vec![
        (
            "
            type T = 1;
            {
              type T = 2;
            }
                  ",
            None,
            None,
            None,
        ),
        (
            "
            type T = 1;
            function foo<T>(arg: T) {}
                  ",
            None,
            None,
            None,
        ),
        (
            "
            function foo<T>() {
              return function <T>() {};
            }
                  ",
            None,
            None,
            None,
        ),
        (
            "
            type T = string;
            function foo<T extends (arg: any) => void>(arg: T) {}
                  ",
            None,
            None,
            None,
        ),
        (
            "
            const x = 1;
            {
              type x = string;
            }
                  ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": false }])),
            None,
            None,
        ),
        (
            "
            type Foo = 1;
                  ",
            Some(
                serde_json::json!([ { "builtinGlobals": true, "ignoreTypeValueShadow": false, }, ]),
            ),
            Some(serde_json::json!({ "globals": { "Foo": "writable" } })),
            None,
        ), // { "globals": { "Foo": "writable", }, },
        (
            "
            const test = 1;
            type Fn = (test: string) => typeof test;
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
            type Fn = (Foo: string) => typeof Foo;
                  ",
            Some(
                serde_json::json!([ { "builtinGlobals": true, "ignoreFunctionTypeParameterNameValueShadow": false, }, ]),
            ),
            Some(serde_json::json!({ "globals": { "Foo": "writable" } })),
            None,
        ), // { "globals": { "Foo": "writable", }, },
        (
            "
            const arg = 0;

            interface Test {
              (arg: string): typeof arg;
            }
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
            const arg = 0;

            interface Test {
              p1(arg: string): typeof arg;
            }
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
            const arg = 0;

            declare function test(arg: string): typeof arg;
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
            const arg = 0;

            declare const test: (arg: string) => typeof arg;
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
            const arg = 0;

            declare class Test {
              p1(arg: string): typeof arg;
            }
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
            const arg = 0;

            declare const Test: {
              new (arg: string): typeof arg;
            };
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
            const arg = 0;

            type Bar = new (arg: number) => typeof arg;
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
            const arg = 0;

            declare namespace Lib {
              function test(arg: string): typeof arg;
            }
                  ",
            Some(serde_json::json!([{ "ignoreFunctionTypeParameterNameValueShadow": false }])),
            None,
            None,
        ),
        (
            "
            import type { foo } from './foo';
            function doThing(foo: number) {}
                  ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": false }])),
            None,
            None,
        ),
        (
            "
            import { type foo } from './foo';
            function doThing(foo: number) {}
                  ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": false }])),
            None,
            None,
        ),
        (
            "
            import { foo } from './foo';
            function doThing(foo: number, bar: number) {}
                  ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": true }])),
            None,
            None,
        ),
        (
            "
            interface Foo {}

            declare module 'bar' {
              export interface Foo {
                x: string;
              }
            }
                  ",
            None,
            None,
            None,
        ),
        (
            "
            import type { Foo } from 'bar';

            declare module 'baz' {
              export interface Foo {
                x: string;
              }
            }
                  ",
            None,
            None,
            None,
        ),
        (
            "
            import { type Foo } from 'bar';

            declare module 'baz' {
              export interface Foo {
                x: string;
              }
            }
                  ",
            None,
            None,
            None,
        ),
        (
            "
            import type { Foo } from 'bar';

            declare module 'bar' {
              export class Foo {}
            }
                  ",
            Some(serde_json::json!([{ "ignoreTypeValueShadow": false }])),
            None,
            None,
        ),
        (
            "
            import type { Foo } from 'bar';

            module bar {
              export interface Foo {
                x: string;
              }
            }
                  ",
            None,
            None,
            None,
        ),
        (
            "
            let x = foo((x, y) => {});
            let y;
                  ",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ), // { "parserOptions": { "ecmaVersion": 6 } },
        (
            "
            let x = foo((x, y) => {});
            let y;
                  ",
            Some(serde_json::json!([{ "hoist": "functions" }])),
            None,
            None,
        ), // { "parserOptions": { "ecmaVersion": 6 } },
        (
            "
            type Foo<A> = 1;
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "types" }])),
            None,
            None,
        ),
        (
            "
            interface Foo<A> {}
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "types" }])),
            None,
            None,
        ),
        (
            "
            interface Foo<A> {}
            interface A {}
                  ",
            Some(serde_json::json!([{ "hoist": "types" }])),
            None,
            None,
        ),
        (
            "
            type Foo<A> = 1;
            interface A {}
                  ",
            Some(serde_json::json!([{ "hoist": "types" }])),
            None,
            None,
        ),
        (
            "
            {
              type A = 1;
            }
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "types" }])),
            None,
            None,
        ),
        (
            "
            {
              interface A {}
            }
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "types" }])),
            None,
            None,
        ),
        (
            "
            type Foo<A> = 1;
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ),
        (
            "
            interface Foo<A> {}
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ),
        (
            "
            interface Foo<A> {}
            interface A {}
                  ",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ),
        (
            "
            type Foo<A> = 1;
            interface A {}
                  ",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ),
        (
            "
            {
              type A = 1;
            }
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ),
        (
            "
            {
              interface A {}
            }
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "all" }])),
            None,
            None,
        ),
        (
            "
            type Foo<A> = 1;
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "functions-and-types" }])),
            None,
            None,
        ),
        (
            "
            interface Foo<A> {}
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "functions-and-types" }])),
            None,
            None,
        ),
        (
            "
            interface Foo<A> {}
            interface A {}
                  ",
            Some(serde_json::json!([{ "hoist": "functions-and-types" }])),
            None,
            None,
        ),
        (
            "
            type Foo<A> = 1;
            interface A {}
                  ",
            Some(serde_json::json!([{ "hoist": "functions-and-types" }])),
            None,
            None,
        ),
        (
            "
            {
              type A = 1;
            }
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "functions-and-types" }])),
            None,
            None,
        ),
        (
            "
            {
              interface A {}
            }
            type A = 1;
                  ",
            Some(serde_json::json!([{ "hoist": "functions-and-types" }])),
            None,
            None,
        ),
        (
            "
            function foo<T extends (...args: any[]) => any>(fn: T, args: any[]) {}
                  ",
            Some(
                serde_json::json!([ { "builtinGlobals": true, "ignoreTypeValueShadow": false, }, ]),
            ),
            Some(serde_json::json!({ "globals": { "args": "writable" } })),
            None,
        ), // { "globals": { "args": "writable", }, },
        (
            "
            declare const has = (environment: 'dev' | 'prod' | 'test') => boolean;
                  ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            Some(serde_json::json!({ "globals": { "has": false } })),
            None,
        ), // { "globals": { "has": false, }, },
        (
            "
            declare const has: (environment: 'dev' | 'prod' | 'test') => boolean;
            const fn = (has: string) => {};
                  ",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
            None,
            Some(PathBuf::from("foo.d.ts")),
        ), // { "globals": { "has": false, }, }
    ];

    Tester::new(NoShadow::NAME, NoShadow::PLUGIN, pass, fail)
        .with_snapshot_suffix("typescript-eslint")
        .test_and_snapshot();
}
