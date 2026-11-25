// `export default class {}` is named "default", and its static elements run after the class
// (including its heritage) is defined.
const log = [];
export default class extends (log.push("heritage"), Object) {
  static x = log.push("field");
  static {
    log.push("block");
  }
  static {
    log.push(this.name);
  }
}

expect(log).toEqual(["heritage", "field", "block", "default"]);
