// Literal names retain each original method and the last definition wins.
{
  const seen = [];
  function dec(method) { seen.push(method()); return () => 3; }
  class C {
    @dec m() { return 1; }
    @dec m() { return 2; }
  }
  expect(seen).toEqual([1, 2]);
  expect(new C().m()).toBe(3);
  class D {
    @dec m() { return 4; }
    m() { return 5; }
  }
  expect(seen).toEqual([1, 2, 4]);
  expect(new D().m()).toBe(5);
}

// Each decorator receives its own method. Later undecorated definitions still win.
for (const key of ["m", Symbol("m"), Symbol()]) {
  for (const isStatic of [false, true]) {
    for (const decorateLast of [false, true]) {
      const seen = [];
      const originals = [];
      function dec(value, context) {
        expect(context.name).toBe(key);
        expect(context.static).toBe(isStatic);
        expect(context.kind).toBe("method");
        originals.push(value);
        seen.push(value.call({ value: 10 }));
        return function () { return value.call(this) + 100; };
      }
      class Base {
        m() { return this.value; }
        static m() { return this.value; }
      }
      const C = isStatic
        ? decorateLast
          ? class extends Base {
              @dec static [key]() { return super.m() + 1; }
              @dec static [key]() { return super.m() + 2; }
            }
          : class extends Base {
              @dec static [key]() { return super.m() + 1; }
              static [key]() { return super.m() + 2; }
            }
        : decorateLast
          ? class extends Base {
              @dec [key]() { return super.m() + 1; }
              @dec [key]() { return super.m() + 2; }
            }
          : class extends Base {
              @dec [key]() { return super.m() + 1; }
              [key]() { return super.m() + 2; }
            };
      expect(seen).toEqual(decorateLast ? [11, 12] : [11]);
      const target = isStatic ? C : C.prototype;
      const receiver = isStatic ? C : new C();
      receiver.value = 20;
      expect(receiver[key]()).toBe(decorateLast ? 122 : 22);
      const descriptor = Object.getOwnPropertyDescriptor(target, key);
      expect(descriptor.configurable).toBe(true);
      expect(descriptor.enumerable).toBe(false);
      expect(descriptor.writable).toBe(true);
      expect(Reflect.ownKeys(target).filter(k => typeof k === "symbol" && k !== (Symbol.metadata || Symbol.for("Symbol.metadata")))).toEqual(
        typeof key === "symbol" ? [key] : [],
      );
      const name = typeof key === "symbol"
        ? key.description === undefined ? "" : `[${key.description}]`
        : key;
      expect(originals[0].name).toBe(name);
      expect(() => new originals[0]()).toThrow(TypeError);
      Object.setPrototypeOf(target, { m() { return 30; } });
      expect(originals[0].call(receiver)).toBe(31);
    }
  }
}

// Repeated getters preserve the paired setter; a later setter preserves the getter.
for (const isStatic of [false, true]) {
  const values = [];
  function dec(value, context) {
    values.push([context.kind, value.name]);
    if (context.kind === "getter") {
      values.push(value.call({ value: 10 }));
      return function () { return value.call(this) + 100; };
    }
    return function (value) { this.value = value * 2; };
  }
  const C = isStatic ? class {
    @dec static get m() { return this.value + 1; }
    static set m(value) { this.value = value; }
    @dec static get m() { return this.value + 2; }
    @dec static set m(value) { this.value = value; }
  } : class {
    @dec get m() { return this.value + 1; }
    set m(value) { this.value = value; }
    @dec get m() { return this.value + 2; }
    @dec set m(value) { this.value = value; }
  };
  const receiver = isStatic ? C : new C();
  receiver.m = 5;
  expect(receiver.m).toBe(112);
  expect(values).toEqual([["getter", "get m"], 11, ["getter", "get m"], 12, ["setter", "set m"]]);
  const descriptor = Object.getOwnPropertyDescriptor(isStatic ? C : C.prototype, "m");
  expect(descriptor.get).toBeTypeOf("function");
  expect(descriptor.set).toBeTypeOf("function");
  expect(descriptor.enumerable).toBe(false);
}

