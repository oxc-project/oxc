const p = (class {
  static get p() {
    return this.name || "empty";
  }
}).p ||= 1;
const obj = {};
obj[() => {}] ??= 1;
