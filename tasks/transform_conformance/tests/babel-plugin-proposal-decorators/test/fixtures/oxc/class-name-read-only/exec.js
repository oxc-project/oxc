const dec = () => {};

class K {
  @dec static assign() { K = null; }
  @dec static compound() { K += ""; }
  @dec static update() { K++; }
  static read() { return K; }
}

const E = class E2 {
  @dec static assign() { E2 = null; }
};

for (const fn of [K.assign, K.compound, K.update, E.assign]) {
  expect(fn).toThrow(TypeError);
}
expect(K.read()).toBe(K);
expect(typeof K).toBe("function");

// Give each write a fresh class so a failed immutable write cannot hide later failures.
function checkWrite(method) {
  const events = [];
  const sink = {
    set before(value) { events.push("before"); },
    set after(value) { events.push("after"); },
  };
  const iterable = {
    [Symbol.iterator]() {
      return {
        next() { events.push("next"); return { value: undefined, done: false }; },
        return() { events.push("close"); return { done: true }; },
      };
    },
  };
  class K {
    @dec static and() { return K &&= (events.push("rhs"), null); }
    static or() { return K ||= (events.push("rhs"), null); }
    static nullish() { return K ??= (events.push("rhs"), null); }
    static array() { [sink.before, K, sink.after] = [1, null, 3]; }
    static arrayDefault() { [K = events.push("default")] = iterable; }
    static arrayRest() { [...K] = [1, 2]; }
    static object() {
      ({ [(events.push("key"), "value")]: K } = {
        get value() { events.push("get"); return null; },
      });
    }
    static shorthand() { ({ K } = { get K() { events.push("get"); return null; } }); }
    static shorthandDefault() { ({ K = events.push("default") } = {}); }
    static objectRest() {
      ({ ...K } = { get value() { events.push("get"); return null; } });
    }
    static forOf() { for (K of iterable) { events.push("body"); break; } }
    static forOfPattern() { for ([K] of [[null]]) events.push("body"); }
    static forOfEmpty() { for (K of []) events.push("body"); }
    static forIn() { for (K in (events.push("rhs"), { key: 1 })) events.push("body"); }
    static forInEmpty() { for (K in {}) events.push("body"); }
    static shadowed() {
      function write(K) { [K] = [null]; return K; }
      events.push(write(1));
    }
    static read() { return K; }
  }
  let threw = false;
  let result;
  try { result = K[method](); } catch (error) {
    if (!(error instanceof TypeError)) throw error;
    threw = true;
  }
  return { method, threw, sameClass: K.read() === K, events,
    shortCircuit: method === "or" || method === "nullish" ? result === K : undefined };
}

const cases = [
  ["and", true, ["rhs"]],
  ["or", false, []],
  ["nullish", false, []],
  ["array", true, ["before"]],
  ["arrayDefault", true, ["next", "default", "close"]],
  ["arrayRest", true, []],
  ["object", true, ["key", "get"]],
  ["shorthand", true, ["get"]],
  ["shorthandDefault", true, ["default"]],
  ["objectRest", true, ["get"]],
  ["forOf", true, ["next", "close"]],
  ["forOfPattern", true, []],
  ["forOfEmpty", false, []],
  ["forIn", true, ["rhs"]],
  ["forInEmpty", false, []],
  ["shadowed", false, [null]],
];
expect(cases.map(([method]) => checkWrite(method))).toEqual(
  cases.map(([method, threw, events]) => ({ method, threw, sameClass: true, events,
    shortCircuit: method === "or" || method === "nullish" ? true : undefined }))
);
