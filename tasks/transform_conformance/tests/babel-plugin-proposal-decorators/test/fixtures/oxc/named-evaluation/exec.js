{
  const methodDecorator = value => {
    expect(typeof value).toBe("function");
    return value;
  };
  const C = class {
    @methodDecorator static name() { return 1; }
  };
  expect(C.name()).toBe(1);
  const D = @((value, context) => {
    expect(context.name).toBe("D");
    expect(typeof value.name).toBe("function");
  }) class {
    static name() { return 2; }
  };
  expect(D.name()).toBe(2);
  let getterCalls = 0;
  const getterKey = "name";
  const E = class {
    @(() => {}) static get [getterKey]() { getterCalls++; return 3; }
  };
  expect(getterCalls).toBe(0);
  expect(E.name).toBe(3);
  const names = [];
  const F = class {
    accessor value;
    static { names.push(this.name); }
  };
  expect(names).toEqual(["F"]);
}

{
  const noop = () => {};
  const classes = [];
  for (let i = 0; i < 2; i++)
    classes.push(class {
      [i ? "b" : ""] = @noop class {}
    });
  expect(new classes[0]() [""].name).toBe("");
  expect(new classes[1]().b.name).toBe("b");
}

// prettier-ignore
const logs = [];
const decFactory = (logs) => (value, context) => {
  expect(value.name).toEqual(context.name);
  logs.push(context.name);
  return value;
};
const dec = decFactory(logs);

const defaultValue = @dec class {}

const atypical = @dec class {}

expect(logs).toEqual(["defaultValue", "atypical"]);

const noop = () => {}

{
  const logs = [];
  const dec = decFactory(logs);

  var A0 = @dec class {};
  let A1 = @dec class { static {} };
  const A2 = @dec class extends A1 {}

  expect(logs).toEqual(["A0", "A1", "A2"]);
}

{
  const logs = [];
  const dec = decFactory(logs);
  const f = () => { logs.push("computing f"); return 8. }

  ({
    A0: @dec class {},
    "1": @dec class { static {} },
    2: @dec class extends class {} {},
    3n: @dec class extends class {} { static {} },
    ["4"]: @dec class { p; },
    [5]: @dec class { p; },
    [6n]: @dec class { p; },
    [f()]: @dec class { @dec static 7() {} },
    [{ [Symbol.toPrimitive]: () => (logs.push("computing symbol"), Symbol(9)) }]: @dec class { p; }
  });

  expect(logs).toEqual(["A0", "1", "2", "3", "4", "5", "6", "computing f", "7", "8", "computing symbol", "[9]"]);
}

{
  const logs = [];
  const dec = decFactory(logs);
  const f = () => { logs.push("computing f"); return 8. }

  class C {
    A0 = @dec class {};
    "1" = @dec class { static {} };
    2 = @dec class extends class {} {};
    3n = @dec class extends class {} { static {} };
    ["4"] = @dec class { p; };
    [5] = @dec class { p; };
    [6n] = @dec class { p; };
    [f()] = @dec class { @dec static 7() {} };
    [{ [Symbol.toPrimitive]: () => (logs.push("computing symbol"), Symbol(9)) }] = @dec class { p; };
    #_10 = @dec class {};
  }

  new C();

  expect(logs).toEqual(["computing f", "computing symbol", "A0", "1", "2", "3", "4", "5", "6", "7", "8", "[9]", "#_10"]);
}

{
  const logs = [];
  const dec = decFactory(logs);
  const f = () => { logs.push("computing f"); return 8. }

  class C {
    static A0 = @dec class {};
    static "1" = @dec class { static {} };
    static 2 = @dec class extends class {} {};
    static 3n = @dec class extends class {} { static {} };
    static ["4"] = @dec class { p; };
    static [5] = @dec class { p; };
    static [6n] = @dec class { p; };
    static [f()] = @dec class { @dec static 7() {} };
    static [{ [Symbol.toPrimitive]: () => (logs.push("computing symbol"), Symbol(9)) }] = @dec class { p; };
    static #_10 = @dec class {};
  }

  expect(logs).toEqual(["computing f", "computing symbol", "A0", "1", "2", "3", "4", "5", "6", "7", "8", "[9]", "#_10"]);
}

