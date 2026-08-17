import { join as pathJoin } from "node:path";
import { parseSync, Visitor } from "oxc-parser";

import type { Plugin } from "rolldown";

// Path to file which defines assertion functions
const ASSERTS_PATH = pathJoin(import.meta.dirname, "../src-js/asserts.ts");

/**
 * Plugin to remove imports of `typeAssertIs` from `src-js/asserts.ts`, and all its call sites.
 *
 * ```ts
 * // Original code
 * import { typeAssertIs } from "./asserts.ts";
 * typeAssertIs<Foo>(node.value);
 * print(node.value);
 *
 * // After transform
 * print(node.value);
 * ```
 *
 * The minifier can already remove the calls on its own, as the function body is empty and lands
 * in the same chunk. But it can't prove that the expressions *inside* the calls have no side effects -
 * `node.value` could be a getter - so it leaves those behind as bare expression statements.
 *
 * This plugin removes the calls entirely, expressions included, which makes `typeAssertIs`
 * cost nothing at all. Adapted from `apps/oxlint/tsdown_plugins/replace_asserts.ts`.
 */
const plugin: Plugin = {
  name: "remove-asserts",
  transform: {
    // Only process TS files in `src-js` directory
    filter: { id: /\/src-js\/.+(?<!\.d)\.ts$/ },

    async handler(code, path, meta) {
      const magicString = meta.magicString!;
      const { program, errors } = parseSync(path, code);
      if (errors.length !== 0) throw new Error(`Failed to parse ${path}: ${errors[0].message}`);

      // Gather names of assertion functions imported from `asserts.ts`.
      // Also gather all identifiers used in the `import` statements, so can avoid erroring on them
      // in visitor below.
      const assertFnNames = new Set<string>(),
        idents = new Set();
      for (const stmt of program.body) {
        if (stmt.type !== "ImportDeclaration") continue;

        // Check if import is from `asserts.ts`.
        // `endsWith` check is just a shortcut to avoid resolving the specifier to a full path for most imports.
        const source = stmt.source.value;
        if (!source.endsWith("/asserts.ts")) continue;

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
        // Replace `typeAssertIs(...)` calls with `null`. Minifier will remove the `null`.
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
