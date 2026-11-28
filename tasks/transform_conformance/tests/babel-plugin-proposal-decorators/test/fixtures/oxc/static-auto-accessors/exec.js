const noop = () => {};
class Base {
  static accessor plain = 1;
  @noop static accessor decorated = 2;
}
class Derived extends Base {}
expect(Derived.plain).toBe(1);
expect(Derived.decorated).toBe(2);
Derived.plain = 3;
Derived.decorated = 4;
expect([Base.plain, Base.decorated]).toEqual([3, 4]);

let C = class {
  @noop static accessor value = 5;
};
const original = C;
C = null;
expect(original.value).toBe(5);
const descriptor = Object.getOwnPropertyDescriptor(original, "value");
expect(descriptor.get.call({})).toBe(5);
descriptor.set.call({}, 6);
expect(original.value).toBe(6);