{
  const logs = [];
  const dec = decFactory(logs);
  const f = () => { logs.push("computing f"); return 8. }

  class C {
    accessor A0 = @dec class {};
    accessor "1" = @dec class { static {} };
    accessor 2 = @dec class extends class {} {};
    accessor 3n = @dec class extends class {} { static {} };
    accessor ["4"] = @dec class { p; };
    accessor [5] = @dec class { p; };
    accessor [6n] = @dec class { p; };
    accessor [f()] = @dec class { @dec static 7() {} };
    accessor [{ [Symbol.toPrimitive]: () => (logs.push("computing symbol"), Symbol(9)) }] = @dec class { p; };
    accessor #_10 = @dec class {};
  }

  new C();

  expect(logs).toEqual(["computing f", "computing symbol", "A0", "1", "2", "3", "4", "5", "6", "7", "8", "[9]", "#_10"]);
}

{
  const logs = [];
  const dec = decFactory(logs);
  const f = () => { logs.push("computing f"); return 8. }

  class C {
    static accessor A0 = @dec class {};
    static accessor "1" = @dec class { static {} };
    static accessor 2 = @dec class extends class {} {};
    static accessor 3n = @dec class extends class {} { static {} };
    static accessor ["4"] = @dec class { p; };
    static accessor [5] = @dec class { p; };
    static accessor [6n] = @dec class { p; };
    static accessor [f()] = @dec class { @dec static 7() {} };
    static accessor [{ [Symbol.toPrimitive]: () => (logs.push("computing symbol"), Symbol(9)) }] = @dec class { p; };
    static accessor #_10 = @dec class {};
  }

  expect(logs).toEqual(["computing f", "computing symbol", "A0", "1", "2", "3", "4", "5", "6", "7", "8", "[9]", "#_10"]);
}

{
  const logs = [];
  const dec = decFactory(logs);
  const f = () => { logs.push("computing f"); return 8. }

  class C {
    static accessor A0 = @dec class {};
    static accessor "1" = @dec class { static {} };
    static accessor 2 = @dec class extends class {} {};
    static accessor 3n = @dec class extends class {} { static {} };
    static accessor ["4"] = @dec class { static accessor p; };
    static accessor [5] = @dec class { @noop static accessor #p; };
    static accessor [6n] = @dec class { accessor p; };
    static accessor [f()] = @dec class { @dec static 7() {} };
    static accessor [{ [Symbol.toPrimitive]: () => (logs.push("computing symbol"), Symbol(9)) }] = @dec class { @noop accessor p; };
    static accessor #_10 = @dec class {};
  }

  expect(logs).toEqual(["computing f", "computing symbol", "A0", "1", "2", "3", "4", "5", "6", "7", "8", "[9]", "#_10"]);
}

{
  const logs = [];
  const dec = () => {
    return {
      init(v) {
        logs.push(v.name);
        return v;
      }
    }
  };
  const f = () => { logs.push("computing f"); return 8. }

  class C {
    @dec static accessor A0 = class { static accessor q; };
    @dec static accessor "1" = class { accessor q; };
    @dec static accessor 2 = class extends class {} { static accessor p; };
    @dec static accessor 3n = class extends class {} { accessor #q; };
    @dec static accessor ["4"] = class { static accessor p; };
    @dec static accessor [5] = class { static accessor #p; };
    @dec static accessor [6n] = class { accessor p; };
    @dec static accessor [f()] = class { @dec static accessor 7 = class { static accessor p }; };
    @dec static accessor [{ [Symbol.toPrimitive]: () => (logs.push("computing symbol"), Symbol(9)) }] = class { static accessor p; };
    @dec static accessor #_10 = class { static accessor #p; };
  }

  expect(logs).toEqual(["computing f", "computing symbol", "A0", "1", "2", "3", "4", "5", "6", "7", "8", "[9]", "#_10"]);
}

{
  const logs = [];
  const dec = decFactory(logs);
  // __proto__ setter should not name anonymous class
  ({
    __proto__: @dec class {},
  });
  ({
    "__proto__": @dec class {},
  });
  expect(logs).toEqual(["", ""]);
}

{
  const logs = [];
  const dec = decFactory(logs);
  // __proto__ has no special meaning in class fields
  class A extends class {
    static __proto__ = @dec class {}
  } {
    static "__proto__" = @dec class {}
  }
  expect(logs).toEqual(["__proto__", "__proto__"]);
}
