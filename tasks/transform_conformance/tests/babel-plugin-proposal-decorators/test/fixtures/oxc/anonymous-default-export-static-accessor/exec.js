// An anonymous `export default` class with a decorated static accessor is named "default",
// and the accessor can be read while the class is being defined.
let Default;
function dec({ get, set }, context) {
  return { get, set, init: (v) => v * 2 };
}
export default class {
  @dec static accessor a = 1;
  static #b = 2;
  static sum = this.a + this.#b;
  static name_ = this.name;
  static {
    Default = this;
  }
}

expect(Default.a).toBe(2);
Default.a = 5;
expect(Default.a).toBe(5);
expect(Default.sum).toBe(4);
expect(Default.name_).toBe("default");
expect(Default.name).toBe("default");
