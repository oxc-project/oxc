const classes = [];
for (let i = 0; i < 2; i++) {
  const object = classes[i - 1] || {};
  let has;
  class C {
    static #m() {}
    [(C => #m in C)(object)]() {}
    [(C => #m in C)(object)] = 1;
    [(has = C => #m in C, "key")]() {}
  }
  expect(Object.getOwnPropertyNames(C.prototype)).toEqual(["constructor", "false", "key"]);
  expect(Object.keys(new C())).toEqual(["false"]);
  expect(has(C)).toBe(true);
  expect(has(object)).toBe(false);
  classes.push(C);
}

const object = {};
class StaticKey {
  static #m() {}
  static [(StaticKey => #m in StaticKey)(object)] = 2;
}
expect(Object.keys(StaticKey)).toEqual(["false"]);
