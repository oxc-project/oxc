// Anonymous class and function initializers of class fields get their `name` from the field
// key (ClassFieldDefinitionEvaluation -> NamedEvaluation), even when fields are lowered.

let keyCalls = 0;
const key = () => (keyCalls++, "computed");
const k = "kk";
const sym = Symbol("desc");
const anonSym = Symbol();
const objKey = { toString: () => "fromToString" };

class F {
  x = class {};
  static y = class {};
  "z" = class {};
  #w = class {};
  static #sw = class {};
  1 = class {};
  static 2n = class {};
  "a-b" = class {};
  __proto__ = class {};
  fn = function () {};
  arrow = () => {};
  asyncFn = async function () {};
  gen = function* () {};
  paren = (class {});
  named = class Named {};
  [k] = class {};
  static [k] = () => {};
  [sym] = class {};
  [anonSym] = class {};
  static [sym] = function () {};
  [key()] = class {};
  ["lit"] = function () {};
  [`tpl`] = () => {};
  [objKey] = class {};
  static privates(f) {
    return [f.#w.name, F.#sw.name];
  }
}

const f = new F();
expect(f.x.name).toBe("x");
expect(F.y.name).toBe("y");
expect(f.z.name).toBe("z");
expect(F.privates(f)).toEqual(["#w", "#sw"]);
expect(f[1].name).toBe("1");
expect(F[2].name).toBe("2");
expect(f["a-b"].name).toBe("a-b");
expect(Object.getOwnPropertyDescriptor(f, "__proto__").value.name).toBe("__proto__");
expect(f.fn.name).toBe("fn");
expect(f.arrow.name).toBe("arrow");
expect(f.gen.name).toBe("gen");
expect(f.paren.name).toBe("paren");
expect(f.named.name).toBe("Named");
expect(f.kk.name).toBe("kk");
expect(F.kk.name).toBe("kk");
expect(f[sym].name).toBe("[desc]");
expect(f[anonSym].name).toBe("");
expect(F[sym].name).toBe("[desc]");
expect(f.computed.name).toBe("computed");
expect(f.lit.name).toBe("lit");
expect(f.tpl.name).toBe("tpl");
expect(f.fromToString.name).toBe("fromToString");

// Computed key is evaluated once, at class definition
new F();
expect(keyCalls).toBe(1);

// Naming must not shadow an outer binding referenced inside the class or function
const shadowed = "outer";
class Shadow {
  shadowed = class {
    m() {
      return shadowed;
    }
  };
  static shadowed = function () {
    return shadowed;
  };
}
expect(new Shadow().shadowed.name).toBe("shadowed");
expect(new new Shadow().shadowed().m()).toBe("outer");
expect(Shadow.shadowed.name).toBe("shadowed");
expect(Shadow.shadowed()).toBe("outer");

// A function's own parameter or `var` with the same name
class Param {
  p = function (p) {
    return p;
  };
  v = function () {
    var v = 1;
    return v;
  };
}
expect(new Param().p.name).toBe("p");
expect(new Param().p(2)).toBe(2);
expect(new Param().v.name).toBe("v");
expect(new Param().v()).toBe(1);

// Anonymous classes which are lowered too are named before their static initializers run
const dyn = "dyn";
class Nested {
  inst = class { static n = this.name; static #p = 1; };
  static stat = class { static n = this.name; };
  [dyn + "Inst"] = class { static n = this.name; };
  static [dyn + "Stat"] = class { static n = this.name; };
  static [sym] = class { static n = this.name; };
}
const nested = new Nested();
expect(nested.inst.name).toBe("inst");
expect(nested.inst.n).toBe("inst");
expect(Nested.stat.name).toBe("stat");
expect(Nested.stat.n).toBe("stat");
expect(nested.dynInst.name).toBe("dynInst");
expect(nested.dynInst.n).toBe("dynInst");
expect(Nested.dynStat.name).toBe("dynStat");
expect(Nested.dynStat.n).toBe("dynStat");
expect(Nested[sym].name).toBe("[desc]");
expect(Nested[sym].n).toBe("[desc]");

// A static `name` member still overrides the inferred name
class StaticName {
  a = class { static name = "custom"; };
  [k] = class { static name() {} };
  static [dyn] = class { static name = "custom"; static q = 1; };
  static get = class { static get name() { return "getter"; } };
}
expect(new StaticName().a.name).toBe("custom");
expect(typeof new StaticName().kk.name).toBe("function");
expect(StaticName.dyn.name).toBe("custom");
expect(StaticName.get.name).toBe("getter");

// Not anonymous function definitions: no name inferred
class NotAnonymous {
  seq = (0, class {});
  call = ((c) => c)(class {});
}
expect(new NotAnonymous().seq.name).toBe("");
expect(new NotAnonymous().call.name).toBe("");
expect(f.asyncFn.name).toBe("asyncFn");
