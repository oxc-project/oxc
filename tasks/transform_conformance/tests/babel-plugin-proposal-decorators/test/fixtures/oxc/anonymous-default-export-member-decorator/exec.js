// An anonymous `export default` class with only member decorators is named "default".
const log = [];
let Default;
function dec(value, context) {
  log.push(context.name);
}
export default class {
  @dec method() {}
  static name_ = this.name;
  static {
    Default = this;
  }
}

expect(log).toEqual(["method"]);
expect(Default.name_).toBe("default");
expect(Default.name).toBe("default");
