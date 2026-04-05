/**
 * Patch `WeakMap`, to emulate how a `WeakMap` keyed by `context.sourceCode` would behave if every file
 * had a different value for `context.sourceCode` (as it does in ESLint).
 *
 * Oxlint differs from ESLint in that `context.sourceCode` is always the singleton `SOURCE_CODE`,
 * which is constant across all rules and files.
 *
 * This breaks plugins which use `WeakMap`s keyed by `context.sourceCode` to store data for each file,
 * shared between different rules, as they rely on `sourceCode` being different for every file.
 * This patch to `WeakMap` solves that problem.
 *
 * See: https://github.com/oxc-project/oxc/issues/20700
 */

import { SOURCE_CODE } from "./source_code.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

/**
 * Entry in `trackedWeakMaps` array representing a `WeakMap` which has been used with `SOURCE_CODE` as key.
 */
interface TrackedWeakMap {
  // `WeakRef` containing `WeakMap` instance
  ref: WeakRef<PatchedWeakMap<WeakKey, unknown>>;
  // Index of this entry in `trackedWeakMaps` array
  index: number;
}

// `WeakMap`s which have been used with `SOURCE_CODE` as key.
const trackedWeakMaps: TrackedWeakMap[] = [];

// `FinalizationRegistry` to remove entries from `trackedWeakMaps` array when the `WeakMap` they hold
// is garbage collected.
const registry = new FinalizationRegistry<TrackedWeakMap>((entryToRemove) => {
  // Remove `entryToRemove` from array using the same method as Rust's `Vec::swap_remove`.
  //
  // * If the element we want to remove is the last one, just pop it off the array.
  // * Otherwise, pop last element, and overwrite the element we're removing with it.
  //
  // This avoids having to shuffle up all entries when an entry is removed.
  // Each element stores its index in `trackedWeakMaps` array inline, to avoid needing to search the whole array
  // to find the element we want to remove.
  // Cost of this whole operation is constant, regardless of how many `WeakMap`s are tracked.
  const lastEntry = trackedWeakMaps.pop();
  debugAssertIsNonNull(lastEntry, "`trackedWeakMaps` should not be empty");
  debugAssert(lastEntry.index === trackedWeakMaps.length, "Incorrect `index` for last entry");

  if (lastEntry !== entryToRemove) {
    const { index } = entryToRemove;
    debugAssert(trackedWeakMaps[index] === entryToRemove, "Entry is in wrong position");
    lastEntry.index = index;
    trackedWeakMaps[index] = lastEntry;
  }
});

let resetWeakMapsFn: () => void;

/**
 * Patched `WeakMap` class, which replaces native `WeakMap` class.
 *
 * This is a subclass of native `WeakMap` class which emulates how a `WeakMap` keyed by `context.sourceCode` would
 * behave if every file had a different value for `context.sourceCode`.
 *
 * It alters all methods to behave differently when `key` is `SOURCE_CODE` singleton.
 *
 * The value set for `SOURCE_CODE` is stored in `#value` field, and `#valueIsSet` is set to `true` when a value
 * has been set.
 *
 * The `WeakMap` is added to `trackedWeakMaps` array.
 *
 * When a file completes linting, `lintFile` calls `resetWeakMaps`, which loops through all `WeakMap`s which have
 * been used with `SOURCE_CODE` as key (`trackedWeakMaps` array), and resets their `#value` and `#valueIsSet` fields.
 * This means that the next time `map.get(SOURCE_CODE)` is called, it will return `undefined`.
 *
 * To avoid `trackedWeakMaps` array growing indefinitely and holding on to `WeakMap`s which are no longer referenced
 * anywhere else, `WeakMap`s are stored wrapped in `WeakRef`s, and removed from `trackedWeakMaps` array when
 * the `WeakMap`s are garbage collected, by the `FinalizationRegistry` defined above.
 *
 * When key is anything other than `SOURCE_CODE`, the `WeakMap` behaves normally.
 */
class PatchedWeakMap<Key extends WeakKey, Value> extends WeakMap<Key, Value> {
  // Value set for `SOURCE_CODE` key for this file.
  #value: Value | undefined;

  // `true` if a value has been set for `SOURCE_CODE` key for this file.
  #valueIsSet: boolean = false;

  // `true` if this `WeakMap` has been used with `SOURCE_CODE` as key, has been added to `trackedWeakMaps` array,
  // and registered with the `FinalizationRegistry`.
  #isTracked: boolean = false;

