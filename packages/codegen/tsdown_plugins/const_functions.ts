import { parseSync } from "oxc-parser";

import type { Plugin } from "rolldown";

/**
 * Plugin to rewrite the printer's top-level function declarations as `const` bindings.
 *
 * ```ts
 * // Original code
 * function printIf(node: ESTree.IfStatement, state: State): void {
 *   // ...
 * }
 *
 * // After transform
 * const printIf = (node: ESTree.IfStatement, state: State): void => {
 *   // ...
 * };
 * ```
 *
 * A function declaration creates a binding which could be assigned to later, so a call has to read
 * the current value out of the module's scope. A `const` binding cannot, so V8 can treat it
 * as a constant and call it directly. Measured on all 7 benchmark fixtures, it is worth ~3%.
 *
 * It really is the `const`, and not the arrow function or the smaller output it allows - the same rewrite
 * with `let` measures ~4% slower than this one, and slower than leaving the declarations alone.
 * Do not relax it to `let`.
 *
 * The source keeps its declarations - they hoist, they read better, and they are what the rest of
 * the codebase looks like. This is why the rewrite is a build step rather than how printer is written.
 */
const plugin: Plugin = {
  name: "const-functions",
  transform: {
    // Only process TS files in `src-js/print` directory
    filter: { id: /\/src-js\/print\/.+\.ts$/ },

    handler(code, path, meta) {
      const magicString = meta.magicString!;
      const { program, errors } = parseSync(path, code);
      if (errors.length !== 0) throw new Error(`Failed to parse ${path}: ${errors[0].message}`);

      for (const statement of program.body) {
        let fn;
        if (statement.type === "FunctionDeclaration") {
          fn = statement;
        } else if (
          statement.type === "ExportNamedDeclaration" &&
          statement.declaration?.type === "FunctionDeclaration"
        ) {
          fn = statement.declaration;
        } else {
          continue;
        }

        if (fn.async || fn.generator) continue;

        // `function name` -> `const name =`, `{` of the body -> `=> {`, and `}` -> `};`
        magicString.overwrite(fn.start, fn.id!.end, `const ${fn.id!.name} =`);
        magicString.appendLeft(fn.body!.start, "=> ");
        magicString.appendRight(fn.end, ";");
      }

      return { code: magicString };
    },
  },
};

export default plugin;
