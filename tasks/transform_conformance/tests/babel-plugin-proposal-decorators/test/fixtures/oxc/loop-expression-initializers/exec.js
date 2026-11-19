const capture = value => () => () => value;

// Each condition evaluation owns the initializer state retained by its class.
{
  const classes = [];
  let i = 0;
  const dec = () => {
    const value = i;
    return () => value;
  };
  while (i++ < 2 && classes.push(class { @dec x; })) {}
  expect(classes.map(C => new C().x)).toEqual([1, 2]);
}

// Labelled continue must still evaluate the next do-while condition.
{
  const classes = [];
  let i = 0;
  outer: inner: do {
    ++i;
    continue outer;
  } while (i < 3 && classes.push(class { @capture(i) x; }));
  expect(classes.map(C => new C().x)).toEqual([1, 2]);
  expect(i).toBe(3);
}

// Preserve the separate lexical environments of the body and the next update/test.
{
  const tests = [], updates = [], bodies = [];
  const testClasses = [], updateClasses = [];
  outer: for (let i = 0;
    i < 2 && tests.push(() => i) && testClasses.push(class { @capture(i) x; read() { return i; } });
    updates.push(() => i), updateClasses.push(class { @capture(i) x; read() { return i; } }), ++i
  ) {
    bodies.push(() => i);
    continue outer;
  }
  expect(tests.map(read => read())).toEqual([0, 1]);
  expect(bodies.map(read => read())).toEqual([0, 1]);
  expect(updates.map(read => read())).toEqual([1, 2]);
  expect(testClasses.map(C => [new C().x, new C().read()])).toEqual([[0, 0], [1, 1]]);
  expect(updateClasses.map(C => [new C().x, new C().read()])).toEqual([[0, 1], [1, 2]]);
}

// Moving a test alone leaves the original update and initializer semantics intact.
{
  const classes = [];
  for (let i = 0; i < 2 && classes.push(class { @capture(i) x; }); ++i) {
    const capture = null;
    expect(capture).toBe(null);
  }
  expect(classes.map(C => new C().x)).toEqual([0, 1]);
}

// Keep const initializers' named evaluation and support expression/absent initializers.
{
  const classes = [];
  let i = 0;
  for (const Named = class { static nameAtEvaluation = this.name; };
    i < 2;
    classes.push(class { @capture(i++) x; })
  ) {
    expect(Named.nameAtEvaluation).toBe("Named");
  }
  for (i = 2; i < 4; classes.push(class { @capture(i++) x; })) {}
  for (; i < 6; classes.push(class { @capture(i++) x; })) {}
  for (var j = 0; j < 2; classes.push(class { @capture(j++) x; })) {}
  expect(classes.map(C => new C().x)).toEqual([0, 1, 2, 3, 4, 5, 0, 1]);
}

// The header retains lexical this, arguments and super, including named-class TDZ.
{
  class Base {
    get decorator() { return capture(this.value); }
  }
  class Derived extends Base {
    value = 7;
    run(limit) {
      const classes = [];
      let i = 0;
      while (i++ < limit && classes.push(class extends (expect(arguments[0]).toBe(limit), Base) {
        @super.decorator x = this;
        @capture(this.value + i) y;
      })) {}
      return classes.map(C => [new C().x, new C().y]);
    }
  }
  expect(new Derived().run(2)).toEqual([[7, 8], [7, 9]]);
  expect(() => {
    while (class Named extends (Named, Base) { @capture(0) x; }) {}
  }).toThrow(ReferenceError);
}

// Suspension in eager class expressions stays in the surrounding generator.
{
  function* run() {
    const classes = [];
    let i = 0;
    while (i++ < 2 && classes.push(class extends (yield i) {
      @capture(i) [yield `key:${i}`];
    })) {}
    return classes.map(C => new C().x);
  }
  const iterator = run();
  expect(iterator.next()).toEqual({ value: 1, done: false });
  expect(iterator.next(class {})).toEqual({ value: "key:1", done: false });
  expect(iterator.next("x")).toEqual({ value: 2, done: false });
  expect(iterator.next(class {})).toEqual({ value: "key:2", done: false });
  expect(iterator.next("x")).toEqual({ value: [1, 2], done: true });
}

// Await in a do-while test and for update keeps its order and initializer lifetime.
return (async () => {
  const classes = [], log = [];
  let i = 0;
  do {
    ++i;
  } while (i < 3 && classes.push(class extends (log.push(`base:${i}`), await Promise.resolve(class {})) {
    @capture(i) [await Promise.resolve("x")];
  }));
  outer: for (let j = 3; j < 5;
    classes.push(class extends (await Promise.resolve(class {})) {
      @capture(j) [await Promise.resolve("x")];
    }), log.push(`update:${j}`), ++j
  ) {
    log.push(`body:${j}`);
    continue outer;
  }
  expect(classes.map(C => new C().x)).toEqual([1, 2, 3, 4]);
  expect(log).toEqual(["base:1", "base:2", "body:3", "update:3", "body:4", "update:4"]);
})();

// A decorated class created by a function in an unbraced loop body keeps per-call state.
{
  const makers = [];
  for (let i = 0; i < 2; i++) makers.push(function () { return class { @capture(i) x; }; });
  const [A, B] = makers.map((make) => make());
  expect([new A().x, new B().x, new (makers[0]())().x]).toEqual([0, 1, 0]);
}
