// Unit tests for the patched `WeakMap` class (defined in `src-js/plugins/weak_map.ts`).
//
// These tests verify that the patched `WeakMap` behaves identically to a native `WeakMap`
// when used with normal object keys (i.e. not the `SOURCE_CODE` singleton).
//
// This test file runs in a separate process (via Vitest's `forks` pool) to ensure the
// `globalThis.WeakMap` patch does not leak into other unit tests.

import { afterEach, describe, expect, it, vi } from "vitest";

// Mock `SOURCE_CODE` to avoid pulling in heavy dependencies from `source_code.ts`.
// The mock just needs to be a unique frozen object, same as the real `SOURCE_CODE`.
const SOURCE_CODE = Object.freeze({ __mock: true });
vi.mock("../src-js/plugins/source_code.ts", () => ({ SOURCE_CODE }));

// Get native `WeakMap` class before patching
const WeakMapOriginal = globalThis.WeakMap;

// Import after mock is set up. This patches `globalThis.WeakMap`.
const { resetWeakMaps } = await import("../src-js/plugins/weak_map.ts");

it("`WeakMap` is patched", () => {
  // Sanity check: The import should have replaced `globalThis.WeakMap`
  expect(WeakMap).not.toBe(WeakMapOriginal);
  expect(WeakMap).not.toBe(undefined);

  const weakMap = new WeakMap();
  expect(weakMap).toBeInstanceOf(WeakMap);
  expect(weakMap).toBeInstanceOf(WeakMapOriginal);
});

// Test that all `WeakMap` methods work correctly for normal object keys,
// despite the global `WeakMap` class being patched
describe("`WeakMap` with normal keys", () => {
  it("constructor with no params", () => {
    const map = new WeakMap();
    expect(map).toBeInstanceOf(WeakMap);
  });

  it("constructor with `undefined`", () => {
    const map = new WeakMap(undefined);
    expect(map).toBeInstanceOf(WeakMap);
  });

  it("constructor with `null`", () => {
    const map = new WeakMap(null);
    expect(map).toBeInstanceOf(WeakMap);
  });

  it("constructor with empty array", () => {
    const map = new WeakMap([]);
    expect(map).toBeInstanceOf(WeakMap);
  });

  it("constructor with array", () => {
    const key1 = {};
    const key2 = {};
    const map = new WeakMap<object, string>([
      [key1, "a"],
      [key2, "b"],
    ]);
    expect(map.has(key1)).toBe(true);
    expect(map.has(key2)).toBe(true);
    expect(map.get(key1)).toBe("a");
    expect(map.get(key2)).toBe("b");
  });

  it("constructor with non-array iterable", () => {
    const key1 = {};
    const key2 = {};
    function* entries(): Generator<[object, string]> {
      yield [key1, "a"];
      yield [key2, "b"];
    }
    const map = new WeakMap<object, string>(entries());
    expect(map.has(key1)).toBe(true);
    expect(map.has(key2)).toBe(true);
    expect(map.get(key1)).toBe("a");
    expect(map.get(key2)).toBe("b");
  });

  it("`has`", () => {
    const map = new WeakMap<object, number>();
    const key = {};
    expect(map.has(key)).toBe(false);
    map.set(key, 42);
    expect(map.has(key)).toBe(true);
  });

  it("`get` returns `undefined` for missing key", () => {
    const map = new WeakMap<object, string>();
    expect(map.get({})).toBe(undefined);
  });

  it("`set` and `get`", () => {
    const map = new WeakMap<object, string>();
    const key = {};
    map.set(key, "hello");
    expect(map.get(key)).toBe("hello");
  });

  it("`set` overwrites existing value", () => {
    const map = new WeakMap<object, number>();
    const key = {};
    map.set(key, 1);
    map.set(key, 2);
    expect(map.get(key)).toBe(2);
  });

  it("`set` returns the map (for chaining)", () => {
    const map = new WeakMap<object, number>();
    const key = {};
    expect(map.set(key, 1)).toBe(map);
  });

  it("`delete`", () => {
    const map = new WeakMap<object, string>();
    const key = {};
    expect(map.delete(key)).toBe(false);
    map.set(key, "value");
    expect(map.delete(key)).toBe(true);
    expect(map.has(key)).toBe(false);
    expect(map.get(key)).toBe(undefined);
    expect(map.delete(key)).toBe(false);
  });

  it("multiple keys are independent", () => {
    const map = new WeakMap<object, string>();
    const key1 = {};
    const key2 = {};
    map.set(key1, "a");
    map.set(key2, "b");
    expect(map.get(key1)).toBe("a");
    expect(map.get(key2)).toBe("b");
  });

  it("`getOrInsert` returns existing value", () => {
    const map = new WeakMap<object, string>();
    const key = {};
    map.set(key, "existing");
    expect(map.getOrInsert(key, "new")).toBe("existing");
    expect(map.get(key)).toBe("existing");
  });

  it("`getOrInsert` inserts and returns default when key is missing", () => {
    const map = new WeakMap<object, string>();
    const key = {};
    expect(map.getOrInsert(key, "default")).toBe("default");
    expect(map.get(key)).toBe("default");
  });

  it("`getOrInsertComputed` returns existing value without calling callback", () => {
    const map = new WeakMap<object, string>();
    const key = {};
    map.set(key, "existing");
    const callback = vi.fn(() => "computed");
    expect(map.getOrInsertComputed(key, callback)).toBe("existing");
    expect(callback).not.toHaveBeenCalled();
  });

  it("`getOrInsertComputed` calls callback with key and inserts result", () => {
    const map = new WeakMap<object, string>();
    const key = {};
    const callback = vi.fn((k: object) => `computed-${k === key}`);
    expect(map.getOrInsertComputed(key, callback)).toBe("computed-true");
    expect(callback).toHaveBeenCalledOnce();
    expect(callback).toHaveBeenCalledWith(key);
    expect(map.get(key)).toBe("computed-true");
  });
});

