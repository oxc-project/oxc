const log = [];
const keep = (value) => { log.push("decorate"); return value; };

@keep
class Decorated {
  static #value = 1;
  static { log.push(this.#value); }
  static read() { return this.#value; }
}

class Plain {
  static #value = 2;
  static { log.push(this.#value); }
  static read() { return this.#value; }
}

class Accessor {
  accessor value = 3;
  static accessor count = 4;
  static #value = 5;
  static { log.push(this.#value); }
  static read() { return this.#value; }
}

expect(log).toEqual(["decorate", 1, 2, 5]);
expect([Decorated.read(), Plain.read(), Accessor.read(), new Accessor().value]).toEqual([1, 2, 5, 3]);
class Derived extends Accessor {}
Derived.count = 6;
expect(Accessor.count).toBe(6);

const Anonymous = class {
  static accessor value = this.name;
  static { log.push(this.value); }
};
expect([Anonymous.name, Anonymous.value, log[4]]).toEqual(["Anonymous", "Anonymous", "Anonymous"]);

class Outer {
  #value = { read() { return this === outerValue; } };
  create() {
    const outer = this;
    @keep class Inner {
      #own = 7;
      static { log.push("inner"); }
      read() { return outer.#value?.read() && this.#own; }
    }
    return new Inner();
  }
  getValue() { return this.#value; }
}
const outer = new Outer();
const outerValue = outer.getValue();
expect(outer.create().read()).toBe(7);
