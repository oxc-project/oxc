const fns = [];
const capture = (fn) => {
  fns.push(fn);
  expect(fn).toThrow(ReferenceError);
  return () => {};
};

@capture(() => Foo)
class Foo {
  @capture(() => Foo) method() {}
  @capture(() => Foo) static field;
  @capture(() => Foo) accessor acc;
}

const original = Foo;
for (const fn of fns) expect(fn()).toBe(original);
Foo = null;
expect(fns.shift()()).toBe(null);
for (const fn of fns) expect(fn()).toBe(original);
