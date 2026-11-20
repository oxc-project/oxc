const contexts = new Map();
const names = [];
function record(value, context) {
  contexts.set(context.name, context);
  if (context.kind !== "field") names.push(value && value.name);
  if (context.kind === "method") {
    context.addInitializer(function () {
      expect(context.access.has(this)).toBe(true);
      expect(context.access.get(this).call(this)).toBe(7);
    });
    return function () { return 7; };
  }
  if (context.kind === "accessor") {
    expect(value.get.name).toBe("get #auto");
    expect(value.set.name).toBe("set #auto");
    return { init: value => value + 1 };
  }
}
class C {
  @record #field = 1;
  @record #method() { return 2; }
  @record accessor #auto = 3;
  set #pair(value) { this.#field = value; }
  @record get #pair() { return this.#field; }
  read() { return [this.#field, this.#method(), this.#auto, this.#pair]; }
}
const c = new C();
expect(c.read()).toEqual([1, 7, 4, 1]);
expect(names.slice(0, 1)).toEqual(["#method"]);
for (const context of contexts.values()) {
  expect(context.private).toBe(true);
  expect(context.access.has(c)).toBe(true);
  expect(context.access.has({})).toBe(false);
  expect(() => context.access.has(1)).toThrow(TypeError);
  if (context.access.get) expect(() => context.access.get({})).toThrow(TypeError);
  if (context.access.set) {
    expect(() => context.access.set({}, 2)).toThrow(TypeError);
    expect(context.access.set(c, 5)).toBeUndefined();
  }
}
expect(contexts.get("#method").access.get(c)).toBe(contexts.get("#method").access.get(c));
expect(Object.keys(contexts.get("#pair").access)).toEqual(["has", "get"]);

let original, fieldAccess, methodAccess;
function replace(C) { original = C; return class {}; }
@replace
class Static {
  @((_, context) => { fieldAccess = context.access; }) static #field = 4;
  @((value, context) => { methodAccess = context.access; }) static #method() {}
  static {
    expect(fieldAccess.has(this)).toBe(true);
    expect(methodAccess.has(this)).toBe(false);
  }
}
expect(fieldAccess.has(Static)).toBe(true);
expect(fieldAccess.has(original)).toBe(false);
expect(methodAccess.has(Static)).toBe(false);
expect(methodAccess.has(original)).toBe(true);
expect(fieldAccess.get(Static)).toBe(4);
expect(fieldAccess.set(Static, 5)).toBeUndefined();
expect(fieldAccess.get(Static)).toBe(5);
expect(() => methodAccess.get(Static)).toThrow(TypeError);