// Test that the patched `WeakMap` correctly intercepts operations when the key is the `SOURCE_CODE` singleton,
// and that `resetWeakMaps` clears the stored values
describe("`WeakMap` with `SOURCE_CODE` key", () => {
  afterEach(() => {
    resetWeakMaps();
  });

  it("constructor with `SOURCE_CODE` in initial entries", () => {
    const map = new WeakMap<object, string>([[SOURCE_CODE, "source"]]);
    expect(map.has(SOURCE_CODE)).toBe(true);
    expect(map.get(SOURCE_CODE)).toBe("source");
  });

  it("constructor with `SOURCE_CODE` in non-array iterable", () => {
    function* entries(): Generator<[object, string]> {
      yield [SOURCE_CODE, "source"];
    }
    const map = new WeakMap<object, string>(entries());
    expect(map.has(SOURCE_CODE)).toBe(true);
    expect(map.get(SOURCE_CODE)).toBe("source");
  });

  it("constructor with `SOURCE_CODE` in initial entries is cleared by `resetWeakMaps`", () => {
    const map = new WeakMap<object, string>([[SOURCE_CODE, "source"]]);
    resetWeakMaps();
    expect(map.has(SOURCE_CODE)).toBe(false);
    expect(map.get(SOURCE_CODE)).toBe(undefined);
  });

  it("`has`", () => {
    const map = new WeakMap<object, number>();
    expect(map.has(SOURCE_CODE)).toBe(false);
    map.set(SOURCE_CODE, 42);
    expect(map.has(SOURCE_CODE)).toBe(true);
  });

  it("`get` returns `undefined` for missing key", () => {
    const map = new WeakMap<object, string>();
    expect(map.get(SOURCE_CODE)).toBe(undefined);
  });

  it("`set` and `get`", () => {
    const map = new WeakMap<object, string>();
    map.set(SOURCE_CODE, "file-data");
    expect(map.get(SOURCE_CODE)).toBe("file-data");
  });

  it("`set` overwrites existing value", () => {
    const map = new WeakMap<object, number>();
    map.set(SOURCE_CODE, 1);
    map.set(SOURCE_CODE, 2);
    expect(map.get(SOURCE_CODE)).toBe(2);
  });

  it("`set` returns the map (for chaining)", () => {
    const map = new WeakMap<object, number>();
    expect(map.set(SOURCE_CODE, 1)).toBe(map);
  });

  it("`delete`", () => {
    const map = new WeakMap<object, string>();
    expect(map.delete(SOURCE_CODE)).toBe(false);
    map.set(SOURCE_CODE, "data");
    expect(map.delete(SOURCE_CODE)).toBe(true);
    expect(map.has(SOURCE_CODE)).toBe(false);
    expect(map.get(SOURCE_CODE)).toBe(undefined);
    expect(map.delete(SOURCE_CODE)).toBe(false);
  });

  it("`getOrInsert` returns existing value", () => {
    const map = new WeakMap<object, string>();
    map.set(SOURCE_CODE, "existing");
    expect(map.getOrInsert(SOURCE_CODE, "new")).toBe("existing");
    expect(map.get(SOURCE_CODE)).toBe("existing");
  });

  it("`getOrInsert` inserts and returns default when key is missing", () => {
    const map = new WeakMap<object, string>();
    expect(map.getOrInsert(SOURCE_CODE, "default")).toBe("default");
    expect(map.has(SOURCE_CODE)).toBe(true);
    expect(map.get(SOURCE_CODE)).toBe("default");
  });

  it("`getOrInsertComputed` returns existing value without calling callback", () => {
    const map = new WeakMap<object, string>();
    map.set(SOURCE_CODE, "existing");
    const callback = vi.fn(() => "computed");
    expect(map.getOrInsertComputed(SOURCE_CODE, callback)).toBe("existing");
    expect(callback).not.toHaveBeenCalled();
    expect(map.has(SOURCE_CODE)).toBe(true);
    expect(map.get(SOURCE_CODE)).toBe("existing");
  });

  it("`getOrInsertComputed` calls callback with key and inserts result", () => {
    const map = new WeakMap<object, string>();
    const callback = vi.fn((k: object) => `computed-${k === SOURCE_CODE}`);
    expect(map.getOrInsertComputed(SOURCE_CODE, callback)).toBe("computed-true");
    expect(callback).toHaveBeenCalledOnce();
    expect(callback).toHaveBeenCalledWith(SOURCE_CODE);
    expect(map.has(SOURCE_CODE)).toBe(true);
    expect(map.get(SOURCE_CODE)).toBe("computed-true");
  });
});

