const dec = () => {};

class K {
  @dec static assign() { K = null; }
  @dec static compound() { K += ""; }
  @dec static update() { K++; }
  static read() { return K; }
}

const E = class E2 {
  @dec static assign() { E2 = null; }
};

for (const fn of [K.assign, K.compound, K.update, E.assign]) {
  expect(fn).toThrow(TypeError);
}
expect(K.read()).toBe(K);
expect(typeof K).toBe("function");
