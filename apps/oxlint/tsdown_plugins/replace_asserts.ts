import { join as pathJoin } from "node:path";
import { Visitor } from "oxc-parser";
import { parse } from "./utils.ts";

import type { Plugin } from "rolldown";

// Path to file which defines assertion functions
const ASSERTS_PATH = pathJoin(import.meta.dirname, "../src-js/utils/asserts.ts");

/**
 * Plugin to remove imports of `debugAssert*` / `typeAssert*` functions from
 * `src-js/utils/asserts.ts`, and all their call sites.
 *
 * ```ts
 * // Original code
 * import { debugAssertIsNonNull } from "../utils/asserts.ts";
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
 * Minifier can already remove all calls to these functions as dead code, but only if the functions
 * are defined in the same file as the call sites.
 *
 * Problem is that `asserts.ts` is imported by files which end up in all output chunks. So without
 * this transform, TSDown creates a shared chunk for `asserts.ts`. Minifier works chunk-by-chunk, so
 * can't see that these functions are no-ops, and doesn't remove the function calls.
 *
 * # 2. Not entirely removed
 *
 * Even if minifier does remove all calls to these functions, it can't prove that expressions
 * _inside_ the calls don't have side effects.
 *
 * In example above, it can't know if `foo` has a getter for `bar` property.
 * So it removes the call to `debugAssertIsNonNull`, but leaves behind the `foo.bar` expression.
 *
 * ```ts
 * const foo = getFoo();
 * foo.bar;
 * ```
 *
 * This plugin visits AST and removes all calls to `debugAssert*` / `typeAssert*` functions
 * entirely, _including_ the expressions inside the calls.
 *
 * This makes these debug assertion functions act like `debug_assert!` in Rust.
 */
const plugin: Plugin = {
  name: "replace-asserts",
  transform: {
    // Only process TS files in `src-js` directory
    filter: { id: /\/src-js\/.+(?<!\.d)\.ts$/ },

    async handler(code, path, meta) {
      const magicString = meta.magicString!;
      const program = parse(path, code);

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

export default plugin;
