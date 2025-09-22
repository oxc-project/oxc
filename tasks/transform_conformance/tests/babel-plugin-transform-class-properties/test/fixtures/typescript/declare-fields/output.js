
class B {
  constructor(value) {
    babelHelpers.defineProperty(this, "value", 3);
    if (value !== undefined) {
      this.value = value;
    }
  }
}
class C extends B {
  log() {
    return "C " + this.value;
  }
}
expect(new C(6).log()).toBe("C 6");
