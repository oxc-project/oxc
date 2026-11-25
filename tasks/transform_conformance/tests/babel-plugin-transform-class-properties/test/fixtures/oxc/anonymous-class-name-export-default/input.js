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
