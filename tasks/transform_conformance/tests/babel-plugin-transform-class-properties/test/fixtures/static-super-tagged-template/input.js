class S {
  static method() {
    return this;
  }
}

class C extends S {
  static prop = super.method`xyz`;
}

expect(C.prop).toBe(C);
