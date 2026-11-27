const log: string[] = [];
const k = (name: string) => (log.push(name), name);

function argumentsKey(_k: string) {
  return class {
    [arguments[0]] = 1;
  };
}
expect(new (argumentsKey("x"))()).toEqual({ x: 1 });

function orderedInitializers(this: { key: string }) {
  const initialized: string[] = [];
  const field = (name: string) => (initialized.push(name), name);
  class C {
    static before = field("static before");
    static [this.key] = field("static computed");
    static after = field("static after");
    before = field("instance before");
    [this.key] = field("instance computed");
    after = field("instance after");
  }
  expect(C.x).toBe("static computed");
  expect(new C().x).toBe("instance computed");
  expect(initialized).toEqual([
    "static before", "static computed", "static after",
    "instance before", "instance computed", "instance after",
  ]);
}
orderedInitializers.call({ key: "x" });

let staticAssignment: number | undefined;
let instanceAssignment: number | undefined;
class WithSetters {
  static set x(value: number) { staticAssignment = value; }
  set x(value: number) { instanceAssignment = value; }
}
function withComputedAssignments() {
  return class extends WithSetters {
    static [(void this, "x")] = 1;
    [(void this, "x")] = 2;
  };
}
const Assigned = withComputedAssignments();
const assigned = new Assigned();
expect(staticAssignment).toBe(1);
expect(instanceAssignment).toBe(2);
expect(Object.hasOwn(Assigned, "x")).toBe(false);
expect(Object.hasOwn(assigned, "x")).toBe(false);

const erasedKeys: string[] = [];
function withErasedFields() {
  return class {
    static [(void this, erasedKeys.push("static"), "erased")]: number;
    [(void this, erasedKeys.push("instance"), "erased")]: number;
  };
}
const Erased = withErasedFields();
expect(erasedKeys).toEqual(["static", "instance"]);
expect(Object.hasOwn(Erased, "erased")).toBe(false);
expect(Object.hasOwn(new Erased(), "erased")).toBe(false);

class WithParameterProperty {
  constructor(public n: number) {}
  before = this.n;
  [(void this, "computed")] = this.n;
}
const withParameter = new WithParameterProperty(42);
expect(withParameter.before).toBe(42);
expect(withParameter.computed).toBe(42);

function* afterLastComputedKey() {
  return class {
    [(yield "a", k("a"))] = 1;
    [k("b")]() {}
    static [(yield "c", k("c"))] = 2;
    m() {}
  };
}
const ordered = afterLastComputedKey();
expect(ordered.next().value).toBe("a");
expect(ordered.next().value).toBe("c");
expect(log).toEqual(["a", "b"]);
const Ordered = ordered.next().value as any;
expect(log).toEqual(["a", "b", "c"]);
expect(Ordered.c).toBe(2);
expect(new Ordered().a).toBe(1);
expect(typeof Ordered.prototype.b).toBe("function");

let heritageLog: string[] = [];
function* afterHeritage() {
  return class extends (heritageLog.push("heritage"), Object) {
    static [(yield "a", heritageLog.push("key"), "x")] = 1;
  };
}
const inherited = afterHeritage();
expect(inherited.next().value).toBe("a");
expect(heritageLog).toEqual(["heritage"]);
const Inherited = inherited.next().value as any;
expect(heritageLog).toEqual(["heritage", "key"]);
expect(Inherited.x).toBe(1);

function* beforeDeclaration() {
  class C {
    static [yield "a"] = 1;
  }
  return C;
}
const declared = beforeDeclaration();
declared.next();
const Declared = declared.next("x").value as any;
expect(Declared.x).toBe(1);
expect(Declared.name).toBe("C");

function* beforeNamedExpression() {
  const C = class {
    static [yield "a"] = 1;
  };
  return C;
}
const named = beforeNamedExpression();
named.next();
const Named = named.next("x").value as any;
expect(Named.x).toBe(1);
expect(Named.name).toBe("C");

const Inner = "outer";
let readInner: () => unknown;
const NamedClass = class Inner {
  static [(this, readInner = () => Inner, "x")] = 1;
};
expect(readInner!()).toBe(NamedClass);

const keyLog: string[] = [];
const objectKey = {
  [Symbol.toPrimitive]() {
    keyLog.push("coerce");
    return "m";
  },
};
const OrderedKeys = class {
  [objectKey]() {}
  static [(this, keyLog.push("next"), "x")] = 1;
  static {
    expect(Object.getOwnPropertySymbols(this)).toEqual([]);
  }
};
expect(keyLog).toEqual(["coerce", "next"]);
expect(Object.getOwnPropertySymbols(OrderedKeys)).toEqual([]);
expect(OrderedKeys.x).toBe(1);

class Outer {
  #key = "x";
  *make(Symbol: undefined, globalThis: undefined) {
    return class {
      static [(yield "key", this.#key)] = 1;
    };
  }
}
const outer = new Outer().make(undefined, undefined);
expect(outer.next().value).toBe("key");
const PrivateKey = outer.next().value as any;
expect(PrivateKey.x).toBe(1);
expect(Object.getOwnPropertySymbols(PrivateKey)).toEqual([]);

try {
  const BeforeDeclaration = class {
    [notInitialized]() {}
    static [(this, keyLog.push("should not run"), "x")] = 1;
  };
} catch (error) {
  expect(error instanceof ReferenceError).toBe(true);
}
const notInitialized = "m";
expect(keyLog).toEqual(["coerce", "next"]);

return (async () => {
  async function awaitKey(key: Promise<string>) {
    return class {
      static [await key] = 1;
    };
  }
  expect(((await awaitKey(Promise.resolve("x"))) as any).x).toBe(1);
})();
