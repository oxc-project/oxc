let f;

class S {}

class C extends S {
  constructor(x) {
    super(super(), this.x = x, f = async () => this);
  }
}

try { new C(123) } catch {}

return f().then((c) => {
  expect(c).toBeInstanceOf(C);
  expect(c.x).toBe(123);
});
