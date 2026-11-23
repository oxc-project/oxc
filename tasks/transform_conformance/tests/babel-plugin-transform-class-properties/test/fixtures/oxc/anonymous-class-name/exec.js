// Anonymous classes get their `name` from context (NamedEvaluation), or `""` if none.
// Transforming the class must not name it after a temp var.

function make(v) {
  return class extends v { static r = 1 };
}
expect(make(class {}).name).toBe("");

function id(c) {
  return c;
}
expect(id(class { static x = 1 }).name).toBe("");
expect(id(class { #m() {} }).name).toBe("");
expect((0, class { static x = 1 }).name).toBe("");
const holder = {};
holder.c = class { static x = 1 };
expect(holder.c.name).toBe("");

function withDefault(c = class { static x = 1 }) {
  return c;
}
expect(withDefault().name).toBe("c");

const obj = {
  x: class { static y = 1 },
  "a-b": class { static y = 1 },
  ["c"]: class { static y = 1 },
  ["__proto__"]: class { static y = 1 },
};
expect(obj.x.name).toBe("x");
expect(obj["a-b"].name).toBe("a-b");
expect(obj.c.name).toBe("c");
expect(Object.getOwnPropertyDescriptor(obj, "__proto__").value.name).toBe("__proto__");

const literalKeys = {
  1: class { static y = 1 },
  2n: class { static y = 1 },
  [3]: class { static y = 1 },
  [null]: class { static y = 1 },
};
expect(literalKeys[1].name).toBe("1");
expect(literalKeys[2].name).toBe("2");
expect(literalKeys[3].name).toBe("3");
expect(literalKeys.null.name).toBe("null");

const proto = { __proto__: class { static y = 1 } };
expect(Object.getPrototypeOf(proto).name).toBe("");

const C = class { static y = 1 };
expect(C.name).toBe("C");
let D;
D = class { static y = 1 };
expect(D.name).toBe("D");
let E;
E ??= class { static y = 1 };
expect(E.name).toBe("E");
const [F = class { static y = 1 }] = [];
expect(F.name).toBe("F");
let G;
({ G = class { static y = 1 } } = {});
expect(G.name).toBe("G");
const H = class { #m() {} };
expect(H.name).toBe("H");

class Outer {
  static inner = class { static y = 1 };
  static #privateInner = class { static y = 1 };
  static getPrivateInner() {
    return Outer.#privateInner;
  }
}
expect(Outer.inner.name).toBe("inner");
class NumericKey {
  static 1 = class { static y = 1 };
}
expect(NumericKey[1].name).toBe("1");
expect(Outer.getPrivateInner().name).toBe("#privateInner");

// Naming the class must not shadow an outer binding of the same name referenced inside it.
let Shadowed;
Shadowed = class {
  static y = 1;
  m() {
    return Shadowed;
  }
};
const instance = new Shadowed();
expect(Shadowed.name).toBe("Shadowed");
Shadowed = 1;
expect(instance.m()).toBe(1);

// Class which already has a name keeps it.
const Alias = class Named { static y = 1 };
expect(Alias.name).toBe("Named");

// Static initializers observe the final name.
const Observed = class { static n = this.name };
expect(Observed.n).toBe("Observed");
expect(id(class { static n = this.name }).n).toBe("");

// A static `name` member still overrides the inferred name.
const WithStaticName = class { static name = "custom"; static y = 1 };
expect(WithStaticName.name).toBe("custom");