  constructor(entries?: Iterable<readonly [Key, Value]> | null) {
    // Pass no entries to `super()`. The native `WeakMap` constructor calls `this.set()` for each entry,
    // but private fields are not initialized until after `super()` returns, so `this.set()` would fail
    // if the entry's key is `SOURCE_CODE`. Instead, insert entries ourselves after construction.
    super();

    if (entries != null) {
      for (const [key, value] of entries) {
        if (key === SOURCE_CODE) {
          this.#setSourceCodeValue(value);
        } else {
          super.set(key, value);
        }
      }
    }
  }

  has(key: Key): boolean {
    if (key === SOURCE_CODE) return this.#valueIsSet;
    return super.has(key);
  }

  get(key: Key): Value | undefined {
    if (key === SOURCE_CODE) {
      return this.#valueIsSet === true ? this.#value : undefined;
    }

    return super.get(key);
  }

  set(key: Key, value: Value): this {
    if (key === SOURCE_CODE) {
      this.#setSourceCodeValue(value);
      return this;
    }

    return super.set(key, value);
  }

  delete(key: Key): boolean {
    if (key === SOURCE_CODE) {
      const valueWasSet = this.#valueIsSet;
      this.#value = undefined;
      this.#valueIsSet = false;
      return valueWasSet;
    }

    return super.delete(key);
  }

  // `getOrInsert` is not supported in NodeJS at present (March 2026), but presumably it will be in future.
  // So we want to add this method, to support plugins which rely on it in future.
  // But we have to implement it manually, rather than delegating to `super.getOrInsert`.
  getOrInsert(key: Key, value: Value): Value {
    if (key === SOURCE_CODE) {
      if (this.#valueIsSet === true) return this.#value!;

      this.#setSourceCodeValue(value);
      return value;
    }

    if (super.has(key)) return super.get(key)!;
    super.set(key, value);
    return value;
  }

  // `getOrInsertComputed` is not supported in NodeJS at present (March 2026), but presumably it will be in future.
  // So we want to add this method, to support plugins which rely on it in future.
  // But we have to implement it manually, rather than delegating to `super.getOrInsertComputed`.
  getOrInsertComputed(key: Key, getValue: (key: Key) => Value): Value {
    if (key === SOURCE_CODE) {
      if (this.#valueIsSet === true) return this.#value!;

      const value = getValue(key);
      this.#setSourceCodeValue(value);
      return value;
    }

    if (super.has(key)) return super.get(key)!;
    const value = getValue(key);
    super.set(key, value);
    return value;
  }

  /**
   * Set value for `SOURCE_CODE` key for this file.
   */
  #setSourceCodeValue(value: Value): void {
    // Set value
    this.#value = value;
    this.#valueIsSet = true;

    // If this `WeakMap` wasn't already added to `trackedWeakMaps` array, add it now, wrapped in a `WeakRef`.
    // Register it with the `FinalizationRegistry`, so the entry is removed from `trackedWeakMaps` array
    // when the `WeakMap` is garbage collected.
    if (this.#isTracked === false) {
      const tracked = {
        ref: new WeakRef(this),
        index: trackedWeakMaps.length,
      };
      trackedWeakMaps.push(tracked);
      registry.register(this, tracked);
      this.#isTracked = true;
    }
  }

  static {
    /**
     * Reset any `WeakMap`s which have been used with `SOURCE_CODE` as key.
     * These `WeakMap`s will now return `false` for `map.has(SOURCE_CODE)` and `undefined` for `map.get(SOURCE_CODE)`.
     * Called by `lintFile` after linting a file.
     * This function is defined inside the class, so it can access private fields.
     */
    resetWeakMapsFn = () => {
      const trackedWeakMapsLen = trackedWeakMaps.length;
      for (let i = 0; i < trackedWeakMapsLen; i++) {
        const weakMap = trackedWeakMaps[i].ref.deref();
        if (weakMap !== undefined) {
          weakMap.#value = undefined;
          weakMap.#valueIsSet = false;
        }
      }
    };
  }
}

// Set class name to `WeakMap` so the patch is invisible to users.
// Note: We don't set name with `static name = "WeakMap";` in class body,
// because that makes the `name` property writable and enumerable.
Object.defineProperty(PatchedWeakMap, "name", { value: "WeakMap" });

// Replace global `WeakMap` with patched version
globalThis.WeakMap = PatchedWeakMap;

// Export `resetWeakMaps` function for `lintFile` to use
export const resetWeakMaps: () => void = resetWeakMapsFn;
