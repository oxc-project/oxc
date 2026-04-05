import fs from "node:fs";
import { join as pathJoin, relative as pathRelative, dirname } from "node:path";
import { Visitor } from "oxc-parser";
import { parse } from "./utils.ts";

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

// Global properties which cannot be converted to top-level vars, because they're methods which use `this`.
// e.g. `const r = Promise.resolve; r(1);` throws "TypeError: PromiseResolve called on non-object".
const SKIP_GLOBALS = new Set(["Promise.resolve", "Promise.allSettled"]);

// Path to file which exports global vars
const GLOBALS_PATH = pathJoin(import.meta.dirname, "../src-js/utils/globals.ts");

// Parse the file to get the list of global vars it exports
const availableGlobals = getAvailableGlobals(GLOBALS_PATH);

/**
 * Plugin to replace usage of properties of globals with global vars defined in `utils/globals.ts`.
 *
 * This is more performant, due to reduced property lookups, and minifies better.
 *
 * ```ts
 * // Original code
 * const keys = Object.keys(obj);
 *
 * // After transform
 * import { ObjectKeys } from "../utils/globals.ts";
 * const keys = ObjectKeys(obj);
 * ```
 *
 * If TSDown produces any errors about missing imports, likely you need to add the missing global(s)
 * to `utils/globals.ts`.
 */
const plugin: Plugin = {
  name: "replace-globals",
  transform: {
    // Only process JS and TS files in `src-js` directory
    filter: { id: /\/src-js\/.+(?<!\.d)\.[jt]s$/ },

    async handler(code, path, meta) {
      const magicString = meta.magicString!;
      const program = parse(path, code);

      // Visit AST and replace all references to globals with top-level vars
      const varNames = new Set<string>(),
        visitedMemberExpressions = new Set(),
        missingGlobalVars = new Set<string>();

      const visitor = new Visitor({
        MemberExpression(node) {
          // Skip nested `MemberExpression`s e.g. `Object.prototype` in `Object.prototype.toString`
          if (visitedMemberExpressions.has(node)) return;

          // Exit if computed (`obj[prop]`) or private property (`obj.#prop`).
          let { object, property } = node;
          if (node.computed || property.type !== "Identifier") return;

          // Gather all properties in reverse order.
          // e.g. `Object.prototype.toString` -> `propNames = ["toString", "prototype"]`.
          const propNames: string[] = [property.name];
          while (true) {
            // If `object` is an identifier, `node` is a member expression of form `a.b`, `a.b.c`, etc.
            if (object.type === "Identifier") break;

            // If `object` is not a member expression, exit e.g. `foo().x`
            if (object.type !== "MemberExpression") return;

            // We can't handle deep nesting yet
            // oxlint-disable-next-line no-constant-condition
            if (1) return;

            // Avoid processing the nested member expression again when it's visited later
            visitedMemberExpressions.add(object);

            // Exit if computed (`obj[prop]`) or private property (`obj.#prop`).
            property = object.property;
            if (object.computed || property.type !== "Identifier") return;

            // `node` of form `<SOMETHING>.a.b` or `<SOMETHING>.a.b.c`.
            // Loop round to process the `<SOMETHING>` part.
            propNames.push(property.name);

            object = object.object;
          }

          // Found a member expression of form `obj.a`, or `obj.a.b`, `obj.a.b.c`, etc.
          // Exit if `obj` is not a global.
          const globalName = object.name;
          if (!GLOBALS.has(globalName)) return;

          const propName = propNames.reverse().join(".");

          const fullName = `${object.name}.${propName}`;
          if (SKIP_GLOBALS.has(fullName)) return;

          const mapping = availableGlobals.get(globalName);
          if (!mapping) {
            missingGlobalVars.add(`\`${fullName}\``);
            return;
          }

          const varName = mapping.get(propName);
          if (!varName) {
            missingGlobalVars.add(`\`${fullName}\``);
            return;
          }

          // Add var name (e.g. `ObjectHasOwn`) to set of vars to import
          varNames.add(varName);

          // Replace `Object.hasOwn` with `ObjectHasOwn`
          magicString.overwrite(node.start, node.end, varName);
        },
      });
      visitor.visit(program);

      // Log any globals that were not converted because `utils/globals.ts` has no export for them
      if (missingGlobalVars.size > 0) {
        // oxlint-disable-next-line no-console
        console.error(
          "--------------------------------------------------------------------------------\n" +
            `WARNING: Unable to convert ${[...missingGlobalVars].join(" or ")} to global vars.\n` +
            `Add exports to \`utils/globals.ts\` for them.\n` +
            "--------------------------------------------------------------------------------",
        );
      }

      if (varNames.size === 0) return;

      // Some globals were found. Import them from `utils/globals.ts`.
      let relativePath = pathRelative(dirname(path), GLOBALS_PATH);
      relativePath = relativePath.replace(/\\/g, "/");
      relativePath = relativePath.startsWith(".") ? relativePath : `./${relativePath}`;
      const importStmt = `import { ${[...varNames].join(", ")} } from ${JSON.stringify(relativePath)};\n`;

      magicString.prepend(importStmt);

      return { code: magicString };
    },
  },
};

export default plugin;

/**
 * Parse `utils/globals.ts` and return a list of globals and global vars it exports.
 * @param path - Path to `utils/globals.ts`
 * @returns Mapping from global name (e.g. `Object`) to mapping of properties of that global to var names
 *   (e.g. `hasOwn` -> `ObjectHasOwn`).
 */
function getAvailableGlobals(path: string): Map<string, Map<string, string>> {
  const code = fs.readFileSync(path, "utf8");
  const program = parse(path, code);

  const globals = new Map<string, Map<string, string>>();

  const visitor = new Visitor({
    ExportNamedDeclaration(node) {
      const { declaration } = node;
      if (declaration == null || declaration.type !== "VariableDeclaration") return;
      const declarator = declaration.declarations[0];
      if (!declarator) return;
      const { init } = declarator;
      if (!init || init.type !== "Identifier") return;

      const obj = declarator.id;
      if (obj.type !== "ObjectPattern") return;

      const globalName = init.name;
      let mapping = globals.get(globalName);
      if (!mapping) {
        mapping = new Map();
        globals.set(globalName, mapping);
      }

      for (const prop of obj.properties) {
        if (prop.type !== "Property" || prop.method || prop.computed) continue;

        const { key, value } = prop;
        if (key.type !== "Identifier" || value.type !== "Identifier") continue;

        mapping.set(key.name, value.name);
      }
    },
  });
  visitor.visit(program);

  return globals;
}
