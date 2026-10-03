const events = [];
class A {
  static x = 42;
  static #value = 7;
  static { events.push(this.x, this.#value); Object.preventExtensions(this); }
  static { events.push(this.x, this.#value); }
}
expect(A.x).toBe(42);
expect(events).toEqual([42, 7, 42, 7]);
expect(Object.isExtensible(A)).toBe(false);
const BlockOnly = class {
  static { Object.preventExtensions(this); }
  static { events.push("done"); }
};
expect(Object.isExtensible(BlockOnly)).toBe(false);
expect(events[4]).toBe("done");
