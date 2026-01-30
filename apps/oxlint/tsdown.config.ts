import fs from "node:fs";
import { join as pathJoin, relative as pathRelative, dirname } from "node:path";
import { defineConfig } from "tsdown";
import { parseSync, Visitor } from "oxc-parser";

import type { Plugin } from "rolldown";

const { env } = process;
const isEnabled = (env: string | undefined) => env === "true" || env === "1";

// When run with `CONFORMANCE=true pnpm run build-js`, generate a conformance build with alterations to behavior.
// Also enables debug assertions.
// This is the build used in conformance tests.
const CONFORMANCE = isEnabled(env.CONFORMANCE);

// When run with `DEBUG=true pnpm run build-js`, generate a debug build with extra assertions.
// This is the build used in tests.
const DEBUG = CONFORMANCE || isEnabled(env.DEBUG);

const commonConfig = defineConfig({
  platform: "node",
  target: "node20",
  outDir: "dist",
  clean: true,
  unbundle: false,
  hash: false,
  fixedExtension: false,
  // tsdown warns about final bundled modules by `unbundle` + `external`.
  // But we know what we are doing, just suppress the warnings.
  inlineOnly: false,
});

const plugins = [createReplaceGlobalsPlugin()];
if (!DEBUG) plugins.push(createReplaceAssertsPlugin());

export default defineConfig([
  // Main build
  {
    ...commonConfig,
    entry: ["src-js/cli.ts", "src-js/index.ts", "src-js/plugin.ts", "src-js/rule-tester.ts"],
    format: "esm",
    external: [
      // External native bindings
      "./oxlint.*.node",
      "@oxlint/*",
    ],
    // At present only compress syntax.
    // Don't mangle identifiers or remove whitespace, so `dist` code remains somewhat readable.
    minify: {
      compress: { keepNames: { function: true, class: true } },
      mangle: false,
      codegen: { removeWhitespace: false },
    },
    dts: true,
    attw: { profile: "esm-only" },
    define: {
      DEBUG: DEBUG ? "true" : "false",
      CONFORMANCE: CONFORMANCE ? "true" : "false",
    },
    plugins,
    inputOptions: {
      // For `replaceAssertsPlugin` and `replaceGlobalsPlugin`
      experimental: { nativeMagicString: true },
    },
  },
  // TypeScript.
  // Bundled separately and lazy-loaded, as it's a lot of code.
  // Only used for tokens APIs.
  {
    ...commonConfig,
    entry: "src-js/plugins/typescript.cjs",
    format: "commonjs",
    // Minify as this bundle is just dependencies. We don't need to be able to debug it.
    // Minification halves the size of the bundle.
    minify: true,
  },
]);

/**
 * Create a plugin to remove imports of `debugAssert*` / `typeAssert*` functions from `src-js/utils/asserts.ts`,
 * and all their call sites.
 *
 * ```ts
 * // Original code
 * import { debugAssertIsNonNull } from '../utils/asserts.ts';
 * const foo = getFoo();
 * debugAssertIsNonNull(foo.bar);
 *
 * // After transform
 * const foo = getFoo();
 * ```
 *
 * This solves 2 problems:
 *
 * # 1. Minifier works chunk-by-chunk
 *
 * Minifier can already remove all calls to these functions as dead code, but only if the functions are defined
 * in the same file as the call sites.
 *
 * Problem is that `asserts.ts` is imported by files which end up in all output chunks.
 * So without this transform, TSDown creates a shared chunk for `asserts.ts`. Minifier works chunk-by-chunk,
 * so can't see that these functions are no-ops, and doesn't remove the function calls.
 *
 * # 2. Not entirely removed
 *
 * Even if minifier does remove all calls to these functions, it can't prove that expressions *inside* the calls
 * don't have side effects.
 *
 * In example above, it can't know if `foo` has a getter for `bar` property.
 * So it removes the call to `debugAssertIsNonNull`, but leaves behind the `foo.bar` expression.
 *
 * ```ts
 * const foo = getFoo();
 * foo.bar;
 * ```
 *
 * This plugin visits AST and removes all calls to `debugAssert*` / `typeAssert*` functions entirely,
 * *including* the expressions inside the calls.
 *
 * This makes these debug assertion functions act like `debug_assert!` in Rust.
 *
 * @returns Plugin
 */