// Test that `SOURCE_CODE` and normal key entries do not interfere with each other within the same `WeakMap`
describe("`WeakMap` with mixed keys", () => {
  afterEach(() => {
    resetWeakMaps();
  });

  it("constructor with `SOURCE_CODE` and normal keys in initial entries", () => {
    const key = {};
    const map = new WeakMap<object, string>([
      [key, "normal"],
      [SOURCE_CODE, "source"],
    ]);
    expect(map.has(SOURCE_CODE)).toBe(true);
    expect(map.get(SOURCE_CODE)).toBe("source");
    expect(map.has(key)).toBe(true);
    expect(map.get(key)).toBe("normal");
  });

  it("constructor with `SOURCE_CODE` in initial entries is cleared by `resetWeakMaps`", () => {
    const key = {};
    const map = new WeakMap<object, string>([
      [key, "normal"],
      [SOURCE_CODE, "source"],
    ]);
    resetWeakMaps();
    expect(map.has(SOURCE_CODE)).toBe(false);
    expect(map.get(SOURCE_CODE)).toBe(undefined);
    expect(map.has(key)).toBe(true);
    expect(map.get(key)).toBe("normal");
  });

  it("`has` with normal key is unaffected by `SOURCE_CODE` entry", () => {
    const map = new WeakMap<object, string>();
    map.set(SOURCE_CODE, "source");
    const key = {};
    expect(map.has(key)).toBe(false);
    map.set(key, "normal");
    expect(map.has(key)).toBe(true);
    map.delete(SOURCE_CODE);
    expect(map.has(key)).toBe(true);
    map.set(SOURCE_CODE, "source");
    expect(map.has(key)).toBe(true);
    expect(map.delete(key)).toBe(true);
    expect(map.has(key)).toBe(false);
  });

  it("`has` with `SOURCE_CODE` key is unaffected by normal entries", () => {
    const map = new WeakMap<object, string>();
    const key = {};
    map.set(key, "normal");
    expect(map.has(SOURCE_CODE)).toBe(false);
    map.set(SOURCE_CODE, "source");
    expect(map.has(SOURCE_CODE)).toBe(true);
    map.delete(key);
    expect(map.has(SOURCE_CODE)).toBe(true);
  });

  it("`get` with normal key is unaffected by `SOURCE_CODE` entry", () => {
    const map = new WeakMap<object, string>();
    map.set(SOURCE_CODE, "source");
    const key = {};
    expect(map.get(key)).toBe(undefined);
    map.set(key, "normal");
    expect(map.get(key)).toBe("normal");
    map.delete(SOURCE_CODE);
    expect(map.get(key)).toBe("normal");
  });

  it("`get` with `SOURCE_CODE` key is unaffected by normal entries", () => {
    const map = new WeakMap<object, string>();
    const key = {};
    map.set(key, "normal");
    expect(map.get(SOURCE_CODE)).toBe(undefined);
    map.set(SOURCE_CODE, "source");
    expect(map.get(SOURCE_CODE)).toBe("source");
    map.delete(key);
    expect(map.get(SOURCE_CODE)).toBe("source");
  });

  it("`set` with normal key does not affect `SOURCE_CODE` entry", () => {
    const map = new WeakMap<object, string>();
    map.set(SOURCE_CODE, "source");
    const key = {};
    map.set(key, "normal");
    expect(map.get(SOURCE_CODE)).toBe("source");
    expect(map.get(key)).toBe("normal");
    map.delete(key);
    expect(map.get(SOURCE_CODE)).toBe("source");
    map.set(SOURCE_CODE, "source2");
    expect(map.get(SOURCE_CODE)).toBe("source2");
  });

  it("`set` with `SOURCE_CODE` key does not affect normal entries", () => {
    const map = new WeakMap<object, string>();
    const key = {};
    map.set(key, "normal");
    map.set(SOURCE_CODE, "source");
    expect(map.get(key)).toBe("normal");
    expect(map.get(SOURCE_CODE)).toBe("source");
    map.delete(SOURCE_CODE);
    expect(map.get(key)).toBe("normal");
    map.set(key, "normal2");
    expect(map.get(key)).toBe("normal2");
  });

  it("`delete` normal key does not affect `SOURCE_CODE` entry", () => {
    const map = new WeakMap<object, string>();
    const key = {};
    map.set(key, "normal");
    map.set(SOURCE_CODE, "source");
    map.delete(key);
    expect(map.has(key)).toBe(false);
    expect(map.has(SOURCE_CODE)).toBe(true);
    expect(map.get(SOURCE_CODE)).toBe("source");
  });

  it("`delete` `SOURCE_CODE` key does not affect normal entries", () => {
    const map = new WeakMap<object, string>();
    const key = {};
    map.set(key, "normal");
    map.set(SOURCE_CODE, "source");
    map.delete(SOURCE_CODE);
    expect(map.has(SOURCE_CODE)).toBe(false);
    expect(map.has(key)).toBe(true);
    expect(map.get(key)).toBe("normal");
  });

  it("`getOrInsert` with normal key is unaffected by `SOURCE_CODE` entry", () => {
    const map = new WeakMap<object, string>();
    map.set(SOURCE_CODE, "source");
    const key = {};
    expect(map.getOrInsert(key, "default")).toBe("default");
    expect(map.get(key)).toBe("default");
    expect(map.get(SOURCE_CODE)).toBe("source");
    expect(map.getOrInsert(key, "altered")).toBe("default");
  });

  it("`getOrInsert` with `SOURCE_CODE` key is unaffected by normal entries", () => {
    const map = new WeakMap<object, string>();
    const key = {};
    map.set(key, "normal");
    expect(map.getOrInsert(SOURCE_CODE, "default")).toBe("default");
    expect(map.get(SOURCE_CODE)).toBe("default");
    expect(map.get(key)).toBe("normal");
    expect(map.getOrInsert(SOURCE_CODE, "altered")).toBe("default");
  });

  it("`getOrInsertComputed` with normal key is unaffected by `SOURCE_CODE` entry", () => {
    const map = new WeakMap<object, string>();
    map.set(SOURCE_CODE, "source");
    const key = {};
    const callback = vi.fn((k: object) => `computed-${k === key}`);
    expect(map.getOrInsertComputed(key, callback)).toBe("computed-true");
    expect(callback).toHaveBeenCalledOnce();
    expect(callback).toHaveBeenCalledWith(key);
    expect(map.get(SOURCE_CODE)).toBe("source");
  });

  it("`getOrInsertComputed` with `SOURCE_CODE` key is unaffected by normal entries", () => {
    const map = new WeakMap<object, string>();
    const key = {};
    map.set(key, "normal");
    const callback = vi.fn((k: object) => `computed-${k === SOURCE_CODE}`);
    expect(map.getOrInsertComputed(SOURCE_CODE, callback)).toBe("computed-true");
    expect(callback).toHaveBeenCalledOnce();
    expect(callback).toHaveBeenCalledWith(SOURCE_CODE);
    expect(map.get(key)).toBe("normal");
  });
});

