use oxc_ast::{
    ast::{CallExpression, ChainElement, Expression, ExpressionStatement},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::UnaryOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(no-floating-promises): Promises must be awaited, end with a call to .catch, or end with a call to .then with a rejection handler.")]
#[diagnostic(severity(warning), help("Add `await` or `return`, call `.then()` with two arguments or `.catch()` with one argument."))]
struct NoFloatingPromisesDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoFloatingPromises {
    ignore_iife: bool,
    ignore_void: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require Promise-like statements to be handled appropriately
    ///
    /// ### Why is this bad?
    /// A "floating" Promise is one that is created without any code set up to handle any errors it might throw. Floating Promises can cause several issues, such as improperly sequenced operations, ignored Promise rejections, and more.
    ///
    /// ### Example
    /// ```javascript
    /// const promise = new Promise((resolve, reject) => resolve('value'));
    /// promise;
    ///
    /// async function returnsPromise() {
    ///   return 'value';
    /// }
    /// returnsPromise().then(() => {});
    ///
    /// Promise.reject('value').catch();
    ///
    /// Promise.reject('value').finally();
    ///
    /// [1, 2, 3].map(async x => x + 1);
    /// ```
    NoFloatingPromises,
    nursery,
    true
);

impl Rule for NoFloatingPromises {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self {
            ignore_iife: value
                .get(0)
                .and_then(|x| x.get("ignoreIIFE"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            ignore_void: value
                .get(0)
                .and_then(|x| x.get("ignoreVoid"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExpressionStatement(stmt) = node.kind() else { return };
        if self.ignore_iife && is_async_iife(stmt) {
            return;
        }

        let result = match &stmt.expression {
            Expression::ChainExpression(chain) => {
                self.is_unhandled_promise_chain(&chain.expression, ctx)
            }
            expr => self.is_unhandled_promise(expr, ctx),
        };

        // Handle result
        todo!()
    }
}

impl NoFloatingPromises {
    fn is_unhandled_promise<'a>(&self, node: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
        if let Expression::SequenceExpression(expr) = node {
            // TODO: needs to return first unhandled
            return expr.expressions.iter().all(|e| !self.is_unhandled_promise(e, ctx));
        }

        if !self.ignore_void {
            if let Expression::UnaryExpression(expr) = node {
                if expr.operator == UnaryOperator::Void {
                    return self.is_unhandled_promise(&expr.argument, ctx);
                }
            }
        }

        // TODO
        // if isPromiseArray(node, ctx) {
        //     return true; // { promiseArray: true };
        // }

        // TODO
        // if !isPromiseLike(node, ctx) {
        //     return false;
        // }

        match node {
            Expression::CallExpression(expr) => is_unhandled_call_expression(expr),
            Expression::ConditionalExpression(_) => todo!(),
            Expression::MemberExpression(_)
            | Expression::Identifier(_)
            | Expression::NewExpression(_) => todo!(),
            Expression::LogicalExpression(_) => todo!(),

            _ => false,
        }
    }

    fn is_unhandled_promise_chain<'a>(
        &self,
        node: &ChainElement<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        // TODO
        // if isPromiseArray(node, ctx) {
        //     return true; // { promiseArray: true };
        // }

        // TODO
        // if !isPromiseLike(node, ctx) {
        //     return false;
        // }

        match node {
            ChainElement::CallExpression(expr) => is_unhandled_call_expression(expr),
            ChainElement::MemberExpression(_) => todo!(),
        }
    }
}

fn is_async_iife<'a>(node: &ExpressionStatement<'a>) -> bool {
    let Expression::CallExpression(ref expr) = node.expression else { return false };
    expr.callee.is_function()
}

fn is_unhandled_call_expression<'a>(node: &CallExpression<'a>) -> bool {
    todo!()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
            async function test() {
              await Promise.resolve('value');
              Promise.resolve('value').then(
                () => {},
                () => {},
              );
              Promise.resolve('value')
                .then(() => {})
                .catch(() => {});
              Promise.resolve('value')
                .then(() => {})
                .catch(() => {})
                .finally(() => {});
              Promise.resolve('value').catch(() => {});
              return Promise.resolve('value');
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              void Promise.resolve('value');
            }
                  ",
            Some(serde_json::json!([{ "ignoreVoid": true }])),
        ),
        (
            "
            async function test() {
              await Promise.reject(new Error('message'));
              Promise.reject(new Error('message')).then(
                () => {},
                () => {},
              );
              Promise.reject(new Error('message'))
                .then(() => {})
                .catch(() => {});
              Promise.reject(new Error('message'))
                .then(() => {})
                .catch(() => {})
                .finally(() => {});
              Promise.reject(new Error('message')).catch(() => {});
              return Promise.reject(new Error('message'));
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              await (async () => true)();
              (async () => true)().then(
                () => {},
                () => {},
              );
              (async () => true)()
                .then(() => {})
                .catch(() => {});
              (async () => true)()
                .then(() => {})
                .catch(() => {})
                .finally(() => {});
              (async () => true)().catch(() => {});
              return (async () => true)();
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              async function returnsPromise() {}
              await returnsPromise();
              returnsPromise().then(
                () => {},
                () => {},
              );
              returnsPromise()
                .then(() => {})
                .catch(() => {});
              returnsPromise()
                .then(() => {})
                .catch(() => {})
                .finally(() => {});
              returnsPromise().catch(() => {});
              return returnsPromise();
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              const x = Promise.resolve();
              const y = x.then(() => {});
              y.catch(() => {});
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              Math.random() > 0.5 ? Promise.resolve().catch(() => {}) : null;
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              Promise.resolve().catch(() => {}), 123;
              123,
                Promise.resolve().then(
                  () => {},
                  () => {},
                );
              123,
                Promise.resolve().then(
                  () => {},
                  () => {},
                ),
                123;
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              void Promise.resolve().catch(() => {});
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              Promise.resolve().catch(() => {}) ||
                Promise.resolve().then(
                  () => {},
                  () => {},
                );
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              declare const promiseValue: Promise<number>;

              await promiseValue;
              promiseValue.then(
                () => {},
                () => {},
              );
              promiseValue.then(() => {}).catch(() => {});
              promiseValue
                .then(() => {})
                .catch(() => {})
                .finally(() => {});
              promiseValue.catch(() => {});
              return promiseValue;
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              declare const promiseUnion: Promise<number> | number;

              await promiseUnion;
              promiseUnion.then(
                () => {},
                () => {},
              );
              promiseUnion.then(() => {}).catch(() => {});
              promiseUnion
                .then(() => {})
                .catch(() => {})
                .finally(() => {});
              promiseUnion.catch(() => {});
              promiseValue.finally(() => {});
              return promiseUnion;
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              declare const promiseIntersection: Promise<number> & number;

              await promiseIntersection;
              promiseIntersection.then(
                () => {},
                () => {},
              );
              promiseIntersection.then(() => {}).catch(() => {});
              promiseIntersection.catch(() => {});
              return promiseIntersection;
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              class CanThen extends Promise<number> {}
              const canThen: CanThen = Foo.resolve(2);

              await canThen;
              canThen.then(
                () => {},
                () => {},
              );
              canThen.then(() => {}).catch(() => {});
              canThen
                .then(() => {})
                .catch(() => {})
                .finally(() => {});
              canThen.catch(() => {});
              return canThen;
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              await (Math.random() > 0.5 ? numberPromise : 0);
              await (Math.random() > 0.5 ? foo : 0);
              await (Math.random() > 0.5 ? bar : 0);

              declare const intersectionPromise: Promise<number> & number;
              await intersectionPromise;
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              class Thenable {
                then(callback: () => void): Thenable {
                  return new Thenable();
                }
              }
              const thenable = new Thenable();

              await thenable;
              thenable;
              thenable.then(() => {});
              return thenable;
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              class NonFunctionParamThenable {
                then(param: string, param2: number): NonFunctionParamThenable {
                  return new NonFunctionParamThenable();
                }
              }
              const thenable = new NonFunctionParamThenable();

              await thenable;
              thenable;
              thenable.then('abc', 'def');
              return thenable;
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              class NonFunctionThenable {
                then: number;
              }
              const thenable = new NonFunctionThenable();

              thenable;
              thenable.then;
              return thenable;
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              class CatchableThenable {
                then(resolve: () => void, reject: () => void): CatchableThenable {
                  return new CatchableThenable();
                }
              }
              const thenable = new CatchableThenable();

              await thenable;
              return thenable;
            }
                ",
            None,
        ),
        (
            "
            // https://github.com/DefinitelyTyped/DefinitelyTyped/blob/master/types/promise-polyfill/index.d.ts
            // Type definitions for promise-polyfill 6.0
            // Project: https://github.com/taylorhakes/promise-polyfill
            // Definitions by: Steve Jenkins <https://github.com/skysteve>
            //                 Daniel Cassidy <https://github.com/djcsdy>
            // Definitions: https://github.com/DefinitelyTyped/DefinitelyTyped

            interface PromisePolyfillConstructor extends PromiseConstructor {
              _immediateFn?: (handler: (() => void) | string) => void;
            }

            declare const PromisePolyfill: PromisePolyfillConstructor;

            async function test() {
              const promise = new PromisePolyfill(() => {});

              await promise;
              promise.then(
                () => {},
                () => {},
              );
              promise.then(() => {}).catch(() => {});
              promise
                .then(() => {})
                .catch(() => {})
                .finally(() => {});
              promise.catch(() => {});
              return promise;
            }
                ",
            None,
        ),
        (
            "
            async function test() {
              declare const returnsPromise: () => Promise<void> | null;
              await returnsPromise?.();
              returnsPromise()?.then(
                () => {},
                () => {},
              );
              returnsPromise()
                ?.then(() => {})
                ?.catch(() => {});
              returnsPromise()?.catch(() => {});
              return returnsPromise();
            }
                ",
            None,
        ),
        (
            "
            const doSomething = async (
              obj1: { a?: { b?: { c?: () => Promise<void> } } },
              obj2: { a?: { b?: { c: () => Promise<void> } } },
              obj3: { a?: { b: { c?: () => Promise<void> } } },
              obj4: { a: { b: { c?: () => Promise<void> } } },
              obj5: { a?: () => { b?: { c?: () => Promise<void> } } },
              obj6?: { a: { b: { c?: () => Promise<void> } } },
              callback?: () => Promise<void>,
            ): Promise<void> => {
              await obj1.a?.b?.c?.();
              await obj2.a?.b?.c();
              await obj3.a?.b.c?.();
              await obj4.a.b.c?.();
              await obj5.a?.().b?.c?.();
              await obj6?.a.b.c?.();

              return callback?.();
            };

            void doSomething();
                ",
            None,
        ),
        (
            "
                    (async () => {
                      await something();
                    })();
                  ",
            Some(serde_json::json!([{ "ignoreIIFE": true }])),
        ),
        (
            "
                    (async () => {
                      something();
                    })();
                  ",
            Some(serde_json::json!([{ "ignoreIIFE": true }])),
        ),
        ("(async function foo() {})();", Some(serde_json::json!([{ "ignoreIIFE": true }]))),
        (
            "
                    function foo() {
                      (async function bar() {})();
                    }
                  ",
            Some(serde_json::json!([{ "ignoreIIFE": true }])),
        ),
        (
            "
                    const foo = () =>
                      new Promise(res => {
                        (async function () {
                          await res(1);
                        })();
                      });
                  ",
            Some(serde_json::json!([{ "ignoreIIFE": true }])),
        ),
        (
            "
                    (async function () {
                      await res(1);
                    })();
                  ",
            Some(serde_json::json!([{ "ignoreIIFE": true }])),
        ),
        (
            "
            async function foo() {
              const myPromise = async () => void 0;
              const condition = true;
              void (condition && myPromise());
            }
                  ",
            None,
        ),
        (
            "
            async function foo() {
              const myPromise = async () => void 0;
              const condition = true;
              await (condition && myPromise());
            }
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            async function foo() {
              const myPromise = async () => void 0;
              const condition = true;
              condition && void myPromise();
            }
                  ",
            None,
        ),
        (
            "
            async function foo() {
              const myPromise = async () => void 0;
              const condition = true;
              condition && (await myPromise());
            }
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            async function foo() {
              const myPromise = async () => void 0;
              let condition = false;
              condition && myPromise();
              condition = true;
              condition || myPromise();
              condition ?? myPromise();
            }
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            declare const definitelyCallable: () => void;
            Promise.reject().catch(definitelyCallable);
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            Promise.reject()
              .catch(() => {})
              .finally(() => {});
                  ",
            None,
        ),
        (
            "
            Promise.reject()
              .catch(() => {})
              .finally(() => {})
              .finally(() => {});
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            Promise.reject()
              .catch(() => {})
              .finally(() => {})
              .finally(() => {})
              .finally(() => {});
                  ",
            None,
        ),
        (
            "
            await Promise.all([Promise.resolve(), Promise.resolve()]);
                  ",
            None,
        ),
        (
            "
            declare const promiseArray: Array<Promise<unknown>>;
            void promiseArray;
                  ",
            None,
        ),
        (
            "
            [Promise.reject(), Promise.reject()].then(() => {});
                  ",
            None,
        ),
        (
            "
            [1, 2, void Promise.reject(), 3];
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            ['I', 'am', 'just', 'an', 'array'];
                  ",
            None,
        ),
        (
            "
            declare const myTag: (strings: TemplateStringsArray) => Promise<void>;
            myTag`abc`.catch(() => {});
                  ",
            None,
        ),
        (
            "
            declare const myTag: (strings: TemplateStringsArray) => string;
            myTag`abc`;
                  ",
            None,
        ),
    ];

    let fail = vec![
        (
            "
            async function test() {
              Promise.resolve('value');
              Promise.resolve('value').then(() => {});
              Promise.resolve('value').catch();
              Promise.resolve('value').finally();
            }
                  ",
            None,
        ),
        (
            "
            const doSomething = async (
              obj1: { a?: { b?: { c?: () => Promise<void> } } },
              obj2: { a?: { b?: { c: () => Promise<void> } } },
              obj3: { a?: { b: { c?: () => Promise<void> } } },
              obj4: { a: { b: { c?: () => Promise<void> } } },
              obj5: { a?: () => { b?: { c?: () => Promise<void> } } },
              obj6?: { a: { b: { c?: () => Promise<void> } } },
              callback?: () => Promise<void>,
            ): Promise<void> => {
              obj1.a?.b?.c?.();
              obj2.a?.b?.c();
              obj3.a?.b.c?.();
              obj4.a.b.c?.();
              obj5.a?.().b?.c?.();
              obj6?.a.b.c?.();

              callback?.();
            };

            doSomething();
                  ",
            None,
        ),
        (
            "
            declare const myTag: (strings: TemplateStringsArray) => Promise<void>;
            myTag`abc`;
                  ",
            None,
        ),
        (
            "
            declare const myTag: (strings: TemplateStringsArray) => Promise<void>;
            myTag`abc`.then(() => {});
                  ",
            None,
        ),
        (
            "
            declare const myTag: (strings: TemplateStringsArray) => Promise<void>;
            myTag`abc`.finally(() => {});
                  ",
            None,
        ),
        (
            "
            async function test() {
              Promise.resolve('value');
            }
                  ",
            Some(serde_json::json!([{ "ignoreVoid": true }])),
        ),
        (
            "
            async function test() {
              Promise.reject(new Error('message'));
              Promise.reject(new Error('message')).then(() => {});
              Promise.reject(new Error('message')).catch();
              Promise.reject(new Error('message')).finally();
            }
                  ",
            None,
        ),
        (
            "
            async function test() {
              (async () => true)();
              (async () => true)().then(() => {});
              (async () => true)().catch();
            }
                  ",
            None,
        ),
        (
            "
            async function test() {
              async function returnsPromise() {}

              returnsPromise();
              returnsPromise().then(() => {});
              returnsPromise().catch();
              returnsPromise().finally();
            }
                  ",
            None,
        ),
        (
            "
            async function test() {
              Math.random() > 0.5 ? Promise.resolve() : null;
              Math.random() > 0.5 ? null : Promise.resolve();
            }
                  ",
            None,
        ),
        (
            "
            async function test() {
              Promise.resolve(), 123;
              123, Promise.resolve();
              123, Promise.resolve(), 123;
            }
                  ",
            None,
        ),
        (
            "
            async function test() {
              void Promise.resolve();
            }
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            async function test() {
              const promise = new Promise((resolve, reject) => resolve('value'));
              promise;
            }
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            async function returnsPromise() {
              return 'value';
            }
            void returnsPromise();
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            async function returnsPromise() {
              return 'value';
            }
            void /* ... */ returnsPromise();
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            async function returnsPromise() {
              return 'value';
            }
            1, returnsPromise();
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            async function returnsPromise() {
              return 'value';
            }
            bool ? returnsPromise() : null;
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            async function test() {
              const obj = { foo: Promise.resolve() };
              obj.foo;
            }
                  ",
            None,
        ),
        (
            "
            async function test() {
              new Promise(resolve => resolve());
            }
                  ",
            None,
        ),
        (
            "
            async function test() {
              declare const promiseValue: Promise<number>;

              promiseValue;
              promiseValue.then(() => {});
              promiseValue.catch();
              promiseValue.finally();
            }
                  ",
            None,
        ),
        (
            "
            async function test() {
              declare const promiseUnion: Promise<number> | number;

              promiseUnion;
            }
                  ",
            None,
        ),
        (
            "
            async function test() {
              declare const promiseIntersection: Promise<number> & number;

              promiseIntersection;
              promiseIntersection.then(() => {});
              promiseIntersection.catch();
            }
                  ",
            None,
        ),
        (
            "
            async function test() {
              class CanThen extends Promise<number> {}
              const canThen: CanThen = Foo.resolve(2);

              canThen;
              canThen.then(() => {});
              canThen.catch();
              canThen.finally();
            }
                  ",
            None,
        ),
        (
            "
            async function test() {
              class CatchableThenable {
                then(callback: () => void, callback: () => void): CatchableThenable {
                  return new CatchableThenable();
                }
              }
              const thenable = new CatchableThenable();

              thenable;
              thenable.then(() => {});
            }
                  ",
            None,
        ),
        (
            "
            // https://github.com/DefinitelyTyped/DefinitelyTyped/blob/master/types/promise-polyfill/index.d.ts
            // Type definitions for promise-polyfill 6.0
            // Project: https://github.com/taylorhakes/promise-polyfill
            // Definitions by: Steve Jenkins <https://github.com/skysteve>
            //                 Daniel Cassidy <https://github.com/djcsdy>
            // Definitions: https://github.com/DefinitelyTyped/DefinitelyTyped

            interface PromisePolyfillConstructor extends PromiseConstructor {
              _immediateFn?: (handler: (() => void) | string) => void;
            }

            declare const PromisePolyfill: PromisePolyfillConstructor;

            async function test() {
              const promise = new PromisePolyfill(() => {});

              promise;
              promise.then(() => {});
              promise.catch();
            }
                  ",
            None,
        ),
        (
            "
                    (async () => {
                      await something();
                    })();
                  ",
            None,
        ),
        (
            "
                    (async () => {
                      something();
                    })();
                  ",
            None,
        ),
        ("(async function foo() {})();", None),
        (
            "
                    function foo() {
                      (async function bar() {})();
                    }
                  ",
            None,
        ),
        (
            "
                    const foo = () =>
                      new Promise(res => {
                        (async function () {
                          await res(1);
                        })();
                      });
                  ",
            None,
        ),
        (
            "
                    (async function () {
                      await res(1);
                    })();
                  ",
            None,
        ),
        (
            "
                    (async function () {
                      Promise.resolve();
                    })();
                  ",
            Some(serde_json::json!([{ "ignoreIIFE": true }])),
        ),
        (
            "
                    (async function () {
                      declare const promiseIntersection: Promise<number> & number;
                      promiseIntersection;
                      promiseIntersection.then(() => {});
                      promiseIntersection.catch();
                      promiseIntersection.finally();
                    })();
                  ",
            Some(serde_json::json!([{ "ignoreIIFE": true }])),
        ),
        (
            "
            async function foo() {
              const myPromise = async () => void 0;
              const condition = true;

              void condition || myPromise();
            }
                  ",
            None,
        ),
        (
            "
            async function foo() {
              const myPromise = async () => void 0;
              const condition = true;

              (await condition) && myPromise();
            }
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            async function foo() {
              const myPromise = async () => void 0;
              const condition = true;

              condition && myPromise();
            }
                  ",
            None,
        ),
        (
            "
            async function foo() {
              const myPromise = async () => void 0;
              const condition = false;

              condition || myPromise();
            }
                  ",
            None,
        ),
        (
            "
            async function foo() {
              const myPromise = async () => void 0;
              const condition = null;

              condition ?? myPromise();
            }
                  ",
            None,
        ),
        (
            "
            async function foo() {
              const myPromise = Promise.resolve(true);
              let condition = true;
              condition && myPromise;
            }
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            async function foo() {
              const myPromise = Promise.resolve(true);
              let condition = false;
              condition || myPromise;
            }
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            async function foo() {
              const myPromise = Promise.resolve(true);
              let condition = null;
              condition ?? myPromise;
            }
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            async function foo() {
              const myPromise = async () => void 0;
              const condition = false;

              condition || condition || myPromise();
            }
                  ",
            None,
        ),
        (
            "
            declare const maybeCallable: string | (() => void);
            declare const definitelyCallable: () => void;
            Promise.resolve().then(() => {}, undefined);
            Promise.resolve().then(() => {}, null);
            Promise.resolve().then(() => {}, 3);
            Promise.resolve().then(() => {}, maybeCallable);
            Promise.resolve().then(() => {}, definitelyCallable);

            Promise.resolve().catch(undefined);
            Promise.resolve().catch(null);
            Promise.resolve().catch(3);
            Promise.resolve().catch(maybeCallable);
            Promise.resolve().catch(definitelyCallable);
                  ",
            None,
        ),
        (
            "
            Promise.reject() || 3;
                  ",
            None,
        ),
        (
            "
            void Promise.resolve().then(() => {}, undefined);
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            declare const maybeCallable: string | (() => void);
            Promise.resolve().then(() => {}, maybeCallable);
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            declare const maybeCallable: string | (() => void);
            declare const definitelyCallable: () => void;
            Promise.resolve().then(() => {}, undefined);
            Promise.resolve().then(() => {}, null);
            Promise.resolve().then(() => {}, 3);
            Promise.resolve().then(() => {}, maybeCallable);
            Promise.resolve().then(() => {}, definitelyCallable);

            Promise.resolve().catch(undefined);
            Promise.resolve().catch(null);
            Promise.resolve().catch(3);
            Promise.resolve().catch(maybeCallable);
            Promise.resolve().catch(definitelyCallable);
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            Promise.reject() || 3;
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            Promise.reject().finally(() => {});
                  ",
            None,
        ),
        (
            "
            Promise.reject()
              .finally(() => {})
              .finally(() => {});
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            Promise.reject()
              .finally(() => {})
              .finally(() => {})
              .finally(() => {});
                  ",
            None,
        ),
        (
            "
            Promise.reject()
              .then(() => {})
              .finally(() => {});
                  ",
            None,
        ),
        (
            "
            declare const returnsPromise: () => Promise<void> | null;
            returnsPromise()?.finally(() => {});
                  ",
            None,
        ),
        (
            "
            const promiseIntersection: Promise<number> & number;
            promiseIntersection.finally(() => {});
                  ",
            None,
        ),
        (
            "
            Promise.resolve().finally(() => {}), 123;
                  ",
            None,
        ),
        (
            "
            (async () => true)().finally();
                  ",
            None,
        ),
        (
            "
            Promise.reject(new Error('message')).finally(() => {});
                  ",
            None,
        ),
        (
            "
            function _<T, S extends Array<T | Promise<T>>>(
              maybePromiseArray: S | undefined,
            ): void {
              maybePromiseArray?.[0];
            }
                  ",
            None,
        ),
        (
            "
            [1, 2, 3].map(() => Promise.reject());
                  ",
            None,
        ),
        (
            "
            declare const array: unknown[];
            array.map(() => Promise.reject());
                  ",
            None,
        ),
        (
            "
            declare const promiseArray: Array<Promise<unknown>>;
            void promiseArray;
                  ",
            Some(serde_json::json!([{ "ignoreVoid": false }])),
        ),
        (
            "
            [1, 2, Promise.reject(), 3];
                  ",
            None,
        ),
        (
            "
            [1, 2, Promise.reject().catch(() => {}), 3];
                  ",
            None,
        ),
        (
            "
            const data = ['test'];
            data.map(async () => {
              await new Promise((_res, rej) => setTimeout(rej, 1000));
            });
                  ",
            None,
        ),
        (
            "
            function _<T, S extends Array<T | Array<T | Promise<T>>>>(
              maybePromiseArrayArray: S | undefined,
            ): void {
              maybePromiseArrayArray?.[0];
            }
                  ",
            None,
        ),
        (
            "
            function f<T extends Array<Promise<number>>>(a: T): void {
              a;
            }
                  ",
            None,
        ),
        (
            "
            declare const a: Array<Promise<number>> | undefined;
            a;
                  ",
            None,
        ),
        (
            "
            function f<T extends Array<Promise<number>>>(a: T | undefined): void {
              a;
            }
                  ",
            None,
        ),
        (
            "
            [Promise.reject()] as const;
                  ",
            None,
        ),
        (
            "
            declare function cursed(): [Promise<number>, Promise<string>];
            cursed();
                  ",
            None,
        ),
        (
            "
            [
              'Type Argument number ',
              1,
              'is not',
              Promise.resolve(),
              'but it still is flagged',
            ] as const;
                  ",
            None,
        ),
        (
            "
                    declare const arrayOrPromiseTuple:
                      | Array<number>
                      | [number, number, Promise<unknown>, string];
                    arrayOrPromiseTuple;
                  ",
            None,
        ),
        (
            "
                    declare const okArrayOrPromiseArray: Array<number> | Array<Promise<unknown>>;
                    okArrayOrPromiseArray;
                  ",
            None,
        ),
    ];

    Tester::new(NoFloatingPromises::NAME, pass, fail).test_and_snapshot();
}
