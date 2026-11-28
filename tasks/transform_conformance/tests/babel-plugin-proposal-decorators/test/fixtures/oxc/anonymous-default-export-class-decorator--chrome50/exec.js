// An anonymous `export default` class is named "default", also when it's decorated.
const log = [];
function dec(cls, context) {
  log.push(context.name, cls.name);
}
export default @dec class {
  static self = this;
  static {
    log.push(this.name);
  }
}

expect(log).toEqual(["default", "default", "default"]);
