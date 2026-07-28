import { Visitor } from "oxc-parser";
import { parse } from "./utils.ts";

import type { Expression, MemberExpression } from "@oxc-project/types";
import type { Plugin } from "rolldown";

// Globals.
// prettier-ignore
const GLOBALS = new Set([
  "Object", "Array", "Math", "JSON", "Reflect", "Symbol", "Function", "Number", "Boolean", "String", "Date", "Promise",
  "RegExp", "BigInt", "Map", "Set", "Error", "AggregateError", "EvalError", "RangeError", "ReferenceError",
  "SyntaxError", "TypeError", "URIError", "Buffer", "ArrayBuffer", "SharedArrayBuffer", "Atomics", "Uint8Array",
  "Int8Array", "Uint16Array", "Int16Array", "Uint32Array", "Int32Array", "BigUint64Array", "BigInt64Array",
  "Uint8ClampedArray", "Float32Array", "Float64Array", "Float16Array", "DataView", "WebAssembly", "Iterator",
  "WeakMap", "WeakSet", "Proxy", "FinalizationRegistry", "WeakRef", "URL", "URLSearchParams", "TextEncoder",
  "TextDecoder", "BroadcastChannel", "MessageChannel", "MessagePort", "Blob", "File"
]);

// Global properties which cannot be converted to top-level vars when used as callee of a `CallExpression`,
// because they're methods which use `this`.
// e.g. `const r = Promise.resolve; r(1);` throws "TypeError: PromiseResolve called on non-object".
const SKIP_WHEN_CALLEE = new Set(["Promise.resolve", "Promise.allSettled"]);

// `Function.prototype` methods which use the function they're called on as `this`, so break when detached.
// A trailing one is left attached rather than folded into the var name, e.g. `Array.prototype.slice.call(x)`
// becomes `ArrayPrototypeSlice.call(x)`, not `ArrayPrototypeSliceCall(x)`.
const BOUND_METHODS = new Set(["call", "apply", "bind"]);

/**
 * Plugin to replace usage of properties of globals with top-level vars.
 *
 * ```ts
 * // Original code
 * function f(obj) {
 *   return Object.keys(obj);
 * }
 *
 * // After transform
 * const ObjectKeys = Object.keys;
 * function f(obj) {
 *   return ObjectKeys(obj);
 * }
 * ```
 *
 * This has several advantages:
 *
 * 1. Slightly more performant, due to reduced book-keeping to check if globals are mutated.
 * 2. Minifies better.
 * 3. Code is resilient in face of user code (plugins) re-assigning global methods
 *    e.g. `Object.keys = function myWeirdFunction() {}`.
 *
 * Accesses on globals listed in `GLOBALS` are replaced, including chained accesses.
 * Each level of a chain becomes its own var, referencing the var for the level above.
 * e.g. `Object.prototype.toString` becomes `ObjectPrototypeToString`, declared as:
 *
 * ```ts
 * const ObjectPrototype = Object.prototype;
 * const ObjectPrototypeToString = ObjectPrototype.toString;
 * ```
 *
 * All properties are assumed to work when detached from their global.
 * Any methods which rely on `this` (e.g. `Promise.resolve`, or `Uint8Array.from`) must be added to `SKIP_WHEN_CALLEE`,
 * otherwise the generated `const` will throw when called.
 *
 * A trailing `.call` / `.apply` / `.bind` is left attached rather than detached into a var, since those rely
 * on their receiver, e.g. `Array.prototype.slice.call(arguments)` becomes `ArrayPrototypeSlice.call(arguments)`.
 *
 * This replacement is simplistic. It does not do scope analysis, so care must be taken not to break it.
 * Patterns which will break code:
 *
 * ```ts
 * // Transform adds `const ObjectKeys = Object.keys;` which clashes with existing var.
 * let ObjectKeys = 123; Object.keys({x: 1});
 *
 * // Transform replaces `Object.keys` with `ObjectKeys` but that var is shadowed by function param
 * function f(obj, ObjectKeys) {
 *   return Object.keys(obj);
 * }
 *
 * // Transform wrongly identifies `Object.keys` as a global property access
 * function g(Object) {
 *   return Object.keys;
 * }
 * ```
 *
 * These patterns are all pretty weird, so problems are unlikely in practice.
 */