function createReplaceAssertsPlugin(): Plugin {
  const ASSERTS_PATH = pathJoin(import.meta.dirname, "src-js/utils/asserts.ts");

  return {
    name: "replace-asserts",
    transform: {
      // Only process TS files in `src-js` directory
      filter: { id: /\/src-js\/.+\.ts$/ },

      async handler(code, path, meta) {
        const magicString = meta.magicString!;
        const { program, errors } = parseSync(path, code);
        if (errors.length !== 0) throw new Error(`Failed to parse ${path}: ${errors[0].message}`);

        // Gather names of assertion functions imported from `asserts.ts`.
        // Also gather all identifiers used in the `import` statements, so can avoid erroring on them in visitor below.
        const assertFnNames: Set<string> = new Set(),
          idents = new Set();
        for (const stmt of program.body) {
          if (stmt.type !== "ImportDeclaration") continue;

          // Check if import is from `utils/asserts.ts`.
          // `endsWith` check is just a shortcut to avoid resolving the specifier to a full path for most imports.
          const source = stmt.source.value;
          if (!source.endsWith("/asserts.ts") && !source.endsWith("/asserts.js")) continue;
          // oxlint-disable-next-line no-await-in-loop
          const importedId = await this.resolve(source, path);
          if (importedId === null || importedId.id !== ASSERTS_PATH) continue;

          // Remove `import` statement
          for (const specifier of stmt.specifiers) {
            if (specifier.type !== "ImportSpecifier") {
              throw new Error(`Only use named imports when importing from \`asserts.ts\`: ${path}`);
            }
            idents.add(specifier.local);
            if (specifier.imported.type === "Identifier") idents.add(specifier.imported);
            assertFnNames.add(specifier.local.name);
          }
          magicString.remove(stmt.start, stmt.end);
        }

        if (assertFnNames.size === 0) return;

        // Visit AST and remove all calls to assertion functions
        const visitor = new Visitor({
          // Replace `debugAssert(...)` calls with `null`. Minifier will remove the `null`.
          CallExpression(node) {
            const { callee } = node;
            if (callee.type !== "Identifier") return;
            if (assertFnNames.has(callee.name)) {
              idents.add(callee);
              magicString.overwrite(node.start, node.end, "null");
            }
          },
          // Error if assertion functions are used in any other way. We lack logic to deal with that.
          Identifier(node) {
            const { name } = node;
            if (assertFnNames.has(name) && !idents.has(node)) {
              throw new Error(
                `Do not use \`${name}\` imported from \`asserts.ts\` except in function calls: ${path}`,
              );
            }
          },
        });
        visitor.visit(program);

        return { code: magicString };
      },
    },
  };
}

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

/**
 * Create a plugin to replace usage of properties of globals with global vars defined in `utils/globals.ts`.
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
function createReplaceGlobalsPlugin(): Plugin {
  // Path to file which exports global vars
  const GLOBALS_PATH = pathJoin(import.meta.dirname, "src-js/utils/globals.ts");

  // Parse the file to get the list of global vars it exports
  const availableGlobals = getAvailableGlobals(GLOBALS_PATH);

  return {
    name: "replace-globals",
    transform: {
      // Only process TS files in `src-js` directory
      filter: { id: /\/src-js\/.+\.ts$/ },

      async handler(code, path, meta) {
        const magicString = meta.magicString!;
        const { program, errors } = parseSync(path, code);
        if (errors.length !== 0) throw new Error(`Failed to parse ${path}: ${errors[0].message}`);

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

            const mapping = availableGlobals.get(globalName);
            if (!mapping) {
              missingGlobalVars.add(`\`${object.name}.${propName}\``);
              return;
            }

            const varName = mapping.get(propName);
            if (!varName) {
              missingGlobalVars.add(`\`${object.name}.${propName}\``);
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
        const relativePath = pathRelative(dirname(path), GLOBALS_PATH);
        const importStmt = `import { ${[...varNames].join(", ")} } from ${JSON.stringify(relativePath)};\n`;

        magicString.prepend(importStmt);

        return { code: magicString };
      },
    },
  };
}

/**
 * Parse `utils/globals.ts` and return a list of globals and global vars it exports.
 * @param path - Path to `utils/globals.ts`
 * @returns Mapping from global name (e.g. `Object`) to mapping of properties of that global to var names
 *   (e.g. `hasOwn` -> `ObjectHasOwn`).
 */
function getAvailableGlobals(path: string): Map<string, Map<string, string>> {
  const code = fs.readFileSync(path, "utf8");
  const { program, errors } = parseSync(path, code);
  if (errors.length !== 0) throw new Error(`Failed to parse ${path}: ${errors[0].message}`);

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
