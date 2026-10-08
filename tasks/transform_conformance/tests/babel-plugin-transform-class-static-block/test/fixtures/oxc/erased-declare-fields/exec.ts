const calls = [];
class Before {
  static { calls.push("before"); }
  declare static omitted: unknown;
}
class After {
  declare static omitted: unknown;
  static { calls.push("after"); }
}
class Mixed {
  static { calls.push("first"); }
  declare static omitted: unknown;
  static present = (calls.push("field"), 1);
  declare static trailing: unknown;
  static { calls.push("last"); }
}
expect(calls).toEqual(["before", "after", "first", "field", "last"]);
expect(Object.hasOwn(Before, "omitted")).toBe(false);
expect(Object.hasOwn(After, "omitted")).toBe(false);
expect(Object.hasOwn(Mixed, "omitted")).toBe(false);
expect(Object.hasOwn(Mixed, "trailing")).toBe(false);
expect(Mixed.present).toBe(1);
