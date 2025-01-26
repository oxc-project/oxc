let c;

class S {}

class C extends S {
  prop = 123;
  constructor() {
    super(c = super());
  }
}

try { new C() } catch {}

expect(c).toBeInstanceOf(C);
expect(c.prop).toBe(123);
