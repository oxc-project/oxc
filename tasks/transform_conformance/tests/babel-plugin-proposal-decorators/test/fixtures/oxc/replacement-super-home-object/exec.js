class OriginalBase {
  static identify() { return ["original", this]; }
  static set record(value) { this.originalRecord = value; }
  identify() { return ["original", this]; }
}

class ReplacementBase {
  static identify() { return ["replacement", this]; }
  static set record(value) { this.replacementRecord = value; }
  identify() { return ["replacement", this]; }
}

let original;
function replace(value) {
  original = value;
  return class Replacement extends ReplacementBase {};
}

@replace
class Decorated extends OriginalBase {
  static field = super.identify();
  static { super.record = this; }
  static #read() { return super.identify(); }
  static read() { return this.#read(); }
  #readInstance() { return super.identify(); }
  read() { return this.#readInstance(); }
}

expect(Decorated.field[0]).toBe("original");
expect(Decorated.field[1]).toBe(Decorated);
expect(Decorated.originalRecord).toBe(Decorated);
expect(Decorated.replacementRecord).toBeUndefined();
expect(original.read()[0]).toBe("original");
expect(original.read()[1]).toBe(original);
const instance = new original();
expect(instance.read()[0]).toBe("original");
expect(instance.read()[1]).toBe(instance);

Object.setPrototypeOf(original, ReplacementBase);
expect(original.read()[0]).toBe("replacement");

const expression = @replace class extends OriginalBase {
  static field = super.identify();
};
expect(expression.field[0]).toBe("original");
expect(expression.field[1]).toBe(expression);
