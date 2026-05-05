use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::valid_expect_in_promise::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct ValidExpectInPromise;

declare_oxc_lint!(
    ValidExpectInPromise,
    jest,
    correctness,
    docs = DOCUMENTATION,
    version = "1.60.0",
);

impl Rule for ValidExpectInPromise {
    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(possible_jest_node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("test('something', () => Promise.resolve().then(() => expect(1).toBe(2)));", None, None),
        ("Promise.resolve().then(() => expect(1).toBe(2))", None, None),
        ("const x = Promise.resolve().then(() => expect(1).toBe(2))", None, None),
        (r#"it.todo("something")"#, None, None),
        (
            "it('is valid', () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              expect(promise).resolves.toBe(1);
            });",
            None,
            None,
        ),
        (
            "it('is valid', () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              expect(promise).resolves.not.toBe(2);
            });",
            None,
            None,
        ),
        (
            "it('is valid', () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              expect(promise).rejects.toBe(1);
            });",
            None,
            None,
        ),
        (
            "it('is valid', () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              expect(promise).rejects.not.toBe(2);
            });",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              expect(await promise).toBeGreaterThan(1);
            });",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              expect(await promise).resolves.toBeGreaterThan(1);
            });",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              expect(1).toBeGreaterThan(await promise);
            });",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              expect.this.that.is(await promise);
            });",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
              expect(await loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              })).toBeGreaterThan(1);
            });",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              expect([await promise]).toHaveLength(1);
            });",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              expect([,,await promise,,]).toHaveLength(1);
            });",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              expect([[await promise]]).toHaveLength(1);
            });",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              logValue(await promise);
            });",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return 1;
              });
              expect.assertions(await promise);
            });",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
              await loadNumber().then(number => {
                expect(typeof number).toBe('number');
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', () => new Promise((done) => {
              test()
                .then(() => {
                  expect(someThing).toEqual(true);
                  done();
                });
            }));",
            None,
            None,
        ),
        (
            "it('it1', () => {
              return new Promise(done => {
                test().then(() => {
                  expect(someThing).toEqual(true);
                  done();
                });
              });
            });",
            None,
            None,
        ),
        (
            "it('passes', () => {
              Promise.resolve().then(() => {
                grabber.grabSomething();
              });
            });",
            None,
            None,
        ),
        (
            "it('passes', async () => {
              const grabbing = Promise.resolve().then(() => {
                grabber.grabSomething();
              });
              await grabbing;
              expect(grabber.grabbedItems).toHaveLength(1);
            });",
            None,
            None,
        ),
        (
            "const myFn = () => {
              Promise.resolve().then(() => {
                expect(true).toBe(false);
              });
            };",
            None,
            None,
        ),
        (
            "const myFn = () => {
              Promise.resolve().then(() => {
                subject.invokeMethod();
              });
            };",
            None,
            None,
        ),
        (
            "const myFn = () => {
              Promise.resolve().then(() => {
                expect(true).toBe(false);
              });
            };
            it('it1', () => {
              return somePromise.then(() => {
                expect(someThing).toEqual(true);
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', () => new Promise((done) => {
              test()
                .finally(() => {
                  expect(someThing).toEqual(true);
                  done();
                });
            }));",
            None,
            None,
        ),
        (
            "it('it1', () => {
              return somePromise.then(() => {
                expect(someThing).toEqual(true);
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', () => {
              return somePromise.finally(() => {
                expect(someThing).toEqual(true);
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', function() {
              return somePromise.catch(function() {
                expect(someThing).toEqual(true);
              });
            });",
            None,
            None,
        ),
        (
            "xtest('it1', function() {
              return somePromise.catch(function() {
                expect(someThing).toEqual(true);
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', function() {
              return somePromise.then(function() {
                doSomeThingButNotExpect();
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', function() {
              return getSomeThing().getPromise().then(function() {
                expect(someThing).toEqual(true);
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', function() {
              return Promise.resolve().then(function() {
                expect(someThing).toEqual(true);
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', function () {
              return Promise.resolve().then(function () {
                /*fulfillment*/
                expect(someThing).toEqual(true);
              }, function () {
                /*rejection*/
                expect(someThing).toEqual(true);
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', function () {
              Promise.resolve().then(/*fulfillment*/ function () {
              }, undefined, /*rejection*/ function () {
                expect(someThing).toEqual(true)
              })
            });",
            None,
            None,
        ),
        (
            "it('it1', function () {
              return Promise.resolve().then(function () {
                /*fulfillment*/
              }, function () {
                /*rejection*/
                expect(someThing).toEqual(true);
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', function () {
              return somePromise.then()
            });",
            None,
            None,
        ),
        (
            "it('it1', async () => {
              await Promise.resolve().then(function () {
                expect(someThing).toEqual(true)
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', async () => {
              await somePromise.then(() => {
                expect(someThing).toEqual(true)
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', async () => {
              await getSomeThing().getPromise().then(function () {
                expect(someThing).toEqual(true)
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', () => {
              return somePromise.then(() => {
                expect(someThing).toEqual(true);
              })
              .then(() => {
                expect(someThing).toEqual(true);
              })
            });",
            None,
            None,
        ),
        (
            "it('it1', () => {
              return somePromise.then(() => {
                return value;
              })
              .then(value => {
                expect(someThing).toEqual(value);
              })
            });",
            None,
            None,
        ),
        (
            "it('it1', () => {
              return somePromise.then(() => {
                expect(someThing).toEqual(true);
              })
              .then(() => {
                console.log('this is silly');
              })
            });",
            None,
            None,
        ),
        (
            "it('it1', () => {
              return somePromise.then(() => {
                expect(someThing).toEqual(true);
              })
              .catch(() => {
                expect(someThing).toEqual(false);
              })
            });",
            None,
            None,
        ),
        (
            "test('later return', () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              return promise;
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              await promise;
            });",
            None,
            None,
        ),
        (
            "test.only('later return', () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              return promise;
            });",
            None,
            None,
        ),
        (
            "test('that we bailout if destructuring is used', () => {
              const [promise] = something().then(value => {
                expect(value).toBe('red');
              });
            });",
            None,
            None,
        ),
        (
            "test('that we bailout if destructuring is used', async () => {
              const [promise] = await something().then(value => {
                expect(value).toBe('red');
              });
            });",
            None,
            None,
        ),
        (
            "test('that we bailout if destructuring is used', () => {
              const [promise] = [
                something().then(value => {
                  expect(value).toBe('red');
                })
              ];
            });",
            None,
            None,
        ),
        (
            "test('that we bailout if destructuring is used', () => {
              const {promise} = {
                promise: something().then(value => {
                  expect(value).toBe('red');
                })
              };
            });",
            None,
            None,
        ),
        (
            "test('that we bailout in complex cases', () => {
              promiseSomething({
                timeout: 500,
                promise: something().then(value => {
                  expect(value).toBe('red');
                })
              });
            });",
            None,
            None,
        ),
        (
            "it('shorthand arrow', () =>
              something().then(value => {
                expect(() => {
                  value();
                }).toThrow();
              })
            );",
            None,
            None,
        ),
        (
            "it('crawls for files based on patterns', () => {
              const promise = nodeCrawl({}).then(data => {
                expect(childProcess.spawn).lastCalledWith('find');
              });
              return promise;
            });",
            None,
            None,
        ),
        (
            "it('is a test', async () => {
              const value = await somePromise().then(response => {
                expect(response).toHaveProperty('data');
                return response.data;
              });
              expect(value).toBe('hello world');
            });",
            None,
            None,
        ),
        (
            "it('is a test', async () => {
              return await somePromise().then(response => {
                expect(response).toHaveProperty('data');
                return response.data;
              });
            });",
            None,
            None,
        ),
        (
            "it('is a test', async () => {
              return somePromise().then(response => {
                expect(response).toHaveProperty('data');
                return response.data;
              });
            });",
            None,
            None,
        ),
        (
            "it('is a test', async () => {
              await somePromise().then(response => {
                expect(response).toHaveProperty('data');
                return response.data;
              });
            });",
            None,
            None,
        ),
        (
            "it(
              'test function',
              () => {
                return Builder
                  .getPromiseBuilder()
                  .get().build()
                  .then((data) => {
                    expect(data).toEqual('Hi');
                  });
              }
            );",
            None,
            None,
        ),
        (
            "notATestFunction(
              'not a test function',
              () => {
                Builder
                  .getPromiseBuilder()
                  .get()
                  .build()
                  .then((data) => {
                    expect(data).toEqual('Hi');
                  });
              }
            );",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
              const promiseOne = loadNumber().then(number => {
                expect(typeof number).toBe('number');
              });
              const promiseTwo = loadNumber().then(number => {
                expect(typeof number).toBe('number');
              });
              await promiseTwo;
              await promiseOne;
            });",
            None,
            None,
        ),
        (
            r#"it("it1", () => somePromise.then(() => {
              expect(someThing).toEqual(true)
            }))"#,
            None,
            None,
        ),
        (r#"it("it1", () => somePromise.then(() => expect(someThing).toEqual(true)))"#, None, None),
        (
            "it('promise test with done', (done) => {
              const promise = getPromise();
              promise.then(() => expect(someThing).toEqual(true));
            });",
            None,
            None,
        ),
        (
            "it('name of done param does not matter', (nameDoesNotMatter) => {
              const promise = getPromise();
              promise.then(() => expect(someThing).toEqual(true));
            });",
            None,
            None,
        ),
        (
            "it.each([])('name of done param does not matter', (nameDoesNotMatter) => {
              const promise = getPromise();
              promise.then(() => expect(someThing).toEqual(true));
            });",
            None,
            None,
        ),
        (
            "it.each`\n`('name of done param does not matter', ({}, nameDoesNotMatter) => {
              const promise = getPromise();
              promise.then(() => expect(someThing).toEqual(true));
            });",
            None,
            None,
        ),
        (
            "test('valid-expect-in-promise', async () => {
              const text = await fetch('url')
                  .then(res => res.text())
                  .then(text => text);
              expect(text).toBe('text');
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              let somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              }), x = 1;
              await somePromise;
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              let x = 1, somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              await somePromise;
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              let somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              await somePromise;
              somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              await somePromise;
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              let somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              await somePromise;
              somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              return somePromise;
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              let somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              {}
              await somePromise;
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              const somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              {
                await somePromise;
              }
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              let somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              {
                await somePromise;
                somePromise = getPromise().then((data) => {
                  expect(data).toEqual('foo');
                });
                await somePromise;
              }
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              let somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              await somePromise;
              {
                somePromise = getPromise().then((data) => {
                  expect(data).toEqual('foo');
                });
                await somePromise;
              }
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              let somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              somePromise = somePromise.then((data) => {
                expect(data).toEqual('foo');
              });
              await somePromise;
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              let somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              somePromise = somePromise
                .then((data) => data)
                .then((data) => data)
                .then((data) => {
                  expect(data).toEqual('foo');
                });
              await somePromise;
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              let somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              somePromise = somePromise
                .then((data) => data)
                .then((data) => data)
              await somePromise;
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              let somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              await somePromise;
              {
                somePromise = getPromise().then((data) => {
                  expect(data).toEqual('foo');
                });
                {
                  await somePromise;
                }
              }
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              const somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              await Promise.all([somePromise]);
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              const somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              return Promise.all([somePromise]);
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              const somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              return Promise.resolve(somePromise);
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              const somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              return Promise.reject(somePromise);
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              const somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              await Promise.resolve(somePromise);
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              const somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              await Promise.reject(somePromise);
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const onePromise = something().then(value => {
                console.log(value);
              });
              const twoPromise = something().then(value => {
                expect(value).toBe('red');
              });
              return Promise.all([onePromise, twoPromise]);
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const onePromise = something().then(value => {
                console.log(value);
              });
              const twoPromise = something().then(value => {
                expect(value).toBe('red');
              });
              return Promise.allSettled([onePromise, twoPromise]);
            });",
            None,
            None,
        ),
    ];

    let fail = vec![
        (
            "const myFn = () => {
              Promise.resolve().then(() => {
                expect(true).toBe(false);
              });
            };
            it('it1', () => {
              somePromise.then(() => {
                expect(someThing).toEqual(true);
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', () => {
              somePromise.then(() => {
                expect(someThing).toEqual(true);
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', () => {
              somePromise.finally(() => {
                expect(someThing).toEqual(true);
              });
            });",
            None,
            None,
        ),
        (
            "
                   it('it1', () => {
                     somePromise['then'](() => {
                       expect(someThing).toEqual(true);
                     });
                   });
                  ",
            None,
            None,
        ),
        (
            "it('it1', function() {
              getSomeThing().getPromise().then(function() {
                expect(someThing).toEqual(true);
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', function() {
              Promise.resolve().then(function() {
                expect(someThing).toEqual(true);
              });
            });",
            None,
            None,
        ),
        (
            "it('it1', function() {
              somePromise.catch(function() {
                expect(someThing).toEqual(true)
              })
            })",
            None,
            None,
        ),
        (
            "xtest('it1', function() {
              somePromise.catch(function() {
                expect(someThing).toEqual(true)
              })
            })",
            None,
            None,
        ),
        (
            "it('it1', function() {
              somePromise.then(function() {
                expect(someThing).toEqual(true)
              })
            })",
            None,
            None,
        ),
        (
            "it('it1', function () {
              Promise.resolve().then(/*fulfillment*/ function () {
                expect(someThing).toEqual(true);
              }, /*rejection*/ function () {
                expect(someThing).toEqual(true);
              })
            })",
            None,
            None,
        ),
        (
            "it('it1', function () {
              Promise.resolve().then(/*fulfillment*/ function () {
              }, /*rejection*/ function () {
                expect(someThing).toEqual(true)
              })
            });",
            None,
            None,
        ),
        (
            "it('test function', () => {
              Builder.getPromiseBuilder()
                .get()
                .build()
                .then(data => expect(data).toEqual('Hi'));
            });",
            None,
            None,
        ),
        (
            "
                    it('test function', async () => {
                      Builder.getPromiseBuilder()
                        .get()
                        .build()
                        .then(data => expect(data).toEqual('Hi'));
                    });
                  ",
            None,
            None,
        ),
        (
            "it('it1', () => {
              somePromise.then(() => {
                doSomeOperation();
                expect(someThing).toEqual(true);
              })
            });",
            None,
            None,
        ),
        (
            "it('is a test', () => {
              somePromise
                .then(() => {})
                .then(() => expect(someThing).toEqual(value))
            });",
            None,
            None,
        ),
        (
            "it('is a test', () => {
              somePromise
                .then(() => expect(someThing).toEqual(value))
                .then(() => {})
            });",
            None,
            None,
        ),
        (
            "it('is a test', () => {
              somePromise.then(() => {
                return value;
              })
              .then(value => {
                expect(someThing).toEqual(value);
              })
            });",
            None,
            None,
        ),
        (
            "it('is a test', () => {
              somePromise.then(() => {
                expect(someThing).toEqual(true);
              })
              .then(() => {
                console.log('this is silly');
              })
            });",
            None,
            None,
        ),
        (
            "it('is a test', () => {
              somePromise.then(() => {
                // return value;
              })
              .then(value => {
                expect(someThing).toEqual(value);
              })
            });",
            None,
            None,
        ),
        (
            "it('is a test', () => {
              somePromise.then(() => {
                return value;
              })
              .then(value => {
                expect(someThing).toEqual(value);
              })
              return anotherPromise.then(() => expect(x).toBe(y));
            });",
            None,
            None,
        ),
        (
            "it('is a test', () => {
              somePromise
                .then(() => 1)
                .then(x => x + 1)
                .catch(() => -1)
                .then(v => expect(v).toBe(2));
              return anotherPromise.then(() => expect(x).toBe(y));
            });",
            None,
            None,
        ),
        (
            "it('is a test', () => {
              somePromise
                .then(() => 1)
                .then(v => expect(v).toBe(2))
                .then(x => x + 1)
                .catch(() => -1);
              return anotherPromise.then(() => expect(x).toBe(y));
            });",
            None,
            None,
        ),
        (
            "it('it1', () => {
              somePromise.finally(() => {
                doSomeOperation();
                expect(someThing).toEqual(true);
              })
            });",
            None,
            None,
        ),
        (
            r#"test('invalid return', () => {
              const promise = something().then(value => {
                const foo = "foo";
                return expect(value).toBe('red');
              });
            });"#,
            None,
            None,
        ),
        (
            "fit('it1', () => {
              somePromise.then(() => {
                doSomeOperation();
                expect(someThing).toEqual(true);
              })
            });",
            None,
            None,
        ),
        (
            "it.skip('it1', () => {
              somePromise.then(() => {
                doSomeOperation();
                expect(someThing).toEqual(true);
              })
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              promise;
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              return;
              await promise;
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              return 1;
              await promise;
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              return [];
              await promise;
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              return Promise.all([anotherPromise]);
              await promise;
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              return {};
              await promise;
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              return Promise.all([]);
              await promise;
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              await 1;
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              await [];
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              await Promise.all([anotherPromise]);
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              await {};
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              await Promise.all([]);
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              }), x = 1;
            });",
            None,
            None,
        ),
        (
            "test('later return', async () => {
              const x = 1, promise = something().then(value => {
                expect(value).toBe('red');
              });
            });",
            None,
            None,
        ),
        (
            "import { test } from '@jest/globals';
            test('later return', async () => {
              const x = 1, promise = something().then(value => {
                expect(value).toBe('red');
              });
            });",
            None,
            None,
        ),
        (
            "it('promise test', () => {
              const somePromise = getThatPromise();
              somePromise.then((data) => {
                expect(data).toEqual('foo');
              });
              expect(somePromise).toBeDefined();
              return somePromise;
            });",
            None,
            None,
        ),
        (
            "test('promise test', function () {
              let somePromise = getThatPromise();
              somePromise.then((data) => {
                expect(data).toEqual('foo');
              });
              expect(somePromise).toBeDefined();
              return somePromise;
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              let somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              somePromise = null;
              await somePromise;
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              let somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              await somePromise;
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              let somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              ({ somePromise } = {})
            });",
            None,
            None,
        ),
        (
            "test('promise test', async function () {
              let somePromise = getPromise().then((data) => {
                expect(data).toEqual('foo');
              });
              {
                somePromise = getPromise().then((data) => {
                  expect(data).toEqual('foo');
                });
                await somePromise;
              }
            });",
            None,
            None,
        ),
        (
            "test('that we error on this destructuring', async () => {
              [promise] = something().then(value => {
                expect(value).toBe('red');
              });
            });",
            None,
            None,
        ),
        (
            "test('that we error on this', () => {
              const promise = something().then(value => {
                expect(value).toBe('red');
              });
              log(promise);
            });",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              expect(promise).toBeInstanceOf(Promise);
            });",
            None,
            None,
        ),
        (
            "it('is valid', async () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              expect(anotherPromise).resolves.toBe(1);
            });",
            None,
            None,
        ),
        (
            "import { it as promiseThatThis } from '@jest/globals';
            promiseThatThis('is valid', async () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              expect(anotherPromise).resolves.toBe(1);
            });",
            None,
            None,
        ),
        /*
         * jest alias not supported
        (
            "promiseThatThis('is valid', async () => {
              const promise = loadNumber().then(number => {
                expect(typeof number).toBe('number');
                return number + 1;
              });
              expect(anotherPromise).resolves.toBe(1);
            });",
            None,
            Some(
                serde_json::json!({ "settings": { "jest": { "globalAliases": { "xit": ["promiseThatThis"] } } } }),
            ),
        ),
         */
    ];

    Tester::new(ValidExpectInPromise::NAME, ValidExpectInPromise::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
