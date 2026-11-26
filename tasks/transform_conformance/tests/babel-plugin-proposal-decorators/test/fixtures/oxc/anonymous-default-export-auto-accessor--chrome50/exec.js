// An anonymous `export default` class with an undecorated static accessor is named "default".
let Default;
export default class {
  static accessor a = 1;
  static name_ = this.name;
  static {
    Default = this;
  }
}

expect(Default.a).toBe(1);
expect(Default.name_).toBe("default");
expect(Default.name).toBe("default");