// Additional tests for `resetWeakMaps`
describe("`resetWeakMaps`", () => {
  afterEach(() => {
    resetWeakMaps();
  });

  it("clears `SOURCE_CODE` entries", () => {
    const map = new WeakMap<object, string>();
    map.set(SOURCE_CODE, "file-data");
    resetWeakMaps();
    expect(map.has(SOURCE_CODE)).toBe(false);
    expect(map.get(SOURCE_CODE)).toBe(undefined);
  });

  it("does not affect normal keys", () => {
    const map = new WeakMap<object, string>();
    const key = {};
    map.set(key, "normal");
    map.set(SOURCE_CODE, "source");
    resetWeakMaps();
    expect(map.has(key)).toBe(true);
    expect(map.get(key)).toBe("normal");
    expect(map.has(SOURCE_CODE)).toBe(false);
    expect(map.get(SOURCE_CODE)).toBe(undefined);
  });

  it("`SOURCE_CODE` key can be re-set after reset", () => {
    const map = new WeakMap<object, string>();
    map.set(SOURCE_CODE, "file1");
    expect(map.get(SOURCE_CODE)).toBe("file1");
    resetWeakMaps();
    map.set(SOURCE_CODE, "file2");
    expect(map.get(SOURCE_CODE)).toBe("file2");
  });

  it("clears multiple `WeakMap`s", () => {
    const map1 = new WeakMap<object, string>();
    const map2 = new WeakMap<object, number>();
    map1.set(SOURCE_CODE, "data1");
    map2.set(SOURCE_CODE, 42);
    resetWeakMaps();
    expect(map1.has(SOURCE_CODE)).toBe(false);
    expect(map2.has(SOURCE_CODE)).toBe(false);
  });

  it("does not affect `WeakMap`s that have not used `SOURCE_CODE` key", () => {
    const mapWithSourceCode = new WeakMap<object, string>();
    const mapWithoutSourceCode = new WeakMap<object, string>();
    const key = {};
    mapWithSourceCode.set(SOURCE_CODE, "source");
    mapWithoutSourceCode.set(key, "normal");
    resetWeakMaps();
    expect(mapWithoutSourceCode.has(key)).toBe(true);
    expect(mapWithoutSourceCode.get(key)).toBe("normal");
  });

  it("`getOrInsert` inserts fresh value after reset", () => {
    const map = new WeakMap<object, string>();
    map.getOrInsert(SOURCE_CODE, "first");
    resetWeakMaps();
    expect(map.getOrInsert(SOURCE_CODE, "second")).toBe("second");
  });

  it("`getOrInsertComputed` calls callback again after reset", () => {
    const map = new WeakMap<object, string>();
    let callCount = 0;
    const callback = () => `call-${++callCount}`;
    expect(map.getOrInsertComputed(SOURCE_CODE, callback)).toBe("call-1");
    expect(map.getOrInsertComputed(SOURCE_CODE, callback)).toBe("call-1");
    resetWeakMaps();
    expect(map.getOrInsertComputed(SOURCE_CODE, callback)).toBe("call-2");
  });

  it("calling `resetWeakMaps` with no tracked `WeakMap`s is a no-op", () => {
    expect(() => resetWeakMaps()).not.toThrow();
  });

  it("calling `resetWeakMaps` twice is safe", () => {
    const map = new WeakMap<object, string>();
    map.set(SOURCE_CODE, "data");
    resetWeakMaps();
    resetWeakMaps();
    expect(map.has(SOURCE_CODE)).toBe(false);
  });
});