const plugin: Plugin = {
  name: "replace-globals",
  transform: {
    // Only process JS and TS files in `src-js` directory
    filter: { id: /\/src-js\/.+(?<!\.d)\.[jt]s$/ },

    handler(code, path, meta) {
      const magicString = meta.magicString!;
      const program = parse(path, code);

      // Map of global access (e.g. `Object.keys`) to the var it's replaced with (e.g. `ObjectKeys`).
      // Dedupes declarations, and lets each level of a chain reference the var for the level above.
      const accesses = new Map<string, string>();

      // Inner member expressions of a chain (e.g. `Object.prototype` within `Object.prototype.toString`).
      // They're subsumed by the outer access, so must not be replaced on their own.
      const consumed = new Set<MemberExpression>();

      // Declarations to prepend, e.g. `const ObjectKeys = Object.keys;`
      let declarations = "";

      // Callee of most recently visited `CallExpression` or `TaggedTemplateExpression` - both pass `this`
      // into the callee. Wrappers around it are unwrapped, so e.g. `(Promise.resolve)(x)` still matches.
      let lastCallee: Expression | null = null;

      const visitor = new Visitor({
        CallExpression(node) {
          lastCallee = unwrapExpression(node.callee);
        },

        TaggedTemplateExpression(node) {
          lastCallee = unwrapExpression(node.tag);
        },

        MemberExpression(node) {
          // Skip if already handled as an inner part of an enclosing chain
          if (consumed.has(node)) return;

          // A trailing `.call` / `.apply` / `.bind` is bound to its receiver, so leave it attached and convert
          // the chain it's called on instead. e.g. `Array.prototype.slice.call` -> `ArrayPrototypeSlice.call`.
          if (node.computed || node.property.type !== "Identifier") return;
          if (BOUND_METHODS.has(node.property.name)) {
            if (node.object.type !== "MemberExpression") return; // Nothing to convert e.g. `x.call`
            node = node.object;
            consumed.add(node);
          }

          // Walk down `node`'s chain to its root, recording prop names (leaf-first).
          // e.g. `Object.prototype.toString` -> root `Object`, props `["toString", "prototype"]`.
          const propNames: string[] = [];
          let object: Expression = node;
          do {
            // Bail on computed (`obj[x]`) or private (`obj.#x`) access.
            // Any inner static chain (e.g. `Object.prototype` in `Object.prototype[x].y`)
            // is converted when visited as its own node.
            if (object.computed || object.property.type !== "Identifier") return;
            propNames.push(object.property.name);
            if (object !== node) consumed.add(object);
            object = object.object;
          } while (object.type === "MemberExpression");

          // Bail if root of chain is not a global e.g. `foo().x` or `notAGlobal.x`
          if (object.type !== "Identifier") return;
          const globalName = object.name;
          if (!GLOBALS.has(globalName)) return;

          propNames.reverse();

          // Bail if needs to be skipped because is used as callee and method requires `this`
          // e.g. `Promise.resolve(x)`
          if (lastCallee === node && SKIP_WHEN_CALLEE.has(`${globalName}.${propNames.join(".")}`)) {
            return;
          }

          // Create a var for each level from root to leaf.
          // Each references the var above e.g. `const ObjectPrototype = Object.prototype;`,
          // then `const ObjectPrototypeToString = ObjectPrototype.toString;`.
          let varName = globalName,
            access = globalName;
          for (const propName of propNames) {
            access += `.${propName}`;

            const existingVarName = accesses.get(access);
            if (existingVarName === undefined) {
              const init = `${varName}.${propName}`;
              varName += `${propName[0].toUpperCase()}${propName.slice(1)}`;
              accesses.set(access, varName);
              declarations += `const ${varName} = ${init};\n`;
            } else {
              varName = existingVarName;
            }
          }

          // Replace the access
          magicString.overwrite(node.start, node.end, varName);
        },
      });
      visitor.visit(program);

      if (declarations === "") return;

      magicString.prepend(declarations);

      return { code: magicString };
    },
  },
};

export default plugin;

/**
 * Unwrap transparent wrappers around an `Expression` - parentheses and TS type assertions.
 * These still pass `this` into a callee, e.g. the callee of `(Promise.resolve)(x)`,
 * `Promise.resolve!(x)`, or `(Promise.resolve as T)(x)`.
 * @param expr - Expression to unwrap
 * @returns `expr` with any enclosing parentheses and TS type assertions removed
 */
function unwrapExpression(expr: Expression): Expression {
  while (
    expr.type === "ParenthesizedExpression" ||
    expr.type === "TSNonNullExpression" ||
    expr.type === "TSAsExpression" ||
    expr.type === "TSSatisfiesExpression"
  ) {
    expr = expr.expression;
  }
  return expr;
}
