let receiver;
let replacement;
let value;
let events = [];

class Accessors {
  get #value() {
    events.push("get");
    receiver = replacement;
    return value;
  }

  set #value(next) {
    events.push(["set", this, next]);
  }

  static get #staticValue() {
    events.push("get");
    receiver = replacement;
    return value;
  }

  static set #staticValue(next) {
    events.push(["set", this, next]);
  }

  run(operator, rhs) {
    switch (operator) {
      case "&&=": return (receiver.#value &&= rhs());
      case "||=": return (receiver.#value ||= rhs());
      case "??=": return (receiver.#value ??= rhs());
    }
  }

  static run(operator, rhs) {
    switch (operator) {
      case "&&=": return (receiver.#staticValue &&= rhs());
      case "||=": return (receiver.#staticValue ||= rhs());
      case "??=": return (receiver.#staticValue ??= rhs());
    }
  }
}

for (const original of [new Accessors(), Accessors]) {
  for (const [operator, taken, skipped] of [["&&=", 1, 0], ["||=", 0, 1], ["??=", null, 1]]) {
    // Rebinding to a wrong-brand object must not redirect the write or throw.
    // Rebinding to a valid instance must still write to the original instance.
    for (replacement of [{}, new Accessors()]) {
      receiver = original;
      value = taken;
      events = [];
      expect(original.run(operator, () => {
        events.push("rhs");
        return 2;
      })).toBe(2);
      expect(events).toEqual(["get", "rhs", ["set", original, 2]]);

      receiver = original;
      value = skipped;
      events = [];
      expect(original.run(operator, () => {
        throw new Error("RHS must be skipped");
      })).toBe(skipped);
      expect(events).toEqual(["get"]);
    }

    receiver = {};
    events = [];
    expect(() => original.run(operator, () => {
      events.push("rhs");
    })).toThrow(TypeError);
    expect(events).toEqual([]);
  }
}

// The temporary receiver must belong to each invocation, including when the
// assignment is nested inside a default parameter and its getter reenters it.
class Recursive {
  constructor(name) {
    this.name = name;
  }

  get #value() {
    events.push("get:" + this.name);
    if (this.name === "outer") {
      receiver = inner;
      expect(inner.run()).toEqual([2]);
    }
    receiver = {};
    return 1;
  }

  set #value(next) {
    events.push("set:" + this.name);
  }

  run(result = [receiver.#value &&= (events.push("rhs:" + this.name), 2)]) {
    return result;
  }
}

const outer = new Recursive("outer");
const inner = new Recursive("inner");
receiver = outer;
events = [];
expect(outer.run()).toEqual([2]);
expect(events).toEqual(["get:outer", "get:inner", "rhs:inner", "set:inner", "rhs:outer", "set:outer"]);
