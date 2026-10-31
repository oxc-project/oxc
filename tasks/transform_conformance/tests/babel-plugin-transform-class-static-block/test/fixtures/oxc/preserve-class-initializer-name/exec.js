class A {
  static {}
  static C = class { static seen = this.name; };
  static {}
  static Custom = class { static name = "custom"; };
  static {}
  static Frozen = class { static { Object.freeze(this); } };
  static {}
  static #Private = class { static seen = this.name; };
  static {}
  static getPrivate() { return this.#Private; }
  static Method = class { static name() {} static seen = this.name; };
  static {}
  static Getter = class { static get name() { return "getter"; } static seen = this.name; };
  static {}
}
expect([A.C.name, A.C.seen]).toEqual(["C", "C"]);
expect(A.Custom.name).toBe("custom");
expect(A.Frozen.name).toBe("Frozen");
expect(Object.isFrozen(A.Frozen)).toBe(true);
expect([A.getPrivate().name, A.getPrivate().seen]).toEqual(["#Private", "#Private"]);
expect(A.Method.seen).toBe(A.Method.name);
expect(typeof A.Method.name).toBe("function");
expect([A.Getter.name, A.Getter.seen]).toEqual(["getter", "getter"]);
const MethodName = class { static name() {} static { this.seen = this.name; } };
expect(MethodName.seen).toBe(MethodName.name);
expect(typeof MethodName.name).toBe("function");
const methodKey = "name";
const ComputedMethodName = class { static [methodKey]() {} static { this.seen = this.name; } };
expect(ComputedMethodName.seen).toBe(ComputedMethodName.name);
expect(typeof ComputedMethodName.name).toBe("function");