// Switching between data and accessor descriptors removes the previous definition.
{
  const seen = [];
  function dec(value) { seen.push(value.call({ value: 10 })); }
  class C {
    @dec m() { return 1; }
    get m() { return this.value; }
    @dec get n() { return 2; }
    n() { return 3; }
  }
  expect(seen).toEqual([1, 2]);
  const c = new C();
  c.value = 4;
  expect(c.m).toBe(4);
  expect(c.n()).toBe(3);
  expect(Object.getOwnPropertyDescriptor(C.prototype, "m").value).toBeUndefined();
  expect(Object.getOwnPropertyDescriptor(C.prototype, "n").get).toBeUndefined();
}

// Auto-accessor getter/setter pairs are captured together; each storage still initializes.
for (const isStatic of [false, true]) {
  const events = [];
  function dec(value, context) {
    expect(context.kind).toBe("accessor");
    expect(value.get.name).toBe("get m");
    expect(value.set.name).toBe("set m");
    events.push("decorate");
    return {
      get() { return value.get.call(this) + 10; },
      set(v) { value.set.call(this, v * 2); },
      init(v) { events.push(v); return v + 1; },
    };
  }
  const C = isStatic ? class {
    @dec static accessor m = 1;
    @dec static accessor m = 2;
  } : class {
    @dec accessor m = 1;
    @dec accessor m = 2;
  };
  const receiver = isStatic ? C : new C();
  expect(events).toEqual(["decorate", "decorate", 1, 2]);
  expect(receiver.m).toBe(13);
  receiver.m = 4;
  expect(receiver.m).toBe(18);
}
{
  const values = [];
  function dec(value, context) { values.push(context.kind === "accessor" ? value.get.name : value.name); }
  class C {
    @dec accessor m = 1;
    m() { return 2; }
    @dec n() { return 3; }
    accessor n = 4;
  }
  const c = new C();
  expect(values).toEqual(["get m", "n"]);
  expect(c.m()).toBe(2);
  expect(c.n).toBe(4);
}

// Undecorated auto-accessor overrides do not consume decorated-field initializer slots.
{
  function dec(value, context) {
    if (context.kind === "field") return value => value * 10;
    return { init(value) { return value + 1; } };
  }
  class C {
    @dec accessor m = 1;
    accessor m = 2;
    @dec field = 3;
  }
  const c = new C();
  expect(c.m).toBe(2);
  expect(c.field).toBe(30);
}

// Raw auto-accessor pairs are initially visible before ordered method decoration.
for (const isStatic of [false, true]) {
  let initial;
  const pairs = [];
  function dec(value, context) {
    if (context.kind === "method") {
      const target = value();
      expect(context.access.has(target)).toBe(true);
      initial = Object.getOwnPropertyDescriptor(target, "m");
      expect(initial.value).toBeUndefined();
      expect(initial.enumerable).toBe(false);
    } else {
      pairs.push(value);
    }
  }
  const C = isStatic ? class C {
    @dec static m() { return C; }
    @dec static accessor m = 1;
    @dec static accessor m = 2;
  } : class C {
    @dec m() { return C.prototype; }
    @dec accessor m = 1;
    @dec accessor m = 2;
  };
  expect(initial.get).toBe(pairs[1].get);
  expect(initial.set).toBe(pairs[1].set);
  const receiver = isStatic ? C : new C();
  expect(receiver.m).toBe(2);
}

// Key coercion and decorator expressions keep their order and execute once per element.
{
  const events = [];
  const key = { [Symbol.toPrimitive]() { events.push("key"); return "m"; } };
  function dec(id) {
    events.push(`evaluate ${id}`);
    return function (method) { events.push(`decorate ${method()}`); };
  }
  class C {
    @dec(1) [key]() { return 1; }
    @dec(2) [key]() { return 2; }
    [key]() { return 3; }
  }
  expect(events).toEqual(["evaluate 1", "key", "evaluate 2", "key", "key", "decorate 1", "decorate 2"]);
  expect(new C().m()).toBe(3);
}
