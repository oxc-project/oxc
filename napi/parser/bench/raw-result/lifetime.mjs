// oxlint-disable no-console, typescript/unbound-method
import assert from "node:assert/strict";
import v8 from "node:v8";

const [variant = "closure", operation = "discard"] = process.argv.slice(2);
const retained = [];
const nextTurn = () => new Promise((resolve) => setImmediate(resolve));
const descriptor = Object.getOwnPropertyDescriptor;
// oxlint-disable-next-line typescript/no-implied-eval -- V8 intrinsic syntax requires a flagged runtime.
const inYoungGeneration = new Function("value", "return %InYoungGeneration(value)");
class ResultBase {
  constructor(result) {
    return result;
  }
}
class LiteralResult extends ResultBase {
  #data;
  constructor(data) {
    super({
      get program() {
        return this.#data;
      },
    });
    this.#data = data;
  }
}
let privateDescriptors;
class PrivateResult {
  #data;
  constructor(data) {
    this.#data = data;
    Object.defineProperties(this, privateDescriptors);
  }
  get program() {
    return this.#data;
  }
  static {
    const program = Object.getOwnPropertyDescriptor(this.prototype, "program");
    program.enumerable = true;
    privateDescriptors = { program };
  }
}
const shared = descriptor(
  {
    get program() {
      return this;
    },
  },
  "program",
).get;
function wrap(data) {
  if (variant === "closure") {
    return {
      get program() {
        return data;
      },
    };
  }
  if (variant === "field") {
    const program = data;
    return {
      get program() {
        return program;
      },
    };
  }
  if (variant === "define") {
    return Object.defineProperty({}, "program", {
      get() {
        return data;
      },
      configurable: true,
      enumerable: true,
    });
  }
  if (variant === "bound") {
    return Object.defineProperty({}, "program", {
      get: shared.bind(data),
      configurable: true,
      enumerable: true,
    });
  }
  if (variant === "private") return Reflect.construct(PrivateResult, [data], Object);
  if (variant === "literal") return new LiteralResult(data);
  if (variant === "plain") return { program: data };
  throw new Error(variant);
}
for (let i = 0; i < 10000; i++) wrap({ value: i });
await nextTurn();
globalThis.gc();
function allocate() {
  const refs = [];
  let initialYoung = 0;
  for (let i = 0; i < 128; i++) {
    const payload = { body: Array.from({ length: 64 }, (_, value) => ({ value })) };
    const result = wrap(payload);
    initialYoung += inYoungGeneration(payload) ? 1 : 0;
    refs.push({
      payload: new WeakRef(payload),
      wrapper: new WeakRef(result),
      getter: descriptor(result, "program").get
        ? new WeakRef(descriptor(result, "program").get)
        : null,
    });
    if (operation === "delete") {
      for (const key of Reflect.ownKeys(result)) delete result[key];
      retained.push(result);
    }
  }
  return { refs, initialYoung };
}
const { refs, initialYoung } = allocate();
await nextTurn();
console.log("LIFETIME_START");
globalThis.gc({ type: "minor", execution: "sync" });
globalThis.gc({ type: "minor", execution: "sync" });
const alive = () => ({
  payload: refs.filter((ref) => ref.payload.deref()).length,
  wrapper: refs.filter((ref) => ref.wrapper.deref()).length,
  getter: refs.filter((ref) => ref.getter?.deref()).length,
  youngPayload: refs.filter((ref) => {
    const value = ref.payload.deref();
    return value && inYoungGeneration(value);
  }).length,
});
const minor = alive();
await nextTurn();
globalThis.gc();
const major = alive();
console.log("LIFETIME_END");
assert.equal(initialYoung, refs.length);
assert.equal(
  major.payload,
  operation === "delete" && ["private", "literal"].includes(variant) ? refs.length : 0,
);
assert.equal(major.wrapper, operation === "delete" ? refs.length : 0);
let afterRelease;
if (operation === "delete") {
  retained.length = 0;
  await nextTurn();
  globalThis.gc();
  afterRelease = alive();
  assert.equal(afterRelease.payload, 0);
}
console.log(
  JSON.stringify({
    variant,
    operation,
    afterRelease,
    node: process.version,
    v8: process.versions.v8,
    flags: process.execArgv,
    initialYoung,
    minor,
    major,
    spaces: v8.getHeapSpaceStatistics(),
  }),
);
